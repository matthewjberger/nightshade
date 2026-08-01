use crate::commands::{effects, spawn};
use crate::components::{
    Cooldown, Damage, Focus, Health, Kind, Leg, Level, Progress, Reach, Reload, Rising,
};
use crate::contract::Tower;
use crate::queries;
use crate::resources::Bulwark;
use crate::theme;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::Sprite;

struct Mark {
    entity: Entity,
    at: Vec2,
    walked: f32,
    health: f32,
}

pub type Posts<'world, 'item> = Query<
    'world,
    (
        &'item Kind,
        &'item Reach,
        &'item Damage,
        &'item Reload,
        &'item Focus,
        &'item mut Cooldown,
        &'item LocalTransform,
    ),
>;

pub fn pick_and_fire(
    time: Res<Time>,
    bulwark: Res<Bulwark>,
    mut commands: Commands,
    crystals: Query<(&Health, &Leg, &Progress, &LocalTransform)>,
    towers: Posts,
) {
    let mut marks: Vec<Mark> = Vec::new();
    crystals.for_each(|entity, (health, leg, progress, transform)| {
        if health.0 > 0.0 {
            marks.push(Mark {
                entity,
                at: Vec2::new(transform.translation.x, transform.translation.y),
                walked: leg.0 as f32 + progress.0,
                health: health.0,
            });
        }
    });

    let delta_time = time.delta_time * queries::pace(&bulwark);
    let art = bulwark.art;
    towers.for_each(
        |entity, (kind, reach, damage, reload, focus, cooldown, transform)| {
            cooldown.0 -= delta_time;
            if cooldown.0 > 0.0 || marks.is_empty() {
                return;
            }
            let post = Vec2::new(transform.translation.x, transform.translation.y);
            let inside = marks
                .iter()
                .filter(|mark| (mark.at - post).magnitude() <= reach.0);
            let chosen = match focus.0 {
                1 => inside.max_by(|first, second| first.walked.total_cmp(&second.walked)),
                2 => inside.max_by(|first, second| first.health.total_cmp(&second.health)),
                _ => inside.min_by(|first, second| {
                    (first.at - post)
                        .magnitude()
                        .total_cmp(&(second.at - post).magnitude())
                }),
            };
            let Some(mark) = chosen else {
                return;
            };
            cooldown.0 = reload.0;
            let chills = if kind.0 == Tower::Hearth {
                theme::CHILL_SECONDS
            } else {
                0.0
            };
            let splash = if kind.0 == Tower::Bastion {
                theme::SPLASH_REACH
            } else {
                0.0
            };
            let pierce = if kind.0 == Tower::Barbican {
                theme::PIERCE_SHARE
            } else {
                0.0
            };
            spawn::loose_a_bead(
                &mut commands,
                &art,
                entity,
                post,
                mark.entity,
                damage.0,
                chills,
                splash,
                pierce,
            );
            effects::tower_fires(
                &mut commands,
                Vec3::new(post.x, post.y + theme::BEAD_LIFT, 0.0),
            );
        },
    );
}

pub fn settle_the_stone(
    time: Res<Time>,
    bulwark: Res<Bulwark>,
    mut commands: Commands,
    raised: Query<(&Kind, &Level, &mut Rising, &Sprite)>,
) {
    let delta_time = time.delta_time;
    let art = bulwark.art;
    raised.for_each(|entity, (kind, level, rising, sprite)| {
        if rising.0 <= 0.0 {
            return;
        }
        rising.0 = (rising.0 - delta_time).max(0.0);
        let piece = art.towers[kind.0.slot()][(level.0 as usize).saturating_sub(1).min(2)];
        let share = 1.0 - (rising.0 / theme::RISE_SECONDS).clamp(0.0, 1.0);
        let eased = theme::RISE_FROM + (1.0 - theme::RISE_FROM) * share * (2.0 - share);
        commands.set(
            entity,
            sprite
                .clone()
                .with_size(piece.size.x * eased, piece.size.y * eased),
        );
    });
}
