use nightshade_ecs::World;
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

fn main() {
    let mut world = World::new();

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

    world
        .query::<(&mut Position, &Velocity)>()
        .for_each(|_entity, (position, velocity)| {
            position.x += velocity.x;
            position.y += velocity.y;
        });

    for (entity, position) in world.query_ref::<&Position>().iter() {
        println!("{entity:?} moved to ({:.1}, {:.1})", position.x, position.y);
    }

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
