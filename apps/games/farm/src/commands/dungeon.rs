use crate::commands::effects::sparkle;
use crate::commands::life::{spawn_shadow, spawn_standing};
use crate::commands::pocket::{add_item, show_toast};
use crate::components::{
    Coins, Cooldown, Hp, Ladder, Lit, Mimic, NightOnly, Opened, Phase, Rest, Roll, Strength,
    Target, Upright, Vial,
};
use crate::data::{EnemyKind, Item, Potion, Weapon};
use crate::queries::dungeon::{beside_floor, center, index, is_floor, level_for, neighbors};
use crate::queries::sheet::dungeon_standing;
use crate::resources::{Dungeon, Hero, Inventory, Sheet, Toast};
use crate::tags::InHollow;
use crate::theme;
use fastrand::Rng;
use nalgebra_glm::{Quat, Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::{
    spawn_instanced_mesh_with_material, spawn_mesh_with_material,
};
use nightshade_graphics::commands::light::spawn_light_entity;
use nightshade_render::components::light::{Light, LightType};
use nightshade_render::components::mesh::InstanceTransform;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;

pub fn generate(dungeon: &mut Dungeon, depth: u32) {
    dungeon.columns = theme::HOLLOW_COLUMNS;
    dungeon.rows = theme::HOLLOW_ROWS;
    dungeon.origin = theme::HOLLOW_ORIGIN;
    dungeon.depth = depth;
    let cells = (dungeon.columns * dungeon.rows) as usize;
    dungeon.floor = vec![false; cells];
    let mut rng = Rng::with_seed((7919 * depth + 17) as u64);
    let start = (2u32, dungeon.rows / 2);
    let mut at = start;
    let wanted = (cells as f32 * theme::HOLLOW_FLOOR_SHARE) as usize;
    let mut carved = 0;
    let mut heading = 0u32;
    let mut steps = 0;
    while carved < wanted && steps < cells * 40 {
        steps += 1;
        let cell = index(dungeon, at.0, at.1);
        if !dungeon.floor[cell] {
            dungeon.floor[cell] = true;
            carved += 1;
        }
        if rng.f32() < 0.4 {
            for (c, r) in neighbors(dungeon, at.0, at.1).into_iter().flatten() {
                if c >= 1 && c + 1 < dungeon.columns && r >= 1 && r + 1 < dungeon.rows {
                    let nearby = index(dungeon, c, r);
                    if !dungeon.floor[nearby] {
                        dungeon.floor[nearby] = true;
                        carved += 1;
                    }
                }
            }
        }
        if rng.f32() < 0.35 {
            heading = rng.u32(..) % 4;
        }
        let (dx, dy): (i32, i32) = match heading {
            0 => (1, 0),
            1 => (0, 1),
            2 => (-1, 0),
            _ => (0, -1),
        };
        let column = (at.0 as i32 + dx).clamp(1, dungeon.columns as i32 - 2);
        let row = (at.1 as i32 + dy).clamp(1, dungeon.rows as i32 - 2);
        at = (column as u32, row as u32);
    }
    dungeon.entrance = center(dungeon, start.0, start.1);
    let mut farthest = (0i32, start);
    for row in 0..dungeon.rows {
        for column in 0..dungeon.columns {
            if !is_floor(dungeon, column, row) {
                continue;
            }
            let distance =
                (column as i32 - start.0 as i32).abs() + (row as i32 - start.1 as i32).abs();
            if distance > farthest.0 {
                farthest = (distance, (column, row));
            }
        }
    }
    dungeon.exit = center(dungeon, farthest.1.0, farthest.1.1);
}

fn random_floor(dungeon: &Dungeon, rng: &mut Rng, away_from: Vec2, minimum: f32) -> Option<Vec2> {
    for _ in 0..200 {
        let column = rng.u32(..) % dungeon.columns;
        let row = rng.u32(..) % dungeon.rows;
        if !is_floor(dungeon, column, row) {
            continue;
        }
        let spot = center(dungeon, column, row);
        if (spot - away_from).magnitude() >= minimum && (spot - dungeon.exit).magnitude() > 0.6 {
            return Some(spot);
        }
    }
    None
}

fn floor_position(point: Vec2) -> Vec3 {
    Vec3::new(point.x, theme::HOLLOW_Y, point.y)
}

pub fn spawn_denizen(
    commands: &mut Commands,
    sheet: &Sheet,
    position: Vec3,
    kind: EnemyKind,
    depth: u32,
    rng: &mut Rng,
) -> Entity {
    let (region, height) = dungeon_standing(sheet, kind.tile());
    let size = kind.size();
    let sprite = Sprite::new(theme::ATLAS, region)
        .with_size(size, height * size)
        .with_anchor(Vec2::new(0.5, 0.0))
        .with_layer(theme::LAYER_STANDING);
    let entity = spawn_sprite(commands, sprite, position);
    commands.set(entity, Upright::default());
    spawn_shadow(
        commands,
        sheet,
        entity,
        Vec2::new(0.0, 0.0),
        Vec2::new(0.8 * size, 0.5 * size),
    );
    commands.set(
        entity,
        Lit {
            base: [1.0, 1.0, 1.0, 1.0],
        },
    );
    commands.set(entity, kind);
    commands.set(entity, Hp(kind.hp() + depth / 2));
    commands.set(entity, Cooldown(1.0));
    commands.set(entity, Target(Vec2::new(position.x, position.z)));
    commands.set(entity, Rest(0.5));
    commands.set(entity, Roll(rng.clone()));
    commands.add_tag_type::<InHollow>(entity);
    entity
}

pub fn spawn_drop(
    commands: &mut Commands,
    sheet: &Sheet,
    position: Vec3,
    potion: Option<Potion>,
    coins: u32,
) {
    let tile = potion.map_or(theme::COIN_TILE, Potion::tile);
    let (region, height) = dungeon_standing(sheet, tile);
    let sprite = Sprite::new(theme::ATLAS, region)
        .with_size(0.55, height * 0.55)
        .with_anchor(Vec2::new(0.5, 0.0))
        .with_layer(theme::LAYER_STANDING);
    let entity = spawn_sprite(commands, sprite, position);
    commands.set(entity, Upright::default());
    commands.set(
        entity,
        Lit {
            base: [1.0, 1.0, 1.0, 1.0],
        },
    );
    commands.set(entity, Vial(potion));
    commands.set(entity, Coins(coins));
    commands.add_tag_type::<InHollow>(entity);
}

fn spawn_chest(commands: &mut Commands, sheet: &Sheet, position: Vec3, mimic: bool) {
    let (chest, _) = spawn_standing(
        commands,
        sheet,
        dungeon_standing(sheet, theme::CHEST_CLOSED),
        position,
        Vec2::new(0.8, 0.5),
    );
    commands.set(chest, Opened(false));
    commands.set(chest, Mimic(mimic));
    commands.add_tag_type::<InHollow>(chest);
}

fn spawn_ladder(commands: &mut Commands, sheet: &Sheet, position: Vec3, down: bool) {
    let (ladder, _) = spawn_standing(
        commands,
        sheet,
        dungeon_standing(sheet, theme::LADDER_TILE),
        position,
        Vec2::new(0.7, 0.4),
    );
    commands.set(ladder, Ladder { down });
    commands.add_tag_type::<InHollow>(ladder);
}

fn spawn_torch(commands: &mut Commands, sheet: &Sheet, position: Vec3) {
    let (region, height) = dungeon_standing(sheet, theme::TORCH_TILE);
    let sprite = Sprite::new(theme::ATLAS, region)
        .with_size(0.7, height * 0.7)
        .with_anchor(Vec2::new(0.5, 0.0))
        .with_layer(theme::LAYER_STANDING);
    let torch = spawn_sprite(commands, sprite, position + Vec3::new(0.0, 0.45, 0.0));
    commands.set(
        torch,
        Lit {
            base: [1.6, 1.6, 1.6, 1.0],
        },
    );
    commands.add_tag_type::<InHollow>(torch);
    let light = spawn_light_entity(commands, position + Vec3::new(0.0, 1.2, 0.4), "Torch");
    commands.set(
        light,
        Light {
            light_type: LightType::Point,
            color: theme::TORCH_COLOR,
            intensity: theme::TORCH_STRENGTH,
            range: theme::TORCH_RANGE,
            ..Default::default()
        },
    );
    commands.set(light, Phase(position.x * 0.7 + position.z));
    commands.set(light, Strength(theme::TORCH_STRENGTH));
    commands.set(light, NightOnly(false));
    commands.add_tag_type::<InHollow>(light);
}

fn surface_of(column: u32, row: u32, count: usize) -> usize {
    let mixed = column.wrapping_mul(73).wrapping_add(row.wrapping_mul(151));
    (mixed % count as u32) as usize
}

fn spawn_prop(commands: &mut Commands, sheet: &Sheet, position: Vec3, tile: u32) {
    let (region, height) = dungeon_standing(sheet, tile);
    let sprite = Sprite::new(theme::ATLAS, region)
        .with_size(0.6, height * 0.6)
        .with_anchor(Vec2::new(0.5, 0.0))
        .with_layer(theme::LAYER_STANDING);
    let prop = spawn_sprite(commands, sprite, position);
    commands.set(prop, Upright::default());
    commands.set(
        prop,
        Lit {
            base: [1.0, 1.0, 1.0, 1.0],
        },
    );
    commands.add_tag_type::<InHollow>(prop);
}

pub fn spawn_level(commands: &mut Commands, sheet: &Sheet, dungeon: &Dungeon) {
    let depth = dungeon.depth;
    let y = theme::HOLLOW_Y;
    let mut floors: Vec<Vec<InstanceTransform>> =
        vec![Vec::new(); theme::DUNGEON_FLOOR_SURFACES.len()];
    let mut walls: Vec<Vec<InstanceTransform>> =
        vec![Vec::new(); theme::DUNGEON_WALL_SURFACES.len()];
    let mut caps = Vec::new();
    let mut torch_spots = Vec::new();
    let mut prop_spots = Vec::new();
    let mut counted = 0u32;
    let mut floors_seen = 0u32;
    for row in 0..dungeon.rows {
        for column in 0..dungeon.columns {
            let middle = center(dungeon, column, row);
            if is_floor(dungeon, column, row) {
                let pick = surface_of(column, row, theme::DUNGEON_FLOOR_SURFACES.len());
                floors[pick].push(InstanceTransform::new(
                    Vec3::new(middle.x, y - theme::CAP_THICKNESS / 2.0, middle.y),
                    Quat::identity(),
                    Vec3::new(1.0, theme::CAP_THICKNESS, 1.0),
                ));
                floors_seen += 1;
                if floors_seen.is_multiple_of(theme::DUNGEON_PROP_EVERY) {
                    let tile = theme::DUNGEON_PROP_TILES[floors_seen as usize
                        / theme::DUNGEON_PROP_EVERY as usize
                        % theme::DUNGEON_PROP_TILES.len()];
                    prop_spots.push((Vec3::new(middle.x, y, middle.y), tile));
                }
                continue;
            }
            let Some(open) = beside_floor(dungeon, column, row) else {
                continue;
            };
            let pick = surface_of(column, row, theme::DUNGEON_WALL_SURFACES.len());
            for level in 0..theme::HOLLOW_WALL_LEVELS {
                walls[pick].push(InstanceTransform::new(
                    Vec3::new(middle.x, y + level as f32 + 0.5, middle.y),
                    Quat::identity(),
                    Vec3::new(1.0, 1.0, 1.0),
                ));
            }
            caps.push(InstanceTransform::new(
                Vec3::new(
                    middle.x,
                    y + theme::HOLLOW_WALL_HEIGHT + theme::DUNGEON_CAP_HEIGHT / 2.0,
                    middle.y,
                ),
                Quat::identity(),
                Vec3::new(1.04, theme::DUNGEON_CAP_HEIGHT, 1.04),
            ));
            counted += 1;
            if counted.is_multiple_of(theme::TORCH_EVERY) {
                let toward = (center(dungeon, open.0, open.1) - middle) * 0.56;
                torch_spots.push(Vec3::new(middle.x + toward.x, y, middle.y + toward.y));
            }
        }
    }
    commands.queue(move |storage, resources| {
        for (instances, (material, _)) in floors.into_iter().zip(theme::DUNGEON_FLOOR_SURFACES) {
            let material_id =
                nightshade_render::queries::material_registry::material_registry_resolve(
                    resources
                        .res::<nightshade_render::resources::material_registry::MaterialRegistry>(),
                    material,
                )
                .unwrap_or_default();
            let floor = spawn_instanced_mesh_with_material(
                storage,
                resources,
                "Cube",
                instances,
                material_id,
            );
            storage.add_tag_type::<InHollow>(floor);
        }
        for (instances, (material, _)) in walls.into_iter().zip(theme::DUNGEON_WALL_SURFACES) {
            let material_id =
                nightshade_render::queries::material_registry::material_registry_resolve(
                    resources
                        .res::<nightshade_render::resources::material_registry::MaterialRegistry>(),
                    material,
                )
                .unwrap_or_default();
            let wall = spawn_instanced_mesh_with_material(
                storage,
                resources,
                "Cube",
                instances,
                material_id,
            );
            storage.add_tag_type::<InHollow>(wall);
        }
        let material_id = nightshade_render::queries::material_registry::material_registry_resolve(
            resources.res::<nightshade_render::resources::material_registry::MaterialRegistry>(),
            theme::DUNGEON_CAP_SURFACE.0,
        )
        .unwrap_or_default();
        let cap = spawn_instanced_mesh_with_material(storage, resources, "Cube", caps, material_id);
        storage.add_tag_type::<InHollow>(cap);
    });
    for (spot, tile) in prop_spots {
        spawn_prop(commands, sheet, spot, tile);
    }
    let rock = spawn_mesh_with_material(
        commands,
        "Cube",
        Vec3::new(0.0, y - 0.6, 0.0),
        Vec3::new(theme::HOLLOW_ROCK_SIZE, 0.8, theme::HOLLOW_ROCK_SIZE),
        theme::DUNGEON_ROCK_MATERIAL,
    );
    commands.add_tag_type::<InHollow>(rock);
    let roof = spawn_mesh_with_material(
        commands,
        "Cube",
        Vec3::new(0.0, y + theme::HOLLOW_ROOF_HEIGHT, 0.0),
        Vec3::new(theme::HOLLOW_ROCK_SIZE, 0.4, theme::HOLLOW_ROCK_SIZE),
        theme::DUNGEON_ROCK_MATERIAL,
    );
    commands.add_tag_type::<InHollow>(roof);
    for spot in torch_spots {
        spawn_torch(commands, sheet, spot);
    }

    spawn_ladder(commands, sheet, floor_position(dungeon.entrance), false);
    spawn_ladder(commands, sheet, floor_position(dungeon.exit), true);

    let mut rng = Rng::with_seed((104729 * depth + 3) as u64);
    let roster: Vec<EnemyKind> = EnemyKind::ROSTER
        .into_iter()
        .filter(|kind| kind.min_depth() <= depth)
        .collect();
    let count = (theme::ENEMIES_BASE + theme::ENEMIES_PER_DEPTH * (depth - 1)).min(24);
    for _ in 0..count {
        let Some(spot) = random_floor(dungeon, &mut rng, dungeon.entrance, 5.0) else {
            break;
        };
        let kind = roster[(rng.u32(..) as usize) % roster.len()];
        spawn_denizen(commands, sheet, floor_position(spot), kind, depth, &mut rng);
    }
    if depth.is_multiple_of(theme::BOSS_EVERY) {
        let boss_spot = dungeon.exit + Vec2::new(0.0, 1.0);
        spawn_denizen(
            commands,
            sheet,
            floor_position(boss_spot),
            EnemyKind::Crab,
            depth,
            &mut rng,
        );
    }
    for _ in 0..theme::CHESTS_PER_LEVEL {
        if let Some(spot) = random_floor(dungeon, &mut rng, dungeon.entrance, 3.0) {
            let mimic = depth >= 2 && rng.f32() < theme::MIMIC_CHANCE;
            spawn_chest(commands, sheet, floor_position(spot), mimic);
        }
    }
    for _ in 0..theme::POTIONS_PER_LEVEL {
        if let Some(spot) = random_floor(dungeon, &mut rng, dungeon.entrance, 2.0) {
            let potion = Potion::ALL[(rng.u32(..) as usize) % Potion::ALL.len()];
            spawn_drop(commands, sheet, floor_position(spot), Some(potion), 0);
        }
    }
}

pub fn teleport(commands: &mut Commands, entity: Entity, position: Vec3) {
    commands.queue(move |storage, _| {
        if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
            transform.translation = position;
        }
    });
}

