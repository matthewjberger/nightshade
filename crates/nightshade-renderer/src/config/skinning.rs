//! The skeletal animation data the skinned mesh pass uploads: joints,
//! channels, IK and aim constraints, spring chains, and the per-frame
//! snapshot the GPU blends from.

/// A single animation channel flattened to GPU-ready arrays, produced by the
/// engine so the renderer never clones keyframe data per frame.
#[derive(Clone)]
pub struct SkinnedChannelData {
    /// Animated property selector (translation, rotation, scale, or weights).
    pub property: u32,
    /// Interpolation mode (step, linear, or cubic spline).
    pub interpolation: u32,
    /// Keyframe timestamps in seconds.
    pub input: Vec<f32>,
    /// Keyframe output values, one entry per timestamp.
    pub values: Vec<[f32; 4]>,
    /// Output values consumed per keyframe.
    pub stride: u32,
}

/// One joint of a skinned skeleton in the renderer's dispatch order, with its
/// rest pose and resolved animation channels for the current clip, the
/// cross-fade source clip, and each animation layer.
#[derive(Clone)]
pub struct SkinnedJointData {
    /// Index of this joint within the skeleton.
    pub local_index: u32,
    /// Parent joint index, `None` for a root joint.
    pub parent_local: Option<u32>,
    /// Rest-pose translation.
    pub rest_translation: [f32; 3],
    /// Rest-pose rotation quaternion.
    pub rest_rotation: [f32; 4],
    /// Rest-pose scale.
    pub rest_scale: [f32; 3],
    /// Animation channels for each active base-pose clip, in weight-blend order.
    pub pose_channels: Vec<Vec<SkinnedChannelData>>,
    /// Animation channels for each additional animation layer.
    pub layer_channels: Vec<Vec<SkinnedChannelData>>,
    /// Reference-pose channels per layer for additive blending, empty for
    /// override layers.
    pub reference_channels: Vec<Vec<SkinnedChannelData>>,
}

/// A two-bone IK chain resolved to local joint indices. The world-space target,
/// pole, and weight live in the per-frame `ik_runtime` map so moving the target
/// does not rebuild the static payload.
#[derive(Clone)]
pub struct SkinnedIkChainData {
    /// Index of this chain in the player's chain list, for the runtime lookup.
    pub chain_index: u32,
    /// Local joint index of the chain root.
    pub root_local: u32,
    /// Local joint index of the middle joint.
    pub mid_local: u32,
    /// Local joint index of the tip joint.
    pub tip_local: u32,
    /// Whether the chain uses a pole target to bias the bend.
    pub has_pole: bool,
}

/// An N-bone IK chain resolved to local joint indices, ordered root to tip, for
/// the GPU cyclic-coordinate-descent solver. Target, weight, and limits live in
/// the per-frame `ik_multi_runtime` map.
#[derive(Clone)]
pub struct SkinnedMultiIkChainData {
    /// Index of this chain in the player's multi-chain list, for the runtime lookup.
    pub chain_index: u32,
    /// Local joint indices ordered from root to tip.
    pub joint_locals: Vec<u32>,
}

/// A look-at constraint resolved to local joint indices, ordered base to tip.
/// The local forward axis is static; the target, weight, and cone limit live in
/// the per-frame `aim_runtime` map.
#[derive(Clone)]
pub struct SkinnedAimConstraintData {
    /// Index of this constraint in the player's aim list, for the runtime lookup.
    pub chain_index: u32,
    /// Local joint indices ordered from the base of the chain to the tip.
    pub joint_locals: Vec<u32>,
    /// The bones' local forward axis.
    pub forward: [f32; 3],
}

/// A spring-bone chain resolved to local joint indices, ordered root to tip,
/// with each joint's rest direction and bone length. The dynamics parameters
/// live in the per-frame `spring_runtime` map.
#[derive(Clone)]
pub struct SkinnedSpringChainData {
    /// Index of this chain in the player's spring list, for the runtime lookup.
    pub chain_index: u32,
    /// Local joint indices ordered root to tip.
    pub joint_locals: Vec<u32>,
    /// Per joint local rest direction from its parent; the first entry is unused.
    pub rest_axes: Vec<[f32; 3]>,
    /// Per joint length from its parent; the first entry is unused.
    pub bone_lengths: Vec<f32>,
}

