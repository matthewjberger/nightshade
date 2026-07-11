use crate::ecs::loading::{
    loading_pipeline_is_active, loading_pipeline_mark_completed, loading_pipeline_poll_decodes,
    loading_pipeline_pop, loading_pipeline_recipe_ready, loading_pipeline_release_recipe_sources,
    loading_pipeline_requeue_front, loading_pipeline_reset_source_arena,
};
use crate::render::generational_registry::*;
use crate::render::wgpu::DEPTH_PICK_SAMPLE_SIZE;
use crate::render::wgpu::passes;
use crate::render::wgpu::rendergraph;
#[cfg(all(not(target_arch = "wasm32"), feature = "screenshot"))]
use crate::render::wgpu::rendergraph::render_graph_get_texture;
use crate::render::wgpu::rendergraph::{
    render_graph_add_pass, render_graph_compile, render_graph_enable_only_passes,
    render_graph_execute, render_graph_get_pass_mut, render_graph_get_texture_view,
    render_graph_restore_pass_enabled_states, render_graph_save_pass_enabled_states,
    render_graph_set_external_texture, render_graph_set_pass_enabled,
};

fn taa_halton(index: u32, base: u32) -> f32 {
    let mut result = 0.0;
    let mut fraction = 1.0;
    let mut current = index;
    while current > 0 {
        fraction /= base as f32;
        result += fraction * (current % base) as f32;
        current /= base;
    }
    result
}

fn fit_constrained_aspect(tile_width: u32, tile_height: u32, aspect: f32) -> (u32, u32) {
    let aspect = aspect.max(0.0001);
    let tile_aspect = tile_width as f32 / tile_height.max(1) as f32;
    if tile_aspect > aspect {
        let width = (tile_height as f32 * aspect).round() as u32;
        (width.max(1), tile_height.max(1))
    } else {
        let height = (tile_width as f32 / aspect).round() as u32;
        (tile_width.max(1), height.max(1))
    }
}

fn build_render_view(
    world: &crate::ecs::world::World,
    camera_entity: freecs::Entity,
    matrices: &crate::ecs::camera::queries::CameraMatrices,
    taa_jitter: [f32; 2],
    screen_size: (u32, u32),
) -> crate::render::config::RenderView {
    let view = matrices.view;
    let mut projection = matrices.projection;
    projection[(0, 2)] += taa_jitter[0];
    projection[(1, 2)] += taa_jitter[1];
    let view_projection = projection * view;
    let inverse_view = nalgebra_glm::inverse(&view);
    let inverse_projection = nalgebra_glm::inverse(&projection);
    let inverse_view_projection = nalgebra_glm::inverse(&view_projection);
    let camera_right = nalgebra_glm::vec3(
        inverse_view[(0, 0)],
        inverse_view[(1, 0)],
        inverse_view[(2, 0)],
    );
    let camera_up = nalgebra_glm::vec3(
        inverse_view[(0, 1)],
        inverse_view[(1, 1)],
        inverse_view[(2, 1)],
    );
    let frustum_planes = passes::geometry::extract_frustum_planes(&view_projection);
    let (z_near, z_far, y_fov_rad, aspect, orthographic) =
        match world.core.get_camera(camera_entity).map(|c| &c.projection) {
            Some(crate::ecs::camera::components::Projection::Perspective(persp)) => {
                let aspect = persp.aspect_ratio.unwrap_or_else(|| {
                    crate::ecs::camera::queries::query_window_aspect_ratio(world).unwrap_or(1.78)
                });
                (
                    persp.z_near,
                    persp.z_far.unwrap_or(1000.0),
                    persp.y_fov_rad,
                    aspect,
                    None,
                )
            }
            Some(crate::ecs::camera::components::Projection::Orthographic(ortho)) => (
                ortho.z_near,
                ortho.z_far,
                std::f32::consts::FRAC_PI_4,
                1.78,
                Some((ortho.x_mag, ortho.y_mag)),
            ),
            None => (0.1, 1000.0, std::f32::consts::FRAC_PI_4, 1.78, None),
        };
    crate::render::config::RenderView {
        view,
        projection,
        view_projection,
        inverse_view,
        inverse_projection,
        inverse_view_projection,
        camera_position: matrices.camera_position,
        camera_right,
        camera_up,
        frustum_planes,
        z_near,
        z_far,
        y_fov_rad,
        aspect,
        orthographic,
        screen_size,
        constrained_aspect: world
            .core
            .get_constrained_aspect(camera_entity)
            .map(|constrained| constrained.0),
    }
}

fn build_render_lighting(
    world: &crate::ecs::world::World,
    scene: &crate::render::config::RendererState,
) -> crate::render::config::RenderLighting {
    use super::lighting::{calculate_cascade_shadows, collect_area_lights, collect_lights};
    use crate::render::wgpu::passes::geometry::projection::MAX_AREA_LIGHTS;
    const MAX_LIGHTS: usize = 1024;
    let light_result = collect_lights(scene, MAX_LIGHTS);
    let directional = light_result.directional_light;
    let directional_light_direction = directional
        .as_ref()
        .map(|(_light, transform)| {
            let dir =
                crate::render::wgpu::passes::geometry::projection::transform_forward(transform);
            [dir.x, dir.y, dir.z, 0.0]
        })
        .unwrap_or([0.0, -1.0, 0.0, 0.0]);
    let cascade = calculate_cascade_shadows(world, directional.as_ref());
    let area = collect_area_lights(scene, MAX_AREA_LIGHTS);
    let mut sun = None;
    for render_light in &scene.render_lights.lights {
        let light = &render_light.light;
        let transform = &render_light.transform;
        if light.light_type == crate::render::config::RenderLightType::Directional {
            sun = Some((
                -crate::render::wgpu::passes::geometry::projection::transform_forward(transform)
                    .normalize(),
                light.color * light.intensity,
            ));
            break;
        }
    }
    let (sun_direction, sun_color) = sun.unwrap_or((
        nalgebra_glm::vec3(0.35, 0.8, 0.35).normalize(),
        nalgebra_glm::vec3(4.0, 3.8, 3.2),
    ));
    crate::render::config::RenderLighting {
        num_directional_lights: light_result.num_directional_lights,
        has_directional_light: directional.is_some(),
        entity_to_lights_index: light_result.entity_to_index,
        lights_data: light_result.lights_data,
        directional_light_direction,
        cascade_view_projections: cascade.cascade_view_projections,
        cascade_diameters: cascade.cascade_diameters,
        cascade_split_distances: cascade.cascade_split_distances,
        light_view_projection: cascade.light_view_projection,
        shadow_bias: cascade.shadow_bias,
        shadow_normal_bias: cascade.shadow_normal_bias,
        directional_light_size: cascade.light_size,
        shadows_enabled: cascade.shadows_enabled,
        area_lights_data: area.area_lights_data,
        area_entity_to_index: area.entity_to_index,
        sun_direction,
        sun_color,
    }
}

pub(crate) struct WindowDispatchContext<'a> {
    cameras: Vec<freecs::Entity>,
    target_size: (u32, u32),
    dirty_world_spheres: &'a [(nalgebra_glm::Vec3, f32)],
    frame_settings_version: u64,
    original_active_camera: Option<freecs::Entity>,
    focus_policy: crate::render::config::ViewportFocusPolicy,
    mouse_position: nalgebra_glm::Vec2,
    global_dirty_signal: bool,
}

pub fn render_frame(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    world: &mut crate::ecs::world::World,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut inputs = super::inputs::compose_render_inputs(world);
    let result = render_frame_inner(renderer, world, &mut inputs);
    super::inputs::restore_render_inputs(world, inputs);
    result
}

