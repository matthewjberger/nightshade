use crate::contract::input::{Input, MouseState, TouchPhase, TouchPoint};
use nalgebra_glm::Vec2;

pub fn apply_key(
    input: &mut Input,
    key_code: winit::keyboard::KeyCode,
    state: winit::event::ElementState,
    repeat: bool,
    text: Option<&str>,
) {
    let was_pressed = input
        .keyboard
        .keystates
        .get(&key_code)
        .is_some_and(|previous| *previous == winit::event::ElementState::Pressed);

    *input.keyboard.keystates.entry(key_code).or_insert(state) = state;

    let pressed = state == winit::event::ElementState::Pressed;
    if !repeat {
        input.keyboard.frame_keys.push((key_code, pressed));
    }

    if pressed && !was_pressed {
        input.keyboard.just_pressed_keys.insert(key_code);
    } else if !pressed && was_pressed {
        input.keyboard.just_released_keys.insert(key_code);
    }

    if pressed && let Some(text) = text {
        for character in text.chars() {
            input.keyboard.frame_chars.push(character);
        }
    }
}

pub fn apply_touch(input: &mut Input, id: u64, position: Vec2, phase: TouchPhase) {
    let touch_input = &mut input.touch;
    let was_primary = touch_input.primary_touch_id == Some(id);

    match phase {
        TouchPhase::Started => {
            let touch_point = TouchPoint {
                id,
                position,
                start_position: position,
                previous_position: position,
                phase: TouchPhase::Started,
            };
            touch_input.touches.insert(id, touch_point);

            if touch_input.primary_touch_id.is_none() {
                touch_input.primary_touch_id = Some(id);
            } else if touch_input.secondary_touch_id.is_none() {
                touch_input.secondary_touch_id = Some(id);
            }
        }
        TouchPhase::Moved => {
            if let Some(touch_point) = touch_input.touches.get_mut(&id) {
                touch_point.previous_position = touch_point.position;
                touch_point.position = position;
                touch_point.phase = TouchPhase::Moved;
            }
        }
        TouchPhase::Ended => {
            if let Some(touch_point) = touch_input.touches.get_mut(&id) {
                touch_point.previous_position = touch_point.position;
                touch_point.position = position;
                touch_point.phase = TouchPhase::Ended;
            }
        }
        TouchPhase::Cancelled => {
            if let Some(touch_point) = touch_input.touches.get_mut(&id) {
                touch_point.phase = TouchPhase::Cancelled;
            }
        }
    }

    touch_input.update_gesture();

    let active_touches = touch_input.active_touch_count();
    let is_primary = touch_input.primary_touch_id == Some(id);
    let mouse = &mut input.mouse;

    let lifted = matches!(phase, TouchPhase::Ended | TouchPhase::Cancelled);
    if active_touches > 1 || (lifted && was_primary) {
        if lifted && was_primary {
            mouse.position = position;
        }
        let was_clicked = mouse.state.contains(MouseState::LEFT_CLICKED);
        mouse.state.set(MouseState::LEFT_CLICKED, false);
        if was_clicked {
            mouse.state.set(MouseState::LEFT_JUST_RELEASED, true);
        }
        return;
    }

    if !is_primary || active_touches != 1 {
        return;
    }

    match phase {
        TouchPhase::Started => {
            if mouse.position_initialized {
                mouse.position_delta += position - mouse.position;
            } else {
                mouse.position_initialized = true;
            }
            mouse.position = position;
            mouse.state.set(MouseState::MOVED, true);
            let was_clicked = mouse.state.contains(MouseState::LEFT_CLICKED);
            mouse.state.set(MouseState::LEFT_CLICKED, true);
            if !was_clicked {
                mouse.state.set(MouseState::LEFT_JUST_PRESSED, true);
            }
        }
        TouchPhase::Moved => {
            if mouse.position_initialized {
                mouse.position_delta += position - mouse.position;
            }
            mouse.position = position;
            mouse.state.set(MouseState::MOVED, true);
        }
        _ => {}
    }
}
