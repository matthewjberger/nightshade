use nalgebra_glm;
use nightshade_platform::winit;

use crate::contract::{
    CameraMove, CameraSweep, PanOrbitCamera, PendingCameraMove, PendingCameraSweep, SweptDistance,
    ThirdPersonCamera, compute_pan_orbit_transform, compute_third_person_transform,
};
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_camera::components::{Camera, Smoothing};
use nightshade_input::contract::{Actions, EngineAction};
use nightshade_platform::contract::input::MouseState;
use nightshade_platform::queries::input_capture::{keyboard_captured, pointer_captured};

pub fn first_person_camera_look_system(
    active_camera: Res<nightshade_camera::resources::ActiveCamera>,
    input: Res<nightshade_platform::contract::input::Input>,
    actions: Res<Actions<EngineAction>>,
    time: Res<nightshade_platform::contract::time::Time>,
    window: Res<nightshade_platform::contract::window::Window>,
    mut cameras: Query<(&Camera, &mut nightshade_scene::contract::LocalTransform)>,
) {
    let Some(camera_entity) = active_camera.0 else {
        return;
    };

    let pitch_limit = 85_f32.to_radians();
    let cursor_locked = window.cursor_locked;
    let raw_delta = input.mouse.raw_mouse_delta;
    let has_mouse_input = raw_delta.x.abs() > 0.0 || raw_delta.y.abs() > 0.0;

    let look = actions.axis2(EngineAction::Look);
    let has_gamepad_input = look.x.abs() > 0.0 || look.y.abs() > 0.0;

    if !has_mouse_input && !has_gamepad_input {
        return;
    }

    let delta_time = time.delta_time;

    cameras.for_entity(camera_entity, |(camera, camera_transform)| {
        let smoothing = camera.smoothing;
        let mouse_sensitivity = 0.004 * smoothing.map(|s| s.mouse_sensitivity).unwrap_or(0.5);
        let gamepad_sensitivity =
            (5.0 / 3.0) * smoothing.map(|s| s.gamepad_sensitivity).unwrap_or(1.5);

        let delta = if has_gamepad_input {
            Vec2::new(
                look.x * gamepad_sensitivity * delta_time,
                -look.y * gamepad_sensitivity * delta_time,
            )
        } else if cursor_locked {
            raw_delta * mouse_sensitivity
        } else {
            return;
        };

        let yaw = nalgebra_glm::quat_angle_axis(-delta.x, &Vec3::new(0.0, 1.0, 0.0));
        camera_transform.rotation = yaw * camera_transform.rotation;

        let forward =
            nalgebra_glm::quat_rotate_vec3(&camera_transform.rotation, &Vec3::new(0.0, 0.0, -1.0));
        let current_pitch = forward.y.asin();
        let new_pitch = current_pitch - delta.y;

        if new_pitch.abs() <= pitch_limit {
            let pitch = nalgebra_glm::quat_angle_axis(-delta.y, &Vec3::new(1.0, 0.0, 0.0));
            camera_transform.rotation *= pitch;
        }
    });
}