fn render_frame_inner(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    world: &mut crate::ecs::world::World,
    inputs: &mut crate::render::wgpu::render_configs::RenderInputs,
) -> Result<(), Box<dyn std::error::Error>> {
    let _span = tracing::info_span!("render_frame").entered();
    if inputs.scene.gpu_profile.is_none() {
        inputs.scene.gpu_profile = Some(renderer.gpu_profile.clone());
        if renderer.gpu_profile.backend == crate::render::config::GpuBackend::Metal {
            inputs.scene.occlusion_culling_enabled = false;
        }
    }
    renderer.frame_state.index = renderer.frame_state.index.wrapping_add(1);

    if inputs.settings.taa_enabled {
        let jitter_index = (renderer.frame_state.index % 8) as u32 + 1;
        let (render_width, render_height) = renderer.render_buffer_size;
        // Scale the projection jitter by the render scale so the sub-pixel
        // offsets distribute samples across the display-resolution grid the
        // resolve reconstructs, not the smaller render-buffer grid.
        let render_scale = inputs.settings.render_scale.clamp(0.25, 4.0);
        inputs.scene.taa_jitter = [
            (taa_halton(jitter_index, 2) - 0.5) * 2.0 * render_scale / render_width.max(1) as f32,
            (taa_halton(jitter_index, 3) - 0.5) * 2.0 * render_scale / render_height.max(1) as f32,
        ];
    } else {
        inputs.scene.taa_jitter = [0.0, 0.0];
    }

    if let Some(matrices) = crate::ecs::camera::queries::query_active_camera_matrices(world) {
        let view_projection: [[f32; 4]; 4] = (matrices.projection * matrices.view).into();
        inputs.scene.prev_view_projection = inputs.scene.view_projection;
        inputs.scene.view_projection = view_projection;
    }

    renderer.sync_present_mode(inputs.settings.vsync_enabled);
    let current_settings_signature = crate::render::config::render_settings_signature(
        &inputs.settings,
        &inputs.debug_draw,
        &inputs.editor_selection,
    );
    if renderer.frame_state.last_settings_signature != Some(current_settings_signature) {
        inputs.scene.settings_version = inputs.scene.settings_version.wrapping_add(1);
        renderer.frame_state.last_settings_signature = Some(current_settings_signature);
    }

    let frame_time_ms = world.resources.window.timing.delta_time * 1000.0;
    if frame_time_ms > 0.0 {
        inputs
            .scene
            .adaptive_sampling
            .record_frame_time(frame_time_ms);
    }

    #[cfg(feature = "egui")]
    super::egui::prepare_egui_pass(renderer, world);

    if renderer.depth_pick.pending {
        let _ = renderer.device.poll(wgpu::PollType::Poll);

        if renderer
            .depth_pick
            .map_complete
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            let buffer_slice = renderer.depth_pick.staging_buffer.slice(..);
            let data = buffer_slice.get_mapped_range();
            let mut depth_values = Vec::new();
            let mut entity_id_values = Vec::new();
            for chunk in data.chunks_exact(8) {
                let depth = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                let entity_id = u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
                depth_values.push(depth);
                entity_id_values.push(entity_id);
            }
            drop(data);
            renderer.depth_pick.staging_buffer.unmap();

            world.resources.gpu_picking.set_depth_samples(
                depth_values,
                entity_id_values,
                DEPTH_PICK_SAMPLE_SIZE,
                DEPTH_PICK_SAMPLE_SIZE,
                renderer.depth_pick.center.0,
                renderer.depth_pick.center.1,
            );

            if let Some(camera_entity) = renderer.depth_pick.camera
                && let Some(matrices) =
                    crate::ecs::camera::queries::query_camera_matrices(world, camera_entity)
            {
                let (texture_width, texture_height) = renderer.depth_pick.texture_size;
                let inverse_view_proj = (matrices.projection * matrices.view)
                    .try_inverse()
                    .unwrap_or_else(nalgebra_glm::Mat4::identity);

                world.resources.gpu_picking.compute_result(
                    &inverse_view_proj,
                    texture_width as f32,
                    texture_height as f32,
                );
            }

            renderer.depth_pick.pending = false;
            renderer
                .depth_pick
                .map_complete
                .store(false, std::sync::atomic::Ordering::Relaxed);
        }
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "screenshot"))]
    if renderer.screenshot.pending {
        let _ = renderer.device.poll(wgpu::PollType::Poll);

        if renderer
            .screenshot
            .map_complete
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            let buffer_slice = renderer.screenshot.staging_buffer.slice(..);
            let data = buffer_slice.get_mapped_range();

            let width = renderer.screenshot.width;
            let height = renderer.screenshot.height;
            let bytes_per_pixel = 4u32;
            let unpadded_bytes_per_row = width * bytes_per_pixel;
            let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;

            let mut pixels = Vec::with_capacity((width * height * bytes_per_pixel) as usize);
            for row in 0..height {
                let start = (row * padded_bytes_per_row) as usize;
                let end = start + (unpadded_bytes_per_row) as usize;
                pixels.extend_from_slice(&data[start..end]);
            }
            drop(data);
            renderer.screenshot.staging_buffer.unmap();

            if matches!(
                renderer.surface_config.format,
                wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
            ) {
                for pixel in pixels.chunks_exact_mut(4) {
                    pixel.swap(0, 2);
                }
            }

            let path = renderer.screenshot.path.take().unwrap_or_else(|| {
                let screenshots_dir = std::path::PathBuf::from("screenshots");
                if !screenshots_dir.exists() {
                    let _ = std::fs::create_dir_all(&screenshots_dir);
                }
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                screenshots_dir.join(format!("screenshot_{}.png", timestamp))
            });

            let save_width = width;
            let save_height = height;
            let max_dimension = renderer.screenshot.max_dimension.take();
            std::thread::spawn(move || {
                let Some(mut image) = image::RgbaImage::from_raw(save_width, save_height, pixels)
                else {
                    tracing::error!("Failed to create image from screenshot data");
                    return;
                };
                if let Some(limit) = max_dimension {
                    image = crop_center_square(image);
                    let (w, h) = image.dimensions();
                    let longest = w.max(h);
                    if longest > limit {
                        let scale = limit as f32 / longest as f32;
                        let new_w = ((w as f32 * scale).round() as u32).max(1);
                        let new_h = ((h as f32 * scale).round() as u32).max(1);
                        image = image::imageops::resize(
                            &image,
                            new_w,
                            new_h,
                            image::imageops::FilterType::Lanczos3,
                        );
                    }
                }
                if let Err(error) = image.save(&path) {
                    tracing::error!("Failed to save screenshot: {}", error);
                } else {
                    tracing::info!("Screenshot saved to: {}", path.display());
                }
            });

            renderer.screenshot.pending = false;
            renderer
                .screenshot
                .map_complete
                .store(false, std::sync::atomic::Ordering::Relaxed);
        }
    }

    renderer.resize_render_buffers(
        renderer.surface_config.width,
        renderer.surface_config.height,
    );

    let mut ui_image_uploads = Vec::new();
    let mut texture_reload_commands = Vec::new();
    let mut hdr_skybox_commands = Vec::new();
    let mut color_lut_command: Option<Vec<u8>> = None;
    #[cfg(feature = "assets")]
    let mut procedural_ibl_capture: Option<(crate::render::config::Atmosphere, f32)> = None;
    #[cfg(feature = "assets")]
    let mut ibl_snapshot_capture: Option<(crate::render::config::Atmosphere, Vec<f32>)> = None;
    #[cfg(not(target_arch = "wasm32"))]
    let mut screenshot_path: Option<Option<std::path::PathBuf>> = None;
    #[cfg(not(target_arch = "wasm32"))]
    let mut screenshot_max_dimension: Option<u32> = None;

    let render_commands = std::mem::take(&mut world.resources.commands.render);
    for cmd in render_commands {
        match cmd {
            crate::ecs::world::RenderCommand::UploadUiImageLayer {
                layer,
                rgba_data,
                width,
                height,
            } => {
                ui_image_uploads.push((layer, rgba_data, width, height));
            }
            crate::ecs::world::RenderCommand::ReloadTexture {
                name,
                rgba_data,
                width,
                height,
            } => {
                texture_reload_commands.push((name, rgba_data, width, height));
            }
            crate::ecs::world::RenderCommand::LoadHdrSkybox { hdr_data } => {
                hdr_skybox_commands.push(hdr_data);
            }
            crate::ecs::world::RenderCommand::SetColorLut { data } => {
                color_lut_command = Some(data);
            }
            crate::ecs::world::RenderCommand::LoadHdrSkyboxFromPath { path } => {
                match std::fs::read(&path) {
                    Ok(bytes) => {
                        tracing::info!("Successfully read HDR file: {}", path.display());
                        hdr_skybox_commands.push(bytes);
                    }
                    Err(e) => {
                        tracing::error!("Failed to read HDR file {}: {}", path.display(), e);
                    }
                }
            }
            crate::ecs::world::RenderCommand::CaptureScreenshot {
                path,
                max_dimension,
            } => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    screenshot_path = Some(path);
                    screenshot_max_dimension = max_dimension;
                }
                #[cfg(target_arch = "wasm32")]
                {
                    let _ = path;
                    let _ = max_dimension;
                }
            }
        }
    }

    if let Some(data) = color_lut_command
        && let Some(pass) = render_graph_get_pass_mut(&mut renderer.graph, "postprocess_pass")
        && let Some(pass) =
            (pass as &mut dyn std::any::Any).downcast_mut::<passes::PostProcessPass>()
    {
        pass.set_color_lut(data);
    }

    #[cfg(feature = "assets")]
    {
        use crate::render::config::Atmosphere;
        let atmosphere = inputs.settings.atmosphere;
        if atmosphere.is_procedural() {
            let hour = inputs.scene.day_night.hour;
            if renderer.frame_state.captured_ibl_atmosphere != Some(atmosphere) {
                renderer.frame_state.captured_ibl_atmosphere = Some(atmosphere);
                renderer.frame_state.captured_ibl_hour = hour;
                renderer.frame_state.captured_day_night_snapshots = false;
                let time = if atmosphere == Atmosphere::DayNight {
                    hour
                } else {
                    0.0
                };
                procedural_ibl_capture = Some((atmosphere, time));
            } else if atmosphere == Atmosphere::DayNight
                && !renderer.frame_state.captured_day_night_snapshots
                && hour != renderer.frame_state.captured_ibl_hour
            {
                renderer.frame_state.captured_day_night_snapshots = true;
                ibl_snapshot_capture = Some((
                    atmosphere,
                    inputs.scene.day_night.ibl_snapshot_hours.clone(),
                ));
            }
        } else {
            renderer.frame_state.captured_ibl_atmosphere = None;
        }
    }

    if !ui_image_uploads.is_empty() {
        for (layer, rgba_data, width, height) in &ui_image_uploads {
            renderer.ui_texture_array.upload_layer(
                &renderer.queue,
                *layer,
                rgba_data,
                *width,
                *height,
            );
        }
    }

    drain_loading_tasks(renderer, world, inputs);

    if !texture_reload_commands.is_empty() {
        use crate::render::wgpu::texture_cache::{TextureReloadResult, texture_cache_reload};

        for (name, rgba_data, width, height) in texture_reload_commands {
            let result = texture_cache_reload(
                &mut inputs.texture_cache,
                &renderer.device,
                &renderer.queue,
                &name,
                &rgba_data,
                width,
                height,
            );
            if matches!(result, TextureReloadResult::Recreated) {
                let usage_opt =
                    registry_entry_by_name(&inputs.texture_cache.registry, &name).map(|entry| {
                        match entry.texture.format() {
                            wgpu::TextureFormat::Rgba8UnormSrgb
                            | wgpu::TextureFormat::Bgra8UnormSrgb => {
                                crate::render::wgpu::texture_cache::TextureUsage::Color
                            }
                            _ => crate::render::wgpu::texture_cache::TextureUsage::Linear,
                        }
                    });
                if let Some(usage) = usage_opt {
                    renderer.material_texture_arrays.upload(
                        &renderer.device,
                        &renderer.queue,
                        &renderer.mip_generator,
                        crate::render::wgpu::material_texture_arrays::MaterialTextureUpload {
                            name: name.clone(),
                            rgba_data: &rgba_data,
                            width,
                            height,
                            usage,
                            wrap_u: crate::render::wgpu::texture_cache::SamplerWrap::Repeat,
                            wrap_v: crate::render::wgpu::texture_cache::SamplerWrap::Repeat,
                        },
                    );
                }
                if let Some(mesh_pass) = render_graph_get_pass_mut(&mut renderer.graph, "mesh_pass")
                    && let Some(mesh_pass) =
                        (mesh_pass as &mut dyn std::any::Any).downcast_mut::<passes::MeshPass>()
                    && let Some(texture_entry) =
                        registry_entry_by_name(&inputs.texture_cache.registry, &name)
                {
                    mesh_pass.register_texture_with_data(
                        name.clone(),
                        texture_entry.view.clone(),
                        texture_entry.sampler.clone(),
                    );
                }

                if let Some(skinned_mesh_pass) =
                    render_graph_get_pass_mut(&mut renderer.graph, "skinned_mesh_pass")
                    && let Some(skinned_mesh_pass) = (skinned_mesh_pass as &mut dyn std::any::Any)
                        .downcast_mut::<passes::geometry::SkinnedMeshPass>(
                    )
                    && let Some(texture_entry) =
                        registry_entry_by_name(&inputs.texture_cache.registry, &name)
                {
                    skinned_mesh_pass.register_texture(
                        name.clone(),
                        texture_entry.view.clone(),
                        texture_entry.sampler.clone(),
                    );
                }

                if let Some(decal_pass) =
                    render_graph_get_pass_mut(&mut renderer.graph, "decal_pass")
                    && let Some(decal_pass) =
                        (decal_pass as &mut dyn std::any::Any).downcast_mut::<passes::DecalPass>()
                    && let Some(texture_entry) =
                        registry_entry_by_name(&inputs.texture_cache.registry, &name)
                {
                    decal_pass.register_texture(
                        name,
                        texture_entry.view.clone(),
                        texture_entry.sampler.clone(),
                    );
                }
            }
        }
    }

    #[cfg(feature = "assets")]
    if !hdr_skybox_commands.is_empty() {
        let mut ibl_views = None;

        if let Some(sky_pass) = render_graph_get_pass_mut(&mut renderer.graph, "sky_pass")
            && let Some(sky_pass) =
                (sky_pass as &mut dyn std::any::Any).downcast_mut::<passes::geometry::SkyPass>()
        {
            for hdr_data in hdr_skybox_commands {
                ibl_views =
                    Some(sky_pass.load_hdr_skybox(&renderer.device, &renderer.queue, &hdr_data));
            }
        }

        if let Some((irradiance_view, prefiltered_view)) = ibl_views {
            if let Some(mesh_pass) = render_graph_get_pass_mut(&mut renderer.graph, "mesh_pass")
                && let Some(mesh_pass) =
                    (mesh_pass as &mut dyn std::any::Any).downcast_mut::<passes::MeshPass>()
            {
                mesh_pass.update_ibl_textures(
                    renderer.brdf_lut_view.clone(),
                    irradiance_view.clone(),
                    prefiltered_view.clone(),
                );
            }

            if let Some(skinned_mesh_pass) =
                render_graph_get_pass_mut(&mut renderer.graph, "skinned_mesh_pass")
                && let Some(skinned_mesh_pass) = (skinned_mesh_pass as &mut dyn std::any::Any)
                    .downcast_mut::<passes::geometry::SkinnedMeshPass>(
                )
            {
                skinned_mesh_pass.set_ibl_textures(
                    renderer.brdf_lut_view.clone(),
                    irradiance_view.clone(),
                    prefiltered_view.clone(),
                );
            }

            inputs.ibl_views.brdf_lut_view = Some(renderer.brdf_lut_view.clone());
            inputs.ibl_views.irradiance_view = Some(irradiance_view);
            inputs.ibl_views.prefiltered_view = Some(prefiltered_view);
        }
    }

    #[cfg(feature = "assets")]
    if let Some((atmosphere, time)) = procedural_ibl_capture {
        let mut ibl_views = None;

        if let Some(sky_pass) = render_graph_get_pass_mut(&mut renderer.graph, "sky_pass")
            && let Some(sky_pass) =
                (sky_pass as &mut dyn std::any::Any).downcast_mut::<passes::geometry::SkyPass>()
        {
            ibl_views = sky_pass.capture_procedural_atmosphere(
                &renderer.device,
                &renderer.queue,
                atmosphere,
                time,
            );
        }

        if let Some((irradiance_view, prefiltered_view)) = ibl_views {
            if let Some(mesh_pass) = render_graph_get_pass_mut(&mut renderer.graph, "mesh_pass")
                && let Some(mesh_pass) =
                    (mesh_pass as &mut dyn std::any::Any).downcast_mut::<passes::MeshPass>()
            {
                mesh_pass.update_ibl_textures(
                    renderer.brdf_lut_view.clone(),
                    irradiance_view.clone(),
                    prefiltered_view.clone(),
                );
            }

            if let Some(skinned_mesh_pass) =
                render_graph_get_pass_mut(&mut renderer.graph, "skinned_mesh_pass")
                && let Some(skinned_mesh_pass) = (skinned_mesh_pass as &mut dyn std::any::Any)
                    .downcast_mut::<passes::geometry::SkinnedMeshPass>(
                )
            {
                skinned_mesh_pass.set_ibl_textures(
                    renderer.brdf_lut_view.clone(),
                    irradiance_view.clone(),
                    prefiltered_view.clone(),
                );
            }

            inputs.ibl_views.brdf_lut_view = Some(renderer.brdf_lut_view.clone());
            inputs.ibl_views.irradiance_view = Some(irradiance_view);
            inputs.ibl_views.prefiltered_view = Some(prefiltered_view);
        }
    }

    #[cfg(feature = "assets")]
    if let Some((atmosphere, hours)) = ibl_snapshot_capture
        && let Some(sky_pass) = render_graph_get_pass_mut(&mut renderer.graph, "sky_pass")
        && let Some(sky_pass) =
            (sky_pass as &mut dyn std::any::Any).downcast_mut::<passes::geometry::SkyPass>()
    {
        sky_pass.capture_ibl_snapshots(&renderer.device, &renderer.queue, atmosphere, &hours);
    }

    let mut frame_any_camera_wants_normals = false;
    let mut frame_any_camera_wants_bounds = false;
    let mut frame_any_camera_wants_wireframe = false;
    let shading_cameras = world
        .resources
        .user_interface
        .required_cameras
        .iter()
        .copied()
        .chain(world.resources.active_camera);
    for camera in shading_cameras {
        let shading = world
            .core
            .get_viewport_shading(camera)
            .copied()
            .unwrap_or_default();
        let post_process = world.core.get_camera_post_process(camera).copied();
        let culling_mask = world.core.get_camera_culling_mask(camera).copied();
        let environment = world.core.get_camera_environment(camera).copied();
        let effective = crate::ecs::camera::components::effective_shading_for_camera(
            &inputs.settings,
            &inputs.debug_draw,
            &shading,
            post_process.as_ref(),
            culling_mask.as_ref(),
            environment.as_ref(),
        );
        if effective.show_normals {
            frame_any_camera_wants_normals = true;
        }
        if effective.show_bounding_volumes {
            frame_any_camera_wants_bounds = true;
        }
        if effective.show_wireframe {
            frame_any_camera_wants_wireframe = true;
        }
    }
    let lines_show_normals = inputs.debug_draw.show_normals || frame_any_camera_wants_normals;
    let lines_show_bounding_volumes =
        inputs.debug_draw.show_bounding_volumes || frame_any_camera_wants_bounds;

    if let Some(overlay_pass) = render_graph_get_pass_mut(&mut renderer.graph, "scene_overlay_pass")
        && let Some(overlay_pass) =
            (overlay_pass as &mut dyn std::any::Any).downcast_mut::<passes::SceneOverlayPass>()
    {
        let lines_pass = &mut overlay_pass.lines;
        super::lines::sync_lines_data(
            lines_pass,
            &renderer.device,
            &renderer.queue,
            world,
            inputs,
            frame_any_camera_wants_wireframe,
        );
        super::lines::sync_bounding_volume_data(
            lines_pass,
            &renderer.device,
            &renderer.queue,
            world,
            inputs,
            lines_show_bounding_volumes,
        );
        super::lines::sync_normal_data(
            lines_pass,
            &renderer.device,
            &renderer.queue,
            world,
            inputs,
            lines_show_normals,
        );
    }

    let point_shadow_view = render_graph_get_pass_mut(&mut renderer.graph, "shadow_depth_pass")
        .and_then(|pass| {
            (pass as &dyn std::any::Any)
                .downcast_ref::<passes::ShadowDepthPass>()
                .map(|shadow_pass| shadow_pass.point_light_cubemap_view().clone())
        });

    if let Some(view) = point_shadow_view {
        if let Some(mesh_pass) = render_graph_get_pass_mut(&mut renderer.graph, "mesh_pass")
            && let Some(mesh_pass) =
                (mesh_pass as &mut dyn std::any::Any).downcast_mut::<passes::MeshPass>()
        {
            mesh_pass.update_point_shadow_cubemap(view.clone());
        }
        if let Some(skinned_mesh_pass) =
            render_graph_get_pass_mut(&mut renderer.graph, "skinned_mesh_pass")
            && let Some(skinned_mesh_pass) = (skinned_mesh_pass as &mut dyn std::any::Any)
                .downcast_mut::<passes::SkinnedMeshPass>()
        {
            skinned_mesh_pass.update_point_shadow_cubemap(view);
        }
    }

    {
        let raw_hour = inputs.scene.day_night.hour;
        let hour = raw_hour - (raw_hour / 24.0).floor() * 24.0;
        let world_id = world.resources.world_id;
        let mut bracket_views: Option<(
            wgpu::TextureView,
            wgpu::TextureView,
            wgpu::TextureView,
            wgpu::TextureView,
            f32,
        )> = None;

        if let Some(sky_pass) = render_graph_get_pass_mut(&mut renderer.graph, "sky_pass")
            && let Some(sky_pass) =
                (sky_pass as &mut dyn std::any::Any).downcast_mut::<passes::geometry::SkyPass>()
            && let Some((index_a, index_b, blend)) =
                sky_pass.select_ibl_bracket(inputs.settings.atmosphere, hour)
        {
            let irr_a =
                sky_pass.ibl_snapshots[index_a]
                    .1
                    .create_view(&wgpu::TextureViewDescriptor {
                        dimension: Some(wgpu::TextureViewDimension::Cube),
                        ..Default::default()
                    });
            let pref_a =
                sky_pass.ibl_snapshots[index_a]
                    .2
                    .create_view(&wgpu::TextureViewDescriptor {
                        dimension: Some(wgpu::TextureViewDimension::Cube),
                        ..Default::default()
                    });
            let irr_b =
                sky_pass.ibl_snapshots[index_b]
                    .1
                    .create_view(&wgpu::TextureViewDescriptor {
                        dimension: Some(wgpu::TextureViewDimension::Cube),
                        ..Default::default()
                    });
            let pref_b =
                sky_pass.ibl_snapshots[index_b]
                    .2
                    .create_view(&wgpu::TextureViewDescriptor {
                        dimension: Some(wgpu::TextureViewDimension::Cube),
                        ..Default::default()
                    });
            bracket_views = Some((irr_a, pref_a, irr_b, pref_b, blend));
        }

        if let Some((irr_a, pref_a, irr_b, pref_b, blend)) = bracket_views {
            world.resources.render_settings.ibl_blend_factor = blend;
            inputs.settings.ibl_blend_factor = blend;

            if let Some(mesh_pass) = render_graph_get_pass_mut(&mut renderer.graph, "mesh_pass")
                && let Some(mesh_pass) =
                    (mesh_pass as &mut dyn std::any::Any).downcast_mut::<passes::MeshPass>()
            {
                mesh_pass.update_ibl_textures_blended(
                    renderer.brdf_lut_view.clone(),
                    irr_a.clone(),
                    pref_a.clone(),
                    irr_b.clone(),
                    pref_b.clone(),
                    blend,
                );
            }

            if let Some(skinned_mesh_pass) =
                render_graph_get_pass_mut(&mut renderer.graph, "skinned_mesh_pass")
                && let Some(skinned_mesh_pass) = (skinned_mesh_pass as &mut dyn std::any::Any)
                    .downcast_mut::<passes::geometry::SkinnedMeshPass>(
                )
            {
                skinned_mesh_pass.update_ibl_textures_blended_for_world(
                    world_id,
                    passes::geometry::BlendedIblViews {
                        brdf_lut: renderer.brdf_lut_view.clone(),
                        irradiance_a: irr_a.clone(),
                        prefiltered_a: pref_a.clone(),
                        irradiance_b: irr_b,
                        prefiltered_b: pref_b,
                        blend_factor: blend,
                    },
                );
            }

            inputs.ibl_views.brdf_lut_view = Some(renderer.brdf_lut_view.clone());
            inputs.ibl_views.irradiance_view = Some(irr_a);
            inputs.ibl_views.prefiltered_view = Some(pref_a);
        }
    }

    let surface_texture = match renderer.surface.get_current_texture() {
        wgpu::CurrentSurfaceTexture::Success(frame)
        | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
        wgpu::CurrentSurfaceTexture::Outdated => {
            renderer
                .surface
                .configure(&renderer.device, &renderer.surface_config);
            match renderer.surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(frame)
                | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
                other => {
                    panic!("Failed to get surface texture after reconfiguration: {other:?}")
                }
            }
        }
        _ => {
            return Ok(());
        }
    };

    let surface_texture_view = surface_texture
        .texture
        .create_view(&wgpu::TextureViewDescriptor {
            label: wgpu::Label::default(),
            aspect: wgpu::TextureAspect::default(),
            format: Some(renderer.surface_format),
            dimension: None,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
            usage: None,
        });

    render_graph_set_external_texture(
        &mut renderer.graph,
        renderer.targets.swapchain,
        Some(surface_texture.texture.clone()),
        surface_texture_view,
        renderer.surface_config.width,
        renderer.surface_config.height,
    );

    render_graph_set_external_texture(
        &mut renderer.graph,
        renderer.targets.spotlight_shadow_atlas,
        Some(renderer.spotlight_shadow_atlas_texture.clone()),
        renderer.spotlight_shadow_atlas_view.clone(),
        renderer.spotlight_shadow_atlas_texture.width(),
        renderer.spotlight_shadow_atlas_texture.height(),
    );

    let active_camera_entities: std::collections::HashSet<freecs::Entity> = world
        .resources
        .user_interface
        .required_cameras
        .iter()
        .copied()
        .collect();
    renderer.cleanup_unused_camera_viewports(&active_camera_entities);

    world
        .resources
        .user_interface
        .viewport_texture_sizes
        .clear();

    let dirty_world_spheres: Vec<(nalgebra_glm::Vec3, f32)> = {
        let mut spheres = Vec::new();
        let dirty_entities: Vec<_> = world
            .resources
            .mesh_render_state
            .dirty_entities_for_culling()
            .collect();
        for entity in dirty_entities {
            let Some(dynamic) = inputs.scene.render_dynamic_objects.get(&entity) else {
                continue;
            };
            let Some(bounding_volume) = inputs.scene.render_bounds.get(&entity) else {
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
    let frame_settings_version = inputs.scene.settings_version;

    let mouse_position = world.resources.input.mouse.position;
    let original_active_camera = world.resources.active_camera;
    let focus_policy = inputs.scene.focus_policy;

    let primary_cameras = world.resources.user_interface.required_cameras.clone();
    dispatch_window_renders(
        renderer,
        world,
        inputs,
        WindowDispatchContext {
            cameras: primary_cameras,
            target_size: (
                renderer.surface_config.width,
                renderer.surface_config.height,
            ),
            dirty_world_spheres: &dirty_world_spheres,
            frame_settings_version,
            original_active_camera,
            focus_policy,
            mouse_position,
            global_dirty_signal,
        },
    )?;

    #[cfg(not(target_arch = "wasm32"))]
    {
        if !renderer.screenshot.pending
            && let Some(path) = screenshot_path
            && let Some(compute_output_texture) =
                render_graph_get_texture(&renderer.graph, renderer.targets.compute_output)
        {
            let width = renderer.surface_config.width;
            let height = renderer.surface_config.height;
            let bytes_per_pixel = 4u32;
            let unpadded_bytes_per_row = width * bytes_per_pixel;
            let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;

            let required_buffer_size = (padded_bytes_per_row * height) as u64;
            if renderer.screenshot.staging_buffer.size() < required_buffer_size {
                renderer.screenshot.staging_buffer =
                    renderer.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Screenshot Staging Buffer"),
                        size: required_buffer_size,
                        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                        mapped_at_creation: false,
                    });
            }

            let mut encoder =
                renderer
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Screenshot Encoder"),
                    });

            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: compute_output_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &renderer.screenshot.staging_buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(padded_bytes_per_row),
                        rows_per_image: Some(height),
                    },
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );

            renderer.queue.submit(std::iter::once(encoder.finish()));

            renderer.screenshot.path = path;
            renderer.screenshot.width = width;
            renderer.screenshot.height = height;
            renderer.screenshot.max_dimension = screenshot_max_dimension;
            renderer.screenshot.pending = true;

            let map_complete = renderer.screenshot.map_complete.clone();
            renderer.screenshot.staging_buffer.slice(..).map_async(
                wgpu::MapMode::Read,
                move |_| {
                    map_complete.store(true, std::sync::atomic::Ordering::Relaxed);
                },
            );
        }
    }

    surface_texture.present();

    Ok(())
}
pub fn configure_with_state(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    state: &mut dyn crate::run::State,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(pass) = render_graph_get_pass_mut(&mut renderer.graph, "scene_overlay_pass")
        && let Some(overlay) =
            (pass as &mut dyn std::any::Any).downcast_mut::<passes::SceneOverlayPass>()
    {
        overlay
            .text
            .update_glyph_atlas(&renderer.device, &renderer.glyph_atlas.texture_view);
    }

    state.configure_render_graph(
        &mut renderer.graph,
        &renderer.device,
        renderer.surface_format,
        crate::run::RenderResources {
            scene_color: renderer.targets.scene_color,
            depth: renderer.targets.depth,
            compute_output: renderer.targets.compute_output,
            swapchain: renderer.targets.swapchain,
            view_normals: renderer.targets.view_normals,
            velocity: renderer.targets.velocity,
            ssao_raw: renderer.targets.ssao_raw,
            ssao: renderer.targets.ssao,
            ssgi_raw: renderer.targets.ssgi_raw,
            ssgi: renderer.targets.ssgi,
            ssr_raw: renderer.targets.ssr_raw,
            ssr: renderer.targets.ssr,
            surface_width: renderer.surface_config.width,
            surface_height: renderer.surface_config.height,
        },
    );

    let viewport_blit_pass = passes::BlitPass::new(&renderer.device, renderer.surface_format)
        .with_name("viewport_blit_pass");
    render_graph_add_pass(
        &mut renderer.graph,
        Box::new(viewport_blit_pass),
        &[
            ("input", renderer.targets.aa_output),
            ("output", renderer.targets.viewport_resource),
        ],
    )?;

    let cached_viewport_preload_pass =
        passes::BlitPass::new(&renderer.device, renderer.surface_format)
            .with_name("cached_viewport_preload_pass")
            .with_runs_in_full(false)
            .with_runs_in_compose_only(true);
    render_graph_add_pass(
        &mut renderer.graph,
        Box::new(cached_viewport_preload_pass),
        &[
            ("input", renderer.targets.viewport_resource),
            ("output", renderer.targets.aa_output),
        ],
    )?;

    let swapchain_compose_pass =
        passes::ViewportComposePass::new(&renderer.device, renderer.surface_format);
    render_graph_add_pass(
        &mut renderer.graph,
        Box::new(swapchain_compose_pass),
        &[
            ("input", renderer.targets.aa_output),
            ("output", renderer.targets.swapchain),
        ],
    )?;

    let mut ui_pass = passes::UiPass::new(&renderer.device, renderer.surface_format);
    ui_pass.update_glyph_atlas(&renderer.device, &renderer.glyph_atlas.texture_view);
    render_graph_add_pass(
        &mut renderer.graph,
        Box::new(ui_pass),
        &[
            ("color", renderer.targets.swapchain),
            ("depth", renderer.targets.ui_depth),
        ],
    )?;

    if let Some(ui_image_pass) = renderer.ui_image_pass.take() {
        render_graph_add_pass(
            &mut renderer.graph,
            ui_image_pass,
            &[
                ("color", renderer.targets.swapchain),
                ("depth", renderer.targets.ui_depth),
            ],
        )?;
    }

    #[cfg(feature = "egui")]
    {
        let egui_pass = passes::EguiPass::new(&renderer.device, renderer.surface_format);
        render_graph_add_pass(
            &mut renderer.graph,
            Box::new(egui_pass),
            &[("color", renderer.targets.swapchain)],
        )?;
    }

    let pick_keepalive_pass = passes::PickKeepalivePass::new();
    render_graph_add_pass(
        &mut renderer.graph,
        Box::new(pick_keepalive_pass),
        &[
            ("entity_id", renderer.targets.entity_id),
            ("depth", renderer.targets.depth),
        ],
    )?;

    render_graph_compile(&mut renderer.graph)?;
    Ok(())
}

