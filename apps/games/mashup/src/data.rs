use nightshade::prelude::Vec3;

pub const TITLE: &str = "Mashup";
pub const PORTAL_PASS: &str = "mashup_portal_surface";

pub const SPRITE_ATLAS: &str = "mashup_sprites";
pub const SPRITE_ATLAS_SIZE: u32 = 256;
pub const BLOCK_ATLAS_SIZE: u32 = 64;
pub const TILE_TEXELS: u32 = 16;

pub const QUARRY_ORIGIN: Vec3 = Vec3::new(512.0, 0.0, 0.0);

pub const MEADOW_HALF: f32 = 18.0;
pub const MEADOW_SKIRT: f32 = 150.0;
pub const FOG_COLOR: [f32; 3] = [0.70, 0.78, 0.86];
pub const FOG_START: f32 = 46.0;
pub const FOG_END: f32 = 120.0;
pub const MEADOW_GROUND: f32 = 0.0;
pub const MEADOW_START: [f32; 3] = [0.0, 0.0, 4.0];
pub const MEADOW_SPEED: f32 = 5.2;

pub const CAMERA_HEIGHT: f32 = 11.0;
pub const CAMERA_REACH: f32 = 15.0;
pub const CAMERA_FOLLOW: f32 = 6.0;
pub const CAMERA_TURN: f32 = 9.0;
pub const MEADOW_FIELD_OF_VIEW: f32 = 0.62;

pub const QUARRY_SPAN: i32 = 26;
pub const QUARRY_BASE: i32 = 6;
pub const QUARRY_RELIEF: f32 = 3.4;
pub const QUARRY_START: [f32; 3] = [512.0, 0.0, 8.0];
pub const QUARRY_SPEED: f32 = 5.6;
pub const QUARRY_GRAVITY: f32 = 22.0;
pub const QUARRY_JUMP: f32 = 8.2;
pub const QUARRY_STEP: f32 = 1.05;
pub const QUARRY_RADIUS: f32 = 0.32;
pub const EYE_HEIGHT: f32 = 1.62;
pub const TERMINAL_FALL: f32 = 40.0;
pub const STEP_LIMIT: f32 = 0.2;
pub const MAX_SLICES: u32 = 24;

pub const MOUSE_SENSITIVITY: f32 = 0.0022;
pub const PITCH_LIMIT: f32 = 1.5;
pub const STICK_LOOK_SPEED: f32 = 2.6;

pub const FIELD_OF_VIEW: f32 = 1.2;
pub const NEAR_PLANE: f32 = 0.05;
pub const FAR_PLANE: f32 = 240.0;
pub const PLANE_BIAS: f32 = 0.05;

pub const PORTAL_WIDTH: f32 = 2.4;
pub const PORTAL_HEIGHT: f32 = 3.4;
pub const PORTAL_LIFT: f32 = 1.9;
pub const PORTAL_TARGET_WIDTH: u32 = 1280;
pub const PORTAL_TARGET_HEIGHT: u32 = 720;
pub const PORTAL_TARGET_MAX: u32 = 1600;
pub const PORTAL_REACH: f32 = 2.6;

pub const MEADOW_RIM: [f32; 4] = [0.34, 0.74, 1.0, 1.0];
pub const QUARRY_RIM: [f32; 4] = [1.0, 0.62, 0.20, 1.0];

pub const DIVE_SECONDS: f32 = 1.15;
pub const SETTLE_SLOWEST: f32 = 1.4;
pub const SETTLE_QUICKEST: f32 = 0.45;
pub const SETTLE_SPEED: f32 = 22.0;
pub const DIVE_STANDOFF: f32 = 0.22;
pub const CROSS_HOLD: f32 = 0.9;

pub const SPARKS_PER_RIM: usize = 14;
pub const SPARK_RATE: f32 = 30.0;
pub const SPARK_LIFE: [f32; 2] = [0.18, 0.52];
pub const SPARK_SPEED: [f32; 2] = [0.4, 1.9];
pub const SPARK_SPREAD: f32 = 0.85;
pub const SPARK_SIZE: [f32; 2] = [0.13, 0.0];
pub const SPARK_GRAVITY: f32 = -2.4;
pub const SPARK_DRAG: f32 = 1.4;
pub const SPARK_GLOW: f32 = 1.7;
pub const SPARK_TURNS: f32 = 0.16;
pub const SPARK_STAND_OFF: f32 = 0.06;
pub const SPARK_OUTWARD: f32 = 0.42;

pub const MENU_WIDTH: f32 = 340.0;
pub const MENU_HEIGHT: f32 = 198.0;

pub const SHADOW_ALPHA: f32 = 0.34;
pub const SHADOW_LIFT: f32 = 0.03;
pub const VILLAGER_SPEED: f32 = 1.1;
pub const VILLAGER_REACH: f32 = 3.4;

