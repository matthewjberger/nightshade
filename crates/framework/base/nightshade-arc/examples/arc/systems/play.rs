use crate::resources::Swing;
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_arc::queries::{facing_from_yaw, in_arc};
use nightshade_plane::Plane;
use nightshade_plane::commands::move_across;
use nightshade_platform::contract::input::Input;
use nightshade_platform::contract::time::Time;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_scene::contract::LocalTransform;

pub fn take_the_orders(
    input: Res<Input>,
    time: Res<Time>,
    mut swing: ResMut<Swing>,
    storage: &mut Storage,
) {
    let delta = time.delta_time;
    if input.keyboard.is_key_pressed(KeyCode::ArrowLeft) {
        swing.yaw += theme::RATE * delta;
    }
    if input.keyboard.is_key_pressed(KeyCode::ArrowRight) {
        swing.yaw -= theme::RATE * delta;
    }
    if input.keyboard.just_pressed(KeyCode::KeyQ) {
        swing.half_angle = (swing.half_angle - theme::ANGLE_NUDGE).max(theme::NARROWEST_ANGLE);
    }
    if input.keyboard.just_pressed(KeyCode::KeyE) {
        swing.half_angle = (swing.half_angle + theme::ANGLE_NUDGE).min(theme::WIDEST_ANGLE);
    }
    if input.keyboard.just_pressed(KeyCode::KeyZ) {
        swing.reach = (swing.reach - theme::REACH_NUDGE).max(theme::SHORTEST_REACH);
    }
    if input.keyboard.just_pressed(KeyCode::KeyX) {
        swing.reach = (swing.reach + theme::REACH_NUDGE).min(theme::LONGEST_REACH);
    }

    let mut walk = 0.0;
    if input.keyboard.is_key_pressed(KeyCode::ArrowUp) {
        walk += 1.0;
    }
    if input.keyboard.is_key_pressed(KeyCode::ArrowDown) {
        walk -= 1.0;
    }
    let Some(swinger) = swing.swinger else {
        return;
    };
    if walk == 0.0 {
        return;
    }
    let step = facing_from_yaw(swing.yaw) * walk * theme::STEP * delta;
    if let Some(transform) = storage.get_mut::<LocalTransform>(swinger) {
        move_across(Plane::Ground, &mut transform.translation, step);
        transform.translation.x = transform.translation.x.clamp(-theme::RING, theme::RING);
        transform.translation.z = transform.translation.z.clamp(-theme::RING, theme::RING);
    }
}

pub fn sweep_the_arc(swing: Res<Swing>, mut readout: ResMut<Readout>, storage: &mut Storage) {
    let (swinger, targets, yaw, half_angle, reach) = {
        (
            swing.swinger,
            swing.targets.clone(),
            swing.yaw,
            swing.half_angle,
            swing.reach,
        )
    };
    let facing = facing_from_yaw(yaw);
    let Some(swinger) = swinger else {
        return;
    };
    let mut from = Vec3::zeros();
    if let Some(transform) = storage.get_mut::<LocalTransform>(swinger) {
        transform.rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y());
        from = transform.translation;
    }

    let mut struck = 0;
    for target in targets {
        let Some(transform) = storage.get_mut::<LocalTransform>(target) else {
            continue;
        };
        let inside = in_arc(
            Plane::Ground,
            from,
            facing,
            transform.translation,
            reach,
            half_angle,
        );
        transform.translation.y = if inside { theme::LIFT } else { 0.5 };
        if inside {
            struck += 1;
        }
    }

    readout.notes.clear();
    readout.notes.push(format!("{struck} inside the arc"));
    readout
        .notes
        .push(format!("half angle {half_angle:.2}, reach {reach:.1}"));
    readout
        .notes
        .push(format!("yaw {:.2}", yaw.rem_euclid(std::f32::consts::TAU)));
}
