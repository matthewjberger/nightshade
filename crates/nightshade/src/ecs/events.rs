//! Per-frame event queues and the reader and writer system parameters.
//!
//! An [`EventQueue<T>`] holds the events of one type produced during the
//! current frame. The engine fills the queue while winit and the input
//! pipeline deliver events, and clears it once the frame's systems have run,
//! so a system that reads through [`EventReader<T>`] sees exactly this frame's
//! events and never the previous frame's. Reading does not consume, so several
//! systems can read the same queue in one frame without draining it for one
//! another.

use crate::ecs::input::resources::Input;
use crate::ecs::world::World;

/// A frame's worth of events of type `T`, held as a resource in the world's
/// map. The engine clears it at the end of every frame.
pub struct EventQueue<T> {
    events: Vec<T>,
}

impl<T> Default for EventQueue<T> {
    fn default() -> Self {
        Self { events: Vec::new() }
    }
}

impl<T> EventQueue<T> {
    pub fn send(&mut self, event: T) {
        self.events.push(event);
    }

    pub fn extend(&mut self, events: impl IntoIterator<Item = T>) {
        self.events.extend(events);
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.events.iter()
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }
}

/// A non-consuming read-only view over an [`EventQueue<T>`], resolved as a
/// system parameter. Iterating with [`read`](EventReader::read) yields shared
/// references to this frame's events and leaves the queue intact.
pub struct EventReader<'world, T> {
    queue: &'world EventQueue<T>,
}

impl<T> EventReader<'_, T> {
    pub fn read(&self) -> impl Iterator<Item = &T> {
        self.queue.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

/// A writer into an [`EventQueue<T>`], resolved as a system parameter, for
/// systems that produce events of their own.
pub struct EventWriter<'world, T> {
    queue: &'world mut EventQueue<T>,
}

impl<T> EventWriter<'_, T> {
    pub fn send(&mut self, event: T) {
        self.queue.send(event);
    }
}

impl<'a, T: Send + Sync + 'static> freecs::system_param::ResourceParam for EventReader<'a, T> {
    type Resource = EventQueue<T>;
    type Item<'world> = EventReader<'world, T>;
    fn build(resource: &mut EventQueue<T>) -> EventReader<'_, T> {
        EventReader { queue: resource }
    }
}

impl<'a, T: Send + Sync + 'static> freecs::system_param::ResourceParam for EventWriter<'a, T> {
    type Resource = EventQueue<T>;
    type Item<'world> = EventWriter<'world, T>;
    fn build(resource: &mut EventQueue<T>) -> EventWriter<'_, T> {
        EventWriter { queue: resource }
    }
}

/// Records an application event for the frame, mirroring it into both the
/// per-frame [`EventQueue`] read through [`EventReader`] and the legacy
/// `Input::events` list drained by hand. Both hold the same events, so a
/// system reading either sees identical input.
pub(crate) fn emit_app_event(world: &mut World, event: crate::ecs::input::events::AppEvent) {
    world
        .expect_resource_mut::<EventQueue<crate::ecs::input::events::AppEvent>>()
        .send(event.clone());
    world.expect_resource_mut::<Input>().events.push(event);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_sends_reads_and_clears() {
        let mut queue = EventQueue::<i32>::default();
        assert!(queue.is_empty());

        queue.send(1);
        queue.send(2);
        queue.extend([3, 4]);
        assert_eq!(queue.len(), 4);
        assert!(!queue.is_empty());

        let reader = EventReader { queue: &queue };
        let collected: Vec<i32> = reader.read().copied().collect();
        assert_eq!(collected, vec![1, 2, 3, 4]);
        assert_eq!(reader.len(), 4);
        assert!(!reader.is_empty());

        queue.clear();
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());

        let reader = EventReader { queue: &queue };
        assert!(reader.is_empty());
        assert_eq!(reader.read().count(), 0);
    }

    #[test]
    fn writer_sends_into_queue() {
        let mut queue = EventQueue::<i32>::default();
        {
            let mut writer = EventWriter { queue: &mut queue };
            writer.send(7);
            writer.send(8);
        }
        let collected: Vec<i32> = queue.iter().copied().collect();
        assert_eq!(collected, vec![7, 8]);
    }
}