pub fn look_camera_system(
    active_camera: Res<nightshade_camera::resources::ActiveCamera>,
    capture: Res<nightshade_platform::contract::input_capture::InputCapture>,
    input: Res<nightshade_platform::contract::input::Input>,
    time: Res<nightshade_platform::contract::time::Time>,
    mut window: ResMut<nightshade_platform::contract::window::Window>,
    mut cameras: Query<
        (&mut Camera, &mut nightshade_scene::contract::LocalTransform),
        Without<crate::contract::PanOrbitCamera>,
    >,
) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static CAMERA_HAS_CURSOR: AtomicBool = AtomicBool::new(false);

    if pointer_captured(&capture) {
        return;
    }
    let Some(camera_entity) = active_camera.0 else {
        return;
    };

    let delta_time = time.delta_time;
    let raw_delta = input.mouse.raw_mouse_delta;
    let position_delta = input.mouse.position_delta;
    let right_clicked = input.mouse.state.contains(MouseState::RIGHT_CLICKED);
    let middle_clicked = input.mouse.state.contains(MouseState::MIDDLE_CLICKED);

    let mut controls_camera = false;

    cameras.for_entity(camera_entity, |(camera, local_transform)| {
        controls_camera = true;
        let right = local_transform.right_vector();
        let up = local_transform.up_vector();

        if let Some(smoothing) = camera.smoothing.as_mut() {
            if right_clicked {
                let smoothing_factor = if smoothing.mouse_smoothness > 0.0 {
                    1.0 - smoothing.mouse_smoothness.powi(7).powf(delta_time)
                } else {
                    1.0
                };
                smoothing.smoothed_mouse_delta = smoothing.smoothed_mouse_delta
                    * (1.0 - smoothing_factor)
                    + raw_delta * smoothing_factor;

                let pixels_to_radians = (std::f32::consts::PI / 1000.0) * smoothing.mouse_dpi_scale;
                let mut delta = smoothing.smoothed_mouse_delta
                    * smoothing.mouse_sensitivity
                    * pixels_to_radians;
                delta.x *= -1.0;
                delta.y *= -1.0;

                let yaw = nalgebra_glm::quat_angle_axis(delta.x, &Vec3::y());
                local_transform.rotation = yaw * local_transform.rotation;

                let forward = local_transform.forward_vector();
                let current_pitch = forward.y.asin();

                let new_pitch = current_pitch + delta.y;
                if new_pitch.abs() <= 89_f32.to_radians() {
                    let pitch = nalgebra_glm::quat_angle_axis(delta.y, &Vec3::x());
                    local_transform.rotation *= pitch;
                }
            } else {
                let decay_smoothness = (smoothing.mouse_smoothness * 0.5).max(0.01);
                let smoothing_factor = 1.0 - decay_smoothness.powi(7).powf(delta_time);
                smoothing.smoothed_mouse_delta *= 1.0 - smoothing_factor;
            }
        }

        if middle_clicked {
            let mut delta = position_delta * delta_time;
            delta.x *= -1.0;
            delta.y *= -1.0;
            local_transform.translation += right * delta.x;
            local_transform.translation += up * delta.y;
        }
    });

    if right_clicked && controls_camera {
        if !CAMERA_HAS_CURSOR.load(Ordering::Relaxed) {
            nightshade_platform::commands::window::set_pointer_grab(&mut window, true);
            CAMERA_HAS_CURSOR.store(true, Ordering::Relaxed);
        }
    } else if CAMERA_HAS_CURSOR.load(Ordering::Relaxed) {
        nightshade_platform::commands::window::set_pointer_grab(&mut window, false);
        CAMERA_HAS_CURSOR.store(false, Ordering::Relaxed);
    }
}

fn fly_speed(move_speed: f32, sprint_multiplier: f32, sprinting: bool) -> f32 {
    if sprinting {
        move_speed * sprint_multiplier
    } else {
        move_speed
    }
}

pub fn fly_camera_system(
    active_camera: Res<nightshade_camera::resources::ActiveCamera>,
    capture: Res<nightshade_platform::contract::input_capture::InputCapture>,
    actions: Res<Actions<EngineAction>>,
    time: Res<nightshade_platform::contract::time::Time>,
    mut cameras: Query<
        (&mut Camera, &mut nightshade_scene::contract::LocalTransform),
        Without<crate::contract::PanOrbitCamera>,
    >,
) {
    if keyboard_captured(&capture) {
        return;
    }
    let Some(camera_entity) = active_camera.0 else {
        return;
    };
    let delta_time = time.delta_time;

    let walk = actions.axis2(EngineAction::Move);
    let lift = actions.axis(EngineAction::Lift);
    let look = actions.axis2(EngineAction::Look);
    let sprinting = actions.pressed(EngineAction::Sprint);
    let mut target_movement = Vec3::new(walk.x, lift, walk.y);
    if target_movement.magnitude() > 1.0 {
        target_movement = target_movement.normalize();
    }

    cameras.for_entity(camera_entity, |(camera, local_transform)| {
        let (movement, base_speed, rotation_speed) = match camera.smoothing.as_mut() {
            Some(smoothing) => {
                let movement_factor = if smoothing.keyboard_smoothness > 0.0 {
                    1.0 - smoothing.keyboard_smoothness.powi(7).powf(delta_time)
                } else {
                    1.0
                };
                smoothing.smoothed_movement = smoothing.smoothed_movement * (1.0 - movement_factor)
                    + target_movement * movement_factor;
                let look_factor = if smoothing.gamepad_smoothness > 0.0 {
                    1.0 - smoothing.gamepad_smoothness.powi(7).powf(delta_time)
                } else {
                    1.0
                };
                smoothing.smoothed_gamepad_input =
                    smoothing.smoothed_gamepad_input * (1.0 - look_factor) + look * look_factor;
                (
                    smoothing.smoothed_movement,
                    fly_speed(smoothing.move_speed, smoothing.sprint_multiplier, sprinting),
                    smoothing.smoothed_gamepad_input * smoothing.gamepad_sensitivity * 2.0,
                )
            }
            None => {
                let defaults = Smoothing::default();
                (
                    target_movement,
                    fly_speed(defaults.move_speed, defaults.sprint_multiplier, sprinting),
                    Vec2::zeros(),
                )
            }
        };

        let forward = local_transform.forward_vector();
        let right = local_transform.right_vector();
        let up = local_transform.up_vector();

        local_transform.translation += forward * movement.z * base_speed * delta_time;
        local_transform.translation += right * movement.x * base_speed * delta_time;
        local_transform.translation += up * movement.y * base_speed * delta_time;

        if rotation_speed.x.abs() > 0.001 {
            let yaw = nalgebra_glm::quat_angle_axis(-(rotation_speed.x * delta_time), &Vec3::y());
            local_transform.rotation = yaw * local_transform.rotation;
        }

        if rotation_speed.y.abs() > 0.001 {
            let forward = local_transform.forward_vector();
            let current_pitch = forward.y.asin();
            let new_pitch = current_pitch + rotation_speed.y * delta_time;
            if new_pitch.abs() <= 89_f32.to_radians() {
                let pitch =
                    nalgebra_glm::quat_angle_axis(rotation_speed.y * delta_time, &Vec3::x());
                local_transform.rotation *= pitch;
            }
        }
    });
}

