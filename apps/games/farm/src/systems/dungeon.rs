use crate::commands::dungeon::{
    drink, gain_xp, generate, loot_of, open_chest, scatter_coins, spawn_denizen, spawn_drop,
    spawn_level, take_a_hit, teleport,
};
use crate::commands::effects::sparkle;
use crate::commands::pocket::{add_item, show_toast};
use crate::commands::tools::swing_region;
use crate::components::{
    Coins, Cooldown, Hp, Hurt, Ladder, Lit, Mimic, Opened, Rest, Roll, Stride, Target, Vial,
};
use crate::data::{Ask, EnemyKind, Item};
use crate::queries::camera::{eye_offset, screen_right};
use crate::queries::dungeon::{can_step, damage_of};
use crate::queries::sheet::dungeon_tile;
use crate::queries::terrain::height_at;
use crate::resources::{Diorama, Dungeon, Hero, Inventory, Prompt, Rig, Sheet, Terrain, Toast};
use crate::tags::InHollow;
use crate::theme;
use fastrand::Rng;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::Sprite;

pub fn build_the_hollow(
    mut state: ResMut<Dungeon>,
    mut diorama: ResMut<Diorama>,
    sheet: Res<Sheet>,
    terrain: Res<Terrain>,
    mut rig: ResMut<Rig>,
    mut commands: Commands,
    denizens: Query<(&LocalTransform,), WithTag<InHollow>>,
) {
    let (pending, leaving) = (state.pending, state.leaving);
    if pending.is_none() && !leaving {
        return;
    }
    let mut old = Vec::new();
    denizens.for_each(|entity, _| old.push(entity));
    for entity in old {
        commands.despawn_recursive(entity);
    }
    let farmer = diorama.farmer;
    let camera = diorama.camera;
    let sheet = *sheet;
    let mut dungeon = std::mem::take(&mut *state);
    dungeon.leaving = false;
    dungeon.pending = None;
    let landing = match pending {
        Some(depth) if !leaving => {
            generate(&mut dungeon, depth);
            spawn_level(&mut commands, &sheet, &dungeon);
            dungeon.active = true;
            dungeon.deepest = dungeon.deepest.max(depth);
            let spot = dungeon.entrance + Vec2::new(0.9, 0.0);
            Vec3::new(spot.x, theme::HOLLOW_Y, spot.y)
        }
        _ => {
            dungeon.active = false;
            let spot = theme::HOLLOW_DOOR_SPOT;
            let height = height_at(&terrain, spot.x, spot.y).unwrap_or(theme::VILLAGE_HEIGHT);
            Vec3::new(spot.x, height as f32, spot.y)
        }
    };
    *state = dungeon;
    if let Some(farmer) = farmer {
        teleport(&mut commands, farmer, landing);
    }
    if let Some(camera) = camera {
        rig.focus = landing;
        let eye = landing + eye_offset(rig.yaw);
        teleport(&mut commands, camera, eye);
    }
    diorama.feet = landing;
}

pub fn find_the_hollow_prompt(
    dungeon: Res<Dungeon>,
    diorama: ResMut<Diorama>,
    mut prompt: ResMut<Prompt>,
    ladders: Query<(&Ladder, &LocalTransform)>,
    chests: Query<(&Opened, &LocalTransform)>,
) {
    let ground = Vec2::new(diorama.feet.x, diorama.feet.z);
    if !dungeon.active {
        if (ground - theme::HOLLOW_DOOR_SPOT).magnitude() < theme::DOOR_RADIUS {
            *prompt = Prompt::EnterHollow;
        }
        return;
    }
    let mut found = Prompt::None;
    ladders.for_each(|_entity, (ladder, transform)| {
        let spot = Vec2::new(transform.translation.x, transform.translation.z);
        if (spot - ground).magnitude() < 1.0 {
            found = if ladder.down {
                Prompt::ClimbDown
            } else {
                Prompt::ClimbUp
            };
        }
    });
    if found == Prompt::None {
        chests.for_each(|entity, (opened, transform)| {
            let spot = Vec2::new(transform.translation.x, transform.translation.z);
            if !opened.0 && (spot - ground).magnitude() < 1.1 {
                found = Prompt::Open(entity);
            }
        });
    }
    *prompt = found;
}

