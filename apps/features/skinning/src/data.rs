pub const FOX_MODEL: &[u8] = include_bytes!("../../../../assets/gltf/fox.glb");
pub const FOX_SCALE: f32 = 0.02;
pub const GAIT_INTERVAL: u32 = 120;
pub const REPORT_INTERVAL: u32 = 30;
pub const REPORT_UNTIL: u32 = 300;
pub const BLEND_FRAMES: std::ops::RangeInclusive<u32> = 60..=150;
pub const ROOT_BONES: [usize; 4] = [0, 1, 2, 3];
pub const FOX_OFFSETS: [f32; 2] = [-1.5, 1.5];