fn pan_orbit_viewport_size(window: &nightshade_platform::contract::window::Window) -> Vec2 {
    if let Some((width, height)) = window.cached_viewport_size {
        return Vec2::new((width as f32).max(1.0), (height as f32).max(1.0));
    }
    Vec2::new(1920.0, 1080.0)
}

fn pan_orbit_camera_y_fov(camera: &Camera) -> f32 {
    match camera.projection {
        nightshade_camera::components::Projection::Perspective(perspective) => {
            perspective.y_fov_rad
        }
        _ => 45.0_f32.to_radians(),
    }
}

pub fn touch_fly_camera_system(
    active_camera: Res<nightshade_camera::resources::ActiveCamera>,
    capture: Res<nightshade_platform::contract::input_capture::InputCapture>,
    input: Res<nightshade_platform::contract::input::Input>,
    time: Res<nightshade_platform::contract::time::Time>,
    mut cameras: Query<
        (&mut nightshade_scene::contract::LocalTransform,),
        Without<crate::contract::PanOrbitCamera>,
    >,
) {
    use nightshade_platform::contract::input::TouchGesture;

    if pointer_captured(&capture) {
        return;
    }
    let Some(camera_entity) = active_camera.0 else {
        return;
    };

    let gesture = input.touch.gesture;
    let delta_time = time.delta_time;
    let base_speed = 20.0;

    cameras.for_entity(camera_entity, |(local_transform,)| match gesture {
        TouchGesture::SingleDrag { delta } => {
            let pixels_to_radians = std::f32::consts::PI / 500.0;
            let rotation_delta = delta * pixels_to_radians;

            let yaw = nalgebra_glm::quat_angle_axis(-rotation_delta.x, &Vec3::y());
            local_transform.rotation = yaw * local_transform.rotation;

            let forward = local_transform.forward_vector();
            let current_pitch = forward.y.asin();

            let new_pitch = current_pitch - rotation_delta.y;
            if new_pitch.abs() <= 89_f32.to_radians() {
                let pitch = nalgebra_glm::quat_angle_axis(-rotation_delta.y, &Vec3::x());
                local_transform.rotation *= pitch;
            }
        }
        TouchGesture::TwoFingerDrag { delta } => {
            let right = local_transform.right_vector();
            let up = local_transform.up_vector();

            let pan_scale = delta_time * base_speed * 0.1;
            local_transform.translation += right * -delta.x * pan_scale;
            local_transform.translation += up * delta.y * pan_scale;
        }
        TouchGesture::Pinch { delta, .. } => {
            let forward = local_transform.forward_vector();
            local_transform.translation += forward * (delta * delta_time * base_speed * 0.05);
        }
        TouchGesture::None => {}
    });
}

