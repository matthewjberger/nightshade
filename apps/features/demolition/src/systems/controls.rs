use crate::resources::{Cursor, VoxelViewer, Watcher};
use crate::theme;
use nalgebra_glm::{Vec3, quat_angle_axis, quat_rotate_vec3};
use nightshade::prelude::*;
use nightshade_camera::prelude::Camera;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;

pub fn keep_the_cursor(input: Res<Input>, mut window: ResMut<Window>, mut cursor: ResMut<Cursor>) {
    let escape = input.keyboard.just_pressed(KeyCode::Escape);
    let click = input.mouse.state.contains(MouseState::LEFT_JUST_PRESSED);
    let locked = window.cursor_locked;
    if escape && !locked {
        window.should_exit = true;
        return;
    }
    let wanted = if escape {
        false
    } else if click && window.is_focused {
        true
    } else {
        locked
    };
    if locked != wanted {
        set_cursor_locked(&mut window, wanted);
    }
    if cursor.shown == wanted {
        set_cursor_visible(&mut window, !wanted);
        cursor.shown = !wanted;
    }
}

pub fn fly(
    input: Res<Input>,
    time: Res<Time>,
    window: Res<Window>,
    mut watcher: ResMut<Watcher>,
    mut viewer: ResMut<VoxelViewer>,
    mut cameras: Query<(&Camera, &mut LocalTransform)>,
) {
    let Some(camera) = watcher.camera else {
        return;
    };
    let delta_time = time.delta_time.min(0.05);
    let locked = window.cursor_locked;
    if locked {
        let look = input.mouse.raw_mouse_delta * theme::MOUSE_SENSITIVITY;
        watcher.yaw -= look.x;
        watcher.pitch = (watcher.pitch - look.y).clamp(-theme::PITCH_LIMIT, theme::PITCH_LIMIT);
    }

    let (sin_yaw, cos_yaw) = watcher.yaw.sin_cos();
    let level = Vec3::new(-sin_yaw, 0.0, -cos_yaw);
    let right = Vec3::new(cos_yaw, 0.0, -sin_yaw);
    let mut wish = Vec3::zeros();
    if locked {
        if input.keyboard.is_key_pressed(KeyCode::KeyW) {
            wish += level;
        }
        if input.keyboard.is_key_pressed(KeyCode::KeyS) {
            wish -= level;
        }
        if input.keyboard.is_key_pressed(KeyCode::KeyD) {
            wish += right;
        }
        if input.keyboard.is_key_pressed(KeyCode::KeyA) {
            wish -= right;
        }
        if input.keyboard.is_key_pressed(KeyCode::Space) {
            wish += Vec3::y();
        }
        if input.keyboard.is_key_pressed(KeyCode::ShiftLeft) {
            wish -= Vec3::y();
        }
    }
    if wish.norm_squared() > 0.0 {
        wish = wish.normalize();
    }
    let sprint = locked && input.keyboard.is_key_pressed(KeyCode::ControlLeft);
    let speed = if sprint {
        theme::FLY_SPEED * theme::SPRINT_FACTOR
    } else {
        theme::FLY_SPEED
    };
    let mut position = watcher.position + wish * speed * delta_time;
    position.y = position.y.max(theme::FLOOR_HEIGHT);
    watcher.position = position;

    let rotation =
        quat_angle_axis(watcher.yaw, &Vec3::y()) * quat_angle_axis(watcher.pitch, &Vec3::x());
    viewer.eye = position;
    viewer.forward = quat_rotate_vec3(&rotation, &Vec3::new(0.0, 0.0, -1.0));
    cameras.for_entity(camera, |(_, transform)| {
        transform.translation = position;
        transform.rotation = rotation;
    });
}
