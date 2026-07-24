//! Physics-driven ragdoll for skinned skeletons. A ragdoll replaces the animated
//! pose with a chain of capsule bodies linked by spherical joints; the skeleton
//! is driven from those bodies each frame. The player is paused while ragdolled
//! so the GPU pose pipeline leaves its bones to the physics sync.

use nalgebra_glm::Quat;
use nightshade_ecs::Entity;
use serde::{Deserialize, Serialize};

/// One bone driven by a physics proxy body while ragdolled.
#[derive(Debug, Clone, PartialEq)]
pub struct RagdollLink {
    /// The proxy rigid-body entity.
    pub body: Entity,
    /// The skeleton bone this body drives.
    pub bone: Entity,
    /// Rotation carrying the body's orientation back to the bone's.
    pub rotation_offset: Quat,
}

/// A ragdoll rig over a set of bones. Build it with `start_ragdoll` and tear it
/// down with `recover_ragdoll` (both in the API facade).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Ragdoll {
    /// Bones the ragdoll drives, ordered root to tip.
    pub bones: Vec<String>,
    /// Capsule radius for the proxy bodies.
    pub radius: f32,
    /// Whether the ragdoll is simulating.
    pub active: bool,
    /// Runtime bone-to-body links.
    #[serde(skip)]
    pub links: Vec<RagdollLink>,
}