pub fn act_in_the_hollow(
    actions: Res<Actions<Ask>>,
    prompt_state: Res<Prompt>,
    sheet_state: Res<Sheet>,
    mut dungeon_state: ResMut<Dungeon>,
    mut hero_state: ResMut<Hero>,
    mut inventory_state: ResMut<Inventory>,
    mut toast_state: ResMut<Toast>,
    mut commands: Commands,
    mut chests: Query<(&mut Opened, &Mimic, &mut Sprite, &LocalTransform)>,
) {
    let commands = &mut commands;
    if !actions.just_pressed(Ask::Act) {
        return;
    }
    let prompt = prompt_state.clone();
    match prompt {
        Prompt::EnterHollow => {
            dungeon_state.pending = Some(1);
            show_toast(&mut toast_state, "Down into the Hollow");
        }
        Prompt::ClimbUp => {
            let dungeon = &mut *dungeon_state;
            if dungeon.depth <= 1 {
                dungeon.leaving = true;
            } else {
                dungeon.pending = Some(dungeon.depth - 1);
            }
        }
        Prompt::ClimbDown => {
            let dungeon = &mut *dungeon_state;
            dungeon.pending = Some(dungeon.depth + 1);
        }
        Prompt::Open(entity) => {
            let sheet = *sheet_state;
            let depth = dungeon_state.depth;
            let mut hero = std::mem::take(&mut *hero_state);
            let mut inventory = std::mem::take(&mut *inventory_state);
            let mut toast = std::mem::take(&mut *toast_state);
            let mut rng = Rng::with_seed((5381 + depth * 97 + inventory.coins) as u64);
            let mut mimic = None;
            chests.for_entity(entity, |(opened, chest_mimic, sprite, transform)| {
                let position = transform.translation;
                open_chest(
                    commands,
                    &sheet,
                    &mut hero,
                    &mut inventory,
                    &mut toast,
                    opened,
                    chest_mimic,
                    sprite,
                    position,
                    depth,
                    &mut rng,
                );
                if chest_mimic.0 {
                    mimic = Some(position);
                }
            });
            if let Some(position) = mimic {
                commands.despawn_recursive(entity);
                spawn_denizen(
                    commands,
                    &sheet,
                    position,
                    EnemyKind::Mimic,
                    depth,
                    &mut rng,
                );
            }
            *hero_state = hero;
            *inventory_state = inventory;
            *toast_state = toast;
        }
        _ => {}
    }
}

