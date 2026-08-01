use nightshade::prelude::*;
use nightshade_camera::resources::ActiveCamera;
use nightshade_platform::contract::input::MouseState;
use nightshade_platform::contract::window::Window;
use nightshade_platform::prelude::Input;
use nightshade_platform::winit::keyboard::KeyCode;
use nightshade_render::resources::viewport::Viewport;
use nightshade_scene::contract::GlobalTransform;
use nightshade_ui::resources::{RetainedUiInteraction, RetainedUiRuntime};

use crate::commands::gizmo::{
    claim, continue_drag, draw_handles, end_drag, hide_handles, start_drag,
};
use crate::queries::gizmo::{basis_of, group_origin, hit_handle, pointer_ray};
use crate::queries::selection::group;
use crate::resources::ViewerResources;

const SCREEN_SIZE: f32 = 0.28;

pub fn update_gizmo(
    mut viewer: ResMut<ViewerResources>,
    input: Res<Input>,
    window: Res<Window>,
    viewport: Res<Viewport>,
    active_camera: Res<ActiveCamera>,
    runtime: Res<RetainedUiRuntime>,
    interaction: Res<RetainedUiInteraction>,
    mut capture: ResMut<nightshade_platform::contract::input_capture::InputCapture>,
    mut commands: Commands,
    storage: &mut nightshade::ecs::Storage,
) {
    let viewer = &mut *viewer;

    let Some(target) = viewer.selection.selected.filter(|_| viewer.gizmo.enabled) else {
        hide_handles(viewer, storage);
        claim(&mut capture, false);
        return;
    };
    if !storage.is_alive(target) {
        hide_handles(viewer, storage);
        claim(&mut capture, false);
        return;
    }

    let Some(mut basis) = basis_of(storage, target) else {
        hide_handles(viewer, storage);
        claim(&mut capture, false);
        return;
    };
    let group = group(viewer, storage);
    if let Some(origin) = group_origin(storage, &group) {
        basis.origin = origin;
    }
    let Some(ray) = pointer_ray(
        &window,
        &viewport,
        &active_camera,
        storage,
        input.mouse.position,
    ) else {
        hide_handles(viewer, storage);
        claim(&mut capture, false);
        return;
    };
    let camera_position = active_camera
        .0
        .and_then(|camera| storage.get::<GlobalTransform>(camera))
        .map(|global| global.translation())
        .unwrap_or_else(Vec3::zeros);
    let size = (basis.origin - camera_position).norm() * SCREEN_SIZE;
    let snap = viewer.view.snap
        || input.keyboard.is_key_pressed(KeyCode::ControlLeft)
        || input.keyboard.is_key_pressed(KeyCode::ControlRight);

    let over_interface =
        nightshade_ui::ui_pointer_target_of(&runtime, &interaction, storage).is_some();

    if viewer.gizmo.drag.is_some() {
        if input.mouse.state.contains(MouseState::LEFT_JUST_RELEASED) {
            end_drag(viewer, storage);
        } else {
            continue_drag(viewer, storage, &basis, &ray, size, snap);
        }
    } else {
        viewer.gizmo.hovered = (!over_interface)
            .then(|| hit_handle(viewer.gizmo.mode, &basis, &ray, size))
            .flatten();
        if input.mouse.state.contains(MouseState::LEFT_JUST_PRESSED)
            && let Some(handle) = viewer.gizmo.hovered
        {
            start_drag(viewer, storage, &group, &basis, &ray, handle);
        }
    }

    claim(
        &mut capture,
        viewer.gizmo.drag.is_some() || viewer.gizmo.hovered.is_some(),
    );
    draw_handles(viewer, &mut commands, storage, &basis, size);
}
