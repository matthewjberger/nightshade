#[cfg(feature = "gamepad")]
pub use crate::ecs::input::queries::*;

#[cfg(feature = "gamepad")]
use crate::ecs::event_bus::commands::publish_event;
#[cfg(feature = "gamepad")]
use crate::ecs::world::events::*;
use crate::ecs::world::{Vec2, World, resources::MouseState};

pub fn reset_mouse_system(world: &mut World) {
    let _span = tracing::info_span!("reset_mouse").entered();
    let mouse = &mut world.resources.input.mouse;
    if mouse.state.contains(MouseState::SCROLLED) {
        mouse.wheel_delta = Vec2::new(0.0, 0.0);
    }
    mouse.state.set(MouseState::SCROLLED, false);
    if mouse.state.contains(MouseState::MOVED) {
        mouse.position_delta = Vec2::new(0.0, 0.0);
    }
    mouse.state.set(MouseState::MOVED, false);
    mouse.raw_mouse_delta = Vec2::new(0.0, 0.0);
    mouse.state.set(MouseState::LEFT_JUST_PRESSED, false);
    mouse.state.set(MouseState::MIDDLE_JUST_PRESSED, false);
    mouse.state.set(MouseState::RIGHT_JUST_PRESSED, false);
    mouse.state.set(MouseState::LEFT_JUST_RELEASED, false);
    mouse.state.set(MouseState::MIDDLE_JUST_RELEASED, false);
    mouse.state.set(MouseState::RIGHT_JUST_RELEASED, false);

    #[cfg(feature = "multi_window")]
    for state in &mut world.resources.secondary_windows.states {
        if state.mouse.state.contains(MouseState::SCROLLED) {
            state.mouse.wheel_delta = Vec2::new(0.0, 0.0);
        }
        state.mouse.state.set(MouseState::SCROLLED, false);
        if state.mouse.state.contains(MouseState::MOVED) {
            state.mouse.position_delta = Vec2::new(0.0, 0.0);
        }
        state.mouse.state.set(MouseState::MOVED, false);
        state.mouse.raw_mouse_delta = Vec2::new(0.0, 0.0);
        state.mouse.state.set(MouseState::LEFT_JUST_PRESSED, false);
        state
            .mouse
            .state
            .set(MouseState::MIDDLE_JUST_PRESSED, false);
        state.mouse.state.set(MouseState::RIGHT_JUST_PRESSED, false);
        state.mouse.state.set(MouseState::LEFT_JUST_RELEASED, false);
        state
            .mouse
            .state
            .set(MouseState::MIDDLE_JUST_RELEASED, false);
        state
            .mouse
            .state
            .set(MouseState::RIGHT_JUST_RELEASED, false);
    }
}

pub fn reset_touch_system(world: &mut World) {
    let _span = tracing::info_span!("reset_touch").entered();
    use crate::ecs::input::resources::TouchGesture;

    let touch = &mut world.resources.input.touch;
    touch.clear_ended_touches();
    touch.gesture = TouchGesture::None;
}

#[cfg(feature = "gamepad")]
pub fn gamepad_input_system(world: &mut World) {
    let _span = tracing::info_span!("gamepad_input").entered();
    if world.resources.input.gamepad.gilrs.is_none() {
        if let Ok(gilrs) = gilrs::Gilrs::new() {
            world.resources.input.gamepad.gilrs = Some(gilrs);
        } else {
            return;
        }
    }

    world.resources.input.gamepad.events.clear();
    world.resources.input.gamepad.just_pressed_buttons.clear();

    let Some(gilrs) = &mut world.resources.input.gamepad.gilrs else {
        return;
    };

    let mut connection_events = Vec::new();

    while let Some(event) = gilrs.next_event() {
        if matches!(event.event, gilrs::EventType::Connected) {
            connection_events.push((event.id.into(), true));
        } else if matches!(event.event, gilrs::EventType::Disconnected) {
            connection_events.push((event.id.into(), false));
        }
        if let gilrs::EventType::ButtonPressed(button, _) = event.event {
            world
                .resources
                .input
                .gamepad
                .just_pressed_buttons
                .insert(button);
        }

        world.resources.input.gamepad.gamepad = Some(event.id);
        world.resources.input.gamepad.events.push(event);
    }

    for (gamepad_id, connected) in connection_events {
        if connected {
            publish_event(
                world,
                Message::Input {
                    event: InputEvent::GamepadConnected { gamepad_id },
                },
            );
        } else {
            publish_event(
                world,
                Message::Input {
                    event: InputEvent::GamepadDisconnected { gamepad_id },
                },
            );
        }
    }
}

