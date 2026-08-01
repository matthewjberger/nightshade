pub mod generation;

use crate::contract::NavMeshAgentState;
use crate::contract::*;
use crate::resources::*;
use nalgebra_glm::Vec3;
use nightshade::ecs::system_param::Commands;

use nightshade::prelude::*;

pub type NavMeshAgents<'world, 'item> =
    nightshade::ecs::system_param::Query<'world, (&'item mut crate::contract::NavMeshAgent,)>;

pub fn spawn_navmesh_agent(
    commands: &mut Commands<'_>,
    position: Vec3,
    speed: f32,
) -> nightshade::ecs::Entity {
    commands.spawn((
        nightshade_scene::contract::primitives::Name("NavMesh Agent".to_string()),
        nightshade_scene::contract::LocalTransform {
            translation: position,
            scale: nalgebra_glm::vec3(0.4, 0.8, 0.4),
            ..Default::default()
        },
        nightshade_scene::contract::GlobalTransform::default(),
        nightshade_render::components::mesh::RenderMesh::new("Capsule"),
        nightshade_render::components::material::MaterialRef::default(),
        nightshade_render::data::bounding_volume::BoundingVolume::from_mesh_type("Capsule"),
        nightshade_scene::contract::primitives::Visibility { visible: true },
        NavMeshAgent::new().with_speed(speed),
    ))
}

pub fn set_agent_destination(
    agents: &mut NavMeshAgents<'_, '_>,
    entity: nightshade::ecs::Entity,
    destination: Vec3,
) {
    agents.for_entity(entity, |(agent,)| {
        agent.target_position = Some(destination);
        agent.state = NavMeshAgentState::PathPending;
    });
}

pub fn stop_agent(agents: &mut NavMeshAgents<'_, '_>, entity: nightshade::ecs::Entity) {
    agents.for_entity(entity, |(agent,)| agent.clear_destination());
}

pub fn set_agent_speed(
    agents: &mut NavMeshAgents<'_, '_>,
    entity: nightshade::ecs::Entity,
    speed: f32,
) {
    agents.for_entity(entity, |(agent,)| agent.movement_speed = speed);
}

pub fn clear_navmesh(navmesh: &mut NavMeshWorld) {
    navmesh.clear();
}

pub fn set_navmesh_debug_draw(navmesh: &mut NavMeshWorld, enabled: bool) {
    navmesh.debug_draw = enabled;
}

pub fn get_agent_state(
    agents: &mut NavMeshAgents<'_, '_>,
    entity: nightshade::ecs::Entity,
) -> Option<NavMeshAgentState> {
    let mut state = None;
    agents.for_entity(entity, |(agent,)| state = Some(agent.state));
    state
}

pub fn get_agent_path_length(
    agents: &mut NavMeshAgents<'_, '_>,
    entity: nightshade::ecs::Entity,
) -> Option<usize> {
    let mut waypoints = None;
    agents.for_entity(entity, |(agent,)| {
        waypoints = Some(agent.remaining_waypoints())
    });
    waypoints
}
