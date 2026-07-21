//! Animation component definitions.

use crate::ecs::animation::blend_tree::BlendTree;
use crate::ecs::animation::graph::AnimationGraph;
use crate::ecs::animation::ik::{AimConstraint, FootDefinition, IkChain, IkChainMulti};
use crate::ecs::animation::ragdoll::Ragdoll;
use nalgebra_glm::{Quat, Vec3};
use nightshade_ecs::Entity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// One weighted contribution to the base pose. The graph and blend-tree
/// interpreters emit these; the pose pipeline blends the whole set into
/// `bone_transforms`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveClip {
    /// Index into the player's clip list.
    pub clip_index: usize,
    /// Playback time in seconds.
    pub time: f32,
    /// Current blend weight before normalization.
    pub weight: f32,
    /// Weight this clip ramps toward.
    pub target_weight: f32,
    /// Weight change per second; 0.0 snaps to `target_weight`.
    pub fade_rate: f32,
    /// Playback speed multiplier.
    pub speed: f32,
    /// How this clip behaves at its end.
    pub loop_mode: LoopMode,
    /// The clip that mirrors `AnimationPlayer::current_clip`/`time` for the legacy
    /// playback API and seeks.
    pub dominant: bool,
}

/// A named animation sequence with duration and channels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationClip {
    /// Name of the animation (e.g., "Walk", "Idle").
    pub name: String,
    /// Total duration in seconds.
    pub duration: f32,
    /// Channels targeting different properties on nodes.
    pub channels: Vec<AnimationChannel>,
    /// Named markers along the timeline. When playback crosses one, the player
    /// emits an `AnimationEvent`, the cue for footsteps, hit frames, and effects.
    #[serde(default)]
    pub events: Vec<AnimationEvent>,
}

/// A named marker at a point on a clip's timeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationEvent {
    /// Time in seconds from the clip's start.
    pub time: f32,
    /// Label published with the emitted event.
    pub name: String,
}

/// How a layer combines with the base pose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LayerBlendMode {
    /// Blends toward the layer pose, overriding the bones it touches.
    #[default]
    Override,
    /// Adds the layer's difference from a reference pose on top of the base.
    Additive,
}

/// An additional clip blended on top of the base animation, optionally limited
/// to a subset of bones. Layers drive composites like a wave played on the upper
/// body while the legs keep walking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationLayer {
    /// Index of the clip this layer plays.
    pub clip_index: usize,
    /// Blend weight toward this layer's pose, 0.0 ignores it, 1.0 fully
    /// overrides the bones it touches.
    pub weight: f32,
    /// Playback time in seconds.
    pub time: f32,
    /// Playback speed multiplier.
    pub speed: f32,
    /// Whether the layer is advancing.
    pub playing: bool,
    /// Whether the layer loops at the end of its clip.
    pub looping: bool,
    /// Bone names this layer affects. Empty means every bone the clip targets.
    #[serde(default)]
    pub mask: Vec<String>,
    /// How the layer combines with the base pose.
    #[serde(default)]
    pub blend_mode: LayerBlendMode,
    /// Reference clip an additive layer is measured against, sampled at its
    /// first frame; ignored for override layers.
    #[serde(default)]
    pub reference_clip: Option<usize>,
}

impl AnimationLayer {
    /// A new looping layer at full weight on the given clip.
    pub fn new(clip_index: usize) -> Self {
        Self {
            clip_index,
            weight: 1.0,
            time: 0.0,
            speed: 1.0,
            playing: true,
            looping: true,
            mask: Vec::new(),
            blend_mode: LayerBlendMode::Override,
            reference_clip: None,
        }
    }

    /// Sets the blend weight.
    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self
    }

    /// Restricts the layer to the named bones.
    pub fn with_mask(mut self, mask: Vec<String>) -> Self {
        self.mask = mask;
        self
    }
}

