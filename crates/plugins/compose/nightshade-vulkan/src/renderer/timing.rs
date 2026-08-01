use crate::renderer::context::{Context, FRAMES_IN_FLIGHT};
use ash::vk;

const STAMPS_PER_FRAME: u32 = 1024;

const COMPUTE_STAMPS: u32 = STAMPS_PER_FRAME / 4;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Band {
    Compute,
    Drawing,
}

struct Stamped {
    pass: &'static str,
    first: u32,
}

#[derive(Default)]
struct FrameStamps {
    stamped: Vec<Stamped>,
    compute_next: u32,
    drawing_next: u32,
}

#[derive(Clone, Debug)]
pub struct PassTime {
    pub name: &'static str,
    pub milliseconds: f32,
}

pub struct Timings {
    pool: vk::QueryPool,
    period: f32,
    graphics_bits: u32,
    compute_bits: u32,
    frames: Vec<FrameStamps>,
    pub passes: Vec<PassTime>,
    pub frame_milliseconds: f32,
}

pub fn create(context: &Context) -> Result<Timings, Box<dyn std::error::Error>> {
    let properties = unsafe {
        context
            .instance
            .get_physical_device_properties(context.physical_device)
    };
    let families = unsafe {
        context
            .instance
            .get_physical_device_queue_family_properties(context.physical_device)
    };
    let bits_of = |family: u32| {
        families
            .get(family as usize)
            .map(|family| family.timestamp_valid_bits)
            .unwrap_or(0)
    };
    let graphics_bits = bits_of(context.graphics.family);
    let compute_bits = context
        .compute
        .as_ref()
        .map(|lane| bits_of(lane.family))
        .unwrap_or(graphics_bits);
    let create_info = vk::QueryPoolCreateInfo::default()
        .query_type(vk::QueryType::TIMESTAMP)
        .query_count(STAMPS_PER_FRAME * FRAMES_IN_FLIGHT as u32);
    let pool = unsafe { context.device.create_query_pool(&create_info, None)? };
    Ok(Timings {
        pool,
        period: properties.limits.timestamp_period,
        graphics_bits,
        compute_bits,
        frames: (0..FRAMES_IN_FLIGHT)
            .map(|_| FrameStamps::default())
            .collect(),
        passes: Vec::new(),
        frame_milliseconds: 0.0,
    })
}

pub fn destroy(context: &Context, timings: &mut Timings) {
    unsafe { context.device.destroy_query_pool(timings.pool, None) };
    timings.pool = vk::QueryPool::null();
}

fn bits_of(timings: &Timings, band: Band) -> u32 {
    match band {
        Band::Compute => timings.compute_bits,
        Band::Drawing => timings.graphics_bits,
    }
}

fn share_of(frame_index: usize) -> u32 {
    frame_index as u32 * STAMPS_PER_FRAME
}

pub fn collect(timings: &mut Timings, context: &Context, frame_index: usize) {
    let Some(frame) = timings.frames.get_mut(frame_index) else {
        return;
    };
    let written = frame.compute_next.max(frame.drawing_next);
    if frame.stamped.is_empty() || written == 0 {
        return;
    }
    let mut values = vec![0u64; STAMPS_PER_FRAME as usize];
    let runs = [
        (0u32, frame.compute_next),
        (COMPUTE_STAMPS, frame.drawing_next),
    ];
    let mut read = Ok(());
    for (start, count) in runs {
        if count == 0 {
            continue;
        }
        let run = &mut values[start as usize..(start + count) as usize];
        read = read.and(unsafe {
            context.device.get_query_pool_results(
                timings.pool,
                share_of(frame_index) + start,
                run,
                vk::QueryResultFlags::TYPE_64 | vk::QueryResultFlags::WAIT,
            )
        });
    }
    let stamped = std::mem::take(&mut frame.stamped);
    frame.compute_next = 0;
    frame.drawing_next = 0;
    if read.is_err() {
        return;
    }
    let tick_of = |slot: u32, bits: u32| {
        let value = values[slot as usize];
        if bits >= 64 {
            value
        } else {
            value & ((1u64 << bits) - 1)
        }
    };
    let mut passes: Vec<PassTime> = Vec::new();
    let mut first_drawn = u64::MAX;
    let mut last_drawn = 0u64;
    for stamp in &stamped {
        let band = if stamp.first < COMPUTE_STAMPS {
            Band::Compute
        } else {
            Band::Drawing
        };
        let bits = bits_of(timings, band);
        let opened = tick_of(stamp.first, bits);
        let closed = tick_of(stamp.first + 1, bits);
        let milliseconds = closed.saturating_sub(opened) as f32 * timings.period * 1.0e-6;
        match passes.iter_mut().find(|time| time.name == stamp.pass) {
            Some(time) => time.milliseconds += milliseconds,
            None => passes.push(PassTime {
                name: stamp.pass,
                milliseconds,
            }),
        }
        if band == Band::Drawing {
            first_drawn = first_drawn.min(opened);
            last_drawn = last_drawn.max(closed);
        }
    }
    timings.frame_milliseconds = if last_drawn > first_drawn {
        (last_drawn - first_drawn) as f32 * timings.period * 1.0e-6
    } else {
        0.0
    };
    timings.passes = passes;
}

