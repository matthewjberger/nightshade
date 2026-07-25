//! One camera's view of the scene: the per-view matrices and target rect,
//! the viewport rectangle, and the reverse-Z projection they are built
//! with.

/// Per-dispatch camera state the renderer extracts once at the start of each
/// per-camera render and every pass reads, replacing scattered
/// `query_active_camera_matrices` calls and their per-pass re-derivations.
/// `projection` carries the TAA jitter, matching `query_active_camera_matrices`.
#[derive(Clone, Debug)]
pub struct RenderView {
    /// World-to-view matrix.
    pub view: nalgebra_glm::Mat4,
    /// View-to-clip projection, carrying the `TAA` jitter.
    pub projection: nalgebra_glm::Mat4,
    /// Combined world-to-clip matrix.
    pub view_projection: nalgebra_glm::Mat4,
    /// View-to-world matrix.
    pub inverse_view: nalgebra_glm::Mat4,
    /// Clip-to-view matrix.
    pub inverse_projection: nalgebra_glm::Mat4,
    /// Clip-to-world matrix.
    pub inverse_view_projection: nalgebra_glm::Mat4,
    /// Camera position in world space.
    pub camera_position: nalgebra_glm::Vec3,
    /// Camera right axis in world space.
    pub camera_right: nalgebra_glm::Vec3,
    /// Camera up axis in world space.
    pub camera_up: nalgebra_glm::Vec3,
    /// Six world-space frustum planes for culling.
    pub frustum_planes: [nalgebra_glm::Vec4; 6],
    /// Near clip distance in world units.
    pub z_near: f32,
    /// Far clip distance in world units.
    pub z_far: f32,
    /// Vertical field of view in radians.
    pub y_fov_rad: f32,
    /// Aspect ratio (width over height).
    pub aspect: f32,
    /// Orthographic half-width and half-height, `None` for perspective.
    pub orthographic: Option<(f32, f32)>,
    /// Render target size in pixels.
    pub screen_size: (u32, u32),
    /// Fixed aspect the view is letterboxed to, if constrained.
    pub constrained_aspect: Option<f32>,
}

/// Per-entity dynamic render state the renderer refreshes each frame without a
/// full rebuild: whether the object is visible and its current morph weights.
#[derive(Clone, Copy, Debug)]
pub struct DynamicRenderState {
    /// Visibility flag, 1 visible and 0 hidden.
    pub visible: u32,
    /// Active morph target weights.
    pub morph_weights: [f32; 8],
    /// World transform.
    pub transform: nalgebra_glm::Mat4,
    /// Culling layer bitmask tested against camera masks.
    pub culling_mask: u32,
    /// Overlay flag, 1 for overlay-layer entities.
    pub is_overlay: u32,
    /// Render layer index.
    pub render_layer: u32,
}

/// The reverse-Z perspective matrix the render passes expect (near maps to
/// depth 1, far to depth 0). Use this instead of a stock perspective, whose
/// depth convention will not line up with the depth the passes read.
pub fn reverse_z_perspective(
    y_fov_rad: f32,
    aspect: f32,
    z_near: f32,
    z_far: f32,
) -> nalgebra_glm::Mat4 {
    let focal_length = 1.0 / (y_fov_rad / 2.0).tan();
    nalgebra_glm::Mat4::new(
        focal_length / aspect,
        0.0,
        0.0,
        0.0,
        0.0,
        focal_length,
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

/// A camera tile's rectangle in screen pixels.
#[derive(Debug, Clone, Copy, Default)]
pub struct ViewportRect {
    /// Left edge in screen pixels.
    pub x: f32,
    /// Top edge in screen pixels.
    pub y: f32,
    /// Width in screen pixels.
    pub width: f32,
    /// Height in screen pixels.
    pub height: f32,
}

impl ViewportRect {
    /// True when `screen_pos` lies within the rectangle.
    pub fn contains(&self, screen_pos: nalgebra_glm::Vec2) -> bool {
        screen_pos.x >= self.x
            && screen_pos.x <= self.x + self.width
            && screen_pos.y >= self.y
            && screen_pos.y <= self.y + self.height
    }

    /// Converts a screen position into coordinates relative to the top-left.
    pub fn to_local(&self, screen_pos: nalgebra_glm::Vec2) -> nalgebra_glm::Vec2 {
        nalgebra_glm::Vec2::new(screen_pos.x - self.x, screen_pos.y - self.y)
    }
}
