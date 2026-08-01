use nightshade::prelude::*;
use nightshade_animation::contract::{AnimationLayer, AnimationPlayer};

use crate::data;
use crate::queries::clips::index_of;
use crate::resources::LocomotionResources;

pub fn turn_in_place(
    mut locomotion_resources: ResMut<LocomotionResources>,
    time: Res<nightshade_platform::contract::time::Time>,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(model) = locomotion_resources.character.model else {
        return;
    };
    let delta_time = time.delta_time;
    if delta_time <= 0.0 {
        return;
    }

    let character = &locomotion_resources.character;
    if character.airborne || character.jump_weight > 0.0 {
        return;
    }
    let still = character.velocity.norm() < data::TURN_STILL_SPEED;
    let error = character.facing_error;
    let held = character.turn_clip;

    let wanted = if still && error.abs() > data::TURN_ENTER {
        let name = match (error > 0.0, error.abs() > data::TURN_WIDE) {
            (true, true) => data::LEFT_TURN_WIDE,
            (true, false) => data::LEFT_TURN,
            (false, true) => data::RIGHT_TURN_WIDE,
            (false, false) => data::RIGHT_TURN,
        };
        index_of(
            storage
                .get::<AnimationPlayer>(model)
                .expect("the model carries a player"),
            name,
        )
    } else {
        None
    };

    let playing = held.is_some_and(|clip_index| {
        storage
            .get::<AnimationPlayer>(model)
            .and_then(|player| player.layers.first().cloned())
            .is_some_and(|layer| layer.clip_index == clip_index && layer.playing)
    });

    let started = match (held, wanted) {
        (None, Some(clip_index)) => Some(clip_index),
        _ => None,
    };
    let target = if playing || started.is_some() {
        1.0
    } else {
        0.0
    };

    let state = &mut locomotion_resources.character;
    if let Some(clip_index) = started {
        state.turn_clip = Some(clip_index);
    }
    state.turn_weight +=
        (target - state.turn_weight) * (1.0 - (-data::TURN_FADE * delta_time).exp());
    if state.turn_weight < 0.004 && !playing {
        state.turn_weight = 0.0;
        state.turn_clip = None;
    }
    let weight = state.turn_weight;
    let clip = state.turn_clip;

    let Some(player) = storage.get_mut::<AnimationPlayer>(model) else {
        return;
    };
    let Some(clip_index) = clip else {
        return;
    };
    if let Some(index) = started {
        let mut layer = AnimationLayer::new(index);
        layer.looping = false;
        layer.weight = weight;
        player.layers = vec![layer];
        return;
    }
    if let Some(layer) = player.layers.first_mut()
        && layer.clip_index == clip_index
    {
        layer.weight = weight;
        if weight <= 0.0 {
            player.layers.clear();
        }
    }
}