pub fn reset(
    timings: &Timings,
    context: &Context,
    command_buffer: vk::CommandBuffer,
    frame_index: usize,
    band: Band,
) {
    if bits_of(timings, band) == 0 {
        return;
    }
    let (first, count) = match band {
        Band::Compute => (share_of(frame_index), COMPUTE_STAMPS),
        Band::Drawing => (
            share_of(frame_index) + COMPUTE_STAMPS,
            STAMPS_PER_FRAME - COMPUTE_STAMPS,
        ),
    };
    unsafe {
        context
            .device
            .cmd_reset_query_pool(command_buffer, timings.pool, first, count)
    };
}

pub fn open(
    timings: &mut Timings,
    context: &Context,
    command_buffer: vk::CommandBuffer,
    frame_index: usize,
    band: Band,
    pass: &'static str,
) -> Option<u32> {
    let first = reserve(timings, frame_index, band, pass)?;
    write(
        timings,
        context,
        command_buffer,
        share_of(frame_index) + first,
    );
    Some(first)
}

pub fn reserve(
    timings: &mut Timings,
    frame_index: usize,
    band: Band,
    pass: &'static str,
) -> Option<u32> {
    if bits_of(timings, band) == 0 {
        return None;
    }
    let frame = timings.frames.get_mut(frame_index)?;
    let first = match band {
        Band::Compute => {
            if frame.compute_next + 2 > COMPUTE_STAMPS {
                return None;
            }
            let first = frame.compute_next;
            frame.compute_next += 2;
            first
        }
        Band::Drawing => {
            if COMPUTE_STAMPS + frame.drawing_next + 2 > STAMPS_PER_FRAME {
                return None;
            }
            let first = COMPUTE_STAMPS + frame.drawing_next;
            frame.drawing_next += 2;
            first
        }
    };
    frame.stamped.push(Stamped { pass, first });
    Some(first)
}

pub fn open_reserved(
    timings: &Timings,
    context: &Context,
    command_buffer: vk::CommandBuffer,
    frame_index: usize,
    reserved: Option<u32>,
) {
    let Some(first) = reserved else {
        return;
    };
    write(
        timings,
        context,
        command_buffer,
        share_of(frame_index) + first,
    );
}

pub fn close(
    timings: &Timings,
    context: &Context,
    command_buffer: vk::CommandBuffer,
    frame_index: usize,
    opened: Option<u32>,
) {
    let Some(first) = opened else {
        return;
    };
    write(
        timings,
        context,
        command_buffer,
        share_of(frame_index) + first + 1,
    );
}

fn write(timings: &Timings, context: &Context, command_buffer: vk::CommandBuffer, slot: u32) {
    unsafe {
        context.device.cmd_write_timestamp2(
            command_buffer,
            vk::PipelineStageFlags2::ALL_COMMANDS,
            timings.pool,
            slot,
        )
    };
}
