use crate::contract::NavMeshAgent;
use crate::contract::NavMeshAgentState;
use crate::queries::funnel::{simplify_path, smooth_path};
use crate::queries::pathfinding::{
    PathRequest, PathStatus, find_containing_triangle, find_path_with_algorithm,
    sample_navmesh_height_near,
};
use crate::resources::*;
use nalgebra_glm::Vec3;
use nightshade_scene::contract::LocalTransform;

use nightshade::prelude::*;

pub fn update_agent_triangles_system(
    navmesh: Res<NavMeshWorld>,
    agents: Query<(
        &mut crate::contract::NavMeshAgent,
        &nightshade_scene::contract::LocalTransform,
    )>,
) {
    agents.for_each(|_entity, (agent, transform)| {
        agent.current_triangle = find_containing_triangle(&navmesh, transform.translation);
    });
}

pub fn process_path_requests_system(
    navmesh: Res<NavMeshWorld>,
    agents: Query<(&mut NavMeshAgent, &LocalTransform)>,
) {
    agents.for_each(|_entity, (agent, transform)| {
        if agent.state != NavMeshAgentState::PathPending {
            return;
        }
        let Some(target) = agent.target_position else {
            return;
        };
        let start = transform.translation;

        let request = PathRequest::new(start, target).with_radius(agent.agent_radius);
        let result = find_path_with_algorithm(&navmesh, &request, navmesh.algorithm);

        let (path, state) = match result.status {
            PathStatus::Found | PathStatus::PartialPath => {
                let smoothed = smooth_path(&navmesh, &result.triangle_path, start, target);
                let simplified = simplify_path(&smoothed, 1.5);
                let path = if simplified.len() > 1 {
                    simplified[1..].to_vec()
                } else {
                    simplified
                };
                (path, NavMeshAgentState::Moving)
            }
            PathStatus::StartOffMesh | PathStatus::EndOffMesh | PathStatus::NoPath => {
                (Vec::new(), NavMeshAgentState::NoPath)
            }
        };

        agent.current_path = path;
        agent.current_waypoint_index = 0;
        agent.state = state;
    });
}

pub fn gather_agent_neighborhood_system(
    mut neighborhood: ResMut<AgentNeighborhood>,
    agents: Query<(&NavMeshAgent, &LocalTransform)>,
) {
    neighborhood.agents.clear();
    let gathered = &mut neighborhood.agents;
    agents.for_each(|entity, (agent, transform)| {
        gathered.push((
            entity,
            transform.translation,
            agent.agent_radius,
            agent.state == NavMeshAgentState::Moving,
        ));
    });
}

pub fn agent_avoidance_system(
    neighborhood: Res<AgentNeighborhood>,
    agents: Query<(&mut NavMeshAgent,)>,
) {
    let gathered = &neighborhood.agents;
    agents.for_each(|entity, (agent,)| {
        let Some((_, position, radius, is_moving)) =
            gathered.iter().find(|(other, ..)| *other == entity)
        else {
            return;
        };
        if !is_moving {
            agent.avoidance_velocity = Vec3::zeros();
            return;
        }

        let mut avoidance = Vec3::zeros();
        let avoidance_radius = radius * 2.5;
        for (other_entity, other_position, other_radius, _) in gathered {
            if entity == *other_entity {
                continue;
            }

            let diff = *position - *other_position;
            let diff_xz = Vec3::new(diff.x, 0.0, diff.z);
            let distance = nalgebra_glm::length(&diff_xz);
            let min_distance = avoidance_radius + other_radius;

            if distance < min_distance && distance > 0.001 {
                let strength = (min_distance - distance) / min_distance;
                avoidance += nalgebra_glm::normalize(&diff_xz) * strength;
            }
        }
        agent.avoidance_velocity = avoidance;
    });
}

pub fn snap_idle_agents_to_navmesh(
    navmesh: Res<NavMeshWorld>,
    mut commands: Commands,
    agents: Query<(&NavMeshAgent, &LocalTransform)>,
) {
    agents.for_each(|entity, (agent, transform)| {
        if agent.state != NavMeshAgentState::Idle && agent.state != NavMeshAgentState::Arrived {
            return;
        }

        let current = transform.translation;
        let Some(height) =
            sample_navmesh_height_near(&navmesh, current.x, current.z, Some(current.y))
        else {
            return;
        };
        if (height - current.y).abs() <= 0.01 {
            return;
        }

        let mut snapped = *transform;
        snapped.translation.y = height;
        commands.set(entity, snapped);
    });
}

