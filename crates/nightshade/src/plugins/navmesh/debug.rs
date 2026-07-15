use super::components::NavMeshAgentState;
use crate::ecs::lines::components::Line;
use crate::ecs::world::{GLOBAL_TRANSFORM, LINES, LOCAL_TRANSFORM, VISIBILITY, World};
use nalgebra_glm::{Vec4, vec3};

use crate::prelude::*;
const NAVMESH_COLOR: Vec4 = Vec4::new(0.2, 0.7, 0.2, 1.0);
const NAVMESH_EDGE_COLOR: Vec4 = Vec4::new(0.3, 0.8, 0.3, 1.0);
const PATH_COLOR: Vec4 = Vec4::new(1.0, 1.0, 0.0, 1.0);
const AGENT_PATH_COLOR: Vec4 = Vec4::new(0.0, 1.0, 1.0, 1.0);
const WAYPOINT_COLOR: Vec4 = Vec4::new(1.0, 0.5, 0.0, 1.0);
const DESTINATION_COLOR: Vec4 = Vec4::new(1.0, 0.0, 0.5, 1.0);

pub fn navmesh_debug_draw_system(world: &mut World) {
    if !world.plugin_resource::<NavMeshWorld>().debug_draw {
        let debug_entity = world.plugin_resource::<NavMeshWorld>().debug_entity;
        if let Some(entity) = debug_entity
            && let Some(visibility) = world.get_mut::<crate::ecs::primitives::Visibility>(entity)
        {
            visibility.visible = false;
        }
        return;
    }

    let existing_debug_entity = world.plugin_resource::<NavMeshWorld>().debug_entity;
    let debug_entity = match existing_debug_entity {
        Some(entity) => entity,
        None => {
            let entity = spawn_entities(
                world,
                LINES | VISIBILITY | LOCAL_TRANSFORM | GLOBAL_TRANSFORM,
                1,
            )[0];
            world.plugin_resource_mut::<NavMeshWorld>().debug_entity = Some(entity);
            entity
        }
    };

    if let Some(visibility) = world.get_mut::<crate::ecs::primitives::Visibility>(debug_entity) {
        visibility.visible = true;
    }

    let mut lines = Vec::new();

    draw_navmesh_triangles(world, &mut lines);
    draw_agent_paths(world, &mut lines);

    if let Some(lines_component) =
        world.get_mut::<crate::ecs::lines::components::Lines>(debug_entity)
    {
        lines_component.lines = lines;
        lines_component.mark_dirty();
    }
}

fn draw_navmesh_triangles(world: &World, lines: &mut Vec<Line>) {
    let offset = vec3(0.0, 0.15, 0.0);

    for triangle in &world.plugin_resource::<NavMeshWorld>().triangles {
        let vertices = [
            world.plugin_resource::<NavMeshWorld>().vertices[triangle.vertex_indices[0]] + offset,
            world.plugin_resource::<NavMeshWorld>().vertices[triangle.vertex_indices[1]] + offset,
            world.plugin_resource::<NavMeshWorld>().vertices[triangle.vertex_indices[2]] + offset,
        ];

        lines.push(Line {
            start: vertices[0],
            end: vertices[1],
            color: NAVMESH_COLOR,
        });
        lines.push(Line {
            start: vertices[1],
            end: vertices[2],
            color: NAVMESH_COLOR,
        });
        lines.push(Line {
            start: vertices[2],
            end: vertices[0],
            color: NAVMESH_COLOR,
        });
    }

    for edge in &world.plugin_resource::<NavMeshWorld>().edges {
        if edge.is_shared() {
            let start =
                world.plugin_resource::<NavMeshWorld>().vertices[edge.vertex_indices[0]] + offset;
            let end =
                world.plugin_resource::<NavMeshWorld>().vertices[edge.vertex_indices[1]] + offset;

            lines.push(Line {
                start,
                end,
                color: NAVMESH_EDGE_COLOR,
            });
        }
    }
}

fn draw_agent_paths(world: &World, lines: &mut Vec<Line>) {
    let entities: Vec<_> = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::NAVMESH_AGENT)
        .collect();

    for entity in entities {
        let agent = match world.get::<crate::plugins::navmesh::components::NavMeshAgent>(entity) {
            Some(a) => a,
            None => continue,
        };

        let path_offset = vec3(0.0, 0.25, 0.0);

        if let Some(destination) = agent.target_position {
            draw_destination_marker(lines, destination + path_offset, 0.5, DESTINATION_COLOR);
        }

        if agent.current_path.is_empty() || agent.state != NavMeshAgentState::Moving {
            continue;
        }

        let agent_position = world
            .get::<crate::ecs::transform::components::LocalTransform>(entity)
            .map(|t| t.translation)
            .unwrap_or_else(|| vec3(0.0, 0.0, 0.0));

        if agent.current_waypoint_index < agent.current_path.len() {
            let first_waypoint = agent.current_path[agent.current_waypoint_index] + path_offset;
            lines.push(Line {
                start: agent_position + path_offset,
                end: first_waypoint,
                color: AGENT_PATH_COLOR,
            });
        }

        for waypoint_index in agent.current_waypoint_index..agent.current_path.len() {
            let waypoint = agent.current_path[waypoint_index] + path_offset;

            draw_cross(lines, waypoint, 0.15, WAYPOINT_COLOR);

            if waypoint_index + 1 < agent.current_path.len() {
                let next_waypoint = agent.current_path[waypoint_index + 1] + path_offset;
                lines.push(Line {
                    start: waypoint,
                    end: next_waypoint,
                    color: PATH_COLOR,
                });
            }
        }
    }
}

fn draw_cross(lines: &mut Vec<Line>, center: nalgebra_glm::Vec3, size: f32, color: Vec4) {
    let half = size * 0.5;

    lines.push(Line {
        start: center + vec3(-half, 0.0, -half),
        end: center + vec3(half, 0.0, half),
        color,
    });
    lines.push(Line {
        start: center + vec3(-half, 0.0, half),
        end: center + vec3(half, 0.0, -half),
        color,
    });
}

fn draw_destination_marker(
    lines: &mut Vec<Line>,
    center: nalgebra_glm::Vec3,
    size: f32,
    color: Vec4,
) {
    let half = size * 0.5;

    lines.push(Line {
        start: center + vec3(-half, 0.0, 0.0),
        end: center + vec3(half, 0.0, 0.0),
        color,
    });
    lines.push(Line {
        start: center + vec3(0.0, 0.0, -half),
        end: center + vec3(0.0, 0.0, half),
        color,
    });

    let diamond_size = half * 0.7;
    lines.push(Line {
        start: center + vec3(0.0, 0.0, -diamond_size),
        end: center + vec3(diamond_size, 0.0, 0.0),
        color,
    });
    lines.push(Line {
        start: center + vec3(diamond_size, 0.0, 0.0),
        end: center + vec3(0.0, 0.0, diamond_size),
        color,
    });
    lines.push(Line {
        start: center + vec3(0.0, 0.0, diamond_size),
        end: center + vec3(-diamond_size, 0.0, 0.0),
        color,
    });
    lines.push(Line {
        start: center + vec3(-diamond_size, 0.0, 0.0),
        end: center + vec3(0.0, 0.0, -diamond_size),
        color,
    });

    let pole_height = size * 1.5;
    lines.push(Line {
        start: center,
        end: center + vec3(0.0, pole_height, 0.0),
        color,
    });
}
