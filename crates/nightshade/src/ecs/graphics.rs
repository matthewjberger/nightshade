//! Graphics settings and post-processing configuration.
//!
//! Rendering options are split across four resources:
//!
//! - [`RenderSettings`]: User-tunable shading (bloom, SSAO, SSGI, SSR, fog, atmosphere)
//! - [`DebugDraw`]: Debug visualization toggles (grid, normals, bounding volumes)
//! - [`Selection`](selection::Selection): Selected entities and outline highlight state
//! - [`RendererState`]: Runtime renderer bookkeeping (active view, version, LOD chains)
//! - [`ColorGrading`]: Tonemapping, gamma, saturation, contrast
//! - [`DepthOfField`]: Bokeh blur, focus distance, tilt-shift
//! - [`Atmosphere`]: Skybox selection (procedural or HDR cubemap)
//! - [`Fog`]: Distance-based fog color and range
//! - [`PbrDebugMode`]: Visualize individual PBR components

pub mod animation_sync;
pub mod scene_sync;
pub mod selection;
pub mod systems;
