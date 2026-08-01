use crate::commands::send_shot;
use crate::contract::FireQueue;
use crate::data;
use crate::data::Ask;
use crate::data::arsenal::GUN_COUNT;
use crate::queries::{can_fire, held};
use crate::resources::BoomstickResources;
use crate::tags::Eyes;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn update(
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    mut resources: ResMut<BoomstickResources>,
    mut queue: ResMut<FireQueue>,
    eyes: Query<(&mut LocalTransform,), WithTag<Eyes>>,
) {
    let delta_time = time.delta_time;
    let wants_fire = actions.pressed(Ask::Fire);
    let player = &mut resources.player;
    player.cooldown = (player.cooldown - delta_time).max(0.0);
    player.kick -= player.kick * (data::KICK_RECOVER_RATE * delta_time).min(1.0);
    player.punch -= player.punch * (data::VIEWMODEL_RECOVER_RATE * delta_time).min(1.0);

    for (slot, ask) in [Ask::GunOne, Ask::GunTwo, Ask::GunThree, Ask::GunFour]
        .into_iter()
        .enumerate()
    {
        if actions.just_pressed(ask) && player.owned[slot] {
            player.gun = slot;
        }
    }
    if actions.just_pressed(Ask::PreviousGun) {
        player.gun = previous_gun(player.gun, &player.owned);
    }
    if actions.just_pressed(Ask::NextGun) {
        player.gun = next_gun(player.gun, &player.owned);
    }

    let queue = &mut *queue;
    let stand = player.at.unwrap_or_default();
    eyes.for_each(|_entity, (transform,)| {
        if wants_fire && can_fire(player) {
            let gun = held(player);
            let origin = stand + transform.translation;
            let forward =
                nalgebra_glm::quat_rotate_vec3(&transform.rotation, &Vec3::new(0.0, 0.0, -1.0));
            send_shot(queue, player.gun, origin, forward);
            player.spend(gun.ammo);
            player.cooldown = gun.cooldown;
            player.kick += gun.kick;
            player.punch += gun.punch;
        }
        let step = player.kick - player.kick_applied;
        player.kick_applied = player.kick;
        transform.rotation *= nalgebra_glm::quat_angle_axis(step, &Vec3::x());
    });
}

fn next_gun(from: usize, owned: &[bool; GUN_COUNT]) -> usize {
    for step in 1..=GUN_COUNT {
        let slot = (from + step) % GUN_COUNT;
        if owned[slot] {
            return slot;
        }
    }
    from
}

fn previous_gun(from: usize, owned: &[bool; GUN_COUNT]) -> usize {
    for step in 1..=GUN_COUNT {
        let slot = (from + GUN_COUNT - step) % GUN_COUNT;
        if owned[slot] {
            return slot;
        }
    }
    from
}
