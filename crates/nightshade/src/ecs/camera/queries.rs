use crate::ecs::world::{Entity, Mat4, Vec3, Vec4, World};

#[derive(Clone)]
pub struct CameraMatrices {
    pub camera_position: Vec3,
    pub projection: Mat4,
    pub view: Mat4,
}

pub fn query_active_camera_matrices(world: &World) -> Option<CameraMatrices> {
    let active_camera = world
        .res::<crate::ecs::camera::resources::ActiveCamera>()
        .0?;
    query_camera_matrices(world, active_camera)
}

/// How a camera's projection was decided, so the matrix and the scalar
/// parameters the passes need are read from one place.
///
/// Two callers need this and they need different halves of it: the view matrices
/// want the matrix, the frame inputs want the near and far planes and the field
/// of view. Resolving the precedence separately in each is how one of them ends
/// up not knowing about an override.
pub enum ResolvedProjection {
    /// A hand-supplied matrix, which wins outright.
    Override(crate::ecs::camera::components::ProjectionOverride),
    /// Derived from the camera's own perspective settings.
    Perspective(crate::ecs::camera::components::PerspectiveCamera, f32),
    /// Derived from the camera's own orthographic settings.
    Orthographic(crate::ecs::camera::components::OrthographicCamera),
    /// The entity carries no camera.
    None,
}

/// Decides which projection a camera uses, and with what aspect ratio.
pub fn resolve_projection(world: &World, entity: Entity) -> ResolvedProjection {
    if let Some(override_projection) =
        world.get::<crate::ecs::camera::components::ProjectionOverride>(entity)
    {
        return ResolvedProjection::Override(*override_projection);
    }
    match world
        .get::<crate::ecs::camera::components::Camera>(entity)
        .map(|camera| &camera.projection)
    {
        Some(crate::ecs::camera::components::Projection::Perspective(perspective)) => {
            let constrained = world
                .get::<crate::ecs::camera::components::ConstrainedAspect>(entity)
                .map(|constrained| constrained.0);
            let aspect_ratio = constrained
                .or(perspective.aspect_ratio)
                .or_else(|| query_camera_tile_aspect_ratio(world, entity))
                .or_else(|| query_window_aspect_ratio(world))
                .unwrap_or(16.0 / 9.0);
            ResolvedProjection::Perspective(*perspective, aspect_ratio)
        }
        Some(crate::ecs::camera::components::Projection::Orthographic(orthographic)) => {
            ResolvedProjection::Orthographic(*orthographic)
        }
        None => ResolvedProjection::None,
    }
}

pub fn query_camera_matrices(world: &World, entity: Entity) -> Option<CameraMatrices> {
    let camera = world.get::<crate::ecs::camera::components::Camera>(entity)?;
    let global_transform =
        world.get::<crate::ecs::transform::components::GlobalTransform>(entity)?;

    let camera_position = global_transform.translation();
    let forward = global_transform.forward_vector();
    let up = global_transform.up_vector();
    let target = camera_position + forward;

    let projection = match resolve_projection(world, entity) {
        ResolvedProjection::Override(override_projection) => override_projection.matrix,
        ResolvedProjection::Perspective(perspective, aspect_ratio) => {
            perspective.matrix_with_aspect(aspect_ratio)
        }
        ResolvedProjection::Orthographic(_) => camera.projection.matrix(),
        ResolvedProjection::None => return None,
    };

    let view = nalgebra_glm::look_at(&camera_position, &target, &up);

    Some(CameraMatrices {
        camera_position,
        projection,
        view,
    })
}

pub fn query_camera_tile_aspect_ratio(world: &World, entity: Entity) -> Option<f32> {
    let rect = world
        .res::<crate::viewport::Viewport>()
        .camera_tile_rects
        .get(&entity)?;
    if rect.height <= 0.0 {
        return None;
    }
    Some(rect.width / rect.height)
}

pub fn query_window_aspect_ratio(world: &World) -> Option<f32> {
    let (width, height) = world
        .res::<crate::platform::window::Window>()
        .cached_viewport_size?;
    let aspect_ratio = width as f32 / height.max(1) as f32;
    Some(aspect_ratio)
}

#[derive(Clone)]
pub struct CameraFrustumCorners {
    pub near_top_left: Vec3,
    pub near_top_right: Vec3,
    pub near_bottom_left: Vec3,
    pub near_bottom_right: Vec3,
    pub far_top_left: Vec3,
    pub far_top_right: Vec3,
    pub far_bottom_left: Vec3,
    pub far_bottom_right: Vec3,
}

pub fn query_camera_frustum(world: &World, entity: Entity) -> Option<CameraFrustumCorners> {
    let matrices = query_camera_matrices(world, entity)?;
    let view_proj = matrices.projection * matrices.view;
    let inv_view_proj = view_proj.try_inverse()?;

    let near_z = 1.0_f32;
    let far_z = 0.0_f32;

    Some(CameraFrustumCorners {
        near_top_left: unproject_ndc(&inv_view_proj, Vec3::new(-1.0, 1.0, near_z)),
        near_top_right: unproject_ndc(&inv_view_proj, Vec3::new(1.0, 1.0, near_z)),
        near_bottom_left: unproject_ndc(&inv_view_proj, Vec3::new(-1.0, -1.0, near_z)),
        near_bottom_right: unproject_ndc(&inv_view_proj, Vec3::new(1.0, -1.0, near_z)),
        far_top_left: unproject_ndc(&inv_view_proj, Vec3::new(-1.0, 1.0, far_z)),
        far_top_right: unproject_ndc(&inv_view_proj, Vec3::new(1.0, 1.0, far_z)),
        far_bottom_left: unproject_ndc(&inv_view_proj, Vec3::new(-1.0, -1.0, far_z)),
        far_bottom_right: unproject_ndc(&inv_view_proj, Vec3::new(1.0, -1.0, far_z)),
    })
}

fn unproject_ndc(inv_view_proj: &Mat4, ndc: Vec3) -> Vec3 {
    let clip = inv_view_proj * Vec4::new(ndc.x, ndc.y, ndc.z, 1.0);
    Vec3::new(clip.x / clip.w, clip.y / clip.w, clip.z / clip.w)
}
