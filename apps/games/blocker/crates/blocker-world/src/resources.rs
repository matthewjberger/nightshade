use crate::data::world::nodes::WorkKind;
use blocker_blocks::resources::ColumnKey;
use noise::Perlin;
use std::collections::HashMap;

pub struct Digging;

#[derive(Clone, Copy, Default)]
pub struct Seat {
    pub city: [i32; 2],
    pub barrow: [i32; 2],
    pub camp: [i32; 2],
}

pub struct Bench {
    pub seed: u32,
    pub temperature: Perlin,
    pub moisture: Perlin,
    pub elevation: Perlin,
    pub hills: Perlin,
    pub seats: [Seat; crate::data::world::places::CAPITALS.len()],
}

impl Bench {
    pub fn reseed(&mut self, seed: u32) {
        self.seed = seed;
        self.temperature = Perlin::new(seed.wrapping_add(17));
        self.moisture = Perlin::new(seed.wrapping_add(41));
        self.elevation = Perlin::new(seed.wrapping_add(3));
        self.hills = Perlin::new(seed.wrapping_add(59));
        self.seats = crate::queries::settle_the_seats(self);
    }
}

impl Default for Bench {
    fn default() -> Self {
        let mut bench = Self {
            seed: crate::data::world::SEED,
            temperature: Perlin::new(0),
            moisture: Perlin::new(0),
            elevation: Perlin::new(0),
            hills: Perlin::new(0),
            seats: [Seat::default(); crate::data::world::places::CAPITALS.len()],
        };
        bench.reseed(crate::data::world::SEED);
        bench
    }
}

pub struct ColumnReport {
    pub signs: Vec<([i32; 3], String)>,
    pub plaques: Vec<([i32; 3], String)>,
    pub homes: Vec<([i32; 2], [i32; 3])>,
    pub guides: Vec<([i32; 2], [i32; 3])>,
    pub posts: Vec<([i32; 2], [i32; 3], WorkKind)>,
}

pub struct Draft {
    pub seed: u32,
    pub enabled: u32,
    pub asked: bool,
}

impl Default for Draft {
    fn default() -> Self {
        Self {
            seed: crate::queries::fresh_seed(),
            enabled: crate::data::catalog::ALL_CATEGORIES,
            asked: false,
        }
    }
}

pub struct Generation {
    pub seed: u32,
    pub enabled: u32,
    pub spawn: Option<(i32, i32)>,
    pub signs: HashMap<[i32; 3], String>,
    pub plaques: HashMap<[i32; 3], String>,
    pub homes: HashMap<[i32; 2], Vec<[i32; 3]>>,
    pub guides: HashMap<[i32; 2], [i32; 3]>,
    pub posts: HashMap<[i32; 2], Vec<([i32; 3], WorkKind)>>,
}

impl Default for Generation {
    fn default() -> Self {
        Self {
            seed: crate::data::world::SEED,
            enabled: crate::data::catalog::ALL_CATEGORIES,
            spawn: None,
            signs: HashMap::new(),
            plaques: HashMap::new(),
            homes: HashMap::new(),
            guides: HashMap::new(),
            posts: HashMap::new(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct WorldOrder {
    pub seed: u32,
    pub enabled: u32,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Arrival {
    pub landed: usize,
    pub wanted: usize,
    pub stood: bool,
}

#[derive(Default)]
pub struct Effects {
    pub synced_edits: Option<u64>,
    pub center: Option<ColumnKey>,
}
