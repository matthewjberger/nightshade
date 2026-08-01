pub mod humanoid;

use nalgebra_glm::{Quat, Vec3};

use crate::contract::{
    AnimationChannel, AnimationInterpolation, AnimationSamplerOutput, AnimationValue, LoopMode,
};

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

pub fn effective_loop_mode(mode: LoopMode, looping: bool) -> LoopMode {
    match mode {
        LoopMode::Loop if !looping => LoopMode::Once,
        other => other,
    }
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
