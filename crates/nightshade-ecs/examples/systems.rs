//! Systems as plain functions whose parameters are `Res`, `ResMut`, and `Query`,
//! registered on a `Schedule` and run each frame. This is the execution layer
//! over the storage: no trait to implement, the borrow checker is the access
//! checker.
//!
//! ```sh
//! cargo run --example systems
//! ```

use nightshade_ecs::Schedule;
use nightshade_ecs::dynamic::DynWorld;
use nightshade_ecs::system_param::{Query, Res, ResMut, ScheduleExt};

#[derive(Default, Clone)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Default, Clone)]
struct Velocity {
    x: f32,
    y: f32,
}

nightshade_ecs::impl_component!(Position, Velocity);

struct DeltaTime(f32);
struct Steps(u32);

fn movement(delta_time: Res<DeltaTime>, query: Query<(&mut Position, &Velocity)>) {
    query.for_each(|_entity, (position, velocity)| {
        position.x += velocity.x * delta_time.0;
        position.y += velocity.y * delta_time.0;
    });
}

fn count_steps(mut steps: ResMut<Steps>) {
    steps.0 += 1;
}

fn main() {
    let mut world = DynWorld::new();
    world.insert_resources((DeltaTime(0.016), Steps(0)));
    world.spawn((Position::default(), Velocity { x: 10.0, y: 0.0 }));

    let mut schedule = Schedule::new();
    schedule.add_systems((movement, count_steps));

    for _ in 0..3 {
        schedule.run(&mut world);
        world.step();
    }

    let (_entity, position) = world.query_ref::<&Position>().single().unwrap();
    println!(
        "after {} steps, x = {:.3}",
        world.res::<Steps>().0,
        position.x
    );
}