pub fn agent_movement_system(
    time: Res<nightshade_platform::contract::time::Time>,
    navmesh: Res<NavMeshWorld>,
    mut event_writer: EventWriter<crate::contract::NavigationArrived>,
    mut commands: Commands,
    agents: Query<(&mut NavMeshAgent, &LocalTransform)>,
) {
    let delta_time = time.delta_time;

    agents.for_each(|entity, (agent, transform)| {
        if agent.state != NavMeshAgentState::Moving || agent.current_path.is_empty() {
            agent.distance_to_destination = 0.0;
            return;
        }

        let (new_position, new_rotation, arrived, new_waypoint_index, distance) = if agent
            .current_waypoint_index
            >= agent.current_path.len()
        {
            (transform.translation, transform.rotation, true, 0, 0.0)
        } else {
            let mut current_position = transform.translation;
            let avoidance = agent.avoidance_velocity;
            let base_speed = agent.movement_speed;
            let mut remaining_movement = base_speed * delta_time;
            let mut waypoint_index = agent.current_waypoint_index;
            let mut final_rotation = transform.rotation;

            while remaining_movement > 0.0 && waypoint_index < agent.current_path.len() {
                let target_waypoint = agent.current_path[waypoint_index];
                let mut direction = target_waypoint - current_position;
                direction.y = 0.0;
                let distance_to_waypoint = nalgebra_glm::length(&direction);

                if distance_to_waypoint < agent.arrival_threshold {
                    waypoint_index += 1;
                    continue;
                }

                let normalized = nalgebra_glm::normalize(&direction);

                if normalized.x.abs() > 0.001 || normalized.z.abs() > 0.001 {
                    let angle = normalized.z.atan2(normalized.x);
                    final_rotation = nalgebra_glm::quat_angle_axis(
                        -angle + std::f32::consts::FRAC_PI_2,
                        &nalgebra_glm::vec3(0.0, 1.0, 0.0),
                    );
                }

                if remaining_movement >= distance_to_waypoint {
                    current_position = target_waypoint;
                    remaining_movement -= distance_to_waypoint;
                    waypoint_index += 1;
                } else {
                    let move_dir = normalized + avoidance * 0.25;
                    let move_dir_normalized = if nalgebra_glm::length(&move_dir) > 0.001 {
                        nalgebra_glm::normalize(&move_dir)
                    } else {
                        normalized
                    };
                    current_position += move_dir_normalized * remaining_movement;
                    remaining_movement = 0.0;
                }
            }

            let path_distance: f32 = agent
                .current_path
                .iter()
                .skip(waypoint_index)
                .zip(agent.current_path.iter().skip(waypoint_index + 1))
                .map(|(a, b)| nalgebra_glm::distance(a, b))
                .sum::<f32>()
                + if waypoint_index < agent.current_path.len() {
                    nalgebra_glm::distance(&current_position, &agent.current_path[waypoint_index])
                } else {
                    0.0
                };

            let arrived = waypoint_index >= agent.current_path.len();
            (
                current_position,
                final_rotation,
                arrived,
                waypoint_index,
                path_distance,
            )
        };

        let current_y = transform.translation.y;
        let final_position = if let Some(height) =
            sample_navmesh_height_near(&navmesh, new_position.x, new_position.z, Some(current_y))
        {
            let max_step_height = 1.0;
            if (height - current_y).abs() <= max_step_height {
                Vec3::new(new_position.x, height, new_position.z)
            } else {
                Vec3::new(new_position.x, current_y, new_position.z)
            }
        } else {
            new_position
        };

        let mut moved = *transform;
        moved.translation = final_position;
        moved.rotation = new_rotation;
        commands.set(entity, moved);

        agent.current_waypoint_index = new_waypoint_index;
        agent.distance_to_destination = distance;

        if arrived {
            agent.state = NavMeshAgentState::Arrived;
            agent.current_path.clear();
            agent.target_position = None;
            agent.distance_to_destination = 0.0;

            event_writer.send(crate::contract::NavigationArrived { entity });
        }
    });
}
