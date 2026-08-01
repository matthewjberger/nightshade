use nalgebra_glm::Vec3;
use nightshade_phase::PhaseLook;

pub const TITLE: &str = "Phase";
pub const REACH: f32 = 26.0;
pub const FLOOR_TINT: [f32; 4] = [0.20, 0.20, 0.22, 1.0];
pub const WALL_TINT: [f32; 4] = [0.46, 0.44, 0.40, 1.0];
pub const WALLS: usize = 8;
pub const SPACING: f32 = 5.0;
pub const LAMP_HEIGHT: f32 = 3.4;
pub const LAMP_BASE: f32 = 80.0;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.8, 0.0);
pub const CAMERA_BACK: f32 = 24.0;
pub const CAMERA_YAW: f32 = 0.5;
pub const CAMERA_PITCH: f32 = 0.22;

pub const FACES: [&str; 3] = ["fog", "otherworld", "daylight"];

pub fn looks() -> Vec<PhaseLook> {
    vec![
        PhaseLook {
            fog_color: [0.075, 0.078, 0.085],
            fog_start: 1.5,
            fog_end: 24.0,
            saturation: 0.7,
            contrast: 1.12,
            light_color: Vec3::new(1.0, 0.72, 0.45),
        },
        PhaseLook {
            fog_color: [0.11, 0.02, 0.012],
            fog_start: 1.0,
            fog_end: 15.0,
            saturation: 0.55,
            contrast: 1.25,
            light_color: Vec3::new(1.0, 0.2, 0.08),
        },
        PhaseLook {
            fog_color: [0.62, 0.68, 0.75],
            fog_start: 12.0,
            fog_end: 90.0,
            saturation: 1.05,
            contrast: 1.0,
            light_color: Vec3::new(1.0, 0.96, 0.9),
        },
    ]
}