/// Targets a specific property on a node for animation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationChannel {
    /// Index of the target node in the glTF hierarchy.
    pub target_node: usize,
    /// Optional bone name for skeleton-based lookup.
    pub target_bone_name: Option<String>,
    /// Which property to animate.
    pub target_property: AnimationProperty,
    /// Keyframe data and interpolation settings.
    pub sampler: AnimationSampler,
}

/// Properties that can be animated.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AnimationProperty {
    /// Position offset.
    Translation,
    /// Orientation.
    Rotation,
    /// Scale factors.
    Scale,
    /// Morph target blend weights.
    MorphWeights,
}

/// Keyframe data with times and output values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationSampler {
    /// Keyframe times in seconds.
    pub input: Vec<f32>,
    /// Output values at each keyframe.
    pub output: AnimationSamplerOutput,
    /// How to interpolate between keyframes.
    pub interpolation: AnimationInterpolation,
}

/// Output value types for animation samplers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnimationSamplerOutput {
    /// Translation or scale values.
    Vec3(Vec<Vec3>),
    /// Rotation values.
    Quat(Vec<Quat>),
    /// Morph weight arrays.
    Weights(Vec<Vec<f32>>),
    /// Cubic spline Vec3 with tangents.
    CubicSplineVec3 {
        values: Vec<Vec3>,
        in_tangents: Vec<Vec3>,
        out_tangents: Vec<Vec3>,
    },
    /// Cubic spline quaternion with tangents.
    CubicSplineQuat {
        values: Vec<Quat>,
        in_tangents: Vec<Quat>,
        out_tangents: Vec<Quat>,
    },
    /// Cubic spline weights with tangents.
    CubicSplineWeights {
        values: Vec<Vec<f32>>,
        in_tangents: Vec<Vec<f32>>,
        out_tangents: Vec<Vec<f32>>,
    },
}

/// Interpolation method between keyframes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationInterpolation {
    /// Linear interpolation (lerp for Vec3, slerp for Quat).
    Linear,
    /// Immediate jump to next value.
    Step,
    /// Hermite cubic spline interpolation.
    CubicSpline,
}

/// A chain of bones that dangles with spring dynamics for secondary motion like
/// hair, cloth, tails, and antennae. `bones` is ordered root to tip; the root
/// follows the animation and the rest lag with inertia, pulled back toward the
/// animated pose by `stiffness` and slowed by `damping`. Solved on the GPU.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpringChain {
    /// Bone names ordered from the fixed root to the dangling tip.
    pub bones: Vec<String>,
    /// Pull back toward the animated pose per frame, 0.0 loose to 1.0 rigid.
    pub stiffness: f32,
    /// Velocity damping per frame, 0.0 free to 1.0 dead.
    pub damping: f32,
    /// Constant world-space acceleration, e.g. gravity.
    pub gravity: Vec3,
    /// Whether the chain simulates this frame.
    pub enabled: bool,
}

/// How playback behaves at the end of a clip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LoopMode {
    /// Wraps back to the start and keeps playing.
    #[default]
    Loop,
    /// Stops at the last frame and marks playback finished.
    Once,
    /// Holds the last frame indefinitely without finishing.
    Clamp,
    /// Reflects at each end, playing forward then backward.
    PingPong,
}

/// Wraps `time` into a clip of `duration` under a loop mode, returning the new
/// time and whether playback continues.
pub fn wrap_time(time: f32, duration: f32, mode: LoopMode) -> (f32, bool) {
    if duration <= 0.0 || time < duration {
        return (time.max(0.0), true);
    }
    match mode {
        LoopMode::Loop => (time.rem_euclid(duration), true),
        LoopMode::Once => (duration, false),
        LoopMode::Clamp => (duration, true),
        LoopMode::PingPong => {
            let cycle = duration * 2.0;
            let wrapped = time.rem_euclid(cycle);
            let reflected = if wrapped > duration {
                cycle - wrapped
            } else {
                wrapped
            };
            (reflected, true)
        }
    }
}

