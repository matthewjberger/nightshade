//! Standalone use of the archetype ECS: register components, spawn a batch, run
//! a mutable tuple query and a read-only query, and filter by a marker tag. No
//! engine and no renderer, just `nightshade-ecs`.
//!
//! ```sh
//! cargo run --example basic
//! ```

use nightshade_ecs::dynamic::DynWorld;

#[derive(Default, Clone, Debug)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Default, Clone, Debug)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Default, Clone, Debug)]
struct Health {
    value: f32,
}

struct Player;

nightshade_ecs::impl_component!(Position, Velocity, Health);

fn main() {
    let mut world = DynWorld::new();

    let player = world.spawn((
        Position::default(),
        Velocity { x: 1.0, y: 0.5 },
        Health { value: 100.0 },
    ));
    world.add_tag_type::<Player>(player);

    for index in 1..5 {
        world.spawn((
            Position {
                x: index as f32,
                y: 0.0,
            },
            Velocity { x: 0.5, y: 0.0 },
            Health { value: 60.0 },
        ));
    }

    // Borrow mutability comes from the tuple, so only Position is written.
    world
        .query::<(&mut Position, &Velocity)>()
        .for_each(|_entity, (position, velocity)| {
            position.x += velocity.x;
            position.y += velocity.y;
        });

    // A read-only query on &world is a real iterator.
    for (entity, position) in world.query_ref::<&Position>().iter() {
        println!("{entity:?} moved to ({:.1}, {:.1})", position.x, position.y);
    }

    // Tags filter a query without fragmenting archetypes.
    let players = world
        .query_ref::<&Health>()
        .with_tag_type::<Player>()
        .iter()
        .count();
    let total_health: f32 = world
        .query_ref::<&Health>()
        .iter()
        .map(|(_entity, health)| health.value)
        .sum();
    println!("{players} player, {total_health} total health");
}