pub(crate) fn initialize_fonts(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    _world: &mut crate::ecs::world::World,
) {
    renderer.frame_state.glyph_atlas_initialized = true;
}

pub fn update_with_state(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    state: &mut dyn crate::run::State,
    world: &mut crate::ecs::world::World,
) -> Result<(), Box<dyn std::error::Error>> {
    initialize_fonts(renderer, world);

    for pending_load in world.resources.loading.pending_font_loads.drain(..) {
        world
            .resources
            .text
            .font_engine
            .load_font(pending_load.font_data);
    }

    state.update_render_graph(&mut renderer.graph, world);
    sync_cloth_write_targets(renderer, world);
    Ok(())
}

/// Hands the cloth pass its vertex buffer write targets for this frame.
///
/// Cloth entities render through the regular mesh and shadow passes;
/// the cloth simulation writes deformed vertices into those passes'
/// vertex buffers at each cloth's registered mesh range. The targets
/// are refreshed every frame so buffer growth, compaction, and mesh
/// re-registration are picked up, and the mesh cull bounds are widened
/// to the cloth's full deformation range.
pub(crate) fn sync_cloth_write_targets(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    world: &crate::ecs::world::World,
) {
    use crate::ecs::world::{CLOTH, RENDER_MESH};

    let mut cloths: Vec<(String, [f32; 3], f32)> = Vec::new();
    world
        .core
        .query()
        .with(CLOTH | RENDER_MESH)
        .iter(|_entity, table, index| {
            let cloth = &table.cloth[index];
            let bounds = crate::ecs::cloth::systems::cloth_bounding_volume(cloth);
            cloths.push((
                table.render_mesh[index].name.clone(),
                [
                    bounds.obb.center.x,
                    bounds.obb.center.y,
                    bounds.obb.center.z,
                ],
                bounds.sphere_radius,
            ));
        });
    if cloths.is_empty() {
        return;
    }

    let mut targets: std::collections::HashMap<
        String,
        Vec<crate::render::wgpu::passes::geometry::ClothWriteTarget>,
    > = std::collections::HashMap::new();

    if let Some(mesh_pass) = render_graph_get_pass_mut(&mut renderer.graph, "mesh_pass")
        && let Some(mesh_pass) =
            (mesh_pass as &mut dyn std::any::Any).downcast_mut::<passes::MeshPass>()
    {
        for (name, center, radius) in &cloths {
            mesh_pass.override_mesh_bounds(&renderer.queue, name, *center, *radius);
            if let Some((buffer, buffer_generation, vertex_offset)) =
                mesh_pass.vertex_write_target(name)
            {
                targets.entry(name.clone()).or_default().push(
                    crate::render::wgpu::passes::geometry::ClothWriteTarget {
                        buffer,
                        buffer_generation,
                        vertex_offset,
                    },
                );
            }
        }
    }

    if let Some(shadow_pass) = render_graph_get_pass_mut(&mut renderer.graph, "shadow_depth_pass")
        && let Some(shadow_pass) =
            (shadow_pass as &mut dyn std::any::Any).downcast_mut::<passes::ShadowDepthPass>()
    {
        for (name, _, _) in &cloths {
            if let Some((buffer, buffer_generation, vertex_offset)) =
                shadow_pass.vertex_write_target(name)
            {
                targets.entry(name.clone()).or_default().push(
                    crate::render::wgpu::passes::geometry::ClothWriteTarget {
                        buffer,
                        buffer_generation,
                        vertex_offset,
                    },
                );
            }
        }
    }

    if let Some(cloth_pass) = render_graph_get_pass_mut(&mut renderer.graph, "cloth_pass")
        && let Some(cloth_pass) =
            (cloth_pass as &mut dyn std::any::Any).downcast_mut::<passes::geometry::ClothPass>()
    {
        cloth_pass.set_write_targets(targets);
    }
}

