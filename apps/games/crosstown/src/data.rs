mod tables;

pub use tables::*;

use nalgebra_glm::Vec3;

pub const TITLE: &str = "Crosstown";
pub const SETTLED: u32 = 240;

pub const ATLAS: &str = "crosstown";
pub const ATLAS_SIZE: u32 = 2048;
pub const SHADOW: &str = "shadow";
pub const SHADOW_TEXELS: u32 = 32;
pub const PAGE_NAMES: [&str; 5] = ["player", "thug", "runner", "brute", "boss"];
pub const PAGE_BYTES: [&[u8]; 5] = [
    include_bytes!("../assets/toon/player.png"),
    include_bytes!("../assets/toon/thug.png"),
    include_bytes!("../assets/toon/runner.png"),
    include_bytes!("../assets/toon/brute.png"),
    include_bytes!("../assets/toon/boss.png"),
];
pub const PAGE_COLUMNS: u32 = 9;
pub const PAGE_ROWS: u32 = 5;
pub const HERO_PAGE: u32 = 0;

pub const POSE_IDLE: u32 = 0;
pub const POSE_JUMP: u32 = 1;
pub const POSE_FALL: u32 = 2;
pub const POSE_SLIDE: u32 = 10;
pub const POSE_AIR_KICK: u32 = 14;
pub const POSE_HURT: u32 = 33;
pub const POSE_RISE: u32 = 31;
pub const POSE_DOWN: u32 = 44;
pub const POSE_WALK: [u32; 8] = [36, 37, 38, 39, 40, 41, 42, 43];
pub const POSE_RUN: [u32; 3] = [24, 25, 26];
pub const POSE_HOLD: u32 = 21;
pub const POSE_SWING: u32 = 16;
pub const POSE_GRAB: u32 = 32;
pub const POSE_CHEER: u32 = 7;
pub const HERO_BLOWS: [u32; 3] = [27, 28, 19];
pub const THUG_WIND: u32 = 27;
pub const THUG_BLOW: u32 = 29;

pub const LAYER_SHADOW: i32 = -1;
pub const LAYER_FIGHTER: i32 = 0;
pub const SHADOW_ALPHA: f32 = 0.42;
pub const SHADOW_LIFT: f32 = 0.02;
pub const SHADOW_FORWARD: f32 = 0.42;
pub const SHADOW_SIZE: [f32; 2] = [0.95, 0.48];

pub const FIGHTER_HEIGHT: f32 = 1.85;
pub const FIGHTER_WIDTH: f32 = FIGHTER_HEIGHT * 0.75;
pub const STRIDE_RATE: f32 = 8.5;
pub const RUN_STRIDE_RATE: f32 = 13.0;

pub const STREET_START: f32 = -8.0;
pub const STREET_END: f32 = 48.0;
pub const ROAD_HALF: f32 = 3.0;
pub const LANE_HALF: f32 = 2.4;
pub const CURB_HEIGHT: f32 = 0.3;
pub const WALK_WIDTH: f32 = 2.5;
pub const BUILDING_DEPTH: f32 = 8.0;
pub const BUILDING_RUN: f32 = 5.0;
pub const BUILDING_LOW: f32 = 5.0;
pub const BUILDING_HIGH: f32 = 11.0;
pub const LAMP_SPACING: f32 = 9.0;
pub const LAMP_HEIGHT: f32 = 3.4;
pub const LAMP_COLOR: Vec3 = Vec3::new(1.0, 0.78, 0.44);
pub const LAMP_STRENGTH: f32 = 26.0;
pub const LAMP_RANGE: f32 = 13.0;
pub const MOON_COLOR: Vec3 = Vec3::new(0.42, 0.5, 0.72);
pub const MOON_STRENGTH: f32 = 1.1;

pub const FOG_COLOR: [f32; 3] = [0.05, 0.06, 0.10];
pub const FOG_START: f32 = 14.0;
pub const FOG_END: f32 = 46.0;
pub const CLEAR_COLOR: [f32; 4] = [0.03, 0.035, 0.06, 1.0];
pub const BLOOM: f32 = 0.09;

pub const CAMERA_BACK: f32 = 8.0;
pub const CAMERA_HEIGHT: f32 = 3.5;
pub const CAMERA_LOOK: f32 = 1.25;
pub const CAMERA_FOV: f32 = 36.0;
pub const CAMERA_NEAR: f32 = 0.1;
pub const CAMERA_FAR: f32 = 180.0;
pub const CAMERA_FOLLOW: f32 = 5.6;
pub const CAMERA_MARGIN: f32 = 7.0;
pub const CAMERA_SHAKE_FALL: f32 = 7.0;
pub const CAMERA_SHAKE_RATE: f32 = 52.0;

pub const HERO_HEALTH: f32 = 120.0;
pub const HERO_SPEED: f32 = 5.0;
pub const HERO_DEPTH_SPEED: f32 = 3.4;
pub const HERO_RUN: f32 = 8.6;
pub const HERO_RUN_DEPTH: f32 = 4.4;
pub const HERO_RADIUS: f32 = 0.42;

