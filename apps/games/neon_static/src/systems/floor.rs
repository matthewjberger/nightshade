use crate::commands::load_floor_plan;
use crate::contract::NeonStaticEvent;
use crate::data;
use crate::data::Ask;
use crate::resources::{FloorPhase, NeonStaticResources};
use crate::systems::setup::{spawn_actors, spawn_blood};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;

pub fn settle(
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    mut resources: ResMut<NeonStaticResources>,
    events: EventReader<NeonStaticEvent>,
    mut commands: Commands,
) {
    let resources = &mut *resources;
    let delta_time = time.delta_time;

    for event in &events {
        match event {
            NeonStaticEvent::ShotFired { .. } => {
                resources.floor.shake = (resources.floor.shake + data::SHAKE_FIRE).min(1.0);
            }
            NeonStaticEvent::EnemyKilled { .. } => {
                resources.floor.killed += 1;
                resources.floor.remaining = resources.floor.remaining.saturating_sub(1);
                resources.floor.shake = (resources.floor.shake + data::SHAKE_KILL).min(1.0);
            }
            NeonStaticEvent::PlayerKilled { at } => {
                if resources.floor.phase == FloorPhase::Fighting {
                    resources.floor.phase = FloorPhase::Dead;
                    resources.floor.restart_timer = data::RESTART_DELAY;
                    resources.floor.shake = data::SHAKE_DEATH;
                    resources.floor.flash = 1.0;
                    if let Some(player) = resources.player.take() {
                        commands.despawn(player);
                        resources.round.retain(|held| *held != player);
                    }
                    let blood = spawn_blood(&resources.sheets, *at, 0.0, &mut commands);
                    resources.round.push(blood);
                }
            }
        }
    }

    let asked = actions.just_pressed(Ask::Restart);
    match resources.floor.phase {
        FloorPhase::Fighting => {
            resources.floor.elapsed += delta_time;
            if resources.floor.remaining == 0 {
                resources.floor.phase = FloorPhase::Cleared;
            } else if asked {
                restart(resources, &mut commands);
            }
        }
        FloorPhase::Cleared => {
            if asked {
                restart(resources, &mut commands);
            }
        }
        FloorPhase::Dead => {
            resources.floor.restart_timer -= delta_time;
            if asked || resources.floor.restart_timer <= 0.0 {
                restart(resources, &mut commands);
            }
        }
    }

    resources.floor.shake = (resources.floor.shake - data::SHAKE_DECAY * delta_time).max(0.0);
    resources.floor.flash = (resources.floor.flash - data::FLASH_DECAY * delta_time).max(0.0);
}

fn restart(resources: &mut NeonStaticResources, commands: &mut Commands) {
    for entity in std::mem::take(&mut resources.round) {
        commands.despawn(entity);
    }
    resources.enemies.clear();
    resources.player = None;
    load_floor_plan(&mut resources.floor);
    resources.floor.phase = FloorPhase::Fighting;
    spawn_actors(resources, commands);
}
