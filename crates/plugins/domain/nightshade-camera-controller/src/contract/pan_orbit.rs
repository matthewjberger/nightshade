use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum PanOrbitButton {
    #[default]
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PanOrbitModifier {
    Shift,
    Control,
    Alt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PanOrbitBindings {
    pub orbit: PanOrbitButton,
    pub pan: PanOrbitButton,
    pub orbit_modifier: Option<PanOrbitModifier>,
    pub pan_modifier: Option<PanOrbitModifier>,
}

impl Default for PanOrbitBindings {
    fn default() -> Self {
        Self {
            orbit: PanOrbitButton::Left,
            pan: PanOrbitButton::Right,
            orbit_modifier: None,
            pan_modifier: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PanOrbitSensitivity {
    pub orbit: f32,
    pub pan: f32,
    pub zoom: f32,
    pub gamepad_orbit: f32,
    pub gamepad_pan: f32,
    pub gamepad_zoom: f32,
}

impl Default for PanOrbitSensitivity {
    fn default() -> Self {
        Self {
            orbit: 1.0,
            pan: 1.0,
            zoom: 1.0,
            gamepad_orbit: 2.0,
            gamepad_pan: 10.0,
            gamepad_zoom: 5.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PanOrbitSmoothness {
    pub orbit: f32,
    pub pan: f32,
    pub zoom: f32,
    pub gamepad: f32,
}

impl Default for PanOrbitSmoothness {
    fn default() -> Self {
        Self {
            orbit: 0.1,
            pan: 0.02,
            zoom: 0.1,
            gamepad: 0.06,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PanOrbitLimits {
    pub pitch_lower: f32,
    pub pitch_upper: f32,
    pub zoom_lower: f32,
    pub zoom_upper: Option<f32>,
    pub allow_upside_down: bool,
}

impl Default for PanOrbitLimits {
    fn default() -> Self {
        Self {
            pitch_lower: -(std::f32::consts::FRAC_PI_2 - 0.01),
            pitch_upper: std::f32::consts::FRAC_PI_2 - 0.01,
            zoom_lower: 0.05,
            zoom_upper: None,
            allow_upside_down: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PanOrbitCamera {
    pub focus: Vec3,
    pub radius: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub target_focus: Vec3,
    pub target_radius: f32,
    pub target_yaw: f32,
    pub target_pitch: f32,
    pub bindings: PanOrbitBindings,
    pub sensitivity: PanOrbitSensitivity,
    pub smoothness: PanOrbitSmoothness,
    pub limits: PanOrbitLimits,
    pub enabled: bool,
    pub pan_distance: Option<f32>,
    #[serde(skip)]
    pub smoothed_gamepad_orbit: Vec2,
    #[serde(skip)]
    pub smoothed_gamepad_pan: Vec2,
    #[serde(skip)]
    pub is_upside_down: bool,
}

impl Default for PanOrbitCamera {
    fn default() -> Self {
        Self {
            focus: Vec3::zeros(),
            radius: 10.0,
            yaw: 0.0,
            pitch: 0.0,
            target_focus: Vec3::zeros(),
            target_radius: 10.0,
            target_yaw: 0.0,
            target_pitch: 0.0,
            bindings: PanOrbitBindings::default(),
            sensitivity: PanOrbitSensitivity::default(),
            smoothness: PanOrbitSmoothness::default(),
            limits: PanOrbitLimits::default(),
            enabled: true,
            pan_distance: None,
            smoothed_gamepad_orbit: Vec2::zeros(),
            smoothed_gamepad_pan: Vec2::zeros(),
            is_upside_down: false,
        }
    }
}

impl PanOrbitCamera {
    pub fn new(focus: Vec3, radius: f32) -> Self {
        Self {
            focus,
            radius,
            target_focus: focus,
            target_radius: radius,
            ..Default::default()
        }
    }

    pub fn with_yaw_pitch(mut self, yaw: f32, pitch: f32) -> Self {
        self.yaw = yaw;
        self.pitch = pitch;
        self.target_yaw = yaw;
        self.target_pitch = pitch;
        self
    }

    pub fn with_zoom_limits(mut self, min: f32, max: Option<f32>) -> Self {
        self.limits.zoom_lower = min;
        self.limits.zoom_upper = max;
        self
    }

    pub fn with_pitch_limits(mut self, min: f32, max: f32) -> Self {
        self.limits.pitch_lower = min;
        self.limits.pitch_upper = max;
        self
    }

    pub fn with_smoothness(mut self, orbit: f32, pan: f32, zoom: f32) -> Self {
        self.smoothness.orbit = orbit;
        self.smoothness.pan = pan;
        self.smoothness.zoom = zoom;
        self
    }

    pub fn with_buttons(mut self, orbit: PanOrbitButton, pan: PanOrbitButton) -> Self {
        self.bindings.orbit = orbit;
        self.bindings.pan = pan;
        self
    }

    pub fn with_modifiers(
        mut self,
        orbit: Option<PanOrbitModifier>,
        pan: Option<PanOrbitModifier>,
    ) -> Self {
        self.bindings.orbit_modifier = orbit;
        self.bindings.pan_modifier = pan;
        self
    }

    pub fn with_upside_down(mut self, allow: bool) -> Self {
        self.limits.allow_upside_down = allow;
        self
    }

    pub fn compute_camera_transform(&self) -> (Vec3, nalgebra_glm::Quat) {
        compute_pan_orbit_transform(self.focus, self.yaw, self.pitch, self.radius)
    }

    pub fn compute_target_camera_transform(&self) -> (Vec3, nalgebra_glm::Quat) {
        compute_pan_orbit_transform(
            self.target_focus,
            self.target_yaw,
            self.target_pitch,
            self.target_radius,
        )
    }
}

pub fn compute_pan_orbit_transform(
    focus: Vec3,
    yaw: f32,
    pitch: f32,
    radius: f32,
) -> (Vec3, nalgebra_glm::Quat) {
    let yaw_quat = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y());
    let pitch_quat = nalgebra_glm::quat_angle_axis(-pitch, &Vec3::x());
    let rotation = yaw_quat * pitch_quat;
    let position = focus + nalgebra_glm::quat_rotate_vec3(&rotation, &Vec3::new(0.0, 0.0, radius));
    (position, rotation)
}
