use nalgebra_glm::Vec3;
use nightshade_animation::contract::{AnimationPlayer, AnimationProperty, AnimationSamplerOutput};

use crate::data;
use crate::queries::clips::index_of;

pub fn raise_the_jump(player: &mut AnimationPlayer) {
    let Some(clip_index) = index_of(player, data::JUMP) else {
        return;
    };
    let clip = &mut player.clips[clip_index];
    for channel in &mut clip.channels {
        if channel.target_property != AnimationProperty::Translation {
            continue;
        }
        let Some(name) = channel.target_bone_name.as_ref() else {
            continue;
        };
        if !name
            .rsplit(':')
            .next()
            .unwrap_or(name)
            .eq_ignore_ascii_case("hips")
        {
            continue;
        }
        let ground = first_position(&channel.sampler.output).z;
        scale(&mut channel.sampler.output, ground, data::JUMP_HEIGHT_SCALE);
    }
}

fn first_position(output: &AnimationSamplerOutput) -> Vec3 {
    let first = match output {
        AnimationSamplerOutput::Vec3(values) => values.first().copied(),
        AnimationSamplerOutput::CubicSplineVec3 { values, .. } => values.first().copied(),
        _ => None,
    };
    first.unwrap_or_else(Vec3::zeros)
}

fn scale(output: &mut AnimationSamplerOutput, ground: f32, factor: f32) {
    match output {
        AnimationSamplerOutput::Vec3(values) => {
            for value in values.iter_mut() {
                value.z = ground + (value.z - ground) * factor;
            }
        }
        AnimationSamplerOutput::CubicSplineVec3 {
            values,
            in_tangents,
            out_tangents,
        } => {
            for value in values.iter_mut() {
                value.z = ground + (value.z - ground) * factor;
            }
            for tangent in in_tangents.iter_mut().chain(out_tangents.iter_mut()) {
                tangent.z *= factor;
            }
        }
        _ => {}
    }
}
