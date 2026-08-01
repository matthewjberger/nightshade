use crate::data::{BOID_STEP, MAX_BOID_COUNT, MIN_BOID_COUNT};
use nightshade_random::Random;
use nightshade_random::commands::span;

pub fn bounds_for(count: usize) -> f32 {
    const DENSITY_AT: f32 = 1_000_000.0;
    const BOUNDS_AT: f32 = 260.0;

    BOUNDS_AT * (count as f32 / DENSITY_AT).cbrt().max(0.05)
}

pub fn stepped_count(current: usize, steps: i32) -> usize {
    let step = BOID_STEP as i64;
    let requested = current as i64 + step * i64::from(steps);
    requested.clamp(MIN_BOID_COUNT as i64, MAX_BOID_COUNT as i64) as usize
}

pub fn grid_shape(bounds: f32, cell_size: f32) -> (u32, u32) {
    let span = bounds * 2.0 + cell_size * 8.0;
    let dimension = ((span / cell_size).ceil() as u32).max(1);
    (dimension, dimension * dimension * dimension)
}

pub fn seed_flock(count: u32, bounds: f32) -> (Vec<[f32; 4]>, Vec<[f32; 4]>) {
    let mut positions: Vec<[f32; 4]> = Vec::with_capacity(count as usize);
    let mut velocities: Vec<[f32; 4]> = Vec::with_capacity(count as usize);
    let mut noise = Random::seeded(0x2545_f491_4f6c_dd1d);
    let spread = bounds * 0.8;

    for _ in 0..count {
        positions.push([
            span(&mut noise, spread),
            span(&mut noise, spread),
            span(&mut noise, spread),
            1.0,
        ]);
        velocities.push([
            span(&mut noise, 12.0),
            span(&mut noise, 6.0),
            span(&mut noise, 12.0),
            0.0,
        ]);
    }

    (positions, velocities)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::INITIAL_BOID_COUNT;

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

        let reference = density(500_000);
        assert!((reference - density(2_000_000)).abs() / reference < 1.0e-6);
    }

    #[test]
    fn one_step_up_adds_the_step() {
        assert_eq!(
            stepped_count(INITIAL_BOID_COUNT, 1),
            INITIAL_BOID_COUNT + BOID_STEP
        );
    }

    #[test]
    fn the_flock_will_not_grow_past_the_buffers() {
        assert_eq!(stepped_count(MAX_BOID_COUNT, 1), MAX_BOID_COUNT);
        assert_eq!(stepped_count(MAX_BOID_COUNT, 40), MAX_BOID_COUNT);
    }

    #[test]
    fn the_flock_will_not_shrink_below_its_floor() {
        assert_eq!(stepped_count(MIN_BOID_COUNT, -1), MIN_BOID_COUNT);
        assert_eq!(stepped_count(MIN_BOID_COUNT, -20), MIN_BOID_COUNT);
    }

    #[test]
    fn the_grid_covers_the_box_and_the_margin_around_it() {
        let (dimension, cells) = grid_shape(120.0, 5.0);

        assert!(f32::from(u16::try_from(dimension).unwrap()) * 5.0 >= 240.0);
        assert_eq!(cells, dimension * dimension * dimension);
    }

    #[test]
    fn a_larger_box_needs_more_cells() {
        let (small, _) = grid_shape(120.0, 5.0);
        let (large, _) = grid_shape(240.0, 5.0);

        assert!(large > small);
    }

    #[test]
    fn the_seed_spread_stays_inside_the_box() {
        let mut noise = Random::seeded(1);

        for _ in 0..10_000 {
            assert!(span(&mut noise, 96.0).abs() <= 96.0);
        }
    }
}
