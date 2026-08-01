use crate::contract::input::{
    AndroidGamepadEvent, AndroidGamepadEvents, AndroidKeyboard, Input, TouchPhase,
};
use crate::contract::window::Window;
use android_activity::input::{
    Axis, InputEvent, KeyAction, KeyEvent, KeyMapChar, Keycode, MotionAction, MotionEvent, Source,
};
use android_activity::{AndroidApp, InputStatus};
use nalgebra_glm::Vec2;
use nightshade_ecs::ResourceMap;

const GAMEPAD_AXES: [Axis; 10] = [
    Axis::X,
    Axis::Y,
    Axis::Z,
    Axis::Rz,
    Axis::Ltrigger,
    Axis::Rtrigger,
    Axis::Brake,
    Axis::Gas,
    Axis::HatX,
    Axis::HatY,
];

pub fn drain_input(resources: &mut ResourceMap) {
    let Some(android_app) = crate::android::android_app() else {
        return;
    };
    let mut iterator = match android_app.input_events_iter() {
        Ok(iterator) => iterator,
        Err(error) => {
            tracing::warn!("android gave no input iterator: {error:?}");
            return;
        }
    };
    while iterator.next(|event| apply_event(resources, &android_app, event)) {}
}

fn apply_event(
    resources: &mut ResourceMap,
    android_app: &AndroidApp,
    event: &InputEvent<'_>,
) -> InputStatus {
    match event {
        InputEvent::MotionEvent(motion) => {
            if is_gamepad_source(motion.source()) {
                apply_gamepad_motion(resources, motion);
            } else {
                apply_touch_motion(resources, motion);
            }
            InputStatus::Handled
        }
        InputEvent::KeyEvent(key) => apply_key_event(resources, android_app, key),
        _ => InputStatus::Unhandled,
    }
}

fn is_gamepad_source(source: Source) -> bool {
    let bits = u32::from(source);
    let gamepad = u32::from(Source::Gamepad);
    let joystick = u32::from(Source::Joystick);
    bits & gamepad == gamepad || bits & joystick == joystick
}

fn is_gamepad_button(keycode: Keycode) -> bool {
    matches!(
        keycode,
        Keycode::ButtonA
            | Keycode::ButtonB
            | Keycode::ButtonC
            | Keycode::ButtonX
            | Keycode::ButtonY
            | Keycode::ButtonZ
            | Keycode::ButtonL1
            | Keycode::ButtonR1
            | Keycode::ButtonL2
            | Keycode::ButtonR2
            | Keycode::ButtonThumbl
            | Keycode::ButtonThumbr
            | Keycode::ButtonStart
            | Keycode::ButtonSelect
            | Keycode::ButtonMode
    )
}

fn apply_gamepad_motion(resources: &mut ResourceMap, motion: &MotionEvent<'_>) {
    let pointer = motion.pointer_at_index(motion.pointer_index());
    let values = GAMEPAD_AXES
        .iter()
        .map(|axis| (u32::from(*axis), pointer.axis_value(*axis)))
        .collect();
    resources
        .res_mut::<AndroidGamepadEvents>()
        .events
        .push(AndroidGamepadEvent::Axes {
            device: motion.device_id(),
            values,
        });
}

fn apply_touch_motion(resources: &mut ResourceMap, motion: &MotionEvent<'_>) {
    let phase = match motion.action() {
        MotionAction::Down | MotionAction::PointerDown => TouchPhase::Started,
        MotionAction::Up | MotionAction::PointerUp => TouchPhase::Ended,
        MotionAction::Move => TouchPhase::Moved,
        MotionAction::Cancel => TouchPhase::Cancelled,
        _ => return,
    };

    let scale = resources.res::<Window>().pointer_viewport_scale;
    let indices: Vec<usize> = match phase {
        TouchPhase::Started | TouchPhase::Ended => vec![motion.pointer_index()],
        TouchPhase::Moved | TouchPhase::Cancelled => (0..motion.pointer_count()).collect(),
    };

    for index in indices {
        let pointer = motion.pointer_at_index(index);
        let position = Vec2::new(pointer.x(), pointer.y()) * scale;
        let id = pointer.pointer_id() as u64;
        crate::commands::input::apply_touch(resources.res_mut::<Input>(), id, position, phase);
    }
}

