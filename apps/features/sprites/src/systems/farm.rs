use crate::components::{Rest, Roam, Speed, Target};
use crate::systems::setup::Sheet;
use crate::theme;
use fastrand::Rng;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_sprite::prelude::*;

pub fn build(commands: &mut Commands, sheet: &Sheet) -> Entity {
    spawn_ground(commands, sheet);
    spawn_paths(commands, sheet);
    spawn_beds(commands, sheet);
    spawn_barn(commands, sheet);
    spawn_trees(commands, sheet);
    spawn_props(commands, sheet);
    spawn_animals(commands, sheet);
    spawn_farmer(commands, sheet)
}

fn map(sheet: &Sheet, columns: u32, rows: u32, layer: i32) -> Tilemap {
    Tilemap::new(theme::ATLAS, columns, rows, theme::TILE_SIZE)
        .with_sheet(sheet.tiles, theme::SHEET_COLUMNS, theme::SHEET_ROWS)
        .with_layer(layer)
}

fn at(x: f32, y: f32) -> Vec3 {
    Vec3::new(x, y, 0.0)
}

fn spawn_ground(commands: &mut Commands, sheet: &Sheet) {
    let mut ground = Tilemap::new(
        theme::ATLAS,
        theme::GROUND_COLUMNS,
        theme::GROUND_ROWS,
        theme::TILE_SIZE,
    )
    .with_sheet(sheet.grass, 1, 1)
    .with_layer(theme::LAYER_GROUND);
    ground.fill(TileData::new(0));
    spawn_tilemap(
        commands,
        ground,
        at(theme::GROUND_ORIGIN.x, theme::GROUND_ORIGIN.y),
    );
}

fn spawn_paths(commands: &mut Commands, sheet: &Sheet) {
    let columns = 18;
    let mut road = map(sheet, columns, 1, theme::LAYER_PATHS);
    for column in 0..columns {
        let tile = if column == 0 {
            theme::PATH_LEFT
        } else if column == columns - 1 {
            theme::PATH_RIGHT
        } else if column % 2 == 0 {
            theme::PATH_MIDDLE
        } else {
            theme::PATH_MIDDLE_ALT
        };
        road.set_tile(column, 0, Some(TileData::new(tile)));
    }
    spawn_tilemap(commands, road, at(-9.0, -1.0));
}

fn spawn_beds(commands: &mut Commands, sheet: &Sheet) {
    let crops: [&[u32]; 4] = [
        &theme::CARROT,
        &theme::EGGPLANT,
        &theme::TOMATO,
        &theme::WHEAT,
    ];
    for (bed_index, crop) in crops.iter().enumerate() {
        let x = -10.0 + bed_index as f32 * 2.0;
        let y = 1.0;
        let mut bed = map(sheet, 1, 4, theme::LAYER_PATHS);
        bed.set_tile(0, 0, Some(TileData::new(theme::BED_BOTTOM)));
        bed.set_tile(0, 1, Some(TileData::new(theme::BED_MIDDLE)));
        bed.set_tile(0, 2, Some(TileData::new(theme::BED_MIDDLE_ALT)));
        bed.set_tile(0, 3, Some(TileData::new(theme::BED_TOP)));
        spawn_tilemap(commands, bed, at(x, y));

        for row in 0..4u32 {
            let mut animator = SpriteAnimator::from_regions(
                crop.iter().map(|tile| sheet.tile(*tile)),
                theme::GROW_FRAME_SECONDS,
            )
            .with_loop(SpriteLoop::PingPong);
            animator.current_frame = (row as usize + bed_index) % crop.len();
            let sprite = Sprite::new(theme::ATLAS, sheet.tile(crop[animator.current_frame]))
                .with_layer(theme::LAYER_STANDING);
            let plant = spawn_sprite(commands, sprite, at(x + 0.5, y + row as f32 + 0.5));
            commands.set(plant, animator);
        }
    }
}

fn spawn_barn(commands: &mut Commands, sheet: &Sheet) {
    let mut barn = map(
        sheet,
        theme::BARN_COLUMNS,
        theme::BARN_ROWS,
        theme::LAYER_STANDING,
    );
    for (row, tiles) in theme::BARN.iter().enumerate() {
        for (column, tile) in tiles.iter().enumerate() {
            barn.set_tile(column as u32, row as u32, Some(TileData::new(*tile)));
        }
    }
    spawn_tilemap(commands, barn, at(1.0, 2.0));
}

