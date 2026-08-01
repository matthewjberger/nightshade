use crate::renderer::wgpu::render_configs::{
    CameraFrameInputs, FrameOutputs, LinesFrameInputs, RenderInputs, RendererCommand, ViewInputs,
};
use nightshade::ecs::ResourceMap;
use nightshade_render::commands::mesh_state::{
    mesh_state_restore_unconsumed, mesh_state_take_frame,
};
use nightshade_render::queries::mesh_state::{
    mesh_state_dirty_for_culling, mesh_state_needs_full_invalidation,
};

fn map_render_commands(resources: &mut ResourceMap) -> Vec<RendererCommand> {
    std::mem::take(
        &mut resources
            .res_mut::<nightshade_graphics::resources::queues::CommandQueues>()
            .render,
    )
    .into_iter()
    .filter_map(|command| match command {
        nightshade_graphics::data::queues::RenderCommand::ReloadTexture {
            name,
            rgba_data,
            width,
            height,
        } => Some(RendererCommand::ReloadTexture {
            name,
            rgba_data,
            width,
            height,
        }),
        nightshade_graphics::data::queues::RenderCommand::LoadHdrSkybox { hdr_data } => {
            Some(RendererCommand::LoadHdrSkybox { hdr_data })
        }
        nightshade_graphics::data::queues::RenderCommand::SetColorLut { data } => {
            Some(RendererCommand::SetColorLut { data })
        }
        nightshade_graphics::data::queues::RenderCommand::LoadHdrSkyboxFromPath { path } => {
            match std::fs::read(&path) {
                Ok(bytes) => Some(RendererCommand::LoadHdrSkybox { hdr_data: bytes }),
                Err(error) => {
                    tracing::error!("Failed to read HDR file {}: {}", path.display(), error);
                    None
                }
            }
        }
        nightshade_graphics::data::queues::RenderCommand::CaptureScreenshot {
            path,
            max_dimension,
        } => Some(RendererCommand::CaptureScreenshot {
            path,
            max_dimension,
        }),
        nightshade_graphics::data::queues::RenderCommand::RemakeRenderer => None,
    })
    .collect()
}