pub const BUFFER_TIME: f32 = 0.18;
pub const TAP_WINDOW: f32 = 0.26;
pub const MOVE_CANCEL: f32 = 0.1;
pub const AIR_CONTROL: f32 = 0.62;

pub const BLOW_TIME: f32 = 0.22;
pub const BLOW_MOMENT: f32 = 0.11;
pub const COMBO_TIME: f32 = 0.5;
pub const REACH: f32 = 1.15;
pub const LANE_REACH: f32 = 0.8;
pub const HERO_DAMAGE: [f32; 3] = [9.0, 10.0, 20.0];
pub const HERO_PUSH: [f32; 3] = [0.22, 0.28, 1.5];
pub const HERO_LUNGE: [f32; 3] = [2.6, 2.9, 5.4];
pub const JUMP_SPEED: f32 = 8.4;
pub const GRAVITY: f32 = 26.0;
pub const AIR_DAMAGE: f32 = 22.0;
pub const AIR_PUSH: f32 = 1.7;
pub const AIR_LUNGE: f32 = 4.4;
pub const DASH_DAMAGE: f32 = 26.0;
pub const DASH_PUSH: f32 = 2.8;
pub const DASH_LUNGE: f32 = 9.5;
pub const DASH_BLOW_TIME: f32 = 0.34;
pub const DASH_REACH: f32 = 1.35;
pub const GROUND_DRAG: f32 = 0.82;
pub const AIR_DRAG: f32 = 0.986;
pub const LAND_DRAG: f32 = 0.45;
pub const THUG_PUSH: f32 = 0.35;
pub const THUG_LUNGE: f32 = 2.2;
pub const THUG_APPROACH: f32 = 1.0;
pub const THUG_SPREAD: f32 = 1.5;
pub const THUG_STAGGER: f32 = 0.55;
pub const THUG_WIND_TIME: f32 = 0.26;
pub const HURT_TIME: f32 = 0.22;
pub const DOWN_TIME: f32 = 1.25;
pub const RISE_TIME: f32 = 0.45;
pub const HIT_STOP: f32 = 0.055;
pub const HIT_STOP_HEAVY: f32 = 0.11;
pub const SHAKE_HIT: f32 = 0.1;
pub const SHAKE_DOWN: f32 = 0.26;
pub const FLASH_TIME: f32 = 0.09;
pub const FLASH_GAIN: f32 = 2.4;

pub const BODY_RADIUS: f32 = 0.42;
pub const BODY_DEPTH: f32 = 0.36;
pub const KNOCK_RATE: f32 = 12.0;
pub const CROWD_PUSH: f32 = 0.55;

pub const SPAWN_MARGIN: f32 = 8.5;
pub const SPAWN_DELAY: f32 = 0.7;

pub const GO_BLINK: f32 = 0.22;
pub const GO_TIME: f32 = 3.2;

pub const ASPHALT: &str = "asphalt";
pub const PAINT: &str = "paint";
pub const CONCRETE: &str = "concrete";
pub const BRICK: &str = "brick";
pub const WINDOW_DARK: &str = "window_dark";
pub const WINDOW_LIT: &str = "window_lit";
pub const METAL: &str = "metal";
pub const LAMP_GLASS: &str = "lamp_glass";

pub const ASPHALT_TONES: [[u8; 4]; 3] = [[38, 39, 44, 255], [30, 31, 36, 255], [48, 49, 55, 255]];
pub const CONCRETE_TONES: [[u8; 4]; 3] = [
    [104, 104, 100, 255],
    [88, 88, 85, 255],
    [122, 122, 118, 255],
];
pub const PAINT_TONES: [[u8; 4]; 3] = [
    [186, 162, 62, 255],
    [160, 138, 52, 255],
    [206, 184, 84, 255],
];
pub const BRICK_COLOR: [u8; 4] = [96, 54, 46, 255];
pub const BRICK_ALT: [u8; 4] = [82, 46, 40, 255];
pub const BRICK_MORTAR: [u8; 4] = [64, 60, 58, 255];
pub const WINDOW_FRAME: [u8; 4] = [40, 38, 40, 255];
pub const WINDOW_COLD: [u8; 4] = [26, 30, 40, 255];
pub const WINDOW_WARM: [u8; 4] = [236, 206, 148, 255];
pub const METAL_TONES: [[u8; 4]; 3] = [[54, 56, 60, 255], [44, 46, 50, 255], [70, 72, 78, 255]];
pub const WINDOW_GLOW: [f32; 3] = [1.15, 0.82, 0.44];
pub const LAMP_GLOW: [f32; 3] = [3.2, 2.3, 1.2];

pub const TEXEL: u32 = 16;

