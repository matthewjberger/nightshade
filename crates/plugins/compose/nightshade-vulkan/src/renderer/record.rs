use crate::renderer::context::{Context, FRAMES_IN_FLIGHT};
use crate::renderer::glyph_atlas::GlyphAtlas;
use crate::renderer::pass::{Pass, PassRecord, ScopeKind};
use crate::renderer::resource::ResourceStates;
use ash::vk;
use nightshade_render::data::config::RenderView;

pub const WORTH_SPLITTING: usize = 3;

const MOST_LANES: usize = 4;

pub struct Recorders {
    lanes: Vec<Lane>,
}

struct Lane {
    pools: [vk::CommandPool; FRAMES_IN_FLIGHT],
    buffers: [Vec<vk::CommandBuffer>; FRAMES_IN_FLIGHT],
    taken: [usize; FRAMES_IN_FLIGHT],
}

pub fn create_recorders(context: &Context) -> Result<Recorders, Box<dyn std::error::Error>> {
    let mut lanes = Vec::with_capacity(MOST_LANES);
    for _ in 0..lane_count() {
        let mut pools = [vk::CommandPool::null(); FRAMES_IN_FLIGHT];
        for pool in pools.iter_mut() {
            let create_info = vk::CommandPoolCreateInfo::default()
                .queue_family_index(context.families.graphics)
                .flags(vk::CommandPoolCreateFlags::TRANSIENT);
            *pool = unsafe { context.device.create_command_pool(&create_info, None)? };
        }
        lanes.push(Lane {
            pools,
            buffers: std::array::from_fn(|_| Vec::new()),
            taken: [0; FRAMES_IN_FLIGHT],
        });
    }
    Ok(Recorders { lanes })
}

fn lane_count() -> usize {
    std::thread::available_parallelism()
        .map(|found| found.get())
        .unwrap_or(1)
        .clamp(1, MOST_LANES)
}

pub fn destroy_recorders(context: &Context, recorders: &mut Recorders) {
    for lane in recorders.lanes.drain(..) {
        for pool in lane.pools {
            unsafe { context.device.destroy_command_pool(pool, None) };
        }
    }
}

pub fn lanes(recorders: &Recorders) -> usize {
    recorders.lanes.len()
}

pub fn reset(
    context: &Context,
    recorders: &mut Recorders,
    frame_index: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    for lane in recorders.lanes.iter_mut() {
        unsafe {
            context
                .device
                .reset_command_pool(lane.pools[frame_index], vk::CommandPoolResetFlags::empty())?
        };
        lane.taken[frame_index] = 0;
    }
    Ok(())
}

pub fn take(
    context: &Context,
    recorders: &mut Recorders,
    lane: usize,
    frame_index: usize,
    wanted: usize,
) -> Result<Vec<vk::CommandBuffer>, Box<dyn std::error::Error>> {
    let Some(lane) = recorders.lanes.get_mut(lane) else {
        return Ok(Vec::new());
    };
    let from = lane.taken[frame_index];
    let needed = from + wanted;
    if lane.buffers[frame_index].len() < needed {
        let more = needed - lane.buffers[frame_index].len();
        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(lane.pools[frame_index])
            .level(vk::CommandBufferLevel::SECONDARY)
            .command_buffer_count(more as u32);
        let made = unsafe { context.device.allocate_command_buffers(&allocate_info)? };
        lane.buffers[frame_index].extend(made);
    }
    lane.taken[frame_index] = needed;
    Ok(lane.buffers[frame_index][from..needed].to_vec())
}

#[derive(Clone)]
pub struct Inherited {
    pub color_formats: Vec<vk::Format>,
    pub depth_format: vk::Format,
}

pub fn begin_secondary(
    context: &Context,
    command_buffer: vk::CommandBuffer,
    inherited: &Inherited,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rendering = vk::CommandBufferInheritanceRenderingInfo::default()
        .color_attachment_formats(&inherited.color_formats)
        .depth_attachment_format(inherited.depth_format)
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);
    let inheritance = vk::CommandBufferInheritanceInfo::default().push_next(&mut rendering);
    let begin_info = vk::CommandBufferBeginInfo::default()
        .flags(vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE)
        .inheritance_info(&inheritance);
    unsafe {
        context
            .device
            .begin_command_buffer(command_buffer, &begin_info)?
    };
    Ok(())
}

pub fn end_secondary(
    context: &Context,
    command_buffer: vk::CommandBuffer,
) -> Result<(), Box<dyn std::error::Error>> {
    unsafe { context.device.end_command_buffer(command_buffer)? };
    Ok(())
}

