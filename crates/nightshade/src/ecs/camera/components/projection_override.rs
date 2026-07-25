use nalgebra_glm::Mat4;

/// A hand-supplied projection matrix that replaces whatever the camera's
/// [`Projection`](super::Projection) would have produced.
///
/// [`Projection`](super::Projection) covers symmetric perspective and
/// orthographic cameras, which is every projection a camera derives from a field
/// of view and an aspect ratio. Some projections are not derived that way: an
/// off-center frustum for a stereo eye, the oblique frustum a portal needs to
/// clip at its own plane, or an asymmetric one spanning several displays. This
/// component is the escape hatch for those, and it is what the OpenXR plugin
/// writes each frame from the runtime's per-eye field of view.
///
/// `matrix` must follow the engine's depth convention: reverse-Z, so it maps the
/// near plane to 1 and the far plane (or infinity) to 0. Passing a conventional
/// 0-to-1 projection renders inverted or empty against the reverse-Z depth
/// buffer.
///
/// The scalar fields describe the same frustum for the passes that need numbers
/// rather than a matrix: screen-space ambient occlusion, depth of field, and fog
/// all read them through `RenderView`. Fill them with the closest symmetric
/// equivalent of the frustum the matrix encodes.
#[derive(
    Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, enum2schema::Schema,
)]
pub struct ProjectionOverride {
    /// Reverse-Z view-to-clip matrix used verbatim.
    #[schema(type = "array", items = "number", len = 16)]
    pub matrix: Mat4,
    /// Near clip distance in world units.
    pub z_near: f32,
    /// Far clip distance in world units, for passes that need a finite bound.
    pub z_far: f32,
    /// Vertical field of view in radians.
    pub y_fov_rad: f32,
    /// Width over height ratio of the frustum.
    pub aspect: f32,
}

impl Default for ProjectionOverride {
    fn default() -> Self {
        Self {
            matrix: Mat4::identity(),
            z_near: 0.01,
            z_far: 1000.0,
            y_fov_rad: std::f32::consts::FRAC_PI_4,
            aspect: 16.0 / 9.0,
        }
    }
}

impl ProjectionOverride {
    /// Builds an off-center reverse-Z projection with an infinite far plane from
    /// the tangents of the four frustum half-angles.
    ///
    /// This is the form OpenXR reports a view's field of view in, where the
    /// frustum is asymmetric because each eye looks through its lens off axis.
    /// Rows two and three match
    /// `PerspectiveCamera`'s infinite reverse-Z projection exactly, so depth
    /// behaves identically to every other camera in the engine; only the
    /// horizontal and vertical extents differ. With a symmetric frustum it
    /// reduces to the same matrix a `PerspectiveCamera` would produce.
    pub fn from_tangents(
        tan_left: f32,
        tan_right: f32,
        tan_up: f32,
        tan_down: f32,
        z_near: f32,
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
            0.0,
            z_near,
            0.0,
            0.0,
            -1.0,
            0.0,
        );
        Self {
            matrix,
            z_near,
            z_far: 1000.0,
            y_fov_rad: tan_height.atan() * 2.0,
            aspect: tan_width / tan_height,
        }
    }
}
