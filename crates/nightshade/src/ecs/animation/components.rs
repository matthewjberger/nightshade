use freecs::Entity;
use nalgebra_glm::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationClip {
    pub name: String,
    pub duration: f32,
    pub channels: Vec<AnimationChannel>,
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
pub struct AnimationPlayer {
    pub clips: Vec<AnimationClip>,
    pub current_clip: Option<usize>,
    pub time: f32,
    pub speed: f32,
    pub looping: bool,
    pub playing: bool,
    pub play_all: bool,
    pub node_index_to_entity: HashMap<usize, Entity>,
    pub bone_name_to_entity: HashMap<String, Entity>,
    pub blend_from_clip: Option<usize>,
    pub blend_from_time: f32,
    pub blend_factor: f32,
    pub blend_duration: f32,
}

impl Default for AnimationPlayer {
    fn default() -> Self {
        Self {
            clips: Vec::new(),
            current_clip: None,
            time: 0.0,
            speed: 1.0,
            looping: true,
            playing: false,
            play_all: false,
            node_index_to_entity: HashMap::new(),
            bone_name_to_entity: HashMap::new(),
            blend_from_clip: None,
            blend_from_time: 0.0,
            blend_factor: 1.0,
            blend_duration: 0.0,
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
        self.node_index_to_entity.get(&channel.target_node).copied()
    }

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

        if self.time >= clip_duration {
            if self.looping {
                self.time %= clip_duration;
            } else {
                self.time = clip_duration;
                self.playing = false;
            }
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
    }
}

#[derive(Debug, Clone)]
pub enum AnimationValue {
    Vec3(Vec3),
    Quat(Quat),
    Weights(Vec<f32>),
}

pub fn interpolate_vec3(a: &Vec3, b: &Vec3, t: f32, interpolation: AnimationInterpolation) -> Vec3 {
    match interpolation {
        AnimationInterpolation::Linear => *a + (*b - *a) * t,
        AnimationInterpolation::Step => *a,
        AnimationInterpolation::CubicSpline => *a + (*b - *a) * t,
    }
}

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

    let mut key_index = sampler.input.len() - 2;
    for index in 0..sampler.input.len() - 1 {
        if sampler.input[index + 1] > time {
            key_index = index;
            break;
        }
    }

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