/// Resolves the effective loop mode from the mode plus the legacy `looping`
/// flag, so `looping = false` still means play once.
pub fn effective_loop_mode(mode: LoopMode, looping: bool) -> LoopMode {
    match mode {
        LoopMode::Loop if !looping => LoopMode::Once,
        other => other,
    }
}

/// Component controlling animation playback on an entity hierarchy.
///
/// Manages one or more animation clips, playback state, and cross-fading.
/// Maps animation targets to entities via node indices or bone names.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationPlayer {
    /// All available animation clips.
    pub clips: Vec<AnimationClip>,
    /// Index of the currently playing clip.
    pub current_clip: Option<usize>,
    /// Current playback time in seconds.
    pub time: f32,
    /// Playback speed multiplier (1.0 = normal).
    pub speed: f32,
    /// Whether to loop when reaching the end.
    pub looping: bool,
    /// How the current clip behaves at its end.
    #[serde(default)]
    pub loop_mode: LoopMode,
    /// Whether the animation is currently advancing.
    pub playing: bool,
    /// Whether to play all clips sequentially.
    pub play_all: bool,
    /// Maps glTF node indices to entities.
    pub node_index_to_entity: HashMap<usize, Entity>,
    /// Maps bone names to entities for skeleton lookup.
    pub bone_name_to_entity: HashMap<String, Entity>,
    /// Previous clip index for cross-fade blending.
    pub blend_from_clip: Option<usize>,
    /// Time position in the previous clip during blend.
    pub blend_from_time: f32,
    /// Current blend weight (0 = old clip, 1 = new clip).
    pub blend_factor: f32,
    /// Duration of the cross-fade transition.
    pub blend_duration: f32,
    /// Extra clips composited on top of the base animation, each optionally
    /// masked to a subset of bones.
    #[serde(default)]
    pub layers: Vec<AnimationLayer>,
    /// The weighted clip set the pose pipeline blends. Rebuilt each frame by the
    /// graph/blend-tree interpreters or the legacy playback API.
    #[serde(default, skip)]
    pub active_clips: Vec<ActiveClip>,
    /// Named parameters driving blend trees and graph transitions.
    #[serde(default)]
    pub parameters: HashMap<String, f32>,
    /// Optional blend tree producing the base pose from `parameters`.
    #[serde(default)]
    pub blend_tree: Option<BlendTree>,
    /// Optional state machine; supersedes `blend_tree` when present.
    #[serde(default)]
    pub graph: Option<AnimationGraph>,
    /// Two-bone IK chains solved on the GPU pose after blending.
    #[serde(default)]
    pub ik_chains: Vec<IkChain>,
    /// Multi-bone IK chains solved on the GPU with cyclic coordinate descent.
    #[serde(default)]
    pub ik_chains_multi: Vec<IkChainMulti>,
    /// Look-at constraints aiming bone chains at world targets, solved on the GPU.
    #[serde(default)]
    pub aim_constraints: Vec<AimConstraint>,
    /// Spring-bone chains for secondary motion, simulated on the GPU.
    #[serde(default)]
    pub spring_chains: Vec<SpringChain>,
    /// Foot-placement rig steering leg IK chains onto the ground.
    #[serde(default)]
    pub foot_ik: Vec<FootDefinition>,
    /// Whether foot placement runs this frame.
    #[serde(default)]
    pub foot_ik_enabled: bool,
    /// Optional physics ragdoll driving the skeleton from proxy bodies.
    #[serde(default)]
    pub ragdoll: Option<Ragdoll>,
    /// Whether root motion drives the entity transform from the root bone.
    #[serde(default)]
    pub root_motion: bool,
    /// Name of the bone whose translation feeds root motion.
    #[serde(default)]
    pub root_motion_bone: Option<String>,
    /// Whether root motion is confined to the ground plane, leaving vertical
    /// travel to terrain following and foot placement.
    #[serde(default)]
    pub root_motion_planar: bool,
    /// Previous-frame root translation, for per-frame delta extraction.
    #[serde(default, skip)]
    pub root_motion_previous: Option<Vec3>,
    /// Previous-frame player time, to detect loop wraps in root motion.
    #[serde(default, skip)]
    pub root_motion_previous_time: f32,
}

