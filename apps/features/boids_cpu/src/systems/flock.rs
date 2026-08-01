use crate::components::{Position, Velocity};
use crate::data::{Applied, BoidUpdate, FlockGrid, Region, RegionSettings, region_slots};
use crate::resources::{
    ALIGNMENT_WEIGHT, CANDIDATE_CAP, COHESION_WEIGHT, GRID_MARGIN, GridBuilder, Integration,
    SEARCH_RADIUS, SEPARATION_WEIGHT,
};
use nalgebra_glm::Vec3;
use nightshade::ecs::system_param::{Query, ResMut, ScheduleExt};
use nightshade::prelude::Instant;
use std::sync::Arc;

pub fn schedule() -> nightshade::ecs::Schedule<nightshade::ecs::World> {
    let mut schedule = nightshade::ecs::Schedule::<nightshade::ecs::World>::new();
    schedule.add_system("gather", gather);
    schedule.add_system("hand_6", hand_6);
    schedule.add_system("hand_5", hand_5);
    schedule.add_system("hand_4", hand_4);
    schedule.add_system("hand_3", hand_3);
    schedule.add_system("hand_2", hand_2);
    schedule.add_system("hand_1", hand_1);
    schedule.add_system("hand_0", hand_0);
    schedule.add_system("region_0", steer_region::<0>);
    schedule.add_system("region_1", steer_region::<1>);
    schedule.add_system("region_2", steer_region::<2>);
    schedule.add_system("region_3", steer_region::<3>);
    schedule.add_system("region_4", steer_region::<4>);
    schedule.add_system("region_5", steer_region::<5>);
    schedule.add_system("region_6", steer_region::<6>);
    schedule.add_system("region_7", steer_region::<7>);
    schedule.add_system("region_8", steer_region::<8>);
    schedule.add_system("region_9", steer_region::<9>);
    schedule.add_system("region_10", steer_region::<10>);
    schedule.add_system("region_11", steer_region::<11>);
    schedule.add_system("region_12", steer_region::<12>);
    schedule.add_system("region_13", steer_region::<13>);
    schedule.add_system("region_14", steer_region::<14>);
    schedule.add_system("region_15", steer_region::<15>);
    schedule.add_system("region_16", steer_region::<16>);
    schedule.add_system("region_17", steer_region::<17>);
    schedule.add_system("region_18", steer_region::<18>);
    schedule.add_system("region_19", steer_region::<19>);
    schedule.add_system("region_20", steer_region::<20>);
    schedule.add_system("region_21", steer_region::<21>);
    schedule.add_system("region_22", steer_region::<22>);
    schedule.add_system("region_23", steer_region::<23>);
    schedule.add_system("region_24", steer_region::<24>);
    schedule.add_system("region_25", steer_region::<25>);
    schedule.add_system("region_26", steer_region::<26>);
    schedule.add_system("region_27", steer_region::<27>);
    schedule.add_system("region_28", steer_region::<28>);
    schedule.add_system("region_29", steer_region::<29>);
    schedule.add_system("region_30", steer_region::<30>);
    schedule.add_system("region_31", steer_region::<31>);
    schedule.add_system("collect_0", collect_0);
    schedule.add_system("collect_1", collect_1);
    schedule.add_system("collect_2", collect_2);
    schedule.add_system("collect_3", collect_3);
    schedule.add_system("collect_4", collect_4);
    schedule.add_system("collect_5", collect_5);
    schedule.add_system("write_back", write_back);
    schedule
}

fn gather(
    mut builder: ResMut<GridBuilder>,
    mut applied: ResMut<Applied>,
    boids: Query<(&Position, &Velocity)>,
) {
    let started = Instant::now();
    let bounds = builder.bounds;
    let grid = Arc::get_mut(&mut builder.grid)
        .expect("every region drops its handle when it finishes, so this is the only one");

    if applied.handed_over {
        let handover = &mut *applied;
        grid.take_gather(&mut handover.next_position, &mut handover.next_velocity);
    } else {
        grid.start_gather();
        boids.for_each(|_entity, (position, velocity)| {
            grid.push(position.0, velocity.0);
        });
    }
    applied.handed_over = false;

    grid.rebuild(bounds + GRID_MARGIN, SEARCH_RADIUS);
    applied.gather_seconds += started.elapsed().as_secs_f64();
}

