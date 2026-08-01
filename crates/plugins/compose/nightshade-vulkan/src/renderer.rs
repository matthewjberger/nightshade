pub mod capture;
pub mod constraints;
pub mod context;
pub mod cube;
pub mod decals;
pub mod display;
pub mod environment;
pub mod glyph_atlas;
pub mod grab;
pub mod grid;
pub mod hiz;
pub mod image_table;
pub mod lines;
pub mod lookup;
pub mod memory;
pub mod outline;
pub mod paint;
pub mod particles;
pub mod pass;
pub mod picking;
pub mod post;
pub mod rays;
pub mod record;
pub mod resource;
pub mod scene;
pub mod sky;
pub mod sprites;
pub mod swapchain;
pub mod text;
pub mod timing;

use ash::vk;
use context::{Context, FRAMES_IN_FLIGHT, create_context, destroy_context};
use environment::{Environment, create_environment, destroy_environment};
use glyph_atlas::{
    GlyphAtlas, create_atlas as create_glyph_atlas, destroy_atlas as destroy_glyph_atlas,
};
use nightshade_render::data::config::RenderView;
use pass::{Pass, PassRecord, PassSetup, PassStage, ScopeKind};
use resource::{Resource, ResourceStates, Usage};
use scene::geometry::{GeometryArena, create_arena, destroy_arena};
use scene::materials::{MaterialTable, create_table, destroy_table};
use scene::shadow::{ShadowAtlas, create_atlas, destroy_atlas};
use scene::textures::{SceneTextures, create_textures, destroy_textures};
use swapchain::{Swapchain, create_swapchain, destroy_swapchain};

pub struct VulkanRenderer {
    pub render_scale: f32,
    pub viewport_rect: Option<[f32; 4]>,
    pub context: Context,
    pub swapchain: Swapchain,
    pub passes: Vec<(PassStage, Box<dyn Pass>)>,
    pub clear_color: [f32; 4],
    pub swapchain_clear: Option<[f32; 4]>,
    pub view: Option<RenderView>,
    pub capture: Option<capture::CaptureRequest>,
    pub geometry: GeometryArena,
    pub materials: MaterialTable,
    pub shadows: ShadowAtlas,
    pub glyphs: GlyphAtlas,
    pub textures: SceneTextures,
    pub environment: Environment,
    pub air: scene::fog::Air,
    pub probes: scene::probes::Probes,
    pub display: Option<display::DisplayLight>,
    pub kept: Kept,
    pub rays: crate::renderer::rays::Rays,
    pub copies: crate::renderer::scene::copies::CopyRows,
    pub vsync: bool,
    pub hdr_output: bool,
    pub paper_white: f32,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub compute_buffers: Vec<vk::CommandBuffer>,
    pub frames_sent: u64,
    pub frame_prepared: vk::Semaphore,
    pub image_available: Vec<vk::Semaphore>,
    pub render_finished: Vec<vk::Semaphore>,
    pub in_flight: Vec<vk::Fence>,
    pub images_in_flight: Vec<vk::Fence>,
    pub frame_index: usize,
    pub surface_dirty: bool,
    pub states: ResourceStates,
    pub picking: picking::Picking,
    pub timings: timing::Timings,
    pub recorders: record::Recorders,
    torn_down: bool,
}

#[derive(Default)]
pub struct Kept {
    pub sky: Option<SkySource>,
    pub color_lut: Option<Vec<u8>>,
}

pub enum SkySource {
    Bytes(Vec<u8>),
    Path(std::path::PathBuf),
}

fn register_screen_targets(states: &mut ResourceStates, swapchain: &Swapchain) {
    resource::register(
        states,
        Resource::Depth,
        swapchain.depth.image,
        vk::ImageAspectFlags::DEPTH,
    );
    resource::register(
        states,
        Resource::Hdr,
        swapchain.hdr.image,
        vk::ImageAspectFlags::COLOR,
    );
    resource::register(
        states,
        Resource::Normal,
        swapchain.normal.image,
        vk::ImageAspectFlags::COLOR,
    );
    resource::register(
        states,
        Resource::Motion,
        swapchain.motion.image,
        vk::ImageAspectFlags::COLOR,
    );
    resource::register(
        states,
        Resource::EntityId,
        swapchain.entity_id.image,
        vk::ImageAspectFlags::COLOR,
    );
    resource::register(
        states,
        Resource::Accumulated,
        swapchain.accumulated.image,
        vk::ImageAspectFlags::COLOR,
    );
    resource::register(
        states,
        Resource::Revealed,
        swapchain.revealed.image,
        vk::ImageAspectFlags::COLOR,
    );
}

