use crate::commands::{paint, wound};
use crate::components::{
    Damage, EnemyKind, EnemyMood, Fall, Group, Health, Life, Splash, Velocity,
};
use crate::contract::{BoomstickEvent, FireQueue};
use crate::data;
use crate::data::arsenal::{Muzzle, gun};
use crate::queries::{beast_height, falloff, hit_radius, ray_hits_column};
use crate::resources::BoomstickResources;
use crate::systems::effects;
use crate::tags::{EnemyShot, LevelPiece, Lever, PlayerShot, Secret};
use nightshade::prelude::*;
use nightshade_graphics::prelude::spawn_mesh_at;
use nightshade_physics::prelude::{PhysicsWorld, physics_world_cast_ray};
use nightshade_render::data::material::Material;
use nightshade_scene::prelude::*;

struct Standing {
    entity: Entity,
    foot: Vec3,
    height: f32,
    radius: f32,
    health: f32,
}

impl Standing {
    fn center(&self) -> Vec3 {
        self.foot + Vec3::new(0.0, self.height * data::HIT_CENTER_SHARE, 0.0)
    }
}

pub fn resolve(
    physics: Res<PhysicsWorld>,
    mut resources: ResMut<BoomstickResources>,
    mut queue: ResMut<FireQueue>,
    mut events: EventWriter<BoomstickEvent>,
    mut commands: Commands,
    enemies: Query<(&EnemyKind, &EnemyMood, &Health, &LocalTransform)>,
    levers: Query<(&Health, &Group), WithTag<Lever>>,
    secrets: Query<(&Health,), WithTag<Secret>>,
) {
    let shots = std::mem::take(&mut queue.shots);
    let lobs = std::mem::take(&mut queue.lobs);
    if shots.is_empty() && lobs.is_empty() {
        return;
    }
    let shooter = resources.player.body;

    let mut standing: Vec<Standing> = Vec::new();
    enemies.for_each(|entity, (kind, mood, health, transform)| {
        if *mood != EnemyMood::Dying {
            standing.push(Standing {
                entity,
                foot: transform.translation,
                height: beast_height(*kind),
                radius: hit_radius(*kind),
                health: health.0,
            });
        }
    });
    let mut switches: Vec<(Entity, f32, u32)> = Vec::new();
    levers.for_each(|entity, (health, group)| switches.push((entity, health.0, group.0)));
    let mut walls: Vec<(Entity, f32)> = Vec::new();
    secrets.for_each(|entity, (health,)| walls.push((entity, health.0)));

    for (index, shot) in shots.into_iter().enumerate() {
        let origin = Vec3::new(shot.origin[0], shot.origin[1], shot.origin[2]);
        let forward = Vec3::new(shot.direction[0], shot.direction[1], shot.direction[2]);
        let gun = gun(shot.gun);
        effects::muzzle(&mut commands, origin + forward * 0.7, forward);
        match gun.muzzle {
            Muzzle::Pellets => {
                let (right, up) = basis(forward);
                let mut tally: Vec<(Entity, f32)> = Vec::new();
                for pellet in 0..gun.pellets {
                    let seed = (index as u32) * 977 + pellet * 131 + shot.gun as u32 * 17;
                    let scatter = right * scatter_at(seed) + up * scatter_at(seed ^ 0x9e37_79b9);
                    let ray = (forward + scatter * gun.spread).normalize();
                    let wall =
                        physics_world_cast_ray(&physics, origin, ray, data::HITSCAN_RANGE, shooter);
                    let reach = wall
                        .as_ref()
                        .map(|hit| hit.distance)
                        .unwrap_or(data::HITSCAN_RANGE);
                    match nearest(&standing, origin, ray, reach) {
                        Some((entity, range)) => {
                            let drop = if gun.pellets > 1 { falloff(range) } else { 1.0 };
                            effects::blood(&mut commands, origin + ray * range);
                            add(&mut tally, entity, gun.damage * drop);
                        }
                        None => {
                            if let Some(hit) = wall {
                                effects::sparks(&mut commands, hit.point, hit.normal);
                                add(&mut tally, hit.entity, gun.damage);
                            }
                        }
                    }
                }
                apply(
                    &mut tally,
                    &mut standing,
                    &mut switches,
                    &mut walls,
                    &mut resources,
                    &mut events,
                    &mut commands,
                );
            }
            Muzzle::Rocket => {
                let bolt = spawn_bolt(
                    &mut commands,
                    origin + forward * 1.0,
                    forward * data::ROCKET_SPEED,
                    data::ROCKET_DIRECT,
                    data::ROCKET_RADIUS,
                    data::ROCKET_LIFE,
                    Vec4::new(1.0, 0.58, 0.24, 1.0),
                    0.42,
                );
                effects::rocket_look(&mut commands, bolt);
                commands.add_tag_type::<PlayerShot>(bolt);
            }
            Muzzle::Plasma => {
                let bolt = spawn_bolt(
                    &mut commands,
                    origin + forward * 0.9,
                    forward * data::PLASMA_SPEED,
                    data::PLASMA_DAMAGE,
                    0.0,
                    data::PLASMA_LIFE,
                    Vec4::new(0.34, 0.86, 1.0, 1.0),
                    0.26,
                );
                effects::bolt_look(&mut commands, bolt, Vec4::new(0.34, 0.86, 1.0, 1.0));
                commands.add_tag_type::<PlayerShot>(bolt);
            }
        }
    }

    for lob in lobs {
        let origin = Vec3::new(lob.origin[0], lob.origin[1], lob.origin[2]);
        let forward = Vec3::new(lob.direction[0], lob.direction[1], lob.direction[2]);
        effects::muzzle(&mut commands, origin, forward);
        let tint = if lob.splash > 0.0 {
            Vec4::new(1.0, 0.46, 0.20, 1.0)
        } else {
            Vec4::new(1.0, 0.30, 0.36, 1.0)
        };
        let bolt = spawn_bolt(
            &mut commands,
            origin,
            forward * lob.speed,
            lob.damage,
            lob.splash,
            data::ROCKET_LIFE,
            tint,
            if lob.splash > 0.0 { 0.36 } else { 0.24 },
        );
        effects::bolt_look(&mut commands, bolt, tint);
        if lob.fall > 0.0 {
            commands.set(bolt, Fall(lob.fall));
        }
        commands.add_tag_type::<EnemyShot>(bolt);
    }
}

