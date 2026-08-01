use nightshade::prelude::*;
use nightshade_platform::prelude::{Input, KeyCode};
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiFrame;

use crate::data::ViewerAction;
use crate::queries::interface::axis_of;
use crate::queries::poll::{on_click, on_color, on_drag_value, on_dropdown, on_slider, on_toggle};
use crate::resources::ViewerResources;

pub fn poll_ui(
    mut viewer: ResMut<ViewerResources>,
    frame: Res<RetainedUiFrame>,
    input: Res<Input>,
) {
    let viewer = &mut *viewer;
    let mut actions = Vec::new();
    let control = input.keyboard.is_key_pressed(KeyCode::ControlLeft)
        || input.keyboard.is_key_pressed(KeyCode::ControlRight);

    for event in ui_events(&frame) {
        match event {
            UiEvent::ButtonClicked(entity) => actions.extend(on_click(viewer, *entity)),
            UiEvent::SliderChanged { entity, value } => {
                actions.extend(on_slider(viewer, *entity, *value))
            }
            UiEvent::DragValueChanged { entity, value } => {
                actions.extend(on_drag_value(viewer, *entity, *value))
            }
            UiEvent::ToggleChanged { entity, value } => {
                actions.extend(on_toggle(viewer, *entity, *value))
            }
            UiEvent::DropdownChanged {
                entity,
                selected_index,
            } => actions.extend(on_dropdown(viewer, *entity, *selected_index)),
            UiEvent::TextInputChanged { entity, text } => {
                if *entity == viewer.handles.browser_search {
                    viewer.view.search = text.clone();
                    viewer.view.browser_page = 0;
                    viewer.view.browser_dirty = true;
                } else if *entity == viewer.handles.tree_search {
                    viewer.view.tree_filter = text.clone();
                    viewer.view.tree_filter_dirty = true;
                }
            }
            UiEvent::TextInputSubmitted { entity, text } => {
                if *entity == viewer.handles.inspector_name
                    && viewer.selection.selected.is_some()
                    && !text.trim().is_empty()
                {
                    actions.push(ViewerAction::Rename(text.trim().to_string()));
                }
            }
            UiEvent::ColorPickerChanged { entity, color } => {
                actions.extend(on_color(viewer, *entity, *color))
            }
            UiEvent::TreeNodeSelected { node, selected, .. } => {
                if let Some(row) = viewer
                    .handles
                    .tree_rows
                    .iter()
                    .find(|row| row.node == *node)
                {
                    if control {
                        actions.push(ViewerAction::ToggleInSelection(row.target));
                    } else if *selected {
                        actions.push(ViewerAction::Select(Some(row.target)));
                    }
                }
            }
            UiEvent::ModalClosed { entity, .. } => {
                if *entity == viewer.handles.notice_dialog
                    && let Ok(mut guard) = viewer.incoming.notice.lock()
                {
                    *guard = None;
                }
            }
            UiEvent::CanvasClicked {
                entity, command_id, ..
            } => {
                if *entity == viewer.handles.nav_gizmo
                    && let Some(axis) = axis_of(*command_id)
                {
                    actions.push(ViewerAction::SnapAxis(axis));
                }
            }
            _ => {}
        }
    }

    viewer.actions.extend(actions);
}
