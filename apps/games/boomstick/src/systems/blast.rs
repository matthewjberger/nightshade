use crate::commands::{hurt, wound};
use crate::components::{EnemyKind, EnemyMood, Health};
use crate::contract::{BoomstickEvent, FireQueue};
use crate::data;
use crate::queries::hit_center;
use crate::resources::BoomstickResources;
use crate::systems::effects;
use crate::tags::Player;
use nightshade::prelude::*;
use nightshade_physics::contract::components::CharacterControllerComponent;
use nightshade_scene::prelude::*;

pub fn resolve(
    mut queue: ResMut<FireQueue>,
    mut resources: ResMut<BoomstickResources>,
    mut events: EventWriter<BoomstickEvent>,
    mut commands: Commands,
    enemies: Query<(&EnemyKind, &EnemyMood, &Health, &LocalTransform)>,
    mut bodies: Query<(&LocalTransform, &mut CharacterControllerComponent), WithTag<Player>>,
) {
    let blasts = std::mem::take(&mut queue.blasts);
    if blasts.is_empty() {
        return;
    }

    let mut standing: Vec<(Entity, Vec3, f32)> = Vec::new();
    enemies.for_each(|entity, (kind, mood, health, transform)| {
        if *mood != EnemyMood::Dying {
            standing.push((
                entity,
                transform.translation + Vec3::new(0.0, hit_center(*kind), 0.0),
                health.0,
            ));
        }
    });

    let mut struck = 0.0;
    let mut shove = Vec3::zeros();

    for blast in blasts {
        let at = Vec3::new(blast.at[0], blast.at[1], blast.at[2]);
        effects::explosion(&mut commands, at, blast.radius);

        for entry in standing.iter_mut() {
            if entry.2 <= 0.0 {
                continue;
            }
            let reach = (entry.1 - at).magnitude();
            if reach > blast.radius {
                continue;
            }
            let share = 1.0 - (reach / blast.radius).clamp(0.0, 1.0);
            let damage = blast.damage * share;
            entry.2 -= damage;
            let killed = entry.2 <= 0.0;
            let center = entry.1;
            wound(&mut commands, entry.0, entry.2, damage >= data::REEL_DAMAGE);
            events.send(BoomstickEvent::EnemyHit { damage });
            if killed {
                effects::gibs(&mut commands, center);
                events.send(BoomstickEvent::EnemyKilled);
            }
        }

        let Some(player_at) = resources.player.at else {
            continue;
        };
        let away = player_at - at;
        let reach = away.magnitude();
        if reach > blast.radius {
            continue;
        }
        let share = 1.0 - (reach / blast.radius).clamp(0.0, 1.0);
        let owned = if blast.from_player {
            data::BLAST_SELF_SHARE
        } else {
            1.0
        };
        struck += blast.damage * share * owned;
        let lift = if reach < 0.001 {
            Vec3::y()
        } else {
            (away / reach + Vec3::new(0.0, 0.6, 0.0)).normalize()
        };
        shove += lift * data::BLAST_PUSH * share;
    }

    if struck > 0.0 && !resources.player.dead {
        hurt(&mut resources.player, struck);
        resources.player.flash = 1.0;
        resources.player.kick += data::HURT_KICK;
        events.send(BoomstickEvent::PlayerHurt { damage: struck });
    }

    if let Some(body) = resources.player.body
        && shove.magnitude() > 0.0
    {
        bodies.for_entity(body, |(_, controller)| {
            controller.velocity += shove;
        });
    }
}
