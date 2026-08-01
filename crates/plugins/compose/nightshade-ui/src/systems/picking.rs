use crate::resources::{PickingEntry, PickingGrid};
use nalgebra_glm;

const DOUBLE_CLICK_TIME: f64 = 0.3;
const DOUBLE_CLICK_DISTANCE: f32 = 4.0;
const PICKING_CELL_SIZE: f32 = 64.0;

struct HitTestResult {
    entity: Option<nightshade::ecs::Entity>,
    disabled: bool,
}

pub(crate) fn is_descendant_of(
    storage: &nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
    ancestor: nightshade::ecs::Entity,
) -> bool {
    let mut current = entity;
    loop {
        if current == ancestor {
            return true;
        }
        if let Some(child_of) = storage.get::<nightshade::ecs::ChildOf>(current) {
            current = child_of.0;
        } else {
            return false;
        }
    }
}

fn unspin(point: nalgebra_glm::Vec2, pivot: nalgebra_glm::Vec2, angle: f32) -> nalgebra_glm::Vec2 {
    if angle == 0.0 {
        return point;
    }
    let turn = (-angle).cos();
    let lean = (-angle).sin();
    let offset = point - pivot;
    pivot
        + nalgebra_glm::Vec2::new(
            offset.x * turn - offset.y * lean,
            offset.x * lean + offset.y * turn,
        )
}

fn hit_test_grid(grid: &PickingGrid, mouse_position: nalgebra_glm::Vec2) -> HitTestResult {
    let Some(entries) = grid.query(mouse_position) else {
        return HitTestResult {
            entity: None,
            disabled: false,
        };
    };

    for entry in entries {
        if let Some(clip) = entry.clip_rect
            && !clip.contains(mouse_position)
        {
            continue;
        }

        if entry
            .computed_rect
            .contains(unspin(mouse_position, entry.pivot, entry.rotation))
        {
            return HitTestResult {
                entity: Some(entry.entity),
                disabled: entry.disabled,
            };
        }
    }

    HitTestResult {
        entity: None,
        disabled: false,
    }
}

pub fn build_picking_grid(
    storage: &nightshade::ecs::Storage,
    nodes: &[(nightshade::ecs::Entity, f32)],
    viewport: (f32, f32),
) -> PickingGrid {
    let mut grid = PickingGrid::new(viewport.0, viewport.1, PICKING_CELL_SIZE);

    for &(entity, depth) in nodes {
        let Some(node) = storage.get::<crate::components::UiLayoutNode>(entity) else {
            continue;
        };
        if !node.visible || !node.pointer_events {
            continue;
        }
        if storage
            .get::<crate::components::UiNodeInteraction>(entity)
            .is_none()
        {
            continue;
        }

        let disabled = storage
            .get::<crate::components::UiNodeInteraction>(entity)
            .map(|interaction| interaction.disabled)
            .unwrap_or(false);

        grid.insert(PickingEntry {
            entity,
            depth,
            rotation: node.computed_rotation,
            pivot: node.computed_pivot,
            computed_rect: node.computed_rect,
            clip_rect: node.computed_clip_rect,
            disabled,
        });
    }

    grid.sort_cells();
    grid
}

fn hit_test_sorted_nodes(
    storage: &nightshade::ecs::Storage,
    sorted_nodes: &[(nightshade::ecs::Entity, f32)],
    mouse_position: nalgebra_glm::Vec2,
) -> HitTestResult {
    for &(entity, _) in sorted_nodes.iter().rev() {
        let (has_pointer_events, visible, computed_rect, computed_clip_rect, rotation, pivot) = {
            if let Some(node) = storage.get::<crate::components::UiLayoutNode>(entity) {
                (
                    node.pointer_events,
                    node.visible,
                    node.computed_rect,
                    node.computed_clip_rect,
                    node.computed_rotation,
                    node.computed_pivot,
                )
            } else {
                continue;
            }
        };

        if !visible || !has_pointer_events {
            continue;
        }

        if storage
            .get::<crate::components::UiNodeInteraction>(entity)
            .is_none()
        {
            continue;
        }

        if let Some(clip) = computed_clip_rect
            && !clip.contains(mouse_position)
        {
            continue;
        }

        if computed_rect.contains(unspin(mouse_position, pivot, rotation)) {
            let disabled = storage
                .get::<crate::components::UiNodeInteraction>(entity)
                .map(|interaction| interaction.disabled)
                .unwrap_or(false);
            return HitTestResult {
                entity: Some(entity),
                disabled,
            };
        }
    }
    HitTestResult {
        entity: None,
        disabled: false,
    }
}

