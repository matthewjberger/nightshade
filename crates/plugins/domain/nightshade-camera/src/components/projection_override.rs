use nalgebra_glm::Mat4;

#[derive(
    Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, enum2schema::Schema,
)]
pub struct ProjectionOverride {
    #[schema(type = "array", items = "number", len = 16)]
    pub matrix: Mat4,
    pub z_near: f32,
    pub z_far: f32,
    pub y_fov_rad: f32,
    pub aspect: f32,
}

impl Default for ProjectionOverride {
    fn default() -> Self {
        Self {
            matrix: Mat4::identity(),
            z_near: 0.01,
            z_far: Self::DEFAULT_FAR,
            y_fov_rad: std::f32::consts::FRAC_PI_4,
            aspect: 16.0 / 9.0,
        }
    }
}

impl ProjectionOverride {
    pub const DEFAULT_FAR: f32 = 1000.0;

    pub fn from_tangents(
        tan_left: f32,
        tan_right: f32,
        tan_up: f32,
        tan_down: f32,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        let tan_width = tan_right - tan_left;
        let tan_height = tan_up - tan_down;
        let matrix = Mat4::new(
            2.0 / tan_width,
            0.0,
            (tan_right + tan_left) / tan_width,
            0.0,
            0.0,
            2.0 / tan_height,
            (tan_up + tan_down) / tan_height,
            0.0,
            0.0,
            0.0,
            z_near / (z_far - z_near),
            z_near * z_far / (z_far - z_near),
            0.0,
            0.0,
            -1.0,
            0.0,
        );
        Self {
            matrix,
            z_near,
            z_far,
            y_fov_rad: tan_height.atan() * 2.0,
            aspect: tan_width / tan_height,
        }
    }
}