/// One animated skeleton resolved by the engine: its skin and player entities,
/// depth-ordered joints, and how many animation layers the player exposes.
#[derive(Clone)]
pub struct SkinnedSkeletonData {
    /// Entity carrying the skin.
    pub skin_entity: nightshade_ecs::Entity,
    /// Entity carrying the animation player.
    pub player_entity: nightshade_ecs::Entity,
    /// The entity above the skeleton's root joints, captured at build time so
    /// the per-frame armature-root refresh is one transform read instead of a
    /// joint-parent walk.
    pub armature_parent: Option<nightshade_ecs::Entity>,
    /// Number of joints in the skeleton.
    pub joint_count: u32,
    /// Number of active base-pose clips the player blends.
    pub active_count: u32,
    /// Number of animation layers the player exposes.
    pub layer_count: u32,
    /// Blend mode per layer: `0` override, `1` additive.
    pub layer_modes: Vec<u32>,
    /// Local joint index whose translation is locked to rest so root motion
    /// drives the entity instead of the rendered pose, `None` when disabled.
    pub root_motion_local_index: Option<u32>,
    /// Two-bone IK chains solved on the GPU pose after blending.
    pub ik_chains: Vec<SkinnedIkChainData>,
    /// Multi-bone IK chains solved on the GPU with cyclic coordinate descent.
    pub ik_multi_chains: Vec<SkinnedMultiIkChainData>,
    /// Look-at constraints solved on the GPU.
    pub aim_constraints: Vec<SkinnedAimConstraintData>,
    /// Spring-bone chains simulated on the GPU.
    pub spring_chains: Vec<SkinnedSpringChainData>,
    /// Joints in the renderer's depth-first dispatch order.
    pub joints_ordered: Vec<SkinnedJointData>,
}

/// Skinning palette resolved once per frame by the engine so the skinned-mesh
/// and shadow passes build their GPU buffers without reading Skin components or
/// bone transforms. The static layout (`cache`) is rebuilt only when the skinned
/// set changes, bumping `static_generation`; `bone_transforms` refreshes every
/// frame.
#[derive(Default, Clone)]
pub struct RenderSkinning {
    /// Static skinning layout, rebuilt only when the skinned set changes.
    pub cache: crate::skinning::SkinningCache,
    /// Current bone matrices for every skinned joint.
    pub bone_transforms: Vec<nalgebra_glm::Mat4>,
    /// Bumped whenever `cache` is rebuilt.
    pub static_generation: u64,
    /// Bumped whenever any bone transform changed since the last sync, so
    /// consumers can gate uploads on a counter instead of comparing the
    /// whole palette.
    pub bone_transforms_generation: u64,
}

/// Animation state resolved once per frame by the engine so the skinned-mesh
/// compute driver builds its GPU buffers without reading ECS animation, skin,
/// parent, or transform components. The heavy `skeletons` payload is rebuilt
/// only when the engine's animation gate sees a change, bumping `signature`;
/// the runtime maps refresh every frame.
#[derive(Clone, Default)]
pub struct SkinnedAnimationSnapshot {
    /// Bumped when the heavy `skeletons` payload is rebuilt.
    pub signature: u64,
    /// Resolved skeletons, rebuilt only when the animation gate sees a change.
    pub skeletons: Vec<SkinnedSkeletonData>,
    /// Per active base-pose clip runtime state (time and weight), keyed by player
    /// and active-clip index.
    pub pose_runtime: std::collections::HashMap<(nightshade_ecs::Entity, usize), [f32; 2]>,
    /// Per-layer runtime state (time and weight), keyed by player and layer.
    pub layer_runtime: std::collections::HashMap<(nightshade_ecs::Entity, usize), [f32; 2]>,
    /// Per IK chain runtime, keyed by player and chain index: target xyz, pole
    /// xyz, weight, and enabled flag (`1.0` when the chain solves this frame).
    pub ik_runtime: std::collections::HashMap<(nightshade_ecs::Entity, usize), [f32; 8]>,
    /// Per multi-bone IK chain runtime: target xyz, weight, enabled, max angle,
    /// iterations, pad.
    pub ik_multi_runtime: std::collections::HashMap<(nightshade_ecs::Entity, usize), [f32; 8]>,
    /// Per aim constraint runtime: target xyz, weight, enabled, cone angle, pad.
    pub aim_runtime: std::collections::HashMap<(nightshade_ecs::Entity, usize), [f32; 8]>,
    /// Per spring chain runtime: gravity xyz, stiffness, damping, enabled, delta
    /// time, pad.
    pub spring_runtime: std::collections::HashMap<(nightshade_ecs::Entity, usize), [f32; 8]>,
    /// Current armature-root world transform for each player.
    pub armature_roots: std::collections::HashMap<nightshade_ecs::Entity, nalgebra_glm::Mat4>,
}
