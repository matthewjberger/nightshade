pub const BOIDS_GPU_PASS: &str = "boids_gpu_pass";

pub const CELL_CAPACITY: u32 = 32;

pub const WORKGROUP: u32 = 256;

#[cfg(not(target_family = "wasm"))]
pub const INITIAL_BOID_COUNT: usize = 1_000_000;

#[cfg(target_family = "wasm")]
pub const INITIAL_BOID_COUNT: usize = 100_000;

#[cfg(not(target_family = "wasm"))]
pub const MAX_BOID_COUNT: usize = 4_000_000;

#[cfg(target_family = "wasm")]
pub const MAX_BOID_COUNT: usize = 200_000;

#[cfg(not(target_family = "wasm"))]
pub const BOID_STEP: usize = 250_000;

#[cfg(target_family = "wasm")]
pub const BOID_STEP: usize = 25_000;

pub const MIN_BOID_COUNT: usize = BOID_STEP;
