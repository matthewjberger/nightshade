pub mod blend_tree;
pub mod graph;
pub mod ik;

use crate::contract::blend_tree::BlendTree;
use crate::contract::graph::AnimationGraph;
use crate::contract::ik::{AimConstraint, IkChain, IkChainMulti};
use nalgebra_glm::{Quat, Vec3};
use nightshade_ecs::Entity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveClip {
    pub clip_index: usize,
    pub time: f32,
    pub weight: f32,
    pub target_weight: f32,
    pub fade_rate: f32,
    pub speed: f32,
    pub loop_mode: LoopMode,
    pub dominant: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AnimationClip {
    pub name: String,
    pub duration: f32,
    pub channels: Vec<AnimationChannel>,
    pub events: Vec<AnimationEvent>,
    pub source: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationEvent {
    pub time: f32,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LayerBlendMode {
    #[default]
    Override,
    Additive,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationLayer {
    pub clip_index: usize,
    pub weight: f32,
    pub time: f32,
    pub speed: f32,
    pub playing: bool,
    pub looping: bool,
    #[serde(default)]
    pub mask: Vec<String>,
    #[serde(default)]
    pub blend_mode: LayerBlendMode,
    #[serde(default)]
    pub reference_clip: Option<usize>,
}

impl AnimationLayer {
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

    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self
    }

    pub fn with_mask(mut self, mask: Vec<String>) -> Self {
        self.mask = mask;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationChannel {
    pub target_node: usize,
    pub target_bone_name: Option<String>,
    pub target_property: AnimationProperty,
    pub sampler: AnimationSampler,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AnimationProperty {
    Translation,
    Rotation,
    Scale,
    MorphWeights,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationSampler {
    pub input: Vec<f32>,
    pub output: AnimationSamplerOutput,
    pub interpolation: AnimationInterpolation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnimationSamplerOutput {
    Vec3(Vec<Vec3>),
    Quat(Vec<Quat>),
    Weights(Vec<Vec<f32>>),
    CubicSplineVec3 {
        values: Vec<Vec3>,
        in_tangents: Vec<Vec3>,
        out_tangents: Vec<Vec3>,
    },
    CubicSplineQuat {
        values: Vec<Quat>,
        in_tangents: Vec<Quat>,
        out_tangents: Vec<Quat>,
    },
    CubicSplineWeights {
        values: Vec<Vec<f32>>,
        in_tangents: Vec<Vec<f32>>,
        out_tangents: Vec<Vec<f32>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationInterpolation {
    Linear,
    Step,
    CubicSpline,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpringChain {
    pub bones: Vec<String>,
    pub stiffness: f32,
    pub damping: f32,
    pub gravity: Vec3,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LoopMode {
    #[default]
    Loop,
    Once,
    Clamp,
    PingPong,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationPlayer {
    pub clips: Vec<AnimationClip>,
    pub current_clip: Option<usize>,
    pub time: f32,
    pub speed: f32,
    pub looping: bool,
    #[serde(default)]
    pub loop_mode: LoopMode,
    pub playing: bool,
    pub play_all: bool,
    pub node_index_to_entity: HashMap<usize, Entity>,
    pub bone_name_to_entity: HashMap<String, Entity>,
    pub blend_from_clip: Option<usize>,
    pub blend_from_time: f32,
    pub blend_factor: f32,
    pub blend_duration: f32,
    #[serde(default)]
    pub layers: Vec<AnimationLayer>,
    #[serde(default, skip)]
    pub active_clips: Vec<ActiveClip>,
    #[serde(default)]
    pub parameters: HashMap<String, f32>,
    #[serde(default)]
    pub blend_tree: Option<BlendTree>,
    #[serde(default)]
    pub graph: Option<AnimationGraph>,
    #[serde(default)]
    pub ik_chains: Vec<IkChain>,
    #[serde(default)]
    pub ik_chains_multi: Vec<IkChainMulti>,
    #[serde(default)]
    pub aim_constraints: Vec<AimConstraint>,
    #[serde(default)]
    pub spring_chains: Vec<SpringChain>,
    #[serde(default)]
    pub root_motion: bool,
    #[serde(default)]
    pub root_motion_bone: Option<String>,
    #[serde(default)]
    pub root_motion_planar: bool,
    #[serde(default, skip)]
    pub root_motion_previous: Option<Vec3>,
    #[serde(default, skip)]
    pub root_motion_previous_time: f32,
    #[serde(default)]
    pub resolve_by_bone_name: bool,
    #[serde(default)]
    pub synchronize_blend: bool,
    #[serde(default, skip)]
    pub blend_phase: f32,
    #[serde(default, skip)]
    pub routing: BoneRouting,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BoneRouting {
    pub bones: Vec<Entity>,
    pub channel_bone: Vec<Vec<u32>>,
    pub signature: (usize, usize, usize),
}

impl BoneRouting {
    pub const NONE: u32 = u32::MAX;
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
            root_motion: false,
            root_motion_bone: None,
            root_motion_planar: false,
            root_motion_previous: None,
            root_motion_previous_time: 0.0,
            resolve_by_bone_name: false,
            routing: BoneRouting::default(),
            synchronize_blend: false,
            blend_phase: 0.0,
        }
    }
}

impl AnimationPlayer {
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

    pub fn stop(&mut self) {
        self.playing = false;
        self.time = 0.0;
        self.blend_from_clip = None;
        self.blend_from_time = 0.0;
        self.blend_factor = 1.0;
    }

    pub fn pause(&mut self) {
        self.playing = false;
    }

    pub fn resume(&mut self) {
        self.playing = true;
    }

    pub fn get_current_clip(&self) -> Option<&AnimationClip> {
        self.current_clip.and_then(|index| self.clips.get(index))
    }

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

    pub fn blend_to_in_phase(&mut self, clip_index: usize, duration: f32) {
        let Some(target_duration) = self.clips.get(clip_index).map(|clip| clip.duration) else {
            return;
        };
        if self.current_clip == Some(clip_index) {
            return;
        }

        let phase = self
            .current_clip
            .and_then(|index| self.clips.get(index))
            .filter(|clip| clip.duration > 0.0)
            .map(|clip| self.time / clip.duration)
            .unwrap_or(0.0);

        self.blend_to(clip_index, duration);
        self.time = phase.fract() * target_duration;
    }

    pub fn is_blending(&self) -> bool {
        self.blend_from_clip.is_some() && self.blend_factor < 1.0
    }

    pub fn add_clip(&mut self, clip: AnimationClip) -> usize {
        let index = self.clips.len();
        self.clips.push(clip);
        index
    }

    pub fn add_clips(&mut self, clips: impl IntoIterator<Item = AnimationClip>) {
        self.clips.extend(clips);
    }

    pub fn resolve_target_entity(&self, channel: &AnimationChannel) -> Option<Entity> {
        if let Some(ref bone_name) = channel.target_bone_name
            && let Some(&entity) = self.bone_name_to_entity.get(bone_name)
        {
            return Some(entity);
        }
        if self.resolve_by_bone_name {
            return None;
        }
        self.node_index_to_entity.get(&channel.target_node).copied()
    }
}

#[derive(Debug, Clone)]
pub enum AnimationValue {
    Vec3(Vec3),
    Quat(Quat),
    Weights(Vec<f32>),
}

#[derive(Debug, Clone, Copy)]
pub struct AnimationFinished {
    pub entity: Entity,
}

#[derive(Debug, Clone)]
pub struct AnimationMarkerReached {
    pub entity: Entity,
    pub name: String,
}

pub fn find_clip(available: &[AnimationClip], wanted: &str) -> Option<AnimationClip> {
    available
        .iter()
        .find(|clip| clip.name.eq_ignore_ascii_case(wanted))
        .cloned()
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ClipSet {
    pub name: String,
    pub clips: Vec<AnimationClip>,
}

impl ClipSet {
    pub fn bones(&self) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        for clip in &self.clips {
            for channel in &clip.channels {
                if let Some(bone) = &channel.target_bone_name
                    && !names.iter().any(|held| held == bone)
                {
                    names.push(bone.clone());
                }
            }
        }
        names.sort();
        names
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClipSetReport {
    pub clips: usize,
    pub channels_resolved: usize,
    pub channels_dropped: usize,
    pub unresolved_bones: Vec<String>,
}

impl ClipSetReport {
    pub fn is_complete(&self) -> bool {
        self.channels_dropped == 0 && self.unresolved_bones.is_empty()
    }
}
