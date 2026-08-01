use nightshade::prelude::*;
use nightshade_animation::contract::{AnimationLayer, AnimationPlayer};
use nightshade_platform::prelude::*;

use crate::data;
use crate::queries::clips::index_of;
use crate::resources::LocomotionResources;

pub fn jump(
    mut locomotion_resources: ResMut<LocomotionResources>,
    time: Res<nightshade_platform::contract::time::Time>,
    input: Res<Input>,
    pad: Res<nightshade_gamepad::contract::Gamepad>,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(model) = locomotion_resources.character.model else {
        return;
    };
    let delta_time = time.delta_time;
    if delta_time <= 0.0 {
        return;
    }

    let asked = asked_to_jump(&input, &pad);
    let character = &locomotion_resources.character;
    let airborne = character.airborne;
    let moving = character.velocity.norm() > data::JUMP_MOVING_SPEED;
    let ends_at = if moving {
        data::JUMP_LAND
    } else {
        data::JUMP_RECOVERY_END
    };

    let Some(player) = storage.get_mut::<AnimationPlayer>(model) else {
        return;
    };
    let Some(clip_index) = index_of(player, data::JUMP) else {
        return;
    };

    if asked && !airborne {
        let mut layer = AnimationLayer::new(clip_index);
        layer.looping = false;
        layer.weight = 0.0;
        layer.time = data::JUMP_START;
        layer.speed = data::JUMP_ANTICIPATION_SPEED;
        player.layers = vec![layer];
        locomotion_resources.character.airborne = true;
        return;
    }

    let playing = player
        .layers
        .first()
        .map(|layer| layer.playing && layer.time < ends_at)
        .unwrap_or(false);
    let target = if playing { 1.0 } else { 0.0 };
    let sharpness = if playing {
        data::JUMP_FADE_IN
    } else {
        data::JUMP_FADE_OUT
    };

    let state = &mut locomotion_resources.character;
    state.jump_weight += (target - state.jump_weight) * (1.0 - (-sharpness * delta_time).exp());
    if state.jump_weight < 0.002 {
        state.jump_weight = 0.0;
        state.airborne = false;
    }
    let weight = state.jump_weight;

    let Some(player) = storage.get_mut::<AnimationPlayer>(model) else {
        return;
    };
    if weight <= 0.0 {
        if player
            .layers
            .first()
            .map(|layer| layer.clip_index == clip_index)
            .unwrap_or(false)
        {
            player.layers.clear();
        }
        return;
    }
    if let Some(layer) = player.layers.first_mut() {
        layer.weight = weight;
        layer.speed = if layer.time < data::JUMP_TAKEOFF {
            data::JUMP_ANTICIPATION_SPEED
        } else if layer.time < data::JUMP_LAND {
            data::JUMP_AIR_SPEED
        } else {
            data::JUMP_RECOVERY_SPEED
        };
    }
}

fn asked_to_jump(input: &Input, pad_held: &nightshade_gamepad::contract::Gamepad) -> bool {
    let from_keyboard = input.keyboard.just_pressed(KeyCode::Space);
    let from_pad = nightshade_gamepad::query_active_gamepad(pad_held, |pad| {
        pad.just_pressed(nightshade_gamepad::GamepadButton::South)
    })
    .unwrap_or(false);
    from_keyboard || from_pad
}
