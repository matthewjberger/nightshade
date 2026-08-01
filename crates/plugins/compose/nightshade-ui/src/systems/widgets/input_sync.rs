use nalgebra_glm;
use nalgebra_glm::Vec2;
use nightshade_platform::KeyCode;

use crate::prelude::*;
pub fn ui_retained_input_sync_system(
    runtime: nightshade::ecs::system_param::Res<crate::resources::RetainedUiRuntime>,
    input: nightshade::ecs::system_param::Res<nightshade_platform::contract::input::Input>,
    time: nightshade::ecs::system_param::Res<nightshade_platform::contract::time::Time>,
    mut frame: nightshade::ecs::system_param::ResMut<crate::resources::RetainedUiFrame>,
    mut accessibility: nightshade::ecs::system_param::ResMut<
        crate::resources::RetainedUiAccessibility,
    >,
    mut timing: nightshade::ecs::system_param::ResMut<crate::resources::RetainedUiTiming>,
    mut mirror: nightshade::ecs::system_param::ResMut<crate::resources::RetainedUiInputMirror>,
    mut dirty: nightshade::ecs::system_param::ResMut<crate::resources::RetainedUiDirty>,
) {
    if !runtime.visible {
        return;
    }

    frame.events.clear();
    accessibility.announce_queue.clear();
    timing.delta_time = time.delta_time;
    timing.current_time = time.uptime_milliseconds as f64 / 1000.0;

    let keyboard = &input.keyboard;
    let ctrl_held = keyboard.is_key_pressed(KeyCode::ControlLeft)
        || keyboard.is_key_pressed(KeyCode::ControlRight);
    let shift_held =
        keyboard.is_key_pressed(KeyCode::ShiftLeft) || keyboard.is_key_pressed(KeyCode::ShiftRight);

    mirror.frame_chars = filter_chars(keyboard.frame_chars.clone(), ctrl_held);
    mirror.frame_keys = keyboard.frame_keys.clone();
    mirror.scroll_delta = input.mouse.wheel_delta;
    mirror.ctrl_held = ctrl_held;
    mirror.shift_held = shift_held;

    let mouse_state = input.mouse.state;
    let mouse_drives_layout = mouse_state.intersects(
        nightshade_platform::contract::input::MouseState::LEFT_CLICKED
            | nightshade_platform::contract::input::MouseState::LEFT_JUST_PRESSED
            | nightshade_platform::contract::input::MouseState::LEFT_JUST_RELEASED
            | nightshade_platform::contract::input::MouseState::RIGHT_JUST_PRESSED
            | nightshade_platform::contract::input::MouseState::RIGHT_JUST_RELEASED,
    );
    if mouse_drives_layout
        || mirror.scroll_delta != Vec2::zeros()
        || !mirror.frame_chars.is_empty()
        || !mirror.frame_keys.is_empty()
    {
        ui_mark_layout_dirty(&mut dirty);
    }
}

fn filter_chars(chars: Vec<char>, ctrl_held: bool) -> Vec<char> {
    if !ctrl_held {
        return chars;
    }
    chars
        .into_iter()
        .filter(|character| !character.is_ascii())
        .collect()
}
