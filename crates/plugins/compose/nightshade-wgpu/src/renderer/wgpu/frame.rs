use crate::renderer::wgpu::caps::{CASCADE_SLOT_RESOLUTION, MAX_AREA_LIGHTS, MAX_LIGHTS};
use crate::renderer::wgpu::lights::build_render_lighting;
use crate::renderer::wgpu::pass_sync;
use crate::renderer::wgpu::picking::dispatch_pick_compute;
use crate::renderer::wgpu::presentation;
use crate::renderer::wgpu::render_configs::{
    CameraFrameInputs, CameraTarget, FrameOutputs, RenderInputs, RendererCommand,
};
use crate::renderer::wgpu::rendergraph;
use crate::renderer::wgpu::rendergraph::{
    PassStage, render_graph_enable_only_passes, render_graph_execute,
    render_graph_pass_names_in_stage, render_graph_restore_pass_enabled_states,
    render_graph_save_pass_enabled_states, render_graph_set_external_texture,
    render_graph_set_pass_enabled,
};
use crate::renderer::wgpu::texture_uploads;
use crate::renderer::wgpu::view::build_render_view;

pub type FrameInputsFn =
    Box<dyn FnMut(&mut crate::renderer::wgpu::WgpuRenderer, &mut RenderInputs)>;

