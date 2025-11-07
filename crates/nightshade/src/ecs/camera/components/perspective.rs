use nalgebra_glm::Mat4;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PerspectiveCamera {
    pub aspect_ratio: Option<f32>,
    pub y_fov_rad: f32,
    pub z_far: Option<f32>,
    pub z_near: f32,
}

impl Default for PerspectiveCamera {
    fn default() -> Self {
        Self {
            aspect_ratio: None,
            y_fov_rad: 45.0_f32.to_radians(),
            z_far: None,
            z_near: 0.1,
        }
    }
}

impl PerspectiveCamera {
    pub fn matrix(&self) -> Mat4 {
        let aspect_ratio = self.aspect_ratio.unwrap_or(16.0 / 9.0);

        match self.z_far {
            Some(z_far) => {
                nalgebra_glm::perspective_zo(aspect_ratio, self.y_fov_rad, self.z_near, z_far)
            }
            None => {
                nalgebra_glm::infinite_perspective_rh_zo(aspect_ratio, self.y_fov_rad, self.z_near)
            }
        }
    }
}
