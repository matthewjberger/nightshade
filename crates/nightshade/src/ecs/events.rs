//! Application event helpers over nightshade_ecs's shared [`EventBus`].
//!
//! The reader and writer system parameters are nightshade_ecs's
//! [`EventReader<T>`](nightshade_ecs::system_param::EventReader) /
//! [`EventWriter<T>`](nightshade_ecs::system_param::EventWriter): cursor-based,
//! exactly-once reads over a two-frame buffer, coexisting with `&mut World` in
//! the same system. The engine writes application events onto the bus as winit
//! and the input pipeline deliver them and expires them once per frame with
//! `world.ecs.events.update()`, so a reader sees each event once and never the
//! previous frame's.

pub use nightshade_ecs::system_param::{EventReader, EventWriter};

use crate::platform::input::resources::Input;
use crate::ecs::world::World;
use nightshade_ecs::system_param::EventHost;

/// Records an application event on the nightshade_ecs event bus, read through
/// [`EventReader<AppEvent>`](nightshade_ecs::system_param::EventReader), and mirrors it
/// into the legacy `Input::events` list drained by hand. Both hold the same
/// events, so a system reading either sees identical input.
pub(crate) fn emit_app_event(world: &mut World, event: crate::platform::input::events::AppEvent) {
    world.event_bus_mut().send(event.clone());
    world.res_mut::<Input>().events.push(event);
}