fn apply_key_event(
    resources: &mut ResourceMap,
    android_app: &AndroidApp,
    key: &KeyEvent<'_>,
) -> InputStatus {
    if matches!(
        key.key_code(),
        Keycode::VolumeUp | Keycode::VolumeDown | Keycode::VolumeMute
    ) {
        return InputStatus::Unhandled;
    }

    if is_gamepad_source(key.source()) || is_gamepad_button(key.key_code()) {
        let pressed = key.action() == KeyAction::Down;
        resources
            .res_mut::<AndroidGamepadEvents>()
            .events
            .push(AndroidGamepadEvent::Button {
                device: key.device_id(),
                keycode: u32::from(key.key_code()),
                pressed,
            });
        return InputStatus::Handled;
    }

    let Some(key_code) = to_key_code(key.key_code()) else {
        return InputStatus::Handled;
    };
    let state = match key.action() {
        KeyAction::Down => winit::event::ElementState::Pressed,
        _ => winit::event::ElementState::Released,
    };
    let text = combined_character(resources, android_app, key).map(String::from);
    crate::commands::input::apply_key(
        resources.res_mut::<Input>(),
        key_code,
        state,
        key.repeat_count() > 0,
        text.as_deref(),
    );
    InputStatus::Handled
}

fn combined_character(
    resources: &mut ResourceMap,
    android_app: &AndroidApp,
    key: &KeyEvent<'_>,
) -> Option<char> {
    let device_id = key.device_id();
    let key_map = match android_app.device_key_character_map(device_id) {
        Ok(key_map) => key_map,
        Err(error) => {
            tracing::warn!("android device {device_id} has no key character map: {error:?}");
            return None;
        }
    };

    let down = key.action() == KeyAction::Down;
    match key_map.get(key.key_code(), key.meta_state()) {
        Ok(KeyMapChar::Unicode(unicode)) => {
            if !down {
                return Some(unicode);
            }
            let Some(accent) = resources
                .res_mut::<AndroidKeyboard>()
                .combining_accent
                .take()
            else {
                return Some(unicode);
            };
            match key_map.get_dead_char(accent, unicode) {
                Ok(combined) => combined,
                Err(error) => {
                    tracing::warn!("android could not combine two characters: {error:?}");
                    None
                }
            }
        }
        Ok(KeyMapChar::CombiningAccent(accent)) => {
            if down {
                resources.res_mut::<AndroidKeyboard>().combining_accent = Some(accent);
            }
            None
        }
        Ok(KeyMapChar::None) => None,
        Err(error) => {
            tracing::warn!("android gave no character for a key: {error:?}");
            resources.res_mut::<AndroidKeyboard>().combining_accent = None;
            None
        }
    }
}

