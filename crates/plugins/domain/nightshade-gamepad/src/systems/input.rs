use crate::contract::Gamepad;
use crate::resources::GamepadBackend;
use nightshade::prelude::*;

#[cfg(not(target_os = "android"))]
pub fn gamepad_input_system(mut backend: ResMut<GamepadBackend>, mut pad_state: ResMut<Gamepad>) {
    crate::commands::poll_gamepad(&mut backend, &mut pad_state);
}

#[cfg(target_os = "android")]
pub fn gamepad_input_system(
    mut backend: ResMut<GamepadBackend>,
    mut pad_state: ResMut<Gamepad>,
    events: Res<nightshade_platform::contract::input::AndroidGamepadEvents>,
) {
    crate::commands::poll_gamepad(&mut backend, &mut pad_state, &events);
}