impl Default for AnimationPlayer {
    fn default() -> Self {
        Self {
            clips: Vec::new(),
            current_clip: None,
            time: 0.0,
            speed: 1.0,
            looping: true,
            loop_mode: LoopMode::Loop,
            playing: false,
            play_all: false,
            node_index_to_entity: HashMap::new(),
            bone_name_to_entity: HashMap::new(),
            blend_from_clip: None,
            blend_from_time: 0.0,
            blend_factor: 1.0,
            blend_duration: 0.0,
            layers: Vec::new(),
            active_clips: Vec::new(),
            parameters: HashMap::new(),
            blend_tree: None,
            graph: None,
            ik_chains: Vec::new(),
            ik_chains_multi: Vec::new(),
            aim_constraints: Vec::new(),
            spring_chains: Vec::new(),
            foot_ik: Vec::new(),
            foot_ik_enabled: false,
            ragdoll: None,
            root_motion: false,
            root_motion_bone: None,
            root_motion_planar: false,
            root_motion_previous: None,
            root_motion_previous_time: 0.0,
        }
    }
}

impl AnimationPlayer {
    /// Starts playing a clip from the beginning.
    pub fn play(&mut self, clip_index: usize) {
        if clip_index < self.clips.len() {
            self.current_clip = Some(clip_index);
            self.time = 0.0;
            self.playing = true;
            self.blend_from_clip = None;
            self.blend_from_time = 0.0;
            self.blend_factor = 1.0;
        }
    }

    /// Stops playback and resets to the beginning.
    pub fn stop(&mut self) {
        self.playing = false;
        self.time = 0.0;
        self.blend_from_clip = None;
        self.blend_from_time = 0.0;
        self.blend_factor = 1.0;
    }

    /// Pauses playback at the current time.
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Resumes playback from the current time.
    pub fn resume(&mut self) {
        self.playing = true;
    }

    /// Returns the currently playing clip, if any.
    pub fn get_current_clip(&self) -> Option<&AnimationClip> {
        self.current_clip.and_then(|index| self.clips.get(index))
    }

    /// Cross-fades from the current clip to another over the specified duration.
    pub fn blend_to(&mut self, clip_index: usize, duration: f32) {
        if clip_index >= self.clips.len() {
            return;
        }
        if self.current_clip == Some(clip_index) {
            return;
        }

        self.blend_from_clip = self.current_clip;
        self.blend_from_time = self.time;
        self.current_clip = Some(clip_index);
        self.time = 0.0;
        self.blend_factor = 0.0;
        self.blend_duration = duration;
        self.playing = true;
    }

    /// Returns `true` if currently cross-fading between clips.
    pub fn is_blending(&self) -> bool {
        self.blend_from_clip.is_some() && self.blend_factor < 1.0
    }

    /// Adds a clip and returns its index.
    pub fn add_clip(&mut self, clip: AnimationClip) -> usize {
        let index = self.clips.len();
        self.clips.push(clip);
        index
    }

    /// Adds multiple clips.
    pub fn add_clips(&mut self, clips: impl IntoIterator<Item = AnimationClip>) {
        self.clips.extend(clips);
    }

    /// Resolves an animation channel's target to an entity.
    pub fn resolve_target_entity(&self, channel: &AnimationChannel) -> Option<Entity> {
        if let Some(ref bone_name) = channel.target_bone_name
            && let Some(&entity) = self.bone_name_to_entity.get(bone_name)
        {
            return Some(entity);
        }
        self.node_index_to_entity.get(&channel.target_node).copied()
    }