pub fn pan_orbit_mouse_input(
    active_camera: Res<nightshade_camera::resources::ActiveCamera>,
    capture: Res<nightshade_platform::contract::input_capture::InputCapture>,
    input: Res<nightshade_platform::contract::input::Input>,
    viewport: Res<nightshade_render::resources::viewport::Viewport>,
    window: Res<nightshade_platform::contract::window::Window>,
    mut cameras: Query<(&mut crate::contract::PanOrbitCamera, &Camera)>,
) {
    use crate::contract::pan_orbit::{PanOrbitButton, PanOrbitModifier};

    let Some(camera_entity) = active_camera.0 else {
        return;
    };

    if input.touch.active_touch_count() > 0 {
        return;
    }

    let mouse_in_viewport = match viewport.active_viewport_rect {
        Some(viewport_rect) => viewport_rect.contains(input.mouse.position),
        None => true,
    };

    if !mouse_in_viewport || pointer_captured(&capture) {
        return;
    }

    let mouse_state = input.mouse.state;
    let drag_just_started = mouse_state.contains(MouseState::LEFT_JUST_PRESSED)
        || mouse_state.contains(MouseState::RIGHT_JUST_PRESSED)
        || mouse_state.contains(MouseState::MIDDLE_JUST_PRESSED);
    let position_delta = if drag_just_started {
        Vec2::new(0.0, 0.0)
    } else {
        input.mouse.position_delta
    };
    let wheel_delta = input.mouse.wheel_delta;

    let viewport_size = pan_orbit_viewport_size(&window);

    let left_clicked = mouse_state.contains(MouseState::LEFT_CLICKED);
    let right_clicked = mouse_state.contains(MouseState::RIGHT_CLICKED);
    let middle_clicked = mouse_state.contains(MouseState::MIDDLE_CLICKED);
    let scrolled = mouse_state.contains(MouseState::SCROLLED);

    let (shift_pressed, ctrl_pressed, alt_pressed) = {
        let keyboard = &input.keyboard;
        (
            keyboard.is_key_pressed(winit::keyboard::KeyCode::ShiftLeft)
                || keyboard.is_key_pressed(winit::keyboard::KeyCode::ShiftRight),
            keyboard.is_key_pressed(winit::keyboard::KeyCode::ControlLeft)
                || keyboard.is_key_pressed(winit::keyboard::KeyCode::ControlRight),
            keyboard.is_key_pressed(winit::keyboard::KeyCode::AltLeft)
                || keyboard.is_key_pressed(winit::keyboard::KeyCode::AltRight),
        )
    };

    let is_button_pressed = |button: PanOrbitButton| -> bool {
        match button {
            PanOrbitButton::Left => left_clicked,
            PanOrbitButton::Right => right_clicked,
            PanOrbitButton::Middle => middle_clicked,
        }
    };

    let is_modifier_satisfied = |modifier: Option<PanOrbitModifier>| -> bool {
        match modifier {
            None => true,
            Some(PanOrbitModifier::Shift) => shift_pressed,
            Some(PanOrbitModifier::Control) => ctrl_pressed,
            Some(PanOrbitModifier::Alt) => alt_pressed,
        }
    };

    cameras.for_entity(camera_entity, |(pan_orbit, camera)| {
        let camera_y_fov = pan_orbit_camera_y_fov(camera);
        let bindings = pan_orbit.bindings;
        if !pan_orbit.enabled {
            return;
        }

        let orbit_modifier_active = is_modifier_satisfied(bindings.orbit_modifier);
        let pan_modifier_active = is_modifier_satisfied(bindings.pan_modifier);

        let orbit_button_pressed = is_button_pressed(bindings.orbit);
        let pan_button_pressed = is_button_pressed(bindings.pan);

        let wants_zoom_drag = ctrl_pressed && middle_clicked;
        let wants_orbit = orbit_modifier_active
            && orbit_button_pressed
            && !wants_zoom_drag
            && !(bindings.pan_modifier.is_some() && pan_modifier_active);
        let wants_pan = pan_modifier_active
            && pan_button_pressed
            && !wants_zoom_drag
            && !(bindings.orbit_modifier.is_some() && orbit_modifier_active);

        if wants_zoom_drag {
            let zoom_basis = if position_delta.y < 0.0 {
                pan_orbit
                    .target_radius
                    .max(pan_orbit.pan_distance.unwrap_or(pan_orbit.target_radius))
            } else {
                pan_orbit.target_radius
            };
            let zoom_delta = -position_delta.y * zoom_basis * 0.01 * pan_orbit.sensitivity.zoom;

            pan_orbit.target_radius += zoom_delta;
            pan_orbit.target_radius = pan_orbit.target_radius.max(pan_orbit.limits.zoom_lower);
            if let Some(upper) = pan_orbit.limits.zoom_upper {
                pan_orbit.target_radius = pan_orbit.target_radius.min(upper);
            }
        }

        if wants_orbit {
            let delta_x = (position_delta.x / viewport_size.x) * std::f32::consts::PI * 2.0;
            let delta_y = (position_delta.y / viewport_size.y) * std::f32::consts::PI;

            let delta_x = if pan_orbit.is_upside_down {
                -delta_x
            } else {
                delta_x
            };

            pan_orbit.target_yaw -= delta_x * pan_orbit.sensitivity.orbit;
            pan_orbit.target_pitch += delta_y * pan_orbit.sensitivity.orbit;

            if pan_orbit.limits.allow_upside_down {
                pan_orbit.target_pitch %= std::f32::consts::PI * 2.0;
            } else {
                pan_orbit.target_pitch = pan_orbit
                    .target_pitch
                    .clamp(pan_orbit.limits.pitch_lower, pan_orbit.limits.pitch_upper);
            }
        }

        if wants_pan {
            let (_, rotation) = compute_pan_orbit_transform(
                pan_orbit.target_focus,
                pan_orbit.target_yaw,
                pan_orbit.target_pitch,
                pan_orbit.target_radius,
            );

            let right = nalgebra_glm::quat_rotate_vec3(&rotation, &Vec3::x());
            let up = nalgebra_glm::quat_rotate_vec3(&rotation, &Vec3::y());

            let pan_radius_basis = pan_orbit.pan_distance.unwrap_or(pan_orbit.target_radius);
            let world_per_pixel =
                (pan_radius_basis * 2.0 * (camera_y_fov * 0.5).tan()) / viewport_size.y;
            let pan_scale = world_per_pixel * pan_orbit.sensitivity.pan;
            let pan_right = right * -position_delta.x * pan_scale;
            let pan_up = up * position_delta.y * pan_scale;

            pan_orbit.target_focus += pan_right + pan_up;
        }

        if scrolled {
            let zoom_basis = if wheel_delta.y < 0.0 {
                pan_orbit
                    .target_radius
                    .max(pan_orbit.pan_distance.unwrap_or(pan_orbit.target_radius))
            } else {
                pan_orbit.target_radius
            };
            let zoom_delta = -wheel_delta.y * zoom_basis * 0.2 * pan_orbit.sensitivity.zoom;

            pan_orbit.target_radius += zoom_delta;
            pan_orbit.target_radius = pan_orbit.target_radius.max(pan_orbit.limits.zoom_lower);
            if let Some(upper) = pan_orbit.limits.zoom_upper {
                pan_orbit.target_radius = pan_orbit.target_radius.min(upper);
            }
        }

        if !orbit_button_pressed {
            pan_orbit.is_upside_down = pan_orbit.target_pitch.abs() > std::f32::consts::FRAC_PI_2;
        }
    });
}

