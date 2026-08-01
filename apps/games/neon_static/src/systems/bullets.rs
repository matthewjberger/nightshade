use crate::components::{Life, Spin, Velocity};
use crate::contract::NeonStaticEvent;
use crate::data;
use crate::queries::{distance_to_step, solid_at};
use crate::resources::NeonStaticResources;
use crate::systems::setup::spawn_blood;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::*;

pub fn advance(
    time: Res<Time>,
    mut resources: ResMut<NeonStaticResources>,
    mut events: EventWriter<NeonStaticEvent>,
    mut commands: Commands,
    bullets: Query<(&Velocity, &mut Life, &mut LocalTransform), Without<Spin>>,
) {
    let resources = &mut *resources;
    let delta_time = time.delta_time;
    let mut spent = Vec::new();
    let mut kills: Vec<(Entity, Vec2, f32)> = Vec::new();

    {
        let floor = &resources.floor;
        let standing = &resources.enemies;
        bullets.for_each(|entity, (velocity, life, transform)| {
            life.0 -= delta_time;
            if life.0 <= 0.0 {
                spent.push(entity);
                return;
            }
            let previous = Vec2::new(transform.translation.x, transform.translation.y);
            let position = previous + velocity.0 * delta_time;
            if solid_at(floor, position) {
                spent.push(entity);
                return;
            }
            transform.translation = Vec3::new(position.x, position.y, 0.0);

            let angle = velocity.0.y.atan2(velocity.0.x);
            for (target, at) in standing.iter() {
                if kills.iter().any(|(killed, _, _)| killed == target) {
                    continue;
                }
                if distance_to_step(previous, position, *at) < data::HIT_RADIUS {
                    spent.push(entity);
                    kills.push((*target, *at, angle));
                    return;
                }
            }
        });
    }

    for entity in spent {
        commands.despawn(entity);
    }

    for (enemy, at, angle) in kills {
        commands.despawn(enemy);
        resources.enemies.retain(|(held, _)| *held != enemy);
        resources.round.retain(|held| *held != enemy);
        let blood = spawn_blood(&resources.sheets, at, angle, &mut commands);
        resources.round.push(blood);
        events.send(NeonStaticEvent::EnemyKilled { at });
    }
}