pub fn stalk_the_hero(
    time: Res<Time>,
    dungeon: Res<Dungeon>,
    diorama: ResMut<Diorama>,
    rig: Res<Rig>,
    mut hero: ResMut<Hero>,
    mut commands: Commands,
    enemies: Query<(
        &EnemyKind,
        &mut Cooldown,
        &mut Target,
        &mut Rest,
        &mut Roll,
        &mut LocalTransform,
        &mut Sprite,
    )>,
) {
    if !dungeon.active {
        return;
    }
    let delta_time = time.delta_time;
    let right = screen_right(rig.yaw);
    let feet = Vec2::new(diorama.feet.x, diorama.feet.z);
    let mut bitten = false;
    enemies.for_each(
        |_entity, (kind, cooldown, target, rest, roll, transform, sprite)| {
            let ground = Vec2::new(transform.translation.x, transform.translation.z);
            let to_hero = feet - ground;
            let distance = to_hero.magnitude();
            cooldown.0 -= delta_time;
            let goal = if distance < theme::AGGRO_RANGE {
                feet
            } else {
                if (target.0 - ground).magnitude() < 0.1 {
                    rest.0 -= delta_time;
                    if rest.0 <= 0.0 {
                        let turn = roll.0.f32() * std::f32::consts::TAU;
                        target.0 = ground + Vec2::new(turn.cos(), turn.sin()) * 2.5;
                        rest.0 = 1.0 + roll.0.f32() * 2.0;
                    }
                }
                target.0
            };
            let to_goal = goal - ground;
            let span = to_goal.magnitude();
            if span > 0.05 && distance >= theme::BITE_RANGE * 0.8 {
                let step = (kind.speed() * delta_time).min(span);
                let direction = to_goal / span;
                let next = ground + direction * step;
                if can_step(&dungeon, next) {
                    transform.translation.x = next.x;
                    transform.translation.z = next.y;
                } else {
                    let slide_x = Vec2::new(next.x, ground.y);
                    let slide_z = Vec2::new(ground.x, next.y);
                    if can_step(&dungeon, slide_x) {
                        transform.translation.x = slide_x.x;
                    } else if can_step(&dungeon, slide_z) {
                        transform.translation.z = slide_z.y;
                    } else {
                        target.0 = ground;
                    }
                }
                let sideways = direction.dot(&right);
                if sideways.abs() > 0.01 {
                    sprite.flip_x = sideways < 0.0;
                }
            }
            if distance < theme::BITE_RANGE && cooldown.0 <= 0.0 {
                cooldown.0 = theme::BITE_COOLDOWN;
                if take_a_hit(&mut hero, kind.damage()) {
                    bitten = true;
                }
            }
        },
    );
    if bitten && let Some(farmer) = diorama.farmer {
        commands.set(
            farmer,
            Hurt {
                remaining: theme::HURT_SECONDS,
            },
        );
    }
}

pub fn fight(
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    dungeon: Res<Dungeon>,
    diorama: ResMut<Diorama>,
    sheet: Res<Sheet>,
    rig: Res<Rig>,
    mut hero: ResMut<Hero>,
    mut inventory: ResMut<Inventory>,
    mut toast: ResMut<Toast>,
    mut commands: Commands,
    enemies: Query<(&EnemyKind, &mut Hp, &Roll, &mut LocalTransform)>,
    mut farmers: Query<(&Stride, &Sprite)>,
) {
    hero.cooldown = (hero.cooldown - time.delta_time).max(0.0);
    if !dungeon.active || hero.cooldown > 0.0 {
        return;
    }
    if !actions.pressed(Ask::Fight) {
        return;
    }
    let Some(farmer) = diorama.farmer else {
        return;
    };
    hero.cooldown = theme::ATTACK_COOLDOWN;
    let mut flip = false;
    farmers.for_entity(farmer, |(_, sprite)| flip = sprite.flip_x);
    if let Some(tile) = hero.weapon.tile() {
        swing_region(&mut commands, farmer, dungeon_tile(&sheet, tile), flip);
    }
    let facing = if flip { -1.0 } else { 1.0 };
    let right = screen_right(rig.yaw);
    let feet = Vec2::new(diorama.feet.x, diorama.feet.z);
    let damage = damage_of(&hero);
    let mut slain = Vec::new();
    let mut struck = Vec::new();
    enemies.for_each(|entity, (kind, hp, roll, transform)| {
        let ground = Vec2::new(transform.translation.x, transform.translation.z);
        let gap = ground - feet;
        let distance = gap.magnitude();
        if distance > theme::ATTACK_RANGE || gap.dot(&right) * facing < -0.45 {
            return;
        }
        hp.0 = hp.0.saturating_sub(damage);
        let push = if distance > 0.01 {
            gap / distance * theme::KNOCKBACK
        } else {
            right * (facing * theme::KNOCKBACK)
        };
        let shoved = ground + push;
        if can_step(&dungeon, shoved) {
            transform.translation.x = shoved.x;
            transform.translation.z = shoved.y;
        }
        if hp.0 == 0 {
            slain.push((entity, transform.translation, *kind, roll.0.clone()));
        } else {
            struck.push(entity);
        }
    });
    for entity in struck {
        commands.set(
            entity,
            Hurt {
                remaining: theme::HURT_SECONDS,
            },
        );
    }
    for (entity, position, kind, enemy_rng) in slain {
        commands.despawn_recursive(entity);
        sparkle(&mut commands, position + Vec3::new(0.0, 0.4, 0.0));
        let mut rng = enemy_rng;
        let (coins, potion, ore) = loot_of(&mut rng, kind);
        inventory.coins += coins;
        if let Some(potion) = potion {
            spawn_drop(&mut commands, &sheet, position, Some(potion), 0);
        }
        if ore > 0 {
            add_item(&mut inventory, Item::Ore, ore);
        }
        gain_xp(&mut hero, kind.xp(), &mut toast);
        show_toast(
            &mut toast,
            if ore > 0 {
                format!("Felled a {}: {} coins and ore", kind.name(), coins)
            } else {
                format!("Felled a {}: {} coins", kind.name(), coins)
            },
        );
    }
}