pub fn pan_orbit_gamepad_input(
    active_camera: Res<nightshade_camera::resources::ActiveCamera>,
    actions: Res<Actions<EngineAction>>,
    time: Res<nightshade_platform::contract::time::Time>,
    mut cameras: Query<(&mut crate::contract::PanOrbitCamera,)>,
) {
    let Some(camera_entity) = active_camera.0 else {
        return;
    };

    let delta_time = time.delta_time;
    let target_pan = actions.axis2(EngineAction::Move);
    let target_orbit = actions.axis2(EngineAction::Look);
    let zoom_input = actions.axis(EngineAction::Lift);

    cameras.for_entity(camera_entity, |(pan_orbit,)| {
        if !pan_orbit.enabled {
            return;
        }

        let smoothing_factor = if pan_orbit.smoothness.gamepad > 0.0 {
            1.0 - pan_orbit.smoothness.gamepad.powi(7).powf(delta_time)
        } else {
            1.0
        };

        pan_orbit.smoothed_gamepad_orbit = pan_orbit.smoothed_gamepad_orbit
            * (1.0 - smoothing_factor)
            + target_orbit * smoothing_factor;
        pan_orbit.smoothed_gamepad_pan = pan_orbit.smoothed_gamepad_pan * (1.0 - smoothing_factor)
            + target_pan * smoothing_factor;

        let orbit_input = pan_orbit.smoothed_gamepad_orbit;
        let pan_input = pan_orbit.smoothed_gamepad_pan;

        if orbit_input.magnitude() > 0.001 {
            pan_orbit.target_yaw -=
                orbit_input.x * pan_orbit.sensitivity.gamepad_orbit * delta_time;
            pan_orbit.target_pitch +=
                orbit_input.y * pan_orbit.sensitivity.gamepad_orbit * delta_time;

            pan_orbit.target_pitch = pan_orbit
                .target_pitch
                .clamp(pan_orbit.limits.pitch_lower, pan_orbit.limits.pitch_upper);
        }

        if pan_input.magnitude() > 0.001 {
            let (_, rotation) = compute_pan_orbit_transform(
                pan_orbit.target_focus,
                pan_orbit.target_yaw,
                pan_orbit.target_pitch,
                pan_orbit.target_radius,
            );

            let right = nalgebra_glm::quat_rotate_vec3(&rotation, &Vec3::x());
            let forward_flat = {
                let forward = nalgebra_glm::quat_rotate_vec3(&rotation, &-Vec3::z());
                Vec3::new(forward.x, 0.0, forward.z).normalize()
            };

            let pan_scale =
                pan_orbit.target_radius * pan_orbit.sensitivity.gamepad_pan * delta_time * 0.1;
            pan_orbit.target_focus += right * pan_input.x * pan_scale;
            pan_orbit.target_focus += forward_flat * pan_input.y * pan_scale;
        }

        if zoom_input.abs() > 0.1 {
            let zoom_delta = zoom_input
                * pan_orbit.target_radius
                * pan_orbit.sensitivity.gamepad_zoom
                * delta_time
                * 0.1;

            pan_orbit.target_radius += zoom_delta;
            pan_orbit.target_radius = pan_orbit.target_radius.max(pan_orbit.limits.zoom_lower);
            if let Some(upper) = pan_orbit.limits.zoom_upper {
                pan_orbit.target_radius = pan_orbit.target_radius.min(upper);
            }
        }
    });
}

