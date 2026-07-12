//! Per-frame synchronization of pass-internal state that the frame driver
//! requests by name: dirty-state handoff, glyph atlas rebinds, cloth write
//! targets, conditional pass toggles, and the TAA jitter sequence. These
//! operations take plain data; gathering that data from whatever scene
//! representation feeds the renderer is the caller's job.

use crate::config::render_settings_signature;
use crate::mesh_cache::{MeshCache, mesh_cache_take_dirty, mesh_cache_take_dirty_skinned};
use crate::wgpu::WgpuRenderer;
use crate::wgpu::passes;
use crate::wgpu::passes::geometry::MeshRenderStateInner;
use crate::wgpu::passes::geometry::lines::LinesPass;
use crate::wgpu::render_configs::RenderInputs;
use crate::wgpu::rendergraph::{RenderGraph, render_graph_set_pass_enabled};

/// Hands the frame's dirty state to the passes that consume it: the shadow
/// pass gets a coarse scene-dirty flag, the mesh pass gets the full dirty set
/// plus resolved dirty mesh ids, and the skinned pass accumulates the dirty
/// skinned mesh names. Takes the mesh cache's dirty sets in the same step.
pub fn apply_frame_dirty_state(
    renderer: &mut WgpuRenderer,
    frame_state: MeshRenderStateInner,
    mesh_cache: &mut MeshCache,
) {
    let dirty_meshes = mesh_cache_take_dirty(mesh_cache);
    let dirty_skinned = mesh_cache_take_dirty_skinned(mesh_cache);

    let shadow_scene_dirty = !frame_state.transform_dirty.is_empty()
        || !frame_state.entities_added.is_empty()
        || !frame_state.entities_removed.is_empty()
        || frame_state.instanced_meshes_changed
        || !dirty_meshes.is_empty()
        || !dirty_skinned.is_empty();

    if let Some(pass) = crate::wgpu::pass_access::shadow_depth_pass_mut(&mut renderer.graph) {
        pass.shadow_scene_dirty = shadow_scene_dirty;
    }

    if let Some(pass) = crate::wgpu::pass_access::mesh_pass_mut(&mut renderer.graph) {
        pass.frame_dirty = Some(frame_state);
        let dirty_ids: std::collections::HashSet<u32> = dirty_meshes
            .iter()
            .filter_map(|name| pass.mesh_id_for_name(name))
            .collect();
        pass.dirty_mesh_ids = dirty_ids;
    }

    if let Some(pass) = crate::wgpu::pass_access::skinned_mesh_pass_mut(&mut renderer.graph) {
        pass.dirty_skinned_mesh_names.extend(dirty_skinned);
    }
}

/// Rebinds the glyph atlas texture on every pass that samples it. Call after
/// the atlas revision changes (a glyph rasterization grew or repacked it).
pub fn rebind_glyph_atlas(renderer: &mut WgpuRenderer) {
    if let Some(overlay_pass) =
        crate::wgpu::pass_access::scene_overlay_pass_mut(&mut renderer.graph)
    {
        overlay_pass
            .text
            .update_glyph_atlas(&renderer.device, &renderer.glyph_atlas.texture_view);
    }
    if let Some(ui_pass) = crate::wgpu::pass_access::ui_pass_mut(&mut renderer.graph) {
        ui_pass.update_glyph_atlas(&renderer.device, &renderer.glyph_atlas.texture_view);
    }
}

pub fn lines_pass_mut(graph: &mut RenderGraph<RenderInputs>) -> Option<&mut LinesPass> {
    let overlay_pass = crate::wgpu::pass_access::scene_overlay_pass_mut(graph)?;
    Some(&mut overlay_pass.lines)
}

pub fn set_color_lut(renderer: &mut WgpuRenderer, data: Vec<u8>) {
    if let Some(pass) = crate::wgpu::pass_access::postprocess_pass_mut(&mut renderer.graph) {
        pass.set_color_lut(data);
    }
}

/// Rebinds the shadow pass's point-light cubemap on the mesh and skinned mesh
/// passes so lighting samples the cubemap the current shadow pass owns.
pub fn sync_point_shadow_cubemap(renderer: &mut WgpuRenderer) {
    let point_shadow_view = crate::wgpu::pass_access::shadow_depth_pass_mut(&mut renderer.graph)
        .map(|shadow_pass| shadow_pass.point_light_cubemap_view().clone());

    if let Some(view) = point_shadow_view {
        if let Some(mesh_pass) = crate::wgpu::pass_access::mesh_pass_mut(&mut renderer.graph) {
            mesh_pass.update_point_shadow_cubemap(view.clone());
        }
        if let Some(skinned_mesh_pass) =
            crate::wgpu::pass_access::skinned_mesh_pass_mut(&mut renderer.graph)
        {
            skinned_mesh_pass.update_point_shadow_cubemap(view);
        }
    }
}

/// One cloth's mesh name and the widened cull bounds covering its full
/// deformation range.
pub struct ClothWriteBounds {
    pub mesh_name: String,
    pub center: [f32; 3],
    pub sphere_radius: f32,
}

