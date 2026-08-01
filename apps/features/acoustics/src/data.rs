use nightshade::prelude::Vec3;
use nightshade_acoustics::contract::{CLOTH, GLASS, STONE, TIMBER};

pub const CLIP: &str = "room-tone";
pub const RAIN: &str = "rain";
pub const RAIN_REACH: f32 = 7.0;
#[cfg(not(target_arch = "wasm32"))]
pub const RAIN_QUIETEST: f32 = -55.0;
#[cfg(not(target_arch = "wasm32"))]
pub const RAIN_LOUDEST: f32 = -4.0;
#[cfg(not(target_arch = "wasm32"))]
pub const ROOM_TONE: &[u8] = include_bytes!("../assets/room-tone.ogg");

pub struct Slab {
    pub low: Vec3,
    pub high: Vec3,
    pub material: u8,
    pub color: [f32; 4],
}

const SHELL: [f32; 4] = [0.66, 0.64, 0.60, 1.0];
const INNER: [f32; 4] = [0.52, 0.53, 0.56, 1.0];
const LID: [f32; 4] = [0.44, 0.44, 0.46, 1.0];
const GROUND: [f32; 4] = [0.32, 0.31, 0.29, 1.0];
const DOOR: [f32; 4] = [0.54, 0.33, 0.16, 1.0];
const CRATE: [f32; 4] = [0.46, 0.36, 0.24, 1.0];
const PANE: [f32; 4] = [0.62, 0.78, 0.86, 1.0];

pub const GRID_ORIGIN: Vec3 = Vec3::new(-16.0, -3.0, -16.0);
pub const GRID_CELL: f32 = 0.5;
pub const GRID_SIDE: u32 = 64;

pub const SOURCE_PLACE: Vec3 = Vec3::new(-7.0, 2.0, -5.0);
pub const CAMERA_PLACE: Vec3 = Vec3::new(4.0, 1.6, -5.0);
pub const CAMERA_YAW: f32 = 1.57;
pub const CAMERA_PITCH: f32 = -0.04;
pub const EMITTER_REACH: f32 = 34.0;

pub const HEAD_SIZE: Vec3 = Vec3::new(0.44, 0.44, 0.44);
pub const CHEST_SIZE: Vec3 = Vec3::new(0.52, 1.05, 0.52);
pub const CHEST_DROP: f32 = 0.75;
pub const VIEW_BACK: f32 = 4.4;
pub const VIEW_LIFT: f32 = 1.0;
pub const GLASS_ALPHA: f32 = 0.24;

pub const MARKER_SMALLEST: f32 = 0.06;
pub const MARKER_LARGEST: f32 = 0.26;

pub const RAY_SPEED: f32 = 9.0;
pub const RAY_TRAIL: f32 = 3.0;

pub const DOOR_LEAF: Slab = Slab {
    low: Vec3::new(-1.0, 0.0, -6.0),
    high: Vec3::new(1.0, 2.6, -4.0),
    material: TIMBER,
    color: DOOR,
};