pub fn create_renderer<W>(
    window_handle: &W,
    width: u32,
    height: u32,
    vsync: bool,
) -> Result<VulkanRenderer, Box<dyn std::error::Error>>
where
    W: raw_window_handle::HasDisplayHandle + raw_window_handle::HasWindowHandle,
{
    let context = create_context(window_handle)?;
    let display = display::display_light(window_handle);
    if let Some(light) = display {
        tracing::info!(
            "the display goes to {} nits, {} over a whole frame",
            light.peak_nits,
            light.frame_nits
        );
    }
    let render_scale = 1.0;
    let swapchain = create_swapchain(&context, width, height, vsync, render_scale, false)?;
    let geometry = create_arena(&context)?;
    let materials = create_table(&context)?;
    let shadows = create_atlas(&context)?;
    let textures = create_textures(&context)?;
    let environment = create_environment(&context)?;
    let air = scene::fog::create_air(&context)?;
    let probes = scene::probes::create_probes(&context)?;
    let glyphs = create_glyph_atlas(&context)?;

    let mut states = ResourceStates::default();
    register_screen_targets(&mut states, &swapchain);
    resource::register(
        &mut states,
        Resource::ShadowAtlas,
        shadows.image,
        vk::ImageAspectFlags::DEPTH,
    );
    resource::register(
        &mut states,
        Resource::LocalShadowAtlas,
        shadows.local_image,
        vk::ImageAspectFlags::DEPTH,
    );

    let allocate_info = vk::CommandBufferAllocateInfo::default()
        .command_pool(context.graphics.pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(FRAMES_IN_FLIGHT as u32);
    let command_buffers = unsafe { context.device.allocate_command_buffers(&allocate_info)? };
    let compute_buffers = match &context.compute {
        Some(lane) => {
            let allocate_info = vk::CommandBufferAllocateInfo::default()
                .command_pool(lane.pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(FRAMES_IN_FLIGHT as u32);
            unsafe { context.device.allocate_command_buffers(&allocate_info)? }
        }
        None => Vec::new(),
    };

    let semaphore_info = vk::SemaphoreCreateInfo::default();
    let image_available = (0..FRAMES_IN_FLIGHT)
        .map(|_| unsafe { context.device.create_semaphore(&semaphore_info, None) })
        .collect::<Result<Vec<_>, _>>()?;
    let render_finished = (0..swapchain.images.len())
        .map(|_| unsafe { context.device.create_semaphore(&semaphore_info, None) })
        .collect::<Result<Vec<_>, _>>()?;

    let mut timeline_info = vk::SemaphoreTypeCreateInfo::default()
        .semaphore_type(vk::SemaphoreType::TIMELINE)
        .initial_value(0);
    let timeline_create = vk::SemaphoreCreateInfo::default().push_next(&mut timeline_info);
    let frame_prepared = unsafe { context.device.create_semaphore(&timeline_create, None)? };

    let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
    let in_flight = (0..FRAMES_IN_FLIGHT)
        .map(|_| unsafe { context.device.create_fence(&fence_info, None) })
        .collect::<Result<Vec<_>, _>>()?;
    let images_in_flight = vec![vk::Fence::null(); swapchain.images.len()];
    let picking = picking::create(&context, FRAMES_IN_FLIGHT)?;
    let recorders = record::create_recorders(&context)?;
    let rays = crate::renderer::rays::create_rays(&context)?;
    let timings = timing::create(&context)?;

    Ok(VulkanRenderer {
        render_scale,
        viewport_rect: None,
        context,
        swapchain,
        passes: Vec::new(),
        clear_color: [0.02, 0.02, 0.03, 1.0],
        swapchain_clear: None,
        view: None,
        capture: None,
        geometry,
        materials,
        shadows,
        glyphs,
        textures,
        environment,
        air,
        probes,
        display,
        kept: Kept::default(),
        rays,
        copies: crate::renderer::scene::copies::CopyRows::default(),
        vsync,
        hdr_output: false,
        paper_white: 200.0,
        command_buffers,
        compute_buffers,
        frames_sent: 0,
        frame_prepared,
        image_available,
        render_finished,
        in_flight,
        images_in_flight,
        frame_index: 0,
        surface_dirty: false,
        states,
        picking,
        timings,
        recorders,
        torn_down: false,
    })
}

pub fn pass_setup(renderer: &VulkanRenderer) -> PassSetup<'_> {
    PassSetup {
        context: &renderer.context,
        color_format: renderer.swapchain.format.format,
        hdr_format: renderer.swapchain.hdr.format,
        normal_format: renderer.swapchain.normal.format,
        motion_format: renderer.swapchain.motion.format,
        entity_id_format: renderer.swapchain.entity_id.format,
        accumulated_format: renderer.swapchain.accumulated.format,
        revealed_format: renderer.swapchain.revealed.format,
        depth_format: renderer.swapchain.depth.format,
        frames_in_flight: FRAMES_IN_FLIGHT,
        extent: renderer.swapchain.render_extent,
        screen: renderer.swapchain.extent,
        color: renderer.swapchain.hdr.view,
        depth: renderer.swapchain.depth.view,
        normal: renderer.swapchain.normal.view,
        motion: renderer.swapchain.motion.view,
        entity_id: renderer.swapchain.entity_id.view,
        accumulated: renderer.swapchain.accumulated.view,
        revealed: renderer.swapchain.revealed.view,
        glyphs: &renderer.glyphs,
    }
}

pub fn install_sky_pass(renderer: &mut VulkanRenderer, pass: Box<dyn Pass>) {
    install_pass(renderer, PassStage::Sky, pass);
}

pub fn install_scene_pass(renderer: &mut VulkanRenderer, pass: Box<dyn Pass>) {
    install_pass(renderer, PassStage::Scene, pass);
}

pub fn install_decal_pass(renderer: &mut VulkanRenderer, pass: Box<dyn Pass>) {
    install_pass(renderer, PassStage::Decal, pass);
}

pub fn install_post_pass(renderer: &mut VulkanRenderer, pass: Box<dyn Pass>) {
    install_pass(renderer, PassStage::Post, pass);
}

pub fn install_swapchain_pass(renderer: &mut VulkanRenderer, pass: Box<dyn Pass>) {
    install_pass(renderer, PassStage::Swapchain, pass);
}

fn install_pass(renderer: &mut VulkanRenderer, stage: PassStage, pass: Box<dyn Pass>) {
    let at = renderer
        .passes
        .iter()
        .position(|(existing, _)| *existing > stage)
        .unwrap_or(renderer.passes.len());
    pass.register(&mut renderer.states);
    renderer.passes.insert(at, (stage, pass));
}

pub fn pass_mut<'a>(renderer: &'a mut VulkanRenderer, name: &str) -> Option<&'a mut dyn Pass> {
    renderer
        .passes
        .iter_mut()
        .find(|(_, pass)| pass.name() == name)
        .map(|(_, pass)| pass.as_mut())
}

