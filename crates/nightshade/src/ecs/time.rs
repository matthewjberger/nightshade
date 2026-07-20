//! The frame clock. The [`Time`] type lives in `nightshade-ecs`; this module
//! re-exports it so `crate::ecs::time::Time` keeps resolving across the engine.

pub use nightshade_ecs::time::Time;
