use crate::commands::{effects, purse, spawn};
use crate::components::{
    Aim, Backing, Bar, Bounty, Chill, Chills, Cut, Damage, Flash, Health, Leg, MaxHealth, Owner,
    Pace, Pierce, Progress, Shade, Speed, Splash, Splits, Tally, Ward,
};
use crate::data::Walker;
use crate::resources::Bulwark;
use crate::theme;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::Sprite;
use nightshade_waves::WaveProgress;
use nightshade_waves::commands::one_fell;

fn strike(
    storage: &mut nightshade::ecs::Storage,
    commands: &mut Commands,
    bulwark: &mut Bulwark,
    progress: &mut WaveProgress,
    post: Option<Entity>,
    mark: Entity,
    amount: f32,
    pierce: f32,
) {
    let Some(health) = storage.get::<Health>(mark).copied() else {
        return;
    };
    if health.0 <= 0.0 {
        return;
    }
    let ward = storage.get::<Ward>(mark).copied().unwrap_or(Ward(0.0));
    let cold = storage
        .get::<Chill>(mark)
        .is_some_and(|carried| carried.0 > 0.0);
    let chilled = if cold {
        ward.0 * theme::CHILL_WARD_SHARE
    } else {
        ward.0
    };
    let held = chilled * (1.0 - pierce).clamp(0.0, 1.0);
    let through = (amount - held).max(amount * theme::WARD_LEAST_SHARE);
    let left = health.0 - through;
    if left > 0.0 {
        if let Some(carried) = storage.get_mut::<Health>(mark) {
            carried.0 = left;
        }
        if let Some(flash) = storage.get_mut::<Flash>(mark) {
            flash.0 = theme::FLASH_SECONDS;
        }
        if let Some(sprite) = storage.get_mut::<Sprite>(mark) {
            sprite.color = theme::FLASH_COLOR;
        }
        return;
    }

    if let Some(carried) = storage.get_mut::<Health>(mark) {
        carried.0 = 0.0;
    }
    let at = storage
        .get::<LocalTransform>(mark)
        .map(|transform| transform.translation)
        .unwrap_or_default();
    let bounty = storage.get::<Bounty>(mark).copied().unwrap_or(Bounty(0));
    if let Some(bar) = storage.get::<Bar>(mark).copied() {
        commands.despawn(bar.0);
    }
    if let Some(backing) = storage.get::<Backing>(mark).copied() {
        commands.despawn(backing.0);
    }
    if let Some(shade) = storage.get::<Shade>(mark).copied() {
        commands.despawn(shade.0);
    }
    shatter(storage, commands, bulwark, progress, mark);
    commands.despawn(mark);
    if let Some(credited) = post
        && let Some(tally) = storage.get_mut::<Tally>(credited)
    {
        tally.0 += 1;
    }
    purse::pay_the_bounty(bulwark, bounty.0);
    one_fell(progress);
    effects::crystal_breaks(commands, at);
}

fn shatter(
    storage: &mut nightshade::ecs::Storage,
    commands: &mut Commands,
    bulwark: &mut Bulwark,
    progress: &mut WaveProgress,
    mark: Entity,
) {
    let splits = storage.get::<Splits>(mark).copied().unwrap_or(Splits(0));
    if splits.0 == 0 {
        return;
    }
    let Some(cut) = storage.get::<Cut>(mark).copied() else {
        return;
    };
    let whole = storage
        .get::<MaxHealth>(mark)
        .copied()
        .unwrap_or(MaxHealth(0.0));
    let pace = storage.get::<Pace>(mark).copied().unwrap_or(Pace(1.0));
    let bounty = storage.get::<Bounty>(mark).copied().unwrap_or(Bounty(0));
    let ward = storage.get::<Ward>(mark).copied().unwrap_or(Ward(0.0));
    let leg = storage.get::<Leg>(mark).copied().unwrap_or(Leg(0));
    let walked = storage
        .get::<Progress>(mark)
        .copied()
        .unwrap_or(Progress(0.0));

    let child = Walker {
        tier: cut.0.lesser(),
        health: whole.0 * theme::SPLIT_HEALTH_SHARE,
        speed: pace.0 * theme::SPLIT_SPEED_SHARE,
        bounty: (bounty.0 / 2).max(1),
        ward: ward.0 * theme::SPLIT_WARD_SHARE,
        splits: splits.0 - 1,
    };
    let art = bulwark.art;
    let board = bulwark.board;
    for index in 0..theme::SPLIT_COUNT {
        let offset = (index as f32 - (theme::SPLIT_COUNT as f32 - 1.0) * 0.5) * theme::SPLIT_STEP;
        let progress = (walked.0 + offset).clamp(0.0, 0.97);
        spawn::send_a_crystal(commands, &art, board, child, (leg.0, progress), false);
    }
    progress.alive += theme::SPLIT_COUNT;
}

