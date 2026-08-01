use crate::data::rules::{HAND_BOB, HAND_BOB_RATE, HAND_LAG, HAND_SETTLED, HAND_SWAY};
use crate::resources::AshfallResources;
use crate::tags::Flashlight;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn sway(
    time: Res<Time>,
    mut resources: ResMut<AshfallResources>,
    lights: Query<(&mut LocalTransform,), WithTag<Flashlight>>,
) {
    let resources = &mut *resources;
    if !resources.screen.is_in_town() {
        return;
    }
    let head = resources.player.head;
    let share = (time.delta_time * HAND_LAG).min(1.0);
    resources.player.hand = approach(resources.player.hand, head, share);
    let hand = resources.player.hand;
    let behind = nalgebra_glm::quat_cross(&nalgebra_glm::quat_conjugate(&head), &hand);

    let clock = time.uptime_seconds as f32;
    let walking = if resources.player.moving { 1.0 } else { 0.0 };
    let bob = (clock * HAND_BOB_RATE).sin() * HAND_BOB * walking;
    let drift = (clock * HAND_BOB_RATE * 0.5).sin() * HAND_BOB * 0.6 * walking;

    lights.for_each(|_entity, (transform,)| {
        transform.rotation = behind;
        transform.translation = Vec3::new(
            0.14 + drift - behind.j * HAND_SWAY,
            -0.14 + bob + behind.i * HAND_SWAY,
            0.0,
        );
    });
}

fn approach(from: Quat, to: Quat, share: f32) -> Quat {
    let to = if nalgebra_glm::quat_dot(&from, &to) < 0.0 {
        -to
    } else {
        to
    };
    if nalgebra_glm::quat_dot(&from, &to).abs() > HAND_SETTLED {
        return to;
    }
    nalgebra_glm::quat_normalize(&(from + (to - from) * share))
}
