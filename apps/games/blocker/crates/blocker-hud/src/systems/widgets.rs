use crate::queries::icon_region;
use crate::resources::ArtLayers;
use crate::theme;
use blocker_blocks::data::items::Item;
use blocker_chat::data::chat::Channel;
use nalgebra_glm::{Vec4, vec4};
use nightshade::prelude::*;
use nightshade_ui::commands::builder::UiTreeBuilder;
use nightshade_ui::prelude::*;

pub fn channel_color(channel: Channel) -> Vec4 {
    theme::CHANNEL_COLORS[channel.index()]
}

pub fn item_icon(
    tree: &mut UiTreeBuilder<'_, '_>,
    art: &ArtLayers,
    item: Item,
    size: f32,
) -> Entity {
    let node = tree
        .add_node()
        .size_px(size, size)
        .color_raw::<UiBase>(vec4(1.0, 1.0, 1.0, 1.0));
    match icon_region(art, item) {
        Some((layer, uv_min, uv_max)) => node.with_image_uv(layer, uv_min, uv_max).entity(),
        None => node.with_visible(false).entity(),
    }
}
