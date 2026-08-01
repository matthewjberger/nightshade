use nalgebra_glm::{Vec3, Vec4};
pub use nightshade_voxels::data::CHUNK;

pub const TITLE: &str = "Demolition";
pub const SETTLED: u64 = 600;

pub const ATLAS_TEXTURE: &str = "demolition_atlas";
pub const ATLAS_COLUMNS: u32 = 4;
pub const ATLAS_ROWS: u32 = 2;
pub const TILE_TEXELS: u32 = 16;
pub const ATLAS_CELLS: [f32; 2] = [ATLAS_COLUMNS as f32, ATLAS_ROWS as f32];

pub const SOLID_MATERIAL: &str = "demolition_solid";
pub const CUTOUT_MATERIAL: &str = "demolition_cutout";
pub const AIM_MATERIAL: &str = "demolition_aim";
pub const AIM_MESH: &str = "Sphere";
pub const AIM_COLOR: [f32; 4] = [1.0, 0.32, 0.12, 0.42];
pub const AIM_GLOW: [f32; 3] = [0.6, 0.12, 0.02];
pub const AIM_PULSE: f32 = 0.05;
pub const AIM_PULSE_RATE: f32 = 5.0;

pub const BLOCK_SIZE: f32 = 0.125;
pub const GROUND_DEPTH: i32 = 3;
pub const WIDTH: i32 = 96;
pub const DEPTH: i32 = 64;
pub const STOREYS: i32 = 5;
pub const STOREY: i32 = 24;
pub const HEIGHT: i32 = STOREYS * STOREY;
pub const WALL: i32 = 2;
pub const PILLAR: i32 = 2;
pub const PILLAR_SPACING: i32 = 16;
pub const WINDOW_LOW: i32 = 8;
pub const WINDOW_HIGH: i32 = 18;
pub const WINDOW_SPAN: i32 = 12;
pub const WINDOW_GLASS: i32 = 6;
pub const FRAME_TEXELS: u32 = 2;

pub const BLAST_RADIUS: i32 = 7;
pub const BLAST_RADIUS_MIN: i32 = 2;
pub const BLAST_RADIUS_MAX: i32 = 16;
pub const REACH: f32 = 60.0;

pub const GROUND_TILE: [u8; 3] = [98, 110, 74];
pub const CONCRETE_TILE: [u8; 3] = [178, 178, 172];
pub const BRICK_TILE: [u8; 3] = [164, 84, 62];
pub const WOOD_TILE: [u8; 3] = [152, 112, 62];
pub const GLASS_TILE: [u8; 3] = [200, 236, 244];
pub const ROOF_TILE: [u8; 3] = [72, 72, 84];
pub const SPECKLE: i32 = 12;

pub const EYE: Vec3 = Vec3::new(6.0, 8.5, 30.0);
pub const YAW: f32 = 0.0;
pub const PITCH: f32 = -0.12;
pub const PITCH_LIMIT: f32 = 1.55;
pub const MOUSE_SENSITIVITY: f32 = 0.0022;
pub const FLY_SPEED: f32 = 8.0;
pub const SPRINT_FACTOR: f32 = 3.0;
pub const FLOOR_HEIGHT: f32 = 0.4;
pub const CAMERA_FOV_DEGREES: f32 = 60.0;
pub const CAMERA_NEAR: f32 = 0.05;
pub const START_HOUR: f32 = 10.5;

pub const PANEL_COLOR: Vec4 = Vec4::new(0.07, 0.06, 0.09, 0.9);
pub const PANEL_BORDER: Vec4 = Vec4::new(1.0, 0.45, 0.25, 0.9);
pub const TEXT_COLOR: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);
pub const DIM_TEXT_COLOR: Vec4 = Vec4::new(0.75, 0.7, 0.66, 1.0);
pub const ACCENT_COLOR: Vec4 = Vec4::new(1.0, 0.6, 0.35, 1.0);
pub const CROSSHAIR_COLOR: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.9);
pub const CROSSHAIR_LENGTH: f32 = 18.0;
pub const CROSSHAIR_GAP: f32 = 6.0;
pub const CROSSHAIR_THICKNESS: f32 = 2.0;
pub const TITLE_SIZE: f32 = 44.0;
pub const BODY_SIZE: f32 = 18.0;
pub const SMALL_SIZE: f32 = 15.0;
pub const MARGIN: f32 = 14.0;
pub const CARD_WIDTH: f32 = 280.0;
pub const CARD_HEIGHT: f32 = 40.0;
pub const TITLE_WIDTH: f32 = 560.0;
pub const TITLE_HEIGHT: f32 = 150.0;
