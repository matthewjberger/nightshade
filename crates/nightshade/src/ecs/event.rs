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

use crate::ecs::world::{Entity, World};

/// Schema for an [`Entity`] field. The engine entity is a foreign type that
/// cannot derive `Schema`, so this matches its serde form, the same `{ id,
/// generation }` object a command reference uses, so the published event schema
/// is honest about what a serialized event carries. The scripting layer packs
/// an entity into an integer for a script's convenience, but that is a binding
/// detail, not the wire form.
pub fn entity_schema() -> enum2schema::serde_json::Value {
    enum2schema::serde_json::json!({
        "type": "object",
        "properties": {
            "id": { "type": "integer" },
            "generation": { "type": "integer" }
        },
        "required": ["id", "generation"]
    })
}

/// A fact the engine publishes for participants to react to. Curated on
/// purpose: only facts game logic can act on, never raw engine internals.
/// Serializable so it crosses the same boundary commands do: an out of process
/// or language-bound consumer drains events as json and reacts.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, enum2schema::Schema)]
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
    /// Playback of a clip passed a named marker on its notify track this cycle.
    /// `name` is the marker's label, the cue for a footstep, a hit frame, or a
    /// particle spawn.
    AnimationEvent {
        #[schema(with = crate::ecs::event::entity_schema)]
        entity: Entity,
        name: String,
    },
    /// A navmesh agent reached its destination.
    NavigationArrived {
        #[schema(with = crate::ecs::event::entity_schema)]
        entity: Entity,
    },
}

/// One record in the engine's event journal: an event that was published,
/// tagged with the cycle it happened in.
#[derive(Debug, Clone)]
pub struct JournalEntry {
    pub cycle: u64,
    pub event: Event,
}

/// How many journal records to retain when a consumer is slow to drain, so the
/// journal never grows without bound on a long running session.
const JOURNAL_CAPACITY: usize = 4096;

/// The event cycle's shared state: the frozen `current` events, the `pending`
/// events accumulating for next cycle, and an optional journal a tool drains to
/// watch what was published.
#[derive(Default)]
pub struct Events {
    current: Vec<Event>,
    pending: Vec<Event>,
    journal: Vec<JournalEntry>,
    journal_enabled: bool,
    cycle: u64,
}

/// Publishes an event for participants to read next cycle.
pub fn emit_event(world: &mut World, event: Event) {
    let events = &mut world.res_mut::<crate::ecs::event::Events>();
    if events.journal_enabled {
        record(
            events,
            JournalEntry {
                cycle: events.cycle,
                event: event.clone(),
            },
        );
    }
    events.pending.push(event);
}

/// The events visible to participants this cycle.
pub fn events(world: &World) -> &[Event] {
    &world.res::<crate::ecs::event::Events>().current
}

/// The current cycle counter, for stamping a journal entry.
pub fn cycle(world: &World) -> u64 {
    world.res::<crate::ecs::event::Events>().cycle
}

/// Whether the journal is recording.
pub fn journal_enabled(world: &World) -> bool {
    world.res::<crate::ecs::event::Events>().journal_enabled
}

/// Turns the journal on or off. A tool that watches command and event traffic
/// turns it on, a shipping build leaves it off so there is no record keeping
/// cost on the hot path.
pub fn set_journal_enabled(world: &mut World, enabled: bool) {
    world.res_mut::<crate::ecs::event::Events>().journal_enabled = enabled;
    if !enabled {
        world.res_mut::<crate::ecs::event::Events>().journal.clear();
    }
}

/// Takes the journal entries recorded since the last drain.
pub fn drain_journal(world: &mut World) -> Vec<JournalEntry> {
    std::mem::take(&mut world.res_mut::<crate::ecs::event::Events>().journal)
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
    let events = &mut world.res_mut::<crate::ecs::event::Events>();
    events.cycle = events.cycle.wrapping_add(1);
    events.current.clear();
    std::mem::swap(&mut events.current, &mut events.pending);
}

/// Reads the physics collision events produced by the last physics step and
/// republishes them as [`Event::Collision`] for participants.
#[cfg(feature = "physics")]
pub fn emit_collision_events_system(world: &mut World) {
    use crate::plugins::physics::events::CollisionEventKind;
    use crate::plugins::physics::resources::PhysicsWorld;
    let collisions = crate::plugins::physics::resources::physics_world_collision_events(
        world.plugin_resource::<PhysicsWorld>(),
    )
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