/// Hands the cloth pass its vertex buffer write targets for this frame.
///
/// Cloth entities render through the regular mesh and shadow passes;
/// the cloth simulation writes deformed vertices into those passes'
/// vertex buffers at each cloth's registered mesh range. The targets
/// are refreshed every frame so buffer growth, compaction, and mesh
/// re-registration are picked up, and the mesh cull bounds are widened
/// to the cloth's full deformation range.
pub fn set_cloth_write_targets(renderer: &mut WgpuRenderer, cloths: &[ClothWriteBounds]) {
    if cloths.is_empty() {
        return;
    }

    let mut targets: std::collections::HashMap<String, Vec<passes::geometry::ClothWriteTarget>> =
        std::collections::HashMap::new();

    if let Some(mesh_pass) = crate::wgpu::pass_access::mesh_pass_mut(&mut renderer.graph) {
        for cloth in cloths {
            mesh_pass.override_mesh_bounds(
                &renderer.queue,
                &cloth.mesh_name,
                cloth.center,
                cloth.sphere_radius,
            );
            if let Some((buffer, buffer_generation, vertex_offset)) =
                mesh_pass.vertex_write_target(&cloth.mesh_name)
            {
                targets.entry(cloth.mesh_name.clone()).or_default().push(
                    passes::geometry::ClothWriteTarget {
                        buffer,
                        buffer_generation,
                        vertex_offset,
                    },
                );
            }
        }
    }

    if let Some(shadow_pass) = crate::wgpu::pass_access::shadow_depth_pass_mut(&mut renderer.graph)
    {
        for cloth in cloths {
            if let Some((buffer, buffer_generation, vertex_offset)) =
                shadow_pass.vertex_write_target(&cloth.mesh_name)
            {
                targets.entry(cloth.mesh_name.clone()).or_default().push(
                    passes::geometry::ClothWriteTarget {
                        buffer,
                        buffer_generation,
                        vertex_offset,
                    },
                );
            }
        }
    }

    if let Some(cloth_pass) = crate::wgpu::pass_access::cloth_pass_mut(&mut renderer.graph) {
        cloth_pass.set_write_targets(targets);
    }
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
pub fn sync_conditional_pass_toggles(
    renderer: &mut WgpuRenderer,
    inputs: &RenderInputs,
    picking_active: bool,
    has_skinned_meshes: bool,
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
    let _ =
        render_graph_set_pass_enabled(&mut renderer.graph, "pick_keepalive_pass", picking_active);
    let _ =
        render_graph_set_pass_enabled(&mut renderer.graph, "skinned_mesh_pass", has_skinned_meshes);

    // Run the temporal resolve only when enabled; otherwise a cheap blit
    // copies the post-process output through so the compose still has an
    // image, without executing the full resolve pass.
    let taa_on = inputs.settings.taa_enabled;
    let _ = render_graph_set_pass_enabled(&mut renderer.graph, "taa_pass", taa_on);
    let _ = render_graph_set_pass_enabled(&mut renderer.graph, "taa_passthrough", !taa_on);
}

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

/// Advances the Halton jitter sequence for this frame, or zeroes the jitter
/// when TAA is off.
pub fn sync_taa_jitter(renderer: &WgpuRenderer, inputs: &mut RenderInputs) {
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
}

/// Bumps the scene's settings version when the settings signature changed
/// since the last frame, so viewport caching invalidates on settings edits.
pub fn sync_settings_version(renderer: &mut WgpuRenderer, inputs: &mut RenderInputs) {
    let current_settings_signature = render_settings_signature(
        &inputs.settings,
        &inputs.debug_draw,
        &inputs.editor_selection,
    );
    if renderer.frame_state.last_settings_signature != Some(current_settings_signature) {
        inputs.scene.settings_version = inputs.scene.settings_version.wrapping_add(1);
        renderer.frame_state.last_settings_signature = Some(current_settings_signature);
    }
}

/// Stamps the adapter's GPU profile into the scene once, disabling occlusion
/// culling on Metal.
pub fn sync_gpu_profile(renderer: &WgpuRenderer, inputs: &mut RenderInputs) {
    if inputs.scene.gpu_profile.is_none() {
        inputs.scene.gpu_profile = Some(renderer.gpu_profile.clone());
        if renderer.gpu_profile.backend == crate::config::GpuBackend::Metal {
            inputs.scene.occlusion_culling_enabled = false;
        }
    }
}

/// Drains one egui frame's output into the egui pass: uploads the texture
/// deltas and sets the paint jobs and screen descriptor for this frame.
#[cfg(feature = "egui")]
pub fn prepare_egui_pass(
    renderer: &mut WgpuRenderer,
    textures_delta: &egui::TexturesDelta,
    pixels_per_point: f32,
    paint_jobs: Vec<egui::ClippedPrimitive>,
) {
    let screen_descriptor = egui_wgpu::ScreenDescriptor {
        size_in_pixels: [
            renderer.surface_config.width,
            renderer.surface_config.height,
        ],
        pixels_per_point,
    };

    if let Some(egui_pass) = crate::wgpu::pass_access::egui_pass_mut(&mut renderer.graph) {
        egui_pass.update_textures(&renderer.device, &renderer.queue, textures_delta);
        egui_pass.set_screen_descriptor(screen_descriptor);
        egui_pass.set_paint_jobs(paint_jobs);
    }
}
