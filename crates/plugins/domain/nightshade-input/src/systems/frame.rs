use crate::contract::{Action, Actions, Binding, Bindings, Pointer, Press, Stick};
use nalgebra_glm::Vec2;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_gamepad::contract::{Gamepad, GamepadAxis, deadzoned};
use nightshade_platform::contract::Input;
use nightshade_platform::contract::input::mouse::MouseState;
use nightshade_platform::contract::input_capture::InputCapture;

struct Sources<'a> {
    input: &'a Input,
    pad: Option<&'a Gamepad>,
    pointer_captured: bool,
    keyboard_captured: bool,
}

pub fn read_the_actions<A: Action>(
    input: Res<Input>,
    capture: Res<InputCapture>,
    pad_held: Option<Res<Gamepad>>,
    bindings: Res<Bindings<A>>,
    mut actions: ResMut<Actions<A>>,
) {
    let mut held = Vec::new();
    let mut axes = Vec::new();
    {
        let sources = Sources {
            input: &input,
            pad: pad_held.as_deref().filter(|pad| pad.connected),
            pointer_captured: !capture.pointer_claims.is_empty(),
            keyboard_captured: !capture.keyboard_claims.is_empty(),
        };
        let Some(table) = bindings.profiles.get(&bindings.worn) else {
            return;
        };
        for (action, bound) in table {
            let mut down = false;
            let mut reach = Vec2::new(0.0, 0.0);
            for binding in bound {
                match binding {
                    Binding::Key(key) => {
                        down |= press_value(&sources, Press::Key(*key)) > 0.0;
                    }
                    Binding::Keys {
                        up,
                        down: back,
                        left,
                        right,
                    } => {
                        if sources.keyboard_captured {
                            continue;
                        }
                        let keyboard = &sources.input.keyboard;
                        let vertical = f32::from(keyboard.is_key_pressed(*up))
                            - f32::from(keyboard.is_key_pressed(*back));
                        let horizontal = f32::from(keyboard.is_key_pressed(*right))
                            - f32::from(keyboard.is_key_pressed(*left));
                        reach += Vec2::new(horizontal, vertical);
                        down |= horizontal != 0.0 || vertical != 0.0;
                    }
                    Binding::Pointer(button) => {
                        down |= press_value(&sources, Press::Pointer(*button)) > 0.0;
                    }
                    Binding::Pad(button) => {
                        down |= press_value(&sources, Press::Pad(*button)) > 0.0;
                    }
                    Binding::Stick(stick) => {
                        let Some(pad) = sources.pad else {
                            continue;
                        };
                        let (horizontal, vertical) = match stick {
                            Stick::Left => (GamepadAxis::LeftStickX, GamepadAxis::LeftStickY),
                            Stick::Right => (GamepadAxis::RightStickX, GamepadAxis::RightStickY),
                        };
                        let moved = Vec2::new(
                            deadzoned(pad.axis(horizontal)),
                            deadzoned(pad.axis(vertical)),
                        );
                        reach += moved;
                        down |= moved.x != 0.0 || moved.y != 0.0;
                    }
                    Binding::Trigger(pulled) => {
                        let value = press_value(&sources, Press::Trigger(*pulled));
                        reach.x += value;
                        down |= value != 0.0;
                    }
                    Binding::Pair { positive, negative } => {
                        let value =
                            press_value(&sources, *positive) - press_value(&sources, *negative);
                        reach.x += value;
                        down |= value != 0.0;
                    }
                }
            }
            if reach.magnitude() > 1.0 {
                reach = reach.normalize();
            }
            if down {
                held.push(*action);
            }
            axes.push((*action, reach));
        }
    }
    actions.pressed.clear();
    actions.released.clear();
    for action in &held {
        if !actions.held.contains(action) {
            actions.pressed.insert(*action);
        }
    }
    let gone: Vec<A> = actions
        .held
        .iter()
        .filter(|action| !held.contains(action))
        .copied()
        .collect();
    for action in gone {
        actions.released.insert(action);
    }
    actions.held = held.into_iter().collect();
    actions.axes = axes.into_iter().collect();
}

fn press_value(sources: &Sources<'_>, press: Press) -> f32 {
    match press {
        Press::Key(key) => {
            f32::from(!sources.keyboard_captured && sources.input.keyboard.is_key_pressed(key))
        }
        Press::Pointer(button) => f32::from(
            !sources.pointer_captured && sources.input.mouse.state.contains(mouse_flag(button)),
        ),
        Press::Pad(button) => f32::from(sources.pad.is_some_and(|pad| pad.pressed(button))),
        Press::Trigger(pulled) => sources
            .pad
            .map(|pad| deadzoned(pad.axis(pulled)))
            .unwrap_or(0.0),
    }
}

fn mouse_flag(button: Pointer) -> MouseState {
    match button {
        Pointer::Left => MouseState::LEFT_CLICKED,
        Pointer::Middle => MouseState::MIDDLE_CLICKED,
        Pointer::Right => MouseState::RIGHT_CLICKED,
    }
}
