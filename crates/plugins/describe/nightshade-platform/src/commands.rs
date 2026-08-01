use crate::contract::AppEvent;
#[cfg(target_os = "android")]
pub mod android_input;
pub mod host;
pub mod input;
pub mod input_capture;
pub mod time;
pub mod window;

pub use host::*;

pub fn emit_app_event(
    events: &mut nightshade_ecs::EventBus,
    input: &mut crate::contract::input::Input,
    event: AppEvent,
) {
    events.send(event.clone());
    input.events.push(event);
}
