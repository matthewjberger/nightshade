//! Foot IK ground query: a downward raycast that reports the ground hit
//! point so the animation foot IK can plant feet on physics geometry.
//! Installed as the animation domain's ground-query resolver so the
//! animation domain stays independent of the physics plugin.

use crate::ecs::animation::systems::FootIkGroundQuery;
use crate::ecs::world::{Entity, World};
use crate::plugins::physics::resources::{PhysicsWorld, physics_world_cast_ray};
use nalgebra_glm::Vec3;

/// Installs the raycast ground-query resolver into [`FootIkGroundQuery`].
pub fn install_foot_ik_ground_query(world: &mut World) {
    world.res_mut::<FootIkGroundQuery>().resolver = Some(ground_hit_point);
}

fn ground_hit_point(
    world: &World,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    exclude: Option<Entity>,
) -> Option<Vec3> {
    world.ecs.resource::<PhysicsWorld>()?;
    physics_world_cast_ray(
        world.plugin_resource::<PhysicsWorld>(),
        origin,
        direction,
        max_distance,
        exclude,
    )
    .map(|hit| hit.point)
}
