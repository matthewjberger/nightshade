use crate::data;
use crate::resources::{BoomstickResources, Screen};
use crate::tags::{Eyes, Player};
use nightshade::prelude::*;
use nightshade_physics::contract::components::CharacterControllerComponent;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_state::resources::State;

pub fn update(
    time: Res<Time>,
    input: Res<Input>,
    mut resources: ResMut<BoomstickResources>,
    mut bodies: Query<(&LocalTransform, &mut CharacterControllerComponent), WithTag<Player>>,
    eyes: Query<(&mut LocalTransform,), WithTag<Eyes>>,
) {
    let delta_time = time.delta_time;
    let player = &mut resources.player;
    let Some(body) = player.body else {
        return;
    };

    let mut stance = None;
    bodies.for_entity(body, |(transform, controller)| {
        controller.velocity.y = controller.velocity.y.max(-data::TERMINAL_FALL);
        stance = Some((
            transform.translation,
            controller.velocity.xz().magnitude(),
            controller.velocity.y,
            controller.grounded,
        ));
    });
    let Some((position, speed, rise, grounded)) = stance else {
        return;
    };
    player.at = Some(position);

    let descent = player.descent;
    player.descent = rise.min(0.0);
    let landed = grounded && !player.was_grounded;
    player.was_grounded = grounded;
    if landed && descent < -data::DIP_SPEED {
        let share = (descent / -data::TERMINAL_FALL).clamp(0.0, 1.0);
        player.kick -= data::LANDING_DIP * share;
    }

    let moving = grounded && speed > 0.5;
    let wanted = if moving { 1.0 } else { 0.0 };
    player.bob_strength +=
        (wanted - player.bob_strength) * (data::BOB_SETTLE_RATE * delta_time).min(1.0);
    if moving {
        player.travelled += speed * delta_time;
    }

    let phase = player.travelled / data::BOB_DISTANCE_PER_CYCLE * std::f32::consts::TAU;
    let vertical = (phase * 2.0).sin() * data::BOB_VERTICAL * player.bob_strength;
    let lateral = phase.sin() * data::BOB_LATERAL * player.bob_strength;

    let aim = Vec2::new(
        -input.mouse.raw_mouse_delta.x * 0.06,
        -input.mouse.raw_mouse_delta.y * 0.06,
    );
    let lag = (data::SWAY_LAG_RATE * delta_time).min(1.0);
    player.sway += (aim - player.sway) * lag;
    player.sway = Vec2::new(
        player.sway.x.clamp(-data::SWAY_TRAVEL, data::SWAY_TRAVEL),
        player.sway.y.clamp(-data::SWAY_TRAVEL, data::SWAY_TRAVEL),
    );

    let sink = if player.dead { -0.45 } else { 0.0 };
    let offset = Vec3::new(lateral, data::EYE_HEIGHT + vertical + sink, 0.0);
    eyes.for_each(|_entity, (transform,)| {
        transform.translation = offset;
    });
}

pub fn grip(
    screen: Res<State<Screen>>,
    resources: Res<BoomstickResources>,
    mut window: ResMut<Window>,
    bodies: Query<(&mut CharacterControllerComponent,), WithTag<Player>>,
) {
    let playing = screen.current == Screen::Level;
    let wanted = playing && window.is_focused;
    if window.cursor_locked != wanted {
        set_cursor_locked(&mut window, wanted);
        set_cursor_visible(&mut window, !wanted);
    }
    let driving = playing && !resources.player.dead;
    bodies.for_each(|_entity, (controller,)| {
        if controller.engine_input_enabled != driving {
            controller.engine_input_enabled = driving;
        }
    });
}