pub fn gather_the_loot(
    dungeon: Res<Dungeon>,
    diorama: ResMut<Diorama>,
    mut hero: ResMut<Hero>,
    mut inventory: ResMut<Inventory>,
    mut toast: ResMut<Toast>,
    mut commands: Commands,
    drops: Query<(&Vial, &Coins, &LocalTransform)>,
) {
    if !dungeon.active {
        return;
    }
    let feet = Vec2::new(diorama.feet.x, diorama.feet.z);
    let mut picked = Vec::new();
    drops.for_each(|entity, (vial, coins, transform)| {
        let spot = Vec2::new(transform.translation.x, transform.translation.z);
        if (spot - feet).magnitude() < theme::PICKUP_RADIUS {
            picked.push((entity, vial.0, coins.0));
        }
    });
    for (entity, vial, coins) in picked {
        commands.despawn_recursive(entity);
        if let Some(potion) = vial {
            drink(&mut hero, potion, &mut toast);
        }
        if coins > 0 {
            inventory.coins += coins;
            show_toast(&mut toast, format!("{coins} coins"));
        }
    }
}

pub fn tend_the_hero(
    time: Res<Time>,
    mut hero: ResMut<Hero>,
    mut dungeon: ResMut<Dungeon>,
    mut inventory: ResMut<Inventory>,
    mut toast: ResMut<Toast>,
    mut commands: Commands,
    hurting: Query<(&mut Hurt, &mut Lit)>,
) {
    let delta_time = time.delta_time;
    hero.invulnerable = (hero.invulnerable - delta_time).max(0.0);
    hero.strength = (hero.strength - delta_time).max(0.0);
    hero.speed = (hero.speed - delta_time).max(0.0);
    hero.grit = (hero.grit - delta_time).max(0.0);
    if !dungeon.active {
        hero.hp = (hero.hp + theme::SURFACE_REGEN * delta_time).min(hero.max_hp as f32);
    }
    if hero.hp <= 0.0 && dungeon.active {
        hero.deaths += 1;
        hero.hp = (hero.max_hp as f32 * 0.5).max(1.0);
        dungeon.leaving = true;
        scatter_coins(&mut inventory, &mut toast);
    }
    let mut healed = Vec::new();
    hurting.for_each(|entity, (hurt, lit)| {
        hurt.remaining -= delta_time;
        if hurt.remaining <= 0.0 {
            lit.base = [1.0, 1.0, 1.0, 1.0];
            healed.push(entity);
        } else {
            lit.base = [1.0, 0.25, 0.2, 1.0];
        }
    });
    for entity in healed {
        commands.remove::<(Hurt,)>(entity);
    }
}
