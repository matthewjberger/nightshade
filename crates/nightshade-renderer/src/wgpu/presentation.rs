//! Swapchain frame acquisition, the standard presentation pass chain, and the
//! screenshot readback path.

use crate::wgpu::WgpuRenderer;
use crate::wgpu::passes;
use crate::wgpu::render_configs::FrameSkipReason;
#[cfg(all(not(target_arch = "wasm32"), feature = "screenshot"))]
use crate::wgpu::rendergraph::render_graph_get_texture;
use crate::wgpu::rendergraph::{
    render_graph_add_pass, render_graph_compile, render_graph_set_external_texture,
};

/// Acquires the surface texture for this frame, reconfiguring the surface once
/// if it reports outdated or lost, and registers it (plus the spotlight shadow
/// atlas) as the graph's external textures. Returns [`FrameSkipReason`] when no
/// frame is available: [`FrameSkipReason::SurfaceLost`] when an outdated/lost
/// surface could not be reacquired, [`FrameSkipReason::SurfaceUnavailable`] for
/// a transient state such as an occluded or timed-out surface. Present the
/// returned texture after the graph executes.
pub fn acquire_surface_frame(
    renderer: &mut WgpuRenderer,
) -> std::result::Result<wgpu::SurfaceTexture, FrameSkipReason> {
    let surface_texture = match renderer.surface.get_current_texture() {
        wgpu::CurrentSurfaceTexture::Success(frame)
        | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
        wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
            renderer
                .surface
                .configure(&renderer.device, &renderer.surface_config);
            match renderer.surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(frame)
                | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
                _ => {
                    return Err(FrameSkipReason::SurfaceLost);
                }
            }
        }
        _ => {
            return Err(FrameSkipReason::SurfaceUnavailable);
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

    Ok(surface_texture)
}

/// Appends the presentation tail to the graph and compiles it: the viewport
/// blit and cached-viewport preload, the swapchain compose, the retained UI
/// and UI image passes, the egui overlay, and the pick keepalive. Call after
/// the scene passes have been added.
pub fn install_presentation_passes(
    renderer: &mut WgpuRenderer,
) -> Result<(), Box<dyn std::error::Error>> {
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

/// Polls a pending screenshot readback and, once the staging buffer is
/// mapped, unpads the rows, converts BGRA surfaces to RGBA, and saves the
/// image on a background thread.
#[cfg(all(not(target_arch = "wasm32"), feature = "screenshot"))]
pub fn poll_screenshot_readback(renderer: &mut WgpuRenderer) {
    if !renderer.screenshot.pending {
        return;
    }
    let _ = renderer.device.poll(wgpu::PollType::Poll);

    if !renderer
        .screenshot
        .map_complete
        .load(std::sync::atomic::Ordering::Relaxed)
    {
        return;
    }
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
        let Some(mut image) = image::RgbaImage::from_raw(save_width, save_height, pixels) else {
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

/// Copies the compute output into the screenshot staging buffer and starts the
/// async map; [`poll_screenshot_readback`] finishes the capture on a later
/// frame. Skipped while a previous capture is still in flight.
#[cfg(all(not(target_arch = "wasm32"), feature = "screenshot"))]
pub fn request_screenshot_copy(
    renderer: &mut WgpuRenderer,
    path: Option<std::path::PathBuf>,
    max_dimension: Option<u32>,
) {
    if renderer.screenshot.pending {
        return;
    }
    let Some(compute_output_texture) =
        render_graph_get_texture(&renderer.graph, renderer.targets.compute_output)
    else {
        return;
    };
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

    let mut encoder = renderer
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
    renderer.screenshot.max_dimension = max_dimension;
    renderer.screenshot.pending = true;

    let map_complete = renderer.screenshot.map_complete.clone();
    renderer
        .screenshot
        .staging_buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |_| {
            map_complete.store(true, std::sync::atomic::Ordering::Relaxed);
        });
}

#[cfg(all(not(target_arch = "wasm32"), feature = "screenshot"))]
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

/// Clears the swapchain to the given background color (or the default dark
/// gray) so retained-UI-only frames start from a defined surface without
/// running the scene graph.
pub fn clear_swapchain(renderer: &mut WgpuRenderer, background: Option<nalgebra_glm::Vec4>) {
    let Some(view) = crate::wgpu::rendergraph::render_graph_get_texture_view(
        &renderer.graph,
        renderer.targets.swapchain,
    ) else {
        return;
    };
    let clear = background
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
