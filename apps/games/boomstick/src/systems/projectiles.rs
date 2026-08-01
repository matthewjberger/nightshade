use crate::commands::{hurt, send_blast, wound};
use crate::components::{Damage, EnemyKind, EnemyMood, Fall, Health, Life, Splash, Velocity};
use crate::contract::{BoomstickEvent, FireQueue};
use crate::data;
use crate::queries::{beast_height, hit_radius, point_in_column};
use crate::resources::BoomstickResources;
use crate::systems::effects;
use crate::tags::{EnemyShot, PlayerShot};
use nightshade::prelude::*;
use nightshade_physics::prelude::{PhysicsWorld, physics_world_cast_ray};
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

struct Landing {
    bolt: Entity,
    at: Vec3,
    target: Option<Entity>,
    damage: f32,
    splash: f32,
}

pub fn player_shots(
    time: Res<Time>,
    physics: Res<PhysicsWorld>,
    mut queue: ResMut<FireQueue>,
    mut events: EventWriter<BoomstickEvent>,
    mut commands: Commands,
    bolts: Query<
        (
            &mut Velocity,
            &Damage,
            &Splash,
            &mut Life,
            &mut LocalTransform,
        ),
        WithTag<PlayerShot>,
    >,
    enemies: Query<(&EnemyKind, &EnemyMood, &Health, &LocalTransform)>,
) {
    let delta_time = time.delta_time;
    let mut standing: Vec<(Entity, Vec3, f32, f32, f32)> = Vec::new();
    enemies.for_each(|entity, (kind, mood, health, transform)| {
        if *mood != EnemyMood::Dying {
            standing.push((
                entity,
                transform.translation,
                beast_height(*kind),
                hit_radius(*kind) + data::SHOT_RADIUS,
                health.0,
            ));
        }
    });

    let mut landed: Vec<Landing> = Vec::new();
    bolts.for_each(|entity, (velocity, damage, splash, life, transform)| {
        life.0 -= delta_time;
        let from = transform.translation;
        let step = velocity.0 * delta_time;
        let to = from + step;
        transform.translation = to;

        if life.0 <= 0.0 {
            landed.push(Landing {
                bolt: entity,
                at: to,
                target: None,
                damage: damage.0,
                splash: splash.0,
            });
            return;
        }
        for (target, foot, height, radius, _) in &standing {
            if point_in_column(to, *foot, *height, *radius) {
                landed.push(Landing {
                    bolt: entity,
                    at: to,
                    target: Some(*target),
                    damage: damage.0,
                    splash: splash.0,
                });
                return;
            }
        }
        if let Some(hit) = sweep(&physics, from, step) {
            landed.push(Landing {
                bolt: entity,
                at: hit,
                target: None,
                damage: damage.0,
                splash: splash.0,
            });
        }
    });

    for strike in landed {
        commands.despawn_recursive(strike.bolt);
        if strike.splash > 0.0 {
            send_blast(
                &mut queue,
                strike.at,
                strike.splash,
                data::ROCKET_BLAST,
                true,
            );
        } else {
            effects::sparks(&mut commands, strike.at, Vec3::y());
        }
        let Some(target) = strike.target else {
            continue;
        };
        let Some(entry) = standing
            .iter_mut()
            .find(|(held, _, _, _, _)| *held == target)
        else {
            continue;
        };
        if entry.4 <= 0.0 {
            continue;
        }
        entry.4 -= strike.damage;
        let killed = entry.4 <= 0.0;
        let at = entry.1 + Vec3::new(0.0, entry.2 * data::HIT_CENTER_SHARE, 0.0);
        wound(
            &mut commands,
            target,
            entry.4,
            strike.damage >= data::REEL_DAMAGE,
        );
        events.send(BoomstickEvent::EnemyHit {
            damage: strike.damage,
        });
        if killed {
            effects::gibs(&mut commands, at);
            events.send(BoomstickEvent::EnemyKilled);
        }
    }
}

pub type Incoming<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item mut Velocity,
        &'item Damage,
        &'item Splash,
        &'item mut Life,
        &'item mut LocalTransform,
        Option<&'item Fall>,
    ),
    WithTag<EnemyShot>,
>;

pub fn enemy_shots(
    time: Res<Time>,
    physics: Res<PhysicsWorld>,
    mut resources: ResMut<BoomstickResources>,
    mut queue: ResMut<FireQueue>,
    mut events: EventWriter<BoomstickEvent>,
    mut commands: Commands,
    bolts: Incoming<'_, '_>,
) {
    let delta_time = time.delta_time;
    let stance = data::CAPSULE_HALF_HEIGHT + data::CAPSULE_RADIUS;
    let foot = resources
        .player
        .at
        .map(|at| at - Vec3::new(0.0, stance, 0.0));

    let mut landed: Vec<Landing> = Vec::new();
    bolts.for_each(
        |entity, (velocity, damage, splash, life, transform, fall)| {
            life.0 -= delta_time;
            if let Some(fall) = fall {
                velocity.0.y -= fall.0 * delta_time;
            }
            let from = transform.translation;
            let step = velocity.0 * delta_time;
            let to = from + step;
            transform.translation = to;

            if life.0 <= 0.0 {
                landed.push(Landing {
                    bolt: entity,
                    at: to,
                    target: None,
                    damage: damage.0,
                    splash: splash.0,
                });
                return;
            }
            if let Some(foot) = foot
                && point_in_column(
                    to,
                    foot,
                    stance * 2.0,
                    data::CAPSULE_RADIUS + data::SHOT_RADIUS,
                )
            {
                landed.push(Landing {
                    bolt: entity,
                    at: to,
                    target: resources.player.body,
                    damage: damage.0,
                    splash: splash.0,
                });
                return;
            }
            if let Some(hit) = sweep(&physics, from, step) {
                landed.push(Landing {
                    bolt: entity,
                    at: hit,
                    target: None,
                    damage: damage.0,
                    splash: splash.0,
                });
            }
        },
    );

    for strike in landed {
        commands.despawn_recursive(strike.bolt);
        if strike.splash > 0.0 {
            send_blast(&mut queue, strike.at, strike.splash, strike.damage, false);
            continue;
        }
        effects::sparks(&mut commands, strike.at, Vec3::y());
        if strike.target.is_none() || resources.player.dead {
            continue;
        }
        hurt(&mut resources.player, strike.damage);
        resources.player.flash = 1.0;
        resources.player.kick += data::HURT_KICK;
        events.send(BoomstickEvent::PlayerHurt {
            damage: strike.damage,
        });
    }
}

fn sweep(physics: &PhysicsWorld, from: Vec3, step: Vec3) -> Option<Vec3> {
    let travel = step.magnitude();
    if travel <= 0.0 {
        return None;
    }
    physics_world_cast_ray(
        physics,
        from,
        step / travel,
        travel + data::SHOT_RADIUS,
        None,
    )
    .map(|hit| hit.point)
}
