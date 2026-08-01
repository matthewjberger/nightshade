use crate::data::{fill_by_index, fill_runs, sort};
use nalgebra_glm::Vec3;

#[derive(Default)]
pub struct FlockGrid {
    positions: Vec<Vec3>,
    velocities: Vec<Vec3>,
    pub sorted_positions: Vec<Vec3>,
    pub sorted_velocities: Vec<Vec3>,
    pub entries: Vec<u32>,
    pub cell_start: Vec<u32>,
    keys: Vec<u64>,
    pub cell_size: f32,
    pub dimension: usize,
    pub origin: f32,
}

impl FlockGrid {
    pub fn start_gather(&mut self) {
        self.positions.clear();
        self.velocities.clear();
    }

    pub fn take_gather(&mut self, positions: &mut Vec<Vec3>, velocities: &mut Vec<Vec3>) {
        std::mem::swap(&mut self.positions, positions);
        std::mem::swap(&mut self.velocities, velocities);
    }

    pub fn push(&mut self, position: Vec3, velocity: Vec3) {
        self.positions.push(position);
        self.velocities.push(velocity);
    }

    pub fn rebuild(&mut self, bounds: f32, cell_size: f32) {
        let span = bounds * 2.0;
        self.cell_size = cell_size;
        self.origin = -bounds;
        self.dimension = ((span / cell_size).ceil() as usize).max(1);

        let cells = self.dimension * self.dimension * self.dimension;
        let count = self.positions.len();

        let mut keys = std::mem::take(&mut self.keys);
        keys.clear();
        keys.resize(count, 0);
        fill_by_index(&mut keys, |index| {
            ((self.cell_of(self.positions[index]) as u64) << 32) | index as u64
        });
        sort(&mut keys);

        let mut entries = std::mem::take(&mut self.entries);
        entries.clear();
        entries.resize(count, 0);
        fill_by_index(&mut entries, |slot| (keys[slot] & 0xffff_ffff) as u32);

        let mut sorted_positions = std::mem::take(&mut self.sorted_positions);
        sorted_positions.clear();
        sorted_positions.resize(count, Vec3::zeros());
        fill_by_index(&mut sorted_positions, |slot| {
            self.positions[entries[slot] as usize]
        });

        let mut sorted_velocities = std::mem::take(&mut self.sorted_velocities);
        sorted_velocities.clear();
        sorted_velocities.resize(count, Vec3::zeros());
        fill_by_index(&mut sorted_velocities, |slot| {
            self.velocities[entries[slot] as usize]
        });

        let mut cell_start = std::mem::take(&mut self.cell_start);
        cell_start.clear();
        cell_start.resize(cells + 1, 0);
        fill_runs(&mut cell_start, |base, run| {
            let mut slot = keys.partition_point(|key| (key >> 32) < base as u64);
            for (offset, start) in run.iter_mut().enumerate() {
                let cell = (base + offset) as u64;
                while slot < keys.len() && (keys[slot] >> 32) < cell {
                    slot += 1;
                }
                *start = slot as u32;
            }
        });

        self.keys = keys;
        self.entries = entries;
        self.sorted_positions = sorted_positions;
        self.sorted_velocities = sorted_velocities;
        self.cell_start = cell_start;
    }

    pub fn cell_of(&self, position: Vec3) -> usize {
        let axis = |value: f32| {
            let cell = ((value - self.origin) / self.cell_size) as isize;
            cell.clamp(0, self.dimension as isize - 1) as usize
        };
        self.cell_index(axis(position.x), axis(position.y), axis(position.z))
    }

    pub fn cell_index(&self, x: usize, y: usize, z: usize) -> usize {
        (z * self.dimension + y) * self.dimension + x
    }

