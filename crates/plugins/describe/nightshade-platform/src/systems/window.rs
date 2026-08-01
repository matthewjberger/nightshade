use crate::contract::window::Window;
#[cfg(not(target_arch = "wasm32"))]
use nightshade_ecs::system_param::Res;
use nightshade_ecs::system_param::{MainThread, ResMut};

pub fn apply_requested_cursor_system(_main: MainThread, mut window: ResMut<Window>) {
    let Some(handle) = window.handle.clone() else {
        return;
    };
    let requested = window.requested_cursor;
    let applied = window.applied_cursor;
    if requested.is_some() && requested != applied {
        if let Some(cursor) = requested {
            handle.set_cursor(cursor);
        }
        window.applied_cursor = requested;
    } else if requested.is_none()
        && applied.is_some()
        && applied != Some(winit::window::CursorIcon::Default)
    {
        handle.set_cursor(winit::window::CursorIcon::Default);
        window.applied_cursor = Some(winit::window::CursorIcon::Default);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn recenter_locked_cursor_system(
    _main: MainThread,
    window: Res<Window>,
    mut input: ResMut<crate::contract::input::Input>,
) {
    if !window.cursor_locked {
        return;
    }
    let Some(handle) = window.handle.as_ref() else {
        return;
    };
    let size = handle.inner_size();
    if size.width == 0 || size.height == 0 {
        return;
    }
    let center_x = size.width as f32 / 2.0;
    let center_y = size.height as f32 / 2.0;
    if handle
        .set_cursor_position(winit::dpi::PhysicalPosition::new(
            center_x as f64,
            center_y as f64,
        ))
        .is_ok()
    {
        input.mouse.position = nalgebra_glm::vec2(center_x, center_y);
        input.mouse.position_delta = nalgebra_glm::vec2(0.0, 0.0);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn apply_cursor_state_system(_main: MainThread, mut window: ResMut<Window>) {
    let Some(handle) = window.handle.clone() else {
        return;
    };
    let locked = window.cursor_locked;
    let visible = window.show_cursor && !locked;

    if window.applied_cursor_locked != Some(locked) {
        if locked {
            if handle
                .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                .is_err()
            {
                let _ = handle.set_cursor_grab(winit::window::CursorGrabMode::Confined);
            }
        } else {
            let _ = handle.set_cursor_grab(winit::window::CursorGrabMode::None);
        }
        window.applied_cursor_locked = Some(locked);
    }

    if window.applied_cursor_visible != Some(visible) {
        handle.set_cursor_visible(visible);
        window.applied_cursor_visible = Some(visible);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn sync_cursor_lock_state_system(mut window: ResMut<Window>) {
    crate::commands::window::sync_cursor_lock_state(&mut window);
}