pub fn ui_layout_picking_system(
    mut runtime: nightshade::ecs::system_param::ResMut<crate::resources::RetainedUiRuntime>,
    mut interaction: nightshade::ecs::system_param::ResMut<crate::resources::RetainedUiInteraction>,
    timing: nightshade::ecs::system_param::Res<crate::resources::RetainedUiTiming>,
    mut overlays: nightshade::ecs::system_param::ResMut<crate::resources::RetainedUiOverlays>,
    mut frame: nightshade::ecs::system_param::ResMut<crate::resources::RetainedUiFrame>,
    mut drag_drop: nightshade::ecs::system_param::ResMut<crate::resources::RetainedUiDragDrop>,
    mut mirror: nightshade::ecs::system_param::ResMut<crate::resources::RetainedUiInputMirror>,
    mut accessibility: nightshade::ecs::system_param::ResMut<
        crate::resources::RetainedUiAccessibility,
    >,
    mut window: nightshade::ecs::system_param::ResMut<
        nightshade_platform::contract::window::Window,
    >,
    input: nightshade::ecs::system_param::ResMut<nightshade_platform::contract::input::Input>,
    storage: &mut nightshade::ecs::Storage,
) {
    if !runtime.visible {
        return;
    }

    let focused_mouse = &input.mouse;
    let mouse_position = focused_mouse.position;
    let mouse_state = focused_mouse.state;
    let mouse_just_pressed =
        mouse_state.contains(nightshade_platform::contract::input::MouseState::LEFT_JUST_PRESSED);
    let mouse_just_released =
        mouse_state.contains(nightshade_platform::contract::input::MouseState::LEFT_JUST_RELEASED);
    let mouse_down =
        mouse_state.contains(nightshade_platform::contract::input::MouseState::LEFT_CLICKED);

    let previous_active = interaction.active_entity;
    let current_time = timing.current_time;

    let sorted_nodes = std::mem::take(&mut runtime.z_sorted_nodes);

    let interaction_entities: Vec<nightshade::ecs::Entity> = storage
        .query_entities::<(crate::components::UiNodeInteraction,)>()
        .collect();

    for entity in &interaction_entities {
        if let Some(interaction) = storage.get_mut::<crate::components::UiNodeInteraction>(*entity)
        {
            interaction.hovered = false;
            interaction.pressed = false;
            interaction.clicked = false;
            interaction.focused = false;
            interaction.dragging = false;
            interaction.double_clicked = false;
            interaction.right_clicked = false;
        }
    }

    let hit = {
        let grid_ref = runtime.picking_grid.as_ref();
        match grid_ref {
            Some(grid) => hit_test_grid(grid, mouse_position),
            None => hit_test_sorted_nodes(storage, &sorted_nodes, mouse_position),
        }
    };

    if let Some(entity) = hit.entity
        && !hit.disabled
        && let Some(interaction) = storage.get_mut::<crate::components::UiNodeInteraction>(entity)
    {
        interaction.hovered = true;
    }

    if mouse_just_pressed {
        overlays.focus_ring_visible = false;

        if let Some(entity) = hit.entity
            && !hit.disabled
        {
            interaction.active_entity = Some(entity);
            interaction.focused_entity = Some(entity);

            if let Some(interaction) =
                storage.get_mut::<crate::components::UiNodeInteraction>(entity)
            {
                interaction.pressed = true;
                interaction.drag_start = Some(mouse_position);
            }
        } else if hit.entity.is_none() {
            interaction.focused_entity = None;
        }
    }

    if mouse_down
        && let Some(active) = previous_active
        && let Some(interaction) = storage.get_mut::<crate::components::UiNodeInteraction>(active)
    {
        interaction.pressed = true;
        if interaction.drag_start.is_some() {
            interaction.dragging = true;
        }
    }

    if mouse_just_released {
        if let Some(active) = previous_active {
            if hit.entity == Some(active) {
                if let Some(interaction) =
                    storage.get_mut::<crate::components::UiNodeInteraction>(active)
                {
                    interaction.clicked = true;
                }

                let is_double =
                    if let Some((last_entity, last_time, last_position)) = interaction.last_click {
                        last_entity == active
                            && (current_time - last_time) < DOUBLE_CLICK_TIME
                            && (mouse_position - last_position).magnitude() < DOUBLE_CLICK_DISTANCE
                    } else {
                        false
                    };

                if is_double {
                    if let Some(interaction) =
                        storage.get_mut::<crate::components::UiNodeInteraction>(active)
                    {
                        interaction.double_clicked = true;
                    }
                    interaction.last_click = None;
                } else {
                    interaction.last_click = Some((active, current_time, mouse_position));
                }
            }
            if let Some(interaction) =
                storage.get_mut::<crate::components::UiNodeInteraction>(active)
            {
                interaction.drag_start = None;
                interaction.dragging = false;
            }
        }
        interaction.active_entity = None;
    }

    let right_just_released =
        mouse_state.contains(nightshade_platform::contract::input::MouseState::RIGHT_JUST_RELEASED);
    if right_just_released
        && !hit.disabled
        && let Some(entity) = hit.entity
        && let Some(interaction) = storage.get_mut::<crate::components::UiNodeInteraction>(entity)
    {
        interaction.right_clicked = true;
    }

    window.requested_cursor = if hit.disabled {
        None
    } else {
        hit.entity.and_then(|entity| {
            storage
                .get::<crate::components::UiNodeInteraction>(entity)
                .and_then(|interaction| interaction.cursor_icon)
        })
    };

    interaction.hovered_entity = if hit.disabled { None } else { hit.entity };

    let frame_keys = std::mem::take(&mut mirror.frame_keys);
    let shift_held = mirror.shift_held;

    for (key, is_pressed) in &frame_keys {
        if !is_pressed {
            continue;
        }
        match key {
            nightshade_platform::KeyCode::Tab => {
                let mut tabbable: Vec<(nightshade::ecs::Entity, i32)> = Vec::new();
                for entity in &interaction_entities {
                    if let Some(interaction) =
                        storage.get::<crate::components::UiNodeInteraction>(*entity)
                        && let Some(tab_index) = interaction.tab_index
                        && !interaction.disabled
                        && let Some(node) = storage.get::<crate::components::UiLayoutNode>(*entity)
                        && node.visible
                    {
                        tabbable.push((*entity, tab_index));
                    }
                }
                tabbable.sort_by_key(|(_, index)| *index);

                if let Some(modal) = overlays.active_modal {
                    tabbable.retain(|(entity, _)| is_descendant_of(storage, *entity, modal));
                } else if !overlays.popup_entities.is_empty() {
                    let popups = overlays.popup_entities.clone();
                    tabbable.retain(|(entity, _)| {
                        popups
                            .iter()
                            .any(|popup| is_descendant_of(storage, *entity, *popup))
                    });
                }

                if !tabbable.is_empty() {
                    let focused = interaction.focused_entity;
                    let current_index = focused.and_then(|focused_entity| {
                        tabbable
                            .iter()
                            .position(|(entity, _)| *entity == focused_entity)
                    });

                    let next_index = if shift_held {
                        match current_index {
                            Some(0) => tabbable.len() - 1,
                            Some(index) => index - 1,
                            None => tabbable.len() - 1,
                        }
                    } else {
                        match current_index {
                            Some(index) => (index + 1) % tabbable.len(),
                            None => 0,
                        }
                    };

                    let focused = tabbable[next_index].0;
                    interaction.focused_entity = Some(focused);
                    overlays.focus_ring_visible = true;

                    if let Some(interaction) =
                        storage.get::<crate::components::UiNodeInteraction>(focused)
                    {
                        let role_name =
                            interaction.accessible_role.as_ref().map(|role| match role {
                                crate::components::AccessibleRole::Button => "button",
                                crate::components::AccessibleRole::Slider => "slider",
                                crate::components::AccessibleRole::Checkbox => "checkbox",
                                crate::components::AccessibleRole::Radio => "radio",
                                crate::components::AccessibleRole::Toggle => "toggle",
                                crate::components::AccessibleRole::TextInput => "text input",
                                crate::components::AccessibleRole::TextArea => "text area",
                                crate::components::AccessibleRole::Dropdown => "dropdown",
                                crate::components::AccessibleRole::Tab => "tab",
                                crate::components::AccessibleRole::TabPanel => "tab panel",
                                crate::components::AccessibleRole::Tree => "tree",
                                crate::components::AccessibleRole::TreeItem => "tree item",
                                crate::components::AccessibleRole::Grid => "grid",
                                crate::components::AccessibleRole::GridCell => "grid cell",
                                crate::components::AccessibleRole::Dialog => "dialog",
                                crate::components::AccessibleRole::Alert => "alert",
                                crate::components::AccessibleRole::ProgressBar => "progress bar",
                                crate::components::AccessibleRole::Menu => "menu",
                                crate::components::AccessibleRole::MenuItem => "menu item",
                            });
                        let label = interaction.accessible_label.as_deref();
                        let announcement = match (label, role_name) {
                            (Some(label), Some(role)) => format!("{label}, {role}"),
                            (Some(label), None) => label.to_string(),
                            (None, Some(role)) => role.to_string(),
                            (None, None) => String::new(),
                        };
                        if !announcement.is_empty() {
                            accessibility.announce_queue.push(announcement);
                        }
                    }
                }
            }
            nightshade_platform::KeyCode::Enter | nightshade_platform::KeyCode::Space => {
                if overlays.focus_ring_visible
                    && let Some(focused) = interaction.focused_entity
                    && let Some(interaction) =
                        storage.get_mut::<crate::components::UiNodeInteraction>(focused)
                {
                    interaction.clicked = true;
                }
            }
            _ => {}
        }
    }

    if let Some(focused) = interaction.focused_entity
        && let Some(interaction) = storage.get_mut::<crate::components::UiNodeInteraction>(focused)
    {
        interaction.focused = true;
    }

    let dpi_scale = window.cached_scale_factor;
    let drag_threshold = 8.0f32 * dpi_scale;
    if mouse_down
        && let Some(active) = previous_active
        && drag_drop.active.is_none()
        && let Some(interaction) = storage.get::<crate::components::UiNodeInteraction>(active)
        && let Some(start) = interaction.drag_start
        && (mouse_position - start).magnitude() > drag_threshold
        && let Some(source) = storage.get::<crate::components::UiDragSource>(active)
    {
        let payload = source.payload.clone();
        drag_drop.active = Some(crate::resources::ActiveDrag {
            source: active,
            payload: payload.clone(),
            start_position: start,
            current_position: mouse_position,
        });
        frame.events.push(crate::resources::UiEvent::DragStarted {
            source: active,
            payload,
        });
    }

    if let Some(ref mut drag) = drag_drop.active {
        drag.current_position = mouse_position;
    }

    if let Some(ref drag) = drag_drop.active {
        let drag_source = drag.source;
        let drag_payload = drag.payload.clone();

        let current_over = hit.entity.filter(|&entity| {
            storage
                .get::<crate::components::UiDropTarget>(entity)
                .is_some_and(|target| target.filter.accepts(&drag_payload))
        });
        let previous_over = drag_drop.over_entity;

        if current_over != previous_over {
            if let Some(prev) = previous_over {
                frame.events.push(crate::resources::UiEvent::DragLeave {
                    target: prev,
                    source: drag_source,
                });
            }
            if let Some(curr) = current_over {
                frame.events.push(crate::resources::UiEvent::DragEnter {
                    target: curr,
                    source: drag_source,
                    payload: drag_payload.clone(),
                });
            }
            drag_drop.over_entity = current_over;
        }

        if let Some(over) = current_over {
            frame.events.push(crate::resources::UiEvent::DragOver {
                target: over,
                source: drag_source,
                position: mouse_position,
            });
        }
    }

    if mouse_just_released && drag_drop.active.is_some() {
        drag_drop.over_entity = None;
        let drag = drag_drop.active.take().unwrap();
        if let Some(target_entity) = hit.entity
            && storage
                .get::<crate::components::UiDropTarget>(target_entity)
                .is_some_and(|target| target.filter.accepts(&drag.payload))
        {
            frame.events.push(crate::resources::UiEvent::DragDropped {
                source: drag.source,
                target: target_entity,
                payload: drag.payload,
            });
        } else {
            frame.events.push(crate::resources::UiEvent::DragCancelled {
                source: drag.source,
            });
        }
    }

    for &(key, pressed) in &frame_keys {
        if pressed && key == nightshade_platform::KeyCode::Escape && drag_drop.active.is_some() {
            let drag = drag_drop.active.take().unwrap();
            frame.events.push(crate::resources::UiEvent::DragCancelled {
                source: drag.source,
            });
        }
    }

    if let Some(entity) = hit.entity
        && let Some(interaction) = storage.get::<crate::components::UiNodeInteraction>(entity)
        && (interaction.tooltip_text.is_some() || interaction.tooltip_entity.is_some())
    {
        let text = interaction.tooltip_text.clone().unwrap_or_default();
        let tooltip_entity = interaction.tooltip_entity;
        let should_reset = overlays
            .tooltip_state
            .as_ref()
            .is_none_or(|state| state.entity != entity);
        if should_reset {
            overlays.tooltip_state = Some(crate::resources::TooltipActiveState {
                entity,
                hover_start: current_time,
                text,
                tooltip_entity,
            });
        }
    } else {
        if let Some(tooltip_entity) = overlays
            .tooltip_state
            .as_ref()
            .and_then(|prev| prev.tooltip_entity)
            && let Some(node) = storage.get_mut::<crate::components::UiLayoutNode>(tooltip_entity)
        {
            node.visible = false;
        }
        overlays.tooltip_state = None;
    }

    mirror.frame_keys = frame_keys;
    runtime.z_sorted_nodes = sorted_nodes;
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::components::layout_types::FlowDirection;
    use crate::data::units::{Ab, Rl};
    use crate::resources::UiEvent;
    use nalgebra_glm::Vec2;
    use nightshade_platform::contract::input::MouseState;

    fn setup() -> nightshade::ecs::World {
        let mut world = nightshade::ecs::World::default();
        crate::insert_ui_resources(&mut world.resources);
        world
            .res_mut::<nightshade_platform::contract::window::Window>()
            .cached_viewport_size = Some((1000, 800));
        world
            .res_mut::<nightshade_platform::contract::window::Window>()
            .cached_scale_factor = 1.0;
        world
            .res_mut::<crate::resources::RetainedUiRuntime>()
            .visible = true;
        world
    }

    fn run_frame(world: &mut nightshade::ecs::World) {
        world
            .resources
            .res_mut::<nightshade::ecs::HierarchyIndex>()
            .sync(&mut world.storage);
        for mut system in crate::default_ui_systems() {
            system(world);
        }
    }

    fn center(storage: &nightshade::ecs::Storage, entity: nightshade::ecs::Entity) -> Vec2 {
        let rect = storage
            .get::<crate::components::UiLayoutNode>(entity)
            .expect("layout node")
            .computed_rect;
        (rect.min + rect.max) * 0.5
    }

    fn interactive_box(
        storage: &mut nightshade::ecs::Storage,
        resources: &mut nightshade::ecs::ResourceMap,
    ) -> nightshade::ecs::Entity {
        crate::commands::builder::tree_over(storage, resources, |tree| {
            tree.add_node()
                .boundary(Ab(Vec2::new(100.0, 100.0)), Ab(Vec2::new(300.0, 200.0)))
                .with_interaction()
                .entity()
        })
    }

    #[test]
    fn mouse_over_node_sets_hover() {
        let mut world = setup();
        let node = interactive_box(&mut world.storage, &mut world.resources);
        run_frame(&mut world);
        world
            .res_mut::<nightshade_platform::contract::input::Input>()
            .mouse
            .position = center(&world.storage, node);
        run_frame(&mut world);
        assert!(
            world
                .get::<crate::components::UiNodeInteraction>(node)
                .unwrap()
                .hovered,
            "node under the cursor should be hovered"
        );
    }

    #[test]
    fn mouse_outside_node_no_hover() {
        let mut world = setup();
        let node = interactive_box(&mut world.storage, &mut world.resources);
        run_frame(&mut world);
        world
            .res_mut::<nightshade_platform::contract::input::Input>()
            .mouse
            .position = Vec2::new(900.0, 700.0);
        run_frame(&mut world);
        assert!(
            !world
                .get::<crate::components::UiNodeInteraction>(node)
                .unwrap()
                .hovered,
            "node away from the cursor should not be hovered"
        );
    }

    #[test]
    fn press_focuses_node() {
        let mut world = setup();
        let node = interactive_box(&mut world.storage, &mut world.resources);
        run_frame(&mut world);
        world
            .res_mut::<nightshade_platform::contract::input::Input>()
            .mouse
            .position = center(&world.storage, node);
        world
            .res_mut::<nightshade_platform::contract::input::Input>()
            .mouse
            .state = MouseState::LEFT_JUST_PRESSED | MouseState::LEFT_CLICKED;
        run_frame(&mut world);
        assert_eq!(
            world
                .res::<crate::resources::RetainedUiInteraction>()
                .focused_entity,
            Some(node)
        );
    }

    #[test]
    fn button_click_emits_event() {
        let mut world = setup();
        let button =
            crate::commands::builder::tree_over(&mut world.storage, &mut world.resources, |tree| {
                let container = tree
                    .add_node()
                    .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
                    .flow(FlowDirection::Vertical, 0.0, 0.0)
                    .entity();

                tree.in_parent(container, |tree| tree.add_button("OK"))
            });
        run_frame(&mut world);
        let position = center(&world.storage, button);

        world
            .res_mut::<nightshade_platform::contract::input::Input>()
            .mouse
            .position = position;
        world
            .res_mut::<nightshade_platform::contract::input::Input>()
            .mouse
            .state = MouseState::LEFT_JUST_PRESSED | MouseState::LEFT_CLICKED;
        run_frame(&mut world);

        world
            .res_mut::<nightshade_platform::contract::input::Input>()
            .mouse
            .state = MouseState::LEFT_JUST_RELEASED;
        run_frame(&mut world);

        let clicked = world
            .res::<crate::resources::RetainedUiFrame>()
            .events
            .iter()
            .any(|event| matches!(event, UiEvent::ButtonClicked(entity) if *entity == button));
        assert!(
            clicked,
            "releasing over the button should emit ButtonClicked"
        );
    }

    #[test]
    fn outside_press_closes_dropdown() {
        let mut world = setup();
        let dropdown =
            crate::commands::builder::tree_over(&mut world.storage, &mut world.resources, |tree| {
                let container = tree
                    .add_node()
                    .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
                    .flow(FlowDirection::Vertical, 0.0, 0.0)
                    .entity();

                tree.in_parent(container, |tree| tree.add_dropdown(&["A", "B", "C"], 0))
            });
        run_frame(&mut world);
        world
            .get_mut::<crate::components::UiDropdownData>(dropdown)
            .unwrap()
            .open = true;

        world
            .res_mut::<nightshade_platform::contract::input::Input>()
            .mouse
            .position = Vec2::new(500.0, 700.0);
        world
            .res_mut::<nightshade_platform::contract::input::Input>()
            .mouse
            .state = MouseState::LEFT_JUST_PRESSED | MouseState::LEFT_CLICKED;
        run_frame(&mut world);

        assert!(
            !world
                .get::<crate::components::UiDropdownData>(dropdown)
                .unwrap()
                .open,
            "pressing outside an open dropdown should close it"
        );
    }
}
