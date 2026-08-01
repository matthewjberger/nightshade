use crate::commands::pickups::drop_item;
use crate::commands::world::block;
use crate::components::{
    Base, Home, Lean, Lift, Lit, Phase, Reach, Regrow, Rest, Ripe, Rocks, Roll, Shaking, Speed,
    Spot, Strength, Stride, Target, Tree, Villager,
};
use crate::data::{self, Placement};
use crate::queries::cylinder;
use crate::queries::sheet::{standing, standing_column, standing_stack};
use crate::queries::town::{blocked, center, ground, lift_at, neighbors, open};
use crate::resources::{Sheet, Town};
use crate::tags::{Bush, Player};
use fastrand::Rng;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::ecs::ChildOf;
use nightshade::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;

const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

pub fn spawn_standing(
    commands: &mut Commands,
    sheet: &Sheet,
    turntable: Entity,
    picture: (SpriteRegion, f32),
    placement: Placement,
) -> Entity {
    let (region, height) = picture;
    let scale = placement.scale;
    let sprite = Sprite::new(data::ATLAS, region)
        .with_size(scale, height * scale)
        .with_anchor(Vec2::new(0.5, 0.0))
        .with_layer(data::LAYER_STANDING);
    let stand = cylinder::place(placement.spot, placement.lift);
    let entity = spawn_sprite(commands, sprite, stand);
    commands.set(
        entity,
        LocalTransform {
            translation: stand,
            rotation: cylinder::tilt(placement.spot.y),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    commands.set(entity, Lit(WHITE));
    commands.set(entity, ChildOf(turntable));
    if let Some(size) = placement.shadow {
        spawn_shadow(commands, sheet, entity, size);
    }
    entity
}

pub fn spawn_shadow(commands: &mut Commands, sheet: &Sheet, parent: Entity, size: Vec2) {
    let sprite = Sprite::new(data::ATLAS, sheet.shadow)
        .with_size(size.x, size.y)
        .with_color([0.0, 0.0, 0.0, data::SHADOW_ALPHA])
        .with_layer(data::LAYER_SHADOW);
    let shadow = spawn_sprite(commands, sprite, Vec3::new(0.0, 0.0, 0.0));
    commands.set(
        shadow,
        LocalTransform {
            translation: Vec3::new(0.0, data::SHADOW_LIFT, size.y * data::SHADOW_FORWARD),
            rotation: nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_2, &Vec3::x()),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    commands.set(shadow, ChildOf(parent));
}

pub fn spawn_walker(
    commands: &mut Commands,
    sheet: &Sheet,
    turntable: Entity,
    picture: (SpriteRegion, f32),
    placement: Placement,
    speed: f32,
) -> Entity {
    let spot = placement.spot;
    let lift = placement.lift;
    let entity = spawn_standing(commands, sheet, turntable, picture, placement);
    commands.set(entity, Spot(spot));
    commands.set(entity, Base(Vec2::new(1.0, picture.1)));
    commands.set(entity, Lift(lift));
    commands.set(entity, Speed(speed));
    commands.set(entity, Stride(0.0));
    entity
}

pub fn spawn_player(
    commands: &mut Commands,
    sheet: &Sheet,
    town: &Town,
    turntable: Entity,
    spot: Vec2,
) -> Entity {
    let placement = Placement::new(spot, lift_at(town, spot)).with_shadow(Vec2::new(0.62, 0.36));
    let player = spawn_walker(
        commands,
        sheet,
        turntable,
        standing(sheet, data::PLAYER_TILE),
        placement,
        data::WALK_SPEED,
    );
    commands.add_tag_type::<Player>(player);
    player
}

pub fn spawn_villagers(commands: &mut Commands, sheet: &Sheet, town: &Town, turntable: Entity) {
    for (index, neighbor) in data::NEIGHBORS.iter().enumerate() {
        let spot = Vec2::new(neighbor.column, neighbor.row);
        let placement = Placement::new(spot, lift_at(town, spot)).with_shadow(Vec2::new(0.6, 0.34));
        let villager = spawn_walker(
            commands,
            sheet,
            turntable,
            (sheet.villagers[index], 1.0),
            placement,
            data::NEIGHBOR_SPEED,
        );
        commands.set(villager, Villager(index));
        commands.set(villager, Home(spot));
        commands.set(villager, Target(spot));
        commands.set(villager, Reach(data::NEIGHBOR_REACH));
        commands.set(villager, Rest(index as f32 * 0.7));
        commands.set(
            villager,
            Roll(Rng::with_seed((7717 + index as u32 * 31) as u64)),
        );
    }
}

pub fn spawn_animals(commands: &mut Commands, sheet: &Sheet, town: &Town, turntable: Entity) {
    let herds = [
        (data::CHICKEN_TILE, data::CHICKEN_SPOTS.as_slice(), 0.5),
        (data::SHEEP_TILE, data::SHEEP_SPOTS.as_slice(), 0.7),
        (data::COW_TILE, data::COW_SPOTS.as_slice(), 0.8),
    ];
    let mut rng = Rng::with_seed((4241u32) as u64);
    for (tile, spots, size) in herds {
        for spot in spots {
            let placement = Placement::new(*spot, lift_at(town, *spot))
                .with_shadow(Vec2::new(size, size * 0.6));
            let animal = spawn_walker(
                commands,
                sheet,
                turntable,
                standing(sheet, tile),
                placement,
                data::ANIMAL_SPEED,
            );
            commands.set(animal, Home(*spot));
            commands.set(animal, Target(*spot));
            commands.set(animal, Reach(data::ANIMAL_REACH));
            commands.set(animal, Rest(rng.f32() * data::ANIMAL_REST));
            commands.set(animal, Roll(Rng::with_seed((331 + rng.u32(..)) as u64)));
        }
    }
}

pub fn spawn_town_tree(commands: &mut Commands, sheet: &Sheet, town: &mut Town, turntable: Entity) {
    let spot = data::TOWN_TREE;
    let placement = Placement::new(spot, lift_at(town, spot))
        .with_shadow(Vec2::new(1.9, 1.0))
        .with_scale(data::TOWN_TREE_SCALE);
    let tree = spawn_standing(
        commands,
        sheet,
        turntable,
        standing_stack(sheet, data::PINE_TOP, data::PINE_BOTTOM),
        placement,
    );
    commands.set(tree, Phase(spot.x * 1.7));
    commands.set(tree, Strength(data::SWAY_STRENGTH * 0.6));
    let column = spot.x.floor() as u32;
    let row = cylinder::wrap(spot.y).floor() as u32 % town.rows;
    block(town, column, row);
}

pub fn spawn_shop_signs(commands: &mut Commands, sheet: &Sheet, town: &Town, turntable: Entity) {
    for (index, tile) in data::SHOP_SIGNS {
        let cottage = &data::COTTAGES[index];
        let face = if cottage.door_west {
            cottage.column
        } else {
            cottage.column + data::COTTAGE_WIDTH - 1
        };
        let reach = if cottage.door_west { -1.12 } else { 1.12 };
        let row = (cottage.row + data::COTTAGE_DEPTH / 2) % town.rows;
        let spot = center(face, row) + Vec2::new(reach, 0.14);
        let placement = Placement::new(spot, data::LAND_HEIGHT as f32 + 1.42).with_scale(0.5);
        spawn_standing(commands, sheet, turntable, standing(sheet, tile), placement);
    }
}

pub fn spawn_plants(commands: &mut Commands, sheet: &Sheet, town: &mut Town, turntable: Entity) {
    let mut rng = Rng::with_seed((1913u32) as u64);
    plant(commands, sheet, town, turntable, &mut rng, 300);
    scatter_small(commands, sheet, town, turntable, &mut rng);
}

fn plant(
    commands: &mut Commands,
    sheet: &Sheet,
    town: &mut Town,
    turntable: Entity,
    rng: &mut Rng,
    count: u32,
) {
    let mut planted = 0;
    let mut tries = 0;
    while planted < count && tries < count * 30 {
        tries += 1;
        let column = rng.u32(..) % town.columns;
        let row = rng.u32(..) % town.rows;
        if !plantable(town, column, row) {
            continue;
        }
        let spot = center(column, row);
        let roll = rng.u32(..) % 10;
        let (picture, shadow, strength) = if roll < 8 {
            (
                standing_stack(sheet, data::PINE_TOP, data::PINE_BOTTOM),
                Vec2::new(1.1, 0.6),
                1.0,
            )
        } else if roll < 9 {
            (
                standing_column(sheet, &data::BARE_TREE),
                Vec2::new(1.0, 0.56),
                0.6,
            )
        } else {
            (standing(sheet, data::SMALL_TREE), Vec2::new(0.8, 0.44), 0.7)
        };
        let placement = Placement::new(spot, lift_at(town, spot)).with_shadow(shadow);
        let tree = spawn_standing(commands, sheet, turntable, picture, placement);
        commands.set(tree, Phase(spot.x * 1.7 + spot.y * 2.3));
        commands.set(tree, Strength(data::SWAY_STRENGTH * strength));
        commands.set(tree, Spot(spot));
        commands.set(tree, Tree(0.0));
        commands.set(tree, Shaking(0.0));
        commands.set(tree, Lean(0.0));
        block(town, column, row);
        planted += 1;
    }
}

fn scatter_small(
    commands: &mut Commands,
    sheet: &Sheet,
    town: &mut Town,
    turntable: Entity,
    rng: &mut Rng,
) {
    let kinds = [
        (data::BUSH, 80u32, Some(Vec2::new(0.6, 0.34)), 0.7, 0.9),
        (data::BERRY_BUSH, 50, Some(Vec2::new(0.6, 0.34)), 0.7, 0.9),
        (data::FLOWER, 70, None, 1.0, 0.7),
        (data::GRASS_TUFT, 140, None, 1.2, 0.8),
        (data::ROCKS, 36, Some(Vec2::new(0.5, 0.3)), 0.0, 0.8),
        (data::ROCK_PILE, 20, Some(Vec2::new(0.6, 0.34)), 0.0, 0.9),
    ];
    for (tile, count, shadow, strength, scale) in kinds {
        let mut placed = 0;
        let mut tries = 0;
        while placed < count && tries < count * 30 {
            tries += 1;
            let column = rng.u32(..) % town.columns;
            let row = rng.u32(..) % town.rows;
            if !plantable(town, column, row) {
                continue;
            }
            let spot =
                center(column, row) + Vec2::new(rng.f32() * 0.5 - 0.25, rng.f32() * 0.5 - 0.25);
            let mut placement = Placement::new(spot, lift_at(town, spot)).with_scale(scale);
            if let Some(size) = shadow {
                placement = placement.with_shadow(size);
            }
            let plant =
                spawn_standing(commands, sheet, turntable, standing(sheet, tile), placement);
            if strength > 0.0 {
                commands.set(plant, Phase(spot.x * 2.1 + spot.y * 1.3));
                commands.set(plant, Strength(data::SWAY_STRENGTH * strength));
            }
            if tile == data::BERRY_BUSH {
                commands.set(plant, Spot(spot));
                commands.set(plant, Ripe(true));
                commands.set(plant, Regrow(0.0));
                commands.add_tag_type::<Bush>(plant);
            }
            if tile == data::ROCK_PILE {
                commands.set(plant, Spot(spot));
                commands.set(plant, Rocks(0.0));
            }
            placed += 1;
        }
    }
}

pub fn spawn_shell(
    commands: &mut Commands,
    sheet: &Sheet,
    town: &Town,
    turntable: Entity,
    rng: &mut Rng,
) -> bool {
    for _ in 0..40 {
        let west = rng.u32(..).is_multiple_of(2);
        let column = if west {
            data::WEST_SEA_END + rng.u32(..) % (data::WEST_BEACH_END - data::WEST_SEA_END)
        } else {
            data::EAST_BEACH_START + rng.u32(..) % (data::EAST_SEA_START - data::EAST_BEACH_START)
        };
        let row = rng.u32(..) % town.rows;
        if ground(town, column, row) != data::Ground::Sand || blocked(town, column, row) {
            continue;
        }
        let spot = center(column, row) + Vec2::new(rng.f32() * 0.6 - 0.3, rng.f32() * 0.6 - 0.3);
        drop_item(
            commands,
            sheet,
            turntable,
            data::Item::Shell,
            spot,
            lift_at(town, spot),
        );
        return true;
    }
    false
}

pub fn spawn_loose_goods(commands: &mut Commands, sheet: &Sheet, town: &Town, turntable: Entity) {
    let mut rng = Rng::with_seed((6421u32) as u64);
    for _ in 0..data::SHELL_COUNT {
        spawn_shell(commands, sheet, town, turntable, &mut rng);
    }
    let mut placed = 0;
    let mut tries = 0;
    while placed < data::MUSHROOM_COUNT && tries < data::MUSHROOM_COUNT * 40 {
        tries += 1;
        let column = rng.u32(..) % town.columns;
        let row = rng.u32(..) % town.rows;
        if !open(town, column, row) {
            continue;
        }
        let spot = center(column, row);
        drop_item(
            commands,
            sheet,
            turntable,
            data::Item::Mushroom,
            spot,
            lift_at(town, spot),
        );
        placed += 1;
    }
}

pub fn spawn_props(commands: &mut Commands, sheet: &Sheet, town: &mut Town, turntable: Entity) {
    let props = [data::BARREL, data::CRATE, data::BASKET, data::SACK];
    for (index, cottage) in data::COTTAGES.iter().enumerate() {
        let tile = props[index % props.len()];
        let column = if cottage.door_west {
            cottage.column.saturating_sub(1)
        } else {
            cottage.column + data::COTTAGE_WIDTH
        };
        let row = (cottage.row + data::COTTAGE_DEPTH) % town.rows;
        if column >= town.columns || !open(town, column, row) {
            continue;
        }
        let spot = center(column, row);
        let placement = Placement::new(spot, lift_at(town, spot)).with_shadow(Vec2::new(0.5, 0.3));
        spawn_standing(commands, sheet, turntable, standing(sheet, tile), placement);
        block(town, column, row);
    }
    for (index, spot) in data::SHEEP_SPOTS.iter().enumerate() {
        let bale = Vec2::new(spot.x + 1.5, spot.y + 1.5);
        let tile = if index.is_multiple_of(2) {
            data::HAY_BALE
        } else {
            data::HAY
        };
        let placement = Placement::new(bale, lift_at(town, bale)).with_shadow(Vec2::new(0.6, 0.34));
        spawn_standing(commands, sheet, turntable, standing(sheet, tile), placement);
    }
}

fn plantable(town: &Town, column: u32, row: u32) -> bool {
    if !open(town, column, row) {
        return false;
    }
    let spot = center(column, row);
    if cylinder::town_distance(spot, Vec2::new(data::START_COLUMN, data::START_ROW)) < 3.0 {
        return false;
    }
    if data::NEIGHBORS.iter().any(|neighbor| {
        cylinder::town_distance(spot, Vec2::new(neighbor.column, neighbor.row)) < 2.5
    }) {
        return false;
    }
    neighbors(town, column, row)
        .into_iter()
        .flatten()
        .all(|(c, r)| ground(town, c, r) != data::Ground::Path)
}
