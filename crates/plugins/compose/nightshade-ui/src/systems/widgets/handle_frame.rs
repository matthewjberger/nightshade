use nalgebra_glm::Vec2;
use nightshade::ecs::system_param::{Query, Res, ResMut};

use super::handle_scroll::{UiInteractions, UiNodes};
use super::handle_slider::row_interaction;

pub type UiButtons<'world, 'item> = Query<'world, (&'item mut crate::components::UiButtonData,)>;

pub type UiCanvases<'world, 'item> = Query<'world, (&'item crate::components::UiCanvasData,)>;

pub type UiDropdowns<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiDropdownData,)>;

pub type UiMenus<'world, 'item> = Query<'world, (&'item mut crate::components::UiMenuData,)>;

pub type UiContextMenus<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiContextMenuData,)>;

pub type UiNodeContents<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiNodeContent,)>;

pub fn ui_shortcut_system(
    groups: Res<crate::resources::RetainedUiGroups>,
    chrome: Res<crate::resources::WindowChrome>,
    mirror: Res<crate::resources::RetainedUiInputMirror>,
    interaction: Res<crate::resources::RetainedUiInteraction>,
    input: Res<nightshade_platform::contract::input::Input>,
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut inputs: super::handle_input::UiTextInputs<'_, '_>,
    mut areas: super::handle_text_area::UiTextAreas<'_, '_>,
    mut editors: super::handle_rich_text::UiRichTextEditors<'_, '_>,
) {
    if let Some(focused) = interaction.focused_entity {
        let mut is_editor = false;
        inputs.for_entity(focused, |_| is_editor = true);
        areas.for_entity(focused, |_| is_editor = true);
        editors.for_entity(focused, |_| is_editor = true);
        if is_editor {
            return;
        }
    }

    let alt_held = input
        .keyboard
        .is_key_pressed(nightshade_platform::KeyCode::AltLeft)
        || input
            .keyboard
            .is_key_pressed(nightshade_platform::KeyCode::AltRight);

    for &(ref key, pressed) in &mirror.frame_keys {
        if !pressed {
            continue;
        }
        for (binding, command_index) in &groups.shortcuts {
            if binding.key == *key
                && binding.ctrl == mirror.ctrl_held
                && binding.shift == mirror.shift_held
                && binding.alt == alt_held
            {
                frame
                    .events
                    .push(crate::resources::UiEvent::ShortcutTriggered {
                        command_index: *command_index,
                    });
            }
        }
        for shortcut in &chrome.shortcuts {
            if shortcut.binding.key == *key
                && shortcut.binding.ctrl == mirror.ctrl_held
                && shortcut.binding.shift == mirror.shift_held
                && shortcut.binding.alt == alt_held
            {
                frame
                    .events
                    .push(crate::resources::UiEvent::ShortcutTriggered {
                        command_index: shortcut.command_index,
                    });
            }
        }
    }
}

pub fn ui_button_system(
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut buttons: UiButtons<'_, '_>,
    mut interactions: UiInteractions<'_, '_>,
) {
    let mut pending = Vec::new();
    buttons.for_each_mut(|entity, _| pending.push(entity));

    for entity in pending {
        let clicked = row_interaction(&mut interactions, entity).clicked;
        buttons.for_entity(entity, |(data,)| data.clicked = clicked);
        if clicked {
            frame
                .events
                .push(crate::resources::UiEvent::ButtonClicked(entity));
        }
    }
}

pub fn ui_canvas_system(
    input: Res<nightshade_platform::contract::input::Input>,
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut canvases: UiCanvases<'_, '_>,
    mut nodes: UiNodes<'_, '_>,
    mut interactions: UiInteractions<'_, '_>,
) {
    let mut pending = Vec::new();
    canvases.for_each_mut(|entity, (data,)| {
        if data.hit_test_enabled {
            pending.push((entity, data.command_bounds.clone()));
        }
    });

    for (entity, command_bounds) in pending {
        if !row_interaction(&mut interactions, entity).clicked {
            continue;
        }
        let Some(rect) = super::handle_slider::node_rect(&mut nodes, entity) else {
            continue;
        };
        let local_position = input.mouse.position - rect.min;
        for &(command_id, ref bounds) in command_bounds.iter().rev() {
            if bounds.contains(local_position) {
                frame.events.push(crate::resources::UiEvent::CanvasClicked {
                    entity,
                    command_id,
                    position: local_position,
                });
                break;
            }
        }
    }
}

