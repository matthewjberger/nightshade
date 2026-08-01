use crate::commands::audio::play;
use crate::commands::digits::spawn_digits;
use crate::commands::effects;
use crate::commands::{clear_highlights, edit_unit, log, look_at, set_phase, show_area};
use crate::data::{Ability, Team};
use crate::queries::{
    Structures, Units, affected, area_tiles, can_counter, facing_toward, predict, roster,
    roster_index, sites,
};
use crate::resources::{Battle, CameraRig, Cursor, Highlights, Hit, Phase, Sheet, Terrain};
use crate::theme;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn choose_the_target(
    mut battle: ResMut<Battle>,
    mut cursor: ResMut<Cursor>,
    terrain: Res<Terrain>,
    mut highlights: ResMut<Highlights>,
    mut camera_rig: ResMut<CameraRig>,
    mut units: Units<'_, '_>,
) {
    if battle.phase != Phase::ChooseTarget {
        return;
    }
    let (active, ability) = { (battle.active, battle.ability) };
    let (Some(active), Some(ability)) = (active, ability) else {
        set_phase(&mut battle, Phase::Menu);
        return;
    };
    let roster = roster(&mut units);
    let Some(index) = roster_index(&roster, active) else {
        return;
    };
    let unit = roster[index].1.clone();
    let here = (unit.column, unit.row);
    let (cursor_tile, confirm, cancel) =
        { ((cursor.column, cursor.row), cursor.confirm, cursor.cancel) };
    let in_range = battle.targets.contains(&cursor_tile);
    let area = if in_range {
        area_tiles(&terrain, cursor_tile, ability.shape().area)
    } else {
        Vec::new()
    };
    show_area(&mut highlights, area.clone());

    if cancel {
        clear_highlights(&mut highlights);
        let stand = crate::queries::terrain::stand(&terrain, here.0, here.1);
        cursor.column = here.0;
        cursor.row = here.1;
        look_at(&mut camera_rig, stand);
        let back = if ability == Ability::Attack {
            Phase::Menu
        } else {
            Phase::ChooseAbility
        };
        set_phase(&mut battle, back);
        return;
    }
    if !confirm || !in_range {
        return;
    }
    let touched = affected(&roster, &area, &unit, ability);
    if touched.is_empty() {
        log(&mut battle, "Nothing to hit there.");
        return;
    }
    clear_highlights(&mut highlights);
    battle.target_tile = Some(cursor_tile);
    battle.acted = true;
    set_phase(&mut battle, Phase::Acting);
}

