use crate::resources::Sheet;
use crate::theme;
use nalgebra_glm::Vec2;
use nightshade_sprite::prelude::*;

pub fn tile(sheet: &Sheet, index: u32) -> SpriteRegion {
    sheet
        .tiles
        .cell(theme::SHEET_COLUMNS, theme::SHEET_ROWS, index)
}

pub fn dungeon_tile(sheet: &Sheet, index: u32) -> SpriteRegion {
    sheet
        .dungeon
        .cell(theme::SHEET_COLUMNS, theme::SHEET_ROWS, index)
}

fn trim_of(table: &[TileTrim], index: u32) -> TileTrim {
    table.get(index as usize).copied().unwrap_or_default()
}

pub fn standing(sheet: &Sheet, index: u32) -> (SpriteRegion, f32) {
    standing_tile(
        tile(sheet, index),
        trim_of(&sheet.trim, index),
        theme::TILE_TEXELS,
    )
}

pub fn dungeon_standing(sheet: &Sheet, index: u32) -> (SpriteRegion, f32) {
    standing_tile(
        dungeon_tile(sheet, index),
        trim_of(&sheet.dungeon_trim, index),
        theme::TILE_TEXELS,
    )
}

pub fn standing_wide(sheet: &Sheet, left: u32) -> (SpriteRegion, f32) {
    standing_span(
        tile(sheet, left),
        tile(sheet, left + 1),
        trim_of(&sheet.trim, left),
        trim_of(&sheet.trim, left + 1),
        theme::TILE_TEXELS,
    )
}

pub fn dungeon_wide(sheet: &Sheet, left: u32) -> (SpriteRegion, f32) {
    standing_span(
        dungeon_tile(sheet, left),
        dungeon_tile(sheet, left + 1),
        trim_of(&sheet.dungeon_trim, left),
        trim_of(&sheet.dungeon_trim, left + 1),
        theme::TILE_TEXELS,
    )
}

pub fn standing_stack(sheet: &Sheet, top: u32, bottom: u32) -> (SpriteRegion, f32) {
    let (lower, share) = standing(sheet, bottom);
    (
        SpriteRegion::new(tile(sheet, top).uv_min, lower.uv_max),
        1.0 + share,
    )
}

pub fn icon_uv(index: u32) -> (Vec2, Vec2) {
    let (column, row) = if index < theme::CUSTOM_ICON_START {
        (index % theme::SHEET_COLUMNS, index / theme::SHEET_COLUMNS)
    } else {
        (index - theme::CUSTOM_ICON_START, theme::CUSTOM_ICON_ROW)
    };
    let cell = theme::ICON_CELL as f32 / theme::ICON_PAGE as f32;
    let uv_min = Vec2::new(column as f32 * cell, row as f32 * cell);
    (uv_min, uv_min + Vec2::new(cell, cell))
}
