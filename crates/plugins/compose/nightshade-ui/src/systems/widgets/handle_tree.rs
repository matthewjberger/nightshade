use crate::data::state::UiStateTrait as _;
use nalgebra_glm;
use nalgebra_glm::Vec4;
use nightshade::ecs::system_param::{Query, Res, ResMut};
use nightshade_platform::KeyCode;

use super::handle_scroll::{
    ScrollFrameInputs, UiInteractions, UiNodes, UiParents, apply_scroll_frame, is_within,
};

pub type UiTreeViews<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiTreeViewData,)>;

pub type UiTreeNodes<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiTreeNodeData,)>;

pub type UiRowPaint<'world, 'item> = Query<
    'world,
    (
        &'item mut crate::components::UiNodeColor,
        &'item mut crate::components::UiStateWeights,
    ),
>;

struct RowPaint {
    row_entity: nightshade::ecs::Entity,
    selected: bool,
    color: Vec4,
}

fn apply_row_paint(rows: &mut UiRowPaint<'_, '_>, paint: &RowPaint) {
    rows.for_entity(paint.row_entity, |(color, weights)| {
        color.colors[crate::data::state::UiSelected::INDEX] = paint.selected.then_some(paint.color);
        weights.weights[crate::data::state::UiSelected::INDEX] =
            if paint.selected { 1.0 } else { 0.0 };
    });
}

#[derive(Clone)]
struct NodeFacts {
    row_entity: nightshade::ecs::Entity,
    arrow_entity: nightshade::ecs::Entity,
    arrow_text_slot: usize,
    children_container: nightshade::ecs::Entity,
    parent_node: Option<nightshade::ecs::Entity>,
    expanded: bool,
    selected: bool,
    lazy: bool,
    lazy_loaded: bool,
    user_data: u64,
}

fn node_facts(
    tree_nodes: &mut UiTreeNodes<'_, '_>,
    entity: nightshade::ecs::Entity,
) -> Option<NodeFacts> {
    let mut facts = None;
    tree_nodes.for_entity(entity, |(data,)| {
        facts = Some(NodeFacts {
            row_entity: data.row_entity,
            arrow_entity: data.arrow_entity,
            arrow_text_slot: data.arrow_text_slot,
            children_container: data.children_container,
            parent_node: data.parent_node,
            expanded: data.expanded,
            selected: data.selected,
            lazy: data.lazy,
            lazy_loaded: data.lazy_loaded,
            user_data: data.user_data,
        })
    });
    facts
}

struct Toggle {
    node: nightshade::ecs::Entity,
    arrow_text_slot: usize,
    children_container: nightshade::ecs::Entity,
    expanded: bool,
}

fn apply_toggle(
    text_state: &mut nightshade_text::contract::resources::TextState,
    nodes: &mut UiNodes<'_, '_>,
    tree_nodes: &mut UiTreeNodes<'_, '_>,
    toggle: &Toggle,
) {
    text_state.cache.set_text(
        toggle.arrow_text_slot,
        if toggle.expanded {
            "\u{25BC}"
        } else {
            "\u{25B6}"
        },
    );
    nodes.for_entity(toggle.children_container, |(node,)| {
        node.visible = toggle.expanded
    });
    tree_nodes.for_entity(toggle.node, |(data,)| data.expanded = toggle.expanded);
}

