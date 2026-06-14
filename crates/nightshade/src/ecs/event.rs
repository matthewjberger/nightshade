//! The participant cycle: a curated, typed event stream and a deferred command
//! buffer that game logic reads and writes without ever touching the world.
//!
//! A participant is anything that reacts to the simulation and asks it to
//! change: a script, a plugin, native game code. Each cycle it reads
//! [`events`] and pushes [`Command`]s, never mutating the world directly.
//!
//! Two invariants hold across a cycle, which is what lets several participants
//! coexist without seeing each other mid-step:
//!
//! 1. Every participant reads the same [`events`] set. The set is frozen for
//!    the whole cycle by [`swap_events_system`] at frame start.
//! 2. No participant observes another's commands. Commands apply only after
//!    every participant has run, in [`dispatch_commands_system`]. A command's
//!    own effects surface as events one cycle later, never the same one.
//!
//! This is distinct from [`event_bus`](crate::ecs::event_bus), which is a
//! type erased queue for arbitrary app messages. [`Event`] is a closed,
//! schema described set of facts the engine itself publishes.

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
    /// An entity was despawned this cycle, including by a [`Command::Despawn`].
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

/// A mutation a participant requests. Participants append these and the engine
/// applies them in [`dispatch_commands_system`] once the cycle's participants
/// have all run.
#[derive(Debug, Clone)]
pub enum Command {
    /// Set a body's linear velocity.
    SetVelocity {
        entity: Entity,
        velocity: nalgebra_glm::Vec3,
    },
    /// Set an entity's local translation.
    SetTranslation {
        entity: Entity,
        position: nalgebra_glm::Vec3,
    },
    /// Despawn an entity and its descendants.
    Despawn { entity: Entity },
    /// Replace a text entity's string.
    SetText { entity: Entity, text: String },
}

/// The participant cycle's shared state: the frozen `current` events, the
/// `pending` events accumulating for next cycle, and the `commands` buffer
/// participants fill this cycle.
#[derive(Default)]
pub struct Events {
    current: Vec<Event>,
    pending: Vec<Event>,
    commands: Vec<Command>,
}

/// Publishes an event for participants to read next cycle.
pub fn emit_event(world: &mut World, event: Event) {
    world.resources.events.pending.push(event);
}

/// Queues a command to apply after this cycle's participants have run.
pub fn push_command(world: &mut World, command: Command) {
    world.resources.events.commands.push(command);
}

/// The events visible to participants this cycle.
pub fn events(world: &World) -> &[Event] {
    &world.resources.events.current
}

/// Frame start: last cycle's emitted events become this cycle's readable set
/// and the emit buffer is cleared, freezing the set for every participant.
pub fn swap_events_system(world: &mut World) {
    let events = &mut world.resources.events;
    events.current.clear();
    std::mem::swap(&mut events.current, &mut events.pending);
}

/// After participants run: drain and apply their commands. A despawn publishes
/// a [`Event::Despawned`] into the pending buffer, so the effect is observed
/// next cycle rather than the current one.
pub fn dispatch_commands_system(world: &mut World) {
    let commands = std::mem::take(&mut world.resources.events.commands);
    for command in commands {
        apply_command(world, command);
    }
}

fn apply_command(world: &mut World, command: Command) {
    use crate::prelude::{despawn_recursive_immediate, mark_local_transform_dirty};
    match command {
        #[cfg(feature = "physics")]
        Command::SetVelocity { entity, velocity } => {
            crate::ecs::physics::resources::physics_world_set_linear_velocity(
                &mut world.resources.physics,
                entity,
                velocity,
            );
        }
        #[cfg(not(feature = "physics"))]
        Command::SetVelocity { .. } => {}
        Command::SetTranslation { entity, position } => {
            if let Some(transform) = world.core.get_local_transform_mut(entity) {
                transform.translation = position;
            }
            mark_local_transform_dirty(world, entity);
        }
        Command::Despawn { entity } => {
            despawn_recursive_immediate(world, entity);
            emit_event(world, Event::Despawned { entity });
        }
        Command::SetText { .. } => {}
    }
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
