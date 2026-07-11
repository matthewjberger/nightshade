//! The engine-side render driver: everything that reads or writes the ECS
//! `World` to feed the renderer. It drains render commands, executes loading
//! tasks, generates text meshes, extracts per-frame dirty state, resolves
//! per-camera views, and drives the render graph. The renderer itself never
//! sees the `World`; it consumes the `RenderConfigs` contract and the GPU
//! resources this driver uploads.

#[cfg(feature = "egui")]
mod egui;
mod extract;
mod frame;
mod lighting;
mod lines;
mod shadow_atlas;
pub mod text_mesh;

pub use frame::*;
pub use shadow_atlas::*;
