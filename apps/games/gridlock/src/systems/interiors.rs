use crate::commands::rooms::spawn_room;
use crate::components::{Indoor, Velocity, Yaw};
use crate::contract::GridlockEvent;
use crate::data::ask::Ask;
use crate::data::motion::DOOR_STEP;
use crate::data::rooms::ROOM_NAMES;
use crate::data::settings::Settings;
use crate::queries::grid::{cell_of, yaw_of};
use crate::resources::{City, Door, Indoors, Player, Visit};
use nightshade::ecs::Storage;
use nightshade::prelude::*;
use nightshade_camera_controller::contract::ThirdPersonCamera;
use nightshade_config::contract::Config;
use nightshade_input::Actions;
use nightshade_random::prelude::Random;
use nightshade_scene::prelude::LocalTransform;

pub fn enter(
    actions: Res<Actions<Ask>>,
    config: Res<Config<Settings>>,
    player: Res<Player>,
    city: Res<City>,
    mut indoors: ResMut<Indoors>,
    mut random: ResMut<Random>,
    mut events: EventWriter<GridlockEvent>,
    mut commands: Commands,
    storage: &mut Storage,
) {
    indoors.used = false;
    if !actions.just_pressed(Ask::Use) || player.flying || player.car.is_some() {
        return;
    }
    let Some(body) = player.body else {
        return;
    };
    let Some(feet) = storage
        .get::<LocalTransform>(body)
        .map(|transform| transform.translation)
    else {
        return;
    };
    let reach = config.number(Settings::DoorReach);

    if let Some(visit) = indoors.visit {
        if (feet - visit.exit).norm() > reach {
            return;
        }
        let serial = indoors.serial;
        let gone: Vec<Entity> = storage
            .query_ref::<(&Indoor,)>()
            .iter()
            .filter(|(_, (indoor,))| indoor.0 == serial)
            .map(|(entity, _)| entity)
            .collect();
        for entity in gone {
            commands.queue(move |storage, _| {
                storage.despawn_recursive(entity);
            });
        }
        move_hero(storage, &player, body, visit.outside, visit.yaw);
        indoors.visit = None;
        indoors.lots.clear();
        indoors.used = true;
        events.send(GridlockEvent::Exited {
            place: ROOM_NAMES[visit.room as usize].to_string(),
        });
        return;
    }

    let center = cell_of(feet);
    let mut nearest: Option<(f32, Door)> = None;
    for dz in -1..=1 {
        for dx in -1..=1 {
            let Some(layout) = city.cells.get(&[center[0] + dx, center[1] + dz]) else {
                continue;
            };
            for door in &layout.doors {
                let distance = (door.position - feet).norm();
                if distance <= reach && nearest.is_none_or(|(best, _)| distance < best) {
                    nearest = Some((distance, *door));
                }
            }
        }
    }
    let Some((_, door)) = nearest else {
        return;
    };
    indoors.serial += 1;
    let serial = indoors.serial;
    let (lots, exit, entry) = spawn_room(
        &mut commands,
        &mut random,
        door.room,
        door.size,
        serial,
        config.number(Settings::RoomLight),
    );
    indoors.lots = lots;
    indoors.visit = Some(Visit {
        outside: door.position + door.facing * DOOR_STEP,
        yaw: yaw_of(door.facing),
        exit,
        room: door.room,
    });
    move_hero(storage, &player, body, entry, 0.0);
    indoors.used = true;
    events.send(GridlockEvent::Entered {
        place: ROOM_NAMES[door.room as usize].to_string(),
    });
}

fn move_hero(storage: &mut Storage, player: &Player, body: Entity, position: Vec3, yaw: f32) {
    if let Some(transform) = storage.get_mut::<LocalTransform>(body) {
        transform.translation = position;
        transform.rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y());
    }
    storage.set(body, Velocity::default());
    storage.set(body, Yaw(yaw));
    if let Some(camera) = player.walk_camera
        && let Some(third_person) = storage.get_mut::<ThirdPersonCamera>(camera)
    {
        third_person.current_focus = position + Vec3::new(0.0, third_person.height_offset, 0.0);
        third_person.yaw = yaw + std::f32::consts::PI;
        third_person.target_yaw = third_person.yaw;
    }
}