pub fn ui_tree_view_system(
    theme: Res<crate::resources::ThemeState>,
    timing: Res<crate::resources::RetainedUiTiming>,
    mirror: Res<crate::resources::RetainedUiInputMirror>,
    interaction: Res<crate::resources::RetainedUiInteraction>,
    input: Res<nightshade_platform::contract::input::Input>,
    window: Res<nightshade_platform::contract::window::Window>,
    hierarchy: Res<nightshade::ecs::HierarchyIndex>,
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut text_state: ResMut<nightshade_text::contract::resources::TextState>,
    mut trees: UiTreeViews<'_, '_>,
    mut tree_nodes: UiTreeNodes<'_, '_>,
    mut nodes: UiNodes<'_, '_>,
    mut rows: UiRowPaint<'_, '_>,
    mut interactions: UiInteractions<'_, '_>,
    mut parents: UiParents<'_, '_>,
) {
    let accent = theme.active_theme().accent_color;
    let selected_bg = Vec4::new(accent.x, accent.y, accent.z, 0.3);
    let mouse_position = input.mouse.position;
    let dpi_scale = window.cached_scale_factor;
    let ctrl_held = mirror.ctrl_held;
    let focused_entity = interaction.focused_entity;

    let mut pending = Vec::new();
    trees.for_each_mut(|entity, (data,)| {
        data.changed = false;
        data.context_menu_node = None;
        pending.push((entity, data.clone()));
    });

    for (tree_entity, data) in pending {
        let node_entities = data.node_entities.clone();
        let multi_select = data.multi_select;

        for node_entity in &node_entities {
            let Some(facts) = node_facts(&mut tree_nodes, *node_entity) else {
                continue;
            };

            let mut clicked = false;
            let mut double_clicked = false;
            let mut right_clicked = false;
            interactions.for_entity(facts.row_entity, |(row,)| {
                clicked = row.clicked;
                double_clicked = row.double_clicked;
                right_clicked = row.right_clicked;
            });

            let mut arrow_rect = None;
            nodes.for_entity(facts.arrow_entity, |(node,)| {
                arrow_rect = Some(node.computed_rect)
            });
            let arrow_clicked =
                clicked && arrow_rect.is_some_and(|rect| rect.contains(mouse_position));

            if clicked && !arrow_clicked {
                if !multi_select || !ctrl_held {
                    for previous in &data.selected_nodes {
                        if *previous == *node_entity {
                            continue;
                        }
                        let Some(previous_facts) = node_facts(&mut tree_nodes, *previous) else {
                            continue;
                        };
                        apply_row_paint(
                            &mut rows,
                            &RowPaint {
                                row_entity: previous_facts.row_entity,
                                selected: false,
                                color: selected_bg,
                            },
                        );
                        tree_nodes.for_entity(*previous, |(node,)| node.selected = false);
                    }
                }

                let new_selected = if ctrl_held && multi_select {
                    !facts.selected
                } else {
                    true
                };

                apply_row_paint(
                    &mut rows,
                    &RowPaint {
                        row_entity: facts.row_entity,
                        selected: new_selected,
                        color: selected_bg,
                    },
                );
                tree_nodes.for_entity(*node_entity, |(node,)| node.selected = new_selected);

                let mut new_selection = Vec::new();
                for candidate in &node_entities {
                    tree_nodes.for_entity(*candidate, |(node,)| {
                        if node.selected {
                            new_selection.push(*candidate);
                        }
                    });
                }
                trees.for_entity(tree_entity, |(tree,)| {
                    tree.selected_nodes = new_selection;
                    tree.changed = true;
                });

                frame
                    .events
                    .push(crate::resources::UiEvent::TreeNodeSelected {
                        tree: tree_entity,
                        node: *node_entity,
                        selected: new_selected,
                    });
            }

            if arrow_clicked || double_clicked {
                let new_expanded = !facts.expanded;
                if new_expanded && facts.lazy && !facts.lazy_loaded {
                    frame
                        .events
                        .push(crate::resources::UiEvent::TreeNodeExpandRequested {
                            tree: tree_entity,
                            node: *node_entity,
                            user_data: facts.user_data,
                        });
                } else {
                    apply_toggle(
                        &mut text_state,
                        &mut nodes,
                        &mut tree_nodes,
                        &Toggle {
                            node: *node_entity,
                            arrow_text_slot: facts.arrow_text_slot,
                            children_container: facts.children_container,
                            expanded: new_expanded,
                        },
                    );
                    frame
                        .events
                        .push(crate::resources::UiEvent::TreeNodeToggled {
                            tree: tree_entity,
                            node: *node_entity,
                            expanded: new_expanded,
                        });
                }
            }

            if right_clicked {
                trees.for_entity(tree_entity, |(tree,)| {
                    tree.context_menu_node = Some(*node_entity)
                });
                frame
                    .events
                    .push(crate::resources::UiEvent::TreeNodeContextMenu {
                        tree: tree_entity,
                        node: *node_entity,
                        position: mouse_position,
                    });
            }
        }

        if focused_entity == Some(tree_entity) || focused_entity == Some(data.content_entity) {
            let mut visible_nodes = Vec::new();
            for candidate in &node_entities {
                let Some(facts) = node_facts(&mut tree_nodes, *candidate) else {
                    continue;
                };
                let mut current_parent = facts.parent_node;
                let mut visible = true;
                while let Some(parent_entity) = current_parent {
                    let Some(parent_facts) = node_facts(&mut tree_nodes, parent_entity) else {
                        break;
                    };
                    if !parent_facts.expanded {
                        visible = false;
                        break;
                    }
                    current_parent = parent_facts.parent_node;
                }
                if visible {
                    visible_nodes.push(*candidate);
                }
            }

            let current_focused = data
                .selected_nodes
                .first()
                .copied()
                .or_else(|| visible_nodes.first().copied());
            let current_index = current_focused
                .and_then(|focused| visible_nodes.iter().position(|node| *node == focused));

            for &(key, pressed) in &mirror.frame_keys {
                if !pressed {
                    continue;
                }
                let new_focus = match key {
                    KeyCode::ArrowDown => current_index
                        .map(|index| (index + 1).min(visible_nodes.len().saturating_sub(1)))
                        .or(Some(0)),
                    KeyCode::ArrowUp => current_index
                        .map(|index| index.saturating_sub(1))
                        .or(Some(0)),
                    KeyCode::ArrowRight => {
                        if let Some(focus) = current_focused
                            && let Some(facts) = node_facts(&mut tree_nodes, focus)
                            && !facts.expanded
                        {
                            apply_toggle(
                                &mut text_state,
                                &mut nodes,
                                &mut tree_nodes,
                                &Toggle {
                                    node: focus,
                                    arrow_text_slot: facts.arrow_text_slot,
                                    children_container: facts.children_container,
                                    expanded: true,
                                },
                            );
                            frame
                                .events
                                .push(crate::resources::UiEvent::TreeNodeToggled {
                                    tree: tree_entity,
                                    node: focus,
                                    expanded: true,
                                });
                        }
                        None
                    }
                    KeyCode::ArrowLeft => {
                        if let Some(focus) = current_focused
                            && let Some(facts) = node_facts(&mut tree_nodes, focus)
                        {
                            if facts.expanded {
                                apply_toggle(
                                    &mut text_state,
                                    &mut nodes,
                                    &mut tree_nodes,
                                    &Toggle {
                                        node: focus,
                                        arrow_text_slot: facts.arrow_text_slot,
                                        children_container: facts.children_container,
                                        expanded: false,
                                    },
                                );
                                frame
                                    .events
                                    .push(crate::resources::UiEvent::TreeNodeToggled {
                                        tree: tree_entity,
                                        node: focus,
                                        expanded: false,
                                    });
                                None
                            } else if let Some(parent) = facts.parent_node {
                                visible_nodes.iter().position(|node| *node == parent)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    KeyCode::Enter | KeyCode::Space => {
                        if let Some(focus) = current_focused
                            && let Some(facts) = node_facts(&mut tree_nodes, focus)
                        {
                            let new_expanded = !facts.expanded;
                            apply_toggle(
                                &mut text_state,
                                &mut nodes,
                                &mut tree_nodes,
                                &Toggle {
                                    node: focus,
                                    arrow_text_slot: facts.arrow_text_slot,
                                    children_container: facts.children_container,
                                    expanded: new_expanded,
                                },
                            );
                            frame
                                .events
                                .push(crate::resources::UiEvent::TreeNodeToggled {
                                    tree: tree_entity,
                                    node: focus,
                                    expanded: new_expanded,
                                });
                        }
                        None
                    }
                    _ => None,
                };

                let Some(new_index) = new_focus else {
                    continue;
                };
                if new_index >= visible_nodes.len() {
                    continue;
                }
                let new_node = visible_nodes[new_index];

                if let Some(old) = current_focused
                    && let Some(old_facts) = node_facts(&mut tree_nodes, old)
                {
                    apply_row_paint(
                        &mut rows,
                        &RowPaint {
                            row_entity: old_facts.row_entity,
                            selected: false,
                            color: selected_bg,
                        },
                    );
                    tree_nodes.for_entity(old, |(node,)| node.selected = false);
                }

                if let Some(new_facts) = node_facts(&mut tree_nodes, new_node) {
                    apply_row_paint(
                        &mut rows,
                        &RowPaint {
                            row_entity: new_facts.row_entity,
                            selected: true,
                            color: selected_bg,
                        },
                    );
                    tree_nodes.for_entity(new_node, |(node,)| node.selected = true);
                }

                trees.for_entity(tree_entity, |(tree,)| {
                    tree.selected_nodes = vec![new_node];
                    tree.changed = true;
                });

                frame
                    .events
                    .push(crate::resources::UiEvent::TreeNodeSelected {
                        tree: tree_entity,
                        node: new_node,
                        selected: true,
                    });
            }
        }

        let scroll_under_cursor = interaction
            .hovered_entity
            .is_some_and(|hovered| is_within(&mut parents, hovered, tree_entity));
        let children: Vec<nightshade::ecs::Entity> =
            hierarchy.children(data.content_entity).to_vec();
        let result = apply_scroll_frame(
            &mut nodes,
            &mut interactions,
            &children,
            ScrollFrameInputs {
                entity: tree_entity,
                content_entity: data.content_entity,
                thumb_entity: data.thumb_entity,
                track_entity: data.track_entity,
                scroll_offset: data.scroll_offset,
                thumb_dragging: data.thumb_dragging,
                thumb_drag_start_offset: data.thumb_drag_start_offset,
                snap_interval: None,
                scroll_delta: mirror.scroll_delta,
                mouse_position,
                dpi_scale,
                delta_time: timing.delta_time,
                scroll_under_cursor,
            },
        );

        trees.for_entity(tree_entity, |(tree,)| {
            tree.scroll_offset = result.scroll_offset;
            tree.thumb_dragging = result.thumb_dragging;
            tree.thumb_drag_start_offset = result.thumb_drag_start_offset;
        });
    }
}

pub type UiModalDialogs<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiModalDialogData,)>;

pub fn ui_modal_dialog_system(
    mirror: Res<crate::resources::RetainedUiInputMirror>,
    mut overlays: ResMut<crate::resources::RetainedUiOverlays>,
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut dialogs: UiModalDialogs<'_, '_>,
    mut nodes: UiNodes<'_, '_>,
    mut interactions: UiInteractions<'_, '_>,
) {
    let escape_pressed = mirror
        .frame_keys
        .iter()
        .any(|(key, pressed)| *pressed && *key == KeyCode::Escape);

    let mut pending = Vec::new();
    dialogs.for_each_mut(|entity, (data,)| {
        pending.push((
            entity,
            data.ok_button,
            data.cancel_button,
            data.backdrop_entity,
        ))
    });

    for (entity, ok_button, cancel_button, backdrop_entity) in pending {
        let mut visible = false;
        nodes.for_entity(entity, |(node,)| visible = node.visible);
        if !visible {
            dialogs.for_entity(entity, |(data,)| data.result = None);
            continue;
        }

        let mut clicked = |target: nightshade::ecs::Entity| {
            let mut hit = false;
            interactions.for_entity(target, |(interaction,)| hit = interaction.clicked);
            hit
        };

        let mut result = None;
        if ok_button.is_some_and(&mut clicked) {
            result = Some(true);
        }
        if cancel_button.is_some_and(&mut clicked) {
            result = Some(false);
        }
        if clicked(backdrop_entity) {
            result = Some(false);
        }
        if escape_pressed {
            result = Some(false);
        }

        let Some(confirmed) = result else {
            continue;
        };
        nodes.for_entity(backdrop_entity, |(node,)| node.visible = false);
        nodes.for_entity(entity, |(node,)| node.visible = false);
        dialogs.for_entity(entity, |(data,)| data.result = Some(confirmed));
        overlays.active_modal = None;
        frame
            .events
            .push(crate::resources::UiEvent::ModalClosed { entity, confirmed });
    }
}