pub fn pan_orbit_touch_input(
    active_camera: Res<nightshade_camera::resources::ActiveCamera>,
    capture: Res<nightshade_platform::contract::input_capture::InputCapture>,
    input: Res<nightshade_platform::contract::input::Input>,
    window: Res<nightshade_platform::contract::window::Window>,
    mut cameras: Query<(&mut crate::contract::PanOrbitCamera, &Camera)>,
) {
    use nightshade_platform::contract::input::TouchGesture;

    if pointer_captured(&capture) {
        return;
    }
    let Some(camera_entity) = active_camera.0 else {
        return;
    };

    let gesture = input.touch.gesture;
    let viewport_size = pan_orbit_viewport_size(&window);

    cameras.for_entity(camera_entity, |(pan_orbit, camera)| {
        if !pan_orbit.enabled {
            return;
        }
        let camera_y_fov = pan_orbit_camera_y_fov(camera);

        match gesture {
            TouchGesture::SingleDrag { delta } => {
                let delta_x = delta.x / viewport_size.x * std::f32::consts::PI * 2.0;
                let delta_y = delta.y / viewport_size.y * std::f32::consts::PI;

                pan_orbit.target_yaw -= delta_x * pan_orbit.sensitivity.orbit;
                pan_orbit.target_pitch += delta_y * pan_orbit.sensitivity.orbit;

                pan_orbit.target_pitch = pan_orbit
                    .target_pitch
                    .clamp(pan_orbit.limits.pitch_lower, pan_orbit.limits.pitch_upper);
            }
            TouchGesture::TwoFingerDrag { delta } => {
                let (_, rotation) = compute_pan_orbit_transform(
                    pan_orbit.target_focus,
                    pan_orbit.target_yaw,
                    pan_orbit.target_pitch,
                    pan_orbit.target_radius,
                );

                let right = nalgebra_glm::quat_rotate_vec3(&rotation, &Vec3::x());
                let up = nalgebra_glm::quat_rotate_vec3(&rotation, &Vec3::y());

                let world_per_pixel =
                    (pan_orbit.target_radius * 2.0 * (camera_y_fov * 0.5).tan()) / viewport_size.y;
                let pan_scale = world_per_pixel * pan_orbit.sensitivity.pan;
                let pan_right = right * -delta.x * pan_scale;
                let pan_up = up * delta.y * pan_scale;

                pan_orbit.target_focus += pan_right + pan_up;
            }
            TouchGesture::Pinch { delta, .. } => {
                let zoom_delta =
                    -delta * pan_orbit.target_radius * 0.005 * pan_orbit.sensitivity.zoom;

                pan_orbit.target_radius += zoom_delta;
                pan_orbit.target_radius = pan_orbit.target_radius.max(pan_orbit.limits.zoom_lower);
                if let Some(upper) = pan_orbit.limits.zoom_upper {
                    pan_orbit.target_radius = pan_orbit.target_radius.min(upper);
                }
            }
            TouchGesture::None => {}
        }
    });
}

pub fn pan_orbit_update_transform(
    active_camera: Res<nightshade_camera::resources::ActiveCamera>,
    time: Res<nightshade_platform::contract::time::Time>,
    mut cameras: Query<(
        &mut crate::contract::PanOrbitCamera,
        &mut nightshade_scene::contract::LocalTransform,
    )>,
) {
    let Some(camera_entity) = active_camera.0 else {
        return;
    };
    let delta_time = time.delta_time;

    cameras.for_entity(camera_entity, |(pan_orbit, local_transform)| {
        if !pan_orbit.enabled {
            return;
        }

        let orbit_smoothness = pan_orbit.smoothness.orbit;
        let pan_smoothness = pan_orbit.smoothness.pan;
        let zoom_smoothness = pan_orbit.smoothness.zoom;

        pan_orbit.yaw = lerp_and_snap_f32(
            pan_orbit.yaw,
            pan_orbit.target_yaw,
            orbit_smoothness,
            delta_time,
        );
        pan_orbit.pitch = lerp_and_snap_f32(
            pan_orbit.pitch,
            pan_orbit.target_pitch,
            orbit_smoothness,
            delta_time,
        );
        pan_orbit.radius = lerp_and_snap_f32(
            pan_orbit.radius,
            pan_orbit.target_radius,
            zoom_smoothness,
            delta_time,
        );
        pan_orbit.focus = lerp_and_snap_vec3(
            pan_orbit.focus,
            pan_orbit.target_focus,
            pan_smoothness,
            delta_time,
        );

        let (position, rotation) = compute_pan_orbit_transform(
            pan_orbit.focus,
            pan_orbit.yaw,
            pan_orbit.pitch,
            pan_orbit.radius,
        );

        local_transform.rotation = rotation;
        local_transform.translation = position;
    });
}

