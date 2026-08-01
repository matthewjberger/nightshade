use crate::resources::DecalType;
use nalgebra_glm::Vec3;

pub const HELMET_GLTF: &[u8] = include_bytes!("../../../../assets/gltf/DamagedHelmet.glb");
pub const HDR_SKYBOX: &[u8] = include_bytes!("../../../../assets/sky/moonrise.hdr");
pub const AWESOMEFACE_PNG: &[u8] = include_bytes!("../../../../assets/textures/awesomeface.png");

pub const AWESOMEFACE_TEXTURE: &str = "awesomeface";
pub const CHECKERBOARD_TEXTURE: &str = "checkerboard";

pub const HELP_TEXT: &str =
    "Left Click: Place Decal | Tab: Cycle Type | R: Random | C: Clear | H: Help";

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 3.0, 0.0);
pub const CAMERA_DISTANCE: f32 = 6.0;
pub const CAMERA_YAW: f32 = 0.5;
pub const CAMERA_PITCH: f32 = 0.3;

pub const SUN_INTENSITY: f32 = 3.0;

pub const BLOOM_INTENSITY: f32 = 0.05;
pub const DEFAULT_DECAL_SIZE: f32 = 1.0;
pub const DEFAULT_EMISSIVE_STRENGTH: f32 = 1.5;

pub const PANEL_WIDTH: f32 = 240.0;
pub const PANEL_PADDING: f32 = 8.0;
pub const PANEL_SPACING: f32 = 6.0;

pub const DECAL_SIZE_RANGE: (f32, f32) = (0.2, 5.0);
pub const EMISSIVE_STRENGTH_RANGE: (f32, f32) = (0.0, 10.0);
pub const BLOOM_INTENSITY_RANGE: (f32, f32) = (0.0, 1.0);

pub const RANDOM_DECAL_COUNT: usize = 20;
pub const RANDOM_DECAL_SIZE_RANGE: (f32, f32) = (0.5, 2.0);

pub const HELMET_POSITION: Vec3 = Vec3::new(0.0, 2.0, 0.0);

pub const PREVIEW_COLOR: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
pub const PREVIEW_NORMAL_COLOR: [f32; 4] = [1.0, 1.0, 0.0, 1.0];

pub struct RoomPiece {
    pub mesh: &'static str,
    pub position: Vec3,
    pub scale: Vec3,
    pub base_color: [f32; 4],
    pub roughness: f32,
    pub metallic: f32,
}

pub const ROOM: [RoomPiece; 11] = [
    RoomPiece {
        mesh: "Cube",
        position: Vec3::new(0.0, -0.5, 0.0),
        scale: Vec3::new(20.0, 1.0, 20.0),
        base_color: [0.15, 0.12, 0.1, 1.0],
        roughness: 0.85,
        metallic: 0.0,
    },
    RoomPiece {
        mesh: "Cube",
        position: Vec3::new(0.0, 5.0, -10.0),
        scale: Vec3::new(20.0, 10.0, 0.5),
        base_color: [0.85, 0.75, 0.65, 1.0],
        roughness: 0.7,
        metallic: 0.0,
    },
    RoomPiece {
        mesh: "Cube",
        position: Vec3::new(0.0, 5.0, 10.0),
        scale: Vec3::new(20.0, 10.0, 0.5),
        base_color: [0.7, 0.8, 0.75, 1.0],
        roughness: 0.7,
        metallic: 0.0,
    },
    RoomPiece {
        mesh: "Cube",
        position: Vec3::new(-10.0, 5.0, 0.0),
        scale: Vec3::new(0.5, 10.0, 20.0),
        base_color: [0.75, 0.55, 0.55, 1.0],
        roughness: 0.7,
        metallic: 0.0,
    },
    RoomPiece {
        mesh: "Cube",
        position: Vec3::new(10.0, 5.0, 0.0),
        scale: Vec3::new(0.5, 10.0, 20.0),
        base_color: [0.55, 0.65, 0.75, 1.0],
        roughness: 0.7,
        metallic: 0.0,
    },
    RoomPiece {
        mesh: "Cube",
        position: Vec3::new(-5.0, 3.0, -5.0),
        scale: Vec3::new(1.0, 6.0, 1.0),
        base_color: [0.6, 0.55, 0.5, 1.0],
        roughness: 0.5,
        metallic: 0.3,
    },
    RoomPiece {
        mesh: "Cube",
        position: Vec3::new(5.0, 3.0, -5.0),
        scale: Vec3::new(1.0, 6.0, 1.0),
        base_color: [0.6, 0.55, 0.5, 1.0],
        roughness: 0.5,
        metallic: 0.3,
    },
    RoomPiece {
        mesh: "Cube",
        position: Vec3::new(-5.0, 3.0, 5.0),
        scale: Vec3::new(1.0, 6.0, 1.0),
        base_color: [0.6, 0.55, 0.5, 1.0],
        roughness: 0.5,
        metallic: 0.3,
    },
    RoomPiece {
        mesh: "Cube",
        position: Vec3::new(5.0, 3.0, 5.0),
        scale: Vec3::new(1.0, 6.0, 1.0),
        base_color: [0.6, 0.55, 0.5, 1.0],
        roughness: 0.5,
        metallic: 0.3,
    },
    RoomPiece {
        mesh: "Sphere",
        position: Vec3::new(-3.0, 2.0, 0.0),
        scale: Vec3::new(2.0, 2.0, 2.0),
        base_color: [0.8, 0.3, 0.3, 1.0],
        roughness: 0.4,
        metallic: 0.1,
    },
    RoomPiece {
        mesh: "Cylinder",
        position: Vec3::new(3.0, 2.0, 0.0),
        scale: Vec3::new(1.5, 3.0, 1.5),
        base_color: [0.3, 0.5, 0.8, 1.0],
        roughness: 0.3,
        metallic: 0.2,
    },
];

