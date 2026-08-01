use crate::commands::interface::say;
use crate::components::{
    Ammo, Combo, Flash, Fling, Flying, Heals, Health, Holding, Kind, Pose, Stun, Weapon,
};
use crate::data;
use crate::resources::{Ring, Talk};
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn gather(
    ring: Res<Ring>,
    mut talk: ResMut<Talk>,
    mut commands: Commands,
    foods: Query<(&Heals, &Kind, &LocalTransform)>,
    weapons: Query<(&Weapon, &LocalTransform), Without<Flying>>,
    heroes: Query<(&mut Health, &mut Holding, &mut Ammo, &Combo)>,
) {
    let feet = ring.feet;
    let mut healed = 0.0;
    let mut eaten = None;
    let mut taken = None;
    foods.for_each(|entity, (heals, kind, transform)| {
        if !near(feet, transform.translation) {
            return;
        }
        healed += heals.0;
        eaten = Some(kind.0);
        commands.despawn_recursive(entity);
    });
    weapons.for_each(|entity, (weapon, transform)| {
        if taken.is_some() || !near(feet, transform.translation) {
            return;
        }
        taken = Some((entity, weapon.0));
    });

    let mut picked = None;
    heroes.for_each(|_entity, (health, holding, ammo, _)| {
        if healed > 0.0 {
            health.0 = (health.0 + healed).min(data::HERO_HEALTH);
        }
        let Some((entity, kind)) = taken else {
            return;
        };
        if holding.0 != 0 {
            return;
        }
        holding.0 = kind + 1;
        ammo.0 = data::WEAPON_KINDS[kind as usize].swings;
        picked = Some((entity, kind));
    });

    if let Some(kind) = eaten {
        say(&mut talk, data::PICKUP_KINDS[kind as usize].line);
    }
    if let Some((entity, kind)) = picked {
        commands.despawn_recursive(entity);
        say(&mut talk, data::WEAPON_KINDS[kind as usize].taken);
    }
}

pub fn fly(
    time: Res<Time>,
    mut ring: ResMut<Ring>,
    mut commands: Commands,
    shots: Query<(&mut Flying, &mut LocalTransform, &Weapon)>,
    thugs: Struck,
) {
    let delta_time = if ring.hit_stop > 0.0 {
        0.0
    } else {
        time.delta_time
    };
    let mut live: Vec<(Entity, Vec3, f32)> = Vec::new();
    shots.for_each(|entity, (flying, transform, _)| {
        transform.translation.x += flying.0 * delta_time;
        let out = transform.translation.x < data::STREET_START - 2.0
            || transform.translation.x > data::STREET_END + 2.0;
        if out {
            commands.despawn_recursive(entity);
            return;
        }
        live.push((entity, transform.translation, flying.0.signum()));
    });

    let mut spent = Vec::new();
    let mut score = 0;
    thugs.for_each(
        |_entity, (health, stun, pose, fling, flash, transform, kind)| {
            if health.0 <= 0.0 {
                return;
            }
            let plan = &data::THUG_KINDS[kind.0 as usize];
            let Some(shot) = live.iter().find(|(entity, at, _)| {
                !spent.contains(entity)
                    && (at.x - transform.translation.x).abs() < data::THROW_REACH + plan.radius
                    && (at.z - transform.translation.z).abs() < data::LANE_REACH
            }) else {
                return;
            };
            let (entity, _, facing) = *shot;
            spent.push(entity);
            health.0 = (health.0 - data::THROW_DAMAGE).max(0.0);
            stun.0 = data::DOWN_TIME;
            pose.0 = data::POSE_DOWN;
            flash.0 = data::FLASH_TIME;
            fling.0 = facing * data::FLING_SPEED * 0.6;
            if health.0 <= 0.0 {
                score += plan.score;
            }
        },
    );

    for entity in spent {
        commands.despawn_recursive(entity);
    }
    if score > 0 {
        ring.score += score;
        ring.shake = ring.shake.max(data::SHAKE_HIT);
    }
}

fn near(feet: Vec3, at: Vec3) -> bool {
    (feet.x - at.x).abs() < data::PICKUP_REACH && (feet.z - at.z).abs() < data::PICKUP_REACH
}

pub type Struck<'world, 'item> = Query<
    'world,
    (
        &'item mut Health,
        &'item mut Stun,
        &'item mut Pose,
        &'item mut Fling,
        &'item mut Flash,
        &'item LocalTransform,
        &'item Kind,
    ),
    Without<Combo>,
>;
