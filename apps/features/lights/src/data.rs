use nightshade::prelude::*;

pub const NUM_LIGHTS: usize = 1024;
pub const ARENA_SIZE: f32 = 100.0;
pub const NUM_PILLARS_PER_SIDE: usize = 8;

pub const COLORS: [Vec3; 8] = [
    Vec3::new(1.0, 0.3, 0.1),
    Vec3::new(0.1, 1.0, 0.3),
    Vec3::new(0.3, 0.1, 1.0),
    Vec3::new(1.0, 1.0, 0.1),
    Vec3::new(0.1, 1.0, 1.0),
    Vec3::new(1.0, 0.1, 1.0),
    Vec3::new(1.0, 0.5, 0.0),
    Vec3::new(0.0, 0.5, 1.0),
];