pub(crate) struct MaterialTextureUploadRequest<'a> {
    texture: crate::render::asset_id::TextureId,
    rgba_data: &'a [u8],
    width: u32,
    height: u32,
    usage: crate::render::wgpu::texture_cache::TextureUsage,
    sampler: crate::render::wgpu::texture_cache::SamplerSettings,
}

/// Enables conditional passes only when their consumers are active so the
/// graph can discard unused attachment stores. Disabling a pass drops its
/// declared reads from store-op analysis: with SSAO and depth of field off
/// nothing reads `view_normals` or the final `depth` contents, with no
/// pending pick nothing reads `entity_id`, and with no skinned meshes the
/// skinned pass stops holding `entity_id` stored for the mesh pass.
/// The postprocess pass samples `scene_color` directly while depth of
/// field is disabled, so the depth of field node can turn off instead of
/// blitting the scene into `dof_output` every frame.
pub(crate) fn sync_conditional_pass_toggles(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    world: &crate::ecs::world::World,
    inputs: &mut crate::render::wgpu::render_configs::RenderInputs,
) {
    let view = &inputs.scene.active_view;
    let _ = render_graph_set_pass_enabled(&mut renderer.graph, "ssao_pass", view.ssao_enabled);
    let _ = render_graph_set_pass_enabled(&mut renderer.graph, "ssao_blur_pass", view.ssao_enabled);
    let _ = render_graph_set_pass_enabled(&mut renderer.graph, "bloom_pass", view.bloom_enabled);
    let _ = render_graph_set_pass_enabled(
        &mut renderer.graph,
        "depth_of_field_pass",
        view.depth_of_field.enabled,
    );
    let picking_active = world.resources.gpu_picking.has_pending_request();
    let _ =
        render_graph_set_pass_enabled(&mut renderer.graph, "pick_keepalive_pass", picking_active);
    let has_skinned_meshes = world
        .core
        .query_entities(crate::ecs::world::SKIN)
        .next()
        .is_some();
    let _ =
        render_graph_set_pass_enabled(&mut renderer.graph, "skinned_mesh_pass", has_skinned_meshes);

    // Run the temporal resolve only when enabled; otherwise a cheap blit
    // copies the post-process output through so the compose still has an
    // image, without executing the full resolve pass.
    let taa_on = inputs.settings.taa_enabled;
    let _ = render_graph_set_pass_enabled(&mut renderer.graph, "taa_pass", taa_on);
    let _ = render_graph_set_pass_enabled(&mut renderer.graph, "taa_passthrough", !taa_on);
}

