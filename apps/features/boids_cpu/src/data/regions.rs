use crate::data::FlockGrid;
use nalgebra_glm::{Quat, Vec3};
use std::ops::Range;
use std::sync::Arc;

pub const REGION_COUNT: usize = 32;

pub fn regions_for(threads: usize) -> usize {
    threads.saturating_mul(2).clamp(1, REGION_COUNT)
}

pub fn region_slots(count: usize, index: usize, regions: usize) -> Range<usize> {
    if regions == 0 || index >= regions {
        return 0..0;
    }

    let start = count * index / regions;
    let end = count * (index + 1) / regions;
    start..end
}

#[derive(Clone, Copy)]
pub struct RegionSettings {
    pub delta_time: f32,
    pub max_speed: f32,
    pub bounds: f32,
    pub turn_strength: f32,
    pub regions: usize,
}

impl Default for RegionSettings {
    fn default() -> Self {
        Self {
            delta_time: 0.0,
            max_speed: 0.0,
            bounds: 0.0,
            turn_strength: 0.0,
            regions: 1,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct BoidUpdate {
    pub gathered: u32,
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Quat,
}

pub struct Region<const INDEX: usize> {
    pub grid: Arc<FlockGrid>,
    pub settings: RegionSettings,
    pub updates: Vec<BoidUpdate>,
}

impl<const INDEX: usize> Region<INDEX> {
    pub fn take(&mut self, grid: &Arc<FlockGrid>, settings: RegionSettings) {
        self.grid = Arc::clone(grid);
        self.settings = settings;
    }

    pub fn release(&mut self) -> Arc<FlockGrid> {
        std::mem::take(&mut self.grid)
    }
}

impl<const INDEX: usize> Default for Region<INDEX> {
    fn default() -> Self {
        Self {
            grid: Arc::default(),
            settings: RegionSettings::default(),
            updates: Vec::new(),
        }
    }
}

#[derive(Default)]
pub struct Applied {
    pub position: Vec<Vec3>,
    pub velocity: Vec<Vec3>,
    pub rotation: Vec<Quat>,
    pub region_counts: [usize; REGION_COUNT],
    pub next_position: Vec<Vec3>,
    pub next_velocity: Vec<Vec3>,
    pub handed_over: bool,
    pub gather_seconds: f64,
    pub collect_seconds: f64,
    pub write_seconds: f64,
}

impl Applied {
    pub fn scatter(&mut self, region: usize, updates: &mut Vec<BoidUpdate>) {
        self.region_counts[region] = updates.len();

        let highest = updates.iter().map(|update| update.gathered).max();
        if let Some(highest) = highest {
            let needed = highest as usize + 1;
            if needed > self.position.len() {
                self.position.resize(needed, Vec3::zeros());
                self.velocity.resize(needed, Vec3::zeros());
                self.rotation.resize(needed, Quat::identity());
            }
        }

        for update in updates.drain(..) {
            let gathered = update.gathered as usize;
            self.position[gathered] = update.position;
            self.velocity[gathered] = update.velocity;
            self.rotation[gathered] = update.rotation;
        }
    }

    pub fn count(&self) -> usize {
        self.region_counts.iter().sum()
    }

    pub fn regions_used(&self) -> usize {
        self.region_counts
            .iter()
            .filter(|&&count| count > 0)
            .count()
            .max(1)
    }

    pub fn imbalance(&self) -> f32 {
        let total: usize = self.region_counts.iter().sum();
        if total == 0 {
            return 1.0;
        }
        let busiest = self.region_counts.iter().copied().max().unwrap_or(0);
        busiest as f32 * self.regions_used() as f32 / total as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_regions_cover_the_flock_exactly() {
        let count = 1000;
        let mut covered = vec![0_u32; count];

        for index in 0..8 {
            for slot in region_slots(count, index, 8) {
                covered[slot] += 1;
            }
        }

        assert!(covered.iter().all(|&times| times == 1));
    }

    #[test]
    fn every_region_gets_the_same_share_to_within_one() {
        let count = 1003;
        let regions = 8;

        let sizes: Vec<usize> = (0..regions)
            .map(|index| region_slots(count, index, regions).len())
            .collect();

        let smallest = sizes.iter().min().copied().unwrap();
        let largest = sizes.iter().max().copied().unwrap();
        assert!(largest - smallest <= 1);
        assert_eq!(sizes.iter().sum::<usize>(), count);
    }

    #[test]
    fn a_region_past_the_ones_in_use_owns_nothing() {
        assert!(region_slots(1000, 9, 8).is_empty());
    }

    #[test]
    fn an_empty_flock_gives_every_region_nothing() {
        for index in 0..8 {
            assert!(region_slots(0, index, 8).is_empty());
        }
    }

    #[test]
    fn a_pool_gets_two_regions_a_thread() {
        assert_eq!(regions_for(8), 16);
        assert_eq!(regions_for(1), 2);
    }

    #[test]
    fn the_count_never_passes_what_was_compiled() {
        assert_eq!(regions_for(1024), REGION_COUNT);
    }

    #[test]
    fn an_even_split_is_not_lopsided() {
        let applied = Applied {
            region_counts: [100; REGION_COUNT],
            ..Default::default()
        };

        assert!((applied.imbalance() - 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn one_region_holding_everything_is_not_lopsided_on_its_own() {
        let mut region_counts = [0; REGION_COUNT];
        region_counts[3] = 800;
        let applied = Applied {
            region_counts,
            ..Default::default()
        };

        assert_eq!(applied.regions_used(), 1);
        assert!((applied.imbalance() - 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn one_region_of_many_holding_everything_is_as_lopsided_as_it_gets() {
        let mut region_counts = [1; REGION_COUNT];
        region_counts[3] = 1_000_000;
        let applied = Applied {
            region_counts,
            ..Default::default()
        };

        assert_eq!(applied.regions_used(), REGION_COUNT);
        assert!(applied.imbalance() > REGION_COUNT as f32 * 0.9);
    }

    #[test]
    fn a_partly_used_pool_still_reads_as_even() {
        let mut region_counts = [0; REGION_COUNT];
        for count in region_counts.iter_mut().take(8) {
            *count = 100;
        }
        let applied = Applied {
            region_counts,
            ..Default::default()
        };

        assert_eq!(applied.regions_used(), 8);
        assert!((applied.imbalance() - 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn an_empty_frame_reports_no_lopsidedness() {
        assert!((Applied::default().imbalance() - 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn a_scatter_puts_each_result_at_its_own_boid() {
        let mut applied = Applied::default();
        let mut updates = vec![
            BoidUpdate {
                gathered: 2,
                position: Vec3::new(1.0, 0.0, 0.0),
                velocity: Vec3::zeros(),
                rotation: Quat::identity(),
            },
            BoidUpdate {
                gathered: 0,
                position: Vec3::new(3.0, 0.0, 0.0),
                velocity: Vec3::zeros(),
                rotation: Quat::identity(),
            },
        ];

        applied.scatter(0, &mut updates);

        assert_eq!(applied.position.len(), 3);
        assert_eq!(applied.position[2].x, 1.0);
        assert_eq!(applied.position[0].x, 3.0);
        assert_eq!(applied.region_counts[0], 2);
        assert!(updates.is_empty());
    }
}
