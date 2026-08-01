#[cfg(target_os = "android")]
pub mod android;

#[cfg(not(target_os = "android"))]
use crate::contract::{GamepadAxis, GamepadButton};

#[cfg(not(target_os = "android"))]
pub(crate) const BUTTONS: [(gilrs::Button, GamepadButton); 17] = [
    (gilrs::Button::South, GamepadButton::South),
    (gilrs::Button::East, GamepadButton::East),
    (gilrs::Button::West, GamepadButton::West),
    (gilrs::Button::North, GamepadButton::North),
    (gilrs::Button::LeftTrigger, GamepadButton::LeftBumper),
    (gilrs::Button::RightTrigger, GamepadButton::RightBumper),
    (gilrs::Button::LeftTrigger2, GamepadButton::LeftTrigger),
    (gilrs::Button::RightTrigger2, GamepadButton::RightTrigger),
    (gilrs::Button::Select, GamepadButton::Select),
    (gilrs::Button::Start, GamepadButton::Start),
    (gilrs::Button::Mode, GamepadButton::Mode),
    (gilrs::Button::LeftThumb, GamepadButton::LeftThumb),
    (gilrs::Button::RightThumb, GamepadButton::RightThumb),
    (gilrs::Button::DPadUp, GamepadButton::DPadUp),
    (gilrs::Button::DPadDown, GamepadButton::DPadDown),
    (gilrs::Button::DPadLeft, GamepadButton::DPadLeft),
    (gilrs::Button::DPadRight, GamepadButton::DPadRight),
];

#[cfg(not(target_os = "android"))]
pub(crate) const AXES: [(gilrs::Axis, GamepadAxis); 6] = [
    (gilrs::Axis::LeftStickX, GamepadAxis::LeftStickX),
    (gilrs::Axis::LeftStickY, GamepadAxis::LeftStickY),
    (gilrs::Axis::RightStickX, GamepadAxis::RightStickX),
    (gilrs::Axis::RightStickY, GamepadAxis::RightStickY),
    (gilrs::Axis::LeftZ, GamepadAxis::LeftTrigger),
    (gilrs::Axis::RightZ, GamepadAxis::RightTrigger),
];

#[cfg(not(target_os = "android"))]
pub(crate) fn map_button(button: gilrs::Button) -> Option<GamepadButton> {
    BUTTONS
        .iter()
        .find(|(raw, _)| *raw == button)
        .map(|(_, mapped)| *mapped)
}
