use nightshade_ecs::World;

use nightshade_ecs::system_param::{Query, ScheduleExt};

const ENTITY_COUNT: usize = 100_000;
const FRAME_COUNT: usize = 120;

#[derive(Default, Clone)]
struct Phase(f32);

#[derive(Default, Clone)]
struct Position(f32);

#[derive(Default, Clone)]
struct Velocity(f32);

#[derive(Default, Clone)]
struct Heat(f32);

#[derive(Default, Clone)]
struct Charge(f32);

#[derive(Default, Clone)]
struct Wear(f32);

#[derive(Default, Clone)]
struct Spin(f32);

fn integrate(query: Query<(&mut Position, &Phase)>) {
    query.for_each(|_entity, (position, phase)| {
        position.0 = (position.0 + phase.0.sin()).rem_euclid(1024.0);
    });
}

fn drift(query: Query<(&mut Velocity, &Phase)>) {
    query.for_each(|_entity, (velocity, phase)| {
        velocity.0 = (velocity.0 * 0.99 + phase.0.cos()).clamp(-64.0, 64.0);
    });
}

fn warm(query: Query<(&mut Heat, &Phase)>) {
    query.for_each(|_entity, (heat, phase)| {
        heat.0 = (heat.0 + phase.0.abs().sqrt()).rem_euclid(512.0);
    });
}

fn charge(query: Query<(&mut Charge, &Phase)>) {
    query.for_each(|_entity, (charge, phase)| {
        charge.0 = (charge.0 + phase.0.tan().clamp(-8.0, 8.0)).rem_euclid(256.0);
    });
}

fn abrade(query: Query<(&mut Wear, &Phase)>) {
    query.for_each(|_entity, (wear, phase)| {
        wear.0 = (wear.0 + (phase.0 * 0.5).exp().min(32.0)).rem_euclid(128.0);
    });
}

fn rotate(query: Query<(&mut Spin, &Phase)>) {
    query.for_each(|_entity, (spin, phase)| {
        spin.0 = (spin.0 + phase.0.ln_1p().abs()).rem_euclid(64.0);
    });
}

fn main() {
    let mut world = World::new();
    world.spawn_entities::<(Phase, Position, Velocity, Heat, Charge, Wear, Spin)>(ENTITY_COUNT);
    world
        .query::<&mut Phase>()
        .for_each(|_entity, phase| phase.0 = 0.5);

    let mut schedule = nightshade_ecs::Schedule::<World>::new();
    schedule
        .add_system("integrate", integrate)
        .add_system("drift", drift)
        .add_system("warm", warm)
        .add_system("charge", charge)
        .add_system("abrade", abrade)
        .add_system("rotate", rotate);

    let names: Vec<String> = schedule.names().map(str::to_owned).collect();
    println!("systems: {}", names.len());
    println!("entities: {ENTITY_COUNT}");
    println!("frames: {FRAME_COUNT}");
    println!(
        "parallel feature: {}",
        if cfg!(feature = "parallel") {
            "on"
        } else {
            "off"
        }
    );
    let batches = schedule.batches();
    println!("batches: {}", batches.len());
    for batch in &batches {
        let members: Vec<&str> = names[batch.clone()].iter().map(String::as_str).collect();
        println!("  {members:?}");
    }

    schedule.run(&mut world);

    let started = std::time::Instant::now();
    for _ in 0..FRAME_COUNT {
        schedule.run(&mut world);
    }
    let elapsed = started.elapsed();

    println!(
        "total: {:.3} s, per frame: {:.3} ms",
        elapsed.as_secs_f64(),
        elapsed.as_secs_f64() * 1000.0 / FRAME_COUNT as f64
    );
}