pub const TAGLINE: &str = "The Ironmouth crew took the block. Take it back.";
pub const HERO_NAME: &str = "RAY";
pub const LINE_TIME: f32 = 3.0;
pub const OPENING: [&str; 2] = [
    "IRONMOUTH: Wrong street, old man.",
    "RAY: It was my street first.",
];
pub const TAUNTS: [&str; 6] = [
    "IRONMOUTH: Hold him!",
    "IRONMOUTH: That all you got?",
    "IRONMOUTH: Break his arms!",
    "IRONMOUTH: Come on, come on!",
    "IRONMOUTH: You are bleeding, old man.",
    "IRONMOUTH: The boss wants him quiet.",
];
pub const DOWNED: [&str; 4] = [
    "RAY: Stay down.",
    "RAY: Next.",
    "RAY: Told you.",
    "RAY: Who is next?",
];
pub const CLEARED_LINE: &str = "RAY: One block down. The rest of the street is waiting.";
pub const HURT_LINE: &str = "RAY: Still standing.";
pub const CAPTION_SIZE: f32 = 21.0;
pub const TITLE_SIZE: f32 = 74.0;
pub const GO_SIZE: f32 = 88.0;
pub const BAR_WIDTH: f32 = 306.0;
pub const BAR_HEIGHT: f32 = 26.0;
pub const BAR_MARGIN: f32 = 18.0;
pub const PANEL_BACK: [f32; 4] = [0.04, 0.04, 0.06, 0.86];
pub const PANEL_EDGE: [f32; 4] = [1.0, 1.0, 1.0, 0.12];
pub const CLEAR_EDGE: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
pub const HERO_BAR_COLOR: [f32; 4] = [0.36, 0.82, 0.42, 1.0];
pub const HERO_BAR_LOW: [f32; 4] = [0.92, 0.32, 0.28, 1.0];
pub const THUG_BAR_COLOR: [f32; 4] = [0.92, 0.42, 0.32, 1.0];
pub const BOSS_BAR_COLOR: [f32; 4] = [0.98, 0.26, 0.2, 1.0];
pub const GO_COLOR: [f32; 4] = [1.0, 0.88, 0.28, 1.0];
pub const TEXT_COLOR: [f32; 4] = [0.88, 0.89, 0.94, 1.0];
pub const DIM_COLOR: [f32; 4] = [0.62, 0.64, 0.72, 1.0];
pub const LOW_HEALTH: f32 = 0.3;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Walk,
    WalkAcross,
    WalkAway,
    TapLeft,
    TapRight,
    Punch,
    Jump,
    Throw,
    Pause,
}

impl nightshade_input::Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::Walk,
            Self::WalkAcross,
            Self::WalkAway,
            Self::TapLeft,
            Self::TapRight,
            Self::Punch,
            Self::Jump,
            Self::Throw,
            Self::Pause,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Walk => "walk",
            Self::WalkAcross => "walk across",
            Self::WalkAway => "walk away",
            Self::TapLeft => "tap left",
            Self::TapRight => "tap right",
            Self::Punch => "punch",
            Self::Jump => "jump",
            Self::Throw => "throw",
            Self::Pause => "pause",
        }
    }

    fn default_bindings(&self) -> &'static [nightshade_input::Binding] {
        use nightshade_gamepad::contract::GamepadButton;
        use nightshade_input::{Binding, Press, Stick};
        use nightshade_platform::KeyCode;
        match self {
            Self::Walk => &[
                Binding::Keys {
                    up: KeyCode::KeyW,
                    down: KeyCode::KeyS,
                    left: KeyCode::KeyA,
                    right: KeyCode::KeyD,
                },
                Binding::Keys {
                    up: KeyCode::ArrowUp,
                    down: KeyCode::ArrowDown,
                    left: KeyCode::ArrowLeft,
                    right: KeyCode::ArrowRight,
                },
                Binding::Stick(Stick::Left),
            ],
            Self::WalkAcross => &[Binding::Pair {
                positive: Press::Pad(GamepadButton::DPadRight),
                negative: Press::Pad(GamepadButton::DPadLeft),
            }],
            Self::WalkAway => &[Binding::Pair {
                positive: Press::Pad(GamepadButton::DPadUp),
                negative: Press::Pad(GamepadButton::DPadDown),
            }],
            Self::TapLeft => &[
                Binding::Key(KeyCode::KeyA),
                Binding::Key(KeyCode::ArrowLeft),
                Binding::Pad(GamepadButton::DPadLeft),
            ],
            Self::TapRight => &[
                Binding::Key(KeyCode::KeyD),
                Binding::Key(KeyCode::ArrowRight),
                Binding::Pad(GamepadButton::DPadRight),
            ],
            Self::Punch => &[
                Binding::Key(KeyCode::KeyJ),
                Binding::Key(KeyCode::KeyZ),
                Binding::Pad(GamepadButton::South),
                Binding::Pad(GamepadButton::West),
            ],
            Self::Jump => &[
                Binding::Key(KeyCode::Space),
                Binding::Key(KeyCode::KeyK),
                Binding::Key(KeyCode::KeyX),
                Binding::Pad(GamepadButton::East),
                Binding::Pad(GamepadButton::North),
            ],
            Self::Throw => &[
                Binding::Key(KeyCode::KeyL),
                Binding::Key(KeyCode::KeyC),
                Binding::Pad(GamepadButton::RightBumper),
                Binding::Pad(GamepadButton::LeftBumper),
            ],
            Self::Pause => &[
                Binding::Key(KeyCode::Escape),
                Binding::Pad(GamepadButton::Start),
            ],
        }
    }
}
