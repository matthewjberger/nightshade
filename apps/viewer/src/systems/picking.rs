use nightshade::prelude::*;
use nightshade_camera::resources::ActiveCamera;
use nightshade_graphics::commands::gpu_picking::request_pick_of;
use nightshade_platform::contract::input::MouseState;
use nightshade_platform::prelude::*;
use nightshade_render::components::lines::{Line, Lines};
use nightshade_render::prelude::*;
use nightshade_scene::contract::GlobalTransform;
use nightshade_scene::contract::primitives::{Name, Visibility};
use nightshade_ui::resources::{RetainedUiInteraction, RetainedUiRuntime};

use crate::commands::picking::{cycle_select, reset_cycle};
use crate::data::{PICK_PATIENCE, PickKind, ViewerAction};
use crate::queries::camera::box_corners;
use crate::queries::lines::BOX_EDGES;
use crate::queries::picking::find_entity_by_id;
use crate::resources::ViewerResources;
use crate::theme::{GROUP_COLOR, HOVER_COLOR};

const CLICK_DISTANCE: f32 = 5.0;

pub fn request_pick(
    mut viewer: ResMut<ViewerResources>,
    input: Res<Input>,
    mut gpu_picking: ResMut<GpuPicking>,
    window: Res<nightshade_platform::contract::window::Window>,
    viewport: Res<Viewport>,
    settings: Res<RenderSettings>,
    runtime: Res<RetainedUiRuntime>,
    interaction: Res<RetainedUiInteraction>,
    storage: &mut nightshade::ecs::Storage,
) {
    let viewer = &mut *viewer;
    let over_interface = nightshade_ui::ui_pointer_target_of(&runtime, &interaction, storage)
        .is_some()
        || viewer.gizmo.hovered.is_some()
        || viewer.gizmo.drag.is_some();
    let position = input.mouse.position;
    let held = input.mouse.state.contains(MouseState::LEFT_CLICKED)
        || input.mouse.state.contains(MouseState::RIGHT_CLICKED)
        || input.mouse.state.contains(MouseState::MIDDLE_CLICKED);

    if input.mouse.state.contains(MouseState::LEFT_JUST_PRESSED) {
        viewer.picking.last_press = (!over_interface).then_some(position);
    }

    if input.mouse.state.contains(MouseState::LEFT_JUST_RELEASED)
        && let Some(press) = viewer.picking.last_press.take()
        && !over_interface
        && (position - press).magnitude() <= CLICK_DISTANCE
    {
        viewer.picking.click_wanted = Some(position);
        viewer.picking.click_adds = input.keyboard.is_key_pressed(KeyCode::ControlLeft)
            || input.keyboard.is_key_pressed(KeyCode::ControlRight);
    }

    if viewer.picking.outstanding.is_some() {
        viewer.picking.waited += 1;
        if viewer.picking.waited > PICK_PATIENCE {
            viewer.picking.outstanding = None;
            viewer.picking.waited = 0;
        }
        return;
    }

    if let Some(click) = viewer.picking.click_wanted.take() {
        request_pick_of(
            viewport.active_viewport_rect,
            window.cached_viewport_size,
            settings.render_scale,
            &mut gpu_picking,
            click,
        );
        viewer.picking.outstanding = Some(PickKind::Click);
        viewer.picking.waited = 0;
        return;
    }

    if over_interface || held {
        viewer.picking.hovered = None;
        return;
    }
    if viewer.picking.hover_position == Some(position) {
        return;
    }
    viewer.picking.hover_position = Some(position);
    request_pick_of(
        viewport.active_viewport_rect,
        window.cached_viewport_size,
        settings.render_scale,
        &mut gpu_picking,
        position,
    );
    viewer.picking.outstanding = Some(PickKind::Hover);
    viewer.picking.waited = 0;
}

pub fn apply_pick(
    mut viewer: ResMut<ViewerResources>,
    mut gpu_picking: ResMut<GpuPicking>,
    active_camera: Res<ActiveCamera>,
    storage: &mut nightshade::ecs::Storage,
) {
    let viewer = &mut *viewer;
    let Some(kind) = viewer.picking.outstanding else {
        return;
    };
    let Some(result) = gpu_picking.take_result() else {
        return;
    };
    viewer.picking.outstanding = None;
    viewer.picking.waited = 0;

    let leaf = if result.depth > 0.0 {
        result
            .entity_id
            .and_then(|id| find_entity_by_id(storage, id))
    } else {
        None
    };

    match kind {
        PickKind::Hover => viewer.picking.hovered = leaf,
        PickKind::Click => {
            if viewer.view.depth_of_field
                && result.depth > 0.0
                && let Some(camera) = active_camera
                    .0
                    .and_then(|camera| storage.get::<GlobalTransform>(camera))
            {
                let distance = (result.world_position - camera.translation()).norm();
                viewer
                    .actions
                    .push(ViewerAction::SetFocusDistance(distance));
            }
            match leaf {
                Some(leaf) if viewer.picking.click_adds => {
                    reset_cycle(viewer);
                    viewer.actions.push(ViewerAction::ToggleInSelection(leaf));
                }
                Some(leaf) => cycle_select(viewer, storage, leaf),
                None if viewer.picking.click_adds => {}
                None => {
                    reset_cycle(viewer);
                    viewer.actions.push(ViewerAction::Select(None));
                }
            }
        }
    }
}

pub fn draw_hover(
    mut viewer: ResMut<ViewerResources>,
    mut commands: Commands,
    storage: &mut nightshade::ecs::Storage,
) {
    let viewer = &mut *viewer;
    let target = viewer
        .picking
        .hovered
        .filter(|entity| storage.is_alive(*entity))
        .filter(|entity| viewer.selection.selected != Some(*entity))
        .filter(|entity| !viewer.selection.others.contains(entity));
    let mut boxes: Vec<([Vec3; 8], Vec4)> = viewer
        .selection
        .others
        .iter()
        .filter_map(|entity| box_corners(storage, *entity))
        .map(|corners| (corners, GROUP_COLOR))
        .collect();
    if let Some(corners) = target.and_then(|entity| box_corners(storage, entity)) {
        boxes.push((corners, HOVER_COLOR));
    }

    let lines_entity = match viewer.picking.hover_lines {
        Some(entity) if storage.has::<Lines>(entity) => entity,
        _ => {
            let entity = commands.spawn((
                Name("Hover Box".to_string()),
                Lines::new_always_on_top(Vec::new()),
                Visibility { visible: false },
                GlobalTransform::default(),
            ));
            viewer.picking.hover_lines = Some(entity);
            return;
        }
    };

    if boxes.is_empty() {
        if let Some(visibility) = storage.get_mut::<Visibility>(lines_entity)
            && visibility.visible
        {
            visibility.visible = false;
            if let Some(lines) = storage.get_mut::<Lines>(lines_entity) {
                lines.clear();
            }
        }
        return;
    }

    if let Some(visibility) = storage.get_mut::<Visibility>(lines_entity) {
        visibility.visible = true;
    }
    if let Some(lines) = storage.get_mut::<Lines>(lines_entity) {
        lines.clear();
        for (corners, color) in boxes {
            for (start, end) in BOX_EDGES {
                lines.push(Line {
                    start: corners[start],
                    end: corners[end],
                    color,
                });
            }
        }
    }
}
