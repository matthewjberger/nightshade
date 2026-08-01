use crate::queries::physics_world::physics_world_query_pipeline;
use crate::resources::PhysicsWorld;
use nalgebra_glm;
use nightshade_camera_controller::contract::CameraSweep;

pub fn camera_collision_distance(physics: &PhysicsWorld, ask: &CameraSweep) -> f32 {
    let direction = ask.ideal_position - ask.focus;
    let ray_length = nalgebra_glm::length(&direction);
    if ray_length <= 0.001 {
        return ask.distance;
    }
    let ray_dir = direction / ray_length;
    let ray = rapier3d::prelude::Ray::new(
        rapier3d::na::Point3::new(ask.focus.x, ask.focus.y, ask.focus.z),
        rapier3d::na::Vector3::new(ray_dir.x, ray_dir.y, ray_dir.z),
    );
    let query_pipeline = physics_world_query_pipeline(physics);
    if let Some((_handle, toi)) = query_pipeline.cast_ray(&ray, ray_length, true) {
        let safe_distance = (toi - ask.radius).max(0.5);
        if safe_distance < ask.distance {
            return safe_distance;
        }
    }
    ask.distance
}
