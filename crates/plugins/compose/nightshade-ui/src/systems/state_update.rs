use crate::components::UiAnimationPhase;
use crate::data::state::{UiBase, UiDisabled, UiFocused, UiHover, UiPressed, UiStateTrait};

use crate::prelude::*;
pub fn ui_layout_state_update_system(
    mut build: crate::commands::builder::UiBuild,
    time: nightshade::ecs::system_param::Res<nightshade_platform::contract::time::Time>,
    mut progress_bar: nightshade::ecs::system_param::ResMut<crate::resources::TopProgressBar>,
    storage: &mut nightshade::ecs::Storage,
) {
    if !build.runtime().visible {
        return;
    }

    let delta_time = time.delta_time;
    let reduced_motion = build.accessibility().reduced_motion;

    let entities: Vec<nightshade::ecs::Entity> = storage
        .query_entities::<(crate::components::UiStateWeights,)>()
        .collect();

    let mut any_weight_changed = false;

    for entity in entities {
        let (hover_target, pressed_target, focused_target, disabled_target) = {
            match storage.get::<crate::components::UiNodeInteraction>(entity) {
                Some(interaction) => (
                    if interaction.hovered { 1.0 } else { 0.0 },
                    if interaction.pressed { 1.0 } else { 0.0 },
                    if interaction.focused { 1.0 } else { 0.0 },
                    if interaction.disabled { 1.0 } else { 0.0 },
                ),
                None => (0.0, 0.0, 0.0, 0.0),
            }
        };

        if let Some(state_weights) = storage.get_mut::<crate::components::UiStateWeights>(entity) {
            let targets = [
                (UiHover::INDEX, hover_target),
                (UiPressed::INDEX, pressed_target),
                (UiFocused::INDEX, focused_target),
                (UiDisabled::INDEX, disabled_target),
            ];

            for (index, target) in targets {
                let previous = state_weights.weights[index];

                if (state_weights.targets[index] - target).abs() > f32::EPSILON {
                    state_weights.targets[index] = target;
                    state_weights.start_weights[index] = previous;
                    state_weights.progress[index] = 0.0;
                }

                if reduced_motion {
                    state_weights.weights[index] = state_weights.targets[index];
                    state_weights.progress[index] = 1.0;
                    state_weights.velocity[index] = 0.0;
                } else {
                    let transition = state_weights.transitions[index].unwrap_or_default();

                    if let Some(spring) = transition.spring {
                        let mass = spring.mass.max(0.0001);
                        let displacement = state_weights.weights[index] - target;
                        let acceleration = (-spring.stiffness * displacement
                            - spring.damping * state_weights.velocity[index])
                            / mass;
                        state_weights.velocity[index] += acceleration * delta_time;
                        state_weights.weights[index] += state_weights.velocity[index] * delta_time;

                        let speed_settled = state_weights.velocity[index].abs() < 0.0005;
                        let position_settled = displacement.abs() < 0.0005;
                        if speed_settled && position_settled {
                            state_weights.weights[index] = target;
                            state_weights.velocity[index] = 0.0;
                            state_weights.progress[index] = 1.0;
                        } else {
                            state_weights.progress[index] = 0.0;
                        }

                        state_weights.weights[index] =
                            state_weights.weights[index].clamp(-0.5, 1.5);
                    } else if state_weights.progress[index] < 1.0 {
                        let speed = if target > state_weights.start_weights[index] {
                            transition.enter_speed
                        } else {
                            transition.exit_speed
                        };

                        state_weights.progress[index] =
                            (state_weights.progress[index] + speed * delta_time).min(1.0);

                        let eased = transition.easing.evaluate(state_weights.progress[index]);
                        let start = state_weights.start_weights[index];
                        state_weights.weights[index] =
                            (start + (target - start) * eased).clamp(0.0, 1.0);
                        state_weights.velocity[index] = 0.0;
                    }
                }

                if (state_weights.weights[index] - previous).abs() > f32::EPSILON {
                    any_weight_changed = true;
                }
            }

            let non_base_sum: f32 = state_weights.weights[1..]
                .iter()
                .map(|weight| weight.clamp(0.0, 1.0))
                .sum();
            state_weights.weights[UiBase::INDEX] = (1.0 - non_base_sum).clamp(0.0, 1.0);
        }
    }

    if any_weight_changed {
        ui_mark_layout_dirty(build.dirty());
    }

    let anim_entities: Vec<nightshade::ecs::Entity> = storage
        .query_entities::<(crate::components::UiLayoutNode,)>()
        .collect();

    for entity in anim_entities {
        let should_hide = if let Some(node) =
            storage.get_mut::<crate::components::UiLayoutNode>(entity)
            && let Some(animation) = &mut node.animation
        {
            match animation.phase {
                UiAnimationPhase::IntroPlaying => {
                    animation.progress += delta_time / animation.duration;
                    if animation.progress >= 1.0 {
                        animation.progress = 1.0;
                        animation.phase = UiAnimationPhase::Idle;
                    }
                    ui_mark_layout_dirty(build.dirty());
                    false
                }
                UiAnimationPhase::OutroPlaying => {
                    animation.progress += delta_time / animation.duration;
                    if animation.progress >= 1.0 {
                        animation.progress = 1.0;
                        animation.phase = UiAnimationPhase::OutroComplete;
                        ui_mark_layout_dirty(build.dirty());
                        true
                    } else {
                        ui_mark_layout_dirty(build.dirty());
                        false
                    }
                }
                _ => false,
            }
        } else {
            false
        };

        if should_hide
            && let Some(node) = storage.get_mut::<crate::components::UiLayoutNode>(entity)
        {
            node.visible = false;
        }
    }

    ui_tick_toasts(&mut build.context(), storage);

    if progress_bar.is_active() {
        progress_bar.phase = (progress_bar.phase + delta_time).rem_euclid(2.0);
        build.dirty().render_dirty = true;
    }
}
