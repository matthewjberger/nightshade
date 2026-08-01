use nalgebra_glm;
use nalgebra_glm::{Quat, Vec3};
use nightshade_ecs;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ThirdPersonCamera {
    pub follow_target: Option<nightshade_ecs::Entity>,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub target_yaw: f32,
    pub target_pitch: f32,
    pub target_distance: f32,
    pub height_offset: f32,
    pub pitch_upper_limit: f32,
    pub pitch_lower_limit: f32,
    pub distance_min: f32,
    pub distance_max: f32,
    pub orbit_sensitivity: f32,
    pub zoom_sensitivity: f32,
    pub orbit_smoothness: f32,
    pub zoom_smoothness: f32,
    pub follow_smoothness: f32,
    pub collision_enabled: bool,
    pub collision_radius: f32,
    pub current_focus: Vec3,
}

impl Default for ThirdPersonCamera {
    fn default() -> Self {
        Self {
            follow_target: None,
            yaw: 0.0,
            pitch: 0.3,
            distance: 8.0,
            target_yaw: 0.0,
            target_pitch: 0.3,
            target_distance: 8.0,
            height_offset: 1.5,
            pitch_upper_limit: std::f32::consts::FRAC_PI_2 - 0.05,
            pitch_lower_limit: -0.3,
            distance_min: 1.0,
            distance_max: 30.0,
            orbit_sensitivity: 1.0,
            zoom_sensitivity: 1.0,
            orbit_smoothness: 0.08,
            zoom_smoothness: 0.1,
            follow_smoothness: 0.05,
            collision_enabled: true,
            collision_radius: 0.3,
            current_focus: Vec3::zeros(),
        }
    }
}

impl ThirdPersonCamera {
    pub fn new(distance: f32) -> Self {
        Self {
            distance,
            target_distance: distance,
            ..Default::default()
        }
    }

    pub fn with_target(mut self, target: nightshade_ecs::Entity) -> Self {
        self.follow_target = Some(target);
        self
    }

    pub fn with_yaw_pitch(mut self, yaw: f32, pitch: f32) -> Self {
        self.yaw = yaw;
        self.pitch = pitch;
        self.target_yaw = yaw;
        self.target_pitch = pitch;
        self
    }

    pub fn with_height_offset(mut self, offset: f32) -> Self {
        self.height_offset = offset;
        self
    }

    pub fn with_distance_limits(mut self, min: f32, max: f32) -> Self {
        self.distance_min = min;
        self.distance_max = max;
        self
    }

    pub fn with_pitch_limits(mut self, min: f32, max: f32) -> Self {
        self.pitch_lower_limit = min;
        self.pitch_upper_limit = max;
        self
    }

    pub fn with_smoothness(mut self, orbit: f32, zoom: f32, follow: f32) -> Self {
        self.orbit_smoothness = orbit;
        self.zoom_smoothness = zoom;
        self.follow_smoothness = follow;
        self
    }

    pub fn compute_camera_transform(&self, focus: Vec3) -> (Vec3, Quat) {
        compute_third_person_transform(focus, self.yaw, self.pitch, self.distance)
    }
}

pub fn compute_third_person_transform(
    focus: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
) -> (Vec3, Quat) {
    let yaw_quat = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y());
    let pitch_quat = nalgebra_glm::quat_angle_axis(-pitch, &Vec3::x());
    let rotation = yaw_quat * pitch_quat;
    let offset = nalgebra_glm::quat_rotate_vec3(&rotation, &Vec3::new(0.0, 0.0, distance));
    let position = focus + offset;
    (position, rotation)
}