fn settings_from(integration: &Integration) -> RegionSettings {
    RegionSettings {
        delta_time: integration.delta_time,
        max_speed: integration.max_speed,
        bounds: integration.bounds,
        turn_strength: integration.turn_strength,
        regions: integration.regions,
    }
}

fn hand_0(
    builder: ResMut<GridBuilder>,
    integration: ResMut<Integration>,
    mut region_0: ResMut<Region<0>>,
    mut region_1: ResMut<Region<1>>,
    mut region_2: ResMut<Region<2>>,
    mut region_3: ResMut<Region<3>>,
    mut region_4: ResMut<Region<4>>,
) {
    let settings = settings_from(&integration);
    region_0.take(&builder.grid, settings);
    region_1.take(&builder.grid, settings);
    region_2.take(&builder.grid, settings);
    region_3.take(&builder.grid, settings);
    region_4.take(&builder.grid, settings);
}

fn hand_1(
    builder: ResMut<GridBuilder>,
    integration: ResMut<Integration>,
    mut region_5: ResMut<Region<5>>,
    mut region_6: ResMut<Region<6>>,
    mut region_7: ResMut<Region<7>>,
    mut region_8: ResMut<Region<8>>,
    mut region_9: ResMut<Region<9>>,
) {
    let settings = settings_from(&integration);
    region_5.take(&builder.grid, settings);
    region_6.take(&builder.grid, settings);
    region_7.take(&builder.grid, settings);
    region_8.take(&builder.grid, settings);
    region_9.take(&builder.grid, settings);
}

fn hand_2(
    builder: ResMut<GridBuilder>,
    integration: ResMut<Integration>,
    mut region_10: ResMut<Region<10>>,
    mut region_11: ResMut<Region<11>>,
    mut region_12: ResMut<Region<12>>,
    mut region_13: ResMut<Region<13>>,
    mut region_14: ResMut<Region<14>>,
) {
    let settings = settings_from(&integration);
    region_10.take(&builder.grid, settings);
    region_11.take(&builder.grid, settings);
    region_12.take(&builder.grid, settings);
    region_13.take(&builder.grid, settings);
    region_14.take(&builder.grid, settings);
}

fn hand_3(
    builder: ResMut<GridBuilder>,
    integration: ResMut<Integration>,
    mut region_15: ResMut<Region<15>>,
    mut region_16: ResMut<Region<16>>,
    mut region_17: ResMut<Region<17>>,
    mut region_18: ResMut<Region<18>>,
    mut region_19: ResMut<Region<19>>,
) {
    let settings = settings_from(&integration);
    region_15.take(&builder.grid, settings);
    region_16.take(&builder.grid, settings);
    region_17.take(&builder.grid, settings);
    region_18.take(&builder.grid, settings);
    region_19.take(&builder.grid, settings);
}

fn hand_4(
    builder: ResMut<GridBuilder>,
    integration: ResMut<Integration>,
    mut region_20: ResMut<Region<20>>,
    mut region_21: ResMut<Region<21>>,
    mut region_22: ResMut<Region<22>>,
    mut region_23: ResMut<Region<23>>,
    mut region_24: ResMut<Region<24>>,
) {
    let settings = settings_from(&integration);
    region_20.take(&builder.grid, settings);
    region_21.take(&builder.grid, settings);
    region_22.take(&builder.grid, settings);
    region_23.take(&builder.grid, settings);
    region_24.take(&builder.grid, settings);
}

fn hand_5(
    builder: ResMut<GridBuilder>,
    integration: ResMut<Integration>,
    mut region_25: ResMut<Region<25>>,
    mut region_26: ResMut<Region<26>>,
    mut region_27: ResMut<Region<27>>,
    mut region_28: ResMut<Region<28>>,
    mut region_29: ResMut<Region<29>>,
) {
    let settings = settings_from(&integration);
    region_25.take(&builder.grid, settings);
    region_26.take(&builder.grid, settings);
    region_27.take(&builder.grid, settings);
    region_28.take(&builder.grid, settings);
    region_29.take(&builder.grid, settings);
}