pub(crate) fn dispatch_pick_compute_if_pending(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    world: &mut crate::ecs::world::World,
) {
    if renderer.depth_pick.pending {
        return;
    }
    let Some(request) = world.resources.gpu_picking.take_pending_request() else {
        return;
    };
    let Some(depth_texture_view) =
        render_graph_get_texture_view(&renderer.graph, renderer.targets.depth)
    else {
        return;
    };
    let Some(entity_id_texture_view) =
        render_graph_get_texture_view(&renderer.graph, renderer.targets.entity_id)
    else {
        return;
    };

    let uniform_data: [u32; 4] = [
        request.screen_x,
        request.screen_y,
        DEPTH_PICK_SAMPLE_SIZE,
        0,
    ];
    renderer.queue.write_buffer(
        &renderer.depth_pick.uniform_buffer,
        0,
        bytemuck::cast_slice(&uniform_data),
    );

    let bind_group = renderer
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Depth Pick Bind Group"),
            layout: &renderer.depth_pick.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(depth_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: renderer.depth_pick.storage_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: renderer.depth_pick.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(entity_id_texture_view),
                },
            ],
        });

    let mut encoder = renderer
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Depth Pick Encoder"),
        });

    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Depth Pick Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&renderer.depth_pick.compute_pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(DEPTH_PICK_SAMPLE_SIZE, DEPTH_PICK_SAMPLE_SIZE, 1);
    }

    encoder.copy_buffer_to_buffer(
        &renderer.depth_pick.storage_buffer,
        0,
        &renderer.depth_pick.staging_buffer,
        0,
        (DEPTH_PICK_SAMPLE_SIZE * DEPTH_PICK_SAMPLE_SIZE * 8) as u64,
    );

    renderer.queue.submit(std::iter::once(encoder.finish()));
    renderer.depth_pick.bind_group = Some(bind_group);
    renderer.depth_pick.pending = true;
    renderer.depth_pick.center = (request.screen_x, request.screen_y);
    renderer.depth_pick.texture_size = renderer.render_buffer_size;
    renderer.depth_pick.camera = world.resources.active_camera;

    let map_complete = renderer.depth_pick.map_complete.clone();
    renderer
        .depth_pick
        .staging_buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |_| {
            map_complete.store(true, std::sync::atomic::Ordering::Relaxed);
        });
}

