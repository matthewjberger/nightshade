use crate::components::PickupKind;
use crate::data::shape::Part;
use nightshade::prelude::Vec3;

const WHITE: [f32; 4] = [0.92, 0.93, 0.95, 1.0];
const RED: [f32; 4] = [0.74, 0.13, 0.13, 1.0];
const GREEN: [f32; 4] = [0.18, 0.42, 0.24, 1.0];
const OLIVE: [f32; 4] = [0.34, 0.35, 0.20, 1.0];
const BRASS: [f32; 4] = [0.62, 0.48, 0.22, 1.0];
const STEEL: [f32; 4] = [0.32, 0.33, 0.37, 1.0];
const IRON: [f32; 4] = [0.18, 0.19, 0.22, 1.0];
const GLASS: [f32; 4] = [0.36, 0.78, 0.92, 1.0];
const YELLOW: [f32; 4] = [0.88, 0.70, 0.16, 1.0];
const CRIMSON: [f32; 4] = [0.92, 0.24, 0.22, 1.0];
const AZURE: [f32; 4] = [0.34, 0.56, 0.98, 1.0];

const DARK: [f32; 3] = [0.0, 0.0, 0.0];
const SERUM: [f32; 3] = [0.24, 1.05, 1.25];
const CORE: [f32; 3] = [0.22, 1.45, 1.95];
const WARHEAD: [f32; 3] = [0.55, 0.34, 0.06];
const RED_CARD: [f32; 3] = [0.85, 0.16, 0.14];
const BLUE_CARD: [f32; 3] = [0.20, 0.42, 0.95];

static MEDKIT: [Part; 3] = [
    Part {
        at: Vec3::new(0.0, 0.0, 0.0),
        size: Vec3::new(0.72, 0.46, 0.52),
        tint: WHITE,
        glow: DARK,
    },
    Part {
        at: Vec3::new(0.0, 0.245, 0.0),
        size: Vec3::new(0.44, 0.06, 0.14),
        tint: RED,
        glow: DARK,
    },
    Part {
        at: Vec3::new(0.0, 0.245, 0.0),
        size: Vec3::new(0.14, 0.06, 0.32),
        tint: RED,
        glow: DARK,
    },
];

static STIMPACK: [Part; 3] = [
    Part {
        at: Vec3::new(0.0, 0.0, 0.0),
        size: Vec3::new(0.20, 0.52, 0.20),
        tint: GLASS,
        glow: SERUM,
    },
    Part {
        at: Vec3::new(0.0, 0.30, 0.0),
        size: Vec3::new(0.26, 0.12, 0.26),
        tint: STEEL,
        glow: DARK,
    },
    Part {
        at: Vec3::new(0.0, -0.30, 0.0),
        size: Vec3::new(0.26, 0.12, 0.26),
        tint: STEEL,
        glow: DARK,
    },
];

static PLATE: [Part; 3] = [
    Part {
        at: Vec3::new(0.0, 0.0, 0.0),
        size: Vec3::new(0.78, 0.68, 0.16),
        tint: GREEN,
        glow: DARK,
    },
    Part {
        at: Vec3::new(0.0, 0.19, 0.02),
        size: Vec3::new(0.80, 0.10, 0.18),
        tint: STEEL,
        glow: DARK,
    },
    Part {
        at: Vec3::new(0.0, -0.19, 0.02),
        size: Vec3::new(0.80, 0.10, 0.18),
        tint: STEEL,
        glow: DARK,
    },
];

static SHELLS: [Part; 4] = [
    Part {
        at: Vec3::new(0.0, 0.0, 0.0),
        size: Vec3::new(0.66, 0.34, 0.44),
        tint: RED,
        glow: DARK,
    },
    Part {
        at: Vec3::new(0.0, 0.19, 0.0),
        size: Vec3::new(0.68, 0.08, 0.46),
        tint: BRASS,
        glow: DARK,
    },
    Part {
        at: Vec3::new(-0.17, 0.27, 0.0),
        size: Vec3::new(0.11, 0.12, 0.11),
        tint: BRASS,
        glow: DARK,
    },
    Part {
        at: Vec3::new(0.17, 0.27, 0.0),
        size: Vec3::new(0.11, 0.12, 0.11),
        tint: BRASS,
        glow: DARK,
    },
];

