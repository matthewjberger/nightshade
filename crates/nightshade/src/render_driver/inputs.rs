use crate::ecs::world::CORE;
use crate::ecs::world::World;
#[cfg(feature = "terrain")]
use crate::render::terrain_config::TerrainSettings;
use crate::render::wgpu::render_configs::{
    CameraFrameInputs, CameraMatricesInputs, CameraProjectionParams, FrameOutputs,
    LinesFrameInputs, RenderInputs, RendererCommand, UiFrameInputs, ViewInputs,
};
use crate::render_driver::{render_entity, scene_entity};

fn snapshot_camera(world: &World, entity: freecs::Entity) -> CameraFrameInputs {
    let matrices =
        crate::ecs::camera::queries::query_camera_matrices(world, entity).map(|matrices| {
            CameraMatricesInputs {
                view: matrices.view,
                projection: matrices.projection,
                camera_position: matrices.camera_position,
            }
        });
    let projection = match world
        .get::<crate::ecs::camera::components::Camera>(entity)
        .map(|c| &c.projection)
    {
        Some(crate::ecs::camera::components::Projection::Perspective(persp)) => {
            CameraProjectionParams {
                z_near: persp.z_near,
                z_far: persp.z_far.unwrap_or(1000.0),
                y_fov_rad: persp.y_fov_rad,
                aspect: persp.aspect_ratio.unwrap_or_else(|| {
                    crate::ecs::camera::queries::query_window_aspect_ratio(world).unwrap_or(1.78)
                }),
                orthographic: None,
            }
        }
        Some(crate::ecs::camera::components::Projection::Orthographic(ortho)) => {
            CameraProjectionParams {
                z_near: ortho.z_near,
                z_far: ortho.z_far,
                y_fov_rad: std::f32::consts::FRAC_PI_4,
                aspect: 1.78,
                orthographic: Some((ortho.x_mag, ortho.y_mag)),
            }
        }
        None => CameraProjectionParams::default(),
    };
    let shading = world
        .get::<crate::ecs::camera::components::ViewportShading>(entity)
        .copied()
        .unwrap_or_default();
    let post_process = world
        .get::<crate::ecs::camera::components::CameraPostProcess>(entity)
        .copied();
    let culling_mask = world
        .get::<crate::ecs::primitives::CameraCullingMask>(entity)
        .copied();
    let environment = world
        .get::<crate::ecs::camera::components::CameraEnvironment>(entity)
        .copied();
    let effective_shading = crate::ecs::camera::components::effective_shading_for_camera(
        world.expect_resource::<crate::render::config::RenderSettings>(),
        world.expect_resource::<crate::render::config::DebugDraw>(),
        world.expect_resource::<crate::ecs::graphics::selection::Selection>(),
        &shading,
        post_process.as_ref(),
        culling_mask.as_ref(),
        environment.as_ref(),
    );
    CameraFrameInputs {
        entity: render_entity(entity),
        matrices,
        projection,
        constrained_aspect: world
            .get::<crate::ecs::camera::components::ConstrainedAspect>(entity)
            .map(|constrained| constrained.0),
        effective_shading,
        update_mode: world
            .get::<crate::render::config::ViewportUpdateMode>(entity)
            .copied()
            .unwrap_or_default(),
        world_transform: world
            .get::<crate::ecs::transform::components::GlobalTransform>(entity)
            .map(|gt| gt.0),
    }
}