pub fn drink(hero: &mut Hero, potion: Potion, toast: &mut Toast) {
    match potion {
        Potion::Heal => {
            hero.hp = (hero.hp + theme::HEAL_AMOUNT as f32).min(hero.max_hp as f32);
        }
        Potion::Strength => hero.strength = theme::BUFF_SECONDS,
        Potion::Speed => hero.speed = theme::BUFF_SECONDS,
        Potion::Grit => hero.grit = theme::BUFF_SECONDS,
    }
    show_toast(toast, format!("Drank a {}", potion.name()));
}

pub fn gain_xp(hero: &mut Hero, xp: u32, toast: &mut Toast) {
    hero.xp += xp;
    let level = level_for(hero.xp);
    while hero.level < level {
        hero.level += 1;
        hero.max_hp += theme::LEVEL_HP;
        hero.hp = hero.max_hp as f32;
        show_toast(
            toast,
            format!("Level {}! Tougher and stronger.", hero.level),
        );
    }
}

pub fn take_a_hit(hero: &mut Hero, damage: u32) -> bool {
    if hero.invulnerable > 0.0 {
        return false;
    }
    let grit = if hero.grit > 0.0 {
        theme::GRIT_BONUS
    } else {
        0
    };
    let dealt = damage.saturating_sub(grit).max(1);
    hero.hp -= dealt as f32;
    hero.invulnerable = theme::INVULNERABLE_SECONDS;
    true
}

