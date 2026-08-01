use crate::data;
use crate::resources::Sheet;
use nightshade_sprite::prelude::*;

pub fn tile(sheet: &Sheet, index: u32) -> SpriteRegion {
    sheet
        .tiles
        .cell(data::SHEET_COLUMNS, data::SHEET_ROWS, index)
}

fn trim_of(sheet: &Sheet, index: u32) -> TileTrim {
    sheet.trim.get(index as usize).copied().unwrap_or_default()
}

pub fn standing(sheet: &Sheet, index: u32) -> (SpriteRegion, f32) {
    standing_tile(tile(sheet, index), trim_of(sheet, index), data::TILE_TEXELS)
}

pub fn item_picture(sheet: &Sheet, item: data::Item) -> (SpriteRegion, f32) {
    match item {
        data::Item::Shell => (sheet.shell, 1.0),
        data::Item::Fish => (sheet.fish, 1.0),
        data::Item::Berry => standing(sheet, data::BERRY_TILE),
        data::Item::Mushroom => standing(sheet, data::MUSHROOM_TILE),
        data::Item::Wood => standing(sheet, data::WOOD_TILE),
        data::Item::Stone => standing(sheet, data::STONE_TILE),
    }
}

pub fn standing_column(sheet: &Sheet, tiles: &[u32]) -> (SpriteRegion, f32) {
    let Some((bottom, above)) = tiles.split_last() else {
        return (SpriteRegion::FULL, 1.0);
    };
    let (lower, share) = standing(sheet, *bottom);
    let top = tile(sheet, *above.first().unwrap_or(bottom));
    (
        SpriteRegion::new(top.uv_min, lower.uv_max),
        above.len() as f32 + share,
    )
}

pub fn standing_stack(sheet: &Sheet, top: u32, bottom: u32) -> (SpriteRegion, f32) {
    let (lower, share) = standing(sheet, bottom);
    (
        SpriteRegion::new(tile(sheet, top).uv_min, lower.uv_max),
        1.0 + share,
    )
}
