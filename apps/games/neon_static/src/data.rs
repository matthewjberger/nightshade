use nalgebra_glm::{Vec2, Vec4};

pub const TITLE: &str = "Neon Static";

pub const ATLAS: &str = "neon";
pub const ATLAS_SIZE: u32 = 2048;
pub const TILE_SHEET: &str = "tiles";
pub const CHARACTER_SHEET: &str = "characters";

pub const TILE_SHEET_BYTES: &[u8] =
    include_bytes!("../assets/kenney_top_down_shooter/tilesheet_complete.png");
pub const CHARACTER_SHEET_BYTES: &[u8] =
    include_bytes!("../assets/kenney_top_down_shooter/spritesheet_characters.png");

pub const TILE_SHEET_COLUMNS: u32 = 27;
pub const TILE_SHEET_ROWS: u32 = 20;
pub const CHARACTER_SHEET_WIDTH: f32 = 512.0;
pub const CHARACTER_SHEET_HEIGHT: f32 = 256.0;

pub const TILE: f32 = 1.0;
pub const TILE_SIZE: Vec2 = Vec2::new(TILE, TILE);
pub const TEXELS_PER_TILE: f32 = 64.0;
pub const PIXELS_PER_UNIT: f32 = 96.0;

pub const FLOOR_TILE: u32 = 10;
pub const FLOOR_TILE_MARKED: u32 = 8;
pub const WALL_TILE: u32 = 12;
pub const WALL_TILE_ALTERNATE: u32 = 13;
pub const BLOOD_TILE: u32 = 185;
pub const SHARD_TILE: u32 = 290;
pub const WHITE_TILE: u32 = 11;

pub const LAYER_FLOOR: i32 = 0;
pub const LAYER_WALL: i32 = 1;
pub const LAYER_BLOOD: i32 = 2;
pub const LAYER_BODY: i32 = 3;
pub const LAYER_BULLET: i32 = 4;

pub const FLOOR_TINT: [f32; 4] = [0.075, 0.045, 0.16, 1.0];
pub const WALL_TINT: [f32; 4] = [1.0, 0.44, 0.30, 1.0];
pub const BLOOD_TINT: [f32; 4] = [0.98, 0.13, 0.20, 0.92];
pub const BULLET_TINT: [f32; 4] = [1.0, 0.92, 0.42, 1.0];
pub const SHELL_TINT: [f32; 4] = [1.0, 0.78, 0.30, 1.0];

pub const PANEL_BACKGROUND: Vec4 = Vec4::new(0.04, 0.02, 0.08, 0.82);
pub const PANEL_BORDER: Vec4 = Vec4::new(1.0, 0.36, 0.52, 0.55);
pub const TEXT_PRIMARY: Vec4 = Vec4::new(1.0, 0.93, 0.78, 1.0);
pub const TEXT_CLEARED: Vec4 = Vec4::new(0.55, 1.0, 0.72, 1.0);
pub const TEXT_DEAD: Vec4 = Vec4::new(1.0, 0.36, 0.44, 1.0);
pub const FLASH_COLOR: Vec4 = Vec4::new(0.85, 0.05, 0.18, 1.0);

pub const READOUT_WIDTH: f32 = 420.0;
pub const READOUT_HEIGHT: f32 = 54.0;
pub const BANNER_WIDTH: f32 = 620.0;
pub const BANNER_HEIGHT: f32 = 96.0;
pub const READOUT_TEXT_SIZE: f32 = 24.0;
pub const BANNER_TEXT_SIZE: f32 = 44.0;
pub const MARGIN: f32 = 18.0;

pub const PLAYER_SPEED: f32 = 6.6;
pub const ENEMY_SPEED: f32 = 3.3;
pub const ENEMY_SIGHT: f32 = 26.0;
pub const BULLET_SPEED: f32 = 27.0;
pub const BULLET_LIFE: f32 = 1.5;
pub const BULLET_SIZE: Vec2 = Vec2::new(0.42, 0.10);
pub const FIRE_INTERVAL: f32 = 0.14;
pub const BODY_RADIUS: f32 = 0.26;
pub const HIT_RADIUS: f32 = 0.34;
pub const TOUCH_RADIUS: f32 = 0.52;
pub const BLOOD_SIZE: Vec2 = Vec2::new(1.3, 1.3);
pub const SHELL_SIZE: Vec2 = Vec2::new(0.36, 0.36);
pub const SHELL_LIFE: f32 = 6.0;
pub const SHELL_TOSS: f32 = 2.6;
pub const SHELL_DRIFT: f32 = 1.2;
pub const SHELL_SPIN: f32 = 12.0;
pub const SHELL_DRAG: f32 = 7.0;
pub const MUZZLE_REACH: f32 = 0.55;

pub const RESTART_DELAY: f32 = 0.55;
pub const SHAKE_FIRE: f32 = 0.12;
pub const SHAKE_KILL: f32 = 0.34;
pub const SHAKE_DEATH: f32 = 0.9;
pub const SHAKE_DECAY: f32 = 5.5;
pub const SHAKE_REACH: f32 = 0.32;
pub const FLASH_DECAY: f32 = 2.6;
pub const CAMERA_FOLLOW: f32 = 9.0;
pub const CAMERA_LEAN: f32 = 0.22;
pub const CAMERA_LEAN_LIMIT: f32 = 2.6;

pub const CHARACTER_HEIGHT: f32 = 43.0;
pub const CHARACTER_BODY_CENTER: f32 = 15.0;

pub struct CharacterPose {
    pub x: f32,
    pub y: f32,
    pub width: f32,
}

pub const PLAYER_POSE: CharacterPose = CharacterPose {
    x: 164.0,
    y: 88.0,
    width: 49.0,
};

pub const ENEMY_POSES: [CharacterPose; 4] = [
    CharacterPose {
        x: 426.0,
        y: 132.0,
        width: 33.0,
    },
    CharacterPose {
        x: 459.0,
        y: 44.0,
        width: 33.0,
    },
    CharacterPose {
        x: 390.0,
        y: 176.0,
        width: 35.0,
    },
    CharacterPose {
        x: 460.0,
        y: 132.0,
        width: 33.0,
    },
];

pub const WALL: u8 = b'#';
pub const START: u8 = b'@';
pub const SPAWN: u8 = b'e';

pub const FLOOR_PLAN: &[&str] = &[
    "############################",
    "#........#.................#",
    "#........#....e......e.....#",
    "#...@....#.................#",
    "#........#....#########....#",
    "#........#....#.......#....#",
    "#........#....#...e...#....#",
    "####.#####....#.......#....#",
    "#.....................#....#",
    "#....e...#....#########....#",
    "#........#.................#",
    "#........#.....e...........#",
    "#........#.................#",
    "############################",
];

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Walk,
    Fire,
    Restart,
}

impl nightshade_input::Action for Ask {
    fn all() -> &'static [Self] {
        &[Self::Walk, Self::Fire, Self::Restart]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Walk => "walk",
            Self::Fire => "fire",
            Self::Restart => "restart",
        }
    }

    fn default_bindings(&self) -> &'static [nightshade_input::Binding] {
        use nightshade_input::{Binding, Pointer};
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
            ],
            Self::Fire => &[
                Binding::Pointer(Pointer::Left),
                Binding::Key(KeyCode::Space),
            ],
            Self::Restart => &[Binding::Key(KeyCode::KeyR)],
        }
    }
}
