//! Geometry render passes and their shared GPU data. Groups the forward mesh,
//! skinned mesh, and meshlet passes with the overlay, water, decal, sky, cloth,
//! and helper passes drawn against the scene color and depth targets, plus the
//! projection, cluster, material, and shadow-input helpers they share.

/// View and projection matrix construction plus the frustum-culling and
/// light-collection helpers the geometry passes share.
pub mod projection;
pub use crate::bounding_volume::extract_frustum_planes;
pub use projection::{atlas_shadow_fov, light_casts_atlas_shadow, sphere_in_frustum};

pub mod cluster;
pub mod ltc;
/// Conversion from CPU materials to the packed GPU material data.
pub mod material_gpu;
/// Shared shadow-input helpers the geometry passes read.
pub mod shadow_inputs;

mod cloth;
pub use cloth::{ClothPass, ClothWriteTarget};

mod decal;
pub use decal::DecalPass;

mod sky;
pub use sky::SkyPass;

#[cfg(feature = "grass")]
mod grass;
#[cfg(feature = "grass")]
pub use grass::GrassPass;

mod grid;
pub use grid::GridPass;

/// Debug line, bounding-volume, and vertex-normal overlay drawing.
pub mod lines;
pub use lines::{GpuBoundingVolumeData, LinesPass};

mod hiz;
pub use hiz::HizPass;

mod mesh;
pub use mesh::MeshPass;

#[cfg(feature = "meshlet")]
pub mod meshlet;
#[cfg(feature = "meshlet")]
pub use meshlet::MeshletPass;

mod scene_overlay;
pub use scene_overlay::SceneOverlayPass;

#[cfg(feature = "terrain")]
mod terrain;
#[cfg(feature = "terrain")]
pub use terrain::TerrainPass;

mod text;
pub use text::TextPass;

mod particles;
pub use particles::ParticlePass;

mod water;
pub use water::WaterPass;

mod skinned_mesh;
pub use skinned_mesh::{BlendedIblViews, SkinnedMeshPass};

mod ui_image;
pub use ui_image::UiImagePass;

mod ui_pass;
pub use ui_pass::UiPass;
