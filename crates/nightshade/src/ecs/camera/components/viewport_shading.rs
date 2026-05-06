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

use crate::ecs::graphics::resources::Graphics;


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
        }
    }

    pub fn for_camera(graphics: &Graphics, shading: &ViewportShading) -> Self {
        let mut effective = Self::from_graphics(graphics);
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
        }
    }
}

/// Update mode for a camera's viewport tile, mirroring Godot's
/// `SubViewport.update_mode` and Unreal's per-viewport "Realtime" toggle.
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

