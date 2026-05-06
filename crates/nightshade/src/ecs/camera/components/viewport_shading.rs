//! Per-camera viewport shading mode and overlay toggle, plus the
//! per-frame effective shading the renderer derives from it.
//!
//! [`ViewportShading`] is a component the editor (or any caller) attaches
//! to a camera entity to choose the shading mode and overlay visibility
//! for that camera's tile. The renderer reads it together with the global
//! [`Graphics`](crate::ecs::graphics::resources::Graphics) defaults to
//! produce an [`EffectiveShading`] for the camera, then writes it into
//! `world.resources.graphics.active_view` while the camera's render
//! graph executes. Passes read from `graphics.active_view`, never from
//! the user-set defaults on `Graphics` directly, so each camera renders
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

use crate::ecs::graphics::resources::{ColorGrading, DepthOfField, Graphics};
use crate::ecs::render_layer::components::CameraCullingMask;


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

pub const FLAT_SHADING_COLOR: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.72, 0.72, 0.72, 1.0);

#[derive(Debug, Clone, Copy)]
pub struct EffectiveShading {
    pub unlit_mode: bool,
    pub bloom_enabled: bool,
    pub ssao_enabled: bool,
    pub ssgi_enabled: bool,
    pub ssr_enabled: bool,
    pub show_normals: bool,
    pub show_bounding_volumes: bool,
    pub selection_outline_enabled: bool,
    pub flat_shading_color: Option<nalgebra_glm::Vec4>,
    pub shadow_depth_enabled: bool,
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
    pub ambient_light: [f32; 4],
    /// Bitmask of "culling layers" this view will render. Compared
    /// against each entity's [`CullingMask`](crate::ecs::render_layer::components::CullingMask)
    /// at render-collection time; an entity is skipped when
    /// `(entity_layers & culling_mask) == 0`. Defaults to `!0` (all
    /// layers) so cameras without a `CameraCullingMask` component
    /// continue to render every entity.
    pub culling_mask: u32,
}

impl EffectiveShading {
    pub fn from_graphics(graphics: &Graphics) -> Self {
        Self {
            unlit_mode: graphics.unlit_mode,
            bloom_enabled: graphics.bloom_enabled,
            ssao_enabled: graphics.ssao_enabled,
            ssgi_enabled: graphics.ssgi_enabled,
            ssr_enabled: graphics.ssr_enabled,
            show_normals: graphics.show_normals,
            show_bounding_volumes: graphics.show_bounding_volumes,
            selection_outline_enabled: graphics.selection_outline_enabled,
            flat_shading_color: None,
            shadow_depth_enabled: true,
            lines_enabled: graphics.show_normals || graphics.show_bounding_volumes,
            color_grading: graphics.color_grading,
            depth_of_field: graphics.depth_of_field,
            bloom_intensity: graphics.bloom_intensity,
            bloom_threshold: graphics.bloom_threshold,
            bloom_knee: graphics.bloom_knee,
            bloom_filter_radius: graphics.bloom_filter_radius,
            ssao_radius: graphics.ssao_radius,
            ssao_bias: graphics.ssao_bias,
            ssao_intensity: graphics.ssao_intensity,
            ssao_sample_count: graphics.ssao_sample_count,
            ssao_visualization: graphics.ssao_visualization,
            ssao_blur_depth_threshold: graphics.ssao_blur_depth_threshold,
            ssao_blur_normal_power: graphics.ssao_blur_normal_power,
            ssgi_radius: graphics.ssgi_radius,
            ssgi_intensity: graphics.ssgi_intensity,
            ssgi_max_steps: graphics.ssgi_max_steps,
            ssr_max_steps: graphics.ssr_max_steps,
            ssr_thickness: graphics.ssr_thickness,
            ssr_max_distance: graphics.ssr_max_distance,
            ssr_stride: graphics.ssr_stride,
            ssr_fade_start: graphics.ssr_fade_start,
            ssr_fade_end: graphics.ssr_fade_end,
            ssr_intensity: graphics.ssr_intensity,
            ambient_light: graphics.ambient_light,
            culling_mask: !0,
        }
    }

    pub fn for_camera(
        graphics: &Graphics,
        shading: &ViewportShading,
        post_process: Option<&CameraPostProcess>,
        culling_mask: Option<&CameraCullingMask>,
    ) -> Self {
        let mut effective = Self::from_graphics(graphics);
        if let Some(mask) = culling_mask {
            effective.culling_mask = mask.0;
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
                effective.show_normals = true;
                effective.show_bounding_volumes = true;
            }
        }
        if !shading.show_overlays {
            effective.show_normals = false;
            effective.show_bounding_volumes = false;
            effective.selection_outline_enabled = false;
        }
        effective.lines_enabled = match shading.mode {
            ShadingMode::Wireframe => shading.show_overlays,
            _ => effective.show_normals || effective.show_bounding_volumes,
        };
        effective
    }
}

impl Default for EffectiveShading {
    fn default() -> Self {
        Self {
            unlit_mode: false,
            bloom_enabled: true,
            ssao_enabled: false,
            ssgi_enabled: false,
            ssr_enabled: false,
            show_normals: false,
            show_bounding_volumes: false,
            selection_outline_enabled: false,
            flat_shading_color: None,
            shadow_depth_enabled: true,
            lines_enabled: false,
            color_grading: ColorGrading::default(),
            depth_of_field: DepthOfField::default(),
            bloom_intensity: 0.08,
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
            ambient_light: [0.1, 0.1, 0.1, 1.0],
            culling_mask: !0,
        }
    }
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

