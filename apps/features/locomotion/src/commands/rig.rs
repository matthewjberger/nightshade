use nalgebra_glm::{Quat, Vec3};
use nightshade_animation::contract::{AnimationPlayer, AnimationProperty, AnimationSamplerOutput};

use crate::queries::clips::index_of;

pub fn drop_bone_offsets(player: &mut AnimationPlayer) {
    for clip in &mut player.clips {
        clip.channels.retain(|channel| {
            if channel.target_property != AnimationProperty::Translation {
                return true;
            }
            channel
                .target_bone_name
                .as_ref()
                .map(|name| {
                    name.rsplit(':')
                        .next()
                        .unwrap_or(name)
                        .eq_ignore_ascii_case("hips")
                })
                .unwrap_or(false)
        });
    }
}

pub fn hold_the_facing(player: &mut AnimationPlayer, clip_name: &str) {
    let Some(clip_index) = index_of(player, clip_name) else {
        return;
    };
    let clip = &mut player.clips[clip_index];
    for channel in &mut clip.channels {
        if channel.target_property != AnimationProperty::Rotation {
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
        untwist(&mut channel.sampler.output);
    }
}

fn untwist(output: &mut AnimationSamplerOutput) {
    let up = Vec3::new(0.0, 0.0, 1.0);
    match output {
        AnimationSamplerOutput::Quat(values) => {
            let Some(first) = values.first().copied() else {
                return;
            };
            for value in values.iter_mut() {
                *value = without_twist(*value, first, up);
            }
        }
        AnimationSamplerOutput::CubicSplineQuat {
            values,
            in_tangents,
            out_tangents,
        } => {
            let Some(first) = values.first().copied() else {
                return;
            };
            for value in values.iter_mut() {
                *value = without_twist(*value, first, up);
            }
            let still = Quat::new(0.0, 0.0, 0.0, 0.0);
            for tangent in in_tangents.iter_mut().chain(out_tangents.iter_mut()) {
                *tangent = still;
            }
        }
        _ => {}
    }
}

fn without_twist(value: Quat, first: Quat, axis: Vec3) -> Quat {
    let delta = value * first.conjugate();
    let vector = Vec3::new(delta.i, delta.j, delta.k);
    let projected = axis * vector.dot(&axis);
    let twist = Quat::new(delta.w, projected.x, projected.y, projected.z);
    if twist.norm() < 1.0e-6 {
        return value;
    }
    let swing = delta * twist.normalize().conjugate();
    (swing * first).normalize()
}
