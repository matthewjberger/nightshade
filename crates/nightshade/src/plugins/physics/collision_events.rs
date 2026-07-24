//! Republishes physics collision events as engine `Event::Collision`.

/// Reads the physics collision events produced by the last physics step and
/// republishes them as [`Event::Collision`] for participants.
#[cfg(feature = "physics")]
pub fn emit_collision_events_system(world: &mut crate::ecs::world::World) {
    use crate::plugins::physics::events::CollisionEventKind;
    use crate::plugins::physics::resources::PhysicsWorld;
    if world.ecs.resource::<PhysicsWorld>().is_none() {
        return;
    }
    let collisions = crate::plugins::physics::resources::physics_world_collision_events(
        world.plugin_resource::<PhysicsWorld>(),
    )
    .to_vec();
    for collision in collisions {
        crate::ecs::event::emit_event(
            world,
            crate::ecs::event::Event::Collision {
                a: collision.entity_a,
                b: collision.entity_b,
                sensor: collision.is_sensor,
                started: collision.kind == CollisionEventKind::Started,
            },
        );
    }
}
