use crate::data::shape::Part;
use nightshade::prelude::Vec3;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ammo {
    #[default]
    Shells,
    Bullets,
    Rockets,
    Cells,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Muzzle {
    Pellets,
    Rocket,
    Plasma,
}

pub struct Gun {
    pub name: &'static str,
    pub ammo: Ammo,
    pub muzzle: Muzzle,
    pub cooldown: f32,
    pub damage: f32,
    pub pellets: u32,
    pub spread: f32,
    pub kick: f32,
    pub punch: f32,
    pub parts: &'static [Part],
}

pub const SHOTGUN: usize = 0;
pub const CHAINGUN: usize = 1;
pub const LAUNCHER: usize = 2;
pub const PLASMA: usize = 3;
pub const GUN_COUNT: usize = 4;

const STEEL: [f32; 4] = [0.30, 0.31, 0.35, 1.0];
const IRON: [f32; 4] = [0.19, 0.20, 0.23, 1.0];
const BRASS: [f32; 4] = [0.62, 0.48, 0.22, 1.0];
const RUST: [f32; 4] = [0.44, 0.24, 0.16, 1.0];
const GLASS: [f32; 4] = [0.36, 0.78, 0.92, 1.0];
const DARK: [f32; 3] = [0.0, 0.0, 0.0];
const CORE: [f32; 3] = [0.20, 1.60, 2.20];
const HEAT: [f32; 3] = [1.40, 0.42, 0.10];

pub static GUNS: [Gun; GUN_COUNT] = [
    Gun {
        name: "shotgun",
        ammo: Ammo::Shells,
        muzzle: Muzzle::Pellets,
        cooldown: 0.72,
        damage: 12.0,
        pellets: 9,
        spread: 0.055,
        kick: 0.080,
        punch: 0.26,
        parts: &[
            Part {
                at: Vec3::new(0.0, 0.0, 0.0),
                size: Vec3::new(0.11, 0.10, 0.46),
                tint: RUST,
                glow: DARK,
            },
            Part {
                at: Vec3::new(-0.032, 0.035, -0.42),
                size: Vec3::new(0.045, 0.045, 0.62),
                tint: IRON,
                glow: DARK,
            },
            Part {
                at: Vec3::new(0.032, 0.035, -0.42),
                size: Vec3::new(0.045, 0.045, 0.62),
                tint: IRON,
                glow: DARK,
            },
            Part {
                at: Vec3::new(0.0, -0.03, -0.30),
                size: Vec3::new(0.09, 0.05, 0.20),
                tint: BRASS,
                glow: DARK,
            },
            Part {
                at: Vec3::new(0.0, -0.10, 0.16),
                size: Vec3::new(0.07, 0.16, 0.10),
                tint: RUST,
                glow: DARK,
            },
        ],
    },
    Gun {
        name: "chaingun",
        ammo: Ammo::Bullets,
        muzzle: Muzzle::Pellets,
        cooldown: 0.085,
        damage: 7.5,
        pellets: 1,
        spread: 0.024,
        kick: 0.012,
        punch: 0.055,
        parts: &[
            Part {
                at: Vec3::new(0.0, 0.0, 0.0),
                size: Vec3::new(0.15, 0.14, 0.40),
                tint: STEEL,
                glow: DARK,
            },
            Part {
                at: Vec3::new(-0.04, 0.04, -0.42),
                size: Vec3::new(0.035, 0.035, 0.56),
                tint: IRON,
                glow: DARK,
            },
            Part {
                at: Vec3::new(0.04, 0.04, -0.42),
                size: Vec3::new(0.035, 0.035, 0.56),
                tint: IRON,
                glow: DARK,
            },
            Part {
                at: Vec3::new(-0.04, -0.04, -0.42),
                size: Vec3::new(0.035, 0.035, 0.56),
                tint: IRON,
                glow: DARK,
            },
            Part {
                at: Vec3::new(0.04, -0.04, -0.42),
                size: Vec3::new(0.035, 0.035, 0.56),
                tint: IRON,
                glow: DARK,
            },
            Part {
                at: Vec3::new(0.13, 0.0, 0.06),
                size: Vec3::new(0.10, 0.20, 0.22),
                tint: BRASS,
                glow: DARK,
            },
        ],
    },
    Gun {
        name: "launcher",
        ammo: Ammo::Rockets,
        muzzle: Muzzle::Rocket,
        cooldown: 0.92,
        damage: 0.0,
        pellets: 1,
        spread: 0.0,
        kick: 0.11,
        punch: 0.34,
        parts: &[
            Part {
                at: Vec3::new(0.0, 0.0, -0.10),
                size: Vec3::new(0.17, 0.17, 0.78),
                tint: IRON,
                glow: DARK,
            },
            Part {
                at: Vec3::new(0.0, 0.0, -0.52),
                size: Vec3::new(0.21, 0.21, 0.10),
                tint: STEEL,
                glow: DARK,
            },
            Part {
                at: Vec3::new(0.0, 0.12, -0.18),
                size: Vec3::new(0.05, 0.07, 0.34),
                tint: RUST,
                glow: DARK,
            },
            Part {
                at: Vec3::new(0.0, -0.12, 0.08),
                size: Vec3::new(0.07, 0.14, 0.12),
                tint: IRON,
                glow: DARK,
            },
            Part {
                at: Vec3::new(0.0, 0.0, -0.60),
                size: Vec3::new(0.07, 0.07, 0.07),
                tint: [1.0, 0.62, 0.30, 1.0],
                glow: HEAT,
            },
        ],
    },
    Gun {
        name: "plasma gun",
        ammo: Ammo::Cells,
        muzzle: Muzzle::Plasma,
        cooldown: 0.10,
        damage: 0.0,
        pellets: 1,
        spread: 0.0,
        kick: 0.022,
        punch: 0.075,
        parts: &[
            Part {
                at: Vec3::new(0.0, 0.0, 0.0),
                size: Vec3::new(0.14, 0.13, 0.50),
                tint: STEEL,
                glow: DARK,
            },
            Part {
                at: Vec3::new(-0.06, 0.03, -0.44),
                size: Vec3::new(0.035, 0.035, 0.46),
                tint: IRON,
                glow: DARK,
            },
            Part {
                at: Vec3::new(0.06, 0.03, -0.44),
                size: Vec3::new(0.035, 0.035, 0.46),
                tint: IRON,
                glow: DARK,
            },
            Part {
                at: Vec3::new(0.0, 0.06, -0.18),
                size: Vec3::new(0.09, 0.09, 0.16),
                tint: GLASS,
                glow: CORE,
            },
            Part {
                at: Vec3::new(0.0, -0.11, 0.10),
                size: Vec3::new(0.07, 0.14, 0.12),
                tint: IRON,
                glow: DARK,
            },
        ],
    },
];

pub fn gun(slot: usize) -> &'static Gun {
    &GUNS[slot.min(GUN_COUNT - 1)]
}
