use nalgebra_glm::Vec3;

pub const SOLID_MATERIAL: &str = "voxels_solid";

pub const GLASS_ROUGHNESS: f32 = 0.08;

pub const WATER_RUN: u8 = 3;

pub const RIPPLE_TEXTURE: &str = "voxels_ripples";

pub const RIPPLE_TEXELS: u32 = 64;

pub const RIPPLE_HEIGHT: f32 = 0.09;

pub const RIPPLE_SCALE: f32 = 3.0;

pub const RIPPLE_DRIFT: [f32; 2] = [0.05, 0.031];

pub const WATER_DRIFT: [f32; 2] = [0.09, 0.055];

pub const RIPPLE_STRENGTH: f32 = 0.3;

pub const WATER_ROUGHNESS: f32 = 0.34;

pub const CRACK_INFLATE: f32 = 0.004;

pub const CARROT_FOOD: f32 = 4.0;

pub const WHEAT_FOOD: f32 = 3.0;

pub const GLASS_CELL: (u32, u32) = (9, 2);

pub const CHEST_TOP_CELL: (u32, u32) = (24, 0);

pub const CHEST_SIDE_CELL: (u32, u32) = (25, 0);

pub const WATER_COLOR: [f32; 4] = [0.30, 0.62, 0.78, 1.0];

pub const ITEM_DEPTH: f32 = 2.0;

pub const SHEET_BYTES: &[u8] = include_bytes!("../../../assets/tiny_farm/tilemap_packed.png");

pub const SPRITE_SHEET: &str = "tiny_farm";

pub const SHEET_COLUMNS: u32 = 12;

pub const SHEET_ROWS: u32 = 11;

pub const SHEET_TILES: usize = (SHEET_COLUMNS * SHEET_ROWS) as usize;

pub const CAMPFIRE_COLOR: Vec3 = Vec3::new(1.0, 0.68, 0.36);

pub const CAMPFIRE_STRENGTH: f32 = 18.0;

pub const CAMPFIRE_RANGE: f32 = 11.0;

pub const TORCH_COLOR: Vec3 = Vec3::new(1.0, 0.72, 0.42);

pub const TORCH_STRENGTH: f32 = 9.0;

pub const TORCH_RANGE: f32 = 13.0;

pub const FOOTING_RISE: i32 = 1;

pub const FOOTING_DROP: i32 = 6;

pub const STEP_RISE: i32 = 1;