pub fn ui_error_border_system(
    theme: Res<crate::resources::ThemeState>,
    mut interactions: UiInteractions<'_, '_>,
    mut contents: UiNodeContents<'_, '_>,
) {
    let error_color = theme.active_theme().error_color;

    let mut pending = Vec::new();
    interactions.for_each_mut(|entity, (interaction,)| {
        if interaction.error_text.is_some() {
            pending.push(entity);
        }
    });

    for entity in pending {
        contents.for_entity(entity, |(content,)| {
            if let crate::components::UiNodeContent::Rect {
                border_color,
                border_width,
                ..
            } = content
            {
                *border_color = error_color;
                if *border_width < 1.0 {
                    *border_width = 1.0;
                }
            }
        });
    }
}

pub fn ui_popup_dismiss_system(
    mirror: Res<crate::resources::RetainedUiInputMirror>,
    input: Res<nightshade_platform::contract::input::Input>,
    mut dirty: ResMut<crate::resources::RetainedUiDirty>,
    mut overlays: ResMut<crate::resources::RetainedUiOverlays>,
    mut dropdowns: UiDropdowns<'_, '_>,
    mut menus: UiMenus<'_, '_>,
    mut context_menus: UiContextMenus<'_, '_>,
    mut nodes: UiNodes<'_, '_>,
) {
    if mirror.scroll_delta.y.abs() <= f32::EPSILON {
        return;
    }

    let mouse_position = input.mouse.position;
    let mut open_dropdowns = Vec::new();
    dropdowns.for_each_mut(|entity, (data,)| {
        if data.open {
            open_dropdowns.push((entity, data.popup_container_entity));
        }
    });
    let mut open_menus = Vec::new();
    menus.for_each_mut(|entity, (data,)| {
        if data.open {
            open_menus.push((entity, data.popup_container_entity));
        }
    });
    let mut open_context_menus = Vec::new();
    context_menus.for_each_mut(|entity, (data,)| {
        if data.open {
            open_context_menus.push((entity, data.clone()));
        }
    });

    if is_mouse_over_popup(
        &mut nodes,
        &open_dropdowns,
        &open_menus,
        &open_context_menus,
        mouse_position,
    ) {
        return;
    }

    for (entity, popup_container) in open_dropdowns {
        if hide_popup(&mut nodes, popup_container) {
            crate::commands::widgets::world_interaction::ui_mark_layout_dirty(&mut dirty);
        }
        dropdowns.for_entity(entity, |(data,)| data.open = false);
    }

    for (entity, popup_container) in open_menus {
        if hide_popup(&mut nodes, popup_container) {
            crate::commands::widgets::world_interaction::ui_mark_layout_dirty(&mut dirty);
        }
        menus.for_entity(entity, |(data,)| data.open = false);
    }

    for (entity, data) in open_context_menus {
        hide_popup(&mut nodes, data.popup_entity);
        super::handle_context_menu::close_submenu_popups(&mut nodes, &data.item_defs);
        context_menus.for_entity(entity, |(widget_data,)| {
            widget_data.open = false;
            super::handle_context_menu::close_submenu_defs_state(&mut widget_data.item_defs);
        });
        overlays.active_context_menu = None;
    }
}

fn hide_popup(nodes: &mut UiNodes<'_, '_>, popup: nightshade::ecs::Entity) -> bool {
    let mut was_visible = false;
    nodes.for_entity(popup, |(node,)| {
        was_visible = node.visible;
        node.visible = false;
    });
    was_visible
}

fn is_mouse_over_popup(
    nodes: &mut UiNodes<'_, '_>,
    dropdowns: &[(nightshade::ecs::Entity, nightshade::ecs::Entity)],
    menus: &[(nightshade::ecs::Entity, nightshade::ecs::Entity)],
    context_menus: &[(
        nightshade::ecs::Entity,
        crate::components::UiContextMenuData,
    )],
    mouse_position: Vec2,
) -> bool {
    let popups = dropdowns
        .iter()
        .map(|(_, popup)| *popup)
        .chain(menus.iter().map(|(_, popup)| *popup))
        .chain(context_menus.iter().map(|(_, data)| data.popup_entity));
    for popup in popups {
        let mut over = false;
        nodes.for_entity(popup, |(node,)| {
            over = node.computed_rect.contains(mouse_position)
        });
        if over {
            return true;
        }
    }
    false
}
