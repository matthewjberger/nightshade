use crate::queries;
use crate::resources::{Behavior, Ground};
use nightshade::prelude::*;
use nightshade_navmesh::commands::NavMeshAgents;
use nightshade_navmesh::contract::NavMeshAgentState;

pub fn update_wander_mode(
    behavior: Res<Behavior>,
    ground: Res<Ground>,
    mut agents: NavMeshAgents<'_, '_>,
) {
    if !behavior.wander {
        return;
    }

    agents.for_each_mut(|_entity, (agent,)| {
        let waiting = agent.state == NavMeshAgentState::Idle
            || agent.state == NavMeshAgentState::Arrived
            || agent.state == NavMeshAgentState::NoPath;
        if !waiting {
            return;
        }

        let x = (rand::random::<f32>() - 0.5) * 60.0;
        let z = (rand::random::<f32>() - 0.5) * 60.0;
        let y = queries::sample_terrain_height(x, z, &ground.0);
        agent.target_position = Some(nalgebra_glm::vec3(x, y, z));
        agent.state = NavMeshAgentState::PathPending;
    });
}
