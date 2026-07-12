//! Per-camera viewport shading mode and overlay toggle, plus the
//! per-frame effective shading the renderer derives from it.
//!
//! [`ViewportShading`] is a component the editor (or any caller) attaches
//! to a camera entity to choose the shading mode and overlay visibility
//! for that camera's tile. The renderer reads it together with the global
//! [`RenderSettings`](crate::render::config::RenderSettings) defaults to
//! produce an [`EffectiveShading`] for the camera, then writes it into
//! `world.resources.renderer_state.active_view` while the camera's render
//! graph executes. Passes read from `renderer_state.active_view`, never from
//! the user-set defaults on `RenderSettings` directly, so each camera renders
//! with its own view-local shading without the renderer having to
//! mutate or restore unrelated global state between cameras.
//!
//! [`ShadingMode`] selects the post-processing stack:
//!
//! - `Rendered`: full pipeline (bloom, SSAO, SSGI, SSR, lit shading).
//! - `Solid`: unlit base color with all post-processing disabled.
//! - `Flat`: lit shading with every material's surface color, metallic,
//!   roughness, and emissive replaced with CAD-style neutral values.
//! - `Wireframe`: solid base + GPU-driven edge overlay through the
//!   existing lines pass.
//!
//! `show_overlays` gates editor-visible decorations (gizmos, debug lines,
//! the selection outline) for that camera only. The world ground grid is
//! a separate global toggle.

use crate::ecs::primitives::CameraCullingMask;
use crate::render::config::{
    Atmosphere, ColorGrading, DebugDraw, DepthOfField, Fog, RenderSettings,
};

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

use crate::render::config::{EffectiveShading, FLAT_SHADING_COLOR, WIREFRAME_SENTINEL_COLOR};

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
        show_normals: debug.show_normals,
        show_bounding_volumes: debug.show_bounding_volumes,
        show_wireframe: false,
        selection_outline_enabled: debug.selection_outline_enabled,
        flat_shading_color: None,
        shadow_depth_enabled: true,
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
        ambient_light: render.ambient_light,
        culling_mask: !0,
        fog: render.fog,
        atmosphere: render.atmosphere,
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
        if let Some(value) = overrides.ambient_light {
            effective.ambient_light = value;
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
            effective.shadow_depth_enabled = false;
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
            effective.shadow_depth_enabled = false;
            effective.flat_shading_color = Some(WIREFRAME_SENTINEL_COLOR);
            effective.show_wireframe = true;
        }
    }
    if !shading.show_overlays {
        effective.show_normals = false;
        effective.show_bounding_volumes = false;
        effective.selection_outline_enabled = false;
    }
    effective.lines_enabled =
        effective.show_normals || effective.show_bounding_volumes || effective.show_wireframe;
    effective
}

/// Locks a camera's projection aspect to a fixed value regardless
/// of the tile size it renders into. The renderer fits a
/// constrained-aspect inner rect inside the tile and letterboxes /
/// pillarboxes the remaining bars in the tile background. Without
/// this component the camera's projection aspect matches the tile
/// rect (the default behavior). With it, the aspect stays
/// constant and the rendered region is centered inside the tile.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ConstrainedAspect(pub f32);

impl Default for ConstrainedAspect {
    fn default() -> Self {
        Self(16.0 / 9.0)
    }
}

/// Per-camera environment overrides for atmosphere and fog. Each
/// field opts into changing the resolved view state. `Inherit`
/// for fog or `None` for atmosphere keeps the corresponding
/// global `Graphics` value.
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

/// Per-camera post-process parameter overrides. Anything left as
/// `None` (or `Default`) inherits the corresponding global value
/// from `Graphics`. The renderer resolves overrides into
/// `graphics.active_view` per camera each frame so passes read a
/// single set of resolved values.
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
    pub ambient_light: Option<[f32; 4]>,
}

/// Update mode for a camera's viewport tile.
///
/// - `Always`: re-render every frame regardless of dirty state.
/// - `WhenVisible`: same as `Always` for now (the engine treats any camera
///   in `required_cameras` as visible). Reserved for future
///   visibility/focus tracking.
/// - `WhenDirty`: re-render only when the active camera, when first
///   becoming visible (no cached frame yet), or when the frame's coarse
///   dirty signal indicates a transform/scene mutation. Otherwise blit
///   the cached viewport texture from the previous render.
/// - `Once`: render exactly the first time the camera is visible, then
///   reuse the cached texture forever.
/// - `Disabled`: never render after the first frame; the cached texture
///   is whatever the first render produced (or zero-initialized memory if
///   the camera was never rendered).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ViewportUpdateMode {
    #[default]
    Always,
    WhenVisible,
    WhenDirty,
    Once,
    Disabled,
}