pub struct Slab {
    pub center: [f32; 3],
    pub size: [f32; 3],
    pub color: [f32; 4],
}

pub fn art_color(mark: char) -> Option<[f32; 3]> {
    match mark {
        'k' => Some([0.12, 0.10, 0.14]),
        'g' => Some([0.20, 0.52, 0.24]),
        'G' => Some([0.31, 0.69, 0.33]),
        'w' => Some([0.45, 0.31, 0.19]),
        'W' => Some([0.57, 0.41, 0.26]),
        's' => Some([0.95, 0.78, 0.62]),
        'h' => Some([0.32, 0.20, 0.12]),
        'b' => Some([0.24, 0.38, 0.72]),
        'r' => Some([0.78, 0.24, 0.22]),
        'y' => Some([0.93, 0.81, 0.33]),
        'p' => Some([0.92, 0.52, 0.68]),
        _ => None,
    }
}

pub const TREE_ART: [&str; 24] = [
    "      GGGG      ",
    "    GGGGGGGG    ",
    "   GGGGGGGGGG   ",
    "  GGGGggggGGGG  ",
    "  GGGggggggGGG  ",
    " GGGggggggggGGG ",
    " GGgggggggggGGG ",
    " GGggggggggggGG ",
    " GGggggggggggGG ",
    " GGgggggggggGGG ",
    "  GGgggggggGGG  ",
    "  GGGggggggGG   ",
    "   GGGGGGGGG    ",
    "    GGGGGGG     ",
    "      wwww      ",
    "      wwww      ",
    "      wWww      ",
    "      wwww      ",
    "      wwww      ",
    "      wWww      ",
    "      wwww      ",
    "     wwwwww     ",
    "    wwwwwwww    ",
    "   wwwwwwwwww   ",
];

pub const VILLAGER_ART: [&str; 20] = [
    "     hhhhhh     ",
    "    hhhhhhhh    ",
    "    hssssssh    ",
    "    skssssks    ",
    "    hssssssh    ",
    "     ssssss     ",
    "    bbbbbbbb    ",
    "   bbbbbbbbbb   ",
    "   bsbbbbbbsb   ",
    "   bsbbbbbbsb   ",
    "   bbbbbbbbbb   ",
    "    bbbbbbbb    ",
    "    bbbbbbbb    ",
    "    ww    ww    ",
    "    ww    ww    ",
    "    ww    ww    ",
    "    ww    ww    ",
    "   kkk    kkk   ",
    "   kkk    kkk   ",
    "                ",
];

pub const FARMER_ART: [&str; 20] = [
    "    yyyyyyyy    ",
    "   yyyyyyyyyy   ",
    "     hhhhhh     ",
    "    hssssssh    ",
    "    skssssks    ",
    "     ssssss     ",
    "    rrrrrrrr    ",
    "   rrrrrrrrrr   ",
    "   rsrrrrrrsr   ",
    "   rsrrrrrrsr   ",
    "   rrrrrrrrrr   ",
    "    rrrrrrrr    ",
    "    rrrrrrrr    ",
    "    ww    ww    ",
    "    ww    ww    ",
    "    ww    ww    ",
    "    ww    ww    ",
    "   kkk    kkk   ",
    "   kkk    kkk   ",
    "                ",
];

pub const FLOWER_ART: [&str; 12] = [
    "                ",
    "      pp        ",
    "     pppp       ",
    "    pppypp      ",
    "    pppypp      ",
    "     pppp       ",
    "      gg        ",
    "      gg        ",
    "     ggg        ",
    "      gg        ",
    "      gg        ",
    "                ",
];

const MEADOW_GRASS: [f32; 4] = [0.32, 0.55, 0.28, 1.0];
const MEADOW_PATH: [f32; 4] = [0.62, 0.54, 0.38, 1.0];
const MEADOW_STONE: [f32; 4] = [0.58, 0.58, 0.62, 1.0];
const MEADOW_PLASTER: [f32; 4] = [0.88, 0.83, 0.72, 1.0];
const MEADOW_ROOF: [f32; 4] = [0.62, 0.28, 0.24, 1.0];
const MEADOW_WATER: [f32; 4] = [0.26, 0.48, 0.66, 1.0];