fn spawn_trees(commands: &mut Commands, sheet: &Sheet) {
    let pines = [
        (-14.0, 5.0),
        (-12.0, 7.0),
        (-15.5, 8.5),
        (10.0, 6.0),
        (12.0, 4.0),
        (13.5, 8.0),
        (-6.0, -7.0),
        (9.0, -8.0),
        (-13.0, -5.0),
        (14.0, -3.0),
    ];
    for (x, y) in pines {
        let mut pine = map(sheet, 1, 2, theme::LAYER_STANDING);
        pine.set_tile(0, 0, Some(TileData::new(theme::PINE_BOTTOM)));
        pine.set_tile(0, 1, Some(TileData::new(theme::PINE_TOP)));
        spawn_tilemap(commands, pine, at(x, y));
    }

    let round_trees = [(-9.5, 8.0), (7.0, 8.5), (-3.0, 9.0)];
    for (x, y) in round_trees {
        let mut tree = map(sheet, 1, 2, theme::LAYER_STANDING);
        tree.set_tile(0, 0, Some(TileData::new(theme::BUSH)));
        tree.set_tile(0, 1, Some(TileData::new(theme::ROUND_TREE_TOP)));
        spawn_tilemap(commands, tree, at(x, y));
    }
}

fn spawn_props(commands: &mut Commands, sheet: &Sheet) {
    let mut rng = Rng::with_seed((7u32) as u64);
    for index in 0..60 {
        let x = -22.0 + (rng.f32()) * 44.0;
        let y = -13.0 + (rng.f32()) * 26.0;
        let tuft = theme::TUFTS[index % theme::TUFTS.len()];
        spawn_standing(commands, sheet, tuft, x, y);
    }

    let props = [
        (theme::ROCKS, 5.0, 5.0),
        (theme::ROCKS, -16.0, 2.0),
        (theme::HAY, 5.5, 2.5),
        (theme::HAY, 6.5, 2.5),
        (theme::BARREL, 0.0, 2.5),
        (theme::SUNFLOWER, -4.0, 6.0),
        (theme::SUNFLOWER, -5.0, 6.0),
        (theme::SUNFLOWER, -4.5, 7.0),
        (theme::SUNFLOWER, 11.0, -6.0),
    ];
    for (tile, x, y) in props {
        spawn_standing(commands, sheet, tile, x, y);
    }
}

fn spawn_standing(commands: &mut Commands, sheet: &Sheet, tile: u32, x: f32, y: f32) -> Entity {
    let sprite = Sprite::new(theme::ATLAS, sheet.tile(tile))
        .with_anchor(Vec2::new(0.5, 0.0))
        .with_layer(theme::LAYER_STANDING);
    spawn_sprite(commands, sprite, at(x, y))
}

fn spawn_animals(commands: &mut Commands, sheet: &Sheet) {
    let grazers = [(theme::COW, -2.0, -5.0), (theme::SHEEP, 3.0, -7.0)];
    for (index, (tile, x, y)) in grazers.into_iter().enumerate() {
        let grazer = spawn_standing(commands, sheet, tile, x, y);
        commands.set(grazer, Target(Vec2::new(x, y)));
        commands.set(grazer, Speed(0.8));
        commands.set(grazer, Rest(1.0 + index as f32));
        commands.set(grazer, Roam(Rng::with_seed((101 + index as u32) as u64)));
    }

    let chickens = [(-3.0, 4.0), (-1.5, 4.5), (-2.5, 3.0)];
    for (index, (x, y)) in chickens.into_iter().enumerate() {
        let chicken = spawn_standing(commands, sheet, theme::CHICKEN, x, y);
        commands.set(chicken, Target(Vec2::new(x, y)));
        commands.set(chicken, Speed(1.6));
        commands.set(chicken, Rest(0.5 + index as f32 * 0.7));
        commands.set(chicken, Roam(Rng::with_seed((301 + index as u32) as u64)));
    }
}

fn spawn_farmer(commands: &mut Commands, sheet: &Sheet) -> Entity {
    let farmer = spawn_standing(commands, sheet, theme::FARMER[0], 0.0, -1.5);
    let mut walk = SpriteAnimator::from_regions(
        theme::FARMER.iter().map(|tile| sheet.tile(*tile)),
        theme::WALK_FRAME_SECONDS,
    );
    walk.pause();
    commands.set(farmer, walk);
    commands.set(farmer, Speed(theme::FARMER_SPEED));
    farmer
}
