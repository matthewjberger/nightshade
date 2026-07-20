//! Gamepad input over gilrs: the pad state resource, the per-frame poll,
//! and the input-mode detector. [`GamepadPlugin`] installs the poll at the
//! top of every frame; without it the engine never opens the gilrs context.

use crate::app::{App, Plugin, Stage};
use crate::ecs::event_bus::commands::publish_event;
use crate::ecs::world::World;
use crate::ecs::world::events::*;

pub use nightshade_platform::input::gamepad::Gamepad;

pub fn with_active_gamepad<R>(world: &World, f: impl FnOnce(gilrs::Gamepad<'_>) -> R) -> Option<R> {
    let gamepad = world.res::<Gamepad>();
    let guard = gamepad.gilrs.lock().unwrap();
    let gilrs = guard.as_ref()?;
    let id = gamepad.gamepad?;
    Some(f(gilrs.gamepad(id)))
}

pub fn gamepad_input_system(world: &mut World) {
    let _span = tracing::info_span!("gamepad_input").entered();
    let mut connection_events = Vec::new();
    {
        let gamepad = world.res_mut::<Gamepad>();
        gamepad.events.clear();
        gamepad.just_pressed_buttons.clear();

        let mut guard = gamepad.gilrs.lock().unwrap();
        if guard.is_none() {
            match gilrs::Gilrs::new() {
                Ok(new_gilrs) => *guard = Some(new_gilrs),
                Err(_) => return,
            }
        }
        let gilrs = guard.as_mut().unwrap();

        while let Some(event) = gilrs.next_event() {
            if matches!(event.event, gilrs::EventType::Connected) {
                connection_events.push((event.id.into(), true));
            } else if matches!(event.event, gilrs::EventType::Disconnected) {
                connection_events.push((event.id.into(), false));
            }
            if let gilrs::EventType::ButtonPressed(button, _) = event.event {
                gamepad.just_pressed_buttons.insert(button);
            }

            gamepad.gamepad = Some(event.id);
            gamepad.events.push(event);
        }
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

    let has_gamepad_input = with_active_gamepad(world, |gamepad| {
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
    })
    .unwrap_or(false);

    if has_gamepad_input
        && world
            .res::<crate::ecs::input::resources::Input>()
            .input_mode
            != InputMode::Gamepad
    {
        world
            .res_mut::<crate::ecs::input::resources::Input>()
            .input_mode = InputMode::Gamepad;
    } else if (has_keyboard_input || has_mouse_input)
        && world
            .res::<crate::ecs::input::resources::Input>()
            .input_mode
            != InputMode::MouseKeyboard
    {
        world
            .res_mut::<crate::ecs::input::resources::Input>()
            .input_mode = InputMode::MouseKeyboard;
    }
}

/// Installs gamepad input: polls devices at the top of every frame and
/// forwards connection and button events onto `world.res::<crate::ecs::input::resources::Input>().events`.
/// Without this plugin the engine never opens the gilrs context.
pub struct GamepadPlugin;

impl Plugin for GamepadPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(Stage::First, |world: &mut World| {
            gamepad_input_system(world);
            let gamepad_events = world.res::<Gamepad>().events.clone();
            for event in gamepad_events {
                crate::ecs::events::emit_app_event(
                    world,
                    crate::ecs::input::events::AppEvent::Gamepad(event),
                );
            }
        });
    }
}