    pub fn for_each_near(&self, position: Vec3, mut visit: impl FnMut(usize) -> bool) {
        if self.dimension == 0 {
            return;
        }

        let axis = |value: f32| {
            let cell = ((value - self.origin) / self.cell_size) as isize;
            cell.clamp(0, self.dimension as isize - 1)
        };
        let (center_x, center_y, center_z) = (axis(position.x), axis(position.y), axis(position.z));
        let last = self.dimension as isize - 1;

        for offset_z in -1..=1 {
            let z = center_z + offset_z;
            if z < 0 || z > last {
                continue;
            }
            for offset_y in -1..=1 {
                let y = center_y + offset_y;
                if y < 0 || y > last {
                    continue;
                }
                for offset_x in -1..=1 {
                    let x = center_x + offset_x;
                    if x < 0 || x > last {
                        continue;
                    }
                    let cell = self.cell_index(x as usize, y as usize, z as usize);
                    let run = self.cell_start[cell] as usize..self.cell_start[cell + 1] as usize;
                    for slot in run {
                        if !visit(slot) {
                            return;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid_of(positions: &[Vec3], bounds: f32, cell_size: f32) -> FlockGrid {
        let mut grid = FlockGrid::default();
        grid.start_gather();
        for (index, position) in positions.iter().enumerate() {
            grid.push(*position, Vec3::new(index as f32, 0.0, 0.0));
        }
        grid.rebuild(bounds, cell_size);
        grid
    }

    #[test]
    fn every_boid_lands_in_exactly_one_cell() {
        let positions = [
            Vec3::new(-9.0, -9.0, -9.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(9.0, 9.0, 9.0),
        ];

        let grid = grid_of(&positions, 10.0, 5.0);

        assert_eq!(grid.sorted_positions.len(), 3);
        for position in positions {
            assert!(grid.sorted_positions.contains(&position));
        }
    }

    #[test]
    fn the_sort_keeps_a_boid_with_its_own_velocity() {
        let positions: Vec<Vec3> = (0..32)
            .map(|index| Vec3::new(index as f32 * 0.6 - 9.0, 0.0, 0.0))
            .collect();

        let grid = grid_of(&positions, 10.0, 5.0);

        for (slot, position) in grid.sorted_positions.iter().enumerate() {
            let original = positions.iter().position(|item| item == position).unwrap();
            assert_eq!(grid.sorted_velocities[slot].x, original as f32);
        }
    }

    #[test]
    fn the_cell_runs_cover_every_entry() {
        let positions: Vec<Vec3> = (0..64)
            .map(|index| Vec3::new(index as f32 * 0.3 - 9.0, 0.0, 0.0))
            .collect();

        let grid = grid_of(&positions, 10.0, 5.0);

        let cells = grid.dimension * grid.dimension * grid.dimension;
        assert_eq!(grid.cell_start[0], 0);
        assert_eq!(grid.cell_start[cells] as usize, positions.len());
    }

    #[test]
    fn a_cell_run_holds_only_that_cell_s_boids() {
        let positions: Vec<Vec3> = (0..64)
            .map(|index| Vec3::new(index as f32 * 0.3 - 9.0, 0.0, 0.0))
            .collect();

        let grid = grid_of(&positions, 10.0, 5.0);

        let cells = grid.dimension * grid.dimension * grid.dimension;
        for cell in 0..cells {
            let run = grid.cell_start[cell] as usize..grid.cell_start[cell + 1] as usize;
            for slot in run {
                assert_eq!(grid.cell_of(grid.sorted_positions[slot]), cell);
            }
        }
    }

    #[test]
    fn a_near_scan_finds_a_neighbor_in_the_next_cell() {
        let positions = [Vec3::new(-0.1, 0.0, 0.0), Vec3::new(0.1, 0.0, 0.0)];
        let grid = grid_of(&positions, 10.0, 5.0);

        let mut found = Vec::new();
        grid.for_each_near(positions[0], |slot| {
            found.push(grid.sorted_positions[slot]);
            true
        });

        assert!(found.contains(&positions[1]));
    }

    #[test]
    fn a_near_scan_skips_a_boid_three_cells_away() {
        let positions = [Vec3::new(-9.0, 0.0, 0.0), Vec3::new(9.0, 0.0, 0.0)];
        let grid = grid_of(&positions, 10.0, 2.0);

        let mut found = Vec::new();
        grid.for_each_near(positions[0], |slot| {
            found.push(grid.sorted_positions[slot]);
            true
        });

        assert!(!found.contains(&positions[1]));
    }

    #[test]
    fn a_scan_stops_when_the_visitor_says_so() {
        let positions: Vec<Vec3> = (0..32).map(|_| Vec3::zeros()).collect();
        let grid = grid_of(&positions, 10.0, 5.0);

        let mut seen = 0;
        grid.for_each_near(Vec3::zeros(), |_slot| {
            seen += 1;
            seen < 4
        });

        assert_eq!(seen, 4);
    }

    #[test]
    fn a_position_outside_the_bounds_still_resolves_to_a_cell() {
        let positions = [Vec3::new(500.0, -500.0, 500.0)];

        let grid = grid_of(&positions, 10.0, 5.0);

        assert_eq!(grid.sorted_positions.len(), 1);
    }

    #[test]
    fn a_slot_reports_the_boid_it_came_from() {
        let positions: Vec<Vec3> = (0..32)
            .map(|index| Vec3::new(index as f32 * 0.6 - 9.0, 0.0, 0.0))
            .collect();

        let grid = grid_of(&positions, 10.0, 5.0);

        for (slot, position) in grid.sorted_positions.iter().enumerate() {
            let gathered = grid.entries[slot] as usize;
            assert_eq!(positions[gathered], *position);
        }
    }

    #[test]
    fn every_boid_is_named_by_exactly_one_slot() {
        let positions: Vec<Vec3> = (0..64)
            .map(|index| Vec3::new(index as f32 * 0.3 - 9.0, 0.0, 0.0))
            .collect();

        let grid = grid_of(&positions, 10.0, 5.0);

        let mut seen = grid.entries.clone();
        seen.sort_unstable();
        assert_eq!(seen, (0..positions.len() as u32).collect::<Vec<_>>());
    }

    #[test]
    fn a_cell_run_holds_every_boid_of_that_cell_and_no_other() {
        let positions: Vec<Vec3> = (0..64)
            .map(|index| Vec3::new(index as f32 * 0.3 - 9.0, 0.0, 0.0))
            .collect();

        let grid = grid_of(&positions, 10.0, 5.0);

        for cell in 0..grid.cell_start.len() - 1 {
            let run = grid.cell_start[cell] as usize..grid.cell_start[cell + 1] as usize;
            for slot in run {
                assert_eq!(grid.cell_of(grid.sorted_positions[slot]), cell);
            }
        }
    }

    #[test]
    fn a_rebuild_reuses_its_buffers() {
        let positions: Vec<Vec3> = (0..64)
            .map(|index| Vec3::new(index as f32 * 0.3 - 9.0, 0.0, 0.0))
            .collect();
        let mut grid = grid_of(&positions, 10.0, 5.0);

        let capacity = grid.sorted_positions.capacity();
        grid.rebuild(10.0, 5.0);

        assert_eq!(grid.sorted_positions.capacity(), capacity);
        assert_eq!(grid.sorted_positions.len(), positions.len());
    }
}
