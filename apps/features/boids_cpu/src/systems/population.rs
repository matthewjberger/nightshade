use crate::components::{Position, Shade, Velocity};
use crate::data::Watched;
use crate::resources::{
    BOID_STEP, BoidsResources, GridBuilder, Integration, MIN_BOID_COUNT, bounds_for,
};
use nalgebra_glm::Vec3;
use nightshade::prelude::*;

pub fn apply_requested_count(
    mut resources: ResMut<BoidsResources>,
    grid: Option<ResMut<GridBuilder>>,
    integration: Option<ResMut<Integration>>,
    storage: &mut nightshade::ecs::Storage,
) {
    let requested = resources.requested_count;
    let current = resources.boids.len();

    if requested == current {
        return;
    }

    if requested > current {
        let added = spawn_boids(storage, requested - current, current);
        resources.boids.extend(added);
    } else {
        let removed: Vec<Entity> = resources.boids.split_off(requested);
        storage.despawn_entities(&removed);
    }

    apply_bounds(grid, integration, requested);
}

pub fn spawn_boids(
    storage: &mut nightshade::ecs::Storage,
    count: usize,
    already_spawned: usize,
) -> Vec<Entity> {
    let bounds = bounds_for(already_spawned + count);
    let entities = storage.spawn_entities::<(Position, Velocity, Shade)>(count);

    let mut noise = fastrand::Rng::with_seed(
        0x2545_f491_4f6c_dd1d ^ (already_spawned as u64).wrapping_mul(0x9e37_79b9),
    );

    if already_spawned == 0 {
        let mut filled = 0_usize;
        storage
            .query::<(&mut Position, &mut Velocity, &mut Shade)>()
            .for_each(|_entity, (position, velocity, shade)| {
                position.0 = random_point(&mut noise, bounds);
                velocity.0 = random_heading(&mut noise);
                shade.0 = shade_for(filled);
                filled += 1;
            });
    } else {
        for (offset, entity) in entities.iter().enumerate() {
            let index = already_spawned + offset;
            storage.set(*entity, Position(random_point(&mut noise, bounds)));
            storage.set(*entity, Velocity(random_heading(&mut noise)));
            storage.set(*entity, Shade(shade_for(index)));
        }
    }

    if already_spawned == 0
        && let Some(first) = entities.first()
    {
        storage.add_tag_type::<Watched>(*first);
    }

    entities
}

fn apply_bounds(
    grid: Option<ResMut<GridBuilder>>,
    integration: Option<ResMut<Integration>>,
    count: usize,
) {
    let bounds = bounds_for(count);
    if let Some(mut builder) = grid {
        builder.bounds = bounds;
    }
    if let Some(mut integration) = integration {
        integration.bounds = bounds;
    }
}

pub fn stepped_count(current: usize, steps: i32) -> usize {
    let step = BOID_STEP as i64;
    let requested = current as i64 + step * i64::from(steps);
    requested.max(MIN_BOID_COUNT as i64) as usize
}

fn random_point(noise: &mut fastrand::Rng, bounds: f32) -> Vec3 {
    Vec3::new(
        random_span(noise, bounds * 0.8),
        random_span(noise, bounds * 0.8),
        random_span(noise, bounds * 0.8),
    )
}

fn random_heading(noise: &mut fastrand::Rng) -> Vec3 {
    Vec3::new(
        random_span(noise, 12.0),
        random_span(noise, 6.0),
        random_span(noise, 12.0),
    )
}

pub fn shade_for(index: usize) -> [f32; 4] {
    let turn = |step: f64| ((index as f64 + 1.0) * step).fract() as f32;

    let hue = turn(0.618_033_988_749_895);
    let saturation = 0.78 + turn(0.276_393_202_250_021) * 0.22;
    let value = 0.55 + turn(0.414_213_562_373_095) * 0.40;

    let [red, green, blue] = hue_to_rgb(hue, saturation, value);
    [red, green, blue, 1.0]
}

fn hue_to_rgb(hue: f32, saturation: f32, value: f32) -> [f32; 3] {
    let sector = hue * 6.0;
    let offset = sector - sector.floor();
    let falling = value * (1.0 - saturation);
    let rising = value * (1.0 - saturation * offset);
    let peaking = value * (1.0 - saturation * (1.0 - offset));

    match sector as u32 % 6 {
        0 => [value, peaking, falling],
        1 => [rising, value, falling],
        2 => [falling, value, peaking],
        3 => [falling, rising, value],
        4 => [peaking, falling, value],
        _ => [value, falling, rising],
    }
}

fn random_span(noise: &mut fastrand::Rng, span: f32) -> f32 {
    (noise.f32() * 2.0 - 1.0) * span
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::INITIAL_BOID_COUNT;

    #[test]
    fn no_two_boids_share_a_colour() {
        let mut seen = std::collections::HashSet::new();

        for index in 0..20_000 {
            let shade = shade_for(index);
            let key = shade.map(|channel| (channel * 100_000.0) as i64);
            assert!(seen.insert(key), "index {index} repeated a colour");
        }
    }

    #[test]
    fn every_colour_channel_stays_in_range() {
        for index in [0, 1, 7, 999, 123_456] {
            for channel in shade_for(index) {
                assert!((0.0..=1.0).contains(&channel));
            }
        }
    }

    #[test]
    fn neighbouring_boids_are_not_neighbouring_colours() {
        let first = shade_for(1000);
        let second = shade_for(1001);

        let apart: f32 = (0..3).map(|band| (first[band] - second[band]).abs()).sum();
        assert!(apart > 0.3);
    }

    #[test]
    fn one_step_up_adds_the_step() {
        assert_eq!(
            stepped_count(INITIAL_BOID_COUNT, 1),
            INITIAL_BOID_COUNT + BOID_STEP
        );
    }

    #[test]
    fn one_step_down_takes_the_step() {
        assert_eq!(
            stepped_count(INITIAL_BOID_COUNT, -1),
            INITIAL_BOID_COUNT - BOID_STEP
        );
    }

    #[test]
    fn the_flock_will_not_shrink_below_its_floor() {
        assert_eq!(stepped_count(MIN_BOID_COUNT, -1), MIN_BOID_COUNT);
        assert_eq!(stepped_count(MIN_BOID_COUNT, -20), MIN_BOID_COUNT);
    }

    #[test]
    fn ten_times_the_boids_is_a_little_over_twice_the_box() {
        let small = bounds_for(100_000);
        let large = bounds_for(1_000_000);

        assert!((large / small - 10.0_f32.cbrt()).abs() < 1.0e-4);
    }

    #[test]
    fn the_box_holds_density_steady() {
        let density = |count: usize| {
            let side = bounds_for(count) * 2.0;
            count as f32 / (side * side * side)
        };

        assert!((density(50_000) - density(2_000_000)).abs() < 1.0e-9);
    }
}