pub fn compose_render_inputs(mut ecs: nightshade::ecs::World) -> RenderInputs {
    let commands = map_render_commands(&mut ecs.resources);

    let views = ecs
        .resources
        .res::<nightshade_render::contract::RenderViews>();
    let cameras: Vec<CameraFrameInputs> = views.dispatched.clone();
    let active_camera_frame = views.active.clone();
    let active_camera = active_camera_frame.as_ref().map(|frame| frame.entity);
    let active_camera_target = active_camera_frame
        .as_ref()
        .and_then(|frame| frame.target)
        .map(|target| target.size);
    let fallback_shading = views.fallback_shading;

    let mut wants_normals = ecs
        .resources
        .res::<nightshade_render::data::config::DebugDraw>()
        .show_normals;
    let mut wants_bounds = ecs
        .resources
        .res::<nightshade_render::data::config::DebugDraw>()
        .show_bounding_volumes;
    let mut wants_wireframe = false;
    for camera in cameras.iter().chain(active_camera_frame.iter()) {
        wants_normals |= camera.effective_shading.show_normals;
        wants_bounds |= camera.effective_shading.show_bounding_volumes;
        wants_wireframe |= camera.effective_shading.show_wireframe;
    }
    let debug_lines = LinesFrameInputs {
        lines: nightshade_graphics::queries::lines::gather_lines_data(
            &ecs.storage,
            ecs.resources.res(),
            ecs.resources.res(),
            wants_wireframe,
        ),
        bounding_volumes: nightshade_graphics::queries::lines::gather_bounding_volume_data(
            &ecs.storage,
            ecs.resources.res(),
            ecs.resources.res(),
            ecs.resources.res(),
            ecs.resources.res(),
            wants_bounds,
        ),
        normals: nightshade_graphics::queries::lines::gather_normal_data(
            &ecs.storage,
            ecs.resources.res(),
            ecs.resources.res(),
            ecs.resources.res(),
            wants_normals,
        ),
    };

    let pick_request = ecs
        .resources
        .res_mut::<nightshade_render::resources::gpu_picking::GpuPicking>()
        .take_pending_request()
        .map(|request| (request.screen_x, request.screen_y));
    let has_skinned_meshes = ecs
        .storage
        .query_entities::<(nightshade_render::components::skin::Skin,)>()
        .next()
        .is_some();

    let dirty_world_spheres: Vec<(nalgebra_glm::Vec3, f32)> = {
        let dirty: Vec<nightshade::ecs::Entity> = mesh_state_dirty_for_culling(
            ecs.resources
                .res::<nightshade_render::resources::mesh_state::MeshRenderState>(),
        )
        .collect();
        let mut spheres = Vec::new();
        for entity in dirty {
            let Some(global_transform) = ecs
                .storage
                .get::<nightshade_render::components::render_world::Transform>(entity)
                .map(|transform| transform.0)
                .or_else(|| {
                    ecs.resources
                        .res::<nightshade_render::data::config::RendererState>()
                        .render_skinned_dynamic_state
                        .get(&entity)
                        .map(|dynamic| dynamic.transform)
                })
            else {
                continue;
            };
            let Some(bounding_volume) = ecs
                .storage
                .get::<nightshade_render::data::config::RenderBounds>(entity)
                .copied()
            else {
                continue;
            };
            let world_center = global_transform
                * nalgebra_glm::Vec4::new(
                    bounding_volume.center.x,
                    bounding_volume.center.y,
                    bounding_volume.center.z,
                    1.0,
                );
            let world_center =
                nalgebra_glm::Vec3::new(world_center.x, world_center.y, world_center.z);
            let scale = nalgebra_glm::length(&nalgebra_glm::Vec3::new(
                global_transform[(0, 0)],
                global_transform[(1, 0)],
                global_transform[(2, 0)],
            ));
            let world_radius = bounding_volume.sphere_radius * scale;
            spheres.push((world_center, world_radius));
        }
        spheres
    };
    let global_dirty_signal = mesh_state_needs_full_invalidation(
        ecs.resources
            .res::<nightshade_render::resources::mesh_state::MeshRenderState>(),
    );
    let mesh_frame_state = Some(mesh_state_take_frame(
        ecs.resources
            .res_mut::<nightshade_render::resources::mesh_state::MeshRenderState>(),
    ));
    let mouse_position = ecs
        .resources
        .res::<nightshade_platform::contract::input::Input>()
        .mouse
        .position;
    let force_render_cameras = ecs
        .resources
        .res::<nightshade_render::resources::viewport::Viewport>()
        .force_render_cameras
        .iter()
        .copied()
        .collect();

    let debug_draw = std::mem::take(
        ecs.resources
            .res_mut::<nightshade_render::data::config::DebugDraw>(),
    );
    let settings = std::mem::take(
        ecs.resources
            .res_mut::<nightshade_render::data::config::RenderSettings>(),
    );
    let window_uptime_milliseconds = ecs
        .resources
        .res::<nightshade_platform::contract::time::Time>()
        .uptime_milliseconds;
    let window_delta_time = ecs
        .resources
        .res::<nightshade_platform::contract::time::Time>()
        .delta_time;
    let window_cached_viewport_size = ecs
        .resources
        .res::<nightshade_platform::contract::window::Window>()
        .cached_viewport_size;
    let active_camera_target_size =
        active_camera_target.filter(|(width, height)| *width > 0 && *height > 0);
    let window_camera_tile_rects: std::collections::HashMap<_, _> = ecs
        .resources
        .res::<nightshade_render::resources::viewport::Viewport>()
        .camera_tile_rects
        .iter()
        .map(|(entity, rect)| (*entity, *rect))
        .collect();
    let window_camera_tile_render_iteration = ecs
        .resources
        .res::<nightshade_render::resources::viewport::Viewport>()
        .camera_tile_render_iteration;
    let window_active_viewport_rect = ecs
        .resources
        .res::<nightshade_render::resources::viewport::Viewport>()
        .active_viewport_rect;
    let scene = std::mem::take(
        ecs.resources
            .res_mut::<nightshade_render::data::config::RendererState>(),
    );
    let mesh_cache = std::mem::take(
        ecs.resources
            .res_mut::<nightshade_render::resources::mesh_cache::MeshCache>(),
    );
    let texture_cache = std::mem::take(
        ecs.resources
            .res_mut::<nightshade_render::resources::texture_cache::TextureCache>(),
    );
    let background_color = settings.swapchain_clear_color.map(nalgebra_glm::Vec4::from);

    let vsync = ecs
        .resources
        .res::<nightshade_render::data::config::FramePacing>()
        .vsync_enabled;
    let scene_world = ecs;

    RenderInputs {
        hdr_skybox_data: Vec::new(),
        dirty_meshes: Default::default(),
        dirty_skinned_meshes: Default::default(),
        scene_world,
        settings,
        vsync,
        debug_draw,
        scene,
        mesh_cache,
        texture_cache,
        owned: Default::default(),
        view: ViewInputs {
            active_camera,
            uptime_milliseconds: window_uptime_milliseconds,
            delta_time: window_delta_time,
            viewport_size: window_cached_viewport_size,
            scene_target_size: active_camera_target_size,
            camera_tile_rects: window_camera_tile_rects,
            camera_tile_render_iteration: window_camera_tile_render_iteration,
            active_viewport_rect: window_active_viewport_rect,
        },
        frame: crate::renderer::wgpu::render_configs::FrameInputs {
            commands,
            cameras,
            active_camera_frame,
            fallback_shading,
            pick_request,
            has_skinned_meshes,
            dirty_world_spheres,
            global_dirty_signal,
            mouse_position,
            force_render_cameras,
            debug_lines,
            background_clear_color: background_color,
        },
        mesh_frame_state,
    }
}

