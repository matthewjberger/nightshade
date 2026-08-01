use crate::data::{self, LevelSpec};
use crate::resources::{RunState, Sheets};
use crate::theme;
use nalgebra_glm::IVec2;
use nightshade::prelude::Vec2;
use nightshade_sprite::prelude::SpriteRegion;
use nightshade_tiles::prelude::{TileMap, at};

pub fn current_level(run: &RunState) -> &'static LevelSpec {
    &data::LEVELS[run.level.min(data::LEVELS.len() - 1)]
}

pub fn level_columns(level: &LevelSpec) -> i32 {
    level
        .rows
        .iter()
        .map(|row| row.len() as i32)
        .max()
        .unwrap_or(0)
}

pub fn level_rows(level: &LevelSpec) -> i32 {
    level.rows.len() as i32
}

pub fn level_cell(level: &LevelSpec, column: i32, row: i32) -> u8 {
    if column < 0 || row < 0 || row >= level_rows(level) {
        return b' ';
    }
    let line = level.rows[(level_rows(level) - 1 - row) as usize].as_bytes();
    line.get(column as usize).copied().unwrap_or(b' ')
}

pub fn cell_is_solid(level: &LevelSpec, column: i32, row: i32) -> bool {
    matches!(level_cell(level, column, row), data::GROUND | data::BLOCK)
}

pub fn prop_picture(mark: u8) -> Option<u32> {
    data::PROPS
        .iter()
        .find(|(key, _)| *key == mark)
        .map(|(_, tile)| *tile)
}

pub fn solid_at(solids: &TileMap, column: i32, row: i32) -> bool {
    if column < 0 || column >= solids.width as i32 {
        return true;
    }
    if row < 0 || row >= solids.height as i32 {
        return false;
    }
    let mark = at(solids, IVec2::new(column, solids.height as i32 - 1 - row));
    matches!(mark, data::GROUND | data::BLOCK)
}

pub fn ground_picture(level: &LevelSpec, column: i32, row: i32) -> u32 {
    let palette = &level.palette;
    if cell_is_solid(level, column, row + 1) {
        return palette.fill;
    }
    let left = cell_is_solid(level, column - 1, row);
    let right = cell_is_solid(level, column + 1, row);
    match (left, right) {
        (true, true) => palette.top[2],
        (false, true) => palette.top[1],
        (true, false) => palette.top[3],
        (false, false) => palette.top[0],
    }
}

pub fn tile_region(sheets: &Sheets, index: u32) -> SpriteRegion {
    sheets
        .tiles
        .cell(theme::TILE_COLUMNS, theme::TILE_ROWS, index)
}

pub fn character_region(sheets: &Sheets, index: u32) -> SpriteRegion {
    sheets
        .characters
        .cell(theme::CHARACTER_COLUMNS, theme::CHARACTER_ROWS, index)
}

pub fn boxes_overlap(
    first_center: Vec2,
    first_half: Vec2,
    second_center: Vec2,
    second_half: Vec2,
) -> bool {
    (first_center.x - second_center.x).abs() < first_half.x + second_half.x
        && (first_center.y - second_center.y).abs() < first_half.y + second_half.y
}
