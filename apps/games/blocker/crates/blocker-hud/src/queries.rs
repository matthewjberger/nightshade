use crate::resources::ArtLayers;
use crate::theme;
use blocker_blocks::data::items::ART_ITEMS;
use blocker_blocks::data::items::{Item, ItemStack};
use blocker_blocks::queries::art_color;
use blocker_blocks::queries::cell_uv_exact;
use blocker_screen::data::screen::Screen;
use nalgebra_glm::{Vec2, vec2};
use nightshade_kit::prelude::ICONS;

pub fn decode_frame_border() -> Option<(Vec<u8>, u32, u32)> {
    let (_, bytes) = ICONS.iter().find(|(name, _)| *name == theme::FRAME_IMAGE)?;
    let picture = image::load_from_memory(bytes).ok()?.to_rgba8();
    let (width, height) = picture.dimensions();
    Some((picture.into_raw(), width, height))
}

pub fn art_sheet() -> (Vec<u8>, u32, u32) {
    let mut pixels = vec![0u8; (theme::TOOL_SHEET_WIDTH * theme::TOOL_SHEET_HEIGHT * 4) as usize];
    for item in ART_ITEMS {
        let Some((art, index)) = item.art().zip(item.art_index()) else {
            continue;
        };
        for (row, line) in art.iter().enumerate() {
            for (column, character) in line.chars().enumerate() {
                let Some(color) = art_color(character) else {
                    continue;
                };
                let x = index as u32 * blocker_blocks::data::atlas::TILE_TEXELS + column as u32;
                let y = row as u32;
                let offset = ((y * theme::TOOL_SHEET_WIDTH + x) * 4) as usize;
                for channel in 0..4 {
                    pixels[offset + channel] = (color[channel] * 255.0).round() as u8;
                }
            }
        }
    }
    (pixels, theme::TOOL_SHEET_WIDTH, theme::TOOL_SHEET_HEIGHT)
}

pub fn backdrop_wanted(screen: Screen) -> bool {
    !matches!(
        screen,
        Screen::Playing | Screen::Dialogue | Screen::Character
    )
}

pub fn count_label(stack: ItemStack) -> String {
    if stack.count > 1 {
        stack.count.to_string()
    } else {
        String::new()
    }
}

pub fn icon_region(layers: &ArtLayers, item: Item) -> Option<(u32, Vec2, Vec2)> {
    match item {
        Item::Block(kind) => {
            let layer = layers.atlas?;
            let info = kind.info();
            let (min, max) = cell_uv_exact(info.side);
            let scale = vec2(
                1.0,
                blocker_blocks::data::atlas::ATLAS_ROWS as f32
                    * blocker_blocks::data::atlas::TILE_TEXELS as f32
                    / theme::TOOL_SHEET_WIDTH as f32,
            );
            Some((layer, min.component_mul(&scale), max.component_mul(&scale)))
        }
        _ => {
            let layer = layers.tools?;
            let texel =
                blocker_blocks::data::atlas::TILE_TEXELS as f32 / theme::TOOL_SHEET_WIDTH as f32;
            let left = item.art_index()? as f32 * texel;
            Some((layer, vec2(left, 0.0), vec2(left + texel, texel)))
        }
    }
}