pub fn pass_with_context<'a>(
    renderer: &'a mut VulkanRenderer,
    name: &str,
) -> Option<(&'a Context, &'a mut dyn Pass)> {
    let VulkanRenderer {
        context, passes, ..
    } = renderer;
    let pass = passes
        .iter_mut()
        .find(|(_, pass)| pass.name() == name)
        .map(|(_, pass)| pass.as_mut())?;
    Some((context, pass))
}

pub fn pass_with_glyphs<'a>(
    renderer: &'a mut VulkanRenderer,
    name: &str,
) -> Option<(&'a Context, &'a mut GlyphAtlas, &'a mut dyn Pass)> {
    let VulkanRenderer {
        context,
        glyphs,
        passes,
        ..
    } = renderer;
    let pass = passes
        .iter_mut()
        .find(|(_, pass)| pass.name() == name)
        .map(|(_, pass)| pass.as_mut())?;
    Some((context, glyphs, pass))
}

pub fn resize_surface(
    renderer: &mut VulkanRenderer,
    width: u32,
    height: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    if width == 0 || height == 0 {
        return Ok(());
    }
    unsafe { renderer.context.device.device_wait_idle()? };
    let shown_before = renderer.swapchain.format;
    destroy_swapchain(&renderer.context, &mut renderer.swapchain);
    renderer.swapchain = create_swapchain(
        &renderer.context,
        width,
        height,
        renderer.vsync,
        renderer.render_scale,
        renderer.hdr_output,
    )?;
    swapchain::declare_display_light(&renderer.context, &renderer.swapchain, renderer.display);
    if renderer.swapchain.format != shown_before {
        let mut passes = std::mem::take(&mut renderer.passes);
        let setup = pass_setup(renderer);
        for (_, pass) in passes.iter_mut() {
            pass.retarget(&setup);
        }
        renderer.passes = passes;
    }
    let targets = pass::ScreenTargets {
        extent: renderer.swapchain.render_extent,
        screen: renderer.swapchain.extent,
        color: renderer.swapchain.hdr.view,
        depth: renderer.swapchain.depth.view,
        normal: renderer.swapchain.normal.view,
        motion: renderer.swapchain.motion.view,
        entity_id: renderer.swapchain.entity_id.view,
        accumulated: renderer.swapchain.accumulated.view,
        revealed: renderer.swapchain.revealed.view,
    };
    let mut states = std::mem::take(&mut renderer.states);
    register_screen_targets(&mut states, &renderer.swapchain);
    for (_, pass) in renderer.passes.iter_mut() {
        pass.resize(&renderer.context, targets);
        pass.register(&mut states);
    }
    renderer.states = states;
    renderer.images_in_flight = vec![vk::Fence::null(); renderer.swapchain.images.len()];
    renderer.frame_index = 0;
    renderer.surface_dirty = false;
    Ok(())
}

pub fn render_frame(renderer: &mut VulkanRenderer) -> Result<(), Box<dyn std::error::Error>> {
    if renderer.surface_dirty {
        let extent = renderer.swapchain.extent;
        resize_surface(renderer, extent.width, extent.height)?;
    }

    crate::renderer::memory::reclaim_sent_work(&renderer.context);
    crate::renderer::rays::retire_replaced(
        &mut renderer.rays,
        &renderer.context,
        crate::renderer::context::FRAMES_IN_FLIGHT as u64,
    );
    crate::renderer::image_table::retire_replaced(
        &mut renderer.textures.table,
        &renderer.context,
        crate::renderer::context::FRAMES_IN_FLIGHT as u64,
    );

    let fence = renderer.in_flight[renderer.frame_index];
    unsafe {
        renderer
            .context
            .device
            .wait_for_fences(&[fence], true, u64::MAX)?
    };
    timing::collect(
        &mut renderer.timings,
        &renderer.context,
        renderer.frame_index,
    );
    if let Err(error) = record::reset(
        &renderer.context,
        &mut renderer.recorders,
        renderer.frame_index,
    ) {
        tracing::error!("the lanes of the frame were not emptied: {error}");
    }

    let acquired = unsafe {
        renderer.swapchain.loader.acquire_next_image(
            renderer.swapchain.handle,
            u64::MAX,
            renderer.image_available[renderer.frame_index],
            vk::Fence::null(),
        )
    };
    let image_index = match acquired {
        Ok((index, _suboptimal)) => index,
        Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
            renderer.surface_dirty = true;
            return Ok(());
        }
        Err(error) => return Err(error.into()),
    };

    let waiting = renderer.images_in_flight[image_index as usize];
    if waiting != vk::Fence::null() {
        unsafe {
            renderer
                .context
                .device
                .wait_for_fences(&[waiting], true, u64::MAX)?
        };
    }
    renderer.images_in_flight[image_index as usize] = fence;
    unsafe { renderer.context.device.reset_fences(&[fence])? };
    crate::renderer::environment::settle_scene_set(
        &renderer.context,
        &mut renderer.environment,
        renderer.frame_index,
    );
    crate::renderer::scene::shadow::settle_world_set(
        &renderer.context,
        &mut renderer.shadows,
        renderer.frame_index,
    );
    picking::collect(&mut renderer.picking, renderer.frame_index);

    record_frame(renderer, image_index)?;
    submit_before_drawing(renderer)?;
    submit_frame(renderer, image_index, fence)?;
    present_frame(renderer, image_index)?;

    renderer.frames_sent += 1;
    renderer.frame_index = (renderer.frame_index + 1) % FRAMES_IN_FLIGHT;
    Ok(())
}

