use nalgebra_glm::Mat4;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OrthographicCamera {
    pub x_mag: f32,
    pub y_mag: f32,
    pub z_far: f32,
    pub z_near: f32,
}

impl Default for OrthographicCamera {
    fn default() -> Self {
        Self {
            x_mag: 10.0,
            y_mag: 10.0,
            z_far: 1000.0,
            z_near: 0.1,
        }
    }
}

impl OrthographicCamera {
    pub fn matrix(&self) -> Mat4 {
        nalgebra_glm::ortho_rh_zo(
            -self.x_mag,
            self.x_mag,
            -self.y_mag,
            self.y_mag,
            self.z_near,
            self.z_far,
        )
    }
}
