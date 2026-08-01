use crate::contract::NavMeshAgent;
use crate::contract::NavMeshAgentState;
use crate::resources::*;
use nalgebra_glm::{Vec4, vec3};
use nightshade_render::components::lines::Line;
use nightshade_scene::contract::LocalTransform;

use nightshade::prelude::*;
const NAVMESH_COLOR: Vec4 = Vec4::new(0.2, 0.7, 0.2, 1.0);
const NAVMESH_EDGE_COLOR: Vec4 = Vec4::new(0.3, 0.8, 0.3, 1.0);
const PATH_COLOR: Vec4 = Vec4::new(1.0, 1.0, 0.0, 1.0);
const AGENT_PATH_COLOR: Vec4 = Vec4::new(0.0, 1.0, 1.0, 1.0);
const WAYPOINT_COLOR: Vec4 = Vec4::new(1.0, 0.5, 0.0, 1.0);
const DESTINATION_COLOR: Vec4 = Vec4::new(1.0, 0.0, 0.5, 1.0);

pub fn navmesh_debug_draw_system(
    mut navmesh: ResMut<NavMeshWorld>,
    mut commands: Commands,
    agents: Query<(&NavMeshAgent, &LocalTransform)>,
    mut target: Query<(
        &mut nightshade_render::components::lines::Lines,
        &mut nightshade_scene::contract::primitives::Visibility,
    )>,
) {
    if !navmesh.debug_draw {
        if let Some(entity) = navmesh.debug_entity {
            target.for_entity(entity, |(_, visibility)| visibility.visible = false);
        }
        return;
    }

    let debug_entity = match navmesh.debug_entity {
        Some(entity) => entity,
        None => {
            let entity = commands.spawn((
                nightshade_render::components::lines::Lines::default(),
                nightshade_scene::contract::primitives::Visibility::default(),
                nightshade_scene::contract::LocalTransform::default(),
                nightshade_scene::contract::GlobalTransform::default(),
            ));
            navmesh.debug_entity = Some(entity);
            return;
        }
    };

    let mut lines = Vec::new();
    draw_navmesh_triangles(&navmesh, &mut lines);
    draw_agent_paths(agents, &mut lines);

    target.for_entity(debug_entity, |(lines_component, visibility)| {
        visibility.visible = true;
        lines_component.lines = std::mem::take(&mut lines);
        lines_component.mark_dirty();
    });
}

fn draw_navmesh_triangles(navmesh: &NavMeshWorld, lines: &mut Vec<Line>) {
    let offset = vec3(0.0, 0.15, 0.0);

    for triangle in &navmesh.triangles {
        let vertices = [
            navmesh.vertices[triangle.vertex_indices[0]] + offset,
            navmesh.vertices[triangle.vertex_indices[1]] + offset,
            navmesh.vertices[triangle.vertex_indices[2]] + offset,
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

    for edge in &navmesh.edges {
        if edge.is_shared() {
            let start = navmesh.vertices[edge.vertex_indices[0]] + offset;
            let end = navmesh.vertices[edge.vertex_indices[1]] + offset;

            lines.push(Line {
                start,
                end,
                color: NAVMESH_EDGE_COLOR,
            });
        }
    }
}

fn draw_agent_paths(agents: Query<(&NavMeshAgent, &LocalTransform)>, lines: &mut Vec<Line>) {
    let path_offset = vec3(0.0, 0.25, 0.0);

    agents.for_each(|_entity, (agent, transform)| {
        if let Some(destination) = agent.target_position {
            draw_destination_marker(lines, destination + path_offset, 0.5, DESTINATION_COLOR);
        }

        if agent.current_path.is_empty() || agent.state != NavMeshAgentState::Moving {
            return;
        }

        let agent_position = transform.translation;

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
    });
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
