use nalgebra_glm::Mat4;

#[derive(
    Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, enum2schema::Schema,
)]
pub struct OrthographicCamera {
    pub x_mag: Option<f32>,
    pub y_mag: f32,
    pub z_far: f32,
    pub z_near: f32,
}

impl Default for OrthographicCamera {
    fn default() -> Self {
        Self {
            x_mag: None,
            y_mag: 10.0,
            z_far: 1000.0,
            z_near: 0.01,
        }
    }
}

impl OrthographicCamera {
    pub fn from_pixels_per_unit(viewport_height: f32, pixels_per_unit: f32) -> Self {
        Self {
            y_mag: viewport_height / pixels_per_unit.max(f32::EPSILON) / 2.0,
            ..Default::default()
        }
    }

    pub fn half_width(&self, aspect_ratio: f32) -> f32 {
        self.x_mag.unwrap_or(self.y_mag * aspect_ratio)
    }

    pub fn matrix(&self) -> Mat4 {
        self.matrix_with_aspect(16.0 / 9.0)
    }

    pub fn matrix_with_aspect(&self, aspect_ratio: f32) -> Mat4 {
        let half_width = self.half_width(aspect_ratio);
        reverse_z_ortho(
            -half_width,
            half_width,
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
