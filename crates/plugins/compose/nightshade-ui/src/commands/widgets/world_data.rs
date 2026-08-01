use crate::commands::builder::UiBuildContext;

use crate::prelude::*;
pub fn ui_data_grid_set_row_count(
    context: &mut UiBuildContext<'_, '_>,
    entity: nightshade::ecs::Entity,
    count: usize,
) {
    context.commands.queue(move |storage, _| {
        if let Some(data) = storage.get_mut::<crate::components::UiDataGridData>(entity) {
            data.total_rows = count;
            data.selected_rows.retain(|&row| row < count);
        }
    });
    ui_mark_layout_dirty(context.dirty);
}

pub fn ui_data_grid_set_cell(
    context: &mut UiBuildContext<'_, '_>,
    entity: nightshade::ecs::Entity,
    data_row: usize,
    column: usize,
    text: &str,
) {
    let Some(text_slot) = grid_cell_slot(context, entity, data_row, column) else {
        return;
    };
    context.text_state.cache.set_text(text_slot, text);
}

fn grid_cell_slot(
    context: &UiBuildContext<'_, '_>,
    entity: nightshade::ecs::Entity,
    data_row: usize,
    column: usize,
) -> Option<usize> {
    let data = context.component::<crate::components::UiDataGridData>(entity)?;
    if data_row < data.visible_start {
        return None;
    }
    let pool_index = data_row - data.visible_start;
    let pool_row = data.pool_rows.get(pool_index)?;
    pool_row.cell_text_slots.get(column).copied()
}
