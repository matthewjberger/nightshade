use nightshade_render::data::config::{Atmosphere, ColorGrading, DepthOfField, Fog};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ShadingMode {
    Wireframe,
    Solid,
    Flat,
    #[default]
    Rendered,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ViewportShading {
    pub mode: ShadingMode,
    pub show_overlays: bool,
}

impl Default for ViewportShading {
    fn default() -> Self {
        Self {
            mode: ShadingMode::Rendered,
            show_overlays: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ConstrainedAspect(pub f32);

impl Default for ConstrainedAspect {
    fn default() -> Self {
        Self(16.0 / 9.0)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CameraEnvironment {
    pub atmosphere: Option<Atmosphere>,
    pub fog: FogOverride,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FogOverride {
    #[default]
    Inherit,
    Disabled,
    Override(Fog),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CameraPostProcess {
    pub color_grading: Option<ColorGrading>,
    pub depth_of_field: Option<DepthOfField>,
    pub bloom_intensity: Option<f32>,
    pub bloom_threshold: Option<f32>,
    pub bloom_knee: Option<f32>,
    pub bloom_filter_radius: Option<f32>,
    pub ssao_intensity: Option<f32>,
    pub ssao_radius: Option<f32>,
    pub ssgi_intensity: Option<f32>,
    pub ssr_intensity: Option<f32>,
}