pub fn fly_and_strike(
    time: Res<Time>,
    mut bulwark: ResMut<Bulwark>,
    mut progress: ResMut<WaveProgress>,
    mut commands: Commands,
    storage: &mut nightshade::ecs::Storage,
) {
    let delta_time = time.delta_time * crate::queries::pace(&bulwark);
    let progress = &mut *progress;
    let beads: Vec<Entity> = storage.query_entities::<(Aim, Damage, Speed)>().collect();

    for bead in beads {
        let Some(aim) = storage.get::<Aim>(bead).copied() else {
            continue;
        };
        let Some(damage) = storage.get::<Damage>(bead).copied() else {
            continue;
        };
        let Some(speed) = storage.get::<Speed>(bead).copied() else {
            continue;
        };
        let Some(health) = storage.get::<Health>(aim.0).copied() else {
            commands.despawn(bead);
            continue;
        };
        if health.0 <= 0.0 {
            commands.despawn(bead);
            continue;
        }
        let Some(mark) = storage
            .get::<LocalTransform>(aim.0)
            .map(|transform| transform.translation)
        else {
            commands.despawn(bead);
            continue;
        };
        let Some(at) = storage
            .get::<LocalTransform>(bead)
            .map(|transform| transform.translation)
        else {
            continue;
        };

        let toward = Vec2::new(mark.x - at.x, mark.y + theme::BEAD_LIFT - at.y);
        let gap = toward.magnitude();
        let step = speed.0 * delta_time;
        if gap > step && gap > f32::EPSILON {
            let moved = Vec2::new(at.x, at.y) + toward / gap * step;
            if let Some(transform) = storage.get_mut::<LocalTransform>(bead) {
                transform.translation = Vec3::new(moved.x, moved.y, 0.0);
            }
            continue;
        }

        commands.despawn(bead);

        let chills = storage.get::<Chills>(bead).copied().unwrap_or(Chills(0.0));
        if chills.0 > 0.0
            && let Some(carried) = storage.get_mut::<Chill>(aim.0)
        {
            carried.0 = carried.0.max(chills.0);
        }

        let post = storage.get::<Owner>(bead).copied().map(|owner| owner.0);
        let pierce = storage.get::<Pierce>(bead).copied().unwrap_or(Pierce(0.0));
        let splash = storage.get::<Splash>(bead).copied().unwrap_or(Splash(0.0));
        if splash.0 > 0.0 {
            effects::splash_lands(&mut commands, mark);
            let others: Vec<Entity> = storage
                .query_entities::<(Health, Bounty)>()
                .filter(|other| *other != aim.0)
                .collect();
            for other in others {
                let Some(near) = storage
                    .get::<LocalTransform>(other)
                    .map(|transform| transform.translation)
                else {
                    continue;
                };
                let reach = Vec2::new(near.x - mark.x, near.y - mark.y).magnitude();
                if reach > splash.0 {
                    continue;
                }
                strike(
                    storage,
                    &mut commands,
                    &mut bulwark,
                    progress,
                    post,
                    other,
                    damage.0 * theme::SPLASH_SHARE,
                    pierce.0,
                );
            }
        }

        strike(
            storage,
            &mut commands,
            &mut bulwark,
            progress,
            post,
            aim.0,
            damage.0,
            pierce.0,
        );
    }
}