pub(crate) fn clear_swapchain_for_window_ui(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    world: &crate::ecs::world::World,
) {
    let Some(view) =
        rendergraph::render_graph_get_texture_view(&renderer.graph, renderer.targets.swapchain)
    else {
        return;
    };
    let clear = world
        .resources
        .retained_ui
        .background_color
        .map(|color| wgpu::Color {
            r: color.x as f64,
            g: color.y as f64,
            b: color.z as f64,
            a: color.w as f64,
        })
        .unwrap_or(wgpu::Color {
            r: 0.04,
            g: 0.04,
            b: 0.06,
            a: 1.0,
        });
    let mut encoder = renderer
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Window UI Swapchain Clear"),
        });
    {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Window UI Swapchain Clear Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
    renderer.queue.submit(std::iter::once(encoder.finish()));
}

pub(crate) fn dispatch_window_renders(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    world: &mut crate::ecs::world::World,
    inputs: &mut crate::render::wgpu::render_configs::RenderInputs,
    ctx: WindowDispatchContext<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let _span =
        tracing::info_span!("dispatch_window_renders", cameras = ctx.cameras.len(),).entered();
    let WindowDispatchContext {
        cameras,
        target_size,
        dirty_world_spheres,
        frame_settings_version,
        original_active_camera,
        focus_policy,
        mouse_position,
        global_dirty_signal,
    } = ctx;

    let (surface_width, surface_height) = target_size;
    let has_viewports = !cameras.is_empty();
    let _ = render_graph_set_pass_enabled(&mut renderer.graph, "viewport_blit_pass", has_viewports);
    let _ = render_graph_set_pass_enabled(&mut renderer.graph, "viewport_compose_pass", true);

    if !has_viewports {
        renderer.resize_render_buffers_for_window(surface_width, surface_height);
    }

    world.resources.window.camera_tile_render_iteration = 0;
    inputs.view.camera_tile_render_iteration = 0;

    let ui_pass_states_before_loop = if has_viewports {
        let saved_states = render_graph_save_pass_enabled_states(&renderer.graph);
        let saved = (
            saved_states.get("ui_pass").copied().unwrap_or(true),
            saved_states.get("ui_image_pass").copied().unwrap_or(true),
        );
        let _ = render_graph_set_pass_enabled(&mut renderer.graph, "ui_pass", false);
        let _ = render_graph_set_pass_enabled(&mut renderer.graph, "ui_image_pass", false);
        Some(saved)
    } else {
        None
    };

    let mouse_focused_camera = world
        .resources
        .window
        .camera_tile_rects
        .iter()
        .find(|(_, rect)| rect.contains(mouse_position))
        .map(|(entity, _)| *entity);
    let focused_camera = mouse_focused_camera.or(original_active_camera);

    let requested_anisotropy = inputs.settings.material_anisotropy_filtering;
    let anisotropy_changed = renderer
        .material_texture_arrays
        .set_anisotropy(&renderer.device, requested_anisotropy);
    if renderer.material_texture_arrays.take_bindless_dirty() || anisotropy_changed {
        if let Some(pass) = render_graph_get_pass_mut(&mut renderer.graph, "mesh_pass")
            && let Some(mesh_pass) =
                (pass as &mut dyn std::any::Any).downcast_mut::<passes::MeshPass>()
        {
            mesh_pass.apply_material_textures(&renderer.device, &renderer.material_texture_arrays);
        }
        if let Some(pass) = render_graph_get_pass_mut(&mut renderer.graph, "skinned_mesh_pass")
            && let Some(skinned) =
                (pass as &mut dyn std::any::Any).downcast_mut::<passes::geometry::SkinnedMeshPass>()
        {
            skinned.apply_material_textures(&renderer.device, &renderer.material_texture_arrays);
        }
    }

    for (iteration_index, camera_entity) in cameras.iter().enumerate() {
        let (tile_width, tile_height) = world
            .resources
            .window
            .camera_tile_rects
            .get(camera_entity)
            .map(|rect| {
                (
                    (rect.width.round() as u32).max(1),
                    (rect.height.round() as u32).max(1),
                )
            })
            .unwrap_or((surface_width.max(1), surface_height.max(1)));
        let constrained = world.core.get_constrained_aspect(*camera_entity).copied();
        let (display_width, display_height) = if let Some(aspect) = constrained {
            fit_constrained_aspect(tile_width, tile_height, aspect.0)
        } else {
            (tile_width, tile_height)
        };
        let render_scale = inputs.settings.render_scale.clamp(0.25, 4.0);
        let is_main_viewport_camera = Some(*camera_entity) == original_active_camera;
        let render_width = ((display_width as f32 * render_scale).round() as u32).max(1);
        let render_height = ((display_height as f32 * render_scale).round() as u32).max(1);

        renderer.ensure_camera_viewport(*camera_entity, render_width, render_height);
        renderer.resize_render_buffers_for_window(render_width, render_height);

        let viewport_cloned = renderer
            .camera_viewports
            .get(camera_entity)
            .map(|viewport| {
                (
                    viewport.texture.clone(),
                    viewport.view.clone(),
                    viewport.size,
                    viewport.has_rendered_at_least_once,
                    viewport.last_active_view,
                    viewport.last_settings_version,
                    viewport.last_camera_world_transform,
                )
            });

        if let Some((
            texture,
            view,
            size,
            has_rendered_once,
            last_active_view,
            last_settings_version,
            last_camera_world_transform,
        )) = viewport_cloned
        {
            render_graph_set_external_texture(
                &mut renderer.graph,
                renderer.targets.viewport_resource,
                Some(texture),
                view,
                render_width,
                render_height,
            );

            world.resources.active_camera = Some(*camera_entity);
            inputs.view.active_camera = Some(*camera_entity);
            world.resources.window.camera_tile_render_iteration = iteration_index as u32;
            inputs.view.camera_tile_render_iteration = iteration_index as u32;

            let viewport_shading = world
                .core
                .get_viewport_shading(*camera_entity)
                .copied()
                .unwrap_or_default();
            let update_mode = world
                .core
                .get_viewport_update_mode(*camera_entity)
                .copied()
                .unwrap_or_default();

            let camera_post_process = world.core.get_camera_post_process(*camera_entity).copied();
            let camera_culling_mask = world.core.get_camera_culling_mask(*camera_entity).copied();
            let camera_environment = world.core.get_camera_environment(*camera_entity).copied();
            let effective = crate::ecs::camera::components::effective_shading_for_camera(
                &inputs.settings,
                &inputs.debug_draw,
                &viewport_shading,
                camera_post_process.as_ref(),
                camera_culling_mask.as_ref(),
                camera_environment.as_ref(),
            );

            let is_focused_view = Some(*camera_entity) == focused_camera;
            let focus_forces_render = matches!(
                focus_policy,
                crate::render::config::ViewportFocusPolicy::FocusedAlways
            ) && is_focused_view;
            let view_changed = last_active_view != Some(effective);
            let settings_changed = last_settings_version != frame_settings_version;
            let camera_matrices =
                crate::ecs::camera::queries::query_camera_matrices(world, *camera_entity);
            let camera_world_transform = world
                .core
                .get_global_transform(*camera_entity)
                .map(|gt| gt.0);
            let camera_self_dirty = match (last_camera_world_transform, camera_world_transform) {
                (Some(prev), Some(curr)) => prev != curr,
                (None, _) => true,
                (_, None) => false,
            };
            let frustum_dirty = if let Some(matrices) = camera_matrices.as_ref() {
                let view_proj = matrices.projection * matrices.view;
                let frustum_planes = passes::geometry::extract_frustum_planes(&view_proj);
                dirty_world_spheres.iter().any(|(center, radius)| {
                    passes::geometry::sphere_in_frustum(center, *radius, &frustum_planes)
                })
            } else {
                !dirty_world_spheres.is_empty()
            };
            let pick_forces_render =
                is_main_viewport_camera && world.resources.gpu_picking.has_pending_request();
            let camera_force_render = world
                .resources
                .window
                .force_render_cameras
                .contains(camera_entity);
            let should_render = pick_forces_render
                || camera_force_render
                || match update_mode {
                    crate::ecs::camera::components::ViewportUpdateMode::Always => true,
                    crate::ecs::camera::components::ViewportUpdateMode::WhenVisible => true,
                    crate::ecs::camera::components::ViewportUpdateMode::WhenDirty => {
                        focus_forces_render
                            || !has_rendered_once
                            || global_dirty_signal
                            || settings_changed
                            || view_changed
                            || camera_self_dirty
                            || frustum_dirty
                    }
                    crate::ecs::camera::components::ViewportUpdateMode::Once => !has_rendered_once,
                    crate::ecs::camera::components::ViewportUpdateMode::Disabled => {
                        !has_rendered_once
                    }
                };

            inputs.scene.active_view = effective;
            let render_view =
                crate::ecs::camera::queries::query_active_camera_matrices(world).map(|matrices| {
                    build_render_view(
                        world,
                        *camera_entity,
                        &matrices,
                        inputs.scene.taa_jitter,
                        (render_width, render_height),
                    )
                });
            inputs.scene.render_view = render_view;
            let render_lighting = build_render_lighting(world, &inputs.scene);
            inputs.scene.render_lighting = Some(render_lighting);
            sync_conditional_pass_toggles(renderer, world, inputs);

            if renderer.frame_state.extract_dirty_done_for_frame != renderer.frame_state.index {
                let _span = tracing::info_span!("extract_frame_dirty_state").entered();
                super::extract::extract_frame_dirty_state(renderer, world, inputs);
                renderer.frame_state.extract_dirty_done_for_frame = renderer.frame_state.index;
            }
            if renderer.frame_state.text_meshes_prepared_for_frame != renderer.frame_state.index {
                let _span = tracing::info_span!("prepare_text_meshes").entered();
                super::extract::prepare_text_meshes(renderer, world, inputs);
                renderer.frame_state.text_meshes_prepared_for_frame = renderer.frame_state.index;
            }
            let phase = if should_render {
                rendergraph::ExecutePhase::Full
            } else {
                rendergraph::ExecutePhase::ComposeOnly
            };
            let command_buffers = {
                let _span = tracing::info_span!("render_graph_execute", phase = ?phase).entered();
                rendergraph::render_graph_execute_with_phase(
                    &mut renderer.graph,
                    &renderer.device,
                    &renderer.queue,
                    inputs,
                    phase,
                )?
            };
            {
                let _span = tracing::info_span!("queue_submit").entered();
                renderer.queue.submit(command_buffers);
            }

            if should_render
                && let Some(viewport) = renderer.camera_viewports.get_mut(camera_entity)
            {
                viewport.has_rendered_at_least_once = true;
                viewport.last_active_view = Some(effective);
                viewport.last_settings_version = frame_settings_version;
                viewport.last_render_frame = renderer.frame_state.index;
                viewport.last_camera_world_transform = camera_world_transform;
            }
            if should_render {
                world
                    .resources
                    .window
                    .force_render_cameras
                    .remove(camera_entity);
            }

            if is_main_viewport_camera {
                dispatch_pick_compute_if_pending(renderer, world);
            }

            world
                .resources
                .user_interface
                .viewport_texture_sizes
                .push(size);
        }
    }

    world.resources.active_camera = original_active_camera;
    inputs.view.active_camera = original_active_camera;
    inputs.scene.active_view = match original_active_camera.filter(|_| cameras.is_empty()) {
        Some(camera) => {
            let shading = world
                .core
                .get_viewport_shading(camera)
                .copied()
                .unwrap_or_default();
            let post_process = world.core.get_camera_post_process(camera).copied();
            let culling_mask = world.core.get_camera_culling_mask(camera).copied();
            let environment = world.core.get_camera_environment(camera).copied();
            crate::ecs::camera::components::effective_shading_for_camera(
                &inputs.settings,
                &inputs.debug_draw,
                &shading,
                post_process.as_ref(),
                culling_mask.as_ref(),
                environment.as_ref(),
            )
        }
        None => crate::ecs::camera::components::effective_shading_from_graphics(
            &inputs.settings,
            &inputs.debug_draw,
        ),
    };
    sync_conditional_pass_toggles(renderer, world, inputs);

    if let Some((ui_was_enabled, ui_image_was_enabled)) = ui_pass_states_before_loop {
        let _span = tracing::info_span!("ui_composite").entered();

        let _ = rendergraph::render_graph_resize_transient_resource(
            &mut renderer.graph,
            &renderer.device,
            renderer.targets.ui_depth,
            surface_width.max(1),
            surface_height.max(1),
        );

        let swapped_ui = swap_window_ui_frame(world, target_size);

        let saved_pass_states = render_graph_save_pass_enabled_states(&renderer.graph);
        render_graph_enable_only_passes(
            &mut renderer.graph,
            &[
                ("ui_pass", ui_was_enabled),
                ("ui_image_pass", ui_image_was_enabled),
            ],
        );
        super::extract::extract_frame_dirty_state(renderer, world, inputs);
        super::extract::prepare_text_meshes(renderer, world, inputs);
        let command_buffers = {
            let _span = tracing::info_span!("ui_composite_execute").entered();
            render_graph_execute(
                &mut renderer.graph,
                &renderer.device,
                &renderer.queue,
                inputs,
            )?
        };
        renderer.queue.submit(command_buffers);

        restore_window_ui_frame(world, swapped_ui);

        render_graph_restore_pass_enabled_states(&mut renderer.graph, &saved_pass_states);
        let _ = render_graph_set_pass_enabled(&mut renderer.graph, "ui_pass", ui_was_enabled);
        let _ = render_graph_set_pass_enabled(
            &mut renderer.graph,
            "ui_image_pass",
            ui_image_was_enabled,
        );
    }

    if cameras.is_empty() {
        world.resources.window.camera_tile_render_iteration = 0;
        inputs.view.camera_tile_render_iteration = 0;

        if inputs.settings.render_world_to_swapchain {
            let render_view =
                crate::ecs::camera::queries::query_active_camera_matrices(world).map(|matrices| {
                    build_render_view(
                        world,
                        world.resources.active_camera.unwrap_or(freecs::Entity {
                            id: 0,
                            generation: 0,
                        }),
                        &matrices,
                        inputs.scene.taa_jitter,
                        renderer.render_buffer_size,
                    )
                });
            inputs.scene.render_view = render_view;
            let render_lighting = build_render_lighting(world, &inputs.scene);
            inputs.scene.render_lighting = Some(render_lighting);
            super::extract::extract_frame_dirty_state(renderer, world, inputs);
            super::extract::prepare_text_meshes(renderer, world, inputs);
            let command_buffers = render_graph_execute(
                &mut renderer.graph,
                &renderer.device,
                &renderer.queue,
                inputs,
            )?;
            renderer.queue.submit(command_buffers);
            dispatch_pick_compute_if_pending(renderer, world);
        } else {
            let swapped_ui = swap_window_ui_frame(world, target_size);
            clear_swapchain_for_window_ui(renderer, world);

            let _ = rendergraph::render_graph_resize_transient_resource(
                &mut renderer.graph,
                &renderer.device,
                renderer.targets.ui_depth,
                surface_width.max(1),
                surface_height.max(1),
            );

            let saved_pass_states = render_graph_save_pass_enabled_states(&renderer.graph);
            render_graph_enable_only_passes(
                &mut renderer.graph,
                &[("ui_pass", true), ("ui_image_pass", true)],
            );

            super::extract::extract_frame_dirty_state(renderer, world, inputs);
            super::extract::prepare_text_meshes(renderer, world, inputs);
            let command_buffers = render_graph_execute(
                &mut renderer.graph,
                &renderer.device,
                &renderer.queue,
                inputs,
            )?;
            renderer.queue.submit(command_buffers);

            restore_window_ui_frame(world, swapped_ui);

            render_graph_restore_pass_enabled_states(&mut renderer.graph, &saved_pass_states);
            let _ = render_graph_set_pass_enabled(&mut renderer.graph, "ui_pass", true);
            let _ = render_graph_set_pass_enabled(&mut renderer.graph, "ui_image_pass", true);
        }
    }

    let dispatched_settings_version = inputs.scene.settings_version;
    renderer.record_window_dispatch(dispatched_settings_version);
    Ok(())
}

