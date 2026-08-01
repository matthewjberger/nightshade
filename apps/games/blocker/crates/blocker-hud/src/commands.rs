use crate::queries::{count_label, icon_region};
use crate::resources::ArtLayers;
use blocker_blocks::data::items::ItemStack;
use nalgebra_glm::{Vec2, Vec4};
use nightshade::prelude::*;
use nightshade_ui::components::UiNodeContent;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;
use nightshade_ui_menu::prelude::SlotHandles;

pub fn paint_slot(
    layers: &ArtLayers,
    cell: &SlotHandles,
    stack: Option<ItemStack>,
    was: Option<ItemStack>,
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
) {
    match stack {
        Some(stack) => {
            if was.map(|was| was.item) != Some(stack.item)
                && let Some(region) = icon_region(layers, stack.item)
            {
                set_icon(commands, cell.icon, region);
            }
            if was.is_none() {
                ui_set_visible(dirty, commands, cell.icon, true);
            }
            ui_set_text(commands, cell.count, &count_label(stack));
        }
        None => {
            ui_set_visible(dirty, commands, cell.icon, false);
            ui_set_text(commands, cell.count, "");
        }
    }
}

pub fn set_icon(commands: &mut Commands, entity: Entity, region: (u32, Vec2, Vec2)) {
    let (layer, uv_min, uv_max) = region;
    commands.queue(move |storage, _| {
        if let Some(UiNodeContent::Image {
            texture_index,
            uv_min: min,
            uv_max: max,
        }) = storage.get_mut::<UiNodeContent>(entity)
        {
            *texture_index = layer;
            *min = uv_min;
            *max = uv_max;
        }
    });
}

pub fn set_border(commands: &mut Commands, entity: Entity, color: Vec4) {
    commands.queue(move |storage, _| {
        if let Some(UiNodeContent::Rect { border_color, .. }) =
            storage.get_mut::<UiNodeContent>(entity)
        {
            *border_color = color;
        }
    });
}