pub fn open_chest(
    commands: &mut Commands,
    sheet: &Sheet,
    hero: &mut Hero,
    inventory: &mut Inventory,
    toast: &mut Toast,
    opened: &mut Opened,
    mimic: &Mimic,
    sprite: &mut Sprite,
    position: Vec3,
    depth: u32,
    rng: &mut Rng,
) {
    if opened.0 {
        return;
    }
    opened.0 = true;
    if mimic.0 {
        sprite.region = dungeon_standing(sheet, theme::MIMIC_TILE).0;
        show_toast(toast, "The chest has teeth!");
        return;
    }
    let (region, height) = dungeon_standing(sheet, theme::CHEST_OPEN);
    sprite.region = region;
    sprite.size.y = height;
    sparkle(commands, position + Vec3::new(0.0, 0.5, 0.0));
    let coins = theme::CHEST_COINS + theme::CHEST_COINS_PER_DEPTH * depth + rng.u32(..) % 8;
    inventory.coins += coins;
    let ore = theme::CHEST_ORE + depth / 3;
    add_item(inventory, Item::Ore, ore);
    let found = Weapon::found_at(depth);
    if found > hero.weapon && rng.f32() < 0.5 {
        hero.weapon = found;
        show_toast(
            toast,
            format!(
                "{} coins, {} ore, and a {}!",
                coins,
                ore,
                found.name().to_lowercase()
            ),
        );
        return;
    }
    if rng.f32() < 0.6 {
        let potion = Potion::ALL[(rng.u32(..) as usize) % Potion::ALL.len()];
        spawn_drop(
            commands,
            sheet,
            position + Vec3::new(0.8, 0.0, 0.3),
            Some(potion),
            0,
        );
    }
    show_toast(toast, format!("{} coins in the chest", coins));
}

pub fn loot_of(rng: &mut Rng, kind: EnemyKind) -> (u32, Option<Potion>, u32) {
    let coins = 1 + rng.u32(..) % 3 + kind.xp() / 4;
    let potion = (rng.f32() < theme::DROP_POTION_CHANCE)
        .then(|| Potion::ALL[(rng.u32(..) as usize) % Potion::ALL.len()]);
    let ore = u32::from(rng.f32() < theme::DROP_ORE_CHANCE);
    (coins, potion, ore)
}

pub fn scatter_coins(inventory: &mut Inventory, toast: &mut Toast) {
    let lost = (inventory.coins as f32 * theme::DEATH_COIN_SHARE) as u32;
    inventory.coins -= lost;
    show_toast(toast, format!("You wake at home. {} coins are gone.", lost));
}

pub fn give_dagger(hero: &mut Hero, toast: &mut Toast) {
    if hero.weapon < Weapon::Dagger {
        hero.weapon = Weapon::Dagger;
    }
    show_toast(toast, "Wren gave you a dagger");
}
