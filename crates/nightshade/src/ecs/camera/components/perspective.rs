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
        self.matrix_with_aspect(aspect_ratio)
    }

    pub fn matrix_with_aspect(&self, aspect_ratio: f32) -> Mat4 {
        match self.z_far {
            Some(z_far) => reverse_z_perspective(aspect_ratio, self.y_fov_rad, self.z_near, z_far),
            None => infinite_reverse_z_perspective(aspect_ratio, self.y_fov_rad, self.z_near),
        }
    }
}

fn infinite_reverse_z_perspective(aspect_ratio: f32, y_fov_rad: f32, z_near: f32) -> Mat4 {
    let f = 1.0 / (y_fov_rad / 2.0).tan();
    Mat4::new(
        f / aspect_ratio,
        0.0,
        0.0,
        0.0,
        0.0,
        f,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        z_near,
        0.0,
        0.0,
        -1.0,
        0.0,
    )
}

fn reverse_z_perspective(aspect_ratio: f32, y_fov_rad: f32, z_near: f32, z_far: f32) -> Mat4 {
    let f = 1.0 / (y_fov_rad / 2.0).tan();
    Mat4::new(
        f / aspect_ratio,
        0.0,
        0.0,
        0.0,
        0.0,
        f,
        0.0,
        0.0,
        0.0,
        0.0,
        z_near / (z_far - z_near),
        z_near * z_far / (z_far - z_near),
        0.0,
        0.0,
        -1.0,
        0.0,
    )
}
