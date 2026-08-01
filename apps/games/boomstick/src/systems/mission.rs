use crate::contract::BoomstickEvent;
use crate::data;
use crate::data::levels::{LEVEL_COUNT, plan};
use crate::resources::{BoomstickResources, Kit, Mission, Screen};
use crate::systems::level;
use crate::tags::{Eyes, LevelPiece, Player};
use nightshade::prelude::*;
use nightshade_camera::resources::ActiveCamera;
use nightshade_physics::contract::components::CharacterControllerComponent;
use nightshade_platform::contract::time::Time;
use nightshade_scene::prelude::*;
use nightshade_state::resources::NextState;

pub fn request(mut resources: ResMut<BoomstickResources>) {
    if !resources.mission.built {
        resources.mission.wanted = true;
    }
}

pub fn build(
    mut resources: ResMut<BoomstickResources>,
    mut active_camera: ResMut<ActiveCamera>,
    mut commands: Commands,
    pieces: Query<(&LocalTransform,), WithTag<LevelPiece>>,
    bodies: Query<(&mut LocalTransform, &mut CharacterControllerComponent), WithTag<Player>>,
    eyes: Query<(&mut LocalTransform,), WithTag<Eyes>>,
) {
    if !resources.mission.wanted {
        return;
    }
    pieces.for_each(|entity, _| commands.despawn_recursive(entity));

    let level = resources.mission.level;
    let plan = plan(level);
    let retry = resources.mission.retry;
    let kit = if retry {
        resources.mission.kit
    } else {
        Kit {
            health: resources.player.health.max(data::MEDKIT),
            armor: resources.player.armor,
            ammo: resources.player.ammo,
            owned: resources.player.owned,
            gun: resources.player.gun,
        }
    };

    resources.mission = Mission {
        level,
        kit,
        built: true,
        hidden: plan.secrets.len() as u32,
        banner: plan.brief.to_string(),
        banner_left: data::BRIEF_TIME,
        ambushes: plan.ambushes.to_vec(),
        hazards: plan.hazards.to_vec(),
        ..Default::default()
    };
    level::build(&mut commands, plan);

    let player = &mut resources.player;
    player.health = kit.health;
    player.armor = kit.armor;
    player.ammo = kit.ammo;
    player.owned = kit.owned;
    player.gun = kit.gun;
    player.red_key = false;
    player.blue_key = false;
    player.dead = false;
    player.flash = 0.0;
    player.cooldown = 0.0;
    player.travelled = 0.0;
    player.hazard_clock = 0.0;

    let start = plan.start + Vec3::new(0.0, data::STANCE_HEIGHT, 0.0);
    let mut moved = None;
    bodies.for_each(|entity, (transform, controller)| {
        transform.translation = start;
        controller.velocity = Vec3::zeros();
        controller.grounded = false;
        moved = Some(entity);
    });
    if let Some(body) = moved {
        nightshade_physics::commands::reset_physics_interpolation(&mut commands, body);
    }
    let facing = nalgebra_glm::quat_angle_axis(plan.facing, &Vec3::y());
    eyes.for_each(|_entity, (transform,)| {
        transform.rotation = facing;
    });

    active_camera.0 = resources.player.eyes;
    resources.shown.health = u32::MAX;
    resources.shown.armor = u32::MAX;
    resources.shown.ammo = u32::MAX;
    resources.shown.message.clear();
}

pub fn track(
    time: Res<Time>,
    mut resources: ResMut<BoomstickResources>,
    events: EventReader<BoomstickEvent>,
) {
    let delta_time = time.delta_time;
    resources.player.flash = (resources.player.flash - delta_time * data::FLASH_FADE).max(0.0);
    let mut killed = 0;
    for event in &events {
        if event == &BoomstickEvent::EnemyKilled {
            killed += 1;
        }
    }
    let mission = &mut resources.mission;
    mission.kills += killed;
    mission.banner_left = (mission.banner_left - delta_time).max(0.0);
    if !mission.populated {
        return;
    }
    if mission.done || mission.failed {
        mission.since_end += delta_time;
        return;
    }
    mission.clock += delta_time;
    if mission.hint.is_empty() && mission.kills >= mission.standing && mission.standing > 0 {
        mission.hint = "the stair stands open".to_string();
    }
    if resources.player.dead {
        let mission = &mut resources.mission;
        mission.failed = true;
        mission.since_end = 0.0;
        mission.banner = "they dragged you out".to_string();
        mission.banner_left = data::DEATH_DELAY;
    }
}

pub fn finish(mut resources: ResMut<BoomstickResources>, mut next: ResMut<NextState<Screen>>) {
    let done = resources.mission.done;
    let failed = resources.mission.failed;
    if !done && !failed {
        return;
    }
    let delay = if done {
        data::CLEAR_DELAY
    } else {
        data::DEATH_DELAY
    };
    if resources.mission.since_end < delay {
        return;
    }

    if failed {
        resources.mission.retry = true;
        resources.mission.wanted = true;
        resources.mission.failed = false;
        return;
    }

    let level = resources.mission.level;
    resources.record.cleared[level] = true;
    resources.record.kills += resources.mission.kills;
    resources.record.secrets += resources.mission.secrets;
    resources.record.time += resources.mission.clock;
    resources.mission.done = false;
    resources.shown.board.clear();
    next.pending = Some(if level + 1 >= LEVEL_COUNT {
        Screen::Ending
    } else {
        Screen::Tally
    });
}
