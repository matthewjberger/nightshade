//! Mesh system for 3D geometry.
//!
//! Provides components and data structures for renderable geometry:
//!
//! - [`RenderMesh`]: Component referencing a named mesh in the cache
//! - [`Mesh`]: CPU-side mesh data with vertices, indices, and optional skin/morph data
//! - [`Vertex`]: Standard vertex with position, normal, tangent, UVs, and color
//! - [`SkinnedVertex`]: Vertex with joint indices and weights for skeletal animation
//! - [`InstancedMesh`]: Component for GPU instancing of repeated geometry
//!
//! Built-in primitive generators create cube, sphere, plane, cylinder, cone, and torus meshes.

pub mod commands;
pub mod components;
pub mod queries;
pub mod systems;

pub use commands::*;
pub use components::*;
pub use queries::*;
pub use systems::*;
