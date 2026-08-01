use crate::data;
use crate::queries::cell_center;
use crate::resources::{Floor, Sheets};
use nalgebra_glm::Vec2;
use nightshade_render::data::decoded_image::decode_to_rgba8;
use nightshade_sprite::prelude::{
    SpriteAtlases, SpriteFilter, sprite_atlas_create, sprite_atlas_insert,
};

pub fn pack_sheets(atlases: &mut SpriteAtlases) -> Sheets {
    sprite_atlas_create(
        atlases,
        data::ATLAS,
        data::ATLAS_SIZE,
        SpriteFilter::Nearest,
    );
    Sheets {
        tiles: insert_sheet(atlases, data::TILE_SHEET, data::TILE_SHEET_BYTES),
        characters: insert_sheet(atlases, data::CHARACTER_SHEET, data::CHARACTER_SHEET_BYTES),
    }
}

fn insert_sheet(
    atlases: &mut SpriteAtlases,
    name: &str,
    bytes: &[u8],
) -> nightshade_sprite::prelude::SpriteRegion {
    let decoded = decode_to_rgba8(bytes).expect("the sheet decodes");
    let (width, height) = (decoded.width, decoded.height);
    sprite_atlas_insert(atlases, data::ATLAS, name, &decoded.rgba, width, height)
        .expect("the sheet fits the atlas")
}

pub fn load_floor_plan(floor: &mut Floor) {
    let rows = data::FLOOR_PLAN.len() as u32;
    let columns = data::FLOOR_PLAN
        .iter()
        .map(|line| line.len() as u32)
        .max()
        .unwrap_or(0);

    floor.columns = columns;
    floor.rows = rows;
    floor.solid = vec![true; (columns * rows) as usize];
    floor.spawns.clear();
    floor.start = Vec2::zeros();

    for (index, line) in data::FLOOR_PLAN.iter().enumerate() {
        let row = rows - 1 - index as u32;
        for (column, mark) in line.bytes().enumerate() {
            let column = column as u32;
            let solid = mark == data::WALL;
            floor.solid[(row * columns + column) as usize] = solid;
            match mark {
                data::START => floor.start = cell_center(column, row),
                data::SPAWN => floor.spawns.push(cell_center(column, row)),
                _ => {}
            }
        }
    }

    floor.remaining = floor.spawns.len() as u32;
    floor.killed = 0;
    floor.elapsed = 0.0;
    floor.restart_timer = 0.0;
    floor.shake = 0.0;
    floor.flash = 0.0;
}

pub fn next_signed(floor: &mut Floor) -> f32 {
    floor.rng.f32() * 2.0 - 1.0
}