fn to_key_code(keycode: Keycode) -> Option<winit::keyboard::KeyCode> {
    use winit::keyboard::KeyCode;
    Some(match keycode {
        Keycode::A => KeyCode::KeyA,
        Keycode::B => KeyCode::KeyB,
        Keycode::C => KeyCode::KeyC,
        Keycode::D => KeyCode::KeyD,
        Keycode::E => KeyCode::KeyE,
        Keycode::F => KeyCode::KeyF,
        Keycode::G => KeyCode::KeyG,
        Keycode::H => KeyCode::KeyH,
        Keycode::I => KeyCode::KeyI,
        Keycode::J => KeyCode::KeyJ,
        Keycode::K => KeyCode::KeyK,
        Keycode::L => KeyCode::KeyL,
        Keycode::M => KeyCode::KeyM,
        Keycode::N => KeyCode::KeyN,
        Keycode::O => KeyCode::KeyO,
        Keycode::P => KeyCode::KeyP,
        Keycode::Q => KeyCode::KeyQ,
        Keycode::R => KeyCode::KeyR,
        Keycode::S => KeyCode::KeyS,
        Keycode::T => KeyCode::KeyT,
        Keycode::U => KeyCode::KeyU,
        Keycode::V => KeyCode::KeyV,
        Keycode::W => KeyCode::KeyW,
        Keycode::X => KeyCode::KeyX,
        Keycode::Y => KeyCode::KeyY,
        Keycode::Z => KeyCode::KeyZ,
        Keycode::Keycode0 => KeyCode::Digit0,
        Keycode::Keycode1 => KeyCode::Digit1,
        Keycode::Keycode2 => KeyCode::Digit2,
        Keycode::Keycode3 => KeyCode::Digit3,
        Keycode::Keycode4 => KeyCode::Digit4,
        Keycode::Keycode5 => KeyCode::Digit5,
        Keycode::Keycode6 => KeyCode::Digit6,
        Keycode::Keycode7 => KeyCode::Digit7,
        Keycode::Keycode8 => KeyCode::Digit8,
        Keycode::Keycode9 => KeyCode::Digit9,
        Keycode::Numpad0 => KeyCode::Numpad0,
        Keycode::Numpad1 => KeyCode::Numpad1,
        Keycode::Numpad2 => KeyCode::Numpad2,
        Keycode::Numpad3 => KeyCode::Numpad3,
        Keycode::Numpad4 => KeyCode::Numpad4,
        Keycode::Numpad5 => KeyCode::Numpad5,
        Keycode::Numpad6 => KeyCode::Numpad6,
        Keycode::Numpad7 => KeyCode::Numpad7,
        Keycode::Numpad8 => KeyCode::Numpad8,
        Keycode::Numpad9 => KeyCode::Numpad9,
        Keycode::NumpadAdd => KeyCode::NumpadAdd,
        Keycode::NumpadSubtract => KeyCode::NumpadSubtract,
        Keycode::NumpadMultiply => KeyCode::NumpadMultiply,
        Keycode::NumpadDivide => KeyCode::NumpadDivide,
        Keycode::NumpadEnter => KeyCode::NumpadEnter,
        Keycode::NumpadEquals => KeyCode::NumpadEqual,
        Keycode::NumpadComma => KeyCode::NumpadComma,
        Keycode::NumpadDot => KeyCode::NumpadDecimal,
        Keycode::NumLock => KeyCode::NumLock,
        Keycode::DpadLeft => KeyCode::ArrowLeft,
        Keycode::DpadRight => KeyCode::ArrowRight,
        Keycode::DpadUp => KeyCode::ArrowUp,
        Keycode::DpadDown => KeyCode::ArrowDown,
        Keycode::F1 => KeyCode::F1,
        Keycode::F2 => KeyCode::F2,
        Keycode::F3 => KeyCode::F3,
        Keycode::F4 => KeyCode::F4,
        Keycode::F5 => KeyCode::F5,
        Keycode::F6 => KeyCode::F6,
        Keycode::F7 => KeyCode::F7,
        Keycode::F8 => KeyCode::F8,
        Keycode::F9 => KeyCode::F9,
        Keycode::F10 => KeyCode::F10,
        Keycode::F11 => KeyCode::F11,
        Keycode::F12 => KeyCode::F12,
        Keycode::Space => KeyCode::Space,
        Keycode::Escape => KeyCode::Escape,
        Keycode::Enter => KeyCode::Enter,
        Keycode::Tab => KeyCode::Tab,
        Keycode::PageUp => KeyCode::PageUp,
        Keycode::PageDown => KeyCode::PageDown,
        Keycode::MoveHome => KeyCode::Home,
        Keycode::MoveEnd => KeyCode::End,
        Keycode::Insert => KeyCode::Insert,
        Keycode::Del => KeyCode::Backspace,
        Keycode::ForwardDel => KeyCode::Delete,
        Keycode::Copy => KeyCode::Copy,
        Keycode::Paste => KeyCode::Paste,
        Keycode::Cut => KeyCode::Cut,
        Keycode::VolumeUp => KeyCode::AudioVolumeUp,
        Keycode::VolumeDown => KeyCode::AudioVolumeDown,
        Keycode::VolumeMute => KeyCode::AudioVolumeMute,
        Keycode::MediaPlayPause => KeyCode::MediaPlayPause,
        Keycode::MediaStop => KeyCode::MediaStop,
        Keycode::MediaNext => KeyCode::MediaTrackNext,
        Keycode::MediaPrevious => KeyCode::MediaTrackPrevious,
        Keycode::Plus => KeyCode::Equal,
        Keycode::Minus => KeyCode::Minus,
        Keycode::Equals => KeyCode::Equal,
        Keycode::Semicolon => KeyCode::Semicolon,
        Keycode::Slash => KeyCode::Slash,
        Keycode::Backslash => KeyCode::Backslash,
        Keycode::Comma => KeyCode::Comma,
        Keycode::Period => KeyCode::Period,
        Keycode::Apostrophe => KeyCode::Quote,
        Keycode::Grave => KeyCode::Backquote,
        Keycode::Sysrq => KeyCode::PrintScreen,
        Keycode::Break => KeyCode::Pause,
        Keycode::ScrollLock => KeyCode::ScrollLock,
        Keycode::Yen => KeyCode::IntlYen,
        Keycode::Kana => KeyCode::Lang1,
        Keycode::KatakanaHiragana => KeyCode::KanaMode,
        Keycode::CtrlLeft => KeyCode::ControlLeft,
        Keycode::CtrlRight => KeyCode::ControlRight,
        Keycode::ShiftLeft => KeyCode::ShiftLeft,
        Keycode::ShiftRight => KeyCode::ShiftRight,
        Keycode::AltLeft => KeyCode::AltLeft,
        Keycode::AltRight => KeyCode::AltRight,
        Keycode::MetaLeft => KeyCode::SuperLeft,
        Keycode::MetaRight => KeyCode::SuperRight,
        Keycode::LeftBracket => KeyCode::BracketLeft,
        Keycode::RightBracket => KeyCode::BracketRight,
        Keycode::Power => KeyCode::Power,
        Keycode::Sleep => KeyCode::Sleep,
        Keycode::Wakeup => KeyCode::WakeUp,
        _ => return None,
    })
}