fn record_frame(
    renderer: &mut VulkanRenderer,
    image_index: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let VulkanRenderer {
        context,
        swapchain,
        passes,
        view,
        rays,
        geometry,
        command_buffers,
        compute_buffers,
        frame_index,
        clear_color,
        swapchain_clear,
        states,
        picking,
        glyphs,
        air,
        viewport_rect,
        timings,
        probes,
        recorders,
        ..
    } = renderer;
    let frame_index = *frame_index;
    let command_buffer = command_buffers[frame_index];
    let device = context.device.clone();
    let before_drawing = compute_buffers
        .get(frame_index)
        .copied()
        .unwrap_or(command_buffer);
    unsafe {
        if before_drawing != command_buffer {
            device.reset_command_buffer(before_drawing, vk::CommandBufferResetFlags::empty())?;
            device.begin_command_buffer(before_drawing, &vk::CommandBufferBeginInfo::default())?;
        }
        device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())?;
        device.begin_command_buffer(command_buffer, &vk::CommandBufferBeginInfo::default())?;
    }
    let before_band = if before_drawing == command_buffer {
        timing::Band::Drawing
    } else {
        timing::Band::Compute
    };
    timing::reset(timings, context, before_drawing, frame_index, before_band);
    if before_drawing != command_buffer {
        timing::reset(
            timings,
            context,
            command_buffer,
            frame_index,
            timing::Band::Drawing,
        );
    }

    resource::register(
        states,
        Resource::Swapchain,
        swapchain.images[image_index as usize],
        vk::ImageAspectFlags::COLOR,
    );

    {
        let mut record = PassRecord {
            context,
            command_buffer: before_drawing,
            extent: swapchain.render_extent,
            frame_index,
            view: view.as_ref(),
            scope: ScopeKind::EarlyDepth,
            states,
            glyphs,
            air,
            probes,
        };
        for (_, pass) in passes.iter_mut() {
            let opened = timing::open(
                timings,
                context,
                before_drawing,
                frame_index,
                before_band,
                pass.name(),
            );
            pass.compute(&mut record);
            timing::close(timings, context, before_drawing, frame_index, opened);
        }
        crate::renderer::rays::record_builds(
            rays,
            context,
            before_drawing,
            crate::renderer::rays::ArenaAddresses {
                vertices: crate::renderer::rays::address_of(
                    context,
                    scene::geometry::vertex_buffer(geometry),
                ),
                indices: crate::renderer::rays::address_of(
                    context,
                    scene::geometry::index_buffer(geometry),
                ),
            },
        );
    }
    if before_drawing != command_buffer {
        unsafe { device.end_command_buffer(before_drawing)? };
    }

    let prepass = passes.iter().any(|(_, pass)| pass.has_depth_prepass());
    let asked_about = picking.wanted.take().filter(|_| prepass);
    let after = if asked_about.is_some() {
        HANDING_OVER_AND_PICKING
    } else {
        HANDING_OVER
    };

    let settled_depth = if prepass {
        vk::AttachmentLoadOp::LOAD
    } else {
        vk::AttachmentLoadOp::CLEAR
    };

    let scopes = [
        Scope {
            kind: ScopeKind::EarlyDepth,
            stages: &[PassStage::Scene],
            color_load: vk::AttachmentLoadOp::CLEAR,
            depth_load: vk::AttachmentLoadOp::CLEAR,
            target: ScopeTarget::Hdr,
            color: false,
            depth: true,
            normal: false,
            motion: false,
            ids: true,
        },
        Scope {
            kind: ScopeKind::LateDepth,
            stages: &[PassStage::Scene],
            color_load: vk::AttachmentLoadOp::CLEAR,
            depth_load: vk::AttachmentLoadOp::LOAD,
            target: ScopeTarget::Hdr,
            color: false,
            depth: true,
            normal: false,
            motion: false,
            ids: true,
        },
        Scope {
            kind: ScopeKind::Color,
            stages: &[PassStage::Sky, PassStage::Scene],
            color_load: vk::AttachmentLoadOp::CLEAR,
            depth_load: settled_depth,
            target: ScopeTarget::Hdr,
            color: true,
            depth: true,
            normal: true,
            motion: true,
            ids: false,
        },
        Scope {
            kind: ScopeKind::Refracted,
            stages: &[PassStage::Scene],
            color_load: vk::AttachmentLoadOp::LOAD,
            depth_load: vk::AttachmentLoadOp::LOAD,
            target: ScopeTarget::Hdr,
            color: true,
            depth: true,
            normal: true,
            motion: true,
            ids: false,
        },
        Scope {
            kind: ScopeKind::Blended,
            stages: &[PassStage::Scene],
            color_load: vk::AttachmentLoadOp::CLEAR,
            depth_load: vk::AttachmentLoadOp::LOAD,
            target: ScopeTarget::Blended,
            color: true,
            depth: true,
            normal: true,
            motion: true,
            ids: false,
        },
        Scope {
            kind: ScopeKind::OverSolids,
            stages: &[PassStage::Scene],
            color_load: vk::AttachmentLoadOp::LOAD,
            depth_load: vk::AttachmentLoadOp::LOAD,
            target: ScopeTarget::Hdr,
            color: true,
            depth: true,
            normal: true,
            motion: true,
            ids: false,
        },
        Scope {
            kind: ScopeKind::Decal,
            stages: &[PassStage::Decal],
            color_load: vk::AttachmentLoadOp::LOAD,
            depth_load: vk::AttachmentLoadOp::LOAD,
            target: ScopeTarget::Hdr,
            color: true,
            depth: false,
            normal: false,
            motion: false,
            ids: false,
        },
        Scope {
            kind: ScopeKind::Post,
            stages: &[PassStage::Post],
            color_load: vk::AttachmentLoadOp::CLEAR,
            depth_load: vk::AttachmentLoadOp::LOAD,
            target: ScopeTarget::Swapchain,
            color: true,
            depth: false,
            normal: false,
            motion: false,
            ids: false,
        },
        Scope {
            kind: ScopeKind::Swapchain,
            stages: &[PassStage::Swapchain],
            color_load: vk::AttachmentLoadOp::LOAD,
            depth_load: vk::AttachmentLoadOp::LOAD,
            target: ScopeTarget::Swapchain,
            color: true,
            depth: false,
            normal: false,
            motion: false,
            ids: false,
        },
    ];

    let moments = [
        Moment::Scope(0),
        Moment::Between(ScopeKind::Pyramid),
        Moment::Scope(1),
        Moment::Between(ScopeKind::Gather),
        Moment::Scope(2),
        Moment::Between(ScopeKind::Grab),
        Moment::Scope(3),
        Moment::Scope(4),
        Moment::Scope(5),
        Moment::Scope(6),
        Moment::Scope(7),
        Moment::Scope(8),
    ];

    for (position, moment) in moments.into_iter().enumerate() {
        let kind = match moment {
            Moment::Scope(index) => scopes[index].kind,
            Moment::Between(kind) => kind,
        };
        if !prepass && !settles_color(kind) {
            continue;
        }

        apply_usage(states, &device, command_buffer, passes, kind);

        let mut record = PassRecord {
            context,
            command_buffer,
            extent: match scope_extent(moment, &scopes) {
                ScopeTarget::Hdr | ScopeTarget::Blended => swapchain.render_extent,
                ScopeTarget::Swapchain => swapchain.extent,
            },
            frame_index,
            view: view.as_ref(),
            scope: kind,
            states,
            glyphs,
            air,
            probes,
        };

        let Moment::Scope(index) = moment else {
            for (_, pass) in passes.iter_mut() {
                let opened = timing::open(
                    timings,
                    context,
                    command_buffer,
                    frame_index,
                    timing::Band::Drawing,
                    pass.name(),
                );
                pass.between(&mut record);
                timing::close(timings, context, command_buffer, frame_index, opened);
            }
            continue;
        };
        let scope = &scopes[index];

        if scope.color {
            for stage in scope.stages {
                for (pass_stage, pass) in passes.iter_mut() {
                    if pass_stage == stage {
                        let opened = timing::open(
                            timings,
                            context,
                            command_buffer,
                            frame_index,
                            timing::Band::Drawing,
                            pass.name(),
                        );
                        pass.offscreen(&mut record);
                        timing::close(timings, context, command_buffer, frame_index, opened);
                    }
                }
            }
        }

        let color_view = match scope.target {
            ScopeTarget::Hdr => swapchain.hdr.view,
            ScopeTarget::Blended => swapchain.accumulated.view,
            ScopeTarget::Swapchain => swapchain.views[image_index as usize],
        };
        let rest = &moments[position + 1..];
        let color_resource = match scope.target {
            ScopeTarget::Hdr => Resource::Hdr,
            ScopeTarget::Blended => Resource::Accumulated,
            ScopeTarget::Swapchain => Resource::Swapchain,
        };
        let second_view = match scope.target {
            ScopeTarget::Blended => swapchain.revealed.view,
            _ => swapchain.normal.view,
        };
        let second_resource = match scope.target {
            ScopeTarget::Blended => Resource::Revealed,
            _ => Resource::Normal,
        };
        if scope.color {
            record.require(color_resource, Usage::ColorAttachment);
        }
        if scope.normal {
            record.require(second_resource, Usage::ColorAttachment);
        }
        if scope.motion {
            record.require(Resource::Motion, Usage::ColorAttachment);
        }
        if scope.ids {
            record.require(Resource::EntityId, Usage::ColorAttachment);
        }
        if scope.depth {
            record.require(Resource::Depth, Usage::DepthAttachment);
        }
        let color_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(color_view)
            .image_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL)
            .load_op(scope.color_load)
            .store_op(store_for(color_resource, rest, &scopes, passes, after))
            .clear_value(vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: match (scope.target, *swapchain_clear) {
                        (ScopeTarget::Blended, _) => [0.0; 4],
                        (ScopeTarget::Swapchain, Some(behind)) => behind,
                        _ => *clear_color,
                    },
                },
            });
        let depth_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(swapchain.depth.view)
            .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
            .load_op(scope.depth_load)
            .store_op(store_for(Resource::Depth, rest, &scopes, passes, after))
            .clear_value(vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 0.0,
                    stencil: 0,
                },
            });
        let normal_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(second_view)
            .image_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL)
            .load_op(scope.color_load)
            .store_op(store_for(second_resource, rest, &scopes, passes, after))
            .clear_value(vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: match scope.target {
                        ScopeTarget::Blended => [1.0, 1.0, 1.0, 1.0],
                        _ => [0.0, 0.0, 0.0, 0.0],
                    },
                },
            });
        let motion_load = match scope.target {
            ScopeTarget::Blended => vk::AttachmentLoadOp::LOAD,
            _ => scope.color_load,
        };
        let motion_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(swapchain.motion.view)
            .image_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL)
            .load_op(motion_load)
            .store_op(store_for(Resource::Motion, rest, &scopes, passes, after))
            .clear_value(vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 0.0],
                },
            });
        let id_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(swapchain.entity_id.view)
            .image_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL)
            .load_op(scope.depth_load)
            .store_op(store_for(Resource::EntityId, rest, &scopes, passes, after))
            .clear_value(vk::ClearValue {
                color: vk::ClearColorValue { uint32: [0; 4] },
            });
        let area = match scope.target {
            ScopeTarget::Hdr | ScopeTarget::Blended => swapchain.render_extent,
            ScopeTarget::Swapchain => swapchain.extent,
        };
        let attachments = [color_attachment, normal_attachment];
        let with_motion = [color_attachment, normal_attachment, motion_attachment];
        let rendering_info = vk::RenderingInfo::default()
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: area,
            })
            .layer_count(1)
            .color_attachments(match (scope.color, scope.normal, scope.motion, scope.ids) {
                (true, true, true, _) => &with_motion,
                (true, true, false, _) => &attachments,
                (true, false, _, _) => std::slice::from_ref(&color_attachment),
                (false, _, _, true) => std::slice::from_ref(&id_attachment),
                (false, _, _, false) => &[],
            });
        let rendering_info = if scope.depth {
            rendering_info.depth_attachment(&depth_attachment)
        } else {
            rendering_info
        };

        let mut drawing: Vec<&mut Box<dyn Pass>> = passes
            .iter_mut()
            .filter(|(pass_stage, pass)| {
                scope.stages.contains(pass_stage) && (scope.color || pass.has_depth_prepass())
            })
            .map(|(_, pass)| pass)
            .collect();
        let apart = drawing.len() >= record::WORTH_SPLITTING && record::lanes(recorders) > 1;
        let rendering_info = if apart {
            rendering_info.flags(vk::RenderingFlags::CONTENTS_SECONDARY_COMMAND_BUFFERS)
        } else {
            rendering_info
        };

        unsafe { device.cmd_begin_rendering(command_buffer, &rendering_info) };
        let viewport = match (scope.target, *viewport_rect) {
            (ScopeTarget::Swapchain, Some(rect)) => Some(rect),
            _ => None,
        };
        if apart {
            let split = record::Split {
                context,
                frame_index,
                extent: area,
                view: view.as_ref(),
                scope: kind,
                glyphs,
                air,
                probes,
                inherited: record::Inherited {
                    color_formats: color_formats_of(scope, swapchain),
                    depth_format: if scope.depth {
                        swapchain.depth.format
                    } else {
                        vk::Format::UNDEFINED
                    },
                },
                viewport,
            };
            let recorded = record::record_split(&split, &mut drawing, recorders, timings);
            if !recorded.is_empty() {
                unsafe { device.cmd_execute_commands(command_buffer, &recorded) };
            }
        } else {
            match viewport {
                Some(rect) => record::set_viewport_rect(&device, command_buffer, rect),
                None => record::set_viewport(&device, command_buffer, area),
            }
            for pass in drawing.iter_mut() {
                let opened = timing::open(
                    timings,
                    context,
                    command_buffer,
                    frame_index,
                    timing::Band::Drawing,
                    pass.name(),
                );
                pass.draw(&mut record);
                timing::close(timings, context, command_buffer, frame_index, opened);
            }
        }

        unsafe { device.cmd_end_rendering(command_buffer) };
    }

    if let Some(point) = asked_about {
        resource::require(
            states,
            &device,
            command_buffer,
            Resource::Depth,
            Usage::TransferRead,
        );
        resource::require(
            states,
            &device,
            command_buffer,
            Resource::EntityId,
            Usage::TransferRead,
        );
        picking::copy(
            picking,
            &device,
            command_buffer,
            frame_index,
            swapchain.depth.image,
            swapchain.entity_id.image,
            swapchain.extent,
            point,
        );
    }

    resource::require(
        states,
        &device,
        command_buffer,
        Resource::Swapchain,
        Usage::Present,
    );

    unsafe { device.end_command_buffer(command_buffer)? };
    Ok(())
}

