//! Graphics settings and post-processing configuration.
//!
//! Controls rendering options through the [`Graphics`] resource:
//!
//! - [`Graphics`]: Master graphics settings (bloom, SSAO, debug visualization)
//! - [`ColorGrading`]: Tonemapping, gamma, saturation, contrast
//! - [`DepthOfField`]: Bokeh blur, focus distance, tilt-shift
//! - [`Atmosphere`]: Skybox selection (procedural or HDR cubemap)
//! - [`Fog`]: Distance-based fog color and range
//! - [`PbrDebugMode`]: Visualize individual PBR components

pub mod resources;

pub use resources::*;
