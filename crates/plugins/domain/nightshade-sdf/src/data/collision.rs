pub const CHUNK_CELLS: u32 = 16;

pub const CHUNK_SAMPLES: u32 = CHUNK_CELLS + 2;

pub const CHUNK_POINTS: usize = (CHUNK_SAMPLES * CHUNK_SAMPLES * CHUNK_SAMPLES) as usize;

pub const COLLISION_CELL: f32 = 0.15;

pub const COLLISION_CREW: usize = 4;

pub const COLLISION_TAKEN: usize = 256;