fn scope_extent(moment: Moment, scopes: &[Scope]) -> ScopeTarget {
    match moment {
        Moment::Scope(index) => scopes[index].target,
        Moment::Between(_) => ScopeTarget::Hdr,
    }
}

#[derive(Clone, Copy)]
enum Moment {
    Scope(usize),
    Between(ScopeKind),
}

fn settles_color(kind: ScopeKind) -> bool {
    matches!(
        kind,
        ScopeKind::Color
            | ScopeKind::Grab
            | ScopeKind::Blended
            | ScopeKind::OverSolids
            | ScopeKind::Decal
            | ScopeKind::Post
            | ScopeKind::Swapchain
    )
}

fn apply_usage(
    states: &mut ResourceStates,
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    passes: &[(PassStage, Box<dyn Pass>)],
    kind: ScopeKind,
) {
    for (_, pass) in passes.iter() {
        for (resource, usage) in pass.usage(kind) {
            resource::require(states, device, command_buffer, *resource, *usage);
        }
    }
}

const HANDING_OVER: &[(Resource, Usage)] = &[(Resource::Swapchain, Usage::Present)];
const HANDING_OVER_AND_PICKING: &[(Resource, Usage)] = &[
    (Resource::Swapchain, Usage::Present),
    (Resource::Depth, Usage::TransferRead),
    (Resource::EntityId, Usage::TransferRead),
];

