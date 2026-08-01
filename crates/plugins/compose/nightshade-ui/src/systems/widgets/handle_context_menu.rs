use nalgebra_glm;
use nalgebra_glm::Vec2;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_platform::KeyCode;

use super::handle_frame::UiContextMenus;
use super::handle_scroll::{UiInteractions, UiNodes};

struct SubmenuInputs {
    hovered: Option<nightshade::ecs::Entity>,
    mouse_position: Vec2,
    dpi_scale: f32,
}

pub fn ui_context_menu_system(
    mirror: Res<crate::resources::RetainedUiInputMirror>,
    interaction: Res<crate::resources::RetainedUiInteraction>,
    input: Res<nightshade_platform::contract::input::Input>,
    window: Res<nightshade_platform::contract::window::Window>,
    mut overlays: ResMut<crate::resources::RetainedUiOverlays>,
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut text_state: ResMut<nightshade_text::contract::resources::TextState>,
    mut menus: UiContextMenus<'_, '_>,
    mut nodes: UiNodes<'_, '_>,
    mut interactions: UiInteractions<'_, '_>,
) {
    let inputs = SubmenuInputs {
        hovered: interaction.hovered_entity,
        mouse_position: input.mouse.position,
        dpi_scale: window.cached_scale_factor,
    };
    let escape_pressed = mirror
        .frame_keys
        .iter()
        .any(|(key, pressed)| *pressed && *key == KeyCode::Escape);
    let left_just_pressed = input
        .mouse
        .state
        .contains(nightshade_platform::contract::input::MouseState::LEFT_JUST_PRESSED);

    let mut pending = Vec::new();
    menus.for_each_mut(|entity, (data,)| pending.push((entity, data.clone())));

    for (entity, data) in pending {
        if !data.open {
            menus.for_entity(entity, |(widget_data,)| widget_data.clicked_item = None);
            continue;
        }

        let use_defs = !data.item_defs.is_empty();
        let mut clicked_command = None;
        let mut should_close = escape_pressed;

        if use_defs {
            clicked_command = check_defs_for_click(&mut interactions, &data.item_defs);
            handle_submenu_hover(&mut menus, &mut nodes, entity, &data.item_defs, &inputs);
        } else {
            for (index, item_entity) in data.item_entities.iter().enumerate() {
                let Some(item_entity) = item_entity else {
                    continue;
                };
                if super::handle_slider::row_interaction(&mut interactions, *item_entity).clicked {
                    clicked_command = Some(index);
                    break;
                }
            }
        }

        if clicked_command.is_some() {
            should_close = true;
        }

        if left_just_pressed {
            let is_inside = match inputs.hovered {
                Some(hovered) => {
                    data.item_entities.contains(&Some(hovered))
                        || hovered == data.popup_entity
                        || (use_defs
                            && is_entity_in_submenu_popups(
                                &data.item_defs,
                                hovered,
                                &mut nodes,
                                inputs.mouse_position,
                            ))
                }
                None => false,
            };
            if !is_inside {
                should_close = true;
            }
        }

        if !should_close {
            continue;
        }

        nodes.for_entity(data.popup_entity, |(node,)| node.visible = false);
        close_submenu_popups(&mut nodes, &data.item_defs);
        menus.for_entity(entity, |(widget_data,)| {
            widget_data.open = false;
            widget_data.clicked_item = clicked_command;
            close_submenu_defs_state(&mut widget_data.item_defs);
        });
        overlays.active_context_menu = None;

        if let Some(command_id) = clicked_command {
            menus.for_entity(entity, |(widget_data,)| {
                toggle_checkable_in_defs(
                    &mut widget_data.item_defs,
                    command_id,
                    &mut text_state.cache,
                );
            });
            let tag = lookup_tag(&data.item_defs, command_id);
            frame
                .events
                .push(crate::resources::UiEvent::ContextMenuItemClicked {
                    entity,
                    item_index: command_id,
                    tag,
                });
        }
    }
}

fn toggle_checkable_in_defs(
    defs: &mut [crate::components::ContextMenuItemDef],
    command_id: usize,
    text_cache: &mut nightshade_text::contract::resources::TextCache,
) -> bool {
    for def in defs.iter_mut() {
        let def_command_id = def.command_id;
        if let crate::components::ContextMenuItemKind::Checkable {
            checked,
            check_text_slot,
        } = &mut def.kind
        {
            if def_command_id == Some(command_id) {
                *checked = !*checked;
                text_cache.set_text(*check_text_slot, if *checked { "\u{2713}" } else { "" });
                return true;
            }
        } else if let crate::components::ContextMenuItemKind::Submenu { children, .. } =
            &mut def.kind
            && toggle_checkable_in_defs(children, command_id, text_cache)
        {
            return true;
        }
    }
    false
}

fn lookup_tag(defs: &[crate::components::ContextMenuItemDef], command_id: usize) -> u32 {
    for def in defs {
        match &def.kind {
            crate::components::ContextMenuItemKind::Action
            | crate::components::ContextMenuItemKind::Checkable { .. }
                if def.command_id == Some(command_id) =>
            {
                return def.tag;
            }
            crate::components::ContextMenuItemKind::Submenu { children, .. } => {
                let tag = lookup_tag(children, command_id);
                if tag != 0 {
                    return tag;
                }
            }
            _ => {}
        }
    }
    0
}

