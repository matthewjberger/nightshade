use nalgebra_glm::Vec3;

pub struct ThugKind {
    pub name: &'static str,
    pub page: u32,
    pub health: f32,
    pub speed: f32,
    pub damage: f32,
    pub rest: f32,
    pub scale: f32,
    pub radius: f32,
    pub reach: f32,
    pub flinches: bool,
    pub chief: bool,
    pub score: u32,
    pub tint: [f32; 4],
}

pub const PUNK: u32 = 0;
pub const RUNNER: u32 = 1;
pub const BRUTE: u32 = 2;
pub const ENFORCER: u32 = 3;
pub const BOSS: u32 = 4;
pub const KIND_COUNT: usize = 5;

pub const THUG_KINDS: [ThugKind; KIND_COUNT] = [
    ThugKind {
        name: "PUNK",
        page: 1,
        health: 42.0,
        speed: 2.6,
        damage: 7.0,
        rest: 1.1,
        scale: 1.0,
        radius: 0.42,
        reach: 0.0,
        flinches: true,
        chief: false,
        score: 100,
        tint: [0.86, 0.72, 0.78, 1.0],
    },
    ThugKind {
        name: "RUNNER",
        page: 2,
        health: 30.0,
        speed: 4.1,
        damage: 6.0,
        rest: 0.75,
        scale: 0.96,
        radius: 0.38,
        reach: 0.0,
        flinches: true,
        chief: false,
        score: 150,
        tint: [0.78, 0.84, 1.0, 1.0],
    },
    ThugKind {
        name: "BRUTE",
        page: 3,
        health: 130.0,
        speed: 1.85,
        damage: 15.0,
        rest: 1.6,
        scale: 1.35,
        radius: 0.58,
        reach: 0.25,
        flinches: false,
        chief: false,
        score: 400,
        tint: [0.92, 0.86, 0.8, 1.0],
    },
    ThugKind {
        name: "ENFORCER",
        page: 4,
        health: 210.0,
        speed: 2.7,
        damage: 16.0,
        rest: 1.2,
        scale: 1.38,
        radius: 0.58,
        reach: 0.3,
        flinches: false,
        chief: true,
        score: 900,
        tint: [0.74, 0.84, 0.96, 1.0],
    },
    ThugKind {
        name: "IRONMOUTH",
        page: 4,
        health: 380.0,
        speed: 2.4,
        damage: 19.0,
        rest: 1.05,
        scale: 1.58,
        radius: 0.66,
        reach: 0.4,
        flinches: false,
        chief: true,
        score: 2000,
        tint: [1.0, 0.84, 0.84, 1.0],
    },
];

pub struct WeaponKind {
    pub name: &'static str,
    pub taken: &'static str,
    pub sprite: &'static str,
    pub damage: f32,
    pub reach: f32,
    pub swings: u32,
    pub thrown: f32,
    pub size: [f32; 2],
    pub color: [f32; 4],
}

pub const PIPE: u32 = 0;
pub const BOTTLE: u32 = 1;
pub const WEAPON_COUNT: usize = 2;

pub const WEAPON_KINDS: [WeaponKind; WEAPON_COUNT] = [
    WeaponKind {
        name: "PIPE",
        taken: "RAY: A pipe. Now we are talking.",
        sprite: "pipe",
        damage: 28.0,
        reach: 1.8,
        swings: 6,
        thrown: 34.0,
        size: [1.15, 0.2],
        color: [0.78, 0.8, 0.86, 1.0],
    },
    WeaponKind {
        name: "BOTTLE",
        taken: "RAY: One good throw in this.",
        sprite: "bottle",
        damage: 16.0,
        reach: 1.3,
        swings: 1,
        thrown: 30.0,
        size: [0.36, 0.54],
        color: [0.45, 0.78, 0.42, 1.0],
    },
];

pub struct PickupKind {
    pub name: &'static str,
    pub line: &'static str,
    pub sprite: &'static str,
    pub heal: f32,
    pub size: [f32; 2],
    pub color: [f32; 4],
}

pub const ROAST: u32 = 0;
pub const DRINK: u32 = 1;
pub const PICKUP_COUNT: usize = 2;

