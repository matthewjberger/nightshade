use crate::contract::{GamepadAxis, GamepadButton};
use nightshade_platform::android::android_activity::input::{Axis, Keycode};

const HAT_THRESHOLD: f32 = 0.5;

const BUTTONS: [(Keycode, GamepadButton); 17] = [
    (Keycode::ButtonA, GamepadButton::South),
    (Keycode::ButtonB, GamepadButton::East),
    (Keycode::ButtonX, GamepadButton::West),
    (Keycode::ButtonY, GamepadButton::North),
    (Keycode::ButtonL1, GamepadButton::LeftBumper),
    (Keycode::ButtonR1, GamepadButton::RightBumper),
    (Keycode::ButtonL2, GamepadButton::LeftTrigger),
    (Keycode::ButtonR2, GamepadButton::RightTrigger),
    (Keycode::ButtonSelect, GamepadButton::Select),
    (Keycode::ButtonStart, GamepadButton::Start),
    (Keycode::ButtonMode, GamepadButton::Mode),
    (Keycode::ButtonThumbl, GamepadButton::LeftThumb),
    (Keycode::ButtonThumbr, GamepadButton::RightThumb),
    (Keycode::DpadUp, GamepadButton::DPadUp),
    (Keycode::DpadDown, GamepadButton::DPadDown),
    (Keycode::DpadLeft, GamepadButton::DPadLeft),
    (Keycode::DpadRight, GamepadButton::DPadRight),
];

pub(crate) fn map_button(keycode: u32) -> Option<GamepadButton> {
    BUTTONS
        .iter()
        .find(|(code, _)| u32::from(*code) == keycode)
        .map(|(_, mapped)| *mapped)
}

fn read(values: &[(u32, f32)], axis: Axis) -> f32 {
    values
        .iter()
        .find(|(id, _)| *id == u32::from(axis))
        .map(|(_, value)| *value)
        .unwrap_or(0.0)
}

pub(crate) fn map_axes(values: &[(u32, f32)]) -> Vec<(GamepadAxis, f32)> {
    vec![
        (GamepadAxis::LeftStickX, read(values, Axis::X)),
        (GamepadAxis::LeftStickY, -read(values, Axis::Y)),
        (GamepadAxis::RightStickX, read(values, Axis::Z)),
        (GamepadAxis::RightStickY, -read(values, Axis::Rz)),
        (
            GamepadAxis::LeftTrigger,
            read(values, Axis::Ltrigger).max(read(values, Axis::Brake)),
        ),
        (
            GamepadAxis::RightTrigger,
            read(values, Axis::Rtrigger).max(read(values, Axis::Gas)),
        ),
    ]
}

pub(crate) fn map_hat(values: &[(u32, f32)]) -> Vec<GamepadButton> {
    let horizontal = read(values, Axis::HatX);
    let vertical = read(values, Axis::HatY);
    let mut held = Vec::new();
    if horizontal < -HAT_THRESHOLD {
        held.push(GamepadButton::DPadLeft);
    }
    if horizontal > HAT_THRESHOLD {
        held.push(GamepadButton::DPadRight);
    }
    if vertical < -HAT_THRESHOLD {
        held.push(GamepadButton::DPadUp);
    }
    if vertical > HAT_THRESHOLD {
        held.push(GamepadButton::DPadDown);
    }
    held
}
