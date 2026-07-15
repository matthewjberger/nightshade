use super::components::NavMeshAgentState;
use super::funnel::{simplify_path, smooth_path};
use super::pathfinding::{
    PathRequest, PathStatus, find_containing_triangle, find_path_with_algorithm,
    sample_navmesh_height_near,
};
use crate::ecs::world::{Entity, World};
use nalgebra_glm::Vec3;

use crate::prelude::*;
pub fn run_navmesh_systems(world: &mut World) {
    update_agent_triangles_system(world);
    process_path_requests_system(world);
    agent_avoidance_system(world);
    agent_movement_system(world);
    snap_idle_agents_to_navmesh(world);
    super::debug::navmesh_debug_draw_system(world);
}

pub fn update_agent_triangles_system(world: &mut World) {
    let entities: Vec<_> = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::NAVMESH_AGENT | crate::ecs::world::LOCAL_TRANSFORM)
        .collect();

    for entity in entities {
        let position = match world.get::<crate::ecs::transform::components::LocalTransform>(entity)
        {
            Some(transform) => transform.translation,
            None => continue,
        };

        let triangle = find_containing_triangle(world.plugin_resource::<NavMeshWorld>(), position);

        if let Some(agent) =
            world.get_mut::<crate::plugins::navmesh::components::NavMeshAgent>(entity)
        {
            agent.current_triangle = triangle;
        }
    }
}

pub fn process_path_requests_system(world: &mut World) {
    let entities: Vec<_> = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::NAVMESH_AGENT | crate::ecs::world::LOCAL_TRANSFORM)
        .collect();

    let mut path_updates = Vec::new();

    for entity in &entities {
        let should_process = world
            .get::<crate::plugins::navmesh::components::NavMeshAgent>(*entity)
            .map(|agent| agent.state == NavMeshAgentState::PathPending)
            .unwrap_or(false);

        if !should_process {
            continue;
        }

        let (start, target, radius) = {
            let transform = world.get::<crate::ecs::transform::components::LocalTransform>(*entity);
            let agent = world.get::<crate::plugins::navmesh::components::NavMeshAgent>(*entity);

            match (transform, agent) {
                (Some(t), Some(a)) if a.target_position.is_some() => {
                    (t.translation, a.target_position.unwrap(), a.agent_radius)
                }
                _ => continue,
            }
        };

        let request = PathRequest::new(start, target).with_radius(radius);
        let algorithm = world.plugin_resource::<NavMeshWorld>().algorithm;
        let result =
            find_path_with_algorithm(world.plugin_resource::<NavMeshWorld>(), &request, algorithm);

        match result.status {
            PathStatus::Found | PathStatus::PartialPath => {
                let smoothed = smooth_path(
                    world.plugin_resource::<NavMeshWorld>(),
                    &result.triangle_path,
                    start,
                    target,
                );
                let simplified = simplify_path(&smoothed, 1.5);
                let path = if simplified.len() > 1 {
                    simplified[1..].to_vec()
                } else {
                    simplified
                };
                path_updates.push((*entity, path, NavMeshAgentState::Moving));
            }
            PathStatus::StartOffMesh | PathStatus::EndOffMesh | PathStatus::NoPath => {
                path_updates.push((*entity, Vec::new(), NavMeshAgentState::NoPath));
            }
        }
    }

    for (entity, path, state) in path_updates {
        if let Some(agent) =
            world.get_mut::<crate::plugins::navmesh::components::NavMeshAgent>(entity)
        {
            agent.current_path = path;
            agent.current_waypoint_index = 0;
            agent.state = state;
        }
    }
}

pub fn agent_avoidance_system(world: &mut World) {
    let entities: Vec<_> = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::NAVMESH_AGENT | crate::ecs::world::LOCAL_TRANSFORM)
        .collect();

    let agent_data: Vec<(Entity, Vec3, f32, bool)> = entities
        .iter()
        .filter_map(|&entity| {
            let transform =
                world.get::<crate::ecs::transform::components::LocalTransform>(entity)?;
            let agent = world.get::<crate::plugins::navmesh::components::NavMeshAgent>(entity)?;
            Some((
                entity,
                transform.translation,
                agent.agent_radius,
                agent.state == NavMeshAgentState::Moving,
            ))
        })
        .collect();

    let mut avoidance_updates: Vec<(Entity, Vec3)> = Vec::new();

    for (entity, position, radius, is_moving) in &agent_data {
        if !is_moving {
            avoidance_updates.push((*entity, Vec3::zeros()));
            continue;
        }

        let mut avoidance = Vec3::zeros();
        let avoidance_radius = radius * 2.5;

        for (other_entity, other_position, other_radius, _) in &agent_data {
            if entity == other_entity {
                continue;
            }

            let diff = *position - *other_position;
            let diff_xz = Vec3::new(diff.x, 0.0, diff.z);
            let distance = nalgebra_glm::length(&diff_xz);
            let min_distance = avoidance_radius + other_radius;

            if distance < min_distance && distance > 0.001 {
                let strength = (min_distance - distance) / min_distance;
                let direction = nalgebra_glm::normalize(&diff_xz);
                avoidance += direction * strength;
            }
        }

        avoidance_updates.push((*entity, avoidance));
    }

    for (entity, avoidance) in avoidance_updates {
        if let Some(agent) =
            world.get_mut::<crate::plugins::navmesh::components::NavMeshAgent>(entity)
        {
            agent.avoidance_velocity = avoidance;
        }
    }
}