fn run_frame_inputs_hooks(
    hooks: &mut [FrameInputsFn],
    renderer: &mut crate::renderer::wgpu::WgpuRenderer,
    inputs: &mut RenderInputs,
) {
    for hook in hooks.iter_mut() {
        hook(renderer, inputs);
    }
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

pub fn render_frame(
    renderer: &mut crate::renderer::wgpu::WgpuRenderer,
    hooks: &mut [FrameInputsFn],
    inputs: &mut RenderInputs,
    outputs: &mut FrameOutputs,
) -> Result<(), crate::renderer::wgpu::rendergraph::RenderGraphError> {
    if let Some(open) = renderer.timing.open(&renderer.device) {
        renderer.queue.submit(std::iter::once(open));
    }

    let result = run_frame(renderer, hooks, inputs, outputs);

    if let Some(close) = renderer.timing.close(&renderer.device) {
        renderer.queue.submit(std::iter::once(close));
    }
    renderer.timing.poll();

    result
}

fn run_frame(
    renderer: &mut crate::renderer::wgpu::WgpuRenderer,
    hooks: &mut [FrameInputsFn],
    inputs: &mut RenderInputs,
    outputs: &mut FrameOutputs,
) -> Result<(), crate::renderer::wgpu::rendergraph::RenderGraphError> {
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

    inputs.owned.shadow_atlas = nightshade_render::queries::shadow::assign_spotlight_atlas(
        &inputs.scene_world,
        inputs.frame.active_camera_frame.as_ref(),
        inputs.view.camera_tile_rects.is_empty(),
        crate::renderer::wgpu::caps::SPOTLIGHT_ATLAS_SIZE,
        crate::renderer::wgpu::caps::SPOTLIGHT_ATLAS_SLOTS_PER_ROW,
        crate::renderer::wgpu::caps::MAX_SPOTLIGHT_SHADOWS,
    );

    let camera_position = inputs
        .scene
        .render_view
        .as_ref()
        .map(|view| view.camera_position)
        .unwrap_or_else(|| nalgebra_glm::vec3(0.0, 0.0, 0.0));
    let camera_frustum = if inputs.view.camera_tile_rects.is_empty() {
        inputs.scene.render_view.as_ref().map(|view| {
            let view_projection: [[f32; 4]; 4] = (view.projection * view.view).into();
            nightshade_render::queries::projection::full_frustum_planes(&view_projection)
        })
    } else {
        None
    };
    inputs.owned.point_shadows = nightshade_render::queries::shadow::assign_point_shadows(
        &inputs.scene_world,
        camera_position,
        camera_frustum.as_ref(),
        crate::renderer::wgpu::caps::point_shadow_slots(&renderer.device),
    );

    renderer.sync_present_mode(inputs.vsync);
    pass_sync::sync_settings_version(renderer, inputs);

    let frame_time_ms = inputs.view.delta_time * 1000.0;
    if frame_time_ms > 0.0 {
        inputs
            .scene
            .adaptive_sampling
            .record_frame_time(frame_time_ms);
    }

    #[cfg(not(target_arch = "wasm32"))]
    presentation::poll_screenshot_readback(renderer);

    if inputs.frame.cameras.is_empty() {
        renderer.resize_render_buffers(
            renderer.surface_config.width,
            renderer.surface_config.height,
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    let mut screenshot_request: Option<(Option<std::path::PathBuf>, Option<u32>)> = None;

    for command in std::mem::take(&mut inputs.frame.commands) {
        match command {
            RendererCommand::ReloadTexture {
                name,
                rgba_data,
                width,
                height,
            } => {
                texture_uploads::reload_material_texture(
                    renderer,
                    &inputs.texture_cache,
                    name,
                    &rgba_data,
                    width,
                    height,
                );
            }
            RendererCommand::LoadHdrSkybox { hdr_data } => {
                inputs.hdr_skybox_data.push(hdr_data);
            }
            RendererCommand::SetColorLut { data } => {
                pass_sync::set_color_lut(renderer, data);
            }
            #[cfg(not(target_arch = "wasm32"))]
            RendererCommand::CaptureScreenshot {
                path,
                max_dimension,
            } => {
                screenshot_request = Some((path, max_dimension));
            }
            #[cfg(target_arch = "wasm32")]
            RendererCommand::CaptureScreenshot { .. } => {
                tracing::warn!(
                    "CaptureScreenshot ignored: renderer built without the `screenshot` feature"
                );
            }
        }
    }

    run_frame_inputs_hooks(hooks, renderer, inputs);

    let surface_texture = presentation::acquire_surface_frame(renderer);
    let has_surface = surface_texture.is_some();
    if !has_surface && inputs.frame.cameras.is_empty() {
        return Ok(());
    }
    outputs.frame_executed = true;

    inputs.dirty_meshes =
        nightshade_render::commands::mesh_cache::mesh_cache_take_dirty(&mut inputs.mesh_cache);
    inputs.dirty_skinned_meshes =
        nightshade_render::commands::mesh_cache::mesh_cache_take_dirty_skinned(
            &mut inputs.mesh_cache,
        );

    let active_camera_entities: std::collections::HashSet<nightshade::ecs::Entity> = inputs
        .frame
        .cameras
        .iter()
        .map(|camera| camera.entity)
        .chain(inputs.view.active_camera)
        .collect();
    renderer.cleanup_unused_camera_viewports(&active_camera_entities);

    std::mem::swap(&mut renderer.texture_store, &mut inputs.owned.texture_store);
    let dispatch_result = dispatch_window_renders(renderer, inputs, outputs, has_surface);
    std::mem::swap(&mut renderer.texture_store, &mut inputs.owned.texture_store);
    dispatch_result?;

    #[cfg(not(target_arch = "wasm32"))]
    if let Some((path, max_dimension)) = screenshot_request {
        presentation::request_screenshot_copy(renderer, path, max_dimension);
    }

    if let Some(surface_texture) = surface_texture {
        surface_texture.present();
    }

    Ok(())
}

fn publish_camera_depth(
    renderer: &mut crate::renderer::wgpu::WgpuRenderer,
    camera_entity: nightshade::ecs::Entity,
    width: u32,
    height: u32,
) {
    let Some((destination, array_layer)) = renderer
        .camera_viewports
        .get(&camera_entity)
        .and_then(|viewport| viewport.depth_target.clone())
    else {
        return;
    };
    let Some(source) = crate::renderer::wgpu::rendergraph::render_graph_get_texture(
        &renderer.graph,
        renderer.targets.depth,
    )
    .cloned() else {
        return;
    };

    let mut encoder = renderer
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("camera depth publish"),
        });
    encoder.copy_texture_to_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &source,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyTextureInfo {
            texture: &destination,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: 0,
                y: 0,
                z: array_layer,
            },
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    renderer.queue.submit(std::iter::once(encoder.finish()));
}

struct RedirectedSurface {
    texture: wgpu::Texture,
    paint_size: (u32, u32),
}

fn redirect_paint_target(
    renderer: &mut crate::renderer::wgpu::WgpuRenderer,
) -> Option<RedirectedSurface> {
    let (texture, view, size) = renderer
        .paint_target
        .as_ref()
        .map(|target| (target.texture.clone(), target.view.clone(), target.size))?;
    let surface_texture = crate::renderer::wgpu::rendergraph::render_graph_get_texture(
        &renderer.graph,
        renderer.targets.swapchain,
    )
    .cloned()?;
    render_graph_set_external_texture(
        &mut renderer.graph,
        renderer.targets.swapchain,
        Some(texture),
        view,
        size.0,
        size.1,
    );
    Some(RedirectedSurface {
        texture: surface_texture,
        paint_size: size,
    })
}

fn restore_surface_target(
    renderer: &mut crate::renderer::wgpu::WgpuRenderer,
    redirected: RedirectedSurface,
    surface_width: u32,
    surface_height: u32,
) {
    let view = redirected
        .texture
        .create_view(&wgpu::TextureViewDescriptor {
            format: Some(renderer.surface_format),
            ..Default::default()
        });
    render_graph_set_external_texture(
        &mut renderer.graph,
        renderer.targets.swapchain,
        Some(redirected.texture),
        view,
        surface_width,
        surface_height,
    );
}

fn drawn_pixel(surface_pixel: u32, surface_size: u32, drawn_size: u32) -> u32 {
    (surface_pixel as f32 * drawn_size as f32 / surface_size.max(1) as f32).round() as u32
}

fn dispatch_window_renders(
    renderer: &mut crate::renderer::wgpu::WgpuRenderer,
    inputs: &mut RenderInputs,
    outputs: &mut FrameOutputs,
    has_surface: bool,
) -> Result<(), crate::renderer::wgpu::rendergraph::RenderGraphError> {
    let surface_width = renderer.surface_config.width;
    let surface_height = renderer.surface_config.height;
    let has_viewports = !inputs.frame.cameras.is_empty();
    let _ = render_graph_set_pass_enabled(&mut renderer.graph, "viewport_blit_pass", has_viewports);
    let _ =
        render_graph_set_pass_enabled(&mut renderer.graph, "viewport_compose_pass", has_surface);

    if !has_viewports {
        let render_scale = inputs.settings.render_scale.clamp(0.25, 4.0);
        renderer.resize_render_buffers_for_window(
            ((surface_width as f32 * render_scale).round() as u32).max(1),
            ((surface_height as f32 * render_scale).round() as u32).max(1),
        );
    }

    inputs.view.camera_tile_render_iteration = 0;
    let original_active_camera = inputs.view.active_camera;
    let frame_settings_version = inputs.scene.settings_version;
    let focus_policy = inputs.scene.focus_policy;
    let global_dirty_signal = inputs.frame.global_dirty_signal;

    let swapchain_passes_before_loop = if has_viewports && has_surface {
        let saved_states = render_graph_save_pass_enabled_states(&renderer.graph);
        let saved: Vec<(String, bool)> =
            render_graph_pass_names_in_stage(&renderer.graph, PassStage::Swapchain)
                .into_iter()
                .map(|name| {
                    let enabled = saved_states.get(&name).copied().unwrap_or(true);
                    (name, enabled)
                })
                .collect();
        for (name, _) in &saved {
            let _ = render_graph_set_pass_enabled(&mut renderer.graph, name, false);
        }
        (!saved.is_empty()).then_some(saved)
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
        let (render_width, render_height) = match camera.target {
            Some(CameraTarget {
                size,
                host_owned: true,
            }) => size,
            Some(CameraTarget {
                size: (width, height),
                host_owned: false,
            }) => (
                ((width as f32 * render_scale).round() as u32).max(1),
                ((height as f32 * render_scale).round() as u32).max(1),
            ),
            None => (
                ((display_width as f32 * render_scale).round() as u32).max(1),
                ((display_height as f32 * render_scale).round() as u32).max(1),
            ),
        };

        if !matches!(
            camera.target,
            Some(CameraTarget {
                host_owned: true,
                ..
            })
        ) {
            renderer.ensure_camera_viewport(camera_entity, render_width, render_height);
        }
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
                crate::renderer::config::ViewportFocusPolicy::FocusedAlways
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
                let frustum_planes =
                    nightshade_render::data::bounding_volume::extract_frustum_planes(&view_proj);
                inputs
                    .frame
                    .dirty_world_spheres
                    .iter()
                    .any(|(center, radius)| {
                        nightshade_render::queries::projection::sphere_in_frustum(
                            center,
                            *radius,
                            &frustum_planes,
                        )
                    })
            } else {
                !inputs.frame.dirty_world_spheres.is_empty()
            };
            let pick_forces_render = is_main_viewport_camera && inputs.frame.pick_request.is_some();
            let camera_force_render = inputs.frame.force_render_cameras.contains(&camera_entity);
            let should_render = pick_forces_render
                || camera_force_render
                || match camera.update_mode {
                    crate::renderer::config::ViewportUpdateMode::Always => true,
                    crate::renderer::config::ViewportUpdateMode::WhenVisible => true,
                    crate::renderer::config::ViewportUpdateMode::WhenDirty => {
                        focus_forces_render
                            || !has_rendered_once
                            || global_dirty_signal
                            || settings_changed
                            || view_changed
                            || camera_self_dirty
                            || frustum_dirty
                    }
                    crate::renderer::config::ViewportUpdateMode::Once => !has_rendered_once,
                    crate::renderer::config::ViewportUpdateMode::Disabled => !has_rendered_once,
                };

            let camera_view_projection = camera
                .matrices
                .as_ref()
                .map(|matrices| (matrices.projection * matrices.view).into());
            if let Some(view_projection) = camera_view_projection {
                inputs.scene.prev_view_projection = renderer
                    .camera_viewports
                    .get(&camera_entity)
                    .and_then(|viewport| viewport.last_view_projection)
                    .unwrap_or(view_projection);
                inputs.scene.view_projection = view_projection;
            }

            inputs.scene.active_view = effective;
            inputs.scene.render_view = build_render_view(
                camera,
                inputs.scene.taa_jitter,
                (render_width, render_height),
            );
            let lighting = build_render_lighting(
                &inputs.scene_world,
                Some(camera),
                MAX_LIGHTS,
                MAX_AREA_LIGHTS,
                CASCADE_SLOT_RESOLUTION,
            );
            inputs.scene.render_lighting = Some(lighting);

            let phase = if should_render {
                rendergraph::ExecutePhase::Full
            } else {
                rendergraph::ExecutePhase::ComposeOnly
            };
            let command_buffers = {
                #[cfg(feature = "profiling")]
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
                #[cfg(feature = "profiling")]
                let _span = tracing::info_span!("queue_submit").entered();
                renderer.queue.submit(command_buffers);
            }

            if should_render {
                publish_camera_depth(renderer, camera_entity, render_width, render_height);
            }

            if should_render
                && let Some(viewport) = renderer.camera_viewports.get_mut(&camera_entity)
            {
                viewport.has_rendered_at_least_once = true;
                viewport.last_active_view = Some(effective);
                viewport.last_settings_version = frame_settings_version;
                viewport.last_render_frame = renderer.frame_state.index;
                viewport.last_camera_world_transform = camera.world_transform;
                if let Some(view_projection) = camera_view_projection {
                    viewport.last_view_projection = Some(view_projection);
                }
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

    if let Some(swapchain_passes) = swapchain_passes_before_loop {
        let redirected = redirect_paint_target(renderer);
        let (paint_width, paint_height) = redirected
            .as_ref()
            .map(|redirected| redirected.paint_size)
            .unwrap_or((surface_width, surface_height));
        let surface_viewport_size = inputs.view.viewport_size;
        if redirected.is_some() {
            inputs.view.viewport_size = Some((paint_width, paint_height));
        }

        let _ = rendergraph::render_graph_resize_transient_resource(
            &mut renderer.graph,
            &renderer.device,
            renderer.targets.swapchain_depth,
            paint_width.max(1),
            paint_height.max(1),
        );
        if redirected.is_some() {
            presentation::clear_swapchain(renderer, Some(nalgebra_glm::Vec4::zeros()));
        }

        let saved_pass_states = render_graph_save_pass_enabled_states(&renderer.graph);
        let enable: Vec<(&str, bool)> = swapchain_passes
            .iter()
            .map(|(name, enabled)| (name.as_str(), *enabled))
            .collect();
        render_graph_enable_only_passes(&mut renderer.graph, &enable);
        let command_buffers = {
            render_graph_execute(
                &mut renderer.graph,
                &renderer.device,
                &renderer.queue,
                inputs,
            )?
        };
        renderer.queue.submit(command_buffers);

        render_graph_restore_pass_enabled_states(&mut renderer.graph, &saved_pass_states);
        for (name, enabled) in &swapchain_passes {
            let _ = render_graph_set_pass_enabled(&mut renderer.graph, name, *enabled);
        }

        if let Some(redirected) = redirected {
            restore_surface_target(renderer, redirected, surface_width, surface_height);
            inputs.view.viewport_size = surface_viewport_size;
        }
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
                MAX_LIGHTS,
                MAX_AREA_LIGHTS,
                CASCADE_SLOT_RESOLUTION,
            );
            inputs.scene.render_lighting = Some(lighting);
            let _ = rendergraph::render_graph_resize_transient_resource(
                &mut renderer.graph,
                &renderer.device,
                renderer.targets.swapchain_depth,
                surface_width.max(1),
                surface_height.max(1),
            );
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
                let (render_width, render_height) = renderer.render_buffer_size;
                dispatch_pick_compute(
                    renderer,
                    drawn_pixel(screen_x, surface_width, render_width),
                    drawn_pixel(screen_y, surface_height, render_height),
                    inputs.view.active_camera,
                );
            }
        } else {
            presentation::clear_swapchain(renderer, inputs.frame.background_clear_color);

            let _ = rendergraph::render_graph_resize_transient_resource(
                &mut renderer.graph,
                &renderer.device,
                renderer.targets.swapchain_depth,
                surface_width.max(1),
                surface_height.max(1),
            );

            let swapchain_passes =
                render_graph_pass_names_in_stage(&renderer.graph, PassStage::Swapchain);
            let saved_pass_states = render_graph_save_pass_enabled_states(&renderer.graph);
            let enable: Vec<(&str, bool)> = swapchain_passes
                .iter()
                .map(|name| (name.as_str(), true))
                .collect();
            render_graph_enable_only_passes(&mut renderer.graph, &enable);

            let command_buffers = render_graph_execute(
                &mut renderer.graph,
                &renderer.device,
                &renderer.queue,
                inputs,
            )?;
            renderer.queue.submit(command_buffers);

            render_graph_restore_pass_enabled_states(&mut renderer.graph, &saved_pass_states);
            for name in &swapchain_passes {
                let _ = render_graph_set_pass_enabled(&mut renderer.graph, name, true);
            }
        }
    }

    let dispatched_settings_version = inputs.scene.settings_version;
    renderer.record_window_dispatch(dispatched_settings_version);
    Ok(())
}
