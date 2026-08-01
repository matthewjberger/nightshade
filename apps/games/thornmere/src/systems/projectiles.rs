use crate::commands::audio::cue_one_of;
use crate::commands::effects::burst;
use crate::commands::record_hit;
use crate::commands::spells::{chill_aura, fire_blast, frost_shatter};
use crate::components::{Carried, Health, Projectile, Target};
use crate::data::{HIT_CLIPS, PROJECTILE_HEIGHT, PROJECTILE_SPEED, SOUND_PLAIN};
use crate::queries::is_alive;
use crate::resources::{BurstKind, Bursts, Cues, Hits, ThornmereResources};
use nightshade::prelude::*;
use nightshade_particles::contract::ParticleEmitter;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;
use std::collections::HashMap;

pub fn fly(
    time: Res<Time>,
    mut resources: ResMut<ThornmereResources>,
    mut hits: ResMut<Hits>,
    mut bursts: ResMut<Bursts>,
    mut cues: ResMut<Cues>,
    mut commands: Commands,
    victims: Query<(&Health, &LocalTransform), With<Target>>,
    mut shots: Query<(
        &Projectile,
        &Carried,
        &mut LocalTransform,
        &mut ParticleEmitter,
    )>,
) {
    let delta_time = time.delta_time;
    let mut spots: HashMap<Entity, (bool, Vec3)> = HashMap::new();
    victims.for_each(|entity, (health, transform)| {
        spots.insert(entity, (is_alive(health.0), transform.translation));
    });

    let mut finished = Vec::new();
    shots.for_each_mut(|entity, (projectile, carried, transform, emitter)| {
        let Some(target) = projectile.0 else {
            finished.push((entity, None));
            return;
        };
        let Some(&(alive, spot)) = spots.get(&target) else {
            finished.push((entity, None));
            return;
        };
        let aim = spot + Vec3::new(0.0, PROJECTILE_HEIGHT, 0.0);
        let to_aim = aim - transform.translation;
        let distance = to_aim.magnitude();
        let step = PROJECTILE_SPEED * delta_time;
        if distance <= step + 0.4 {
            transform.translation = aim;
            emitter.position = aim;
            finished.push((entity, Some((carried.0, alive, aim))));
            return;
        }
        transform.translation += to_aim / distance * step;
        emitter.position = transform.translation;
    });

    for (entity, landing) in finished {
        if let Some((hit, alive, aim)) = landing {
            let kind = if hit.chill {
                BurstKind::FrostHit
            } else {
                BurstKind::Fire
            };
            burst(&mut bursts, kind, aim);
            if hit.chill {
                frost_shatter(
                    &mut commands,
                    &resources.palette.clone(),
                    &mut resources.random,
                    aim,
                );
            } else {
                fire_blast(&mut commands, &resources.palette.clone(), aim);
            }
            if alive {
                if hit.chill {
                    chill_aura(&mut commands, hit.target);
                }
                record_hit(
                    &mut hits,
                    hit.attacker,
                    hit.target,
                    hit.damage,
                    hit.crit,
                    hit.chill,
                    hit.source,
                );
                cue_one_of(&mut cues, &mut resources.random, &HIT_CLIPS, SOUND_PLAIN);
            }
        }
        commands.despawn_recursive(entity);
    }
}