pub const PICKUP_KINDS: [PickupKind; PICKUP_COUNT] = [
    PickupKind {
        name: "ROAST",
        line: "RAY: That helps.",
        sprite: "roast",
        heal: 38.0,
        size: [0.54, 0.54],
        color: [0.94, 0.62, 0.32, 1.0],
    },
    PickupKind {
        name: "DRINK",
        line: "RAY: Cold. Good.",
        sprite: "drink",
        heal: 18.0,
        size: [0.34, 0.48],
        color: [0.52, 0.72, 0.92, 1.0],
    },
];

pub struct BreakKind {
    pub material: &'static str,
    pub size: [f32; 3],
    pub health: f32,
    pub score: u32,
    pub radius: f32,
    pub shards: u32,
}

pub const CRATE: u32 = 0;
pub const BARREL: u32 = 1;
pub const BREAK_COUNT: usize = 2;

pub const BREAK_KINDS: [BreakKind; BREAK_COUNT] = [
    BreakKind {
        material: WOOD,
        size: [0.9, 0.9, 0.9],
        health: 30.0,
        score: 50,
        radius: 0.5,
        shards: 6,
    },
    BreakKind {
        material: RUST,
        size: [0.78, 1.15, 0.78],
        health: 44.0,
        score: 70,
        radius: 0.46,
        shards: 7,
    },
];

