pub const TILE_TEXELS: u32 = 16;

pub const KENNEY_ATLAS_BYTES: &[u8] = include_bytes!("../../../../assets/kenney_atlas.png");

pub const ATLAS_TEXTURE: &str = "voxels_block_atlas";

pub const ATLAS_COLUMNS: u32 = 32;

pub const ATLAS_ROWS: u32 = 16;

pub const ATLAS_CELLS: [f32; 2] = [ATLAS_COLUMNS as f32, ATLAS_ROWS as f32];

pub const CUTOUT_MATERIAL: &str = "voxels_cutout";

pub const TRANSPARENT_MATERIAL: &str = "voxels_transparent";

pub const WATER_MATERIAL: &str = "voxels_water";

pub const FLAT_MATERIAL: &str = "voxels_flat";

pub const CRACK_MESH: &str = "voxels_crack";

pub const SPRITE_ATLAS: &str = "voxels_sprites";

pub const SPRITE_ATLAS_SIZE: u32 = 512;