pub(crate) fn upload_material_texture(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    inputs: &mut crate::render::wgpu::render_configs::RenderInputs,
    request: MaterialTextureUploadRequest<'_>,
) {
    let MaterialTextureUploadRequest {
        texture,
        rgba_data,
        width,
        height,
        usage,
        sampler,
    } = request;
    let Some(name) =
        registry_name_for(&inputs.texture_cache.registry, texture.index).map(str::to_string)
    else {
        tracing::error!(
            "upload_material_texture: no name registered for texture id {}",
            texture.index
        );
        return;
    };
    if let Err(e) = crate::render::wgpu::texture_cache::texture_cache_load_from_raw_rgba_with_format(
        &mut inputs.texture_cache,
        &renderer.device,
        &renderer.queue,
        &renderer.mip_generator,
        crate::render::wgpu::texture_cache::TextureUploadRequest {
            name: name.clone(),
            rgba_data,
            dimensions: (width, height),
            spec: crate::render::wgpu::texture_cache::TextureUploadSpec {
                format: usage.wgpu_format(),
                sampler,
            },
        },
    ) {
        tracing::error!("Failed to load texture: {}", e);
        return;
    }

    let layer_index = renderer.material_texture_arrays.upload(
        &renderer.device,
        &renderer.queue,
        &renderer.mip_generator,
        crate::render::wgpu::material_texture_arrays::MaterialTextureUpload {
            name: name.clone(),
            rgba_data,
            width,
            height,
            usage,
            wrap_u: sampler.wrap_u,
            wrap_v: sampler.wrap_v,
        },
    );

    if let Some(mesh_pass) = render_graph_get_pass_mut(&mut renderer.graph, "mesh_pass")
        && let Some(mesh_pass) =
            (mesh_pass as &mut dyn std::any::Any).downcast_mut::<passes::MeshPass>()
        && let Some(texture_entry) = registry_entry_by_name(&inputs.texture_cache.registry, &name)
    {
        mesh_pass.register_texture_with_data(
            name.clone(),
            texture_entry.view.clone(),
            texture_entry.sampler.clone(),
        );
        if let Some(layer) = layer_index {
            mesh_pass.add_material_layer_mapping(
                texture,
                crate::render::wgpu::material_texture_arrays::MaterialTextureLayer {
                    usage,
                    layer,
                    wrap_u: sampler.wrap_u,
                    wrap_v: sampler.wrap_v,
                },
            );
        }
    }

    if let Some(skinned_mesh_pass) =
        render_graph_get_pass_mut(&mut renderer.graph, "skinned_mesh_pass")
        && let Some(skinned_mesh_pass) = (skinned_mesh_pass as &mut dyn std::any::Any)
            .downcast_mut::<passes::geometry::SkinnedMeshPass>()
        && let Some(texture_entry) = registry_entry_by_name(&inputs.texture_cache.registry, &name)
    {
        skinned_mesh_pass.register_texture(
            name.clone(),
            texture_entry.view.clone(),
            texture_entry.sampler.clone(),
        );
        if let Some(layer) = layer_index {
            skinned_mesh_pass.add_material_layer_mapping(
                texture,
                crate::render::wgpu::material_texture_arrays::MaterialTextureLayer {
                    usage,
                    layer,
                    wrap_u: sampler.wrap_u,
                    wrap_v: sampler.wrap_v,
                },
            );
        }
    }

    if let Some(decal_pass) = render_graph_get_pass_mut(&mut renderer.graph, "decal_pass")
        && let Some(decal_pass) =
            (decal_pass as &mut dyn std::any::Any).downcast_mut::<passes::DecalPass>()
        && let Some(texture_entry) = registry_entry_by_name(&inputs.texture_cache.registry, &name)
    {
        decal_pass.register_texture(
            name,
            texture_entry.view.clone(),
            texture_entry.sampler.clone(),
        );
    }
}