pub fn resolve_the_action(
    time: Res<Time>,
    terrain: Res<Terrain>,
    sheet: Res<Sheet>,
    mut battle: ResMut<Battle>,
    mut rig: ResMut<CameraRig>,
    mut cursor: ResMut<Cursor>,
    mut commands: Commands,
    mut units: Units<'_, '_>,
    mut transforms: Query<(&mut LocalTransform,)>,
    mut structures: Structures<'_, '_>,
) {
    if battle.phase != Phase::Acting {
        return;
    }
    let (Some(active), Some(ability), Some(target_tile)) =
        (battle.active, battle.ability, battle.target_tile)
    else {
        set_phase(&mut battle, Phase::TurnEnd);
        return;
    };
    let roster = roster(&mut units);
    let Some(index) = roster_index(&roster, active) else {
        set_phase(&mut battle, Phase::TurnEnd);
        return;
    };
    let attacker = roster[index].1.clone();
    let here = (attacker.column, attacker.row);
    let shape = ability.shape();

    if !battle.hits_rolled {
        battle.hits_rolled = true;
        let sites = sites(&mut structures);
        let area = area_tiles(&terrain, target_tile, shape.area);
        let touched = affected(&roster, &area, &attacker, ability);
        let mut rng = battle.rng.clone();
        let mut hits = Vec::new();
        for target_index in touched {
            let (entity, target) = &roster[target_index];
            let strike = predict(&terrain, &sites, &roster, &attacker, here, target, ability);
            let variance = 0.9 + 0.2 * rng.f32();
            let amount = ((strike.amount as f32) * variance).round().max(1.0) as i32;
            let landed = ability.helps() || (rng.f32() * 100.0) < strike.hit_chance as f32;
            hits.push(Hit {
                entity: *entity,
                amount,
                landed,
                heals: ability.helps(),
            });
        }

        battle.hits = hits;
        let facing = facing_toward(here, target_tile);
        edit_unit(&mut units, active, |unit| {
            unit.mp = (unit.mp - shape.mp).max(0);
            if let Some(facing) = facing {
                unit.facing = facing;
            }
        });
        log(
            &mut battle,
            format!("{} uses {}.", attacker.name, ability.name()),
        );
        look_at(
            &mut rig,
            crate::queries::terrain::stand(&terrain, target_tile.0, target_tile.1),
        );
        cursor.column = target_tile.0;
        cursor.row = target_tile.1;
    }

    battle.timer += time.delta_time;
    let timer = battle.timer;

    let hover = Vec3::new(0.0, attacker.mobility.hover(), 0.0);
    let start = crate::queries::terrain::stand(&terrain, here.0, here.1) + hover;
    let toward = crate::queries::terrain::stand(&terrain, target_tile.0, target_tile.1) - start;
    let flat = Vec2::new(toward.x, toward.z);
    let direction = if flat.magnitude() > 0.01 {
        flat.normalize()
    } else {
        Vec2::zeros()
    };
    let swing = ((timer / theme::ACT_HIT).clamp(0.0, 1.0) * std::f32::consts::PI).sin();
    let lunge = Vec3::new(direction.x, 0.0, direction.y) * (theme::LUNGE * swing);
    transforms.for_entity(active, |(transform,)| {
        transform.translation = start + lunge;
    });

    if timer >= theme::ACT_HIT && !battle.hits_applied {
        battle.hits_applied = true;
        let hits = battle.hits.clone();
        let mut merit = 0;
        let mut felled = 0;
        for hit in hits {
            let mut words = None;
            let mut spot = None;
            let mut top = None;
            edit_unit(&mut units, hit.entity, |target| {
                let stand = crate::queries::terrain::stand(&terrain, target.column, target.row)
                    + Vec3::new(0.0, target.mobility.hover(), 0.0);
                top = Some(stand);
                spot = Some(stand + Vec3::new(0.0, 1.3, 0.0));
                if hit.heals {
                    let before = target.hp;
                    target.hp = (target.hp + hit.amount).min(target.max_hp);
                    target.flash = theme::FLASH_SECONDS;
                    target.flash_tint = theme::HEAL_TINT;
                    merit += (target.hp - before).max(0) as u32 / theme::MERIT_HEAL;
                    words = Some(format!("{} recovers {}.", target.name, target.hp - before));
                } else if hit.landed {
                    target.hp -= hit.amount;
                    merit += hit.amount.max(0) as u32 / theme::MERIT_PER_DAMAGE;
                    if target.hp <= 0 {
                        merit += theme::MERIT_KILL;
                        felled += 1;
                    }
                    target.flash = theme::FLASH_SECONDS;
                    target.flash_tint = theme::HURT_TINT;
                    let mut line = if target.hp <= 0 {
                        format!("{} takes {} and falls.", target.name, hit.amount)
                    } else {
                        format!("{} takes {}.", target.name, hit.amount)
                    };
                    if let Some(status) = shape.inflicts
                        && target.hp > 0
                    {
                        match status {
                            crate::data::Status::Stunned => target.stunned = 1,
                            crate::data::Status::Suppressed => target.suppressed = 2,
                        }
                        line.push_str(&format!(" {} is {}.", target.name, status.name()));
                    }
                    words = Some(line);
                } else {
                    words = Some(format!("{} dodges.", target.name));
                }
            });
            if let Some(top) = top {
                transforms.for_entity(hit.entity, |(transform,)| transform.translation = top);
            }
            if let Some(spot) = spot {
                if hit.heals {
                    spawn_digits(
                        &mut commands,
                        &sheet,
                        spot,
                        &hit.amount.to_string(),
                        theme::HEAL_TINT,
                    );
                    play(&mut commands, theme::SOUND_HEAL, theme::SOUND_PLAIN);
                    effects::sparkle(&mut commands, spot - Vec3::new(0.0, 0.6, 0.0));
                } else if hit.landed {
                    spawn_digits(
                        &mut commands,
                        &sheet,
                        spot,
                        &hit.amount.to_string(),
                        [1.0, 1.0, 1.0, 1.0],
                    );
                    play(&mut commands, theme::SOUND_HIT, theme::SOUND_PLAIN);
                    effects::hit_spark(
                        &mut commands,
                        spot - Vec3::new(0.0, 0.5, 0.0),
                        Vec3::new(1.0, 0.55, 0.25),
                    );
                } else {
                    spawn_digits(&mut commands, &sheet, spot, "?", [0.8, 0.8, 0.85, 1.0]);
                }
            }
            if let Some(words) = words {
                log(&mut battle, words);
            }
        }
        if merit > 0 || felled > 0 {
            edit_unit(&mut units, active, |unit| {
                unit.merit += merit;
                unit.kills += felled;
            });
        }
    }

    if timer >= theme::ACT_HIT + theme::COUNTER_PAUSE && !battle.counters_applied {
        battle.counters_applied = true;
        let standing = crate::queries::roster(&mut units);
        let sites = sites(&mut structures);
        let mut rng = battle.rng.clone();
        let mut strikes = Vec::new();
        for hit in battle.hits.clone() {
            if !hit.landed || hit.heals {
                continue;
            }
            let Some(index) = roster_index(&standing, hit.entity) else {
                continue;
            };
            let target = standing[index].1.clone();
            if !can_counter(&terrain, &target, &attacker, here, ability) {
                continue;
            }
            let strike = predict(
                &terrain,
                &sites,
                &standing,
                &target,
                (target.column, target.row),
                &attacker,
                Ability::Attack,
            );
            if (rng.f32() * 100.0) >= strike.hit_chance as f32 {
                continue;
            }
            let amount = (strike.amount * theme::COUNTER_POWER / 100).max(1);
            strikes.push((target.name, amount));
        }

        if !strikes.is_empty() {
            let mut spot = None;
            edit_unit(&mut units, active, |unit| {
                for (name, amount) in &strikes {
                    unit.hp -= amount;
                    log(&mut battle, format!("{} counters for {}.", name, amount));
                }
                unit.flash = theme::FLASH_SECONDS;
                unit.flash_tint = theme::HURT_TINT;
                spot = Some(
                    crate::queries::terrain::stand(&terrain, unit.column, unit.row)
                        + Vec3::new(0.0, unit.mobility.hover() + 1.3, 0.0),
                );
            });
            if let Some(spot) = spot {
                let total: i32 = strikes.iter().map(|(_, amount)| amount).sum();
                spawn_digits(
                    &mut commands,
                    &sheet,
                    spot,
                    &total.to_string(),
                    theme::COUNTER_TINT,
                );
                effects::hit_spark(
                    &mut commands,
                    spot - Vec3::new(0.0, 0.5, 0.0),
                    Vec3::new(0.9, 0.9, 1.0),
                );
            }
        }
    }

    if timer >= theme::ACT_DONE {
        transforms.for_entity(active, |(transform,)| {
            transform.translation = start;
        });
        let next = if attacker.team == Team::Player && !battle.moved {
            battle.menu_index = 0;
            Phase::Menu
        } else {
            Phase::TurnEnd
        };
        if next == Phase::Menu {
            cursor.column = here.0;
            cursor.row = here.1;
            look_at(&mut rig, start);
        }
        set_phase(&mut battle, next);
    }
}
