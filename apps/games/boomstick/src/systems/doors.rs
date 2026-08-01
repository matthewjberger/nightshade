use crate::components::{Group, Lock, OpenShare, ShutHeight, Travel};
use crate::contract::BoomstickEvent;
use crate::data;
use crate::resources::BoomstickResources;
use crate::systems::effects;
use crate::tags::Door;
use nightshade::prelude::*;
use nightshade_physics::contract::components::RigidBodyComponent;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub type Gates<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item Lock,
        &'item Group,
        &'item ShutHeight,
        &'item Travel,
        &'item mut OpenShare,
        &'item mut LocalTransform,
        &'item mut RigidBodyComponent,
    ),
    WithTag<Door>,
>;

pub fn update(
    time: Res<Time>,
    mut resources: ResMut<BoomstickResources>,
    mut events: EventWriter<BoomstickEvent>,
    mut commands: Commands,
    doors: Gates<'_, '_>,
) {
    let delta_time = time.delta_time;
    let Some(at) = resources.player.at else {
        return;
    };
    let red = resources.player.red_key;
    let blue = resources.player.blue_key;
    let thrown = resources.mission.thrown.clone();

    let mut opened: Vec<Vec3> = Vec::new();
    let mut barred = false;
    doors.for_each(
        |_entity, (lock, group, shut, travel, share, transform, body)| {
            if share.0 > 0.0 {
                if share.0 >= 1.0 {
                    return;
                }
                share.0 = (share.0 + data::DOOR_RATE * delta_time).min(1.0);
                transform.translation.y = shut.0 + travel.0 * share.0;
                body.translation = [
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z,
                ];
                return;
            }

            let near = (transform.translation - at).magnitude() < data::DOOR_REACH + 3.0;
            let allowed = match lock {
                Lock::Lever => thrown.contains(&group.0),
                Lock::Red => red && near,
                Lock::Blue => blue && near,
            };
            if allowed {
                share.0 = 0.001;
                opened.push(transform.translation);
                return;
            }
            if near && *lock != Lock::Lever {
                barred = true;
            }
        },
    );

    resources.mission.hint = if barred {
        "the gate wants a key".to_string()
    } else {
        String::new()
    };

    for at in opened {
        effects::sparks(&mut commands, at, Vec3::y());
        events.send(BoomstickEvent::DoorOpened);
    }
}
