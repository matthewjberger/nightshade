use crate::contract::{Cell, Scenery, TOWER_KINDS, TOWER_LEVELS, Tier};
use nalgebra_glm::Vec2;
use nightshade_sprite::prelude::SpriteRegion;

#[derive(Clone, Copy, Debug, Default)]
pub struct Piece {
    pub region: SpriteRegion,
    pub size: Vec2,
    pub anchor: Vec2,
}

#[derive(Clone, Copy, Debug)]
pub struct Grade {
    pub cost: u32,
    pub reach: f32,
    pub damage: f32,
    pub reload: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Walker {
    pub tier: Tier,
    pub health: f32,
    pub speed: f32,
    pub bounty: u32,
    pub ward: f32,
    pub splits: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct Wave {
    pub count: u32,
    pub tier: Tier,
    pub health: f32,
    pub speed: f32,
    pub ward: f32,
    pub bounty: u32,
    pub spacing: f32,
    pub boss: bool,
    pub splits: u8,
}

nightshade_config::config! {
    pub enum Settings {
        "Battle" {
            Difficulty: choice(1, ["Gentle", "Even", "Harsh"]),
            Rest: number(2.0, 12.0, 6.0),
        }
        "Board" {
            Rings: toggle(true),
            Endless: toggle(false),
            Marks: toggle(true),
        }
        "View" {
            Zoom: number(5.2, 9.0, 6.7),
        }
    }
}

pub const DIFFICULTY_SHARE: [f32; 3] = [0.72, 1.0, 1.4];
pub const DIFFICULTY_PURSE: [f32; 3] = [1.3, 1.0, 0.8];

pub const COLUMNS: u32 = 10;
pub const ROWS: u32 = 10;

const MEADOW_ROAD: [Cell; 32] = [
    Cell::new(0, 0),
    Cell::new(1, 0),
    Cell::new(2, 0),
    Cell::new(3, 0),
    Cell::new(4, 0),
    Cell::new(4, 1),
    Cell::new(4, 2),
    Cell::new(3, 2),
    Cell::new(2, 2),
    Cell::new(1, 2),
    Cell::new(1, 3),
    Cell::new(1, 4),
    Cell::new(1, 5),
    Cell::new(2, 5),
    Cell::new(3, 5),
    Cell::new(4, 5),
    Cell::new(5, 5),
    Cell::new(6, 5),
    Cell::new(6, 4),
    Cell::new(6, 3),
    Cell::new(7, 3),
    Cell::new(8, 3),
    Cell::new(9, 3),
    Cell::new(9, 4),
    Cell::new(9, 5),
    Cell::new(9, 6),
    Cell::new(9, 7),
    Cell::new(9, 8),
    Cell::new(8, 8),
    Cell::new(7, 8),
    Cell::new(6, 8),
    Cell::new(5, 8),
];

const MEADOW_SCATTER: [(Cell, Scenery); 14] = [
    (Cell::new(7, 0), Scenery::PineTall),
    (Cell::new(9, 1), Scenery::PinePair),
    (Cell::new(0, 3), Scenery::RocksPair),
    (Cell::new(3, 3), Scenery::OakBroad),
    (Cell::new(5, 2), Scenery::RocksTall),
    (Cell::new(8, 1), Scenery::RocksSlab),
    (Cell::new(0, 6), Scenery::PineStand),
    (Cell::new(3, 7), Scenery::ScrubLow),
    (Cell::new(5, 6), Scenery::RocksPair),
    (Cell::new(7, 6), Scenery::PineTall),
    (Cell::new(2, 8), Scenery::PinePair),
    (Cell::new(8, 5), Scenery::RocksSlab),
    (Cell::new(4, 4), Scenery::OakBroad),
    (Cell::new(2, 3), Scenery::OakLone),
];

const COIL_ROAD: [Cell; 32] = [
    Cell::new(9, 0),
    Cell::new(8, 0),
    Cell::new(7, 0),
    Cell::new(6, 0),
    Cell::new(5, 0),
    Cell::new(5, 1),
    Cell::new(5, 2),
    Cell::new(5, 3),
    Cell::new(6, 3),
    Cell::new(7, 3),
    Cell::new(8, 3),
    Cell::new(9, 3),
    Cell::new(9, 4),
    Cell::new(9, 5),
    Cell::new(8, 5),
    Cell::new(7, 5),
    Cell::new(6, 5),
    Cell::new(5, 5),
    Cell::new(4, 5),
    Cell::new(3, 5),
    Cell::new(2, 5),
    Cell::new(2, 6),
    Cell::new(2, 7),
    Cell::new(3, 7),
    Cell::new(4, 7),
    Cell::new(5, 7),
    Cell::new(6, 7),
    Cell::new(6, 8),
    Cell::new(6, 9),
    Cell::new(5, 9),
    Cell::new(4, 9),
    Cell::new(3, 9),
];

const COIL_SCATTER: [(Cell, Scenery); 14] = [
    (Cell::new(2, 0), Scenery::ScrubLow),
    (Cell::new(0, 2), Scenery::OakPair),
    (Cell::new(3, 2), Scenery::RocksTrio),
    (Cell::new(8, 1), Scenery::PinePair),
    (Cell::new(1, 4), Scenery::RocksDome),
    (Cell::new(7, 1), Scenery::RocksTall),
    (Cell::new(0, 6), Scenery::OakLone),
    (Cell::new(4, 3), Scenery::PineTrio),
    (Cell::new(8, 7), Scenery::RocksTrio),
    (Cell::new(1, 8), Scenery::ScrubLow),
    (Cell::new(7, 9), Scenery::OakPair),
    (Cell::new(9, 8), Scenery::RocksTall),
    (Cell::new(3, 3), Scenery::PinePair),
    (Cell::new(8, 9), Scenery::ScrubPair),
];

const FORK_ROAD: [Cell; 32] = [
    Cell::new(0, 0),
    Cell::new(0, 1),
    Cell::new(0, 2),
    Cell::new(0, 3),
    Cell::new(1, 3),
    Cell::new(2, 3),
    Cell::new(3, 3),
    Cell::new(3, 2),
    Cell::new(3, 1),
    Cell::new(4, 1),
    Cell::new(5, 1),
    Cell::new(6, 1),
    Cell::new(6, 2),
    Cell::new(6, 3),
    Cell::new(6, 4),
    Cell::new(6, 5),
    Cell::new(5, 5),
    Cell::new(4, 5),
    Cell::new(3, 5),
    Cell::new(2, 5),
    Cell::new(2, 6),
    Cell::new(2, 7),
    Cell::new(2, 8),
    Cell::new(3, 8),
    Cell::new(4, 8),
    Cell::new(5, 8),
    Cell::new(6, 8),
    Cell::new(7, 8),
    Cell::new(7, 7),
    Cell::new(7, 6),
    Cell::new(8, 6),
    Cell::new(9, 6),
];

const FORK_SCATTER: [(Cell, Scenery); 14] = [
    (Cell::new(1, 0), Scenery::PineTrio),
    (Cell::new(8, 0), Scenery::PineStand),
    (Cell::new(9, 1), Scenery::RocksLow),
    (Cell::new(0, 5), Scenery::OakPair),
    (Cell::new(4, 3), Scenery::PineStand),
    (Cell::new(8, 3), Scenery::RocksDome),
    (Cell::new(5, 6), Scenery::ScrubPair),
    (Cell::new(9, 8), Scenery::OakBroad),
    (Cell::new(0, 8), Scenery::RocksLow),
    (Cell::new(4, 2), Scenery::PineTrio),
    (Cell::new(8, 1), Scenery::PineStand),
    (Cell::new(6, 9), Scenery::RocksDome),
    (Cell::new(1, 6), Scenery::OakPair),
    (Cell::new(5, 0), Scenery::PineTall),
];

const LADDER_ROAD: [Cell; 38] = [
    Cell::new(0, 0),
    Cell::new(1, 0),
    Cell::new(2, 0),
    Cell::new(3, 0),
    Cell::new(4, 0),
    Cell::new(5, 0),
    Cell::new(6, 0),
    Cell::new(7, 0),
    Cell::new(7, 1),
    Cell::new(7, 2),
    Cell::new(6, 2),
    Cell::new(5, 2),
    Cell::new(4, 2),
    Cell::new(3, 2),
    Cell::new(2, 2),
    Cell::new(1, 2),
    Cell::new(1, 3),
    Cell::new(1, 4),
    Cell::new(2, 4),
    Cell::new(3, 4),
    Cell::new(4, 4),
    Cell::new(5, 4),
    Cell::new(6, 4),
    Cell::new(7, 4),
    Cell::new(7, 5),
    Cell::new(7, 6),
    Cell::new(6, 6),
    Cell::new(5, 6),
    Cell::new(4, 6),
    Cell::new(3, 6),
    Cell::new(2, 6),
    Cell::new(1, 6),
    Cell::new(1, 7),
    Cell::new(1, 8),
    Cell::new(2, 8),
    Cell::new(3, 8),
    Cell::new(4, 8),
    Cell::new(5, 8),
];

const LADDER_SCATTER: [(Cell, Scenery); 14] = [
    (Cell::new(8, 0), Scenery::OakBroad),
    (Cell::new(9, 1), Scenery::OakLone),
    (Cell::new(0, 1), Scenery::RocksPair),
    (Cell::new(3, 1), Scenery::PineStand),
    (Cell::new(5, 1), Scenery::RocksTall),
    (Cell::new(0, 5), Scenery::RocksSlab),
    (Cell::new(3, 5), Scenery::PineTall),
    (Cell::new(5, 5), Scenery::PinePair),
    (Cell::new(8, 3), Scenery::RocksPair),
    (Cell::new(9, 5), Scenery::OakBroad),
    (Cell::new(0, 9), Scenery::OakLone),
    (Cell::new(7, 9), Scenery::RocksSlab),
    (Cell::new(3, 3), Scenery::PineStand),
    (Cell::new(9, 7), Scenery::ScrubLow),
];

const RIM_ROAD: [Cell; 43] = [
    Cell::new(0, 0),
    Cell::new(1, 0),
    Cell::new(2, 0),
    Cell::new(3, 0),
    Cell::new(4, 0),
    Cell::new(5, 0),
    Cell::new(6, 0),
    Cell::new(7, 0),
    Cell::new(8, 0),
    Cell::new(9, 0),
    Cell::new(9, 1),
    Cell::new(9, 2),
    Cell::new(9, 3),
    Cell::new(9, 4),
    Cell::new(9, 5),
    Cell::new(9, 6),
    Cell::new(9, 7),
    Cell::new(9, 8),
    Cell::new(9, 9),
    Cell::new(8, 9),
    Cell::new(7, 9),
    Cell::new(6, 9),
    Cell::new(5, 9),
    Cell::new(4, 9),
    Cell::new(3, 9),
    Cell::new(2, 9),
    Cell::new(1, 9),
    Cell::new(0, 9),
    Cell::new(0, 8),
    Cell::new(0, 7),
    Cell::new(0, 6),
    Cell::new(0, 5),
    Cell::new(0, 4),
    Cell::new(0, 3),
    Cell::new(1, 3),
    Cell::new(2, 3),
    Cell::new(3, 3),
    Cell::new(3, 4),
    Cell::new(3, 5),
    Cell::new(3, 6),
    Cell::new(4, 6),
    Cell::new(5, 6),
    Cell::new(6, 6),
];

const RIM_SCATTER: [(Cell, Scenery); 14] = [
    (Cell::new(2, 1), Scenery::PinePair),
    (Cell::new(4, 1), Scenery::ScrubPair),
    (Cell::new(6, 1), Scenery::RocksTrio),
    (Cell::new(8, 2), Scenery::OakLone),
    (Cell::new(1, 5), Scenery::RocksDome),
    (Cell::new(5, 2), Scenery::RocksTall),
    (Cell::new(7, 3), Scenery::ScrubLow),
    (Cell::new(2, 7), Scenery::OakPair),
    (Cell::new(5, 8), Scenery::RocksTrio),
    (Cell::new(7, 7), Scenery::PinePair),
    (Cell::new(4, 4), Scenery::ScrubPair),
    (Cell::new(6, 3), Scenery::RocksTall),
    (Cell::new(8, 5), Scenery::OakLone),
    (Cell::new(2, 2), Scenery::PineTrio),
];

const KETTLE_ROAD: [Cell; 37] = [
    Cell::new(0, 1),
    Cell::new(1, 1),
    Cell::new(2, 1),
    Cell::new(3, 1),
    Cell::new(4, 1),
    Cell::new(5, 1),
    Cell::new(6, 1),
    Cell::new(7, 1),
    Cell::new(8, 1),
    Cell::new(8, 2),
    Cell::new(8, 3),
    Cell::new(8, 4),
    Cell::new(8, 5),
    Cell::new(8, 6),
    Cell::new(8, 7),
    Cell::new(8, 8),
    Cell::new(7, 8),
    Cell::new(6, 8),
    Cell::new(5, 8),
    Cell::new(4, 8),
    Cell::new(3, 8),
    Cell::new(2, 8),
    Cell::new(1, 8),
    Cell::new(1, 7),
    Cell::new(1, 6),
    Cell::new(1, 5),
    Cell::new(1, 4),
    Cell::new(2, 4),
    Cell::new(3, 4),
    Cell::new(4, 4),
    Cell::new(5, 4),
    Cell::new(6, 4),
    Cell::new(6, 5),
    Cell::new(6, 6),
    Cell::new(5, 6),
    Cell::new(4, 6),
    Cell::new(3, 6),
];

const KETTLE_SCATTER: [(Cell, Scenery); 14] = [
    (Cell::new(0, 0), Scenery::OakPair),
    (Cell::new(9, 9), Scenery::PineTall),
    (Cell::new(9, 0), Scenery::RocksLow),
    (Cell::new(4, 5), Scenery::ScrubPair),
    (Cell::new(0, 9), Scenery::PineStand),
    (Cell::new(6, 2), Scenery::RocksDome),
    (Cell::new(4, 0), Scenery::PineTrio),
    (Cell::new(2, 2), Scenery::PineStand),
    (Cell::new(0, 4), Scenery::RocksLow),
    (Cell::new(9, 4), Scenery::OakPair),
    (Cell::new(7, 6), Scenery::PineTall),
    (Cell::new(2, 7), Scenery::RocksDome),
    (Cell::new(4, 9), Scenery::ScrubPair),
    (Cell::new(5, 7), Scenery::OakBroad),
];

const BIGHT_ROAD: [Cell; 45] = [
    Cell::new(9, 0),
    Cell::new(8, 0),
    Cell::new(7, 0),
    Cell::new(6, 0),
    Cell::new(5, 0),
    Cell::new(4, 0),
    Cell::new(3, 0),
    Cell::new(2, 0),
    Cell::new(2, 1),
    Cell::new(2, 2),
    Cell::new(3, 2),
    Cell::new(4, 2),
    Cell::new(5, 2),
    Cell::new(6, 2),
    Cell::new(7, 2),
    Cell::new(8, 2),
    Cell::new(9, 2),
    Cell::new(9, 3),
    Cell::new(9, 4),
    Cell::new(8, 4),
    Cell::new(7, 4),
    Cell::new(6, 4),
    Cell::new(5, 4),
    Cell::new(4, 4),
    Cell::new(3, 4),
    Cell::new(2, 4),
    Cell::new(1, 4),
    Cell::new(1, 5),
    Cell::new(1, 6),
    Cell::new(2, 6),
    Cell::new(3, 6),
    Cell::new(4, 6),
    Cell::new(5, 6),
    Cell::new(6, 6),
    Cell::new(7, 6),
    Cell::new(8, 6),
    Cell::new(8, 7),
    Cell::new(8, 8),
    Cell::new(7, 8),
    Cell::new(6, 8),
    Cell::new(5, 8),
    Cell::new(4, 8),
    Cell::new(3, 8),
    Cell::new(2, 8),
    Cell::new(1, 8),
];

const BIGHT_SCATTER: [(Cell, Scenery); 14] = [
    (Cell::new(0, 0), Scenery::PineStand),
    (Cell::new(9, 9), Scenery::ScrubLow),
    (Cell::new(8, 1), Scenery::RocksPair),
    (Cell::new(2, 7), Scenery::PineTall),
    (Cell::new(4, 3), Scenery::RocksTall),
    (Cell::new(7, 5), Scenery::RocksSlab),
    (Cell::new(0, 4), Scenery::OakBroad),
    (Cell::new(0, 9), Scenery::OakLone),
    (Cell::new(4, 9), Scenery::RocksPair),
    (Cell::new(3, 1), Scenery::PineStand),
    (Cell::new(1, 2), Scenery::ScrubLow),
    (Cell::new(3, 5), Scenery::RocksSlab),
    (Cell::new(9, 6), Scenery::PineTall),
    (Cell::new(5, 7), Scenery::PinePair),
];

pub struct Board {
    pub name: &'static str,
    pub road: &'static [Cell],
    pub scatter: &'static [(Cell, Scenery)],
}

pub const BOARDS: [Board; 7] = [
    Board {
        name: "Long Meadow",
        road: &MEADOW_ROAD,
        scatter: &MEADOW_SCATTER,
    },
    Board {
        name: "Coil",
        road: &COIL_ROAD,
        scatter: &COIL_SCATTER,
    },
    Board {
        name: "Fork",
        road: &FORK_ROAD,
        scatter: &FORK_SCATTER,
    },
    Board {
        name: "Ladder",
        road: &LADDER_ROAD,
        scatter: &LADDER_SCATTER,
    },
    Board {
        name: "Rim",
        road: &RIM_ROAD,
        scatter: &RIM_SCATTER,
    },
    Board {
        name: "Kettle",
        road: &KETTLE_ROAD,
        scatter: &KETTLE_SCATTER,
    },
    Board {
        name: "Bight",
        road: &BIGHT_ROAD,
        scatter: &BIGHT_SCATTER,
    },
];

pub const ROAD_MASKS: [u8; 11] = [3, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15];

pub const SHARD_SCALES: [f32; 3] = [0.62, 0.82, 0.95];

pub const GRADES: [[Grade; TOWER_LEVELS]; TOWER_KINDS] = [
    [
        Grade {
            cost: 30,
            reach: 3.2,
            damage: 13.0,
            reload: 0.62,
        },
        Grade {
            cost: 45,
            reach: 3.7,
            damage: 24.0,
            reload: 0.56,
        },
        Grade {
            cost: 80,
            reach: 4.2,
            damage: 42.0,
            reload: 0.48,
        },
    ],
    [
        Grade {
            cost: 55,
            reach: 5.4,
            damage: 30.0,
            reload: 1.35,
        },
        Grade {
            cost: 75,
            reach: 6.1,
            damage: 58.0,
            reload: 1.2,
        },
        Grade {
            cost: 130,
            reach: 6.9,
            damage: 104.0,
            reload: 1.05,
        },
    ],
    [
        Grade {
            cost: 45,
            reach: 2.3,
            damage: 5.0,
            reload: 0.14,
        },
        Grade {
            cost: 60,
            reach: 2.6,
            damage: 9.0,
            reload: 0.12,
        },
        Grade {
            cost: 105,
            reach: 3.0,
            damage: 15.0,
            reload: 0.1,
        },
    ],
    [
        Grade {
            cost: 50,
            reach: 3.4,
            damage: 20.0,
            reload: 0.8,
        },
        Grade {
            cost: 70,
            reach: 3.8,
            damage: 36.0,
            reload: 0.72,
        },
        Grade {
            cost: 120,
            reach: 4.3,
            damage: 62.0,
            reload: 0.64,
        },
    ],
    [
        Grade {
            cost: 95,
            reach: 4.4,
            damage: 95.0,
            reload: 2.6,
        },
        Grade {
            cost: 140,
            reach: 4.8,
            damage: 178.0,
            reload: 2.4,
        },
        Grade {
            cost: 230,
            reach: 5.3,
            damage: 320.0,
            reload: 2.2,
        },
    ],
    [
        Grade {
            cost: 16,
            reach: 2.4,
            damage: 7.0,
            reload: 0.5,
        },
        Grade {
            cost: 24,
            reach: 2.7,
            damage: 13.0,
            reload: 0.46,
        },
        Grade {
            cost: 42,
            reach: 3.0,
            damage: 23.0,
            reload: 0.42,
        },
    ],
];

pub const TOWER_BLURB: [&str; TOWER_KINDS] = [
    "steady stone, splits its hit",
    "far shots, slow and heavy",
    "a fast drum, chills and thins",
    "cuts through a warded shell",
    "one siege shot, slow to load",
    "cheap pickets, thin and many",
];

pub const WAVES: [Wave; 12] = [
    Wave {
        count: 6,
        tier: Tier::Shard,
        health: 42.0,
        speed: 1.8,
        ward: 0.0,
        bounty: 9,
        spacing: 1.5,
        boss: false,
        splits: 0,
    },
    Wave {
        count: 8,
        tier: Tier::Shard,
        health: 58.0,
        speed: 1.9,
        ward: 0.0,
        bounty: 9,
        spacing: 1.3,
        boss: false,
        splits: 0,
    },
    Wave {
        count: 7,
        tier: Tier::Cluster,
        health: 105.0,
        speed: 1.7,
        ward: 0.0,
        bounty: 14,
        spacing: 1.5,
        boss: false,
        splits: 0,
    },
    Wave {
        count: 10,
        tier: Tier::Shard,
        health: 92.0,
        speed: 2.3,
        ward: 3.0,
        bounty: 11,
        spacing: 1.0,
        boss: false,
        splits: 0,
    },
    Wave {
        count: 9,
        tier: Tier::Cluster,
        health: 168.0,
        speed: 1.8,
        ward: 0.0,
        bounty: 16,
        spacing: 1.3,
        boss: false,
        splits: 0,
    },
    Wave {
        count: 5,
        tier: Tier::Monolith,
        health: 340.0,
        speed: 1.4,
        ward: 10.0,
        bounty: 30,
        spacing: 2.0,
        boss: true,
        splits: 1,
    },
    Wave {
        count: 12,
        tier: Tier::Shard,
        health: 150.0,
        speed: 2.7,
        ward: 5.0,
        bounty: 13,
        spacing: 0.85,
        boss: false,
        splits: 0,
    },
    Wave {
        count: 11,
        tier: Tier::Cluster,
        health: 280.0,
        speed: 2.0,
        ward: 0.0,
        bounty: 20,
        spacing: 1.1,
        boss: false,
        splits: 0,
    },
    Wave {
        count: 7,
        tier: Tier::Monolith,
        health: 520.0,
        speed: 1.5,
        ward: 14.0,
        bounty: 36,
        spacing: 1.7,
        boss: true,
        splits: 1,
    },
    Wave {
        count: 14,
        tier: Tier::Cluster,
        health: 400.0,
        speed: 2.4,
        ward: 8.0,
        bounty: 22,
        spacing: 0.8,
        boss: false,
        splits: 0,
    },
    Wave {
        count: 10,
        tier: Tier::Monolith,
        health: 760.0,
        speed: 1.7,
        ward: 0.0,
        bounty: 44,
        spacing: 1.4,
        boss: true,
        splits: 0,
    },
    Wave {
        count: 12,
        tier: Tier::Monolith,
        health: 900.0,
        speed: 2.0,
        ward: 18.0,
        bounty: 55,
        spacing: 1.0,
        boss: true,
        splits: 1,
    },
];

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct Record {
    pub best: u32,
}

impl nightshade_save::Save for Record {
    const FOLDER: &'static str = crate::theme::RECORD_FOLDER;
    const NAME: &'static str = crate::theme::RECORD_NAME;
    const VERSION: u32 = crate::theme::RECORD_VERSION;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    ChooseOne,
    ChooseTwo,
    ChooseThree,
    ChooseFour,
    ChooseFive,
    ChooseSix,
    Pace,
    Build,
    Upgrade,
    Sell,
    Aim,
    Hurry,
    Pause,
}

impl Ask {
    pub const CHOICES: [Ask; 6] = [
        Ask::ChooseOne,
        Ask::ChooseTwo,
        Ask::ChooseThree,
        Ask::ChooseFour,
        Ask::ChooseFive,
        Ask::ChooseSix,
    ];
}

impl nightshade_input::Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::ChooseOne,
            Self::ChooseTwo,
            Self::ChooseThree,
            Self::ChooseFour,
            Self::ChooseFive,
            Self::ChooseSix,
            Self::Pace,
            Self::Build,
            Self::Upgrade,
            Self::Sell,
            Self::Aim,
            Self::Hurry,
            Self::Pause,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::ChooseOne => "choose one",
            Self::ChooseTwo => "choose two",
            Self::ChooseThree => "choose three",
            Self::ChooseFour => "choose four",
            Self::ChooseFive => "choose five",
            Self::ChooseSix => "choose six",
            Self::Pace => "pace",
            Self::Build => "build",
            Self::Upgrade => "upgrade",
            Self::Sell => "sell",
            Self::Aim => "aim",
            Self::Hurry => "hurry",
            Self::Pause => "pause",
        }
    }

    fn default_bindings(&self) -> &'static [nightshade_input::Binding] {
        use nightshade_input::{Binding, Pointer};
        use nightshade_platform::KeyCode;
        match self {
            Self::ChooseOne => &[Binding::Key(KeyCode::Digit1)],
            Self::ChooseTwo => &[Binding::Key(KeyCode::Digit2)],
            Self::ChooseThree => &[Binding::Key(KeyCode::Digit3)],
            Self::ChooseFour => &[Binding::Key(KeyCode::Digit4)],
            Self::ChooseFive => &[Binding::Key(KeyCode::Digit5)],
            Self::ChooseSix => &[Binding::Key(KeyCode::Digit6)],
            Self::Pace => &[Binding::Key(KeyCode::KeyF)],
            Self::Build => &[Binding::Pointer(Pointer::Left)],
            Self::Upgrade => &[Binding::Key(KeyCode::KeyU)],
            Self::Sell => &[Binding::Key(KeyCode::KeyX)],
            Self::Aim => &[Binding::Key(KeyCode::KeyA)],
            Self::Hurry => &[Binding::Key(KeyCode::Space)],
            Self::Pause => &[Binding::Key(KeyCode::Escape)],
        }
    }
}
