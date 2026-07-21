//! The engine-side render driver: everything that reads or writes the ECS
//! `World` around a frame. Before the frame it polls readbacks, drains the
//! loading pipeline, generates text meshes, and composes the `RenderInputs`
//! snapshot; after the frame it restores the taken state and applies the
//! frame's outputs. The frame itself never sees the `World`: once the inputs
//! are composed, rendering runs against them alone, so a mid-frame read of a
//! taken resource is unrepresentable rather than a latent bug.

mod extract;
mod frame;
mod inputs;
mod lines;
mod viewport;

pub use frame::*;
pub use inputs::*;
pub use viewport::Viewport;
