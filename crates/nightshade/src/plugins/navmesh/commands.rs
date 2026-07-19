use super::components::NavMeshAgentState;
use crate::ecs::world::{
    BOUNDING_VOLUME, GLOBAL_TRANSFORM, LOCAL_TRANSFORM, MATERIAL_REF, NAME, NAVMESH_AGENT,
    RENDER_MESH, VISIBILITY, World,
};
use nalgebra_glm::Vec3;

use crate::prelude::*;
pub fn spawn_navmesh_agent(
    world: &mut World,
    position: Vec3,
    speed: f32,
) -> nightshade_ecs::Entity {
    let entity = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | VISIBILITY
            | NAVMESH_AGENT,
        1,
    )[0];

    if let Some(name) = world.get_mut::<crate::ecs::primitives::Name>(entity) {
        name.0 = "NavMesh Agent".to_string();
    }

    if let Some(transform) =
        world.get_mut::<crate::ecs::transform::components::LocalTransform>(entity)
    {
        transform.translation = position;
        transform.scale = nalgebra_glm::vec3(0.4, 0.8, 0.4);
    }

    if let Some(mesh) = world.get_mut::<crate::ecs::mesh::components::RenderMesh>(entity) {
        mesh.name = "Capsule".to_string();
    }

    if let Some(agent) = world.get_mut::<crate::plugins::navmesh::components::NavMeshAgent>(entity)
    {
        agent.movement_speed = speed;
    }

    if let Some(bounding_volume) =
        world.get_mut::<crate::render::bounding_volume::BoundingVolume>(entity)
    {
        *bounding_volume =
            crate::render::bounding_volume::BoundingVolume::from_mesh_type("Capsule");
    }

    if let Some(visibility) = world.get_mut::<crate::ecs::primitives::Visibility>(entity) {
        visibility.visible = true;
    }

    entity
}

pub fn set_agent_destination(world: &mut World, entity: nightshade_ecs::Entity, destination: Vec3) {
    if let Some(agent) = world.get_mut::<crate::plugins::navmesh::components::NavMeshAgent>(entity)
    {
        agent.target_position = Some(destination);
        agent.state = NavMeshAgentState::PathPending;
    }
}

pub fn stop_agent(world: &mut World, entity: nightshade_ecs::Entity) {
    if let Some(agent) = world.get_mut::<crate::plugins::navmesh::components::NavMeshAgent>(entity)
    {
        agent.clear_destination();
    }
}

pub fn set_agent_speed(world: &mut World, entity: nightshade_ecs::Entity, speed: f32) {
    if let Some(agent) = world.get_mut::<crate::plugins::navmesh::components::NavMeshAgent>(entity)
    {
        agent.movement_speed = speed;
    }
}

pub fn clear_navmesh(world: &mut World) {
    world.plugin_resource_mut::<NavMeshWorld>().clear();
}

pub fn set_navmesh_debug_draw(world: &mut World, enabled: bool) {
    world.plugin_resource_mut::<NavMeshWorld>().debug_draw = enabled;
}

pub fn get_agent_state(world: &World, entity: nightshade_ecs::Entity) -> Option<NavMeshAgentState> {
    world
        .get::<crate::plugins::navmesh::components::NavMeshAgent>(entity)
        .map(|agent| agent.state)
}

pub fn get_agent_path_length(world: &World, entity: nightshade_ecs::Entity) -> Option<usize> {
    world
        .get::<crate::plugins::navmesh::components::NavMeshAgent>(entity)
        .map(|agent| agent.remaining_waypoints())
}
