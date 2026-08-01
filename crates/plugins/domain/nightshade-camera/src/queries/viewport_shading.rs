use nightshade_render::components::culling::CameraCullingMask;
use nightshade_render::data::config::{DebugDraw, RenderSettings};

use crate::components::viewport_shading::{FogOverride, ShadingMode, ViewportShading};
use crate::components::{CameraEnvironment, CameraPostProcess};
use nightshade_render::data::config::{
    EffectiveShading, FLAT_SHADING_COLOR, WIREFRAME_SENTINEL_COLOR,
};

pub fn effective_shading_from_graphics(
    render: &RenderSettings,
    debug: &DebugDraw,
) -> EffectiveShading {
    EffectiveShading {
        unlit_mode: render.unlit_mode,
        bloom_enabled: render.bloom_enabled,
        ssao_enabled: render.ssao_enabled,
        ssgi_enabled: render.ssgi_enabled,
        ssr_enabled: render.ssr_enabled,
        anti_aliasing: render.anti_aliasing,
        taa_blend: render.taa_blend,
        taa_sharpness: render.taa_sharpness,
        show_normals: debug.show_normals,
        show_bounding_volumes: debug.show_bounding_volumes,
        show_wireframe: false,
        flat_shading_color: None,
        lines_enabled: debug.show_normals || debug.show_bounding_volumes,
        color_grading: render.color_grading,
        depth_of_field: render.depth_of_field,
        bloom_intensity: render.bloom_intensity,
        bloom_threshold: render.bloom_threshold,
        bloom_knee: render.bloom_knee,
        bloom_filter_radius: render.bloom_filter_radius,
        ssao_radius: render.ssao_radius,
        ssao_bias: render.ssao_bias,
        ssao_intensity: render.ssao_intensity,
        ssao_sample_count: render.ssao_sample_count,
        ssao_visualization: debug.ssao_visualization,
        pbr_debug_mode: debug.pbr_debug_mode,
        material_filtering: debug.material_filtering,
        vertex_snap: render.vertex_snap,
        affine_texture_mapping: render.affine_texture_mapping,
        material_anisotropy_filtering: render.material_anisotropy_filtering,
        ssao_blur_depth_threshold: render.ssao_blur_depth_threshold,
        ssao_blur_normal_power: render.ssao_blur_normal_power,
        ssgi_radius: render.ssgi_radius,
        ssgi_intensity: render.ssgi_intensity,
        ssgi_max_steps: render.ssgi_max_steps,
        ssr_max_steps: render.ssr_max_steps,
        ssr_thickness: render.ssr_thickness,
        ssr_max_distance: render.ssr_max_distance,
        ssr_stride: render.ssr_stride,
        ssr_fade_start: render.ssr_fade_start,
        ssr_fade_end: render.ssr_fade_end,
        ssr_intensity: render.ssr_intensity,
        culling_mask: !0,
        fog: render.fog,
        probe_volume: render.probe_volume,
        atmosphere: render.atmosphere,
        render_scale: render.render_scale,
        hdr_output: render.hdr_output,
        hdr_paper_white: render.hdr_paper_white,
        hdr_peak_nits: render.hdr_peak_nits,
        ssr_traced: render.ssr_traced,
        ssao_traced: render.ssao_traced,
        sun_shadow_traced: render.sun_shadow_traced,
        viewport_rect: None,
    }
}

pub fn effective_shading_for_camera(
    render: &RenderSettings,
    debug: &DebugDraw,
    shading: &ViewportShading,
    post_process: Option<&CameraPostProcess>,
    culling_mask: Option<&CameraCullingMask>,
    environment: Option<&CameraEnvironment>,
) -> EffectiveShading {
    let mut effective = effective_shading_from_graphics(render, debug);
    if let Some(mask) = culling_mask {
        effective.culling_mask = mask.0;
    }
    if let Some(env) = environment {
        if let Some(atmosphere) = env.atmosphere {
            effective.atmosphere = atmosphere;
        }
        effective.fog = match env.fog {
            FogOverride::Inherit => effective.fog,
            FogOverride::Disabled => None,
            FogOverride::Override(fog) => Some(fog),
        };
    }
    if let Some(overrides) = post_process {
        if let Some(cg) = overrides.color_grading {
            effective.color_grading = cg;
        }
        if let Some(dof) = overrides.depth_of_field {
            effective.depth_of_field = dof;
        }
        if let Some(value) = overrides.bloom_intensity {
            effective.bloom_intensity = value;
        }
        if let Some(value) = overrides.bloom_threshold {
            effective.bloom_threshold = value;
        }
        if let Some(value) = overrides.bloom_knee {
            effective.bloom_knee = value;
        }
        if let Some(value) = overrides.bloom_filter_radius {
            effective.bloom_filter_radius = value;
        }
        if let Some(value) = overrides.ssao_intensity {
            effective.ssao_intensity = value;
        }
        if let Some(value) = overrides.ssao_radius {
            effective.ssao_radius = value;
        }
        if let Some(value) = overrides.ssgi_intensity {
            effective.ssgi_intensity = value;
        }
        if let Some(value) = overrides.ssr_intensity {
            effective.ssr_intensity = value;
        }
    }
    match shading.mode {
        ShadingMode::Rendered => {}
        ShadingMode::Solid => {
            effective.unlit_mode = true;
            effective.bloom_enabled = false;
            effective.ssao_enabled = false;
            effective.ssgi_enabled = false;
            effective.ssr_enabled = false;
        }
        ShadingMode::Flat => {
            effective.bloom_enabled = false;
            effective.ssao_enabled = false;
            effective.ssgi_enabled = false;
            effective.ssr_enabled = false;
            effective.flat_shading_color = Some(FLAT_SHADING_COLOR);
        }
        ShadingMode::Wireframe => {
            effective.unlit_mode = true;
            effective.bloom_enabled = false;
            effective.ssao_enabled = false;
            effective.ssgi_enabled = false;
            effective.ssr_enabled = false;
            effective.flat_shading_color = Some(WIREFRAME_SENTINEL_COLOR);
            effective.show_wireframe = true;
        }
    }
    if !shading.show_overlays {
        effective.show_normals = false;
        effective.show_bounding_volumes = false;
    }
    effective.lines_enabled =
        effective.show_normals || effective.show_bounding_volumes || effective.show_wireframe;
    effective
}