pub fn meadow_blocks() -> Vec<Slab> {
    let span = MEADOW_HALF * 2.0;
    let mut blocks = vec![
        Slab {
            center: [0.0, -1.0, 0.0],
            size: [MEADOW_SKIRT, 1.2, MEADOW_SKIRT],
            color: MEADOW_GRASS,
        },
        Slab {
            center: [0.0, -0.4, 0.0],
            size: [span, 0.8, span],
            color: MEADOW_GRASS,
        },
        Slab {
            center: [0.0, 0.02, -1.0],
            size: [3.0, 0.06, 20.0],
            color: MEADOW_PATH,
        },
        Slab {
            center: [-12.5, 0.02, 6.0],
            size: [7.0, 0.08, 7.0],
            color: MEADOW_WATER,
        },
    ];
    for (x, z) in [(-9.0_f32, -6.0_f32), (9.0, -4.0), (11.0, 7.0)] {
        blocks.push(Slab {
            center: [x, 1.4, z],
            size: [4.2, 2.8, 3.6],
            color: MEADOW_PLASTER,
        });
        blocks.push(Slab {
            center: [x, 3.1, z],
            size: [4.8, 0.7, 4.2],
            color: MEADOW_ROOF,
        });
    }
    for side in [-1.0_f32, 1.0] {
        blocks.push(Slab {
            center: [side * 2.0, PORTAL_LIFT, -10.0],
            size: [0.8, PORTAL_HEIGHT + 1.2, 0.8],
            color: MEADOW_STONE,
        });
    }
    blocks.push(Slab {
        center: [0.0, PORTAL_LIFT + PORTAL_HEIGHT * 0.5 + 0.4, -10.0],
        size: [4.8, 0.8, 0.8],
        color: MEADOW_STONE,
    });
    blocks
}

pub fn meadow_trees() -> Vec<[f32; 2]> {
    vec![
        [-14.0, -12.0],
        [-6.0, -13.5],
        [5.0, -13.0],
        [13.5, -11.0],
        [-15.5, 0.5],
        [15.0, 1.5],
        [-13.0, 12.5],
        [-4.0, 14.0],
        [4.5, 13.0],
        [14.0, 13.5],
    ]
}

pub fn meadow_flowers() -> Vec<[f32; 2]> {
    vec![
        [-3.4, 3.2],
        [3.8, 4.6],
        [-6.2, -2.4],
        [6.4, -1.8],
        [-2.2, -6.6],
        [2.6, 9.4],
        [8.2, 3.4],
        [-8.6, 8.2],
    ]
}

pub fn meadow_villagers() -> Vec<[f32; 2]> {
    vec![[-6.5, 2.5], [7.0, 5.5], [1.5, -5.5]]
}

pub fn column_height(x: i32, z: i32) -> i32 {
    let fx = x as f32;
    let fz = z as f32;
    let rolling = (fx * 0.21).sin() * 1.7 + (fz * 0.17).cos() * 1.5;
    let ridge = ((fx + fz) * 0.09).sin() * 1.9;
    let grain = hash_unit(x, z) * 0.9;
    let height = QUARRY_BASE as f32 + (rolling + ridge) * (QUARRY_RELIEF / 3.6) + grain;
    (height.round() as i32).clamp(2, QUARRY_BASE + 8)
}

pub fn hash_unit(x: i32, z: i32) -> f32 {
    let mut seed = (x as u32).wrapping_mul(0x9e3779b9) ^ (z as u32).wrapping_mul(0x85ebca6b);
    seed ^= seed >> 15;
    seed = seed.wrapping_mul(0x2c1b3c6d);
    seed ^= seed >> 12;
    seed = seed.wrapping_mul(0x297a2d39);
    seed ^= seed >> 15;
    (seed & 0xffff) as f32 / 65535.0
}

pub fn quarry_trees() -> Vec<[i32; 2]> {
    vec![
        [-14, -10],
        [-7, 4],
        [3, -12],
        [9, 6],
        [14, -3],
        [-3, 12],
        [11, 13],
        [-16, 8],
    ]
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Walk,
    Look,
    Jump,
    TurnLeft,
    TurnRight,
    Pause,
}

impl nightshade_input::Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::Walk,
            Self::Look,
            Self::Jump,
            Self::TurnLeft,
            Self::TurnRight,
            Self::Pause,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Walk => "walk",
            Self::Look => "look",
            Self::Jump => "jump",
            Self::TurnLeft => "turn left",
            Self::TurnRight => "turn right",
            Self::Pause => "pause",
        }
    }

    fn default_bindings(&self) -> &'static [nightshade_input::Binding] {
        use nightshade_gamepad::contract::GamepadButton;
        use nightshade_input::{Binding, Stick};
        use nightshade_platform::KeyCode;
        match self {
            Self::Walk => &[
                Binding::Keys {
                    up: KeyCode::KeyW,
                    down: KeyCode::KeyS,
                    left: KeyCode::KeyA,
                    right: KeyCode::KeyD,
                },
                Binding::Stick(Stick::Left),
            ],
            Self::Look => &[Binding::Stick(Stick::Right)],
            Self::Jump => &[
                Binding::Key(KeyCode::Space),
                Binding::Pad(GamepadButton::South),
            ],
            Self::TurnLeft => &[
                Binding::Key(KeyCode::KeyQ),
                Binding::Pad(GamepadButton::LeftBumper),
            ],
            Self::TurnRight => &[
                Binding::Key(KeyCode::KeyE),
                Binding::Pad(GamepadButton::RightBumper),
            ],
            Self::Pause => &[Binding::Key(KeyCode::Escape)],
        }
    }
}
