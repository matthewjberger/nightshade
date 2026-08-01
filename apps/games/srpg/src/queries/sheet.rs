use crate::resources::Sheet;
use crate::theme;
use nightshade_sprite::prelude::*;

pub fn tile(sheet: &Sheet, index: u32) -> SpriteRegion {
    sheet
        .tiles
        .cell(theme::SHEET_COLUMNS, theme::SHEET_ROWS, index)
}

pub fn town_tile(sheet: &Sheet, index: u32) -> SpriteRegion {
    sheet
        .town
        .cell(theme::TOWN_COLUMNS, theme::TOWN_ROWS, index)
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

pub fn town_standing(sheet: &Sheet, index: u32) -> (SpriteRegion, f32) {
    standing_tile(
        town_tile(sheet, index),
        trim_of(&sheet.town_trim, index),
        theme::TILE_TEXELS,
    )
}