static BULLETS: [Part; 3] = [
    Part {
        at: Vec3::new(0.0, 0.0, 0.0),
        size: Vec3::new(0.62, 0.32, 0.40),
        tint: OLIVE,
        glow: DARK,
    },
    Part {
        at: Vec3::new(0.0, 0.18, 0.0),
        size: Vec3::new(0.64, 0.08, 0.42),
        tint: IRON,
        glow: DARK,
    },
    Part {
        at: Vec3::new(0.0, 0.26, 0.0),
        size: Vec3::new(0.46, 0.09, 0.14),
        tint: BRASS,
        glow: DARK,
    },
];

static ROCKETS: [Part; 5] = [
    Part {
        at: Vec3::new(0.0, -0.14, 0.0),
        size: Vec3::new(0.70, 0.24, 0.42),
        tint: IRON,
        glow: DARK,
    },
    Part {
        at: Vec3::new(-0.16, 0.06, -0.04),
        size: Vec3::new(0.15, 0.15, 0.54),
        tint: STEEL,
        glow: DARK,
    },
    Part {
        at: Vec3::new(-0.16, 0.06, 0.30),
        size: Vec3::new(0.15, 0.15, 0.16),
        tint: YELLOW,
        glow: WARHEAD,
    },
    Part {
        at: Vec3::new(0.16, 0.06, -0.04),
        size: Vec3::new(0.15, 0.15, 0.54),
        tint: STEEL,
        glow: DARK,
    },
    Part {
        at: Vec3::new(0.16, 0.06, 0.30),
        size: Vec3::new(0.15, 0.15, 0.16),
        tint: YELLOW,
        glow: WARHEAD,
    },
];

static CELLS: [Part; 3] = [
    Part {
        at: Vec3::new(0.0, 0.0, 0.0),
        size: Vec3::new(0.36, 0.36, 0.36),
        tint: GLASS,
        glow: CORE,
    },
    Part {
        at: Vec3::new(0.0, 0.23, 0.0),
        size: Vec3::new(0.46, 0.10, 0.46),
        tint: IRON,
        glow: DARK,
    },
    Part {
        at: Vec3::new(0.0, -0.23, 0.0),
        size: Vec3::new(0.46, 0.10, 0.46),
        tint: IRON,
        glow: DARK,
    },
];

static RED_KEY: [Part; 3] = [
    Part {
        at: Vec3::new(0.0, 0.0, 0.0),
        size: Vec3::new(0.46, 0.62, 0.07),
        tint: CRIMSON,
        glow: RED_CARD,
    },
    Part {
        at: Vec3::new(0.0, 0.17, 0.0),
        size: Vec3::new(0.34, 0.10, 0.09),
        tint: WHITE,
        glow: DARK,
    },
    Part {
        at: Vec3::new(0.0, -0.15, 0.0),
        size: Vec3::new(0.17, 0.13, 0.09),
        tint: BRASS,
        glow: DARK,
    },
];

static BLUE_KEY: [Part; 3] = [
    Part {
        at: Vec3::new(0.0, 0.0, 0.0),
        size: Vec3::new(0.46, 0.62, 0.07),
        tint: AZURE,
        glow: BLUE_CARD,
    },
    Part {
        at: Vec3::new(0.0, 0.17, 0.0),
        size: Vec3::new(0.34, 0.10, 0.09),
        tint: WHITE,
        glow: DARK,
    },
    Part {
        at: Vec3::new(0.0, -0.15, 0.0),
        size: Vec3::new(0.17, 0.13, 0.09),
        tint: BRASS,
        glow: DARK,
    },
];

pub fn shape(kind: PickupKind) -> Option<(&'static str, &'static [Part])> {
    let named: (&'static str, &'static [Part]) = match kind {
        PickupKind::Medkit => ("medkit", &MEDKIT),
        PickupKind::Stimpack => ("stimpack", &STIMPACK),
        PickupKind::Plate => ("plate", &PLATE),
        PickupKind::Shells => ("shells", &SHELLS),
        PickupKind::Bullets => ("bullets", &BULLETS),
        PickupKind::Rockets => ("rockets", &ROCKETS),
        PickupKind::Cells => ("cells", &CELLS),
        PickupKind::RedKey => ("red_key", &RED_KEY),
        PickupKind::BlueKey => ("blue_key", &BLUE_KEY),
        PickupKind::Chaingun | PickupKind::Launcher | PickupKind::Plasma => return None,
    };
    Some(named)
}
