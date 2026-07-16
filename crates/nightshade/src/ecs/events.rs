//! Application event helpers over freecs's shared [`EventBus`].
//!
//! The reader and writer system parameters are freecs's
//! [`EventReader<T>`](freecs::system_param::EventReader) /
//! [`EventWriter<T>`](freecs::system_param::EventWriter): cursor-based,
//! exactly-once reads over a two-frame buffer, coexisting with `&mut World` in
//! the same system. The engine writes application events onto the bus as winit
//! and the input pipeline deliver them and expires them once per frame with
//! `world.ecs.events.update()`, so a reader sees each event once and never the
//! previous frame's.

pub use freecs::system_param::{EventReader, EventWriter};

use crate::ecs::input::resources::Input;
use crate::ecs::world::World;
use freecs::system_param::EventHost;

/// Records an application event on the freecs event bus, read through
/// [`EventReader<AppEvent>`](freecs::system_param::EventReader), and mirrors it
/// into the legacy `Input::events` list drained by hand. Both hold the same
/// events, so a system reading either sees identical input.
pub(crate) fn emit_app_event(world: &mut World, event: crate::ecs::input::events::AppEvent) {
    world.event_bus_mut().send(event.clone());
    world.res_mut::<Input>().events.push(event);
}
