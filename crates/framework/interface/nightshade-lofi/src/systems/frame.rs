use crate::resources::LofiSettings;
use nightshade::prelude::*;
use nightshade_render::data::config::{AntiAliasing, RenderSettings, VertexSnap};

pub fn apply_the_look(mut settings: ResMut<LofiSettings>, mut render: ResMut<RenderSettings>) {
    if !settings.waiting {
        return;
    }
    settings.waiting = false;
    let look = settings.look;
    render.render_scale = look.render_scale;
    render.vertex_snap = look.snap.map(|resolution| VertexSnap { resolution });
    render.affine_texture_mapping = look.affine_textures;
    render.material_anisotropy_filtering = look.anisotropy;
    render.anti_aliasing = if look.smooth_edges {
        AntiAliasing::Fxaa
    } else {
        AntiAliasing::Off
    };
    render.bloom_enabled = look.bloom;
    render.ssao_enabled = look.ambient_occlusion;
    render.color_grading.saturation = look.saturation;
    render.color_grading.contrast = look.contrast;
    render.color_grading.vignette_intensity = look.vignette;
    render.color_grading.vignette_radius = look.vignette_radius;
    render.color_grading.tonemap_algorithm = look.tonemap;
}