    /// Advances playback time and handles looping/blending.
    pub fn update(&mut self, delta_time: f32) {
        if !self.playing {
            return;
        }

        let clip_duration = if let Some(index) = self.current_clip {
            if let Some(clip) = self.clips.get(index) {
                clip.duration
            } else {
                return;
            }
        } else {
            return;
        };

        self.time += delta_time * self.speed;

        let mode = effective_loop_mode(self.loop_mode, self.looping);
        let (wrapped, still_playing) = wrap_time(self.time, clip_duration, mode);
        self.time = wrapped;
        if !still_playing {
            self.playing = false;
        }

        if self.blend_from_clip.is_some() {
            self.blend_factor += delta_time / self.blend_duration.max(0.001);
            if self.blend_factor >= 1.0 {
                self.blend_factor = 1.0;
                self.blend_from_clip = None;
                self.blend_from_time = 0.0;
            } else if let Some(from_index) = self.blend_from_clip
                && let Some(from_clip) = self.clips.get(from_index)
            {
                self.blend_from_time += delta_time * self.speed;
                if self.blend_from_time >= from_clip.duration && self.looping {
                    self.blend_from_time %= from_clip.duration;
                }
            }
        }

        for layer_index in 0..self.layers.len() {
            let layer = &self.layers[layer_index];
            if !layer.playing {
                continue;
            }
            let Some(duration) = self
                .clips
                .get(layer.clip_index)
                .map(|clip| clip.duration)
                .filter(|duration| *duration > 0.0)
            else {
                continue;
            };
            let layer = &mut self.layers[layer_index];
            layer.time += delta_time * layer.speed;
            if layer.time >= duration {
                if layer.looping {
                    layer.time %= duration;
                } else {
                    layer.time = duration;
                    layer.playing = false;
                }
            }
        }
    }
}

/// Sampled animation value at a point in time.
#[derive(Debug, Clone)]
pub enum AnimationValue {
    /// Translation or scale value.
    Vec3(Vec3),
    /// Rotation value.
    Quat(Quat),
    /// Morph target weights.
    Weights(Vec<f32>),
}

/// Interpolates between two Vec3 values.
pub fn interpolate_vec3(a: &Vec3, b: &Vec3, t: f32, interpolation: AnimationInterpolation) -> Vec3 {
    match interpolation {
        AnimationInterpolation::Linear => *a + (*b - *a) * t,
        AnimationInterpolation::Step => *a,
        AnimationInterpolation::CubicSpline => *a + (*b - *a) * t,
    }
}

/// Interpolates between two weight arrays.
pub fn interpolate_weights(
    a: &[f32],
    b: &[f32],
    t: f32,
    interpolation: AnimationInterpolation,
) -> Vec<f32> {
    match interpolation {
        AnimationInterpolation::Linear => a
            .iter()
            .zip(b.iter())
            .map(|(a_val, b_val)| a_val + (b_val - a_val) * t)
            .collect(),
        AnimationInterpolation::Step => a.to_vec(),
        AnimationInterpolation::CubicSpline => a
            .iter()
            .zip(b.iter())
            .map(|(a_val, b_val)| a_val + (b_val - a_val) * t)
            .collect(),
    }
}

fn cubic_spline_weights(
    p0: &[f32],
    m0: &[f32],
    p1: &[f32],
    m1: &[f32],
    t: f32,
    dt: f32,
) -> Vec<f32> {
    let t2 = t * t;
    let t3 = t2 * t;

    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + t;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;

    p0.iter()
        .zip(m0.iter())
        .zip(p1.iter())
        .zip(m1.iter())
        .map(|(((p0_val, m0_val), p1_val), m1_val)| {
            p0_val * h00 + m0_val * (h10 * dt) + p1_val * h01 + m1_val * (h11 * dt)
        })
        .collect()
}

fn cubic_spline_vec3(p0: &Vec3, m0: &Vec3, p1: &Vec3, m1: &Vec3, t: f32, dt: f32) -> Vec3 {
    let t2 = t * t;
    let t3 = t2 * t;

    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + t;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;

    *p0 * h00 + *m0 * (h10 * dt) + *p1 * h01 + *m1 * (h11 * dt)
}

