use crate::ecs::world::{CORE, RENDER, World};
#[cfg(feature = "terrain")]
use crate::render::terrain_config::TerrainSettings;
use crate::render::wgpu::render_configs::{
    CameraFrameInputs, CameraMatricesInputs, CameraProjectionParams, FrameOutputs,
    LinesFrameInputs, RenderInputs, RendererCommand, UiFrameInputs, ViewInputs,
};

fn snapshot_camera(world: &World, entity: nightshade_ecs::Entity) -> CameraFrameInputs {
    let matrices =
        crate::ecs::camera::queries::query_camera_matrices(world, entity).map(|matrices| {
            CameraMatricesInputs {
                view: matrices.view,
                projection: matrices.projection,
                camera_position: matrices.camera_position,
            }
        });
    // Mirrors the precedence in `query_camera_matrices`: an override supplies
    // both the matrix and the scalars the passes need alongside it.
    let projection = if let Some(override_projection) =
        world.get::<crate::ecs::camera::components::ProjectionOverride>(entity)
    {
        CameraProjectionParams {
            z_near: override_projection.z_near,
            z_far: override_projection.z_far,
            y_fov_rad: override_projection.y_fov_rad,
            aspect: override_projection.aspect,
            orthographic: None,
        }
    } else {
        match world
            .get::<crate::ecs::camera::components::Camera>(entity)
            .map(|c| &c.projection)
        {
            Some(crate::ecs::camera::components::Projection::Perspective(persp)) => {
                CameraProjectionParams {
                    z_near: persp.z_near,
                    z_far: persp.z_far.unwrap_or(1000.0),
                    y_fov_rad: persp.y_fov_rad,
                    aspect: persp.aspect_ratio.unwrap_or_else(|| {
                        crate::ecs::camera::queries::query_window_aspect_ratio(world)
                            .unwrap_or(1.78)
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
        }
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
        world.res::<crate::render::config::RenderSettings>(),
        world.res::<crate::render::config::DebugDraw>(),
        world.res::<crate::ecs::graphics::selection::Selection>(),
        &shading,
        post_process.as_ref(),
        culling_mask.as_ref(),
        environment.as_ref(),
    );
    CameraFrameInputs {
        entity,
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
    std::mem::take(
        &mut world
            .res_mut::<crate::ecs::world::commands::CommandQueues>()
            .render,
    )
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
                Ok(bytes) => Some(RendererCommand::LoadHdrSkybox { hdr_data: bytes }),
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
    let image_uploads = std::mem::take(
        &mut world
            .res_mut::<crate::ui::resources::RetainedUiRuntime>()
            .pending_image_uploads,
    );
    for upload in image_uploads {
        world
            .res_mut::<crate::ecs::world::commands::CommandQueues>()
            .render
            .push(crate::ecs::world::RenderCommand::UploadUiImageLayer {
                layer: upload.layer,
                rgba_data: upload.rgba_data,
                width: upload.width,
                height: upload.height,
            });
    }
    let commands = map_render_commands(world);

    let cameras: Vec<CameraFrameInputs> = world
        .res::<crate::ui::UserInterface>()
        .required_cameras
        .iter()
        .map(|&entity| snapshot_camera(world, entity))
        .collect();
    let active_camera = world.res::<crate::ecs::camera::resources::ActiveCamera>().0;
    let active_camera_frame = active_camera.map(|entity| snapshot_camera(world, entity));
    let fallback_shading = crate::ecs::camera::components::effective_shading_from_graphics(
        world.res::<crate::render::config::RenderSettings>(),
        world.res::<crate::render::config::DebugDraw>(),
        world.res::<crate::ecs::graphics::selection::Selection>(),
    );

    let mut wants_normals = world.res::<crate::render::config::DebugDraw>().show_normals;
    let mut wants_bounds = world
        .res::<crate::render::config::DebugDraw>()
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
        .res_mut::<crate::ecs::gpu_picking::GpuPicking>()
        .take_pending_request()
        .map(|request| (request.screen_x, request.screen_y));
    let has_skinned_meshes = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::SKIN)
        .next()
        .is_some();

    let dirty_world_spheres: Vec<(nalgebra_glm::Vec3, f32)> = {
        let dirty: Vec<nightshade_ecs::Entity> = world
            .res::<crate::render::mesh_state::MeshRenderState>()
            .dirty_entities_for_culling()
            .collect();
        let mut spheres = Vec::new();
        for entity in dirty {
            let Some(global_transform) = world.ecs.worlds[RENDER]
                .get::<crate::render::render_world::Transform>(entity)
                .map(|transform| transform.0)
                .or_else(|| {
                    world
                        .res::<crate::render::config::RendererState>()
                        .render_skinned_dynamic_state
                        .get(&entity)
                        .map(|dynamic| dynamic.transform)
                })
            else {
                continue;
            };
            let Some(bounding_volume) = world.ecs.worlds[RENDER]
                .get::<crate::render::config::RenderBounds>(entity)
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
    let global_dirty_signal = world
        .res::<crate::render::mesh_state::MeshRenderState>()
        .requires_full_invalidation();
    let mesh_frame_state = Some(
        world
            .res_mut::<crate::render::mesh_state::MeshRenderState>()
            .take_frame_state(),
    );
    let mouse_position = world
        .res::<crate::platform::input::resources::Input>()
        .mouse
        .position;
    let force_render_cameras = world
        .res::<crate::viewport::Viewport>()
        .force_render_cameras
        .iter()
        .copied()
        .collect();

    #[cfg(feature = "grass")]
    let grass = std::mem::take(world.res_mut::<crate::render::grass_config::GrassSettings>());
    #[cfg(feature = "terrain")]
    let terrain = world
        .ecs
        .resource_mut::<TerrainSettings>()
        .map(std::mem::take)
        .unwrap_or_default();
    #[cfg(feature = "terrain")]
    let terrain_render =
        std::mem::take(world.res_mut::<crate::render::terrain_config::TerrainRenderState>());

    let wind = *world.res::<crate::render::wind::Wind>();
    let debug_draw = std::mem::take(world.res_mut::<crate::render::config::DebugDraw>());
    let settings = std::mem::take(world.res_mut::<crate::render::config::RenderSettings>());
    let window_uptime_milliseconds = world.res::<crate::ecs::time::Time>().uptime_milliseconds;
    let window_delta_time = world.res::<crate::ecs::time::Time>().delta_time;
    let window_cached_viewport_size = world
        .res::<crate::platform::window::Window>()
        .cached_viewport_size;
    let window_camera_tile_rects: std::collections::HashMap<_, _> = world
        .res::<crate::viewport::Viewport>()
        .camera_tile_rects
        .iter()
        .map(|(entity, rect)| (*entity, *rect))
        .collect();
    let window_camera_tile_render_iteration = world
        .res::<crate::viewport::Viewport>()
        .camera_tile_render_iteration;
    let window_active_viewport_rect = world
        .res::<crate::viewport::Viewport>()
        .active_viewport_rect;
    let loading_particle_textures = std::mem::take(
        &mut world
            .res_mut::<crate::ecs::loading::LoadingState>()
            .pending_particle_textures,
    );
    let scene = std::mem::take(world.res_mut::<crate::render::config::RendererState>());
    let scene_world = std::mem::replace(
        &mut world.ecs.worlds[RENDER],
        nightshade_ecs::dynamic::DynWorld::new(),
    );
    let mesh_cache = std::mem::take(world.res_mut::<crate::render::mesh_cache::MeshCache>());
    let world_id = world.res::<crate::ecs::world::WorldId>().0;
    let texture_cache =
        std::mem::take(world.res_mut::<crate::render::wgpu::texture_cache::TextureCache>());
    #[cfg(feature = "egui")]
    let egui_frame_inputs = crate::egui::take_egui_frame_inputs(world);
    let background_color = world
        .res::<crate::ui::resources::RetainedUiRuntime>()
        .background_color;
    let render_slots = std::mem::take(world.res_mut::<crate::ui::resources::RenderSlotAllocator>());
    let frame = world.res_mut::<crate::ui::resources::RetainedUiFrame>();
    RenderInputs {
        scene_world,
        settings,
        debug_draw,
        scene,
        ibl_views: Default::default(),
        shadow_atlas: Default::default(),
        wind,
        mesh_cache,
        texture_cache,
        texture_store: Default::default(),
        pending_particle_textures: loading_particle_textures,
        ui: UiFrameInputs {
            rects: std::mem::take(&mut frame.rects),
            rect_entities: std::mem::take(&mut frame.rect_entities)
                .into_iter()
                .collect(),
            images: std::mem::take(&mut frame.ui_images),
            text_meshes: std::mem::take(&mut frame.text_meshes),
            render_slots,
            background_color,
        },
        view: ViewInputs {
            active_camera,
            uptime_milliseconds: window_uptime_milliseconds,
            delta_time: window_delta_time,
            viewport_size: window_cached_viewport_size,
            camera_tile_rects: window_camera_tile_rects,
            camera_tile_render_iteration: window_camera_tile_render_iteration,
            active_viewport_rect: window_active_viewport_rect,
        },
        world_id,
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
            #[cfg(feature = "egui")]
            egui: egui_frame_inputs,
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
        if let Some(terrain) = world.ecs.resource_mut::<TerrainSettings>() {
            *terrain = inputs.terrain;
        }
        *world.res_mut::<crate::render::terrain_config::TerrainRenderState>() =
            inputs.terrain_render;
    }
    *world.res_mut::<crate::render::config::DebugDraw>() = inputs.debug_draw;
    *world.res_mut::<crate::render::config::RenderSettings>() = inputs.settings;
    for entity in outputs.force_render_cleared {
        world
            .res_mut::<crate::viewport::Viewport>()
            .force_render_cameras
            .remove(&entity);
    }
    let mut pending = inputs.pending_particle_textures;
    pending.extend(std::mem::take(
        &mut world
            .res_mut::<crate::ecs::loading::LoadingState>()
            .pending_particle_textures,
    ));
    world
        .res_mut::<crate::ecs::loading::LoadingState>()
        .pending_particle_textures = pending;
    #[cfg(feature = "grass")]
    {
        *world.res_mut::<crate::render::grass_config::GrassSettings>() = inputs.grass;
    }
    world.ecs.worlds[RENDER] = inputs.scene_world;
    *world.res_mut::<crate::render::config::RendererState>() = inputs.scene;
    *world.res_mut::<crate::render::mesh_cache::MeshCache>() = inputs.mesh_cache;
    *world.res_mut::<crate::render::wgpu::texture_cache::TextureCache>() = inputs.texture_cache;
    *world.res_mut::<crate::ui::resources::RenderSlotAllocator>() = inputs.ui.render_slots;
    let frame = world.res_mut::<crate::ui::resources::RetainedUiFrame>();
    frame.rects = inputs.ui.rects;
    frame.rect_entities = inputs.ui.rect_entities.into_iter().collect();
    frame.ui_images = inputs.ui.images;
    frame.text_meshes = inputs.ui.text_meshes;
    if outputs.frame_executed {
        world
            .res_mut::<crate::ui::UserInterface>()
            .viewport_texture_sizes = outputs.viewport_texture_sizes;
    }
    if let Some(unconsumed) = inputs.frame.mesh_frame_state {
        world
            .res_mut::<crate::render::mesh_state::MeshRenderState>()
            .restore_unconsumed(unconsumed);
    }
    if let Some((screen_x, screen_y)) = inputs.frame.pick_request
        && world
            .res::<crate::ecs::gpu_picking::GpuPicking>()
            .pending_request
            .is_none()
    {
        world
            .res_mut::<crate::ecs::gpu_picking::GpuPicking>()
            .pending_request = Some(crate::ecs::gpu_picking::GpuPickRequest { screen_x, screen_y });
    }
}
