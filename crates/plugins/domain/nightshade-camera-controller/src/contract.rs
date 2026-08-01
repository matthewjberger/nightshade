pub mod collision_hook;
pub mod pan_orbit;
pub mod third_person;

pub use collision_hook::*;
pub use pan_orbit::*;
pub use third_person::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CameraMove {
    Orbit {
        yaw: f32,
        pitch: f32,
    },
    LookAt {
        yaw: Option<f32>,
        pitch: Option<f32>,
    },
}

impl Default for CameraMove {
    fn default() -> Self {
        Self::Orbit {
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PendingCameraMove {
    pub kind: CameraMove,
}
