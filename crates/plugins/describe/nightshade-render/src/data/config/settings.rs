use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ViewportFocusPolicy {
    #[default]
    FocusedAlways,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum AntiAliasing {
    Off,
    #[default]
    Fxaa,
    Taa,
}

impl AntiAliasing {
    pub const ALL: &'static [AntiAliasing] =
        &[AntiAliasing::Off, AntiAliasing::Fxaa, AntiAliasing::Taa];

    pub fn name(&self) -> &'static str {
        match self {
            AntiAliasing::Off => "Off",
            AntiAliasing::Fxaa => "FXAA",
            AntiAliasing::Taa => "TAA",
        }
    }

    pub fn is_temporal(&self) -> bool {
        matches!(self, AntiAliasing::Taa)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PbrDebugMode {
    #[default]
    None,
    BaseColor,
    Normal,
    Metallic,
    Roughness,
    Occlusion,
    Emissive,
    F,
    G,
    D,
    Diffuse,
    Specular,
    MipRainbow,
    Shadow,
}

impl PbrDebugMode {
    pub const ALL: &'static [PbrDebugMode] = &[
        PbrDebugMode::None,
        PbrDebugMode::BaseColor,
        PbrDebugMode::Normal,
        PbrDebugMode::Metallic,
        PbrDebugMode::Roughness,
        PbrDebugMode::Occlusion,
        PbrDebugMode::Emissive,
        PbrDebugMode::F,
        PbrDebugMode::G,
        PbrDebugMode::D,
        PbrDebugMode::Diffuse,
        PbrDebugMode::Specular,
        PbrDebugMode::MipRainbow,
        PbrDebugMode::Shadow,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            PbrDebugMode::None => "None",
            PbrDebugMode::BaseColor => "Base Color",
            PbrDebugMode::Normal => "Normal",
            PbrDebugMode::Metallic => "Metallic",
            PbrDebugMode::Roughness => "Roughness",
            PbrDebugMode::Occlusion => "Occlusion",
            PbrDebugMode::Emissive => "Emissive",
            PbrDebugMode::F => "Fresnel (F)",
            PbrDebugMode::G => "Geometry (G)",
            PbrDebugMode::D => "Distribution (D)",
            PbrDebugMode::Diffuse => "Diffuse",
            PbrDebugMode::Specular => "Specular",
            PbrDebugMode::MipRainbow => "Mip Rainbow",
            PbrDebugMode::Shadow => "Shadow",
        }
    }

    pub fn as_u32(&self) -> u32 {
        match self {
            PbrDebugMode::None => 0,
            PbrDebugMode::BaseColor => 1,
            PbrDebugMode::Normal => 2,
            PbrDebugMode::Metallic => 3,
            PbrDebugMode::Roughness => 4,
            PbrDebugMode::Occlusion => 5,
            PbrDebugMode::Emissive => 6,
            PbrDebugMode::F => 7,
            PbrDebugMode::G => 8,
            PbrDebugMode::D => 9,
            PbrDebugMode::Diffuse => 10,
            PbrDebugMode::Specular => 11,
            PbrDebugMode::MipRainbow => 12,
            PbrDebugMode::Shadow => 13,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VertexSnap {
    pub resolution: [f32; 2],
}

impl Default for VertexSnap {
    fn default() -> Self {
        Self {
            resolution: [160.0, 120.0],
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RenderSettings {
    pub atmosphere: Atmosphere,
    pub show_sky: bool,
    pub render_layer_world_enabled: bool,
    pub render_layer_overlay_enabled: bool,
    pub render_world_to_swapchain: bool,
    pub clear_color: [f32; 4],
    pub swapchain_clear_color: Option<[f32; 4]>,
    pub unlit_mode: bool,
    pub vertex_snap: Option<VertexSnap>,
    pub affine_texture_mapping: bool,
    pub material_anisotropy_filtering: u16,
    pub fog: Option<Fog>,
    pub color_grading: ColorGrading,
    pub bloom_enabled: bool,
    pub bloom_intensity: f32,
    pub bloom_threshold: f32,
    pub bloom_knee: f32,
    pub bloom_filter_radius: f32,
    pub depth_of_field: DepthOfField,
    pub ssao_enabled: bool,
    pub ssao_radius: f32,
    pub ssao_bias: f32,
    pub ssao_intensity: f32,
    pub ssao_sample_count: u32,
    pub ssao_blur_depth_threshold: f32,
    pub ssao_blur_normal_power: f32,
    pub ssgi_enabled: bool,
    pub ssgi_radius: f32,
    pub ssgi_intensity: f32,
    pub ssgi_max_steps: u32,
    pub ssr_enabled: bool,
    pub ssr_max_steps: u32,
    pub ssr_thickness: f32,
    pub ssr_max_distance: f32,
    pub ssr_stride: f32,
    pub ssr_fade_start: f32,
    pub ssr_fade_end: f32,
    pub ssr_intensity: f32,
    pub anti_aliasing: AntiAliasing,
    pub taa_blend: f32,
    pub taa_sharpness: f32,
    pub render_scale: f32,
    pub hdr_output: bool,
    pub hdr_paper_white: f32,
    pub hdr_peak_nits: f32,
    pub ssr_traced: bool,
    pub ssao_traced: bool,
    pub sun_shadow_traced: bool,
    pub named_shadow_casters: bool,
    pub probe_volume: Option<crate::data::config::ProbeVolume>,
    pub ibl_blend_factor: f32,
}

#[derive(Clone)]
pub struct DebugDraw {
    pub show_grid: bool,
    pub show_bounding_volumes: bool,
    pub show_selected_bounding_volume: bool,
    pub show_normals: bool,
    pub normal_line_length: f32,
    pub normal_line_color: [f32; 4],
    pub pbr_debug_mode: PbrDebugMode,
    pub ssao_visualization: bool,
    pub material_filtering: bool,
}

pub fn render_settings_signature(
    render: &RenderSettings,
    debug: &DebugDraw,
    outline_ids: &[u32],
    outline_color: [f32; 4],
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    outline_ids.hash(&mut hasher);
    for component in outline_color {
        component.to_bits().hash(&mut hasher);
    }
    debug.show_grid.hash(&mut hasher);
    debug.show_bounding_volumes.hash(&mut hasher);
    debug.show_selected_bounding_volume.hash(&mut hasher);
    debug.show_normals.hash(&mut hasher);
    debug.normal_line_length.to_bits().hash(&mut hasher);
    for component in debug.normal_line_color {
        component.to_bits().hash(&mut hasher);
    }
    (render.atmosphere as u32).hash(&mut hasher);
    render.show_sky.hash(&mut hasher);
    render.render_layer_world_enabled.hash(&mut hasher);
    render.render_layer_overlay_enabled.hash(&mut hasher);
    render.render_world_to_swapchain.hash(&mut hasher);
    for component in render.clear_color {
        component.to_bits().hash(&mut hasher);
    }
    render.unlit_mode.hash(&mut hasher);
    render.vertex_snap.is_some().hash(&mut hasher);
    if let Some(ref snap) = render.vertex_snap {
        for component in snap.resolution {
            component.to_bits().hash(&mut hasher);
        }
    }
    render.affine_texture_mapping.hash(&mut hasher);
    render.material_anisotropy_filtering.hash(&mut hasher);
    render.fog.is_some().hash(&mut hasher);
    if let Some(ref fog) = render.fog {
        for component in fog.color {
            component.to_bits().hash(&mut hasher);
        }
        fog.start.to_bits().hash(&mut hasher);
        fog.end.to_bits().hash(&mut hasher);
        fog.volumetric.hash(&mut hasher);
        fog.density.to_bits().hash(&mut hasher);
        fog.anisotropy.to_bits().hash(&mut hasher);
    }
    let cg = &render.color_grading;
    cg.exposure.to_bits().hash(&mut hasher);
    cg.exposure_compensation_ev.to_bits().hash(&mut hasher);
    cg.auto_exposure.hash(&mut hasher);
    cg.auto_exposure_target.to_bits().hash(&mut hasher);
    cg.auto_exposure_rate.to_bits().hash(&mut hasher);
    cg.auto_exposure_min_ev.to_bits().hash(&mut hasher);
    cg.auto_exposure_max_ev.to_bits().hash(&mut hasher);
    cg.gamma.to_bits().hash(&mut hasher);
    cg.saturation.to_bits().hash(&mut hasher);
    cg.brightness.to_bits().hash(&mut hasher);
    cg.contrast.to_bits().hash(&mut hasher);
    (cg.tonemap_algorithm as u32).hash(&mut hasher);
    render.bloom_enabled.hash(&mut hasher);
    render.bloom_intensity.to_bits().hash(&mut hasher);
    render.bloom_threshold.to_bits().hash(&mut hasher);
    render.bloom_knee.to_bits().hash(&mut hasher);
    render.bloom_filter_radius.to_bits().hash(&mut hasher);
    let dof = &render.depth_of_field;
    dof.enabled.hash(&mut hasher);
    dof.focus_distance.to_bits().hash(&mut hasher);
    dof.focus_range.to_bits().hash(&mut hasher);
    dof.max_blur_radius.to_bits().hash(&mut hasher);
    dof.bokeh_threshold.to_bits().hash(&mut hasher);
    dof.bokeh_intensity.to_bits().hash(&mut hasher);
    (dof.quality as u32).hash(&mut hasher);
    dof.visualize_coc.hash(&mut hasher);
    dof.tilt_shift_enabled.hash(&mut hasher);
    dof.tilt_shift_angle.to_bits().hash(&mut hasher);
    dof.tilt_shift_center.to_bits().hash(&mut hasher);
    dof.tilt_shift_blur_amount.to_bits().hash(&mut hasher);
    dof.visualize_tilt_shift.hash(&mut hasher);
    render.ssao_enabled.hash(&mut hasher);
    render.ssao_radius.to_bits().hash(&mut hasher);
    render.ssao_bias.to_bits().hash(&mut hasher);
    render.ssao_intensity.to_bits().hash(&mut hasher);
    render.ssao_sample_count.hash(&mut hasher);
    debug.ssao_visualization.hash(&mut hasher);
    render.ssao_blur_depth_threshold.to_bits().hash(&mut hasher);
    render.ssao_blur_normal_power.to_bits().hash(&mut hasher);
    render.ssgi_enabled.hash(&mut hasher);
    render.ssgi_radius.to_bits().hash(&mut hasher);
    render.ssgi_intensity.to_bits().hash(&mut hasher);
    render.ssgi_max_steps.hash(&mut hasher);
    render.ssr_enabled.hash(&mut hasher);
    render.ssr_max_steps.hash(&mut hasher);
    render.ssr_thickness.to_bits().hash(&mut hasher);
    render.ssr_max_distance.to_bits().hash(&mut hasher);
    render.ssr_stride.to_bits().hash(&mut hasher);
    render.ssr_fade_start.to_bits().hash(&mut hasher);
    render.ssr_fade_end.to_bits().hash(&mut hasher);
    render.ssr_intensity.to_bits().hash(&mut hasher);
    debug.pbr_debug_mode.as_u32().hash(&mut hasher);
    debug.material_filtering.hash(&mut hasher);
    render.anti_aliasing.hash(&mut hasher);
    render.taa_blend.to_bits().hash(&mut hasher);
    render.taa_sharpness.to_bits().hash(&mut hasher);
    render.render_scale.to_bits().hash(&mut hasher);
    render.hdr_output.hash(&mut hasher);
    render.hdr_paper_white.to_bits().hash(&mut hasher);
    render.hdr_peak_nits.to_bits().hash(&mut hasher);
    render.ssr_traced.hash(&mut hasher);
    render.sun_shadow_traced.hash(&mut hasher);
    render.ssao_traced.hash(&mut hasher);
    if let Some(volume) = &render.probe_volume {
        volume.counts.hash(&mut hasher);
        for component in volume.origin.iter().chain(volume.spacing.iter()) {
            component.to_bits().hash(&mut hasher);
        }
        volume.intensity.to_bits().hash(&mut hasher);
        volume.hysteresis.to_bits().hash(&mut hasher);
        volume.reach.to_bits().hash(&mut hasher);
        volume.bounce.to_bits().hash(&mut hasher);
    }
    render.probe_volume.is_some().hash(&mut hasher);
    render.ibl_blend_factor.to_bits().hash(&mut hasher);
    hasher.finish()
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            atmosphere: Atmosphere::None,
            show_sky: true,
            render_layer_world_enabled: true,
            render_layer_overlay_enabled: true,
            render_world_to_swapchain: true,
            clear_color: [0.0, 0.0, 0.0, 1.0],
            swapchain_clear_color: None,
            unlit_mode: false,
            vertex_snap: None,
            affine_texture_mapping: false,
            material_anisotropy_filtering: 16,
            fog: None,
            color_grading: ColorGrading::default(),
            bloom_enabled: true,
            bloom_intensity: 0.04,
            bloom_threshold: 1.0,
            bloom_knee: 0.5,
            bloom_filter_radius: 0.005,
            depth_of_field: DepthOfField::default(),
            ssao_enabled: false,
            ssao_radius: 0.5,
            ssao_bias: 0.025,
            ssao_intensity: 1.0,
            ssao_sample_count: 64,
            ssao_blur_depth_threshold: 0.005,
            ssao_blur_normal_power: 8.0,
            ssgi_enabled: false,
            ssgi_radius: 2.0,
            ssgi_intensity: 1.0,
            ssgi_max_steps: 16,
            ssr_enabled: false,
            ssr_max_steps: 64,
            ssr_thickness: 0.3,
            ssr_max_distance: 50.0,
            ssr_stride: 1.0,
            ssr_fade_start: 0.8,
            ssr_fade_end: 1.0,
            ssr_intensity: 1.0,
            anti_aliasing: AntiAliasing::default(),
            taa_blend: 0.12,
            taa_sharpness: 0.5,
            render_scale: 1.0,
            hdr_output: false,
            hdr_paper_white: 200.0,
            hdr_peak_nits: 1000.0,
            ssr_traced: false,
            ssao_traced: false,
            sun_shadow_traced: true,
            named_shadow_casters: false,
            probe_volume: None,
            ibl_blend_factor: 0.0,
        }
    }
}

impl Default for DebugDraw {
    fn default() -> Self {
        Self {
            show_grid: false,
            show_bounding_volumes: false,
            show_selected_bounding_volume: false,
            show_normals: false,
            normal_line_length: 0.1,
            normal_line_color: [0.0, 1.0, 0.0, 1.0],
            pbr_debug_mode: PbrDebugMode::None,
            ssao_visualization: false,
            material_filtering: true,
        }
    }
}

pub const FLAT_SHADING_COLOR: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.72, 0.72, 0.72, 1.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ViewportUpdateMode {
    #[default]
    Always,
    WhenVisible,
    WhenDirty,
    Once,
    Disabled,
}

pub const WIREFRAME_SENTINEL_COLOR: nalgebra_glm::Vec4 =
    nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 2.0);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectiveShading {
    pub unlit_mode: bool,
    pub bloom_enabled: bool,
    pub ssao_enabled: bool,
    pub ssgi_enabled: bool,
    pub ssr_enabled: bool,
    pub anti_aliasing: AntiAliasing,
    pub taa_blend: f32,
    pub taa_sharpness: f32,
    pub show_normals: bool,
    pub show_bounding_volumes: bool,
    pub show_wireframe: bool,
    pub flat_shading_color: Option<nalgebra_glm::Vec4>,
    pub lines_enabled: bool,
    pub color_grading: ColorGrading,
    pub depth_of_field: DepthOfField,
    pub bloom_intensity: f32,
    pub bloom_threshold: f32,
    pub bloom_knee: f32,
    pub bloom_filter_radius: f32,
    pub ssao_radius: f32,
    pub ssao_bias: f32,
    pub ssao_intensity: f32,
    pub ssao_sample_count: u32,
    pub ssao_visualization: bool,
    pub pbr_debug_mode: PbrDebugMode,
    pub material_filtering: bool,
    pub vertex_snap: Option<VertexSnap>,
    pub affine_texture_mapping: bool,
    pub material_anisotropy_filtering: u16,
    pub render_scale: f32,
    pub hdr_output: bool,
    pub hdr_paper_white: f32,
    pub hdr_peak_nits: f32,
    pub ssr_traced: bool,
    pub ssao_traced: bool,
    pub sun_shadow_traced: bool,
    pub viewport_rect: Option<crate::data::config::ViewportRect>,
    pub ssao_blur_depth_threshold: f32,
    pub ssao_blur_normal_power: f32,
    pub ssgi_radius: f32,
    pub ssgi_intensity: f32,
    pub ssgi_max_steps: u32,
    pub ssr_max_steps: u32,
    pub ssr_thickness: f32,
    pub ssr_max_distance: f32,
    pub ssr_stride: f32,
    pub ssr_fade_start: f32,
    pub ssr_fade_end: f32,
    pub ssr_intensity: f32,
    pub culling_mask: u32,
    pub fog: Option<Fog>,
    pub probe_volume: Option<crate::data::config::ProbeVolume>,
    pub atmosphere: Atmosphere,
}

impl EffectiveShading {
    pub fn from_settings(render: &RenderSettings, debug: &DebugDraw) -> Self {
        Self {
            material_filtering: debug.material_filtering,
            unlit_mode: render.unlit_mode,
            bloom_enabled: render.bloom_enabled,
            ssao_enabled: render.ssao_enabled,
            ssgi_enabled: render.ssgi_enabled,
            ssr_enabled: render.ssr_enabled,
            anti_aliasing: render.anti_aliasing,
            taa_blend: render.taa_blend,
            taa_sharpness: render.taa_sharpness,
            show_normals: debug.show_normals,
            pbr_debug_mode: debug.pbr_debug_mode,
            vertex_snap: render.vertex_snap,
            affine_texture_mapping: render.affine_texture_mapping,
            material_anisotropy_filtering: render.material_anisotropy_filtering,
            render_scale: render.render_scale,
            hdr_output: render.hdr_output,
            hdr_paper_white: render.hdr_paper_white,
            hdr_peak_nits: render.hdr_peak_nits,
            ssr_traced: render.ssr_traced,
            ssao_traced: render.ssao_traced,
            sun_shadow_traced: render.sun_shadow_traced,
            viewport_rect: None,
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
        }
    }
}

impl Default for EffectiveShading {
    fn default() -> Self {
        Self {
            material_filtering: true,
            unlit_mode: false,
            bloom_enabled: true,
            ssao_enabled: false,
            ssgi_enabled: false,
            ssr_enabled: false,
            anti_aliasing: AntiAliasing::default(),
            taa_blend: 0.12,
            taa_sharpness: 0.5,
            show_normals: false,
            pbr_debug_mode: PbrDebugMode::None,
            vertex_snap: None,
            affine_texture_mapping: false,
            material_anisotropy_filtering: 16,
            render_scale: 1.0,
            hdr_output: false,
            hdr_paper_white: 200.0,
            hdr_peak_nits: 1000.0,
            ssr_traced: false,
            ssao_traced: false,
            sun_shadow_traced: true,
            viewport_rect: None,
            show_bounding_volumes: false,
            show_wireframe: false,
            flat_shading_color: None,
            lines_enabled: false,
            color_grading: ColorGrading::default(),
            depth_of_field: DepthOfField::default(),
            bloom_intensity: 0.5,
            bloom_threshold: 1.0,
            bloom_knee: 0.5,
            bloom_filter_radius: 0.005,
            ssao_radius: 0.5,
            ssao_bias: 0.025,
            ssao_intensity: 1.0,
            ssao_sample_count: 64,
            ssao_visualization: false,
            ssao_blur_depth_threshold: 0.005,
            ssao_blur_normal_power: 8.0,
            ssgi_radius: 2.0,
            ssgi_intensity: 1.0,
            ssgi_max_steps: 16,
            ssr_max_steps: 64,
            ssr_thickness: 0.3,
            ssr_max_distance: 50.0,
            ssr_stride: 1.0,
            ssr_fade_start: 0.8,
            ssr_fade_end: 1.0,
            ssr_intensity: 1.0,
            culling_mask: !0,
            fog: None,
            probe_volume: None,
            atmosphere: Atmosphere::None,
        }
    }
}
