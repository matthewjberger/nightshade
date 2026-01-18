//! Transform system for spatial positioning and hierarchies.
//!
//! This module provides the core transform components and systems:
//!
//! - [`LocalTransform`]: Position, rotation, and scale relative to parent (or world if no parent)
//! - [`GlobalTransform`]: Computed world-space transformation matrix
//! - [`Parent`]: Establishes parent-child hierarchy relationships
//! - [`Rotation`]: Continuous rotation animation helper
//! - [`IgnoreParentScale`]: Marker to exclude parent scale from transform inheritance
//!
//! The transform system automatically propagates [`LocalTransform`] changes through
//! the entity hierarchy to compute [`GlobalTransform`] matrices each frame.

pub mod commands;
pub mod components;
pub mod queries;
pub mod systems;

pub use commands::*;
pub use components::*;
pub use queries::*;
pub use systems::*;
