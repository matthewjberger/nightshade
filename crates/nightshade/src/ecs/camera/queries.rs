use crate::ecs::world::{Entity, Mat4, Vec3, World};

pub struct CameraMatrices {
    pub camera_position: Vec3,
    pub projection: Mat4,
    pub view: Mat4,
}

pub fn query_active_camera_matrices(world: &World) -> Option<CameraMatrices> {
    let active_camera = world.resources.active_camera?;
    query_camera_matrices(world, active_camera)
}

pub fn query_camera_matrices(world: &World, entity: Entity) -> Option<CameraMatrices> {
    let camera = world.get_camera(entity)?;
    let local_transform = world.get_local_transform(entity)?;
    let global_transform = world.get_global_transform(entity)?;

    let normalized_rotation = local_transform.rotation.normalize();
    let camera_position = global_transform.translation();
    let target = camera_position
        + nalgebra_glm::quat_rotate_vec3(&normalized_rotation, &Vec3::new(0.0, 0.0, -1.0));
    let up = nalgebra_glm::quat_rotate_vec3(&normalized_rotation, &Vec3::new(0.0, 1.0, 0.0));

    let projection = camera.projection.matrix();
    let view = nalgebra_glm::look_at(&camera_position, &target, &up);

    Some(CameraMatrices {
        camera_position,
        projection,
        view,
    })
}

pub fn query_window_aspect_ratio(world: &World) -> Option<f32> {
    let window_handle = world.resources.window.handle.as_ref()?;
    let inner_size = window_handle.inner_size();
    let aspect_ratio = inner_size.width as f32 / inner_size.height.max(1) as f32;
    Some(aspect_ratio)
}
