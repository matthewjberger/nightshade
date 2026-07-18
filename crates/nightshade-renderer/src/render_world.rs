//! The renderer's own ECS schema.
//!
//! The renderer owns this world. A standalone host stands one up with
//! `freecs::dynamic::DynWorld::from_registry(register_render_components())`; a
//! host with its own freecs group composes it in as a member world with
//! `add_world_at`, sharing entity identity so no scene data is copied across
//! the boundary. Nothing here names a type from above the renderer, so the
//! renderer stays reasoned about in isolation and cannot couple upward.

use nalgebra_glm::Mat4;

/// World-space transform of a renderable.
#[derive(Clone, Copy, PartialEq)]
pub struct Transform(pub Mat4);

impl Default for Transform {
    fn default() -> Self {
        Self(Mat4::identity())
    }
}

/// The mesh name a renderable draws, resolved against the mesh cache.
#[derive(Clone, Default, PartialEq)]
pub struct MeshName(pub String);

/// Material id: `entries[id - 1]` in the material table, or the built-in
/// default at 0.
#[derive(Clone, Copy, Default, PartialEq)]
pub struct MaterialId(pub u32);

/// Whether the renderable draws this frame.
#[derive(Clone, Copy, PartialEq)]
pub struct Visible(pub bool);

impl Default for Visible {
    fn default() -> Self {
        Self(true)
    }
}

/// Per-renderable morph-target blend weights.
#[derive(Clone, Copy, Default, PartialEq)]
pub struct MorphWeights(pub [f32; 8]);

/// Culling-layer bitmask compared against the camera's mask.
#[derive(Clone, Copy, PartialEq)]
pub struct CullingMask(pub u32);

impl Default for CullingMask {
    fn default() -> Self {
        Self(u32::MAX)
    }
}

/// The world-vs-overlay render layer.
#[derive(Clone, Copy, Default, PartialEq)]
pub struct Layer(pub u32);

/// A GPU particle emitter's render state. Its own type (rather than
/// [`crate::particles::ParticleEmitter`] directly) so a host that authors
/// emitters in another member world can still register this one without a
/// type living in two worlds at once.
#[derive(Clone, Default)]
pub struct RenderEmitter(pub crate::particles::ParticleEmitter);

/// Marker: this renderable casts shadows. A sparse-set tag, so adding it never
/// migrates the entity's archetype.
pub struct ShadowCaster;

freecs::dynamic_schema! {
    pub fn register_render_components {
        transform: Transform => TRANSFORM,
        mesh: MeshName => MESH,
        material: MaterialId => MATERIAL,
        visible: Visible => VISIBLE,
        morph_weights: MorphWeights => MORPH_WEIGHTS,
        culling_mask: CullingMask => CULLING_MASK,
        layer: Layer => LAYER,
        light: crate::config::RenderLightData => LIGHT,
        decal: crate::config::RenderDecalData => DECAL,
        water: crate::config::RenderWaterData => WATER,
        emitter: RenderEmitter => EMITTER,
        instanced: crate::config::InstancedObjectData => INSTANCED,
    }
}
