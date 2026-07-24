//! Third-person camera collision: a raycast from the focus toward the ideal
//! camera position that pulls the camera in when a physics body blocks the
//! view. Installed as the camera domain's collision resolver so the camera
//! stays independent of the physics plugin.

use crate::ecs::camera::resources::CameraCollision;
use crate::ecs::world::World;
use crate::plugins::physics::resources::{PhysicsWorld, physics_world_query_pipeline};
use nalgebra_glm::Vec3;

/// Installs the raycast collision resolver into [`CameraCollision`].
pub fn install_camera_collision(world: &mut World) {
    world.res_mut::<CameraCollision>().resolver = Some(Box::new(camera_collision_distance));
}

fn camera_collision_distance(
    world: &World,
    focus: Vec3,
    ideal_position: Vec3,
    radius: f32,
    distance: f32,
) -> f32 {
    let direction = ideal_position - focus;
    let ray_length = nalgebra_glm::length(&direction);
    if ray_length <= 0.001 || world.ecs.resource::<PhysicsWorld>().is_none() {
        return distance;
    }
    let ray_dir = direction / ray_length;
    let ray = rapier3d::prelude::Ray::new(
        rapier3d::na::Point3::new(focus.x, focus.y, focus.z),
        rapier3d::na::Vector3::new(ray_dir.x, ray_dir.y, ray_dir.z),
    );
    let query_pipeline = physics_world_query_pipeline(world.plugin_resource::<PhysicsWorld>());
    if let Some((_handle, toi)) = query_pipeline.cast_ray(&ray, ray_length, true) {
        let safe_distance = (toi - radius).max(0.5);
        if safe_distance < distance {
            return safe_distance;
        }
    }
    distance
}