pub struct InitialDecal {
    pub position: Vec3,
    pub normal: Vec3,
    pub decal_type: DecalType,
    pub size: f32,
}

pub const INITIAL_DECALS: [InitialDecal; 6] = [
    InitialDecal {
        position: Vec3::new(0.0, 3.0, -9.7),
        normal: Vec3::new(0.0, 0.0, 1.0),
        decal_type: DecalType::NeonSign,
        size: 2.0,
    },
    InitialDecal {
        position: Vec3::new(-3.0, 2.0, -9.7),
        normal: Vec3::new(0.0, 0.0, 1.0),
        decal_type: DecalType::Spray,
        size: 1.5,
    },
    InitialDecal {
        position: Vec3::new(3.0, 4.0, -9.7),
        normal: Vec3::new(0.0, 0.0, 1.0),
        decal_type: DecalType::BulletHole,
        size: 0.5,
    },
    InitialDecal {
        position: Vec3::new(-9.7, 2.5, 0.0),
        normal: Vec3::new(1.0, 0.0, 0.0),
        decal_type: DecalType::Graffiti,
        size: 3.0,
    },
    InitialDecal {
        position: Vec3::new(2.0, 0.1, 2.0),
        normal: Vec3::new(0.0, 1.0, 0.0),
        decal_type: DecalType::BloodSplatter,
        size: 1.2,
    },
    InitialDecal {
        position: Vec3::new(5.0, 3.0, -9.7),
        normal: Vec3::new(0.0, 0.0, 1.0),
        decal_type: DecalType::AwesomeFace,
        size: 2.5,
    },
];

pub struct RandomWall {
    pub origin: Vec3,
    pub normal: Vec3,
    pub across: Vec3,
    pub across_range: (f32, f32),
    pub up: Vec3,
    pub up_range: (f32, f32),
}

pub const RANDOM_WALLS: [RandomWall; 5] = [
    RandomWall {
        origin: Vec3::new(0.0, 0.1, 0.0),
        normal: Vec3::new(0.0, 1.0, 0.0),
        across: Vec3::new(1.0, 0.0, 0.0),
        across_range: (-9.0, 9.0),
        up: Vec3::new(0.0, 0.0, 1.0),
        up_range: (-9.0, 9.0),
    },
    RandomWall {
        origin: Vec3::new(0.0, 0.0, -9.7),
        normal: Vec3::new(0.0, 0.0, 1.0),
        across: Vec3::new(1.0, 0.0, 0.0),
        across_range: (-9.0, 9.0),
        up: Vec3::new(0.0, 1.0, 0.0),
        up_range: (1.0, 8.0),
    },
    RandomWall {
        origin: Vec3::new(0.0, 0.0, 9.7),
        normal: Vec3::new(0.0, 0.0, -1.0),
        across: Vec3::new(1.0, 0.0, 0.0),
        across_range: (-9.0, 9.0),
        up: Vec3::new(0.0, 1.0, 0.0),
        up_range: (1.0, 8.0),
    },
    RandomWall {
        origin: Vec3::new(-9.7, 0.0, 0.0),
        normal: Vec3::new(1.0, 0.0, 0.0),
        across: Vec3::new(0.0, 0.0, 1.0),
        across_range: (-9.0, 9.0),
        up: Vec3::new(0.0, 1.0, 0.0),
        up_range: (1.0, 8.0),
    },
    RandomWall {
        origin: Vec3::new(9.7, 0.0, 0.0),
        normal: Vec3::new(-1.0, 0.0, 0.0),
        across: Vec3::new(0.0, 0.0, 1.0),
        across_range: (-9.0, 9.0),
        up: Vec3::new(0.0, 1.0, 0.0),
        up_range: (1.0, 8.0),
    },
];