fn hand_6(
    builder: ResMut<GridBuilder>,
    integration: ResMut<Integration>,
    mut region_30: ResMut<Region<30>>,
    mut region_31: ResMut<Region<31>>,
) {
    let settings = settings_from(&integration);
    region_30.take(&builder.grid, settings);
    region_31.take(&builder.grid, settings);
}

fn steer_region<const INDEX: usize>(mut region: ResMut<Region<INDEX>>) {
    let grid = region.release();
    let settings = region.settings;
    let updates = &mut region.updates;
    updates.clear();

    if settings.delta_time <= 0.0 {
        return;
    }

    let slots = region_slots(grid.sorted_positions.len(), INDEX, settings.regions);
    updates.reserve(slots.len());

    for slot in slots {
        let position = grid.sorted_positions[slot];
        let mut velocity = grid.sorted_velocities[slot];

        let neighborhood = survey(&grid, position);
        let steer = neighborhood.separation() * SEPARATION_WEIGHT
            + neighborhood.alignment() * ALIGNMENT_WEIGHT
            + neighborhood.cohesion(position) * COHESION_WEIGHT
            + turn_from_walls(position, settings.bounds) * settings.turn_strength;

        velocity += steer * settings.delta_time;

        let speed = nalgebra_glm::length(&velocity);
        if speed > settings.max_speed {
            velocity *= settings.max_speed / speed;
        } else if speed > f32::EPSILON && speed < settings.max_speed * 0.4 {
            velocity *= settings.max_speed * 0.4 / speed;
        }

        updates.push(BoidUpdate {
            gathered: grid.entries[slot],
            position: position + velocity * settings.delta_time,
            velocity,
            rotation: crate::systems::instances::heading_rotation(velocity),
        });
    }
}

fn collect_0(
    mut applied: ResMut<Applied>,
    mut region_0: ResMut<Region<0>>,
    mut region_1: ResMut<Region<1>>,
    mut region_2: ResMut<Region<2>>,
    mut region_3: ResMut<Region<3>>,
    mut region_4: ResMut<Region<4>>,
    mut region_5: ResMut<Region<5>>,
) {
    let started = Instant::now();
    applied.scatter(0, &mut region_0.updates);
    applied.scatter(1, &mut region_1.updates);
    applied.scatter(2, &mut region_2.updates);
    applied.scatter(3, &mut region_3.updates);
    applied.scatter(4, &mut region_4.updates);
    applied.scatter(5, &mut region_5.updates);
    applied.collect_seconds += started.elapsed().as_secs_f64();
}

fn collect_1(
    mut applied: ResMut<Applied>,
    mut region_6: ResMut<Region<6>>,
    mut region_7: ResMut<Region<7>>,
    mut region_8: ResMut<Region<8>>,
    mut region_9: ResMut<Region<9>>,
    mut region_10: ResMut<Region<10>>,
    mut region_11: ResMut<Region<11>>,
) {
    let started = Instant::now();
    applied.scatter(6, &mut region_6.updates);
    applied.scatter(7, &mut region_7.updates);
    applied.scatter(8, &mut region_8.updates);
    applied.scatter(9, &mut region_9.updates);
    applied.scatter(10, &mut region_10.updates);
    applied.scatter(11, &mut region_11.updates);
    applied.collect_seconds += started.elapsed().as_secs_f64();
}

fn collect_2(
    mut applied: ResMut<Applied>,
    mut region_12: ResMut<Region<12>>,
    mut region_13: ResMut<Region<13>>,
    mut region_14: ResMut<Region<14>>,
    mut region_15: ResMut<Region<15>>,
    mut region_16: ResMut<Region<16>>,
    mut region_17: ResMut<Region<17>>,
) {
    let started = Instant::now();
    applied.scatter(12, &mut region_12.updates);
    applied.scatter(13, &mut region_13.updates);
    applied.scatter(14, &mut region_14.updates);
    applied.scatter(15, &mut region_15.updates);
    applied.scatter(16, &mut region_16.updates);
    applied.scatter(17, &mut region_17.updates);
    applied.collect_seconds += started.elapsed().as_secs_f64();
}

