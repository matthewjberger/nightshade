use crate::ecs::world::{Entity, Mat4, Vec3, World};

#[derive(Clone)]
pub struct CameraMatrices {
    pub camera_position: Vec3,
    pub projection: Mat4,
    pub view: Mat4,
}

pub fn query_active_camera_matrices(world: &World) -> Option<CameraMatrices> {
    #[cfg(feature = "openxr")]
    if let Some(xr_override) = &world.resources.xr.camera_override {
        return Some(xr_override.clone());
    }
    let active_camera = world.resources.active_camera?;
    query_camera_matrices(world, active_camera)
}

pub fn query_camera_matrices(world: &World, entity: Entity) -> Option<CameraMatrices> {
    let camera = world.get_camera(entity)?;
    let global_transform = world.get_global_transform(entity)?;

    let camera_position = global_transform.translation();
    let forward = global_transform.forward_vector();
    let up = global_transform.up_vector();
    let target = camera_position + forward;

    let projection = match &camera.projection {
        crate::ecs::world::components::Projection::Perspective(persp) => {
            let aspect_ratio = persp
                .aspect_ratio
                .or_else(|| query_window_aspect_ratio(world))
                .unwrap_or(16.0 / 9.0);
            persp.matrix_with_aspect(aspect_ratio)
        }
        crate::ecs::world::components::Projection::Orthographic(_) => camera.projection.matrix(),
    };

    let view = nalgebra_glm::look_at(&camera_position, &target, &up);

    Some(CameraMatrices {
        camera_position,
        projection,
        view,
    })
}

pub fn query_window_aspect_ratio(world: &World) -> Option<f32> {
    let (width, height) = world.resources.window.cached_viewport_size?;
    let aspect_ratio = width as f32 / height.max(1) as f32;
    Some(aspect_ratio)
}