fn cubic_spline_quat(p0: &Quat, m0: &Quat, p1: &Quat, m1: &Quat, t: f32, dt: f32) -> Quat {
    let t2 = t * t;
    let t3 = t2 * t;

    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + t;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;

    let w = p0.w * h00 + m0.w * (h10 * dt) + p1.w * h01 + m1.w * (h11 * dt);
    let i = p0.i * h00 + m0.i * (h10 * dt) + p1.i * h01 + m1.i * (h11 * dt);
    let j = p0.j * h00 + m0.j * (h10 * dt) + p1.j * h01 + m1.j * (h11 * dt);
    let k = p0.k * h00 + m0.k * (h10 * dt) + p1.k * h01 + m1.k * (h11 * dt);

    Quat::new(w, i, j, k).normalize()
}

/// Interpolates between two quaternions using slerp.
pub fn interpolate_quat(a: &Quat, b: &Quat, t: f32, interpolation: AnimationInterpolation) -> Quat {
    match interpolation {
        AnimationInterpolation::Linear => {
            let a_norm = a.normalize();
            let b_norm = b.normalize();

            let dot = a_norm.dot(&b_norm);
            if dot < 0.0 {
                nalgebra_glm::quat_slerp(&a_norm, &-b_norm, t).normalize()
            } else {
                nalgebra_glm::quat_slerp(&a_norm, &b_norm, t).normalize()
            }
        }
        AnimationInterpolation::Step => a.normalize(),
        AnimationInterpolation::CubicSpline => {
            let a_norm = a.normalize();
            let b_norm = b.normalize();

            let dot = a_norm.dot(&b_norm);
            if dot < 0.0 {
                nalgebra_glm::quat_slerp(&a_norm, &-b_norm, t).normalize()
            } else {
                nalgebra_glm::quat_slerp(&a_norm, &b_norm, t).normalize()
            }
        }
    }
}

