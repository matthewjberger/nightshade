use nightshade::prelude::*;
use nightshade_ui::components::{UiNodeColor, UiNodeContent, UiThemeBinding};
use nightshade_ui::prelude::*;

use crate::commands::ink::{EDGE_WIDTH, edge_color, ink_color};

pub fn apply(storage: &mut nightshade::ecs::Storage) {
    let bare: Vec<Entity> = storage
        .query_ref::<(&UiNodeContent,)>()
        .iter()
        .filter(|(_, (content,))| {
            matches!(**content, UiNodeContent::Text { outline_width, .. } if outline_width <= 0.0)
        })
        .map(|(entity, _)| entity)
        .collect();
    for entity in bare {
        if let Some(content) = storage.get_mut::<UiNodeContent>(entity)
            && let UiNodeContent::Text {
                outline_color,
                outline_width,
                ..
            } = &mut *content
        {
            *outline_color = edge_color();
            *outline_width = EDGE_WIDTH;
        }
        if let Some(color) = storage.get_mut::<UiNodeColor>(entity) {
            color.colors[UiBase::INDEX] = Some(ink_color());
        }
        if let Some(binding) = storage.get_mut::<UiThemeBinding>(entity) {
            binding.color_roles[UiBase::INDEX] = None;
        }
    }
}
