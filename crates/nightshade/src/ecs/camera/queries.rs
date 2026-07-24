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

pub fn query_camera_matrices(world: &World, entity: Entity) -> Option<CameraMatrices> {
    let camera = world.get::<crate::ecs::camera::components::Camera>(entity)?;
    let global_transform =
        world.get::<crate::ecs::transform::components::GlobalTransform>(entity)?;

    let camera_position = global_transform.translation();
    let forward = global_transform.forward_vector();
    let up = global_transform.up_vector();
    let target = camera_position + forward;

    let projection = match &camera.projection {
        crate::ecs::composition::components::Projection::Perspective(persp) => {
            let constrained = world
                .get::<crate::ecs::camera::components::ConstrainedAspect>(entity)
                .map(|c| c.0);
            let aspect_ratio = constrained
                .or(persp.aspect_ratio)
                .or_else(|| query_camera_tile_aspect_ratio(world, entity))
                .or_else(|| query_window_aspect_ratio(world))
                .unwrap_or(16.0 / 9.0);
            persp.matrix_with_aspect(aspect_ratio)
        }
        crate::ecs::composition::components::Projection::Orthographic(_) => {
            camera.projection.matrix()
        }
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
        .res::<crate::render_driver::Viewport>()
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