fn collect_3(
    mut applied: ResMut<Applied>,
    mut region_18: ResMut<Region<18>>,
    mut region_19: ResMut<Region<19>>,
    mut region_20: ResMut<Region<20>>,
    mut region_21: ResMut<Region<21>>,
    mut region_22: ResMut<Region<22>>,
    mut region_23: ResMut<Region<23>>,
) {
    let started = Instant::now();
    applied.scatter(18, &mut region_18.updates);
    applied.scatter(19, &mut region_19.updates);
    applied.scatter(20, &mut region_20.updates);
    applied.scatter(21, &mut region_21.updates);
    applied.scatter(22, &mut region_22.updates);
    applied.scatter(23, &mut region_23.updates);
    applied.collect_seconds += started.elapsed().as_secs_f64();
}

fn collect_4(
    mut applied: ResMut<Applied>,
    mut region_24: ResMut<Region<24>>,
    mut region_25: ResMut<Region<25>>,
    mut region_26: ResMut<Region<26>>,
    mut region_27: ResMut<Region<27>>,
    mut region_28: ResMut<Region<28>>,
    mut region_29: ResMut<Region<29>>,
) {
    let started = Instant::now();
    applied.scatter(24, &mut region_24.updates);
    applied.scatter(25, &mut region_25.updates);
    applied.scatter(26, &mut region_26.updates);
    applied.scatter(27, &mut region_27.updates);
    applied.scatter(28, &mut region_28.updates);
    applied.scatter(29, &mut region_29.updates);
    applied.collect_seconds += started.elapsed().as_secs_f64();
}

fn collect_5(
    mut applied: ResMut<Applied>,
    mut region_30: ResMut<Region<30>>,
    mut region_31: ResMut<Region<31>>,
) {
    let started = Instant::now();
    applied.scatter(30, &mut region_30.updates);
    applied.scatter(31, &mut region_31.updates);
    applied.collect_seconds += started.elapsed().as_secs_f64();
}

fn write_back(mut applied: ResMut<Applied>, boids: Query<(&mut Position, &mut Velocity)>) {
    if applied.position.is_empty() {
        return;
    }

    let started = Instant::now();
    let mut handed = std::mem::take(&mut applied.next_position);
    let mut handed_velocity = std::mem::take(&mut applied.next_velocity);
    handed.clear();
    handed_velocity.clear();

    let mut gathered = 0_usize;
    boids.for_each(|_entity, (position, velocity)| {
        if gathered < applied.position.len() {
            position.0 = applied.position[gathered];
            velocity.0 = applied.velocity[gathered];
        }
        handed.push(position.0);
        handed_velocity.push(velocity.0);
        gathered += 1;
    });

    applied.handed_over = gathered == applied.position.len();
    applied.next_position = handed;
    applied.next_velocity = handed_velocity;
    applied.write_seconds += started.elapsed().as_secs_f64();
}

#[derive(Default)]
pub struct Neighborhood {
    pushed: Vec3,
    heading: Vec3,
    middle: Vec3,
    found: f32,
}

impl Neighborhood {
    pub fn separation(&self) -> Vec3 {
        self.pushed
    }

    pub fn alignment(&self) -> Vec3 {
        if self.found == 0.0 {
            return Vec3::zeros();
        }
        self.heading / self.found
    }

    pub fn cohesion(&self, position: Vec3) -> Vec3 {
        if self.found == 0.0 {
            return Vec3::zeros();
        }
        self.middle / self.found - position
    }
}

pub fn survey(grid: &FlockGrid, position: Vec3) -> Neighborhood {
    let radius_squared = SEARCH_RADIUS * SEARCH_RADIUS;
    let mut found = Neighborhood::default();
    let mut visited = 0;

    grid.for_each_near(position, |slot| {
        let neighbor = grid.sorted_positions[slot];
        let offset = position - neighbor;
        let distance_squared = nalgebra_glm::length2(&offset);

        if distance_squared > f32::EPSILON && distance_squared < radius_squared {
            found.pushed += offset / distance_squared;
            found.heading += grid.sorted_velocities[slot];
            found.middle += neighbor;
            found.found += 1.0;
        }

        visited += 1;
        visited < CANDIDATE_CAP
    });

    found
}

