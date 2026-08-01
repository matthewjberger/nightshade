use crate::contract::input::{AndroidGamepadEvents, Input};
use crate::contract::window::Window;
use nightshade_ecs::system_param::{Res, ResMut};

pub fn apply_ime_allowed_system(
    _main: nightshade_ecs::system_param::MainThread,
    mut input: ResMut<Input>,
    window: Res<Window>,
) {
    let ime = &input.keyboard.ime;
    let (allowed, applied) = (ime.allowed, ime.applied);
    if allowed == applied {
        return;
    }
    let Some(handle) = window.handle.as_ref() else {
        return;
    };
    handle.set_ime_allowed(allowed);
    input.keyboard.ime.applied = allowed;
}

pub fn set_ime_allowed(input: &mut Input, allowed: bool) {
    input.keyboard.ime.allowed = allowed;
}

pub fn reset_mouse_system(mut input: ResMut<Input>) {
    input.mouse.end_frame();
}

pub fn reset_touch_system(mut input: ResMut<Input>) {
    input.touch.end_frame();
}

pub fn reset_keyboard_system(mut input: ResMut<Input>) {
    input.keyboard.end_frame();
}

pub fn reset_input_events_system(mut input: ResMut<Input>) {
    input.events.clear();
}

pub fn reset_android_gamepad_events_system(mut events: ResMut<AndroidGamepadEvents>) {
    events.end_frame();
}

pub fn escape_key_exit_system(
    input: nightshade_ecs::system_param::Res<crate::contract::input::Input>,
    mut window: nightshade_ecs::system_param::ResMut<crate::contract::window::Window>,
) {
    if input
        .keyboard
        .is_key_pressed(winit::keyboard::KeyCode::Escape)
    {
        window.should_exit = true;
    }
}