pub const DROP_NONE: u32 = 0;
pub const DROP_ROAST: u32 = 1;
pub const DROP_DRINK: u32 = 2;
pub const DROP_PIPE: u32 = 3;
pub const DROP_BOTTLE: u32 = 4;
pub const DROP_TABLE: [u32; 8] = [
    DROP_ROAST,
    DROP_PIPE,
    DROP_DRINK,
    DROP_NONE,
    DROP_DRINK,
    DROP_BOTTLE,
    DROP_ROAST,
    DROP_PIPE,
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Look {
    Street,
    Alley,
    Subway,
    Rooftop,
    Yard,
}

pub struct Wave {
    pub kinds: [u32; KIND_COUNT],
    pub at_once: u32,
    pub gate: f32,
    pub chief: bool,
    pub line: &'static str,
}

pub struct StagePlan {
    pub name: &'static str,
    pub look: Look,
    pub end: f32,
    pub waves: &'static [Wave],
}

pub const STAGE_COUNT: usize = 5;

const SEVENTH_STREET: [Wave; 3] = [
    Wave {
        kinds: [3, 1, 0, 0, 0],
        at_once: 3,
        gate: 15.0,
        chief: false,
        line: "IRONMOUTH: Wrong street, old man.",
    },
    Wave {
        kinds: [4, 2, 0, 0, 0],
        at_once: 4,
        gate: 29.0,
        chief: false,
        line: "RAY: They keep sending the small ones.",
    },
    Wave {
        kinds: [2, 0, 1, 1, 0],
        at_once: 3,
        gate: 43.0,
        chief: true,
        line: "IRONMOUTH: Sweepers. Put him through a window.",
    },
];

const BACK_ALLEY: [Wave; 3] = [
    Wave {
        kinds: [3, 2, 0, 0, 0],
        at_once: 3,
        gate: 14.0,
        chief: false,
        line: "RAY: They keep their pipes back here. Good.",
    },
    Wave {
        kinds: [3, 3, 1, 0, 0],
        at_once: 4,
        gate: 27.0,
        chief: false,
        line: "RAY: Narrow. That suits me.",
    },
    Wave {
        kinds: [2, 1, 0, 1, 0],
        at_once: 3,
        gate: 39.0,
        chief: true,
        line: "IRONMOUTH: You know what he did to the last one.",
    },
];

const LINE_FOUR: [Wave; 3] = [
    Wave {
        kinds: [4, 2, 0, 0, 0],
        at_once: 4,
        gate: 15.0,
        chief: false,
        line: "RAY: The last train went an hour ago. So did the witnesses.",
    },
    Wave {
        kinds: [3, 4, 1, 0, 0],
        at_once: 4,
        gate: 30.0,
        chief: false,
        line: "RAY: Mind the edge.",
    },
    Wave {
        kinds: [2, 2, 1, 1, 0],
        at_once: 4,
        gate: 45.0,
        chief: true,
        line: "IRONMOUTH: Hold the platform!",
    },
];

const ROOFTOP: [Wave; 3] = [
    Wave {
        kinds: [3, 4, 0, 0, 0],
        at_once: 4,
        gate: 15.0,
        chief: false,
        line: "RAY: Long way down. Watch your step.",
    },
    Wave {
        kinds: [2, 4, 2, 0, 0],
        at_once: 4,
        gate: 28.0,
        chief: false,
        line: "RAY: Runners up here. Of course.",
    },
    Wave {
        kinds: [1, 2, 1, 2, 0],
        at_once: 4,
        gate: 39.0,
        chief: true,
        line: "IRONMOUTH: Off the roof with him.",
    },
];

const THE_YARD: [Wave; 3] = [
    Wave {
        kinds: [4, 3, 1, 0, 0],
        at_once: 4,
        gate: 14.0,
        chief: false,
        line: "RAY: The whole yard turned out.",
    },
    Wave {
        kinds: [2, 3, 2, 1, 0],
        at_once: 4,
        gate: 26.0,
        chief: false,
        line: "IRONMOUTH: Everything you have. Now.",
    },
    Wave {
        kinds: [0, 2, 1, 0, 1],
        at_once: 3,
        gate: 35.0,
        chief: true,
        line: "IRONMOUTH: You got this far. Now you stay.",
    },
];

pub const STAGES: [StagePlan; STAGE_COUNT] = [
    StagePlan {
        name: "SEVENTH STREET",
        look: Look::Street,
        end: 48.0,
        waves: &SEVENTH_STREET,
    },
    StagePlan {
        name: "THE BACK ALLEY",
        look: Look::Alley,
        end: 44.0,
        waves: &BACK_ALLEY,
    },
    StagePlan {
        name: "LINE 4 PLATFORM",
        look: Look::Subway,
        end: 50.0,
        waves: &LINE_FOUR,
    },
    StagePlan {
        name: "THE ROOFTOP",
        look: Look::Rooftop,
        end: 44.0,
        waves: &ROOFTOP,
    },
    StagePlan {
        name: "THE YARD",
        look: Look::Yard,
        end: 40.0,
        waves: &THE_YARD,
    },
];

pub const ARCADE_WAVE: [u32; KIND_COUNT] = [4, 2, 1, 0, 0];
pub const ARCADE_STEP: f32 = 0.2;
pub const ARCADE_AT_ONCE: u32 = 4;
pub const ARCADE_LOOKS: [Look; 4] = [Look::Street, Look::Alley, Look::Subway, Look::Rooftop];
pub const ARCADE_CHIEF_ROUND: u32 = 3;

pub const MODEL_BYTES: [&[u8]; 6] = [
    include_bytes!("../../assets/models/detail-bench.glb"),
    include_bytes!("../../assets/models/detail-dumpster-closed.glb"),
    include_bytes!("../../assets/models/detail-barrier-type-a.glb"),
    include_bytes!("../../assets/models/pallet.glb"),
    include_bytes!("../../assets/models/detail-light-traffic.glb"),
    include_bytes!("../../assets/models/tree-small.glb"),
];
pub const MODEL_TEXTURES: [(&str, &[u8]); 7] = [
    (
        "Textures/metal.png",
        include_bytes!("../../assets/models/Textures/metal.png"),
    ),
    (
        "Textures/planks.png",
        include_bytes!("../../assets/models/Textures/planks.png"),
    ),
    (
        "Textures/metal_wall.png",
        include_bytes!("../../assets/models/Textures/metal_wall.png"),
    ),
    (
        "Textures/roof.png",
        include_bytes!("../../assets/models/Textures/roof.png"),
    ),
    (
        "Textures/wall.png",
        include_bytes!("../../assets/models/Textures/wall.png"),
    ),
    (
        "Textures/signs.png",
        include_bytes!("../../assets/models/Textures/signs.png"),
    ),
    (
        "Textures/treeA.png",
        include_bytes!("../../assets/models/Textures/treeA.png"),
    ),
];
pub const MODEL_COUNT: usize = 6;
pub const BENCH: usize = 0;
pub const DUMPSTER: usize = 1;
pub const BARRIER: usize = 2;
pub const PALLET: usize = 3;
pub const TRAFFIC: usize = 4;
pub const TREE: usize = 5;
pub const ROOF_LEDGE: f32 = 2.0;
pub const UNDERSIDE: f32 = 26.0;
pub const PROP_SCALE: f32 = 1.8;
pub const LANE_PROP_SCALE: f32 = 1.15;
pub const LANE_PROP_RADIUS: f32 = 0.75;

pub const BREAK_SPACING: f32 = 5.5;
pub const BREAK_CLUSTER: f32 = 1.05;
pub const PICKUP_REACH: f32 = 1.0;
pub const PICKUP_SPACING: f32 = 17.0;
pub const SHARD_SIZE: f32 = 0.2;
pub const SHARD_LIFE: f32 = 0.9;
pub const SHARD_RISE: f32 = 3.6;
pub const SHARD_SPREAD: f32 = 2.8;

pub const SWING_TIME: f32 = 0.3;
pub const SWING_MOMENT: f32 = 0.15;
pub const SWING_PUSH: f32 = 1.3;
pub const SWING_LUNGE: f32 = 3.4;
pub const FLING_SPEED: f32 = 13.0;
pub const FLING_RISE: f32 = 5.2;
pub const FLING_DAMAGE: f32 = 26.0;
pub const FLING_REACH: f32 = 0.85;
pub const THROW_LIFT: f32 = 1.1;
pub const THROW_REACH: f32 = 0.75;
pub const THROW_DAMAGE: f32 = 30.0;

pub const WOOD: &str = "wood";
pub const TILE_WALL: &str = "tile_wall";
pub const GRAVEL: &str = "gravel";
pub const RUST: &str = "rust";
pub const RAIL: &str = "rail";
pub const WOOD_TONES: [[u8; 4]; 3] = [[150, 106, 62, 255], [122, 84, 48, 255], [176, 130, 80, 255]];
pub const TILE_TONES: [[u8; 4]; 3] = [
    [196, 198, 190, 255],
    [168, 170, 164, 255],
    [214, 216, 210, 255],
];
pub const GRAVEL_TONES: [[u8; 4]; 3] = [[72, 70, 68, 255], [58, 56, 55, 255], [92, 90, 86, 255]];
pub const RUST_TONES: [[u8; 4]; 3] = [[112, 66, 44, 255], [88, 50, 34, 255], [138, 86, 58, 255]];
pub const RAIL_TONES: [[u8; 4]; 3] = [[58, 54, 50, 255], [44, 40, 38, 255], [86, 82, 76, 255]];

pub const SUBWAY_FOG: [f32; 3] = [0.03, 0.035, 0.04];
pub const ROOFTOP_FOG: [f32; 3] = [0.07, 0.08, 0.13];
pub const YARD_FOG: [f32; 3] = [0.06, 0.05, 0.05];
pub const ALLEY_FOG: [f32; 3] = [0.04, 0.04, 0.06];
pub const STRIP_COLOR: Vec3 = Vec3::new(0.82, 0.92, 1.0);
pub const STRIP_STRENGTH: f32 = 30.0;
pub const FLOOD_COLOR: Vec3 = Vec3::new(1.0, 0.92, 0.74);
pub const FLOOD_STRENGTH: f32 = 46.0;

pub const SCORE_SIZE: f32 = 22.0;
pub const STAGE_SIZE: f32 = 18.0;
pub const COMBO_SCORE: u32 = 25;
pub const CLEAR_SCORE: u32 = 500;
pub const CHIEF_SCORE: u32 = 1500;
pub const VICTORY_LINE: &str = "RAY: The block is quiet again. For tonight.";
pub const ARCADE_OVER: &str = "They kept coming. They always do.";
pub const DEAD_LINE: &str = "RAY: Not tonight.";
pub const CHIEF_LINE: &str = "RAY: There he is.";