pub fn turn_from_walls(position: Vec3, bounds: f32) -> Vec3 {
    let mut turn = Vec3::zeros();
    for axis in 0..3 {
        if position[axis] > bounds {
            turn[axis] = bounds - position[axis];
        } else if position[axis] < -bounds {
            turn[axis] = -bounds - position[axis];
        }
    }
    turn / bounds
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::REGION_COUNT;

    const TEST_BOUNDS: f32 = 100.0;

    fn grid_of(positions: &[Vec3], velocities: &[Vec3]) -> FlockGrid {
        let mut grid = FlockGrid::default();
        grid.start_gather();
        for (position, velocity) in positions.iter().zip(velocities) {
            grid.push(*position, *velocity);
        }
        grid.rebuild(TEST_BOUNDS + GRID_MARGIN, SEARCH_RADIUS);
        grid
    }

    #[test]
    fn separation_pushes_away_from_a_close_neighbor() {
        let grid = grid_of(
            &[Vec3::zeros(), Vec3::new(1.0, 0.0, 0.0)],
            &[Vec3::zeros(); 2],
        );

        let steer = survey(&grid, Vec3::zeros()).separation();

        assert!(steer.x < 0.0);
        assert_eq!(steer.y, 0.0);
    }

    #[test]
    fn separation_ignores_a_neighbor_outside_the_radius() {
        let grid = grid_of(
            &[Vec3::zeros(), Vec3::new(SEARCH_RADIUS * 3.0, 0.0, 0.0)],
            &[Vec3::zeros(); 2],
        );

        assert_eq!(survey(&grid, Vec3::zeros()).separation(), Vec3::zeros());
    }

    #[test]
    fn separation_ignores_the_boid_itself() {
        let grid = grid_of(&[Vec3::zeros()], &[Vec3::zeros()]);

        assert_eq!(survey(&grid, Vec3::zeros()).separation(), Vec3::zeros());
    }

    #[test]
    fn alignment_averages_the_headings_of_the_neighbors() {
        let grid = grid_of(
            &[
                Vec3::zeros(),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(2.0, 0.0, 0.0),
            ],
            &[
                Vec3::zeros(),
                Vec3::new(0.0, 4.0, 0.0),
                Vec3::new(0.0, 2.0, 0.0),
            ],
        );

        assert_eq!(
            survey(&grid, Vec3::zeros()).alignment(),
            Vec3::new(0.0, 3.0, 0.0)
        );
    }

    #[test]
    fn alignment_is_zero_without_neighbors() {
        let grid = grid_of(&[Vec3::zeros()], &[Vec3::zeros()]);

        assert_eq!(survey(&grid, Vec3::zeros()).alignment(), Vec3::zeros());
    }

    #[test]
    fn cohesion_points_at_the_middle_of_the_neighbors() {
        let grid = grid_of(
            &[
                Vec3::zeros(),
                Vec3::new(2.0, 0.0, 0.0),
                Vec3::new(4.0, 0.0, 0.0),
            ],
            &[Vec3::zeros(); 3],
        );

        assert_eq!(
            survey(&grid, Vec3::zeros()).cohesion(Vec3::zeros()),
            Vec3::new(3.0, 0.0, 0.0)
        );
    }

    #[test]
    fn the_walls_turn_a_boid_that_passed_them() {
        let turn = turn_from_walls(Vec3::new(12.0, 0.0, -12.0), 10.0);

        assert!(turn.x < 0.0);
        assert_eq!(turn.y, 0.0);
        assert!(turn.z > 0.0);
    }

    #[test]
    fn the_walls_leave_a_boid_inside_them_alone() {
        assert_eq!(
            turn_from_walls(Vec3::new(1.0, -2.0, 3.0), 10.0),
            Vec3::zeros()
        );
    }

    #[test]
    fn every_region_shares_one_batch() {
        let batches = schedule().batches();

        let widest = batches
            .iter()
            .max_by_key(|batch| batch.len())
            .expect("the schedule has systems");

        assert_eq!(widest.len(), REGION_COUNT);
    }

    #[test]
    fn the_work_either_side_of_the_regions_is_serial() {
        let batches = schedule().batches();

        for batch in &batches {
            assert!(batch.len() == 1 || batch.len() == REGION_COUNT);
        }
    }

    #[test]
    fn the_schedule_covers_every_system_once() {
        let schedule = schedule();
        let wanted = schedule.len();
        let batches = schedule.batches();

        let mut next = 0;
        for batch in &batches {
            assert_eq!(batch.start, next);
            next = batch.end;
        }
        assert_eq!(next, wanted);
    }
}
