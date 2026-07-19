//! The default frame driver: renders one frame from a composed
//! [`RenderInputs`], filling [`FrameOutputs`] with the writes that belong to
//! the caller. Any host that can compose the inputs and own a surface can
//! call [`render_frame`] directly; the graph executes per camera with
//! viewport caching, then the retained UI composites over the result.

use crate::wgpu::lights::build_render_lighting;
use crate::wgpu::pass_sync;
use crate::wgpu::passes;
use crate::wgpu::picking::dispatch_pick_compute;
use crate::wgpu::presentation;
use crate::wgpu::render_configs::{CameraFrameInputs, FrameOutputs, RenderInputs, RendererCommand};
use crate::wgpu::rendergraph;
use crate::wgpu::rendergraph::{
    render_graph_enable_only_passes, render_graph_execute,
    render_graph_restore_pass_enabled_states, render_graph_save_pass_enabled_states,
    render_graph_set_external_texture, render_graph_set_pass_enabled,
};
use crate::wgpu::texture_uploads;
use crate::wgpu::view::build_render_view;

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

/// Renders one frame. The renderer's persistent [`IblViews`](crate::config::IblViews)
/// are swapped into the inputs for the duration of the frame and reclaimed
/// afterward, so the caller never holds image-based-lighting state.
pub fn render_frame(
    renderer: &mut crate::wgpu::WgpuRenderer,
    inputs: &mut RenderInputs,
    outputs: &mut FrameOutputs,
) -> Result<(), Box<dyn std::error::Error>> {
    std::mem::swap(&mut renderer.ibl_views, &mut inputs.ibl_views);
    let result = run_frame(renderer, inputs, outputs);
    std::mem::swap(&mut renderer.ibl_views, &mut inputs.ibl_views);
    result
}