pub fn snap_idle_agents_to_navmesh(world: &mut World) {
    let entities: Vec<_> = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::NAVMESH_AGENT | crate::ecs::world::LOCAL_TRANSFORM)
        .collect();

    for entity in entities {
        let is_idle = world
            .get::<crate::plugins::navmesh::components::NavMeshAgent>(entity)
            .map(|agent| {
                agent.state == NavMeshAgentState::Idle || agent.state == NavMeshAgentState::Arrived
            })
            .unwrap_or(true);

        if !is_idle {
            continue;
        }

        let current_pos =
            match world.get::<crate::ecs::transform::components::LocalTransform>(entity) {
                Some(t) => t.translation,
                None => continue,
            };

        let sampled_height = sample_navmesh_height_near(
            world.plugin_resource::<NavMeshWorld>(),
            current_pos.x,
            current_pos.z,
            Some(current_pos.y),
        );
        if let Some(height) = sampled_height
            && (height - current_pos.y).abs() > 0.01
            && let Some(transform) =
                world.get_mut::<crate::ecs::transform::components::LocalTransform>(entity)
        {
            transform.translation.y = height;
        }
    }
}

pub fn agent_movement_system(world: &mut World) {
    let delta_time = world
        .expect_resource_mut::<crate::ecs::time::Time>()
        .delta_time;

    let entities: Vec<_> = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::NAVMESH_AGENT | crate::ecs::world::LOCAL_TRANSFORM)
        .collect();

    for entity in entities {
        let should_move = world
            .get::<crate::plugins::navmesh::components::NavMeshAgent>(entity)
            .map(|agent| agent.state == NavMeshAgentState::Moving && !agent.current_path.is_empty())
            .unwrap_or(false);

        if !should_move {
            if let Some(agent) =
                world.get_mut::<crate::plugins::navmesh::components::NavMeshAgent>(entity)
            {
                agent.distance_to_destination = 0.0;
            }
            continue;
        }

        let movement_data = {
            let agent = world
                .get::<crate::plugins::navmesh::components::NavMeshAgent>(entity)
                .unwrap();
            let transform = world
                .get::<crate::ecs::transform::components::LocalTransform>(entity)
                .unwrap();

            if agent.current_waypoint_index >= agent.current_path.len() {
                Some((transform.translation, transform.rotation, true, 0, 0.0))
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
                        nalgebra_glm::distance(
                            &current_position,
                            &agent.current_path[waypoint_index],
                        )
                    } else {
                        0.0
                    };

                let arrived = waypoint_index >= agent.current_path.len();
                Some((
                    current_position,
                    final_rotation,
                    arrived,
                    waypoint_index,
                    path_distance,
                ))
            }
        };

        if let Some((new_position, new_rotation, arrived, new_waypoint_index, distance)) =
            movement_data
        {
            let current_y = world
                .get::<crate::ecs::transform::components::LocalTransform>(entity)
                .map(|t| t.translation.y)
                .unwrap_or(new_position.y);

            let final_position = if let Some(height) = sample_navmesh_height_near(
                world.plugin_resource::<NavMeshWorld>(),
                new_position.x,
                new_position.z,
                Some(current_y),
            ) {
                let max_step_height = 1.0;
                if (height - current_y).abs() <= max_step_height {
                    Vec3::new(new_position.x, height, new_position.z)
                } else {
                    Vec3::new(new_position.x, current_y, new_position.z)
                }
            } else {
                new_position
            };

            if let Some(transform) =
                world.get_mut::<crate::ecs::transform::components::LocalTransform>(entity)
            {
                transform.translation = final_position;
                transform.rotation = new_rotation;
            }

            if let Some(agent) =
                world.get_mut::<crate::plugins::navmesh::components::NavMeshAgent>(entity)
            {
                agent.current_waypoint_index = new_waypoint_index;
                agent.distance_to_destination = distance;

                if arrived {
                    agent.state = NavMeshAgentState::Arrived;
                    agent.current_path.clear();
                    agent.target_position = None;
                    agent.distance_to_destination = 0.0;
                }
            }

            if arrived {
                crate::ecs::event::emit_event(
                    world,
                    crate::ecs::event::Event::NavigationArrived { entity },
                );
            }
        }
    }
}
