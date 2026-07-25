//! Graphics settings and the renderer's data model.
//!
//! Split by who reads it: what a program sets, what one camera view
//! needs, the environment, the post-process chain, and the per-frame
//! scene data (lights, materials, objects, skinning) the passes upload.

mod device;
mod environment;
mod lights;
mod materials;
mod objects;
mod postprocess;
mod settings;
mod skinning;
mod state;
mod view;

pub use device::*;
pub use environment::*;
pub use lights::*;
pub use materials::*;
pub use objects::*;
pub use postprocess::*;
pub use settings::*;
pub use skinning::*;
pub use state::*;
pub use view::*;