const SNAP_EPSILON: f32 = 0.001;

fn lerp_and_snap_f32(from: f32, to: f32, smoothness: f32, delta_time: f32) -> f32 {
    let t = smoothness.powi(7);
    let result = from + (to - from) * (1.0 - t.powf(delta_time));
    if smoothness < 1.0 && (result - to).abs() < SNAP_EPSILON {
        to
    } else {
        result
    }
}

fn lerp_and_snap_vec3(from: Vec3, to: Vec3, smoothness: f32, delta_time: f32) -> Vec3 {
    let t = smoothness.powi(7);
    let result = from + (to - from) * (1.0 - t.powf(delta_time));
    if smoothness < 1.0 && (result - to).magnitude() < SNAP_EPSILON {
        to
    } else {
        result
    }
}

pub fn third_person_mouse_input(
    active_camera: Res<nightshade_camera::resources::ActiveCamera>,
    capture: Res<nightshade_platform::contract::input_capture::InputCapture>,
    input: Res<nightshade_platform::contract::input::Input>,
    window: Res<nightshade_platform::contract::window::Window>,
    mut cameras: Query<(&mut crate::contract::ThirdPersonCamera,)>,
) {
    if pointer_captured(&capture) {
        return;
    }
    let Some(camera_entity) = active_camera.0 else {
        return;
    };

    let mouse_state = input.mouse.state;
    let cursor_locked = window.cursor_locked;
    let wheel_delta = input.mouse.wheel_delta;

    let window_size = pan_orbit_viewport_size(&window);

    let scrolled = mouse_state.contains(MouseState::SCROLLED);
    let orbit_delta = if cursor_locked {
        Some(input.mouse.raw_mouse_delta)
    } else if mouse_state.contains(MouseState::RIGHT_CLICKED) {
        Some(input.mouse.position_delta)
    } else {
        None
    };

    cameras.for_entity(camera_entity, |(camera,)| {
        if let Some(orbit_delta) = orbit_delta {
            let delta_x = (orbit_delta.x / window_size.x) * std::f32::consts::PI * 2.0;
            let delta_y = (orbit_delta.y / window_size.y) * std::f32::consts::PI;

            camera.target_yaw -= delta_x * camera.orbit_sensitivity;
            camera.target_pitch = (camera.target_pitch + delta_y * camera.orbit_sensitivity)
                .clamp(camera.pitch_lower_limit, camera.pitch_upper_limit);
        }

        if scrolled {
            let zoom_delta =
                -wheel_delta.y * camera.target_distance * 0.2 * camera.zoom_sensitivity;
            camera.target_distance = (camera.target_distance + zoom_delta)
                .clamp(camera.distance_min, camera.distance_max);
        }
    });
}

const THIRD_PERSON_GAMEPAD_ORBIT_RATE: f32 = 2.5;

pub fn third_person_gamepad_input(
    active_camera: Res<nightshade_camera::resources::ActiveCamera>,
    actions: Res<Actions<EngineAction>>,
    time: Res<nightshade_platform::contract::time::Time>,
    mut cameras: Query<(&mut crate::contract::ThirdPersonCamera,)>,
) {
    let Some(camera_entity) = active_camera.0 else {
        return;
    };

    let look = actions.axis2(EngineAction::Look);
    if look == Vec2::zeros() {
        return;
    }
    let delta_time = time.delta_time;

    cameras.for_entity(camera_entity, |(camera,)| {
        let rate = camera.orbit_sensitivity * THIRD_PERSON_GAMEPAD_ORBIT_RATE * delta_time;
        camera.target_yaw -= look.x * rate;
        camera.target_pitch = (camera.target_pitch - look.y * rate)
            .clamp(camera.pitch_lower_limit, camera.pitch_upper_limit);
    });
}