fn store_for(
    resource: Resource,
    rest: &[Moment],
    scopes: &[Scope],
    passes: &[(PassStage, Box<dyn Pass>)],
    after: &[(Resource, Usage)],
) -> vk::AttachmentStoreOp {
    for (named, usage) in after {
        if *named == resource && usage.keeps() {
            return vk::AttachmentStoreOp::STORE;
        }
    }
    for moment in rest {
        let kind = match moment {
            Moment::Scope(index) => scopes[*index].kind,
            Moment::Between(kind) => *kind,
        };
        for (_, pass) in passes.iter() {
            for (named, usage) in pass.usage(kind) {
                if *named == resource && usage.keeps() {
                    return vk::AttachmentStoreOp::STORE;
                }
            }
        }
        let Moment::Scope(index) = moment else {
            continue;
        };
        if loads(&scopes[*index], resource) {
            return vk::AttachmentStoreOp::STORE;
        }
    }
    vk::AttachmentStoreOp::DONT_CARE
}

fn loads(scope: &Scope, resource: Resource) -> bool {
    let color_resource = match scope.target {
        ScopeTarget::Hdr => Resource::Hdr,
        ScopeTarget::Blended => Resource::Accumulated,
        ScopeTarget::Swapchain => Resource::Swapchain,
    };
    let loaded = |load| load == vk::AttachmentLoadOp::LOAD;
    match resource {
        Resource::Depth => scope.depth && loaded(scope.depth_load),
        Resource::EntityId => scope.ids && loaded(scope.depth_load),
        Resource::Normal | Resource::Revealed => scope.normal && loaded(scope.color_load),
        Resource::Motion => {
            scope.motion && (scope.target == ScopeTarget::Blended || loaded(scope.color_load))
        }
        _ => resource == color_resource && scope.color && loaded(scope.color_load),
    }
}

