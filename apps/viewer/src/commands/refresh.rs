use nightshade::prelude::*;
use nightshade_ui::prelude::*;

use crate::queries::layout::panel_top;
use crate::resources::Layout;
use crate::theme::{
    ANIMATION_BAR_HEIGHT, LAYER_PANEL_HEADER, LAYER_ROW_HEIGHT, SELECTED_ROW_COLOR,
};

pub fn size_layer_panel(
    context: &mut nightshade_ui::commands::builder::UiBuildContext<'_, '_>,
    entity: Entity,
    rows: usize,
    layout: Layout,
) {
    let height = if rows == 0 {
        0.0
    } else {
        LAYER_PANEL_HEADER + LAYER_ROW_HEIGHT * rows as f32
    };
    context.commands.queue(move |storage, _| {
        let Some(node) = storage.get_mut::<nightshade_ui::components::UiLayoutNode>(entity) else {
            return;
        };
        let Some(nightshade_ui::components::layout_types::UiLayoutType::Window(window)) =
            node.base_layout.as_mut()
        else {
            return;
        };
        window.size = Ab(vec2(layout.animation_bar_width, height)).into();
        window.position = Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -(24.0 + ANIMATION_BAR_HEIGHT)));
    });
    ui_mark_layout_dirty(context.dirty);
}

pub fn mark_tree_selection(
    context: &mut nightshade_ui::commands::builder::UiBuildContext<'_, '_>,
    view: Entity,
    nodes: Vec<Entity>,
    selected: Vec<Entity>,
) {
    let accent = context.theme.active_theme().accent_color;
    let color = vec4(accent.x, accent.y, accent.z, 0.3);
    context.commands.queue(move |storage, _| {
        use nightshade_ui::components::{UiNodeColor, UiStateWeights, UiTreeNodeData};
        use nightshade_ui::data::state::UiSelected;
        for node in &nodes {
            let on = selected.contains(node);
            let row = storage.get_mut::<UiTreeNodeData>(*node).map(|data| {
                data.selected = on;
                data.row_entity
            });
            let Some(row) = row else {
                continue;
            };
            if let Some(colors) = storage.get_mut::<UiNodeColor>(row) {
                colors.colors[UiSelected::INDEX] = on.then_some(color);
            }
            if let Some(weights) = storage.get_mut::<UiStateWeights>(row) {
                weights.weights[UiSelected::INDEX] = if on { 1.0 } else { 0.0 };
            }
        }
        if let Some(data) = storage.get_mut::<nightshade_ui::components::UiTreeViewData>(view) {
            data.selected_nodes = selected;
        }
    });
}

pub fn open_notice(
    context: &mut nightshade_ui::commands::builder::UiBuildContext<'_, '_>,
    dialog: Entity,
    backdrop: Entity,
) {
    ui_set_visible(context.dirty, context.commands, backdrop, true);
    ui_set_visible(context.dirty, context.commands, dialog, true);
    context.commands.queue(move |storage, _| {
        if let Some(data) = storage.get_mut::<nightshade_ui::components::UiModalDialogData>(dialog)
        {
            data.result = None;
        }
    });
    context.overlays.active_modal = Some(dialog);
}

pub fn move_nav_gizmo(
    context: &mut nightshade_ui::commands::builder::UiBuildContext<'_, '_>,
    entity: Entity,
    offset: f32,
    layout: Layout,
) {
    context.commands.queue(move |storage, _| {
        let Some(node) = storage.get_mut::<nightshade_ui::components::UiLayoutNode>(entity) else {
            return;
        };
        let Some(nightshade_ui::components::layout_types::UiLayoutType::Window(window)) =
            node.base_layout.as_mut()
        else {
            return;
        };
        window.position = Rl(vec2(100.0, 0.0)) + Ab(vec2(offset, panel_top(layout)));
    });
    ui_mark_layout_dirty(context.dirty);
}

pub fn apply_visibility(
    context: &mut nightshade_ui::commands::builder::UiBuildContext<'_, '_>,
    applied: &mut Option<bool>,
    entity: Entity,
    visible: bool,
) {
    if *applied == Some(visible) {
        return;
    }
    *applied = Some(visible);
    ui_set_visible(context.dirty, context.commands, entity, visible);
}

pub fn highlight(
    context: &mut nightshade_ui::commands::builder::UiBuildContext<'_, '_>,
    entity: Entity,
    active: bool,
) {
    let color = if active {
        SELECTED_ROW_COLOR
    } else {
        vec4(0.0, 0.0, 0.0, 0.0)
    };
    ui_set_color::<UiBase>(context.commands, entity, color);
}
