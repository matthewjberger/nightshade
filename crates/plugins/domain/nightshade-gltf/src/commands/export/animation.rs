use std::collections::HashMap;

use nightshade_animation::contract::{
    AnimationClip, AnimationInterpolation, AnimationProperty, AnimationSamplerOutput,
};
use serde_json::{Value, json};

use super::buffer::BufferBuilder;

pub fn animation_json(
    clip: &AnimationClip,
    nodes: &HashMap<String, usize>,
    fallback: &[usize],
    builder: &mut BufferBuilder,
) -> Option<Value> {
    let mut channels = Vec::new();
    let mut samplers = Vec::new();

    for channel in &clip.channels {
        let Some(target) = resolve(
            channel.target_bone_name.as_deref(),
            channel.target_node,
            nodes,
            fallback,
        ) else {
            continue;
        };
        let Some(output) = output_accessor(&channel.sampler.output, builder) else {
            continue;
        };
        let input = builder.floats(&channel.sampler.input, "SCALAR", None);
        samplers.push(json!({
            "input": input,
            "output": output,
            "interpolation": match channel.sampler.interpolation {
                AnimationInterpolation::Linear => "LINEAR",
                AnimationInterpolation::Step => "STEP",
                AnimationInterpolation::CubicSpline => "CUBICSPLINE",
            },
        }));
        channels.push(json!({
            "sampler": samplers.len() - 1,
            "target": {
                "node": target,
                "path": match channel.target_property {
                    AnimationProperty::Translation => "translation",
                    AnimationProperty::Rotation => "rotation",
                    AnimationProperty::Scale => "scale",
                    AnimationProperty::MorphWeights => "weights",
                },
            },
        }));
    }

    if channels.is_empty() {
        return None;
    }

    Some(json!({
        "name": clip.name,
        "channels": channels,
        "samplers": samplers,
    }))
}

fn resolve(
    bone: Option<&str>,
    node: usize,
    nodes: &HashMap<String, usize>,
    fallback: &[usize],
) -> Option<usize> {
    if let Some(bone) = bone {
        if let Some(index) = nodes.get(bone) {
            return Some(*index);
        }
        let tail = bone.rsplit(':').next().unwrap_or(bone);
        return nodes
            .iter()
            .find(|(name, _)| name.rsplit(':').next().unwrap_or(name) == tail)
            .map(|(_, index)| *index);
    }
    fallback.get(node).copied()
}

fn output_accessor(output: &AnimationSamplerOutput, builder: &mut BufferBuilder) -> Option<usize> {
    match output {
        AnimationSamplerOutput::Vec3(values) => {
            let flat: Vec<f32> = values
                .iter()
                .flat_map(|value| [value.x, value.y, value.z])
                .collect();
            Some(builder.floats(&flat, "VEC3", None))
        }
        AnimationSamplerOutput::Quat(values) => {
            let flat: Vec<f32> = values
                .iter()
                .flat_map(|value| [value.i, value.j, value.k, value.w])
                .collect();
            Some(builder.floats(&flat, "VEC4", None))
        }
        AnimationSamplerOutput::Weights(values) => {
            let flat: Vec<f32> = values.iter().flatten().copied().collect();
            Some(builder.floats(&flat, "SCALAR", None))
        }
        AnimationSamplerOutput::CubicSplineVec3 {
            values,
            in_tangents,
            out_tangents,
        } => {
            let mut flat = Vec::with_capacity(values.len() * 9);
            for index in 0..values.len() {
                for source in [&in_tangents[index], &values[index], &out_tangents[index]] {
                    flat.extend_from_slice(&[source.x, source.y, source.z]);
                }
            }
            Some(builder.floats(&flat, "VEC3", None))
        }
        AnimationSamplerOutput::CubicSplineQuat {
            values,
            in_tangents,
            out_tangents,
        } => {
            let mut flat = Vec::with_capacity(values.len() * 12);
            for index in 0..values.len() {
                for source in [&in_tangents[index], &values[index], &out_tangents[index]] {
                    flat.extend_from_slice(&[source.i, source.j, source.k, source.w]);
                }
            }
            Some(builder.floats(&flat, "VEC4", None))
        }
        AnimationSamplerOutput::CubicSplineWeights {
            values,
            in_tangents,
            out_tangents,
        } => {
            let mut flat = Vec::new();
            for index in 0..values.len() {
                for source in [&in_tangents[index], &values[index], &out_tangents[index]] {
                    flat.extend_from_slice(source);
                }
            }
            Some(builder.floats(&flat, "SCALAR", None))
        }
    }
}
