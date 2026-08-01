use crate::components::DemoMesh;
use crate::resources::{PickingMode, PickingResources, UiHandles, mode_label};
use crate::theme::PANEL_RECT;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_ui::commands::widgets::world_layout::UiLayoutNodes;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::{RetainedUiDirty, RetainedUiFrame};

pub fn build(
    context: UiBuildContext,
    initial_mode: PickingMode,
    initial_show_volumes: bool,
    initial_spin: f32,
) -> UiHandles {
    let mut tree = UiTreeBuilder::new(context);
    let panel = tree.add_floating_panel("picking_demo", "Picking Demo", PANEL_RECT);
    let content = tree.panel_content(panel).expect("panel content");
    let mut mode_button = Entity::default();
    let mut status_label = Entity::default();
    let mut detail_label = Entity::default();
    let mut spin_slider = Entity::default();
    let mut debug_volumes_toggle = Entity::default();
    tree.in_parent(content, |tree| {
        tree.add_label("Hover for outline, click to lock.");
        tree.add_label("Drag to orbit, scroll to zoom.");
        tree.add_label("F1 toggles this panel.");
        mode_button = tree.add_button(mode_label(initial_mode));
        status_label = tree.add_label("Selected: none");
        detail_label = tree.add_label("");
        tree.add_label("Spin speed");
        spin_slider = tree.add_slider(initial_spin, 0.0, 5.0);
        debug_volumes_toggle = tree.add_checkbox("Show debug volumes", initial_show_volumes);
    });
    tree.finish();

    UiHandles {
        panel,
        mode_button,
        debug_volumes_toggle,
        spin_slider,
        status_label,
        detail_label,
    }
}

pub fn poll(
    mut picking: ResMut<PickingResources>,
    frame: Res<RetainedUiFrame>,
    mut widgets: Query<(Option<&UiSliderData>, Option<&UiCheckboxData>)>,
) {
    let picking = &mut *picking;
    let mut spin_speed = None;
    widgets.for_entity(picking.handles.spin_slider, |(slider, _)| {
        spin_speed = slider.map(|slider| slider.value);
    });
    if let Some(value) = spin_speed {
        picking.spin_speed = value;
    }
    let mut show_volumes = None;
    widgets.for_entity(picking.handles.debug_volumes_toggle, |(_, toggle)| {
        show_volumes = toggle.map(|toggle| toggle.value);
    });
    if let Some(value) = show_volumes {
        picking.show_debug_volumes = value;
    }

    let mode_clicked = ui_events(&frame).iter().any(|event| {
        matches!(event, UiEvent::ButtonClicked(entity) if *entity == picking.handles.mode_button)
    });
    if mode_clicked {
        picking.highlighted_entity = None;
        picking.mode = match picking.mode {
            PickingMode::GpuHover => PickingMode::CpuHover,
            PickingMode::CpuHover => PickingMode::GpuHover,
        };
    }
}

pub fn refresh_text(
    mut picking: ResMut<PickingResources>,
    input: Res<Input>,
    mut commands: Commands,
    mut labels: Query<(&DemoMesh,)>,
) {
    let picking = &mut *picking;
    ui_button_set_text(
        &mut commands,
        picking.handles.mode_button,
        mode_label(picking.mode),
    );

    let status = match (picking.selected_entity, picking.highlighted_entity) {
        (Some(selected), Some(hovered)) if selected == hovered => {
            format!("Selected: {} (hover)", label_for(&mut labels, selected))
        }
        (Some(selected), Some(hovered)) => format!(
            "Selected: {}    Hover: {}",
            label_for(&mut labels, selected),
            label_for(&mut labels, hovered)
        ),
        (Some(selected), None) => format!("Selected: {}", label_for(&mut labels, selected)),
        (None, Some(hovered)) => format!("Hover: {}", label_for(&mut labels, hovered)),
        (None, None) => "Selected: none".to_string(),
    };
    ui_set_text(&mut commands, picking.handles.status_label, &status);

    let cursor = input.mouse.position;
    let detail = match &picking.last_pick_result {
        Some(result) if result.depth > 0.0 => format!(
            "Mouse {:.0}, {:.0}    Hit ({:.2}, {:.2}, {:.2}) entity {:?}",
            cursor.x,
            cursor.y,
            result.world_position.x,
            result.world_position.y,
            result.world_position.z,
            result.entity_id
        ),
        Some(result) => format!(
            "Mouse {:.0}, {:.0}    empty depth, entity {:?}",
            cursor.x, cursor.y, result.entity_id
        ),
        None => format!(
            "Mouse {:.0}, {:.0}    no gpu result has arrived yet",
            cursor.x, cursor.y
        ),
    };
    ui_set_text(&mut commands, picking.handles.detail_label, &detail);
}

pub fn drain_keyboard(
    mut picking: ResMut<PickingResources>,
    reader: EventReader<AppEvent>,
    mut window: ResMut<Window>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    mut nodes: UiLayoutNodes<'_, '_>,
) {
    let picking = &mut *picking;
    for event in reader.iter() {
        if let AppEvent::Keyboard {
            key,
            state: KeyState::Pressed,
        } = event
        {
            match key {
                KeyCode::Escape => window.should_exit = true,
                KeyCode::F1 => {
                    ui_toggle_visible(&mut nodes, &mut dirty, &mut commands, picking.handles.panel)
                }
                _ => {}
            }
        }
    }
}

fn label_for(labels: &mut Query<'_, (&DemoMesh,)>, entity: Entity) -> String {
    let mut name = None;
    labels.for_entity(entity, |(mesh,)| name = Some(mesh.label.to_string()));
    name.unwrap_or_else(|| "?".to_string())
}
