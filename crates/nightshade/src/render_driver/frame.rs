use crate::ecs::loading::{
    loading_pipeline_is_active, loading_pipeline_mark_completed, loading_pipeline_poll_decodes,
    loading_pipeline_pop, loading_pipeline_recipe_ready, loading_pipeline_release_recipe_sources,
    loading_pipeline_requeue_front, loading_pipeline_reset_source_arena,
};
use crate::render::generational_registry::registry_name_for;
use crate::render::wgpu::DEPTH_PICK_SAMPLE_SIZE;
use crate::render::wgpu::lights::build_render_lighting;
use crate::render::wgpu::pass_sync;
use crate::render::wgpu::passes;
use crate::render::wgpu::picking::dispatch_pick_compute;
use crate::render::wgpu::presentation;
use crate::render::wgpu::render_configs::{
    CameraFrameInputs, FrameOutputs, RenderInputs, RendererCommand,
};
use crate::render::wgpu::rendergraph;
use crate::render::wgpu::rendergraph::{
    render_graph_enable_only_passes, render_graph_execute,
    render_graph_restore_pass_enabled_states, render_graph_save_pass_enabled_states,
    render_graph_set_external_texture, render_graph_set_pass_enabled,
};
use crate::render::wgpu::texture_uploads;
use crate::render::wgpu::view::build_render_view;

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

pub fn render_frame(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    world: &mut crate::ecs::world::World,
) -> Result<(), Box<dyn std::error::Error>> {
    poll_depth_pick_readback(renderer, world);
    drain_loading_tasks(renderer, world);
    super::extract::prepare_text_meshes(renderer, world);
    #[cfg(feature = "egui")]
    super::egui::prepare_egui_pass(renderer, world);

    let mut inputs = super::inputs::compose_render_inputs(world);
    let mut outputs = FrameOutputs::default();
    let result = render_frame_inner(renderer, &mut inputs, &mut outputs);
    super::inputs::restore_render_inputs(world, inputs, outputs);
    result
}