pub fn third_person_update_transform(
    active_camera: Res<nightshade_camera::resources::ActiveCamera>,
    time: Res<nightshade_platform::contract::time::Time>,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(camera_entity) = active_camera.0 else {
        return;
    };

    if !storage.has::<crate::contract::ThirdPersonCamera>(camera_entity) {
        return;
    };

    let delta_time = time.delta_time;

    let (follow_target, height_offset) = {
        let Some(camera) = storage.get::<crate::contract::ThirdPersonCamera>(camera_entity) else {
            return;
        };
        (camera.follow_target, camera.height_offset)
    };

    let target_position = follow_target
        .and_then(|target| storage.get::<nightshade_scene::contract::LocalTransform>(target))
        .map(|transform| transform.translation + Vec3::new(0.0, height_offset, 0.0))
        .unwrap_or_else(|| Vec3::new(0.0, height_offset, 0.0));

    let (focus, yaw, pitch, distance, collision_enabled, collision_radius) = {
        let Some(camera) = storage.get_mut::<crate::contract::ThirdPersonCamera>(camera_entity)
        else {
            return;
        };

        let orbit_smoothness = camera.orbit_smoothness;
        let zoom_smoothness = camera.zoom_smoothness;
        let follow_smoothness = camera.follow_smoothness;

        camera.yaw = lerp_and_snap_f32(camera.yaw, camera.target_yaw, orbit_smoothness, delta_time);
        camera.pitch = lerp_and_snap_f32(
            camera.pitch,
            camera.target_pitch,
            orbit_smoothness,
            delta_time,
        );
        camera.distance = lerp_and_snap_f32(
            camera.distance,
            camera.target_distance,
            zoom_smoothness,
            delta_time,
        );
        camera.current_focus = lerp_and_snap_vec3(
            camera.current_focus,
            target_position,
            follow_smoothness,
            delta_time,
        );

        (
            camera.current_focus,
            camera.yaw,
            camera.pitch,
            camera.distance,
            camera.collision_enabled,
            camera.collision_radius,
        )
    };

    let distance = if collision_enabled {
        let (ideal_position, _) = compute_third_person_transform(focus, yaw, pitch, distance);
        let swept = storage
            .get::<SweptDistance>(camera_entity)
            .map(|swept| swept.distance)
            .unwrap_or(distance);
        storage.set(
            camera_entity,
            PendingCameraSweep {
                ask: CameraSweep {
                    focus,
                    ideal_position,
                    radius: collision_radius,
                    distance,
                },
            },
        );
        swept.min(distance)
    } else {
        distance
    };

    let (position, rotation) = compute_third_person_transform(focus, yaw, pitch, distance);

    let Some(local_transform) =
        storage.get_mut::<nightshade_scene::contract::LocalTransform>(camera_entity)
    else {
        return;
    };

    local_transform.translation = position;
    local_transform.rotation = rotation;
}

pub fn apply_camera_requests_system(
    mut commands: Commands,
    requests: Query<(&PendingCameraMove,)>,
    pan_orbit_cameras: Query<(&PendingCameraMove, &mut PanOrbitCamera)>,
    third_person_cameras: Query<
        (&PendingCameraMove, &mut ThirdPersonCamera),
        Without<PanOrbitCamera>,
    >,
) {
    requests.for_each(|camera, _| {
        commands.remove::<(PendingCameraMove,)>(camera);
    });

    pan_orbit_cameras.for_each(|_camera, (request, pan_orbit)| {
        match request.kind {
            CameraMove::Orbit { yaw, pitch } => {
                let signed_yaw = if pan_orbit.is_upside_down { -yaw } else { yaw };
                pan_orbit.target_yaw += signed_yaw * pan_orbit.sensitivity.orbit;
                pan_orbit.target_pitch += pitch * pan_orbit.sensitivity.orbit;
            }
            CameraMove::LookAt { yaw, pitch } => {
                if let Some(value) = yaw {
                    pan_orbit.target_yaw = shortest_angle_target(pan_orbit.target_yaw, value);
                }
                if let Some(value) = pitch {
                    pan_orbit.target_pitch = value;
                }
            }
        }
        if pan_orbit.limits.allow_upside_down {
            pan_orbit.target_pitch %= std::f32::consts::PI * 2.0;
        } else {
            pan_orbit.target_pitch = pan_orbit
                .target_pitch
                .clamp(pan_orbit.limits.pitch_lower, pan_orbit.limits.pitch_upper);
        }
    });

    third_person_cameras.for_each(|_camera, (request, third_person)| {
        match request.kind {
            CameraMove::Orbit { yaw, pitch } => {
                third_person.target_yaw += yaw * third_person.orbit_sensitivity;
                third_person.target_pitch += pitch * third_person.orbit_sensitivity;
            }
            CameraMove::LookAt { yaw, pitch } => {
                if let Some(value) = yaw {
                    third_person.target_yaw = shortest_angle_target(third_person.target_yaw, value);
                }
                if let Some(value) = pitch {
                    third_person.target_pitch = value;
                }
            }
        }
        third_person.target_pitch = third_person.target_pitch.clamp(
            third_person.pitch_lower_limit,
            third_person.pitch_upper_limit,
        );
    });
}

fn shortest_angle_target(current: f32, target: f32) -> f32 {
    let two_pi = std::f32::consts::PI * 2.0;
    let mut delta = (target - current) % two_pi;
    if delta > std::f32::consts::PI {
        delta -= two_pi;
    } else if delta < -std::f32::consts::PI {
        delta += two_pi;
    }
    current + delta
}