fn map_render_commands(world: &mut World) -> Vec<RendererCommand> {
    std::mem::take(&mut world.resources.commands.render)
        .into_iter()
        .filter_map(|command| match command {
            crate::ecs::world::RenderCommand::UploadUiImageLayer {
                layer,
                rgba_data,
                width,
                height,
            } => Some(RendererCommand::UploadUiImageLayer {
                layer,
                rgba_data,
                width,
                height,
            }),
            crate::ecs::world::RenderCommand::ReloadTexture {
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
            crate::ecs::world::RenderCommand::LoadHdrSkybox { hdr_data } => {
                Some(RendererCommand::LoadHdrSkybox { hdr_data })
            }
            crate::ecs::world::RenderCommand::SetColorLut { data } => {
                Some(RendererCommand::SetColorLut { data })
            }
            crate::ecs::world::RenderCommand::LoadHdrSkyboxFromPath { path } => {
                match std::fs::read(&path) {
                    Ok(bytes) => {
                        tracing::info!("Successfully read HDR file: {}", path.display());
                        Some(RendererCommand::LoadHdrSkybox { hdr_data: bytes })
                    }
                    Err(error) => {
                        tracing::error!("Failed to read HDR file {}: {}", path.display(), error);
                        None
                    }
                }
            }
            crate::ecs::world::RenderCommand::CaptureScreenshot {
                path,
                max_dimension,
            } => Some(RendererCommand::CaptureScreenshot {
                path,
                max_dimension,
            }),
        })
        .collect()
}

/// Composes the renderer's per-frame input by snapshotting everything the
/// frame needs from the world, then moving the large state out of the world.
///
/// Every non-`Copy` input is taken with `std::mem::take`, not cloned, so
/// handing the renderer its inputs costs nothing regardless of scene size.
/// The frame itself never sees the world: anything it needs is captured
/// here, and anything it produces for the world comes back through
/// [`FrameOutputs`] in [`restore_render_inputs`]. The `ibl_views` and
/// `shadow_atlas` fields are renderer-owned and composed empty; the frame
/// driver fills them itself.
pub fn compose_render_inputs(world: &mut World) -> RenderInputs {
    let commands = map_render_commands(world);

    let cameras: Vec<CameraFrameInputs> = world
        .resources
        .user_interface
        .required_cameras
        .iter()
        .map(|&entity| snapshot_camera(world, entity))
        .collect();
    let active_camera = world
        .expect_resource::<crate::ecs::camera::resources::ActiveCamera>()
        .0;
    let active_camera_frame = active_camera.map(|entity| snapshot_camera(world, entity));
    let fallback_shading = crate::ecs::camera::components::effective_shading_from_graphics(
        world.expect_resource::<crate::render::config::RenderSettings>(),
        world.expect_resource::<crate::render::config::DebugDraw>(),
        world.expect_resource::<crate::ecs::graphics::selection::Selection>(),
    );

    let mut wants_normals = world
        .expect_resource::<crate::render::config::DebugDraw>()
        .show_normals;
    let mut wants_bounds = world
        .expect_resource::<crate::render::config::DebugDraw>()
        .show_bounding_volumes;
    let mut wants_wireframe = false;
    for camera in cameras.iter().chain(active_camera_frame.iter()) {
        wants_normals |= camera.effective_shading.show_normals;
        wants_bounds |= camera.effective_shading.show_bounding_volumes;
        wants_wireframe |= camera.effective_shading.show_wireframe;
    }
    let debug_lines = LinesFrameInputs {
        lines: super::lines::gather_lines_data(world, wants_wireframe),
        bounding_volumes: super::lines::gather_bounding_volume_data(world, wants_bounds),
        normals: super::lines::gather_normal_data(world, wants_normals),
    };

    let pick_request = world
        .expect_resource_mut::<crate::ecs::gpu_picking::GpuPicking>()
        .take_pending_request()
        .map(|request| (request.screen_x, request.screen_y));
    let has_skinned_meshes = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::SKIN)
        .next()
        .is_some();

    let dirty_world_spheres: Vec<(nalgebra_glm::Vec3, f32)> = {
        let scene = world.expect_resource::<crate::render::config::RendererState>();
        let mut spheres = Vec::new();
        for entity in world
            .resources
            .mesh_render_state
            .dirty_entities_for_culling()
        {
            let Some(dynamic) = scene.render_dynamic_objects.get(&entity) else {
                continue;
            };
            let Some(bounding_volume) = scene.render_bounds.get(&entity) else {
                continue;
            };
            let global_transform = dynamic.transform;
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
    let global_dirty_signal = world
        .resources
        .mesh_render_state
        .requires_full_invalidation();
    let mesh_frame_state = Some(world.resources.mesh_render_state.take_frame_state());
    let mouse_position = world
        .expect_resource_mut::<crate::ecs::input::resources::Input>()
        .mouse
        .position;
    let force_render_cameras = world
        .expect_resource_mut::<crate::ecs::window::resources::Window>()
        .force_render_cameras
        .iter()
        .copied()
        .map(render_entity)
        .collect();

    #[cfg(feature = "grass")]
    let grass =
        std::mem::take(world.expect_resource_mut::<crate::render::grass_config::GrassSettings>());
    #[cfg(feature = "terrain")]
    let terrain = std::mem::take(world.plugin_resource_mut::<TerrainSettings>());
    #[cfg(feature = "terrain")]
    let terrain_render = std::mem::take(&mut world.resources.terrain_render);

    let wind = *world.expect_resource::<crate::render::wind::Wind>();
    let debug_draw =
        std::mem::take(world.expect_resource_mut::<crate::render::config::DebugDraw>());
    let settings =
        std::mem::take(world.expect_resource_mut::<crate::render::config::RenderSettings>());
    let window_uptime_milliseconds = world
        .expect_resource_mut::<crate::ecs::time::Time>()
        .uptime_milliseconds;
    let window_delta_time = world
        .expect_resource_mut::<crate::ecs::time::Time>()
        .delta_time;
    let window_cached_viewport_size = world
        .expect_resource_mut::<crate::ecs::window::resources::Window>()
        .cached_viewport_size;
    let window_camera_tile_rects: std::collections::HashMap<_, _> = world
        .expect_resource_mut::<crate::ecs::window::resources::Window>()
        .camera_tile_rects
        .iter()
        .map(|(entity, rect)| (render_entity(*entity), *rect))
        .collect();
    let window_camera_tile_render_iteration = world
        .expect_resource_mut::<crate::ecs::window::resources::Window>()
        .camera_tile_render_iteration;
    let window_active_viewport_rect = world
        .expect_resource_mut::<crate::ecs::window::resources::Window>()
        .active_viewport_rect;
    let loading_particle_textures = std::mem::take(
        &mut world
            .expect_resource_mut::<crate::ecs::loading::LoadingState>()
            .pending_particle_textures,
    );
    let scene = std::mem::take(world.expect_resource_mut::<crate::render::config::RendererState>());
    let mesh_cache = std::mem::take(
        &mut world
            .expect_resource_mut::<crate::ecs::asset_state::AssetState>()
            .mesh_cache,
    );
    let resources = &mut world.resources;
    let retained = &mut resources.retained_ui;
    RenderInputs {
        settings,
        debug_draw,
        scene,
        ibl_views: Default::default(),
        shadow_atlas: Default::default(),
        wind,
        mesh_cache,
        texture_cache: std::mem::take(&mut resources.texture_cache),
        pending_particle_textures: loading_particle_textures,
        ui: UiFrameInputs {
            rects: std::mem::take(&mut retained.frame.rects),
            rect_entities: std::mem::take(&mut retained.frame.rect_entities)
                .into_iter()
                .map(|entity| entity.map(render_entity))
                .collect(),
            images: std::mem::take(&mut retained.frame.ui_images),
            text_meshes: std::mem::take(&mut retained.frame.text_meshes),
            render_slots: std::mem::take(&mut retained.render_slots),
            background_color: retained.background_color,
        },
        view: ViewInputs {
            active_camera: active_camera.map(render_entity),
            uptime_milliseconds: window_uptime_milliseconds,
            delta_time: window_delta_time,
            viewport_size: window_cached_viewport_size,
            camera_tile_rects: window_camera_tile_rects,
            camera_tile_render_iteration: window_camera_tile_render_iteration,
            active_viewport_rect: window_active_viewport_rect,
        },
        world_id: resources.world_id,
        frame: crate::render::wgpu::render_configs::FrameInputs {
            commands,
            cameras,
            active_camera_frame,
            fallback_shading,
            pick_request,
            has_skinned_meshes,
            mesh_frame_state,
            dirty_world_spheres,
            global_dirty_signal,
            mouse_position,
            force_render_cameras,
            debug_lines,
        },
        #[cfg(feature = "grass")]
        grass,
        #[cfg(feature = "terrain")]
        terrain,
        #[cfg(feature = "terrain")]
        terrain_render,
    }
}

/// Moves the state carried by [`RenderInputs`] back into the world and
/// applies the frame's [`FrameOutputs`]: the viewport texture sizes, the
/// force-render clears, and any pick request the frame did not consume.
/// Everything taken is a plain move back, except
/// `loading.pending_particle_textures`: the taken contents come back first
/// and anything the engine queued mid-frame is appended after them,
/// preserving the upload cursor the particle pass keeps into that list.
pub fn restore_render_inputs(world: &mut World, inputs: RenderInputs, outputs: FrameOutputs) {
    #[cfg(feature = "terrain")]
    {
        *world.plugin_resource_mut::<TerrainSettings>() = inputs.terrain;
        world.resources.terrain_render = inputs.terrain_render;
    }
    *world.expect_resource_mut::<crate::render::config::DebugDraw>() = inputs.debug_draw;
    *world.expect_resource_mut::<crate::render::config::RenderSettings>() = inputs.settings;
    for entity in outputs.force_render_cleared {
        world
            .expect_resource_mut::<crate::ecs::window::resources::Window>()
            .force_render_cameras
            .remove(&scene_entity(entity));
    }
    let mut pending = inputs.pending_particle_textures;
    pending.extend(std::mem::take(
        &mut world
            .expect_resource_mut::<crate::ecs::loading::LoadingState>()
            .pending_particle_textures,
    ));
    world
        .expect_resource_mut::<crate::ecs::loading::LoadingState>()
        .pending_particle_textures = pending;
    #[cfg(feature = "grass")]
    {
        *world.expect_resource_mut::<crate::render::grass_config::GrassSettings>() = inputs.grass;
    }
    *world.expect_resource_mut::<crate::render::config::RendererState>() = inputs.scene;
    world
        .expect_resource_mut::<crate::ecs::asset_state::AssetState>()
        .mesh_cache = inputs.mesh_cache;
    let resources = &mut world.resources;
    resources.texture_cache = inputs.texture_cache;
    let retained = &mut resources.retained_ui;
    retained.frame.rects = inputs.ui.rects;
    retained.frame.rect_entities = inputs
        .ui
        .rect_entities
        .into_iter()
        .map(|entity| entity.map(scene_entity))
        .collect();
    retained.frame.ui_images = inputs.ui.images;
    retained.frame.text_meshes = inputs.ui.text_meshes;
    retained.render_slots = inputs.ui.render_slots;
    if outputs.frame_executed {
        resources.user_interface.viewport_texture_sizes = outputs.viewport_texture_sizes;
    }
    if let Some(unconsumed) = inputs.frame.mesh_frame_state {
        resources.mesh_render_state.restore_unconsumed(unconsumed);
    }
    if let Some((screen_x, screen_y)) = inputs.frame.pick_request
        && world
            .expect_resource::<crate::ecs::gpu_picking::GpuPicking>()
            .pending_request
            .is_none()
    {
        world
            .expect_resource_mut::<crate::ecs::gpu_picking::GpuPicking>()
            .pending_request = Some(crate::ecs::gpu_picking::GpuPickRequest { screen_x, screen_y });
    }
}