pub struct Split<'a> {
    pub context: &'a Context,
    pub frame_index: usize,
    pub extent: vk::Extent2D,
    pub view: Option<&'a RenderView>,
    pub scope: ScopeKind,
    pub glyphs: &'a GlyphAtlas,
    pub air: &'a crate::renderer::scene::fog::Air,
    pub probes: &'a crate::renderer::scene::probes::Probes,
    pub inherited: Inherited,
    pub viewport: Option<[f32; 4]>,
}

pub fn record_split(
    split: &Split<'_>,
    passes: &mut [&mut Box<dyn Pass>],
    recorders: &mut Recorders,
    timings: &mut crate::renderer::timing::Timings,
) -> Vec<vk::CommandBuffer> {
    use rayon::prelude::*;

    let context = split.context;
    let lanes = lanes(recorders).min(passes.len()).max(1);
    let each = passes.len().div_ceil(lanes);

    let mut plans: Vec<Vec<(vk::CommandBuffer, Option<u32>)>> = Vec::with_capacity(lanes);
    for lane in 0..lanes {
        let from = lane * each;
        let until = ((lane + 1) * each).min(passes.len());
        if from >= until {
            plans.push(Vec::new());
            continue;
        }
        let buffers = match take(context, recorders, lane, split.frame_index, until - from) {
            Ok(buffers) => buffers,
            Err(error) => {
                tracing::error!("a lane of the frame was not given buffers: {error}");
                plans.push(Vec::new());
                continue;
            }
        };
        let mut plan = Vec::with_capacity(until - from);
        for (slot, buffer) in buffers.into_iter().enumerate() {
            let name = passes[from + slot].name();
            let reserved = crate::renderer::timing::reserve(
                timings,
                split.frame_index,
                crate::renderer::timing::Band::Drawing,
                name,
            );
            plan.push((buffer, reserved));
        }
        plans.push(plan);
    }

    let stamped: &crate::renderer::timing::Timings = timings;
    passes
        .par_chunks_mut(each)
        .zip(plans.par_iter())
        .for_each(|(chunk, plan)| {
            for (pass, (buffer, reserved)) in chunk.iter_mut().zip(plan.iter()) {
                if let Err(error) = begin_secondary(context, *buffer, &split.inherited) {
                    tracing::error!("a pass was not opened for recording: {error}");
                    continue;
                }
                match split.viewport {
                    Some(rect) => set_viewport_rect(&context.device, *buffer, rect),
                    None => set_viewport(&context.device, *buffer, split.extent),
                }
                crate::renderer::timing::open_reserved(
                    stamped,
                    context,
                    *buffer,
                    split.frame_index,
                    *reserved,
                );
                let mut apart = ResourceStates::default();
                let mut record = PassRecord {
                    context,
                    command_buffer: *buffer,
                    extent: split.extent,
                    frame_index: split.frame_index,
                    view: split.view,
                    scope: split.scope,
                    states: &mut apart,
                    glyphs: split.glyphs,
                    air: split.air,
                    probes: split.probes,
                };
                pass.draw(&mut record);
                crate::renderer::timing::close(
                    stamped,
                    context,
                    *buffer,
                    split.frame_index,
                    *reserved,
                );
                if let Err(error) = end_secondary(context, *buffer) {
                    tracing::error!("a pass was not closed after recording: {error}");
                }
            }
        });

    plans
        .into_iter()
        .flatten()
        .map(|(buffer, _)| buffer)
        .collect()
}

pub fn set_viewport(device: &ash::Device, command_buffer: vk::CommandBuffer, extent: vk::Extent2D) {
    let viewport = vk::Viewport {
        x: 0.0,
        y: extent.height as f32,
        width: extent.width as f32,
        height: -(extent.height as f32),
        min_depth: 0.0,
        max_depth: 1.0,
    };
    let scissor = vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent,
    };
    unsafe {
        device.cmd_set_viewport(command_buffer, 0, std::slice::from_ref(&viewport));
        device.cmd_set_scissor(command_buffer, 0, std::slice::from_ref(&scissor));
    }
}

pub fn set_viewport_rect(device: &ash::Device, command_buffer: vk::CommandBuffer, rect: [f32; 4]) {
    let viewport = vk::Viewport {
        x: rect[0],
        y: rect[1] + rect[3],
        width: rect[2],
        height: -rect[3],
        min_depth: 0.0,
        max_depth: 1.0,
    };
    let scissor = vk::Rect2D {
        offset: vk::Offset2D {
            x: rect[0] as i32,
            y: rect[1] as i32,
        },
        extent: vk::Extent2D {
            width: rect[2] as u32,
            height: rect[3] as u32,
        },
    };
    unsafe {
        device.cmd_set_viewport(command_buffer, 0, std::slice::from_ref(&viewport));
        device.cmd_set_scissor(command_buffer, 0, std::slice::from_ref(&scissor));
    }
}