struct Scope {
    kind: ScopeKind,
    stages: &'static [PassStage],
    color_load: vk::AttachmentLoadOp,
    depth_load: vk::AttachmentLoadOp,
    target: ScopeTarget,
    color: bool,
    depth: bool,
    normal: bool,
    motion: bool,
    ids: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScopeTarget {
    Hdr,
    Blended,
    Swapchain,
}

fn submit_before_drawing(renderer: &VulkanRenderer) -> Result<(), Box<dyn std::error::Error>> {
    let Some(lane) = &renderer.context.compute else {
        return Ok(());
    };
    let Some(command_buffer) = renderer.compute_buffers.get(renderer.frame_index).copied() else {
        return Ok(());
    };
    let counted = renderer.frames_sent + 1;
    let signal = vk::SemaphoreSubmitInfo::default()
        .semaphore(renderer.frame_prepared)
        .value(counted)
        .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS);
    let buffer = vk::CommandBufferSubmitInfo::default().command_buffer(command_buffer);
    let mut waits = Vec::new();
    if let Some(transfer) = &renderer.context.transfer
        && let Ok(work) = transfer.work.lock()
        && work.sent > 0
    {
        waits.push(
            vk::SemaphoreSubmitInfo::default()
                .semaphore(work.timeline)
                .value(work.sent)
                .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS),
        );
    }
    let submit = vk::SubmitInfo2::default()
        .wait_semaphore_infos(&waits)
        .signal_semaphore_infos(std::slice::from_ref(&signal))
        .command_buffer_infos(std::slice::from_ref(&buffer));
    unsafe {
        renderer
            .context
            .device
            .queue_submit2(lane.queue, std::slice::from_ref(&submit), vk::Fence::null())
            .inspect_err(|outcome| renderer.context.note(*outcome))?
    };
    Ok(())
}

fn submit_frame(
    renderer: &mut VulkanRenderer,
    image_index: u32,
    fence: vk::Fence,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut waits = vec![
        vk::SemaphoreSubmitInfo::default()
            .semaphore(renderer.image_available[renderer.frame_index])
            .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT),
    ];
    if let Some(transfer) = &renderer.context.transfer
        && let Ok(work) = transfer.work.lock()
        && work.sent > 0
    {
        waits.push(
            vk::SemaphoreSubmitInfo::default()
                .semaphore(work.timeline)
                .value(work.sent)
                .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS),
        );
    }
    let counted = renderer.frames_sent + 1;
    if renderer.context.compute.is_some() && !renderer.compute_buffers.is_empty() {
        waits.push(
            vk::SemaphoreSubmitInfo::default()
                .semaphore(renderer.frame_prepared)
                .value(counted)
                .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS),
        );
    }
    let signal = vk::SemaphoreSubmitInfo::default()
        .semaphore(renderer.render_finished[image_index as usize])
        .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS);
    let buffer = vk::CommandBufferSubmitInfo::default()
        .command_buffer(renderer.command_buffers[renderer.frame_index]);
    let submit = vk::SubmitInfo2::default()
        .wait_semaphore_infos(&waits)
        .signal_semaphore_infos(std::slice::from_ref(&signal))
        .command_buffer_infos(std::slice::from_ref(&buffer));
    unsafe {
        renderer
            .context
            .device
            .queue_submit2(
                renderer.context.graphics.queue,
                std::slice::from_ref(&submit),
                fence,
            )
            .inspect_err(|outcome| renderer.context.note(*outcome))?
    };
    Ok(())
}

