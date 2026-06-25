//! Morph targets (blend shapes) for mesh deformation.
//!
//! Animate facial expressions, muscle flexing, and other mesh deformations:
//!
//! - [`MorphWeights`]: Component controlling blend shape weights
//!
//! Morph targets store per-vertex position/normal deltas. Weights control how
//! much each target affects the final mesh. glTF models with blend shapes
//! automatically get morph target support when loaded.
//!
//! # How Morph Targets Work
//!
//! A mesh can have multiple morph targets (e.g., "smile", "frown", "blink").
//! Each target stores vertex displacements. Setting a weight blends that
//! displacement into the mesh:
//!
//! - Weight 0.0 = no effect (base mesh)
//! - Weight 1.0 = full effect (target shape)
//! - Weight 0.5 = halfway blend
//!
//! # Accessing Morph Weights
//!
//! Models with morph targets get a [`MorphWeights`] component:
//!
//! ```ignore
//! if let Some(weights) = world.core.get_morph_weights_mut(entity) {
//!     // Check how many targets exist
//!     let count = weights.weight_count();
//!
//!     // Set individual weights
//!     weights.set_weight(0, 0.8);  // First target at 80%
//!     weights.set_weight(1, 0.3);  // Second target at 30%
//!
//!     // Read current weight
//!     let current = weights.get_weight(0);
//! }
//! ```
//!
//! # Animating Morph Targets
//!
//! Blend smoothly between expressions:
//!
//! ```ignore
//! fn run_systems(&mut self, world: &mut World) {
//!     let dt = world.resources.window.timing.delta_time;
//!
//!     if let Some(weights) = world.core.get_morph_weights_mut(face_entity) {
//!         // Animate smile weight
//!         let current = weights.get_weight(SMILE_INDEX);
//!         let target = if happy { 1.0 } else { 0.0 };
//!         let new_weight = current + (target - current) * dt * 5.0;
//!         weights.set_weight(SMILE_INDEX, new_weight);
//!     }
//! }
//! ```
//!
//! # Common Morph Target Indices
//!
//! glTF models often use these conventions (check your model):
//!
//! | Index | Common Name |
//! |-------|-------------|
//! | 0 | Basis / Neutral |
//! | 1-10 | Visemes (lip sync) |
//! | 11+ | Expressions (smile, frown, etc.) |
//!
//! # With Animation System
//!
//! The animation system can drive morph weights automatically. glTF animations
//! with morph target tracks update weights each frame via the animation player.
//!
//! ```ignore
//! // Load model with morph animations
//! let result = import_gltf_from_bytes(data)?;
//! let entity = spawn_prefab_with_animations(world, &result.prefabs[0], &result.animations, Vec3::zeros());
//!
//! // Play facial animation
//! if let Some(player) = world.core.get_animation_player_mut(entity) {
//!     player.play(TALK_ANIMATION);
//! }
//! // Morph weights are updated automatically by animation system
//! ```
//!
//! # Manual Morph Target Creation
//!
//! For procedural meshes:
//!
//! ```ignore
//! let weights = MorphWeights::from_count(3, "my_mesh");  // 3 morph targets
//! world.core.set_morph_weights(entity, weights);
//! ```
//!
//! [`MorphWeights`]: components::MorphWeights

pub mod components;
pub mod systems;

pub use components::*;
