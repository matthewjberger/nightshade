use crate::renderer::config::render_settings_signature;
use crate::renderer::wgpu::WgpuRenderer;
use crate::renderer::wgpu::render_configs::RenderInputs;

pub fn generate_world_text_mesh(
    renderer: &mut WgpuRenderer,
    font_engine: &mut nightshade_text::contract::data::font_engine::FontEngine,
    text: &str,
    params: &crate::renderer::text_mesh::TextMeshParams,
) -> nightshade_text::contract::data::text_data::TextMesh {
    crate::renderer::text_mesh::generate_text_mesh_world(
        text,
        &mut crate::renderer::text_mesh::TextMeshContext {
            font_engine,
            resolve_glyph: &mut |font_engine, cache_key| {
                renderer.glyph_atlas.ensure_glyph(
                    &renderer.device,
                    &renderer.queue,
                    &mut font_engine.font_system,
                    &mut font_engine.swash_cache,
                    cache_key,
                )
            },
        },
        params,
    )
}

pub fn generate_screen_text_mesh(
    renderer: &mut WgpuRenderer,
    font_engine: &mut nightshade_text::contract::data::font_engine::FontEngine,
    text: &str,
    params: &crate::renderer::text_mesh::TextMeshParams,
    wrap_width: Option<f32>,
) -> nightshade_text::contract::data::text_data::TextMesh {
    crate::renderer::text_mesh::generate_text_mesh_screen(
        text,
        &mut crate::renderer::text_mesh::TextMeshContext {
            font_engine,
            resolve_glyph: &mut |font_engine, cache_key| {
                renderer.glyph_atlas.ensure_glyph(
                    &renderer.device,
                    &renderer.queue,
                    &mut font_engine.font_system,
                    &mut font_engine.swash_cache,
                    cache_key,
                )
            },
        },
        params,
        wrap_width,
    )
}

pub fn glyph_atlas_revision(renderer: &WgpuRenderer) -> u32 {
    renderer.glyph_atlas.revision()
}

pub fn publish_glyph_atlas_if_grown(renderer: &mut WgpuRenderer, pre_revision: u32) {
    if renderer.glyph_atlas.revision() != pre_revision {
        publish_glyph_atlas(renderer);
    }
}

pub fn publish_glyph_atlas(renderer: &mut WgpuRenderer) {
    let texture = renderer.glyph_atlas.texture.clone();
    let view = renderer.glyph_atlas.texture_view.clone();
    let width = texture.width();
    let height = texture.height();
    let identifier = renderer.targets.glyph_atlas;
    crate::renderer::wgpu::rendergraph::render_graph_set_external_texture(
        &mut renderer.graph,
        identifier,
        Some(texture),
        view,
        width,
        height,
    );
}

pub fn set_color_lut(renderer: &mut WgpuRenderer, data: Vec<u8>) {
    if let Some(pass) =
        crate::renderer::wgpu::pass_access::postprocess_pass_mut(&mut renderer.graph)
    {
        pass.set_color_lut(data);
    }
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

pub fn sync_taa_jitter(renderer: &WgpuRenderer, inputs: &mut RenderInputs) {
    if inputs.settings.anti_aliasing.is_temporal() {
        let jitter_index = (renderer.frame_state.index % 8) as u32 + 1;
        let (render_width, render_height) = renderer.render_buffer_size;
        let render_scale = inputs.settings.render_scale.clamp(0.25, 4.0);
        inputs.scene.taa_jitter = [
            (taa_halton(jitter_index, 2) - 0.5) * 2.0 * render_scale / render_width.max(1) as f32,
            (taa_halton(jitter_index, 3) - 0.5) * 2.0 * render_scale / render_height.max(1) as f32,
        ];
    } else {
        inputs.scene.taa_jitter = [0.0, 0.0];
    }
}

pub fn sync_settings_version(renderer: &mut WgpuRenderer, inputs: &mut RenderInputs) {
    let outline_ids: Vec<u32> = inputs
        .scene_world
        .query_tag_type::<nightshade_render::components::render_world::Outlined>()
        .map(|entity| entity.id)
        .collect();
    let current_settings_signature = render_settings_signature(
        &inputs.settings,
        &inputs.debug_draw,
        &outline_ids,
        inputs.scene.outline_color,
    );
    if renderer.frame_state.last_settings_signature != Some(current_settings_signature) {
        inputs.scene.settings_version = inputs.scene.settings_version.wrapping_add(1);
        renderer.frame_state.last_settings_signature = Some(current_settings_signature);
    }
}

pub fn sync_gpu_profile(renderer: &WgpuRenderer, inputs: &mut RenderInputs) {
    if inputs.scene.gpu_profile.is_none() {
        inputs.scene.gpu_profile = Some(renderer.gpu_profile.clone());
        if renderer.gpu_profile.backend == crate::renderer::config::GpuBackend::Metal {
            inputs.scene.occlusion_culling_enabled = false;
        }
    }
}