/// Samples an animation channel at the given time, returning the interpolated value.
pub fn sample_animation_channel(channel: &AnimationChannel, time: f32) -> Option<AnimationValue> {
    let sampler = &channel.sampler;

    if sampler.input.is_empty() {
        return None;
    }

    if time < sampler.input[0] {
        return match &sampler.output {
            AnimationSamplerOutput::Vec3(values) => {
                values.first().map(|v| AnimationValue::Vec3(*v))
            }
            AnimationSamplerOutput::Quat(values) => {
                values.first().map(|q| AnimationValue::Quat(*q))
            }
            AnimationSamplerOutput::Weights(values) => {
                values.first().map(|w| AnimationValue::Weights(w.clone()))
            }
            AnimationSamplerOutput::CubicSplineVec3 { values, .. } => {
                values.first().map(|v| AnimationValue::Vec3(*v))
            }
            AnimationSamplerOutput::CubicSplineQuat { values, .. } => {
                values.first().map(|q| AnimationValue::Quat(*q))
            }
            AnimationSamplerOutput::CubicSplineWeights { values, .. } => {
                values.first().map(|w| AnimationValue::Weights(w.clone()))
            }
        };
    }

    if time >= sampler.input[sampler.input.len() - 1] {
        let last_index = sampler.input.len() - 1;
        return match &sampler.output {
            AnimationSamplerOutput::Vec3(values) => {
                values.get(last_index).map(|v| AnimationValue::Vec3(*v))
            }
            AnimationSamplerOutput::Quat(values) => {
                values.get(last_index).map(|q| AnimationValue::Quat(*q))
            }
            AnimationSamplerOutput::Weights(values) => values
                .get(last_index)
                .map(|w| AnimationValue::Weights(w.clone())),
            AnimationSamplerOutput::CubicSplineVec3 { values, .. } => {
                values.get(last_index).map(|v| AnimationValue::Vec3(*v))
            }
            AnimationSamplerOutput::CubicSplineQuat { values, .. } => {
                values.get(last_index).map(|q| AnimationValue::Quat(*q))
            }
            AnimationSamplerOutput::CubicSplineWeights { values, .. } => values
                .get(last_index)
                .map(|w| AnimationValue::Weights(w.clone())),
        };
    }

    let key_index = sampler
        .input
        .partition_point(|&keyframe_time| keyframe_time <= time)
        - 1;

    let next_key_index = key_index + 1;

    if key_index == next_key_index {
        return match &sampler.output {
            AnimationSamplerOutput::Vec3(values) => {
                values.get(key_index).map(|v| AnimationValue::Vec3(*v))
            }
            AnimationSamplerOutput::Quat(values) => {
                values.get(key_index).map(|q| AnimationValue::Quat(*q))
            }
            AnimationSamplerOutput::Weights(values) => values
                .get(key_index)
                .map(|w| AnimationValue::Weights(w.clone())),
            AnimationSamplerOutput::CubicSplineVec3 { values, .. } => {
                values.get(key_index).map(|v| AnimationValue::Vec3(*v))
            }
            AnimationSamplerOutput::CubicSplineQuat { values, .. } => {
                values.get(key_index).map(|q| AnimationValue::Quat(*q))
            }
            AnimationSamplerOutput::CubicSplineWeights { values, .. } => values
                .get(key_index)
                .map(|w| AnimationValue::Weights(w.clone())),
        };
    }

    let key_time = sampler.input[key_index];
    let next_key_time = sampler.input[next_key_index];
    let dt = next_key_time - key_time;
    let t = (time - key_time) / dt;

    match &sampler.output {
        AnimationSamplerOutput::Vec3(values) => {
            if let (Some(a), Some(b)) = (values.get(key_index), values.get(next_key_index)) {
                Some(AnimationValue::Vec3(interpolate_vec3(
                    a,
                    b,
                    t,
                    sampler.interpolation,
                )))
            } else {
                None
            }
        }
        AnimationSamplerOutput::Quat(values) => {
            if let (Some(a), Some(b)) = (values.get(key_index), values.get(next_key_index)) {
                Some(AnimationValue::Quat(interpolate_quat(
                    a,
                    b,
                    t,
                    sampler.interpolation,
                )))
            } else {
                None
            }
        }
        AnimationSamplerOutput::CubicSplineVec3 {
            values,
            in_tangents,
            out_tangents,
        } => {
            if let (Some(p0), Some(p1), Some(m0), Some(m1)) = (
                values.get(key_index),
                values.get(next_key_index),
                out_tangents.get(key_index),
                in_tangents.get(next_key_index),
            ) {
                Some(AnimationValue::Vec3(cubic_spline_vec3(
                    p0, m0, p1, m1, t, dt,
                )))
            } else {
                None
            }
        }
        AnimationSamplerOutput::CubicSplineQuat {
            values,
            in_tangents,
            out_tangents,
        } => {
            if let (Some(p0), Some(p1), Some(m0), Some(m1)) = (
                values.get(key_index),
                values.get(next_key_index),
                out_tangents.get(key_index),
                in_tangents.get(next_key_index),
            ) {
                Some(AnimationValue::Quat(cubic_spline_quat(
                    p0, m0, p1, m1, t, dt,
                )))
            } else {
                None
            }
        }
        AnimationSamplerOutput::Weights(values) => {
            if let (Some(a), Some(b)) = (values.get(key_index), values.get(next_key_index)) {
                Some(AnimationValue::Weights(interpolate_weights(
                    a,
                    b,
                    t,
                    sampler.interpolation,
                )))
            } else {
                None
            }
        }
        AnimationSamplerOutput::CubicSplineWeights {
            values,
            in_tangents,
            out_tangents,
        } => {
            if let (Some(p0), Some(p1), Some(m0), Some(m1)) = (
                values.get(key_index),
                values.get(next_key_index),
                out_tangents.get(key_index),
                in_tangents.get(next_key_index),
            ) {
                Some(AnimationValue::Weights(cubic_spline_weights(
                    p0, m0, p1, m1, t, dt,
                )))
            } else {
                None
            }
        }
    }
}
