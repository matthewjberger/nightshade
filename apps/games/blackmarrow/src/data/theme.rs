use nightshade::prelude::{Vec3, Vec4};

pub const TITLE: &str = "Blackmarrow";

pub const TORCH_COLOR: Vec3 = Vec3::new(1.0, 0.93, 0.78);
pub const TORCH_INTENSITY: f32 = 155.0;
pub const TORCH_RANGE: f32 = 26.0;
pub const TORCH_INNER: f32 = 0.11;
pub const TORCH_OUTER: f32 = 0.33;
pub const TORCH_SHADOW_BIAS: f32 = 0.0005;
pub const TORCH_SHADOW_SOFTNESS: f32 = 0.05;
pub const TORCH_OFFSET: Vec3 = Vec3::new(0.16, -0.16, 0.0);

pub const LAMP_COLOR: Vec3 = Vec3::new(1.0, 0.72, 0.42);
pub const LAMP_INTENSITY: f32 = 42.0;
pub const LAMP_RANGE: f32 = 7.5;
pub const LAMP_SHADOW_BIAS: f32 = 0.0025;
pub const LAMP_SHADOW_SOFTNESS: f32 = 0.5;

pub const HANGING_INTENSITY: f32 = 30.0;
pub const HANGING_RANGE: f32 = 8.5;
pub const BULB_TINT: [f32; 4] = [1.0, 0.9, 0.66, 1.0];
pub const BULB_GLOW: [f32; 3] = [1.8, 1.2, 0.6];

pub const GLOWSTICK_COLOR: Vec3 = Vec3::new(0.45, 1.0, 0.4);
pub const GLOWSTICK_INTENSITY: f32 = 5.0;
pub const GLOWSTICK_RANGE: f32 = 3.4;
pub const GLOWSTICK_TINT: [f32; 4] = [0.35, 1.0, 0.3, 1.0];
pub const GLOWSTICK_GLOW: [f32; 3] = [0.5, 2.4, 0.45];

pub const PAPER_TINT: [f32; 4] = [0.82, 0.78, 0.68, 1.0];

pub const HIDE_TINT: [f32; 3] = [0.9, 0.8, 0.68];
pub const HIDE_ROUGHNESS: f32 = 0.95;

pub const FOG_COLOR: [f32; 3] = [0.018, 0.02, 0.026];
pub const FOG_START: f32 = 1.5;
pub const FOG_END: f32 = 26.0;
pub const FOG_DENSITY: f32 = 0.006;

pub const EXPOSURE_EV: f32 = -0.2;
pub const VIGNETTE: f32 = 0.62;
pub const SATURATION: f32 = 0.78;
pub const CONTRAST: f32 = 1.12;

pub const TEXT_PRIMARY: Vec4 = Vec4::new(0.86, 0.85, 0.81, 1.0);
pub const TEXT_DIM: Vec4 = Vec4::new(0.56, 0.55, 0.53, 1.0);
pub const TEXT_WARN: Vec4 = Vec4::new(0.83, 0.42, 0.28, 1.0);
pub const TEXT_OUTLINE: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.85);

pub const HUD_SIZE: f32 = 19.0;
pub const LABEL_SIZE: f32 = 17.0;

pub const MENU_CAMERA_EYE: Vec3 = Vec3::new(4.0, 1.9, 5.2);
pub const MENU_CAMERA_TARGET: Vec3 = Vec3::new(4.0, 1.5, 0.4);

pub const PLAYER_START: Vec3 = Vec3::new(4.0, 0.0, 1.6);
pub const PLAYER_FACING: f32 = std::f32::consts::PI;
pub const WOLF_START: Vec3 = Vec3::new(3.4, 0.0, 15.6);
pub const EXIT_AT: Vec3 = Vec3::new(3.5, 0.0, 24.2);

pub fn look() -> nightshade_lofi::LofiLook {
    nightshade_lofi::LofiLook {
        saturation: SATURATION,
        contrast: CONTRAST,
        vignette: VIGNETTE,
        vignette_radius: 0.55,
        ..nightshade_lofi::playstation()
    }
}