fn render_frame_inner(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    inputs: &mut RenderInputs,
    outputs: &mut FrameOutputs,
) -> Result<(), Box<dyn std::error::Error>> {
    let _span = tracing::info_span!("render_frame").entered();
    pass_sync::sync_gpu_profile(renderer, inputs);
    renderer.frame_state.index = renderer.frame_state.index.wrapping_add(1);
    pass_sync::sync_taa_jitter(renderer, inputs);

    if let Some(matrices) = inputs
        .active_camera_frame
        .as_ref()
        .and_then(|camera| camera.matrices)
    {
        let view_projection: [[f32; 4]; 4] = (matrices.projection * matrices.view).into();
        inputs.scene.prev_view_projection = inputs.scene.view_projection;
        inputs.scene.view_projection = view_projection;
    }

    renderer.sync_present_mode(inputs.settings.vsync_enabled);
    pass_sync::sync_settings_version(renderer, inputs);

    let frame_time_ms = inputs.view.delta_time * 1000.0;
    if frame_time_ms > 0.0 {
        inputs
            .scene
            .adaptive_sampling
            .record_frame_time(frame_time_ms);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "screenshot"))]
    presentation::poll_screenshot_readback(renderer);

    renderer.resize_render_buffers(
        renderer.surface_config.width,
        renderer.surface_config.height,
    );

    let mut hdr_skybox_commands = Vec::new();
    #[cfg(all(not(target_arch = "wasm32"), feature = "screenshot"))]
    let mut screenshot_request: Option<(Option<std::path::PathBuf>, Option<u32>)> = None;

    for command in std::mem::take(&mut inputs.commands) {
        match command {
            RendererCommand::UploadUiImageLayer {
                layer,
                rgba_data,
                width,
                height,
            } => {
                renderer.ui_texture_array.upload_layer(
                    &renderer.queue,
                    layer,
                    &rgba_data,
                    width,
                    height,
                );
            }
            RendererCommand::ReloadTexture {
                name,
                rgba_data,
                width,
                height,
            } => {
                texture_uploads::reload_material_texture(
                    renderer,
                    &mut inputs.texture_cache,
                    name,
                    &rgba_data,
                    width,
                    height,
                );
            }
            RendererCommand::LoadHdrSkybox { hdr_data } => {
                hdr_skybox_commands.push(hdr_data);
            }
            RendererCommand::SetColorLut { data } => {
                pass_sync::set_color_lut(renderer, data);
            }
            #[cfg(all(not(target_arch = "wasm32"), feature = "screenshot"))]
            RendererCommand::CaptureScreenshot {
                path,
                max_dimension,
            } => {
                screenshot_request = Some((path, max_dimension));
            }
            #[cfg(not(all(not(target_arch = "wasm32"), feature = "screenshot")))]
            RendererCommand::CaptureScreenshot { .. } => {}
        }
    }

    #[cfg(feature = "assets")]
    crate::render::wgpu::ibl::load_hdr_skyboxes(
        renderer,
        &mut inputs.ibl_views,
        hdr_skybox_commands,
    );
    #[cfg(not(feature = "assets"))]
    drop(hdr_skybox_commands);

    #[cfg(feature = "assets")]
    crate::render::wgpu::ibl::sync_procedural_ibl(renderer, inputs);

    let mesh_frame_state = inputs.mesh_frame_state.take();
    if let Some(frame_state) = mesh_frame_state {
        pass_sync::apply_frame_dirty_state(renderer, frame_state, &mut inputs.mesh_cache);
    }

    if let Some(lines_pass) = pass_sync::lines_pass_mut(&mut renderer.graph) {
        lines_pass.upload_lines(&renderer.device, &renderer.queue, &inputs.debug_lines.lines);
        match &inputs.debug_lines.bounding_volumes {
            Some(bounding_volumes) => {
                lines_pass.upload_bounding_volumes(
                    &renderer.device,
                    &renderer.queue,
                    bounding_volumes,
                );
            }
            None => lines_pass.clear_bounding_volumes(),
        }
        match &inputs.debug_lines.normals {
            Some(normals) => {
                lines_pass.upload_normals(&renderer.device, &renderer.queue, normals);
            }
            None => lines_pass.clear_normals(),
        }
    }

    pass_sync::sync_point_shadow_cubemap(renderer);

    crate::render::wgpu::ibl::apply_ibl_bracket_blend(renderer, inputs);

    let Some(surface_texture) = presentation::acquire_surface_frame(renderer) else {
        return Ok(());
    };

    let active_camera_entities: std::collections::HashSet<freecs::Entity> =
        inputs.cameras.iter().map(|camera| camera.entity).collect();
    renderer.cleanup_unused_camera_viewports(&active_camera_entities);

    dispatch_window_renders(renderer, inputs, outputs)?;

    #[cfg(all(not(target_arch = "wasm32"), feature = "screenshot"))]
    if let Some((path, max_dimension)) = screenshot_request {
        presentation::request_screenshot_copy(renderer, path, max_dimension);
    }

    surface_texture.present();

    Ok(())
}

pub fn configure_with_state(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    state: &mut dyn crate::run::State,
) -> Result<(), Box<dyn std::error::Error>> {
    pass_sync::rebind_glyph_atlas(renderer);

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

    presentation::install_presentation_passes(renderer)
}

