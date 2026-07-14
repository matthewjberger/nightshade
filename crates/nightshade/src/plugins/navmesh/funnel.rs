use super::resources::NavMeshWorld;
use nalgebra_glm::Vec3;

pub fn smooth_path(
    navmesh: &NavMeshWorld,
    triangle_path: &[usize],
    start: Vec3,
    end: Vec3,
) -> Vec<Vec3> {
    if triangle_path.is_empty() {
        return vec![start, end];
    }

    if triangle_path.len() == 1 {
        return vec![start, end];
    }

    let portals = collect_portals(navmesh, triangle_path);

    if portals.is_empty() {
        return vec![start, end];
    }

    funnel_algorithm(start, end, &portals)
}

struct Portal {
    left: Vec3,
    right: Vec3,
}

fn collect_portals(navmesh: &NavMeshWorld, triangle_path: &[usize]) -> Vec<Portal> {
    let mut portals = Vec::new();

    for window_index in 0..triangle_path.len() - 1 {
        let current_triangle = triangle_path[window_index];
        let next_triangle = triangle_path[window_index + 1];

        if let Some(portal) = find_shared_edge_portal(navmesh, current_triangle, next_triangle) {
            portals.push(portal);
        }
    }

    portals
}

fn find_shared_edge_portal(
    navmesh: &NavMeshWorld,
    triangle_a: usize,
    triangle_b: usize,
) -> Option<Portal> {
    let tri_a = navmesh.get_triangle(triangle_a)?;
    let tri_b = navmesh.get_triangle(triangle_b)?;

    if !navmesh.edges.is_empty() {
        for &edge_index in &tri_a.edge_indices {
            if edge_index >= navmesh.edges.len() {
                continue;
            }
            let edge = &navmesh.edges[edge_index];

            if edge.get_other_triangle(triangle_a) == Some(triangle_b) {
                let vertex_left = navmesh.vertices[edge.vertex_indices[0]];
                let vertex_right = navmesh.vertices[edge.vertex_indices[1]];

                let center_a = tri_a.center;
                let edge_center = (vertex_left + vertex_right) * 0.5;

                let to_a = center_a - edge_center;
                let edge_vec = vertex_right - vertex_left;

                let cross_a = edge_vec.x * to_a.z - edge_vec.z * to_a.x;

                if cross_a > 0.0 {
                    return Some(Portal {
                        left: vertex_left,
                        right: vertex_right,
                    });
                } else {
                    return Some(Portal {
                        left: vertex_right,
                        right: vertex_left,
                    });
                }
            }
        }
    }

    let verts_a = &tri_a.vertex_indices;
    let verts_b = &tri_b.vertex_indices;

    let mut shared_verts = Vec::new();
    for &va in verts_a {
        for &vb in verts_b {
            if va == vb {
                shared_verts.push(va);
                break;
            }
        }
    }

    if shared_verts.len() >= 2 {
        let vertex_left = navmesh.vertices[shared_verts[0]];
        let vertex_right = navmesh.vertices[shared_verts[1]];

        let center_a = tri_a.center;
        let edge_center = (vertex_left + vertex_right) * 0.5;

        let to_a = center_a - edge_center;
        let edge_vec = vertex_right - vertex_left;

        let cross_a = edge_vec.x * to_a.z - edge_vec.z * to_a.x;

        if cross_a > 0.0 {
            return Some(Portal {
                left: vertex_left,
                right: vertex_right,
            });
        } else {
            return Some(Portal {
                left: vertex_right,
                right: vertex_left,
            });
        }
    }

    let center_a = tri_a.center;
    let center_b = tri_b.center;
    let midpoint = (center_a + center_b) * 0.5;
    Some(Portal {
        left: midpoint,
        right: midpoint,
    })
}

fn funnel_algorithm(start: Vec3, end: Vec3, portals: &[Portal]) -> Vec<Vec3> {
    let mut path = Vec::new();
    path.push(start);

    if portals.is_empty() {
        path.push(end);
        return path;
    }

    let mut apex = start;
    let mut left = portals[0].left;
    let mut right = portals[0].right;
    let mut left_index = 0;
    let mut right_index = 0;

    let mut portal_index = 1;

    while portal_index <= portals.len() {
        let (portal_left, portal_right) = if portal_index < portals.len() {
            (portals[portal_index].left, portals[portal_index].right)
        } else {
            (end, end)
        };

        if triarea2(apex, right, portal_right) <= 0.0 {
            if vequal(apex, right) || triarea2(apex, left, portal_right) > 0.0 {
                right = portal_right;
                right_index = portal_index;
            } else {
                path.push(left);
                apex = left;

                let new_apex_index = left_index;
                left = apex;
                right = apex;
                left_index = new_apex_index;
                right_index = new_apex_index;

                portal_index = new_apex_index + 1;
                continue;
            }
        }

        if triarea2(apex, left, portal_left) >= 0.0 {
            if vequal(apex, left) || triarea2(apex, right, portal_left) < 0.0 {
                left = portal_left;
                left_index = portal_index;
            } else {
                path.push(right);
                apex = right;

                let new_apex_index = right_index;
                left = apex;
                right = apex;
                left_index = new_apex_index;
                right_index = new_apex_index;

                portal_index = new_apex_index + 1;
                continue;
            }
        }

        portal_index += 1;
    }

    path.push(end);

    path
}

fn triarea2(point_a: Vec3, point_b: Vec3, point_c: Vec3) -> f32 {
    let axis_x = point_b.x - point_a.x;
    let axis_z = point_b.z - point_a.z;
    let cross_x = point_c.x - point_a.x;
    let cross_z = point_c.z - point_a.z;
    cross_x * axis_z - axis_x * cross_z
}

fn vequal(point_a: Vec3, point_b: Vec3) -> bool {
    let epsilon = 0.001;
    nalgebra_glm::distance2(&point_a, &point_b) < epsilon * epsilon
}

pub fn simplify_path(path: &[Vec3], epsilon: f32) -> Vec<Vec3> {
    if path.len() <= 2 {
        return path.to_vec();
    }

    let mut result = Vec::new();
    result.push(path[0]);

    let mut current_index = 0;

    while current_index < path.len() - 1 {
        let current = path[current_index];
        let mut furthest_visible = current_index + 1;

        for test_index in (current_index + 2)..path.len() {
            let test_point = path[test_index];
            let mut can_see = true;

            for intermediate in path.iter().take(test_index).skip(current_index + 1) {
                let distance = point_to_line_distance(*intermediate, current, test_point);

                if distance > epsilon {
                    can_see = false;
                    break;
                }
            }

            if can_see {
                furthest_visible = test_index;
            }
        }

        current_index = furthest_visible;
        result.push(path[current_index]);
    }

    result
}

fn point_to_line_distance(point: Vec3, line_start: Vec3, line_end: Vec3) -> f32 {
    let line_vec = line_end - line_start;
    let point_vec = point - line_start;

    let line_length_squared = nalgebra_glm::length2(&line_vec);

    if line_length_squared < 0.0001 {
        return nalgebra_glm::length(&point_vec);
    }

    let t = (nalgebra_glm::dot(&point_vec, &line_vec) / line_length_squared).clamp(0.0, 1.0);
    let projection = line_start + line_vec * t;

    nalgebra_glm::distance(&point, &projection)
}
