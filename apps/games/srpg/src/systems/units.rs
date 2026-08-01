use crate::commands::log;
use crate::commands::units::{spawn_fighter, spawn_recruit};
use crate::components::{
    Fill, Flash, FlashTint, Flipped, Hp, MaxHp, Owner, StatusIcon, Stunned, Suppressed,
};
use crate::data::{CHAPTERS, Facing, Status, Team};
use crate::queries::{Fighters, squad};
use crate::resources::{Battle, Battlefield, CameraRig, Campaign, Sheet, Terrain};
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;

pub fn spawn_the_squads(
    terrain: Res<Terrain>,
    sheet: Res<Sheet>,
    campaign: Res<Campaign>,
    mut battlefield: ResMut<Battlefield>,
    mut battle: ResMut<Battle>,
    mut commands: Commands,
    mut fighters: Fighters<'_, '_>,
) {
    if !battlefield.spawn_pending {
        return;
    }
    battlefield.spawn_pending = false;
    let squad = squad(&mut fighters);
    let chapter = &CHAPTERS[campaign.chapter.min(CHAPTERS.len() - 1)];
    let mut marched = 0;
    for (entity, fighter) in &squad {
        if !fighter.deployed || fighter.wounded > 0 {
            continue;
        }
        let home = chapter
            .plan
            .spots
            .get(marched)
            .copied()
            .unwrap_or((theme::TERRAIN_COLUMNS / 2, theme::TERRAIN_ROWS - 2));
        spawn_fighter(&mut commands, &terrain, &sheet, *entity, fighter, home);
        marched += 1;
    }
    for recruit in chapter.plan.squad {
        spawn_recruit(&mut commands, &terrain, &sheet, recruit, Team::Enemy);
    }
    log(
        &mut battle,
        format!("{}.  {} march out.", chapter.name, marched),
    );
    log(&mut battle, chapter.objective.detail());
}

pub fn face_the_camera(
    time: Res<Time>,
    rig: Res<CameraRig>,
    looks: Query<(&Facing, &mut Flipped, &mut Flash, &FlashTint, &mut Sprite)>,
) {
    let right = crate::queries::rig::right(&rig);
    let delta_time = time.delta_time;
    looks.for_each(|_entity, (facing, flipped, flash, flash_tint, sprite)| {
        let (column_delta, row_delta) = facing.delta();
        let along = right.x * column_delta as f32 + right.y * row_delta as f32;
        if along.abs() > 0.5 {
            flipped.0 = (along < 0.0) == theme::SPRITES_FACE_RIGHT;
        }
        sprite.flip_x = flipped.0;
        if flash.0 > 0.0 {
            flash.0 -= delta_time;
            sprite.color = flash_tint.0;
        } else if sprite.color != [1.0, 1.0, 1.0, 1.0] {
            sprite.color = [1.0, 1.0, 1.0, 1.0];
        }
    });
}

pub fn show_the_bars(
    battlefield: Res<Battlefield>,
    mut units: Query<(&Hp, &MaxHp, &Team)>,
    bars: Query<(
        &Owner,
        &Fill,
        &mut Sprite,
        &mut Visibility,
        &mut LocalTransform,
    )>,
) {
    let mut states = Vec::new();
    units.for_each_mut(|entity, (hp, max_hp, team)| {
        let share = if max_hp.0 > 0 {
            (hp.0 as f32 / max_hp.0 as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        states.push((entity, share, *team));
    });
    let shown = battlefield.bars_shown;
    bars.for_each(|_entity, (owner, fill, sprite, visibility, transform)| {
        let Some((_, share, team)) = states.iter().find(|(held, _, _)| *held == owner.0) else {
            return;
        };
        if visibility.visible != shown {
            visibility.visible = shown;
        }
        if !fill.0 {
            return;
        }
        sprite.size.x = theme::HEALTH_BAR_SIZE.x * share;
        sprite.color = if *team == Team::Enemy {
            theme::HEALTH_ENEMY
        } else if *share < 0.35 {
            theme::HEALTH_LOW
        } else {
            theme::HEALTH_GOOD
        };
        transform.translation.x = -theme::HEALTH_BAR_SIZE.x / 2.0;
    });
}

pub fn show_the_statuses(
    sheet: Res<Sheet>,
    mut units: Query<(&Stunned, &Suppressed)>,
    icons: Query<(&StatusIcon, &mut Sprite, &mut Visibility)>,
) {
    let mut states = Vec::new();
    units.for_each_mut(|entity, (stunned, suppressed)| {
        let status = if stunned.0 > 0 {
            Some(Status::Stunned)
        } else if suppressed.0 > 0 {
            Some(Status::Suppressed)
        } else {
            None
        };
        states.push((entity, status));
    });
    icons.for_each(|_entity, (icon, sprite, visibility)| {
        let Some((_, status)) = states.iter().find(|(owner, _)| *owner == icon.owner) else {
            return;
        };
        match status {
            Some(status) => {
                let region = crate::queries::sheet::tile(&sheet, status.icon());
                if sprite.region != region {
                    sprite.region = region;
                }
                if !visibility.visible {
                    visibility.visible = true;
                }
            }
            None => {
                if visibility.visible {
                    visibility.visible = false;
                }
            }
        }
    });
}
