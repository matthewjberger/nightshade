//! Per-camera viewport shading mode and overlay toggle.
//!
//! Attached to a camera entity to override the engine's global graphics
//! settings while that camera is rendering its tile. The render system
//! reads this component before each camera-tile pass, snapshots the
//! relevant `world.resources.graphics` flags, applies the overrides for
//! that camera, runs the pass, then restores the snapshot. This keeps
//! the existing render-graph plumbing untouched while letting different
//! viewport tiles in an editor show the same scene with different
//! shading modes.
//!
//! [`ShadingMode`] selects the post-processing stack:
//!
//! - `Rendered`: full pipeline (bloom, SSAO, SSGI, SSR, lit shading).
//! - `Solid`: unlit base color with all post-processing disabled.
//! - `Wireframe`: solid base + GPU-driven edge overlay rendered through
//!   the existing lines pass (silhouette + crease edges derived from the
//!   depth/normal buffer; no per-mesh edge index buffer needed, so it
//!   works on every backend including WebGPU).
//!
//! `show_overlays` gates editor-visible decorations (grid, gizmos, the
//! navigation gizmo, debug lines) for the camera's tile only.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ShadingMode {
    Wireframe,
    Solid,
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
