use crate::components::{Angle, Cooldown, Speed};
use crate::contract::NeonStaticEvent;
use crate::data;
use crate::queries::{blocked, clear_line};
use crate::resources::{FloorPhase, NeonStaticResources};
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::*;

pub fn hunt(
    time: Res<Time>,
    mut resources: ResMut<NeonStaticResources>,
    mut events: EventWriter<NeonStaticEvent>,
    enemies: Query<(&Speed, &mut Angle, &mut LocalTransform), Without<Cooldown>>,
) {
    let resources = &mut *resources;
    let delta_time = time.delta_time;
    let target = resources.player_position;
    let fighting = resources.floor.phase == FloorPhase::Fighting;

    let mut standing = Vec::with_capacity(resources.enemies.len());
    let mut touched = false;
    {
        let floor = &resources.floor;
        enemies.for_each(|entity, (speed, angle, transform)| {
            let mut position = Vec2::new(transform.translation.x, transform.translation.y);
            let toward = target - position;
            let distance = toward.magnitude();
            if fighting && distance > 1.0e-3 {
                let direction = toward / distance;
                angle.0 = direction.y.atan2(direction.x);
                if distance < data::ENEMY_SIGHT && clear_line(floor, position, target) {
                    let step = direction * speed.0 * delta_time;
                    let sideways = Vec2::new(position.x + step.x, position.y);
                    if !blocked(floor, sideways, data::BODY_RADIUS) {
                        position.x = sideways.x;
                    }
                    let forward = Vec2::new(position.x, position.y + step.y);
                    if !blocked(floor, forward, data::BODY_RADIUS) {
                        position.y = forward.y;
                    }
                }
                transform.translation = Vec3::new(position.x, position.y, 0.0);
                transform.rotation = nalgebra_glm::quat_angle_axis(angle.0, &Vec3::z());
                if (target - position).magnitude() < data::TOUCH_RADIUS {
                    touched = true;
                }
            }
            standing.push((entity, position));
        });
    }
    resources.enemies = standing;

    if touched {
        events.send(NeonStaticEvent::PlayerKilled { at: target });
    }
}