fn take_capture(renderer: &mut VulkanRenderer, image_index: u32) {
    let Some(request) = renderer.capture.take() else {
        return;
    };
    if let Err(error) = unsafe { renderer.context.device.device_wait_idle() } {
        tracing::error!("the device did not settle before the screenshot: {error}");
        return;
    }
    match capture::capture_image(
        &renderer.context,
        renderer.swapchain.images[image_index as usize],
        renderer.swapchain.format.format,
        renderer.swapchain.extent,
        renderer.paper_white,
        &request,
    ) {
        Ok(path) => tracing::info!("wrote a screenshot to {}", path.display()),
        Err(error) => tracing::error!("the screenshot was not written: {error}"),
    }
}

fn present_frame(
    renderer: &mut VulkanRenderer,
    image_index: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let semaphores = [renderer.render_finished[image_index as usize]];
    let swapchains = [renderer.swapchain.handle];
    let indices = [image_index];
    let present_info = vk::PresentInfoKHR::default()
        .wait_semaphores(&semaphores)
        .swapchains(&swapchains)
        .image_indices(&indices);
    let outcome = unsafe {
        renderer
            .swapchain
            .loader
            .queue_present(renderer.context.present_queue, &present_info)
    };
    take_capture(renderer, image_index);
    match outcome {
        Ok(true) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
            renderer.surface_dirty = true;
            Ok(())
        }
        Ok(false) => Ok(()),
        Err(error) => {
            renderer.context.note(error);
            Err(error.into())
        }
    }
}

impl VulkanRenderer {
    pub fn is_torn_down(&self) -> bool {
        self.torn_down
    }

    pub fn tear_down(&mut self) {
        if self.torn_down {
            return;
        }
        self.torn_down = true;
        unsafe {
            if let Err(error) = self.context.device.device_wait_idle() {
                tracing::error!("the device did not become idle before teardown: {error}");
            }
            for semaphore in self.image_available.drain(..) {
                self.context.device.destroy_semaphore(semaphore, None);
            }
            for semaphore in self.render_finished.drain(..) {
                self.context.device.destroy_semaphore(semaphore, None);
            }
            self.context
                .device
                .destroy_semaphore(self.frame_prepared, None);
            for fence in self.in_flight.drain(..) {
                self.context.device.destroy_fence(fence, None);
            }
            self.context
                .device
                .free_command_buffers(self.context.graphics.pool, &self.command_buffers);
            if let Some(lane) = &self.context.compute
                && !self.compute_buffers.is_empty()
            {
                self.context
                    .device
                    .free_command_buffers(lane.pool, &self.compute_buffers);
            }
        }
        self.command_buffers.clear();
        self.compute_buffers.clear();
        crate::renderer::rays::destroy_rays(&self.context, &mut self.rays);
        timing::destroy(&self.context, &mut self.timings);
        picking::destroy(&mut self.picking, &self.context);
        destroy_arena(&self.context, &mut self.geometry);
        destroy_table(&self.context, &mut self.materials);
        destroy_atlas(&self.context, &mut self.shadows);
        destroy_glyph_atlas(&self.context, &mut self.glyphs);
        destroy_textures(&self.context, &mut self.textures);
        destroy_environment(&self.context, &mut self.environment);
        for (_, pass) in &mut self.passes {
            pass.destroy(&self.context);
        }
        self.passes.clear();
        scene::fog::destroy_air(&self.context, &mut self.air);
        scene::probes::destroy_probes(&self.context, &mut self.probes);
        record::destroy_recorders(&self.context, &mut self.recorders);
        destroy_swapchain(&self.context, &mut self.swapchain);
        destroy_context(&mut self.context);
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        self.tear_down();
    }
}

pub(crate) struct Transition {
    pub image: vk::Image,
    pub aspect: vk::ImageAspectFlags,
    pub old_layout: vk::ImageLayout,
    pub new_layout: vk::ImageLayout,
    pub source_access: vk::AccessFlags2,
    pub destination_access: vk::AccessFlags2,
    pub source_stage: vk::PipelineStageFlags2,
    pub destination_stage: vk::PipelineStageFlags2,
}

pub(crate) fn transition(
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    step: Transition,
) {
    let barrier = vk::ImageMemoryBarrier2::default()
        .src_stage_mask(step.source_stage)
        .src_access_mask(step.source_access)
        .dst_stage_mask(step.destination_stage)
        .dst_access_mask(step.destination_access)
        .old_layout(step.old_layout)
        .new_layout(step.new_layout)
        .image(step.image)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: step.aspect,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });
    let dependency =
        vk::DependencyInfo::default().image_memory_barriers(std::slice::from_ref(&barrier));
    unsafe { device.cmd_pipeline_barrier2(command_buffer, &dependency) };
}

fn color_formats_of(scope: &Scope, swapchain: &Swapchain) -> Vec<vk::Format> {
    let color = match scope.target {
        ScopeTarget::Hdr => swapchain.hdr.format,
        ScopeTarget::Blended => swapchain.accumulated.format,
        ScopeTarget::Swapchain => swapchain.format.format,
    };
    let second = match scope.target {
        ScopeTarget::Blended => swapchain.revealed.format,
        _ => swapchain.normal.format,
    };
    match (scope.color, scope.normal, scope.motion, scope.ids) {
        (true, true, true, _) => vec![color, second, swapchain.motion.format],
        (true, true, false, _) => vec![color, second],
        (true, false, _, _) => vec![color],
        (false, _, _, true) => vec![swapchain.entity_id.format],
        (false, _, _, false) => Vec::new(),
    }
}
