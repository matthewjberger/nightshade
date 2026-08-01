use crate::contract::{
    AnimationChannel, AnimationClip, AnimationInterpolation, AnimationSamplerOutput,
};
use nalgebra_glm::{Quat, Vec3};

fn simplify_vec3(times: &[f32], values: &[Vec3], tolerance: f32) -> Vec<usize> {
    let count = values.len();
    if count <= 2 {
        return (0..count).collect();
    }
    let mut kept = vec![0usize];
    let mut anchor = 0usize;
    while anchor < count - 1 {
        let mut best = anchor + 1;
        let mut endpoint = anchor + 2;
        while endpoint < count {
            let mut within = true;
            for middle in (anchor + 1)..endpoint {
                let span = times[endpoint] - times[anchor];
                let ratio = if span.abs() > 1.0e-8 {
                    (times[middle] - times[anchor]) / span
                } else {
                    0.0
                };
                let interpolated = values[anchor] + (values[endpoint] - values[anchor]) * ratio;
                if (interpolated - values[middle]).magnitude() > tolerance {
                    within = false;
                    break;
                }
            }
            if within {
                best = endpoint;
                endpoint += 1;
            } else {
                break;
            }
        }
        kept.push(best);
        anchor = best;
    }
    kept
}

fn simplify_quat(times: &[f32], values: &[Quat], tolerance: f32) -> Vec<usize> {
    let count = values.len();
    if count <= 2 {
        return (0..count).collect();
    }
    let mut kept = vec![0usize];
    let mut anchor = 0usize;
    while anchor < count - 1 {
        let mut best = anchor + 1;
        let mut endpoint = anchor + 2;
        while endpoint < count {
            let mut within = true;
            let start = values[anchor].normalize();
            let finish = values[endpoint].normalize();
            for middle in (anchor + 1)..endpoint {
                let span = times[endpoint] - times[anchor];
                let ratio = if span.abs() > 1.0e-8 {
                    (times[middle] - times[anchor]) / span
                } else {
                    0.0
                };
                let interpolated = nalgebra_glm::quat_slerp(&start, &finish, ratio).normalize();
                let dot = interpolated
                    .dot(&values[middle].normalize())
                    .abs()
                    .clamp(0.0, 1.0);
                if 2.0 * dot.acos() > tolerance {
                    within = false;
                    break;
                }
            }
            if within {
                best = endpoint;
                endpoint += 1;
            } else {
                break;
            }
        }
        kept.push(best);
        anchor = best;
    }
    kept
}

fn gather<T: Copy>(source: &[T], indices: &[usize]) -> Vec<T> {
    indices.iter().map(|&index| source[index]).collect()
}

fn compress_channel(channel: &AnimationChannel, tolerance: f32) -> AnimationChannel {
    if channel.sampler.interpolation != AnimationInterpolation::Linear {
        return channel.clone();
    }
    let (input, output) = match &channel.sampler.output {
        AnimationSamplerOutput::Vec3(values) if values.len() == channel.sampler.input.len() => {
            let kept = simplify_vec3(&channel.sampler.input, values, tolerance);
            (
                gather(&channel.sampler.input, &kept),
                AnimationSamplerOutput::Vec3(gather(values, &kept)),
            )
        }
        AnimationSamplerOutput::Quat(values) if values.len() == channel.sampler.input.len() => {
            let kept = simplify_quat(&channel.sampler.input, values, tolerance);
            (
                gather(&channel.sampler.input, &kept),
                AnimationSamplerOutput::Quat(gather(values, &kept)),
            )
        }
        _ => return channel.clone(),
    };
    let mut compressed = channel.clone();
    compressed.sampler.input = input;
    compressed.sampler.output = output;
    compressed
}

pub fn clip_keyframe_count(clip: &AnimationClip) -> usize {
    clip.channels
        .iter()
        .map(|channel| channel.sampler.input.len())
        .sum()
}

pub fn compress_clip(clip: &AnimationClip, tolerance: f32) -> AnimationClip {
    let channels = clip
        .channels
        .iter()
        .map(|channel| compress_channel(channel, tolerance))
        .collect();
    AnimationClip {
        name: clip.name.clone(),
        duration: clip.duration,
        channels,
        events: clip.events.clone(),
        source: 0,
    }
}