pub fn update_with_state(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    state: &mut dyn crate::run::State,
    world: &mut crate::ecs::world::World,
) -> Result<(), Box<dyn std::error::Error>> {
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

/// Gathers each cloth's mesh name and widened cull bounds from the world and
/// hands the cloth pass its vertex buffer write targets for this frame.
pub(crate) fn sync_cloth_write_targets(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    world: &crate::ecs::world::World,
) {
    use crate::ecs::world::{CLOTH, RENDER_MESH};

    let mut cloths: Vec<pass_sync::ClothWriteBounds> = Vec::new();
    world
        .core
        .query()
        .with(CLOTH | RENDER_MESH)
        .iter(|_entity, table, index| {
            let cloth = &table.cloth[index];
            let bounds = crate::ecs::cloth::systems::cloth_bounding_volume(cloth);
            cloths.push(pass_sync::ClothWriteBounds {
                mesh_name: table.render_mesh[index].name.clone(),
                center: [
                    bounds.obb.center.x,
                    bounds.obb.center.y,
                    bounds.obb.center.z,
                ],
                sphere_radius: bounds.sphere_radius,
            });
        });
    pass_sync::set_cloth_write_targets(renderer, &cloths);
}

/// Polls a pending GPU pick readback and, once the staging buffer is mapped,
/// stores the depth and entity-id samples on the picking resource and
/// resolves the picked world position through the requesting camera.
pub(crate) fn poll_depth_pick_readback(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    world: &mut crate::ecs::world::World,
) {
    if !renderer.depth_pick.pending {
        return;
    }
    let _ = renderer.device.poll(wgpu::PollType::Poll);

    if !renderer
        .depth_pick
        .map_complete
        .load(std::sync::atomic::Ordering::Relaxed)
    {
        return;
    }
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

fn dispatch_window_renders(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    inputs: &mut RenderInputs,
    outputs: &mut FrameOutputs,
) -> Result<(), Box<dyn std::error::Error>> {
    let _span =
        tracing::info_span!("dispatch_window_renders", cameras = inputs.cameras.len(),).entered();

    let surface_width = renderer.surface_config.width;
    let surface_height = renderer.surface_config.height;
    let has_viewports = !inputs.cameras.is_empty();
    let _ = render_graph_set_pass_enabled(&mut renderer.graph, "viewport_blit_pass", has_viewports);
    let _ = render_graph_set_pass_enabled(&mut renderer.graph, "viewport_compose_pass", true);

    if !has_viewports {
        renderer.resize_render_buffers_for_window(surface_width, surface_height);
    }

    inputs.view.camera_tile_render_iteration = 0;
    let original_active_camera = inputs.view.active_camera;
    let frame_settings_version = inputs.scene.settings_version;
    let focus_policy = inputs.scene.focus_policy;
    let global_dirty_signal = inputs.global_dirty_signal;

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

    let mouse_focused_camera = inputs
        .view
        .camera_tile_rects
        .iter()
        .find(|(_, rect)| rect.contains(inputs.mouse_position))
        .map(|(entity, _)| *entity);
    let focused_camera = mouse_focused_camera.or(original_active_camera);

    texture_uploads::sync_material_texture_bindings(
        renderer,
        inputs.settings.material_anisotropy_filtering,
    );

    let cameras: Vec<CameraFrameInputs> = inputs.cameras.clone();
    for (iteration_index, camera) in cameras.iter().enumerate() {
        let camera_entity = camera.entity;
        let (tile_width, tile_height) = inputs
            .view
            .camera_tile_rects
            .get(&camera_entity)
            .map(|rect| {
                (
                    (rect.width.round() as u32).max(1),
                    (rect.height.round() as u32).max(1),
                )
            })
            .unwrap_or((surface_width.max(1), surface_height.max(1)));
        let (display_width, display_height) = if let Some(aspect) = camera.constrained_aspect {
            fit_constrained_aspect(tile_width, tile_height, aspect)
        } else {
            (tile_width, tile_height)
        };
        let render_scale = inputs.settings.render_scale.clamp(0.25, 4.0);
        let is_main_viewport_camera = Some(camera_entity) == original_active_camera;
        let render_width = ((display_width as f32 * render_scale).round() as u32).max(1);
        let render_height = ((display_height as f32 * render_scale).round() as u32).max(1);

        renderer.ensure_camera_viewport(camera_entity, render_width, render_height);
        renderer.resize_render_buffers_for_window(render_width, render_height);

        let viewport_cloned = renderer
            .camera_viewports
            .get(&camera_entity)
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

            inputs.view.active_camera = Some(camera_entity);
            inputs.view.camera_tile_render_iteration = iteration_index as u32;

            let effective = camera.effective_shading;

            let is_focused_view = Some(camera_entity) == focused_camera;
            let focus_forces_render = matches!(
                focus_policy,
                crate::render::config::ViewportFocusPolicy::FocusedAlways
            ) && is_focused_view;
            let view_changed = last_active_view != Some(effective);
            let settings_changed = last_settings_version != frame_settings_version;
            let camera_self_dirty = match (last_camera_world_transform, camera.world_transform) {
                (Some(prev), Some(curr)) => prev != curr,
                (None, _) => true,
                (_, None) => false,
            };
            let frustum_dirty = if let Some(matrices) = camera.matrices.as_ref() {
                let view_proj = matrices.projection * matrices.view;
                let frustum_planes = passes::geometry::extract_frustum_planes(&view_proj);
                inputs.dirty_world_spheres.iter().any(|(center, radius)| {
                    passes::geometry::sphere_in_frustum(center, *radius, &frustum_planes)
                })
            } else {
                !inputs.dirty_world_spheres.is_empty()
            };
            let pick_forces_render = is_main_viewport_camera && inputs.pick_request.is_some();
            let camera_force_render = inputs.force_render_cameras.contains(&camera_entity);
            let should_render = pick_forces_render
                || camera_force_render
                || match camera.update_mode {
                    crate::render::config::ViewportUpdateMode::Always => true,
                    crate::render::config::ViewportUpdateMode::WhenVisible => true,
                    crate::render::config::ViewportUpdateMode::WhenDirty => {
                        focus_forces_render
                            || !has_rendered_once
                            || global_dirty_signal
                            || settings_changed
                            || view_changed
                            || camera_self_dirty
                            || frustum_dirty
                    }
                    crate::render::config::ViewportUpdateMode::Once => !has_rendered_once,
                    crate::render::config::ViewportUpdateMode::Disabled => !has_rendered_once,
                };

            inputs.scene.active_view = effective;
            inputs.scene.render_view = build_render_view(
                camera,
                inputs.scene.taa_jitter,
                (render_width, render_height),
            );
            inputs.scene.render_lighting = Some(build_render_lighting(&inputs.scene, Some(camera)));
            pass_sync::sync_conditional_pass_toggles(
                renderer,
                inputs,
                inputs.pick_request.is_some(),
                inputs.has_skinned_meshes,
            );

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
                && let Some(viewport) = renderer.camera_viewports.get_mut(&camera_entity)
            {
                viewport.has_rendered_at_least_once = true;
                viewport.last_active_view = Some(effective);
                viewport.last_settings_version = frame_settings_version;
                viewport.last_render_frame = renderer.frame_state.index;
                viewport.last_camera_world_transform = camera.world_transform;
            }
            if should_render && camera_force_render {
                outputs.force_render_cleared.push(camera_entity);
            }

            if is_main_viewport_camera
                && !renderer.depth_pick.pending
                && let Some((screen_x, screen_y)) = inputs.pick_request.take()
            {
                dispatch_pick_compute(renderer, screen_x, screen_y, Some(camera_entity));
            }

            outputs.viewport_texture_sizes.push(size);
        }
    }

    inputs.view.active_camera = original_active_camera;
    inputs.scene.active_view = match inputs
        .active_camera_frame
        .as_ref()
        .filter(|_| inputs.cameras.is_empty())
    {
        Some(camera) => camera.effective_shading,
        None => inputs.fallback_shading,
    };
    pass_sync::sync_conditional_pass_toggles(
        renderer,
        inputs,
        inputs.pick_request.is_some(),
        inputs.has_skinned_meshes,
    );

    if let Some((ui_was_enabled, ui_image_was_enabled)) = ui_pass_states_before_loop {
        let _span = tracing::info_span!("ui_composite").entered();

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
            &[
                ("ui_pass", ui_was_enabled),
                ("ui_image_pass", ui_image_was_enabled),
            ],
        );
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

        render_graph_restore_pass_enabled_states(&mut renderer.graph, &saved_pass_states);
        let _ = render_graph_set_pass_enabled(&mut renderer.graph, "ui_pass", ui_was_enabled);
        let _ = render_graph_set_pass_enabled(
            &mut renderer.graph,
            "ui_image_pass",
            ui_image_was_enabled,
        );
    }

    if inputs.cameras.is_empty() {
        inputs.view.camera_tile_render_iteration = 0;

        if inputs.settings.render_world_to_swapchain {
            inputs.scene.render_view = inputs.active_camera_frame.as_ref().and_then(|camera| {
                build_render_view(camera, inputs.scene.taa_jitter, renderer.render_buffer_size)
            });
            inputs.scene.render_lighting = Some(build_render_lighting(
                &inputs.scene,
                inputs.active_camera_frame.as_ref(),
            ));
            let command_buffers = render_graph_execute(
                &mut renderer.graph,
                &renderer.device,
                &renderer.queue,
                inputs,
            )?;
            renderer.queue.submit(command_buffers);
            if !renderer.depth_pick.pending
                && let Some((screen_x, screen_y)) = inputs.pick_request.take()
            {
                dispatch_pick_compute(renderer, screen_x, screen_y, inputs.view.active_camera);
            }
        } else {
            presentation::clear_swapchain(renderer, inputs.ui.background_color);

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

            let command_buffers = render_graph_execute(
                &mut renderer.graph,
                &renderer.device,
                &renderer.queue,
                inputs,
            )?;
            renderer.queue.submit(command_buffers);

            render_graph_restore_pass_enabled_states(&mut renderer.graph, &saved_pass_states);
            let _ = render_graph_set_pass_enabled(&mut renderer.graph, "ui_pass", true);
            let _ = render_graph_set_pass_enabled(&mut renderer.graph, "ui_image_pass", true);
        }
    }

    let dispatched_settings_version = inputs.scene.settings_version;
    renderer.record_window_dispatch(dispatched_settings_version);
    Ok(())
}