pub fn restore_render_inputs(
    inputs: RenderInputs,
    outputs: FrameOutputs,
) -> nightshade::ecs::World {
    let mut ecs = inputs.scene_world;

    *ecs.resources
        .res_mut::<nightshade_render::data::config::DebugDraw>() = inputs.debug_draw;
    *ecs.resources
        .res_mut::<nightshade_render::data::config::RenderSettings>() = inputs.settings;
    for entity in outputs.force_render_cleared {
        ecs.resources
            .res_mut::<nightshade_render::resources::viewport::Viewport>()
            .force_render_cameras
            .remove(&entity);
    }
    *ecs.resources
        .res_mut::<nightshade_render::data::config::RendererState>() = inputs.scene;
    *ecs.resources
        .res_mut::<nightshade_render::resources::mesh_cache::MeshCache>() = inputs.mesh_cache;
    *ecs.resources
        .res_mut::<nightshade_render::resources::texture_cache::TextureCache>() =
        inputs.texture_cache;
    if outputs.frame_executed {
        ecs.resources
            .res_mut::<nightshade_render::resources::viewport::Viewport>()
            .viewport_texture_sizes = outputs.viewport_texture_sizes;
    }
    if let Some(unconsumed) = inputs.mesh_frame_state {
        mesh_state_restore_unconsumed(
            ecs.resources
                .res_mut::<nightshade_render::resources::mesh_state::MeshRenderState>(),
            unconsumed,
        );
    }
    if let Some((screen_x, screen_y)) = inputs.frame.pick_request
        && ecs
            .resources
            .res::<nightshade_render::resources::gpu_picking::GpuPicking>()
            .pending_request
            .is_none()
    {
        ecs.resources
            .res_mut::<nightshade_render::resources::gpu_picking::GpuPicking>()
            .pending_request =
            Some(nightshade_render::resources::gpu_picking::GpuPickRequest { screen_x, screen_y });
    }

    ecs
}