pub(crate) fn reserve_material_layer(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    inputs: &mut crate::render::wgpu::render_configs::RenderInputs,
    texture: crate::render::asset_id::TextureId,
    usage: crate::render::wgpu::texture_cache::TextureUsage,
    sampler: crate::render::wgpu::texture_cache::SamplerSettings,
) -> bool {
    let Some(name) =
        registry_name_for(&inputs.texture_cache.registry, texture.index).map(str::to_string)
    else {
        return false;
    };
    let Some(layer) = renderer.material_texture_arrays.reserve_layer(
        &renderer.queue,
        name,
        usage,
        sampler.wrap_u,
        sampler.wrap_v,
    ) else {
        return false;
    };
    let mapping = crate::render::wgpu::material_texture_arrays::MaterialTextureLayer {
        usage,
        layer,
        wrap_u: sampler.wrap_u,
        wrap_v: sampler.wrap_v,
    };
    if let Some(mesh_pass) = render_graph_get_pass_mut(&mut renderer.graph, "mesh_pass")
        && let Some(mesh_pass) =
            (mesh_pass as &mut dyn std::any::Any).downcast_mut::<passes::MeshPass>()
    {
        mesh_pass.add_material_layer_mapping(texture, mapping);
    }
    if let Some(skinned_mesh_pass) =
        render_graph_get_pass_mut(&mut renderer.graph, "skinned_mesh_pass")
        && let Some(skinned_mesh_pass) = (skinned_mesh_pass as &mut dyn std::any::Any)
            .downcast_mut::<passes::geometry::SkinnedMeshPass>()
    {
        skinned_mesh_pass.add_material_layer_mapping(texture, mapping);
    }
    true
}

pub(crate) fn drain_loading_tasks(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    world: &mut crate::ecs::world::World,
    inputs: &mut crate::render::wgpu::render_configs::RenderInputs,
) {
    // Collect images the worker pool finished decoding. The GPU upload below
    // is the only throttled step; decoding already ran off this thread.
    loading_pipeline_poll_decodes(
        &mut world.resources.loading.pipeline,
        crate::ecs::loading::DECODE_POLL_BUDGET,
    );

    let budget = world.resources.loading.pipeline.tasks_per_frame.max(1);
    let mut uploaded_textures: Vec<crate::render::asset_id::TextureId> = Vec::new();
    let mut deferred: Vec<crate::ecs::loading::LoadingTask> = Vec::new();
    let mut reserved_any = false;
    let mut uploads = 0usize;
    while uploads < budget {
        let Some(task) = loading_pipeline_pop(&mut world.resources.loading.pipeline) else {
            break;
        };
        let category = task.category();
        let label = match &task {
            crate::ecs::loading::LoadingTask::ReserveLayer { texture, .. }
            | crate::ecs::loading::LoadingTask::UploadDecodedTexture { texture, .. }
            | crate::ecs::loading::LoadingTask::MaterializeTexture { texture, .. } => {
                registry_name_for(&inputs.texture_cache.registry, texture.index)
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("texture {}", texture.index))
            }
        };
        match task {
            crate::ecs::loading::LoadingTask::ReserveLayer {
                texture,
                usage,
                sampler,
            } => {
                // Reserving a stable layer and writing its placeholder is cheap and
                // must happen before geometry first renders, so it does not draw from
                // the per-frame upload budget.
                if reserve_material_layer(renderer, inputs, texture, usage, sampler) {
                    reserved_any = true;
                }
            }
            crate::ecs::loading::LoadingTask::UploadDecodedTexture {
                texture,
                rgba_data,
                width,
                height,
                usage,
                sampler,
            } => {
                upload_material_texture(
                    renderer,
                    inputs,
                    MaterialTextureUploadRequest {
                        texture,
                        rgba_data: &rgba_data,
                        width,
                        height,
                        usage,
                        sampler,
                    },
                );
                uploaded_textures.push(texture);
                uploads += 1;
            }
            crate::ecs::loading::LoadingTask::MaterializeTexture {
                texture,
                recipe,
                usage,
                sampler,
            } => {
                // Hold the task until every source image it samples has finished
                // decoding in the worker pool, then upload into the reserved layer.
                if !loading_pipeline_recipe_ready(&world.resources.loading.pipeline, &recipe) {
                    deferred.push(crate::ecs::loading::LoadingTask::MaterializeTexture {
                        texture,
                        recipe,
                        usage,
                        sampler,
                    });
                    continue;
                }
                let produced = crate::ecs::loading::execute_texture_recipe(
                    &recipe,
                    &world.resources.loading.pipeline.decoded_images,
                );
                loading_pipeline_release_recipe_sources(
                    &mut world.resources.loading.pipeline,
                    &recipe,
                );
                if let Some(decoded) = produced {
                    // Capture the materialized RGBA into texture_sources for any
                    // texture whose recipe is composite (RG-pack, RGB+A pack,
                    // spec/gloss conversion). Direct-recipe textures already have
                    // their PNG bytes captured at queue_gltf_load and are skipped
                    // here via the `entry`-style insert.
                    let texture_name = world
                        .resources
                        .texture_cache
                        .registry
                        .index_to_name
                        .get(texture.index as usize)
                        .and_then(|slot| slot.clone());
                    if let Some(name) = texture_name {
                        world
                            .resources
                            .assets
                            .texture_sources
                            .entry(name)
                            .or_insert_with(|| crate::ecs::asset_state::TextureSourceBytes {
                                data: crate::ecs::asset_state::TextureSourceData::Rgba {
                                    rgba: decoded.rgba.clone(),
                                    width: decoded.width,
                                    height: decoded.height,
                                },
                                usage,
                                sampler,
                            });
                    }
                    upload_material_texture(
                        renderer,
                        inputs,
                        MaterialTextureUploadRequest {
                            texture,
                            rgba_data: &decoded.rgba,
                            width: decoded.width,
                            height: decoded.height,
                            usage,
                            sampler,
                        },
                    );
                    uploaded_textures.push(texture);
                } else {
                    tracing::warn!("texture recipe produced no data for '{}'", label);
                }
                uploads += 1;
            }
        }
        loading_pipeline_mark_completed(&mut world.resources.loading.pipeline, label, category);
    }
    for task in deferred.into_iter().rev() {
        loading_pipeline_requeue_front(&mut world.resources.loading.pipeline, task);
    }
    // Stable layers mean a streamed pixel upload never changes a material's layer
    // index, so only a fresh reservation (or a freshly inserted material) needs the
    // resolve plus instance rebuild.
    let needs_resolve = reserved_any
        || !world
            .resources
            .assets
            .material_registry
            .pending_resolve
            .is_empty();
    if needs_resolve {
        crate::ecs::material::resources::material_registry_resolve_uploaded_textures(
            &mut world.resources.assets.material_registry,
            &inputs.texture_cache,
            &uploaded_textures,
        );
        world.resources.mesh_render_state.request_full_rebuild();
    }
    if !loading_pipeline_is_active(&world.resources.loading.pipeline) {
        loading_pipeline_reset_source_arena(&mut world.resources.loading.pipeline);
    }

    let evicted =
        crate::render::wgpu::texture_cache::texture_cache_remove_unused(&mut inputs.texture_cache);
    if !evicted.is_empty() {
        for name in &evicted {
            renderer.material_texture_arrays.release(name);
            world.resources.assets.texture_sources.remove(name);
        }
        if let Some(mesh_pass) = render_graph_get_pass_mut(&mut renderer.graph, "mesh_pass")
            && let Some(mesh_pass) =
                (mesh_pass as &mut dyn std::any::Any).downcast_mut::<passes::MeshPass>()
        {
            for name in &evicted {
                mesh_pass.unregister_texture(name);
            }
        }
        if let Some(skinned_mesh_pass) =
            render_graph_get_pass_mut(&mut renderer.graph, "skinned_mesh_pass")
            && let Some(skinned_mesh_pass) = (skinned_mesh_pass as &mut dyn std::any::Any)
                .downcast_mut::<passes::geometry::SkinnedMeshPass>()
        {
            for name in &evicted {
                skinned_mesh_pass.unregister_texture(name);
            }
        }
        if let Some(decal_pass) = render_graph_get_pass_mut(&mut renderer.graph, "decal_pass")
            && let Some(decal_pass) =
                (decal_pass as &mut dyn std::any::Any).downcast_mut::<passes::DecalPass>()
        {
            for name in &evicted {
                decal_pass.unregister_texture(name);
            }
        }
        world.resources.mesh_render_state.request_full_rebuild();
    }
}

struct SwappedWindowUi;

fn swap_window_ui_frame(
    _world: &mut crate::ecs::world::World,
    _target_size: (u32, u32),
) -> SwappedWindowUi {
    SwappedWindowUi
}

fn restore_window_ui_frame(_world: &mut crate::ecs::world::World, _saved: SwappedWindowUi) {}

#[cfg(not(target_arch = "wasm32"))]
fn crop_center_square(image: image::RgbaImage) -> image::RgbaImage {
    let (width, height) = image.dimensions();
    if width == height {
        return image;
    }
    let side = width.min(height);
    let x = (width - side) / 2;
    let y = (height - side) / 2;
    image::imageops::crop_imm(&image, x, y, side, side).to_image()
}