fn run_frame(
    renderer: &mut crate::wgpu::WgpuRenderer,
    inputs: &mut RenderInputs,
    outputs: &mut FrameOutputs,
) -> Result<(), Box<dyn std::error::Error>> {
    let _span = tracing::info_span!("render_frame").entered();
    pass_sync::sync_gpu_profile(renderer, inputs);
    renderer.frame_state.index = renderer.frame_state.index.wrapping_add(1);
    pass_sync::sync_taa_jitter(renderer, inputs);

    if let Some(matrices) = inputs
        .frame
        .active_camera_frame
        .as_ref()
        .and_then(|camera| camera.matrices)
    {
        let view_projection: [[f32; 4]; 4] = (matrices.projection * matrices.view).into();
        inputs.scene.prev_view_projection = inputs.scene.view_projection;
        inputs.scene.view_projection = view_projection;
    }

    inputs.shadow_atlas = passes::shadow_depth::atlas::assign_spotlight_atlas(
        &inputs.scene_world,
        inputs.frame.active_camera_frame.as_ref(),
        inputs.view.camera_tile_rects.is_empty(),
        passes::shadow_depth::atlas::shadow_atlas_size(),
    );

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

    for command in std::mem::take(&mut inputs.frame.commands) {
        match command {
            RendererCommand::UploadUiImageLayer {
                layer,
                rgba_data,
                width,
                height,
            } => {
                crate::wgpu::texture_array_pool::upload_layer(
                    &renderer.ui_texture_array,
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

    #[cfg(feature = "hdr")]
    crate::wgpu::ibl::load_hdr_skyboxes(renderer, &mut inputs.ibl_views, hdr_skybox_commands);
    #[cfg(not(feature = "hdr"))]
    drop(hdr_skybox_commands);

    #[cfg(feature = "hdr")]
    crate::wgpu::ibl::sync_procedural_ibl(renderer, inputs);

    if let Some(lines_pass) = pass_sync::lines_pass_mut(&mut renderer.graph) {
        lines_pass.upload_lines(
            &renderer.device,
            &renderer.queue,
            &inputs.frame.debug_lines.lines,
        );
        match &inputs.frame.debug_lines.bounding_volumes {
            Some(bounding_volumes) => {
                lines_pass.upload_bounding_volumes(
                    &renderer.device,
                    &renderer.queue,
                    bounding_volumes,
                );
            }
            None => lines_pass.clear_bounding_volumes(),
        }
        match &inputs.frame.debug_lines.normals {
            Some(normals) => {
                lines_pass.upload_normals(&renderer.device, &renderer.queue, normals);
            }
            None => lines_pass.clear_normals(),
        }
    }

    pass_sync::sync_point_shadow_cubemap(renderer);

    crate::wgpu::ibl::apply_ibl_bracket_blend(renderer, inputs);

    let Some(surface_texture) = presentation::acquire_surface_frame(renderer) else {
        return Ok(());
    };
    outputs.frame_executed = true;

    // Taken only after a surface frame is secured: on a skipped frame the
    // state stays in the inputs and the caller's restore merges it back, so
    // dirty marks captured while occluded are not lost.
    if let Some(frame_state) = inputs.frame.mesh_frame_state.take() {
        pass_sync::apply_frame_dirty_state(renderer, frame_state, &mut inputs.mesh_cache);
    }

    let active_camera_entities: std::collections::HashSet<freecs::Entity> = inputs
        .frame
        .cameras
        .iter()
        .map(|camera| camera.entity)
        .collect();
    renderer.cleanup_unused_camera_viewports(&active_camera_entities);

    dispatch_window_renders(renderer, inputs, outputs)?;

    #[cfg(all(not(target_arch = "wasm32"), feature = "screenshot"))]
    if let Some((path, max_dimension)) = screenshot_request {
        presentation::request_screenshot_copy(renderer, path, max_dimension);
    }

    surface_texture.present();

    Ok(())
}

fn dispatch_window_renders(
    renderer: &mut crate::wgpu::WgpuRenderer,
    inputs: &mut RenderInputs,
    outputs: &mut FrameOutputs,
) -> Result<(), Box<dyn std::error::Error>> {
    let _span = tracing::info_span!(
        "dispatch_window_renders",
        cameras = inputs.frame.cameras.len(),
    )
    .entered();

    let surface_width = renderer.surface_config.width;
    let surface_height = renderer.surface_config.height;
    let has_viewports = !inputs.frame.cameras.is_empty();
    let _ = render_graph_set_pass_enabled(&mut renderer.graph, "viewport_blit_pass", has_viewports);
    let _ = render_graph_set_pass_enabled(&mut renderer.graph, "viewport_compose_pass", true);

    if !has_viewports {
        renderer.resize_render_buffers_for_window(surface_width, surface_height);
    }

    inputs.view.camera_tile_render_iteration = 0;
    let original_active_camera = inputs.view.active_camera;
    let frame_settings_version = inputs.scene.settings_version;
    let focus_policy = inputs.scene.focus_policy;
    let global_dirty_signal = inputs.frame.global_dirty_signal;

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
        .find(|(_, rect)| rect.contains(inputs.frame.mouse_position))
        .map(|(entity, _)| *entity);
    let focused_camera = mouse_focused_camera.or(original_active_camera);

    texture_uploads::sync_material_texture_bindings(
        renderer,
        inputs.settings.material_anisotropy_filtering,
    );

    let cameras: Vec<CameraFrameInputs> = inputs.frame.cameras.clone();

    // Freezing captures the current view once and holds it, so the cut stays put
    // as the camera flies away from the frustum that chose it. Capturing every
    // frame instead would track the camera and freeze nothing. Cleared the
    // moment freezing turns off. The source is the active camera, whether the
    // frame renders one camera or several. Surface resolution sets the frustum
    // and the pixels-per-unit the cull needs.
    if inputs.debug_draw.meshlet_freeze_cull_to_active {
        if renderer.meshlet_frozen_cull_view.is_none() {
            let source = original_active_camera
                .and_then(|active| cameras.iter().find(|camera| camera.entity == active))
                .or(inputs.frame.active_camera_frame.as_ref());
            renderer.meshlet_frozen_cull_view = source.and_then(|camera| {
                build_render_view(
                    camera,
                    inputs.scene.taa_jitter,
                    (surface_width.max(1), surface_height.max(1)),
                )
            });
        }
    } else {
        renderer.meshlet_frozen_cull_view = None;
    }
    inputs.scene.frozen_cull_view = renderer.meshlet_frozen_cull_view.clone();

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
                crate::config::ViewportFocusPolicy::FocusedAlways
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
                inputs
                    .frame
                    .dirty_world_spheres
                    .iter()
                    .any(|(center, radius)| {
                        passes::geometry::sphere_in_frustum(center, *radius, &frustum_planes)
                    })
            } else {
                !inputs.frame.dirty_world_spheres.is_empty()
            };
            let pick_forces_render = is_main_viewport_camera && inputs.frame.pick_request.is_some();
            let camera_force_render = inputs.frame.force_render_cameras.contains(&camera_entity);
            let should_render = pick_forces_render
                || camera_force_render
                || match camera.update_mode {
                    crate::config::ViewportUpdateMode::Always => true,
                    crate::config::ViewportUpdateMode::WhenVisible => true,
                    crate::config::ViewportUpdateMode::WhenDirty => {
                        focus_forces_render
                            || !has_rendered_once
                            || global_dirty_signal
                            || settings_changed
                            || view_changed
                            || camera_self_dirty
                            || frustum_dirty
                    }
                    crate::config::ViewportUpdateMode::Once => !has_rendered_once,
                    crate::config::ViewportUpdateMode::Disabled => !has_rendered_once,
                };

            inputs.scene.active_view = effective;
            inputs.scene.render_view = build_render_view(
                camera,
                inputs.scene.taa_jitter,
                (render_width, render_height),
            );
            let lighting = build_render_lighting(&inputs.scene_world, Some(camera));
            inputs.scene.render_lighting = Some(lighting);
            pass_sync::sync_conditional_pass_toggles(
                renderer,
                inputs,
                inputs.frame.pick_request.is_some(),
                inputs.frame.has_skinned_meshes,
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
                && let Some((screen_x, screen_y)) = inputs.frame.pick_request.take()
            {
                dispatch_pick_compute(renderer, screen_x, screen_y, Some(camera_entity));
            }

            outputs.viewport_texture_sizes.push(size);
        }
    }

    inputs.view.active_camera = original_active_camera;
    inputs.scene.active_view = match inputs
        .frame
        .active_camera_frame
        .as_ref()
        .filter(|_| inputs.frame.cameras.is_empty())
    {
        Some(camera) => camera.effective_shading,
        None => inputs.frame.fallback_shading,
    };
    pass_sync::sync_conditional_pass_toggles(
        renderer,
        inputs,
        inputs.frame.pick_request.is_some(),
        inputs.frame.has_skinned_meshes,
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

    if inputs.frame.cameras.is_empty() {
        inputs.view.camera_tile_render_iteration = 0;

        if inputs.settings.render_world_to_swapchain {
            inputs.scene.render_view =
                inputs
                    .frame
                    .active_camera_frame
                    .as_ref()
                    .and_then(|camera| {
                        build_render_view(
                            camera,
                            inputs.scene.taa_jitter,
                            renderer.render_buffer_size,
                        )
                    });
            let lighting = build_render_lighting(
                &inputs.scene_world,
                inputs.frame.active_camera_frame.as_ref(),
            );
            inputs.scene.render_lighting = Some(lighting);
            let command_buffers = render_graph_execute(
                &mut renderer.graph,
                &renderer.device,
                &renderer.queue,
                inputs,
            )?;
            renderer.queue.submit(command_buffers);
            if !renderer.depth_pick.pending
                && let Some((screen_x, screen_y)) = inputs.frame.pick_request.take()
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
