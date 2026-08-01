#[cfg(not(target_os = "android"))]
use crate::contract::{Gamepad, GamepadAxis, GamepadButton};
#[cfg(not(target_os = "android"))]
use crate::data::{AXES, BUTTONS, map_button};
use crate::resources::GamepadBackend;

#[cfg(target_os = "android")]
use crate::contract::{Gamepad, GamepadButton};
#[cfg(target_os = "android")]
use crate::data::android::{map_axes, map_button, map_hat};
#[cfg(target_os = "android")]
use nightshade_platform::contract::input::{AndroidGamepadEvent, AndroidGamepadEvents};
#[cfg(target_os = "android")]
use std::collections::HashSet;

#[cfg(not(target_os = "android"))]
pub fn poll_gamepad(backend: &mut GamepadBackend, pad_state: &mut Gamepad) -> Vec<(usize, bool)> {
    let mut connection_events = Vec::new();
    let mut just_pressed = Vec::new();

    let mut guard = backend.gilrs.lock().unwrap();
    if guard.is_none() {
        match gilrs::Gilrs::new() {
            Ok(new_gilrs) => *guard = Some(new_gilrs),
            Err(_) => {
                drop(guard);
                pad_state.clear();
                return connection_events;
            }
        }
    }
    let gilrs = guard.as_mut().unwrap();

    let mut active = backend.gamepad;
    while let Some(event) = gilrs.next_event() {
        match event.event {
            gilrs::EventType::Connected => connection_events.push((event.id.into(), true)),
            gilrs::EventType::Disconnected => connection_events.push((event.id.into(), false)),
            gilrs::EventType::ButtonPressed(button, _) => {
                if let Some(mapped) = map_button(button) {
                    just_pressed.push(mapped);
                }
            }
            _ => {}
        }
        active = Some(event.id);
    }

    if active.is_none_or(|id| gilrs.connected_gamepad(id).is_none()) {
        active = gilrs
            .gamepads()
            .find(|(_, pad)| pad.is_connected())
            .map(|(id, _)| id);
    }

    let snapshot = active.map(|id| {
        let pad = gilrs.gamepad(id);
        let axes: Vec<(GamepadAxis, f32)> = AXES
            .iter()
            .map(|(raw, mapped)| (*mapped, pad.value(*raw)))
            .collect();
        let held: Vec<GamepadButton> = BUTTONS
            .iter()
            .filter(|(raw, _)| pad.is_pressed(*raw))
            .map(|(_, mapped)| *mapped)
            .collect();
        (axes, held)
    });

    drop(guard);
    backend.gamepad = active;

    pad_state.clear();
    if let Some((axes, held)) = snapshot {
        pad_state.connected = true;
        pad_state.axes = axes;
        pad_state.held = held.into_iter().collect();
    }
    pad_state.just_pressed = just_pressed.into_iter().collect();

    connection_events
}

#[cfg(target_os = "android")]
pub fn poll_gamepad(
    backend: &mut GamepadBackend,
    pad_state: &mut Gamepad,
    events: &AndroidGamepadEvents,
) -> Vec<(usize, bool)> {
    let mut connection_events = Vec::new();

    for event in &events.events {
        match event {
            AndroidGamepadEvent::Button {
                device,
                keycode,
                pressed,
            } => {
                record_device(backend, *device, &mut connection_events);
                let Some(button) = map_button(*keycode) else {
                    continue;
                };
                if *pressed {
                    backend.buttons.insert(button);
                } else {
                    backend.buttons.remove(&button);
                }
            }
            AndroidGamepadEvent::Axes { device, values } => {
                record_device(backend, *device, &mut connection_events);
                backend.axes = map_axes(values);
                backend.hat = map_hat(values).into_iter().collect();
            }
        }
    }

    let previously_held = std::mem::take(&mut pad_state.held);
    pad_state.clear();
    if backend.device.is_none() {
        return connection_events;
    }

    let held: HashSet<GamepadButton> = backend.buttons.union(&backend.hat).copied().collect();
    pad_state.connected = true;
    pad_state.axes = backend.axes.clone();
    pad_state.just_pressed = held.difference(&previously_held).copied().collect();
    pad_state.held = held;

    connection_events
}

#[cfg(target_os = "android")]
fn record_device(
    backend: &mut GamepadBackend,
    device: i32,
    connection_events: &mut Vec<(usize, bool)>,
) {
    if backend.device == Some(device) {
        return;
    }
    backend.device = Some(device);
    connection_events.push((device as usize, true));
}
