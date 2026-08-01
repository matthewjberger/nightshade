use crate::contract::{Gamepad, GamepadAxis, GamepadButton, STICK_DEADZONE};
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_platform::ElementState;
use nightshade_platform::contract::input::{Input, InputMode, MouseState};

const INPUT_MODE_TRIGGER: f32 = 0.3;

const MODE_BUTTONS: [GamepadButton; 10] = [
    GamepadButton::South,
    GamepadButton::East,
    GamepadButton::West,
    GamepadButton::North,
    GamepadButton::LeftBumper,
    GamepadButton::RightBumper,
    GamepadButton::LeftTrigger,
    GamepadButton::RightTrigger,
    GamepadButton::LeftThumb,
    GamepadButton::RightThumb,
];

pub fn detect_input_mode_system(mut input: ResMut<Input>, pad: Res<Gamepad>) {
    let has_keyboard_input = input
        .keyboard
        .keystates
        .values()
        .any(|state| *state == ElementState::Pressed);
    let mouse = &input.mouse;
    let has_mouse_input = mouse.raw_mouse_delta.x.abs() > 0.1
        || mouse.raw_mouse_delta.y.abs() > 0.1
        || mouse.state.contains(MouseState::LEFT_CLICKED)
        || mouse.state.contains(MouseState::RIGHT_CLICKED)
        || mouse.wheel_delta.y.abs() > 0.01;

    let has_gamepad_input = pad.connected
        && (pad.axis(GamepadAxis::LeftStickX).abs() > STICK_DEADZONE
            || pad.axis(GamepadAxis::LeftStickY).abs() > STICK_DEADZONE
            || pad.axis(GamepadAxis::RightStickX).abs() > STICK_DEADZONE
            || pad.axis(GamepadAxis::RightStickY).abs() > STICK_DEADZONE
            || pad.axis(GamepadAxis::LeftTrigger) > INPUT_MODE_TRIGGER
            || pad.axis(GamepadAxis::RightTrigger) > INPUT_MODE_TRIGGER
            || MODE_BUTTONS.iter().any(|button| pad.pressed(*button)));

    if has_gamepad_input && input.input_mode != InputMode::Gamepad {
        input.input_mode = InputMode::Gamepad;
    } else if (has_keyboard_input || has_mouse_input)
        && input.input_mode != InputMode::MouseKeyboard
    {
        input.input_mode = InputMode::MouseKeyboard;
    }
}
