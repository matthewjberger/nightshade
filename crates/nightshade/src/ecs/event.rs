//! The participant cycle's engine half: a curated, typed event stream and a
//! deferred command queue that game logic reads and writes without ever
//! touching the world.
//!
//! A participant is anything that reacts to the simulation and asks it to
//! change: a script, a plugin, native game code. Each cycle it reads
//! [`events`] and queues commands, never mutating the world directly.
//!
//! There is exactly one command type, and it lives one layer up in
//! `nightshade-api`: the [`Command`](../../../nightshade_api/enum.Command.html)
//! enum whose every variant is an API call, dispatched by `submit_command`.
//! The engine cannot name that type, so it carries queued commands as opaque
//! values and `nightshade-api` gives them meaning when it drains the queue.
//! That keeps a single command layer rather than a parallel engine copy.
//!
//! Two invariants hold across a cycle:
//!
//! 1. Every participant reads the same [`events`] set, frozen for the whole
//!    cycle by [`swap_events_system`] at frame start.
//! 2. No participant observes another's commands. Commands apply only after
//!    every participant has run, when `nightshade-api` drains the queue. A
//!    command's own effects surface as events one cycle later, never the same
//!    one.

use enum2schema::serde_json::Value;

use crate::ecs::world::{Entity, World};

/// Schema for an [`Entity`] field. The engine entity is a foreign type that
/// cannot derive `Schema`, and participants hold it as the packed integer the
/// scripting API hands them, so that is how the event schema describes it.
pub fn entity_schema() -> enum2schema::serde_json::Value {
    enum2schema::serde_json::json!({ "type": "integer" })
}

/// A fact the engine publishes for participants to react to. Curated on
/// purpose: only facts game logic can act on, never raw engine internals.
#[derive(Debug, Clone, Copy, PartialEq, enum2schema::Schema)]
pub enum Event {
    /// Two colliders began or ended touching. `started` distinguishes the
    /// two, `sensor` marks an intersection with a sensor collider.
    Collision {
        #[schema(with = crate::ecs::event::entity_schema)]
        a: Entity,
        #[schema(with = crate::ecs::event::entity_schema)]
        b: Entity,
        sensor: bool,
        started: bool,
    },
    /// An entity was despawned this cycle.
    Despawned {
        #[schema(with = crate::ecs::event::entity_schema)]
        entity: Entity,
    },
    /// A non looping animation player reached the end of its clip.
    AnimationFinished {
        #[schema(with = crate::ecs::event::entity_schema)]
        entity: Entity,
    },
    /// A navmesh agent reached its destination.
    NavigationArrived {
        #[schema(with = crate::ecs::event::entity_schema)]
        entity: Entity,
    },
}

/// One record in the participant cycle's journal: a command that was applied or
/// an event that was published, tagged with the cycle it happened in. Commands
/// are kept as their already formatted label and detail, because the engine
/// does not know the command type that produced them.
#[derive(Debug, Clone)]
pub enum JournalEntry {
    Event {
        cycle: u64,
        event: Event,
    },
    Command {
        cycle: u64,
        label: String,
        detail: String,
    },
}

/// How many journal records to retain when a consumer is slow to drain, so the
/// journal never grows without bound on a long running session.
const JOURNAL_CAPACITY: usize = 4096;

/// The participant cycle's shared state: the frozen `current` events, the
/// `pending` events accumulating for next cycle, the queued commands awaiting
/// dispatch as opaque values, and an optional journal a tool can drain to watch
/// the traffic.
#[derive(Default)]
pub struct Events {
    current: Vec<Event>,
    pending: Vec<Event>,
    pending_commands: Vec<Value>,
    journal: Vec<JournalEntry>,
    journal_enabled: bool,
    cycle: u64,
}

/// Publishes an event for participants to read next cycle.
pub fn emit_event(world: &mut World, event: Event) {
    let events = &mut world.resources.events;
    events.pending.push(event);
    if events.journal_enabled {
        record(
            events,
            JournalEntry::Event {
                cycle: events.cycle,
                event,
            },
        );
    }
}

/// Queues an opaque command value for `nightshade-api` to dispatch after this
/// cycle's participants have run. The value is the serde form of a
/// `nightshade_api::Command`.
pub fn queue_command(world: &mut World, command: Value) {
    world.resources.events.pending_commands.push(command);
}

/// Takes the queued command values, for the api dispatch to deserialize and
/// submit.
pub fn take_commands(world: &mut World) -> Vec<Value> {
    std::mem::take(&mut world.resources.events.pending_commands)
}

/// The events visible to participants this cycle.
pub fn events(world: &World) -> &[Event] {
    &world.resources.events.current
}

/// The current cycle counter, for stamping a journal entry.
pub fn cycle(world: &World) -> u64 {
    world.resources.events.cycle
}

/// Whether the journal is recording.
pub fn journal_enabled(world: &World) -> bool {
    world.resources.events.journal_enabled
}

/// Turns the journal on or off. A tool that watches command and event traffic
/// turns it on, a shipping build leaves it off so there is no record keeping
/// cost on the hot path.
pub fn set_journal_enabled(world: &mut World, enabled: bool) {
    world.resources.events.journal_enabled = enabled;
    if !enabled {
        world.resources.events.journal.clear();
    }
}

/// Takes the journal entries recorded since the last drain.
pub fn drain_journal(world: &mut World) -> Vec<JournalEntry> {
    std::mem::take(&mut world.resources.events.journal)
}

/// Records a dispatched command in the journal, by its already formatted label
/// and detail. Called by the api dispatch, which knows the command type.
pub fn journal_record_command(world: &mut World, label: String, detail: String) {
    let events = &mut world.resources.events;
    if events.journal_enabled {
        record(
            events,
            JournalEntry::Command {
                cycle: events.cycle,
                label,
                detail,
            },
        );
    }
}

fn record(events: &mut Events, entry: JournalEntry) {
    if events.journal.len() >= JOURNAL_CAPACITY {
        events.journal.remove(0);
    }
    events.journal.push(entry);
}

/// Frame start: last cycle's emitted events become this cycle's readable set
/// and the emit buffer is cleared, freezing the set for every participant.
pub fn swap_events_system(world: &mut World) {
    let events = &mut world.resources.events;
    events.cycle = events.cycle.wrapping_add(1);
    events.current.clear();
    std::mem::swap(&mut events.current, &mut events.pending);
}

/// Reads the physics collision events produced by the last physics step and
/// republishes them as [`Event::Collision`] for participants.
#[cfg(feature = "physics")]
pub fn emit_collision_events_system(world: &mut World) {
    use crate::ecs::physics::events::CollisionEventKind;
    let collisions =
        crate::ecs::physics::resources::physics_world_collision_events(&world.resources.physics)
            .to_vec();
    for collision in collisions {
        emit_event(
            world,
            Event::Collision {
                a: collision.entity_a,
                b: collision.entity_b,
                sensor: collision.is_sensor,
                started: collision.kind == CollisionEventKind::Started,
            },
        );
    }
}
