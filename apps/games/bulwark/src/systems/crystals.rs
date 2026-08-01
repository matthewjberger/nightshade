use nightshade_waves::WaveProgress;
use nightshade_waves::commands::one_fell;

use crate::commands::art::sprite_of;
use crate::commands::{effects, purse};
use crate::components::{
    Backing, Bar, Bob, Chill, Flash, Health, Leg, MaxHealth, Pace, Progress, Shade, Speed, Tint,
};
use crate::queries;
use crate::resources::Bulwark;
use crate::theme;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::Sprite;

pub type Walkers<'world, 'item> = Query<
    'world,
    (
        &'item Speed,
        &'item Bob,
        &'item Bar,
        &'item Backing,
        &'item Shade,
        &'item mut Leg,
        &'item mut Progress,
        &'item mut LocalTransform,
    ),
>;

pub type Dressed<'world, 'item> = Query<
    'world,
    (
        &'item Health,
        &'item MaxHealth,
        &'item Sprite,
        &'item Bar,
        &'item Backing,
        &'item LocalTransform,
    ),
>;

pub fn walk_the_road(
    time: Res<Time>,
    mut bulwark: ResMut<Bulwark>,
    mut waves: ResMut<WaveProgress>,
    mut commands: Commands,
    walkers: Walkers,
) {
    let delta_time = time.delta_time * queries::pace(&bulwark);
    let uptime = time.uptime_milliseconds as f32 * 0.001;
    let board = bulwark.board;
    let final_leg = queries::last_leg(board);
    let art = bulwark.art;

    walkers.for_each(
        |entity, (speed, bob, bar, backing, shade, leg, progress, transform)| {
            let mut walked = progress.0 + speed.0 * delta_time / queries::leg_length(board, leg.0);
            let mut current = leg.0;
            let mut arrived = false;
            while walked >= 1.0 {
                current += 1;
                walked -= 1.0;
                if current >= final_leg {
                    arrived = true;
                    walked = 0.0;
                    break;
                }
            }

            if arrived {
                purse::lose_a_life(&mut bulwark);
                one_fell(&mut waves);
                bulwark.shake = theme::SHAKE_SECONDS;
                effects::keep_is_struck(&mut commands, transform.translation);
                if let Some(keep) = bulwark.keep {
                    commands.set(keep, Flash(theme::BREACH_FLASH));
                    commands.set(
                        keep,
                        sprite_of(art.keep, theme::LAYER_STANDING).with_color(theme::BREACH_TINT),
                    );
                }
                commands.despawn(bar.0);
                commands.despawn(backing.0);
                commands.despawn(shade.0);
                commands.despawn(entity);
                return;
            }

            leg.0 = current;
            progress.0 = walked;
            let center = queries::leg_center(board, current, walked);
            let lift = (uptime * theme::BOB_RATE + bob.0).sin().abs() * theme::BOB_HEIGHT;
            transform.translation = Vec3::new(center.x, center.y + lift, 0.0);
            place(&mut commands, shade.0, Vec3::new(center.x, center.y, 0.0));
        },
    );
}

pub fn cool_the_flash(
    time: Res<Time>,
    bulwark: Res<Bulwark>,
    mut commands: Commands,
    hurt: Query<(&mut Flash, &Sprite, Option<&Tint>)>,
) {
    let delta_time = time.delta_time * queries::pace(&bulwark);
    hurt.for_each(|entity, (flash, sprite, tint)| {
        if flash.0 <= 0.0 {
            return;
        }
        flash.0 -= delta_time;
        if flash.0 > 0.0 {
            return;
        }
        let plain = tint.map_or(theme::SHARD_COLOR, |carried| carried.0);
        commands.set(entity, sprite.clone().with_color(plain));
    });
}

pub fn dress_the_crystals(bulwark: Res<Bulwark>, mut commands: Commands, crystals: Dressed) {
    let piece = bulwark.art.bar;
    crystals.for_each(
        |_entity, (health, max_health, sprite, bar, backing, transform)| {
            let share = (health.0 / max_health.0.max(f32::EPSILON)).clamp(0.0, 1.0);
            let foot = Vec2::new(transform.translation.x, transform.translation.y);
            let crown = foot.y + sprite.size.y * (1.0 - sprite.anchor.y) + theme::BAR_LIFT;
            let left = foot.x - theme::BAR_WIDTH * 0.5;
            let at = Vec3::new(left, crown, 0.0);
            let plain = Sprite::new(theme::ATLAS, piece.region)
                .with_anchor(piece.anchor)
                .with_layer(theme::LAYER_BEAD);

            place(&mut commands, backing.0, at);
            commands.set(
                backing.0,
                plain
                    .clone()
                    .with_size(theme::BAR_WIDTH, theme::BAR_HEIGHT)
                    .with_color(theme::BAR_BACK_COLOR),
            );
            place(&mut commands, bar.0, at);
            commands.set(
                bar.0,
                plain
                    .with_size(theme::BAR_WIDTH * share, theme::BAR_HEIGHT)
                    .with_color(queries::bar_color(share)),
            );
        },
    );
}

fn place(commands: &mut Commands, entity: Entity, at: Vec3) {
    commands.set(
        entity,
        LocalTransform {
            translation: at,
            ..Default::default()
        },
    );
}

pub fn chill_the_crystals(
    time: Res<Time>,
    bulwark: Res<Bulwark>,
    mut commands: Commands,
    crystals: Query<(&mut Chill, &Pace, &mut Speed, &Flash, &Sprite, &Tint)>,
) {
    let delta_time = time.delta_time * queries::pace(&bulwark);
    crystals.for_each(|entity, (chill, pace, speed, flash, sprite, tint)| {
        if chill.0 <= 0.0 {
            return;
        }
        chill.0 = (chill.0 - delta_time).max(0.0);
        let cold = chill.0 > 0.0;
        speed.0 = if cold {
            pace.0 * theme::CHILL_SHARE
        } else {
            pace.0
        };
        if flash.0 > 0.0 {
            return;
        }
        let color = if cold { theme::CHILL_COLOR } else { tint.0 };
        commands.set(entity, sprite.clone().with_color(color));
    });
}
