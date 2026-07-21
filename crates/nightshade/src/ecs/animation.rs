//! Animation system for keyframe-based animation.
//!
//! Provides components and functions for skeletal and transform animation:
//!
//! - [`AnimationPlayer`]: Component controlling playback of animation clips
//! - [`AnimationClip`]: Collection of channels with keyframe data
//! - [`AnimationChannel`]: Targets a specific property on a node
//! - [`AnimationSampler`]: Keyframe times and interpolated values
//!
//! Supports glTF animations with linear, step, and cubic spline interpolation.
//! Animations can target translation, rotation, scale, and morph weights.
//! Cross-fading between clips is supported via [`AnimationPlayer::blend_to`].
//!
//! # Basic Playback
//!
//! Animations are loaded automatically from glTF files via [`spawn_prefab_with_animations`].
//! Control playback through the [`AnimationPlayer`] component:
//!
//! ```ignore
//! if let Some(player) = world.get_mut::<crate::ecs::animation::components::AnimationPlayer>(entity) {
//!     player.play(0);              // Play first animation from start
//!     player.looping = true;       // Enable looping
//!     player.speed = 1.5;          // 1.5x playback speed
//! }
//! ```
//!
//! # Cross-Fading Between Animations
//!
//! Smoothly transition between animations using [`AnimationPlayer::blend_to`]:
//!
//! ```ignore
//! if let Some(player) = world.get_mut::<crate::ecs::animation::components::AnimationPlayer>(entity) {
//!     // Blend from current animation to animation index 2 over 0.3 seconds
//!     player.blend_to(2, 0.3);
//! }
//! ```
//!
//! # Playback Control
//!
//! ```ignore
//! if let Some(player) = world.get_mut::<crate::ecs::animation::components::AnimationPlayer>(entity) {
//!     player.pause();              // Pause at current time
//!     player.resume();             // Continue from paused time
//!     player.stop();               // Stop and reset to beginning
//!     player.time = 0.5;           // Seek to 0.5 seconds
//! }
//! ```
//!
//! # Listing Available Animations
//!
//! ```ignore
//! if let Some(player) = world.get::<crate::ecs::animation::components::AnimationPlayer>(entity) {
//!     for (index, clip) in player.clips.iter().enumerate() {
//!         println!("{}: {} ({:.2}s)", index, clip.name, clip.duration);
//!     }
//! }
//! ```
//!
//! [`spawn_prefab_with_animations`]: crate::ecs::prefab::commands::spawn_prefab_with_animations

pub mod blend_tree;
pub mod components;
pub mod compression;
pub mod graph;
pub mod ik;
pub mod ragdoll;
pub mod retarget;
pub mod systems;

pub use blend_tree::*;
pub use components::*;
pub use compression::{clip_keyframe_count, compress_clip};
pub use graph::*;
pub use ik::{
    AimConstraint, FootDefinition, IkChain, IkChainMulti, look_at_bone, solve_two_bone_ik,
};
pub use ragdoll::Ragdoll;
pub use retarget::{RetargetMap, retarget_clip, retarget_map_by_suffix};
pub use systems::*;
