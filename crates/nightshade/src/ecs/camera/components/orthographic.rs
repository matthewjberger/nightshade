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
            z_near: 0.01,
        }
    }
}

impl OrthographicCamera {
    pub fn matrix(&self) -> Mat4 {
        reverse_z_ortho(
            -self.x_mag,
            self.x_mag,
            -self.y_mag,
            self.y_mag,
            self.z_near,
            self.z_far,
        )
    }
}

fn reverse_z_ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4 {
    let width = right - left;
    let height = top - bottom;
    let depth = far - near;

    Mat4::new(
        2.0 / width,
        0.0,
        0.0,
        -(right + left) / width,
        0.0,
        2.0 / height,
        0.0,
        -(top + bottom) / height,
        0.0,
        0.0,
        1.0 / depth,
        far / depth,
        0.0,
        0.0,
        0.0,
        1.0,
    )
}
