pub const ROCK: &str = "blackmarrow_stone_01";
pub const WALL: &str = "blackmarrow_brick_13";
pub const FLOOR: &str = "blackmarrow_floor_01";
pub const PLATE: &str = "blackmarrow_floor_09";
pub const CEILING: &str = "blackmarrow_wall_09";
pub const METAL: &str = "blackmarrow_metal_02";
pub const RUST: &str = "blackmarrow_metal_04";
pub const DOOR: &str = "blackmarrow_metal_14";
pub const DRAWER: &str = "blackmarrow_wall_12";
pub const GRILLE: &str = "blackmarrow_misc_10";

pub const TEXTURES: &[(&str, &[u8])] = &[
    (ROCK, include_bytes!("../../assets/textures/stone_01.png")),
    (WALL, include_bytes!("../../assets/textures/brick_13.png")),
    (FLOOR, include_bytes!("../../assets/textures/floor_01.png")),
    (PLATE, include_bytes!("../../assets/textures/floor_09.png")),
    (CEILING, include_bytes!("../../assets/textures/wall_09.png")),
    (METAL, include_bytes!("../../assets/textures/metal_02.png")),
    (RUST, include_bytes!("../../assets/textures/metal_04.png")),
    (DOOR, include_bytes!("../../assets/textures/metal_14.png")),
    (DRAWER, include_bytes!("../../assets/textures/wall_12.png")),
    (GRILLE, include_bytes!("../../assets/textures/misc_10.png")),
];

pub const WALL_TINT: [f32; 4] = [0.5, 0.52, 0.55, 1.0];
pub const ROCK_TINT: [f32; 4] = [0.52, 0.52, 0.54, 1.0];
pub const FLOOR_TINT: [f32; 4] = [0.5, 0.52, 0.5, 1.0];
pub const PLATE_TINT: [f32; 4] = [0.6, 0.6, 0.62, 1.0];
pub const CEILING_TINT: [f32; 4] = [0.4, 0.42, 0.46, 1.0];
pub const METAL_TINT: [f32; 4] = [0.5, 0.5, 0.52, 1.0];
pub const RUST_TINT: [f32; 4] = [0.55, 0.5, 0.45, 1.0];
pub const DOOR_TINT: [f32; 4] = [0.55, 0.55, 0.55, 1.0];
pub const DRAWER_TINT: [f32; 4] = [0.5, 0.45, 0.4, 1.0];
pub const GRILLE_TINT: [f32; 4] = [0.5, 0.5, 0.5, 1.0];
pub const KEY_TINT: [f32; 4] = [0.92, 0.82, 0.42, 1.0];

pub const SURFACE_TILING: f32 = 1.0;
pub const TILE: f32 = 1.0;