pub fn reset_keyboard_system(world: &mut World) {
    let keyboard = &mut world.resources.input.keyboard;
    keyboard.just_pressed_keys.clear();
    keyboard.just_released_keys.clear();
    keyboard.frame_chars.clear();
    keyboard.frame_keys.clear();

    #[cfg(feature = "multi_window")]
    for state in &mut world.resources.secondary_windows.states {
        state.keyboard.just_pressed_keys.clear();
        state.keyboard.just_released_keys.clear();
        state.keyboard.frame_chars.clear();
        state.keyboard.frame_keys.clear();
    }
}

pub fn escape_key_exit_system(world: &mut World) {
    if crate::ecs::input::access::keyboard_for_active(world)
        .is_key_pressed(winit::keyboard::KeyCode::Escape)
    {
        world.resources.window.should_exit = true;
    }
}

#[cfg(feature = "gamepad")]
pub fn detect_input_mode_system(world: &mut World) {
    use crate::ecs::input::resources::{InputMode, MouseState};

    let keyboard = crate::ecs::input::access::keyboard_for_active(world);
    let mouse = crate::ecs::input::access::mouse_for_active(world);

    let has_keyboard_input = keyboard
        .keystates
        .values()
        .any(|state| *state == winit::event::ElementState::Pressed);
    let has_mouse_input = mouse.raw_mouse_delta.x.abs() > 0.1
        || mouse.raw_mouse_delta.y.abs() > 0.1
        || mouse.state.contains(MouseState::LEFT_CLICKED)
        || mouse.state.contains(MouseState::RIGHT_CLICKED)
        || mouse.wheel_delta.y.abs() > 0.01;

    let has_gamepad_input = if let Some(gamepad) = query_active_gamepad(world) {
        let left_stick_x = gamepad.value(gilrs::Axis::LeftStickX);
        let left_stick_y = gamepad.value(gilrs::Axis::LeftStickY);
        let right_stick_x = gamepad.value(gilrs::Axis::RightStickX);
        let right_stick_y = gamepad.value(gilrs::Axis::RightStickY);
        let rt_value = gamepad.value(gilrs::Axis::RightZ);
        let lt_value = gamepad.value(gilrs::Axis::LeftZ);

        let deadzone = 0.15;
        left_stick_x.abs() > deadzone
            || left_stick_y.abs() > deadzone
            || right_stick_x.abs() > deadzone
            || right_stick_y.abs() > deadzone
            || rt_value > 0.3
            || lt_value > 0.3
            || gamepad.is_pressed(gilrs::Button::South)
            || gamepad.is_pressed(gilrs::Button::East)
            || gamepad.is_pressed(gilrs::Button::West)
            || gamepad.is_pressed(gilrs::Button::North)
            || gamepad.is_pressed(gilrs::Button::LeftTrigger)
            || gamepad.is_pressed(gilrs::Button::RightTrigger)
            || gamepad.is_pressed(gilrs::Button::LeftTrigger2)
            || gamepad.is_pressed(gilrs::Button::RightTrigger2)
            || gamepad.is_pressed(gilrs::Button::LeftThumb)
            || gamepad.is_pressed(gilrs::Button::RightThumb)
    } else {
        false
    };

    if has_gamepad_input && world.resources.input.input_mode != InputMode::Gamepad {
        world.resources.input.input_mode = InputMode::Gamepad;
    } else if (has_keyboard_input || has_mouse_input)
        && world.resources.input.input_mode != InputMode::MouseKeyboard
    {
        world.resources.input.input_mode = InputMode::MouseKeyboard;
    }
}