pub(crate) fn drain_loading_tasks(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    world: &mut crate::ecs::world::World,
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
                registry_name_for(&world.resources.texture_cache.registry, texture.index)
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
                if texture_uploads::reserve_material_layer(
                    renderer,
                    &world.resources.texture_cache,
                    texture,
                    usage,
                    sampler,
                ) {
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
                texture_uploads::upload_material_texture(
                    renderer,
                    &mut world.resources.texture_cache,
                    texture_uploads::MaterialTextureUploadRequest {
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
                    texture_uploads::upload_material_texture(
                        renderer,
                        &mut world.resources.texture_cache,
                        texture_uploads::MaterialTextureUploadRequest {
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
            &world.resources.texture_cache,
            &uploaded_textures,
        );
        world.resources.mesh_render_state.request_full_rebuild();
    }
    if !loading_pipeline_is_active(&world.resources.loading.pipeline) {
        loading_pipeline_reset_source_arena(&mut world.resources.loading.pipeline);
    }

    let evicted = crate::render::wgpu::texture_cache::texture_cache_remove_unused(
        &mut world.resources.texture_cache,
    );
    if !evicted.is_empty() {
        texture_uploads::unregister_evicted_textures(renderer, &evicted);
        for name in &evicted {
            world.resources.assets.texture_sources.remove(name);
        }
        world.resources.mesh_render_state.request_full_rebuild();
    }
}
