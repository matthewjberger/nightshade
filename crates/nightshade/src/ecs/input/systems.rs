pub use crate::ecs::input::commands::*;
#[cfg(feature = "gamepad")]
pub use crate::ecs::input::queries::*;

#[cfg(feature = "gamepad")]
use crate::ecs::event_bus::commands::publish_event;
use crate::ecs::world::{Vec2, World, resources::MouseState};
#[cfg(feature = "gamepad")]
use crate::ecs::world::events::*;

pub fn reset_mouse_system(world: &mut World) {
    let _span = tracing::info_span!("reset_mouse").entered();
    let mouse = &mut world.resources.input.mouse;
    if mouse.state.contains(MouseState::SCROLLED) {
        mouse.wheel_delta = Vec2::new(0.0, 0.0);
    }
    mouse.state.set(MouseState::SCROLLED, false);
    if mouse.state.contains(MouseState::MOVED) {
        mouse.position_delta = Vec2::new(0.0, 0.0);
    }
    mouse.state.set(MouseState::MOVED, false);
    mouse.raw_mouse_delta = Vec2::new(0.0, 0.0);
    mouse.state.set(MouseState::LEFT_JUST_PRESSED, false);
    mouse.state.set(MouseState::MIDDLE_JUST_PRESSED, false);
    mouse.state.set(MouseState::RIGHT_JUST_PRESSED, false);
    mouse.state.set(MouseState::LEFT_JUST_RELEASED, false);
    mouse.state.set(MouseState::MIDDLE_JUST_RELEASED, false);
    mouse.state.set(MouseState::RIGHT_JUST_RELEASED, false);
}

pub fn reset_touch_system(world: &mut World) {
    let _span = tracing::info_span!("reset_touch").entered();
    use crate::ecs::input::resources::TouchGesture;

    let touch = &mut world.resources.input.touch;
    touch.clear_ended_touches();
    touch.gesture = TouchGesture::None;
}

#[cfg(feature = "gamepad")]
pub fn gamepad_input_system(world: &mut World) {
    let _span = tracing::info_span!("gamepad_input").entered();
    if world.resources.input.gamepad.gilrs.is_none() {
        if let Ok(gilrs) = gilrs::Gilrs::new() {
            world.resources.input.gamepad.gilrs = Some(gilrs);
        } else {
            return;
        }
    }

    world.resources.input.gamepad.events.clear();

    let Some(gilrs) = &mut world.resources.input.gamepad.gilrs else {
        return;
    };

    let mut connection_events = Vec::new();

    while let Some(event) = gilrs.next_event() {
        if matches!(event.event, gilrs::EventType::Connected) {
            connection_events.push((event.id.into(), true));
        } else if matches!(event.event, gilrs::EventType::Disconnected) {
            connection_events.push((event.id.into(), false));
        }

        world.resources.input.gamepad.gamepad = Some(event.id);
        world.resources.input.gamepad.events.push(event);
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

pub fn escape_key_exit_system(world: &mut World) {
    if world
        .resources
        .input
        .keyboard
        .is_key_pressed(winit::keyboard::KeyCode::Escape)
    {
        world.resources.window.should_exit = true;
    }
}
