use nalgebra_glm::Vec2;

pub const TITLE: &str = "Sprites";

pub const ATLAS: &str = "farm";
pub const SHEET: &str = "tiny_farm";
pub const GRASS: &str = "grass";
pub const ATLAS_SIZE: u32 = 256;

pub const SHEET_BYTES: &[u8] = include_bytes!("../assets/tiny_farm/tilemap_packed.png");
pub const SHEET_COLUMNS: u32 = 12;
pub const SHEET_ROWS: u32 = 11;
pub const TILE_TEXELS: u32 = 16;

pub const TILE: f32 = 1.0;
pub const TILE_SIZE: Vec2 = Vec2::new(TILE, TILE);
pub const PIXELS_PER_UNIT: f32 = 64.0;

pub const GRASS_COLOR: [u8; 4] = [132, 198, 105, 255];
pub const GRASS_BLADE_COLOR: [u8; 4] = [150, 212, 120, 255];

pub const GROUND_COLUMNS: u32 = 48;
pub const GROUND_ROWS: u32 = 30;
pub const GROUND_ORIGIN: Vec2 = Vec2::new(-24.0, -15.0);

pub const LAYER_GROUND: i32 = 0;
pub const LAYER_PATHS: i32 = 1;
pub const LAYER_STANDING: i32 = 2;

pub const BED_TOP: u32 = 0;
pub const BED_MIDDLE: u32 = 12;
pub const BED_MIDDLE_ALT: u32 = 13;
pub const BED_BOTTOM: u32 = 24;
pub const PATH_LEFT: u32 = 48;
pub const PATH_MIDDLE: u32 = 49;
pub const PATH_MIDDLE_ALT: u32 = 61;
pub const PATH_RIGHT: u32 = 50;
pub const PINE_TOP: u32 = 3;
pub const PINE_BOTTOM: u32 = 15;
pub const ROUND_TREE_TOP: u32 = 27;
pub const BUSH: u32 = 39;
pub const TUFTS: [u32; 5] = [4, 16, 28, 40, 52];
pub const CARROT: [u32; 4] = [5, 6, 7, 8];
pub const EGGPLANT: [u32; 4] = [17, 18, 19, 20];
pub const TOMATO: [u32; 4] = [41, 42, 43, 44];
pub const WHEAT: [u32; 5] = [64, 65, 66, 67, 68];
pub const SUNFLOWER: u32 = 83;
pub const ROCKS: u32 = 77;
pub const HAY: u32 = 96;
pub const BARREL: u32 = 85;
pub const FARMER: [u32; 2] = [108, 109];
pub const SHEEP: u32 = 120;
pub const COW: u32 = 121;
pub const CHICKEN: u32 = 122;

pub const BARN_COLUMNS: u32 = 3;
pub const BARN_ROWS: u32 = 7;
pub const BARN: [[u32; 3]; 7] = [
    [114, 115, 116],
    [102, 103, 104],
    [90, 91, 92],
    [129, 130, 131],
    [117, 118, 119],
    [105, 106, 107],
    [93, 94, 95],
];

pub const FARMER_SPEED: f32 = 4.0;
pub const WALK_FRAME_SECONDS: f32 = 0.16;
pub const GROW_FRAME_SECONDS: f32 = 1.2;
pub const CUBE_SPIN_SPEED: f32 = 0.9;

pub const CAMERA_FOLLOW: f32 = 6.0;

pub const SETTLED: u64 = 90;
