use crate::contract::Surface;
use nightshade::prelude::Vec3;

pub struct Part {
    pub mesh: &'static str,
    pub offset: Vec3,
    pub scale: Vec3,
    pub surface: Surface,
}

pub struct Tool {
    pub name: &'static str,
    pub at: Vec3,
    pub half: Vec3,
    pub mass: f32,
    pub parts: &'static [Part],
}

pub const RACK_BOARD_AT: Vec3 = Vec3::new(7.83, 1.45, 3.55);
pub const RACK_BOARD_SIZE: Vec3 = Vec3::new(0.04, 1.1, 2.9);
pub const RACK_SHELF_AT: Vec3 = Vec3::new(7.7, 0.985, 3.55);
pub const RACK_SHELF_SIZE: Vec3 = Vec3::new(0.3, 0.03, 2.7);
pub const RACK_BRACKET_SIZE: Vec3 = Vec3::new(0.26, 0.03, 0.04);
pub const RACK_BRACKETS: &[f32] = &[2.4, 3.55, 4.7];
pub const RACK_HOOK_SIZE: Vec3 = Vec3::new(0.08, 0.03, 0.03);
pub const RACK_HOOKS: &[Vec3] = &[
    Vec3::new(7.78, 1.72, 2.6),
    Vec3::new(7.78, 1.72, 3.1),
    Vec3::new(7.78, 1.72, 3.6),
    Vec3::new(7.78, 1.72, 4.1),
    Vec3::new(7.78, 1.72, 4.6),
];
pub const RACK_LAY: f32 = 7.7;

pub const TOOLS: &[Tool] = &[
    Tool {
        name: "Hammer",
        at: Vec3::new(RACK_LAY, 0.0, 2.7),
        half: Vec3::new(0.05, 0.16, 0.025),
        mass: 1.4,
        parts: &[
            part(
                "Cylinder",
                0.0,
                -0.03,
                0.0,
                0.036,
                0.26,
                0.036,
                Surface::Drawer,
            ),
            part("Cube", 0.0, 0.12, 0.0, 0.1, 0.05, 0.05, Surface::Metal),
        ],
    },
    Tool {
        name: "Wrench",
        at: Vec3::new(RACK_LAY, 0.0, 3.15),
        half: Vec3::new(0.04, 0.15, 0.012),
        mass: 0.9,
        parts: &[
            part("Cube", 0.0, -0.02, 0.0, 0.035, 0.24, 0.02, Surface::Metal),
            part("Cube", 0.0, 0.12, 0.0, 0.08, 0.06, 0.02, Surface::Metal),
        ],
    },
    Tool {
        name: "Pipe",
        at: Vec3::new(RACK_LAY, 0.0, 3.7),
        half: Vec3::new(0.022, 0.26, 0.022),
        mass: 2.2,
        parts: &[part(
            "Cylinder",
            0.0,
            0.0,
            0.0,
            0.044,
            0.52,
            0.044,
            Surface::Rust,
        )],
    },
    Tool {
        name: "Crowbar",
        at: Vec3::new(RACK_LAY, 0.0, 4.35),
        half: Vec3::new(0.02, 0.28, 0.02),
        mass: 1.8,
        parts: &[
            part("Cube", 0.0, -0.02, 0.0, 0.03, 0.5, 0.03, Surface::Metal),
            part("Cube", 0.03, 0.26, 0.0, 0.08, 0.03, 0.03, Surface::Metal),
        ],
    },
];

const fn part(
    mesh: &'static str,
    x: f32,
    y: f32,
    z: f32,
    width: f32,
    height: f32,
    depth: f32,
    surface: Surface,
) -> Part {
    Part {
        mesh,
        offset: Vec3::new(x, y, z),
        scale: Vec3::new(width, height, depth),
        surface,
    }
}
