use nalgebra_glm::{Vec2, Vec3};
use nightshade_animation::contract::blend_tree::{Blend2DChild, BlendTree};
use nightshade_animation::contract::{AnimationPlayer, AnimationProperty, AnimationSamplerOutput};

use crate::data;
use crate::queries::clips::index_of;

pub fn build(player: &mut AnimationPlayer) -> bool {
    let mut children: Vec<Blend2DChild> = Vec::new();

    let place = |name: &str, position: Vec2, children: &mut Vec<Blend2DChild>| -> bool {
        match index_of(player, name) {
            Some(clip_index) => {
                children.push(Blend2DChild {
                    position,
                    motion: BlendTree::Clip { clip_index },
                });
                true
            }
            None => false,
        }
    };

    let mut complete = place(data::IDLE, Vec2::new(0.0, 0.0), &mut children);
    complete &= place(
        data::WALKING,
        Vec2::new(0.0, data::WALK_SPEED),
        &mut children,
    );
    complete &= place(
        data::RUNNING,
        Vec2::new(0.0, data::RUN_SPEED),
        &mut children,
    );
    complete &= place(
        data::LEFT_STRAFE_WALKING,
        Vec2::new(-data::WALK_SPEED, 0.0),
        &mut children,
    );
    complete &= place(
        data::RIGHT_STRAFE_WALKING,
        Vec2::new(data::WALK_SPEED, 0.0),
        &mut children,
    );
    complete &= place(
        data::LEFT_STRAFE,
        Vec2::new(-data::RUN_SPEED, 0.0),
        &mut children,
    );
    complete &= place(
        data::RIGHT_STRAFE,
        Vec2::new(data::RUN_SPEED, 0.0),
        &mut children,
    );

    if !complete {
        return false;
    }

    player.blend_tree = Some(BlendTree::Freeform2D {
        parameter_x: data::LATERAL.to_string(),
        parameter_y: data::FORWARD.to_string(),
        children,
    });
    player.parameters.insert(data::LATERAL.to_string(), 0.0);
    player.parameters.insert(data::FORWARD.to_string(), 0.0);
    player.synchronize_blend = true;
    player.looping = true;
    player.playing = true;
    true
}

pub fn hold_the_root_still(player: &mut AnimationPlayer) {
    let mut shared: Option<Vec3> = None;
    for clip in &mut player.clips {
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
            let anchor = *shared.get_or_insert_with(|| first_position(&channel.sampler.output));
            flatten(&mut channel.sampler.output, anchor);
        }
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

fn flatten(output: &mut AnimationSamplerOutput, anchor: Vec3) {
    match output {
        AnimationSamplerOutput::Vec3(values) => {
            for value in values.iter_mut() {
                value.x = anchor.x;
                value.y = anchor.y;
            }
        }
        AnimationSamplerOutput::CubicSplineVec3 {
            values,
            in_tangents,
            out_tangents,
        } => {
            for value in values.iter_mut() {
                value.x = anchor.x;
                value.y = anchor.y;
            }
            for tangent in in_tangents.iter_mut().chain(out_tangents.iter_mut()) {
                tangent.x = 0.0;
                tangent.y = 0.0;
            }
        }
        _ => {}
    }
}