pub const BUILDING: [Slab; 19] = [
    Slab {
        low: Vec3::new(-14.0, -1.0, -11.0),
        high: Vec3::new(14.0, 0.0, 5.0),
        material: STONE,
        color: GROUND,
    },
    Slab {
        low: Vec3::new(-14.0, 0.0, -11.0),
        high: Vec3::new(-13.0, 10.0, 5.0),
        material: STONE,
        color: SHELL,
    },
    Slab {
        low: Vec3::new(13.0, 0.0, -11.0),
        high: Vec3::new(14.0, 10.0, -3.0),
        material: STONE,
        color: SHELL,
    },
    Slab {
        low: Vec3::new(13.0, 0.0, -1.0),
        high: Vec3::new(14.0, 10.0, 5.0),
        material: STONE,
        color: SHELL,
    },
    Slab {
        low: Vec3::new(13.0, 0.0, -3.0),
        high: Vec3::new(14.0, 1.0, -1.0),
        material: STONE,
        color: SHELL,
    },
    Slab {
        low: Vec3::new(13.0, 3.0, -3.0),
        high: Vec3::new(14.0, 10.0, -1.0),
        material: STONE,
        color: SHELL,
    },
    Slab {
        low: Vec3::new(13.0, 1.0, -3.0),
        high: Vec3::new(14.0, 3.0, -1.0),
        material: GLASS,
        color: PANE,
    },
    Slab {
        low: Vec3::new(-14.0, 0.0, -11.0),
        high: Vec3::new(14.0, 10.0, -10.0),
        material: STONE,
        color: SHELL,
    },
    Slab {
        low: Vec3::new(-14.0, 0.0, 4.0),
        high: Vec3::new(-7.0, 10.0, 5.0),
        material: STONE,
        color: SHELL,
    },
    Slab {
        low: Vec3::new(-3.0, 0.0, 4.0),
        high: Vec3::new(14.0, 10.0, 5.0),
        material: STONE,
        color: SHELL,
    },
    Slab {
        low: Vec3::new(-13.0, 9.0, -10.0),
        high: Vec3::new(-1.0, 10.0, -1.0),
        material: STONE,
        color: LID,
    },
    Slab {
        low: Vec3::new(-1.0, 0.0, -10.0),
        high: Vec3::new(1.0, 10.0, -6.0),
        material: STONE,
        color: INNER,
    },
    Slab {
        low: Vec3::new(-1.0, 0.0, -4.0),
        high: Vec3::new(1.0, 10.0, 4.0),
        material: STONE,
        color: INNER,
    },
    Slab {
        low: Vec3::new(-1.0, 2.6, -6.0),
        high: Vec3::new(1.0, 10.0, -4.0),
        material: STONE,
        color: INNER,
    },
    Slab {
        low: Vec3::new(-13.0, 0.0, -1.0),
        high: Vec3::new(-11.0, 10.0, 1.0),
        material: STONE,
        color: INNER,
    },
    Slab {
        low: Vec3::new(-9.0, 0.0, -1.0),
        high: Vec3::new(-1.0, 10.0, 1.0),
        material: STONE,
        color: INNER,
    },
    Slab {
        low: Vec3::new(-11.0, 2.6, -1.0),
        high: Vec3::new(-9.0, 10.0, 1.0),
        material: STONE,
        color: INNER,
    },
    Slab {
        low: Vec3::new(-13.0, 2.6, 1.0),
        high: Vec3::new(-1.0, 3.6, 4.0),
        material: STONE,
        color: LID,
    },
    Slab {
        low: Vec3::new(1.0, 2.6, -10.0),
        high: Vec3::new(13.0, 3.6, 4.0),
        material: STONE,
        color: LID,
    },
];

pub const CLUTTER: [Slab; 5] = [
    Slab {
        low: Vec3::new(2.0, 0.0, -9.0),
        high: Vec3::new(5.0, 2.4, -6.5),
        material: CLOTH,
        color: CRATE,
    },
    Slab {
        low: Vec3::new(7.0, 0.0, -9.5),
        high: Vec3::new(10.0, 2.2, -7.0),
        material: CLOTH,
        color: CRATE,
    },
    Slab {
        low: Vec3::new(10.0, 0.0, -4.5),
        high: Vec3::new(12.5, 2.4, -2.0),
        material: CLOTH,
        color: CRATE,
    },
    Slab {
        low: Vec3::new(1.5, 0.0, 2.0),
        high: Vec3::new(3.5, 2.2, 4.0),
        material: CLOTH,
        color: CRATE,
    },
    Slab {
        low: Vec3::new(9.5, 0.0, 0.5),
        high: Vec3::new(12.0, 2.4, 3.0),
        material: CLOTH,
        color: CRATE,
    },
];