fn spawn_bolt(
    commands: &mut Commands,
    at: Vec3,
    velocity: Vec3,
    damage: f32,
    splash: f32,
    life: f32,
    tint: Vec4,
    size: f32,
) -> Entity {
    let bolt = spawn_mesh_at(commands, "Sphere", at, Vec3::new(size, size, size));
    paint(
        commands,
        bolt,
        &format!(
            "boomstick_bolt_{}_{}_{}",
            (tint.x * 100.0) as u32,
            (tint.y * 100.0) as u32,
            (tint.z * 100.0) as u32
        ),
        Material {
            base_color: [tint.x, tint.y, tint.z, 1.0],
            emissive_factor: [tint.x * 3.0, tint.y * 3.0, tint.z * 3.0],
            roughness: 0.3,
            metallic: 0.0,
            ..Default::default()
        },
    );
    commands.set(bolt, Velocity(velocity));
    commands.set(bolt, Damage(damage));
    commands.set(bolt, Splash(splash));
    commands.set(bolt, Life(life));
    commands.add_tag_type::<LevelPiece>(bolt);
    bolt
}

fn apply(
    tally: &mut Vec<(Entity, f32)>,
    standing: &mut [Standing],
    switches: &mut [(Entity, f32, u32)],
    walls: &mut [(Entity, f32)],
    resources: &mut BoomstickResources,
    events: &mut EventWriter<BoomstickEvent>,
    commands: &mut Commands,
) {
    for (entity, damage) in tally.drain(..) {
        if let Some(target) = standing.iter_mut().find(|held| held.entity == entity) {
            if target.health <= 0.0 {
                continue;
            }
            target.health -= damage;
            let killed = target.health <= 0.0;
            let at = target.center();
            wound(commands, entity, target.health, damage >= data::REEL_DAMAGE);
            events.send(BoomstickEvent::EnemyHit { damage });
            if killed {
                effects::gibs(commands, at);
                events.send(BoomstickEvent::EnemyKilled);
            }
            continue;
        }

        if let Some(lever) = switches.iter_mut().find(|(held, _, _)| *held == entity) {
            if lever.1 <= 0.0 {
                continue;
            }
            lever.1 = 0.0;
            let group = lever.2;
            commands.queue(move |storage, _| {
                if storage.has::<Health>(entity) {
                    storage.set(entity, Health(0.0));
                }
            });
            if !resources.mission.thrown.contains(&group) {
                resources.mission.thrown.push(group);
            }
            events.send(BoomstickEvent::SwitchThrown);
            continue;
        }

        if let Some(wall) = walls.iter_mut().find(|(held, _)| *held == entity) {
            if wall.1 <= 0.0 {
                continue;
            }
            wall.1 = 0.0;
            commands.despawn_recursive(entity);
            resources.mission.secrets += 1;
            events.send(BoomstickEvent::SecretFound);
        }
    }
}

fn add(tally: &mut Vec<(Entity, f32)>, entity: Entity, damage: f32) {
    match tally.iter_mut().find(|(held, _)| *held == entity) {
        Some((_, total)) => *total += damage,
        None => tally.push((entity, damage)),
    }
}

fn nearest(standing: &[Standing], origin: Vec3, ray: Vec3, limit: f32) -> Option<(Entity, f32)> {
    let mut best: Option<(Entity, f32)> = None;
    for target in standing {
        if target.health <= 0.0 {
            continue;
        }
        let Some(range) = ray_hits_column(
            origin,
            ray,
            target.foot,
            target.height,
            target.radius,
            limit,
        ) else {
            continue;
        };
        if best.map(|(_, held)| range < held).unwrap_or(true) {
            best = Some((target.entity, range));
        }
    }
    best
}

fn basis(forward: Vec3) -> (Vec3, Vec3) {
    let reference = if forward.y.abs() > 0.95 {
        Vec3::z()
    } else {
        Vec3::y()
    };
    let right = forward.cross(&reference).normalize();
    let up = right.cross(&forward).normalize();
    (right, up)
}

fn scatter_at(seed: u32) -> f32 {
    fastrand::Rng::with_seed(seed as u64).f32() * 2.0 - 1.0
}