fn check_defs_for_click(
    interactions: &mut UiInteractions<'_, '_>,
    defs: &[crate::components::ContextMenuItemDef],
) -> Option<usize> {
    for def in defs {
        match &def.kind {
            crate::components::ContextMenuItemKind::Action
            | crate::components::ContextMenuItemKind::Checkable { .. } => {
                if let Some(command_id) = def.command_id
                    && let Some(row) = def.row_entity
                    && super::handle_slider::row_interaction(interactions, row).clicked
                {
                    return Some(command_id);
                }
            }
            crate::components::ContextMenuItemKind::Submenu { children, .. } => {
                if let Some(command_id) = check_defs_for_click(interactions, children) {
                    return Some(command_id);
                }
            }
            _ => {}
        }
    }
    None
}

fn handle_submenu_hover(
    menus: &mut UiContextMenus<'_, '_>,
    nodes: &mut UiNodes<'_, '_>,
    menu_entity: nightshade::ecs::Entity,
    defs: &[crate::components::ContextMenuItemDef],
    inputs: &SubmenuInputs,
) {
    for (index, def) in defs.iter().enumerate() {
        let crate::components::ContextMenuItemKind::Submenu {
            popup_entity,
            open,
            children,
            ..
        } = &def.kind
        else {
            continue;
        };
        let child_popup = *popup_entity;
        let was_open = *open;
        let row_hovered = def.row_entity.is_some() && inputs.hovered == def.row_entity;

        if row_hovered && !was_open {
            let row_rect = def
                .row_entity
                .and_then(|row| super::handle_slider::node_rect(nodes, row));
            if let Some(rect) = row_rect {
                let position = Vec2::new(rect.max.x, rect.min.y) / inputs.dpi_scale;
                nodes.for_entity(child_popup, |(node,)| {
                    if let Some(crate::components::layout_types::UiLayoutType::Window(window)) =
                        node.base_layout.as_mut()
                    {
                        window.position = crate::data::units::Ab(position).into();
                        node.visible = true;
                    }
                });
            }
            menus.for_entity(menu_entity, |(menu_data,)| {
                if let Some(item_def) = menu_data.item_defs.get_mut(index)
                    && let crate::components::ContextMenuItemKind::Submenu { open, .. } =
                        &mut item_def.kind
                {
                    *open = true;
                }
            });
        } else if !row_hovered && was_open {
            let child_item_hovered = inputs.hovered == Some(child_popup)
                || is_entity_in_submenu_popups(
                    children,
                    inputs.hovered.unwrap_or_default(),
                    nodes,
                    inputs.mouse_position,
                );
            if !child_item_hovered {
                nodes.for_entity(child_popup, |(node,)| node.visible = false);
                close_submenu_popups(nodes, children);
                menus.for_entity(menu_entity, |(menu_data,)| {
                    if let Some(item_def) = menu_data.item_defs.get_mut(index)
                        && let crate::components::ContextMenuItemKind::Submenu {
                            open,
                            children,
                            ..
                        } = &mut item_def.kind
                    {
                        *open = false;
                        close_submenu_defs_state(children);
                    }
                });
            }
        }

        if was_open {
            let mut child_defs = None;
            menus.for_entity(menu_entity, |(menu_data,)| {
                if let Some(item_def) = menu_data.item_defs.get(index)
                    && let crate::components::ContextMenuItemKind::Submenu { children, .. } =
                        &item_def.kind
                {
                    child_defs = Some(children.clone());
                }
            });
            if let Some(child_defs) = child_defs {
                handle_submenu_hover(menus, nodes, menu_entity, &child_defs, inputs);
            }
        }
    }
}

fn is_entity_in_submenu_popups(
    defs: &[crate::components::ContextMenuItemDef],
    target: nightshade::ecs::Entity,
    nodes: &mut UiNodes<'_, '_>,
    mouse_position: Vec2,
) -> bool {
    for def in defs {
        if def.row_entity == Some(target) {
            return true;
        }
        let crate::components::ContextMenuItemKind::Submenu {
            popup_entity,
            children,
            ..
        } = &def.kind
        else {
            continue;
        };
        if *popup_entity == target {
            return true;
        }
        if super::handle_slider::node_rect(nodes, *popup_entity)
            .is_some_and(|rect| rect.contains(mouse_position))
        {
            return true;
        }
        if is_entity_in_submenu_popups(children, target, nodes, mouse_position) {
            return true;
        }
    }
    false
}

pub(super) fn close_submenu_popups(
    nodes: &mut UiNodes<'_, '_>,
    defs: &[crate::components::ContextMenuItemDef],
) {
    for def in defs {
        if let crate::components::ContextMenuItemKind::Submenu {
            popup_entity,
            children,
            ..
        } = &def.kind
        {
            nodes.for_entity(*popup_entity, |(node,)| node.visible = false);
            close_submenu_popups(nodes, children);
        }
    }
}

pub(super) fn close_submenu_defs_state(defs: &mut [crate::components::ContextMenuItemDef]) {
    for def in defs.iter_mut() {
        if let crate::components::ContextMenuItemKind::Submenu { open, children, .. } =
            &mut def.kind
        {
            *open = false;
            close_submenu_defs_state(children);
        }
    }
}
