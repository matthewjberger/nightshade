use super::resources::NavMeshWorld;
use nalgebra_glm::Vec3;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PathfindingAlgorithm {
    #[default]
    AStar,
    Dijkstra,
    GreedyBestFirst,
}

impl PathfindingAlgorithm {
    pub fn name(&self) -> &'static str {
        match self {
            PathfindingAlgorithm::AStar => "A*",
            PathfindingAlgorithm::Dijkstra => "Dijkstra",
            PathfindingAlgorithm::GreedyBestFirst => "Greedy Best-First",
        }
    }

    pub fn all() -> &'static [PathfindingAlgorithm] {
        &[
            PathfindingAlgorithm::AStar,
            PathfindingAlgorithm::Dijkstra,
            PathfindingAlgorithm::GreedyBestFirst,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct PathRequest {
    pub start: Vec3,
    pub end: Vec3,
    pub agent_radius: f32,
}

impl PathRequest {
    pub fn new(start: Vec3, end: Vec3) -> Self {
        Self {
            start,
            end,
            agent_radius: 0.3,
        }
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.agent_radius = radius;
        self
    }
}

#[derive(Debug, Clone)]
pub struct PathResult {
    pub waypoints: Vec<Vec3>,
    pub triangle_path: Vec<usize>,
    pub total_cost: f32,
    pub status: PathStatus,
}

impl PathResult {
    pub fn no_path() -> Self {
        Self {
            waypoints: Vec::new(),
            triangle_path: Vec::new(),
            total_cost: 0.0,
            status: PathStatus::NoPath,
        }
    }

    pub fn found(waypoints: Vec<Vec3>, triangle_path: Vec<usize>, total_cost: f32) -> Self {
        Self {
            waypoints,
            triangle_path,
            total_cost,
            status: PathStatus::Found,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathStatus {
    Found,
    PartialPath,
    NoPath,
    StartOffMesh,
    EndOffMesh,
}

struct AStarNode {
    triangle_index: usize,
    f_cost: f32,
}

impl PartialEq for AStarNode {
    fn eq(&self, other: &Self) -> bool {
        self.triangle_index == other.triangle_index
    }
}

impl Eq for AStarNode {}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f_cost
            .partial_cmp(&self.f_cost)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn find_path_with_algorithm(
    navmesh: &NavMeshWorld,
    request: &PathRequest,
    algorithm: PathfindingAlgorithm,
) -> PathResult {
    if navmesh.is_empty() {
        return PathResult {
            waypoints: Vec::new(),
            triangle_path: Vec::new(),
            total_cost: 0.0,
            status: PathStatus::NoPath,
        };
    }

    let start_triangle = find_containing_triangle(navmesh, request.start);
    let end_triangle = find_containing_triangle(navmesh, request.end);

    let start_triangle = match start_triangle {
        Some(index) => index,
        None => {
            return PathResult {
                waypoints: Vec::new(),
                triangle_path: Vec::new(),
                total_cost: 0.0,
                status: PathStatus::StartOffMesh,
            };
        }
    };

    let end_triangle = match end_triangle {
        Some(index) => index,
        None => {
            return PathResult {
                waypoints: Vec::new(),
                triangle_path: Vec::new(),
                total_cost: 0.0,
                status: PathStatus::EndOffMesh,
            };
        }
    };

    if start_triangle == end_triangle {
        return PathResult::found(vec![request.start, request.end], vec![start_triangle], 0.0);
    }

    let triangle_path = match algorithm {
        PathfindingAlgorithm::AStar => {
            astar_search(navmesh, start_triangle, end_triangle, request.end)
        }
        PathfindingAlgorithm::Dijkstra => dijkstra_search(navmesh, start_triangle, end_triangle),
        PathfindingAlgorithm::GreedyBestFirst => {
            greedy_search(navmesh, start_triangle, end_triangle, request.end)
        }
    };

    match triangle_path {
        Some((path, cost)) => {
            let mut waypoints = vec![request.start];
            for triangle_index in &path {
                waypoints.push(navmesh.triangles[*triangle_index].center);
            }
            waypoints.push(request.end);

            PathResult::found(waypoints, path, cost)
        }
        None => PathResult::no_path(),
    }
}

fn astar_search(
    navmesh: &NavMeshWorld,
    start: usize,
    goal: usize,
    goal_position: Vec3,
) -> Option<(Vec<usize>, f32)> {
    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<usize, usize> = HashMap::new();
    let mut g_score: HashMap<usize, f32> = HashMap::new();

    let start_h = heuristic(navmesh, start, goal_position);
    g_score.insert(start, 0.0);

    open_set.push(AStarNode {
        triangle_index: start,
        f_cost: start_h,
    });

    while let Some(current) = open_set.pop() {
        if current.triangle_index == goal {
            let path = reconstruct_path(&came_from, current.triangle_index);
            let cost = g_score[&current.triangle_index];
            return Some((path, cost));
        }

        let current_g = g_score[&current.triangle_index];

        for connection in navmesh.get_neighbors(current.triangle_index) {
            let tentative_g = current_g + connection.cost;

            let is_better = g_score
                .get(&connection.target_triangle)
                .map(|&existing_g| tentative_g < existing_g)
                .unwrap_or(true);

            if is_better {
                came_from.insert(connection.target_triangle, current.triangle_index);
                g_score.insert(connection.target_triangle, tentative_g);

                let h = heuristic(navmesh, connection.target_triangle, goal_position);
                let f = tentative_g + h;

                open_set.push(AStarNode {
                    triangle_index: connection.target_triangle,
                    f_cost: f,
                });
            }
        }
    }

    None
}

fn dijkstra_search(navmesh: &NavMeshWorld, start: usize, goal: usize) -> Option<(Vec<usize>, f32)> {
    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<usize, usize> = HashMap::new();
    let mut g_score: HashMap<usize, f32> = HashMap::new();

    g_score.insert(start, 0.0);

    open_set.push(AStarNode {
        triangle_index: start,
        f_cost: 0.0,
    });

    while let Some(current) = open_set.pop() {
        if current.triangle_index == goal {
            let path = reconstruct_path(&came_from, current.triangle_index);
            let cost = g_score[&current.triangle_index];
            return Some((path, cost));
        }

        let current_g = g_score[&current.triangle_index];

        for connection in navmesh.get_neighbors(current.triangle_index) {
            let tentative_g = current_g + connection.cost;

            let is_better = g_score
                .get(&connection.target_triangle)
                .map(|&existing_g| tentative_g < existing_g)
                .unwrap_or(true);

            if is_better {
                came_from.insert(connection.target_triangle, current.triangle_index);
                g_score.insert(connection.target_triangle, tentative_g);

                open_set.push(AStarNode {
                    triangle_index: connection.target_triangle,
                    f_cost: tentative_g,
                });
            }
        }
    }

    None
}

fn greedy_search(
    navmesh: &NavMeshWorld,
    start: usize,
    goal: usize,
    goal_position: Vec3,
) -> Option<(Vec<usize>, f32)> {
    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<usize, usize> = HashMap::new();
    let mut visited: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut total_cost = 0.0;

    let start_h = heuristic(navmesh, start, goal_position);

    open_set.push(AStarNode {
        triangle_index: start,
        f_cost: start_h,
    });

    while let Some(current) = open_set.pop() {
        if current.triangle_index == goal {
            let path = reconstruct_path(&came_from, current.triangle_index);
            for window_index in 0..path.len().saturating_sub(1) {
                let from = path[window_index];
                let to = path[window_index + 1];
                for connection in navmesh.get_neighbors(from) {
                    if connection.target_triangle == to {
                        total_cost += connection.cost;
                        break;
                    }
                }
            }
            return Some((path, total_cost));
        }

        if !visited.insert(current.triangle_index) {
            continue;
        }

        for connection in navmesh.get_neighbors(current.triangle_index) {
            if visited.contains(&connection.target_triangle) {
                continue;
            }

            came_from.insert(connection.target_triangle, current.triangle_index);

            let h = heuristic(navmesh, connection.target_triangle, goal_position);

            open_set.push(AStarNode {
                triangle_index: connection.target_triangle,
                f_cost: h,
            });
        }
    }

    None
}

fn heuristic(navmesh: &NavMeshWorld, triangle_index: usize, goal: Vec3) -> f32 {
    let center = navmesh.triangles[triangle_index].center;
    nalgebra_glm::distance(&center, &goal)
}

fn reconstruct_path(came_from: &HashMap<usize, usize>, mut current: usize) -> Vec<usize> {
    let mut path = vec![current];

    while let Some(&parent) = came_from.get(&current) {
        path.push(parent);
        current = parent;
    }

    path.reverse();
    path
}

pub fn find_containing_triangle(navmesh: &NavMeshWorld, point: Vec3) -> Option<usize> {
    let candidates = navmesh
        .spatial_hash
        .query_radius(point, navmesh.spatial_hash.cell_size);

    let mut containing_triangles: Vec<usize> = Vec::new();

    for &triangle_index in &candidates {
        if point_in_triangle(navmesh, triangle_index, point) {
            containing_triangles.push(triangle_index);
        }
    }

    if containing_triangles.is_empty() {
        for (triangle_index, _) in navmesh.triangles.iter().enumerate() {
            if point_in_triangle(navmesh, triangle_index, point) {
                containing_triangles.push(triangle_index);
            }
        }
    }

    if !containing_triangles.is_empty() {
        return containing_triangles.into_iter().min_by(|&a, &b| {
            let y_a = navmesh.triangles[a].center.y;
            let y_b = navmesh.triangles[b].center.y;
            let diff_a = (y_a - point.y).abs();
            let diff_b = (y_b - point.y).abs();
            diff_a
                .partial_cmp(&diff_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    let mut closest_triangle = None;
    let mut closest_distance = f32::MAX;

    for (triangle_index, triangle) in navmesh.triangles.iter().enumerate() {
        let distance = nalgebra_glm::distance(&triangle.center, &point);
        if distance < closest_distance {
            closest_distance = distance;
            closest_triangle = Some(triangle_index);
        }
    }

    if closest_distance < 5.0 {
        closest_triangle
    } else {
        None
    }
}

fn point_in_triangle(navmesh: &NavMeshWorld, triangle_index: usize, point: Vec3) -> bool {
    let vertices = match navmesh.get_triangle_vertices(triangle_index) {
        Some(v) => v,
        None => return false,
    };

    let vertex_a = nalgebra_glm::vec2(vertices[0].x, vertices[0].z);
    let vertex_b = nalgebra_glm::vec2(vertices[1].x, vertices[1].z);
    let vertex_c = nalgebra_glm::vec2(vertices[2].x, vertices[2].z);
    let test_point = nalgebra_glm::vec2(point.x, point.z);

    let v0 = vertex_c - vertex_a;
    let v1 = vertex_b - vertex_a;
    let v2 = test_point - vertex_a;

    let dot00 = nalgebra_glm::dot(&v0, &v0);
    let dot01 = nalgebra_glm::dot(&v0, &v1);
    let dot02 = nalgebra_glm::dot(&v0, &v2);
    let dot11 = nalgebra_glm::dot(&v1, &v1);
    let dot12 = nalgebra_glm::dot(&v1, &v2);

    let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
    let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
    let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

    u >= 0.0 && v >= 0.0 && (u + v) <= 1.0
}

pub fn find_closest_point_on_navmesh(navmesh: &NavMeshWorld, point: Vec3) -> Option<Vec3> {
    if let Some(triangle_index) = find_containing_triangle(navmesh, point) {
        let vertices = navmesh.get_triangle_vertices(triangle_index)?;
        let y = (vertices[0].y + vertices[1].y + vertices[2].y) / 3.0;
        return Some(nalgebra_glm::vec3(point.x, y, point.z));
    }

    let mut closest_point = None;
    let mut closest_distance = f32::MAX;

    for triangle in &navmesh.triangles {
        let distance = nalgebra_glm::distance(&triangle.center, &point);
        if distance < closest_distance {
            closest_distance = distance;
            closest_point = Some(triangle.center);
        }
    }

    closest_point
}

pub fn sample_navmesh_height(navmesh: &NavMeshWorld, x: f32, z: f32) -> Option<f32> {
    sample_navmesh_height_near(navmesh, x, z, None)
}

pub fn sample_navmesh_height_near(
    navmesh: &NavMeshWorld,
    x: f32,
    z: f32,
    reference_y: Option<f32>,
) -> Option<f32> {
    let point = nalgebra_glm::vec3(x, 0.0, z);

    let candidates = navmesh
        .spatial_hash
        .query_radius(point, navmesh.spatial_hash.cell_size * 2.0);

    let mut valid_heights: Vec<f32> = Vec::new();

    for &triangle_index in &candidates {
        if point_in_triangle(navmesh, triangle_index, point)
            && let Some(vertices) = navmesh.get_triangle_vertices(triangle_index)
        {
            valid_heights.push(barycentric_height(vertices, x, z));
        }
    }

    if valid_heights.is_empty() {
        for (triangle_index, _) in navmesh.triangles.iter().enumerate() {
            if point_in_triangle(navmesh, triangle_index, point)
                && let Some(vertices) = navmesh.get_triangle_vertices(triangle_index)
            {
                valid_heights.push(barycentric_height(vertices, x, z));
            }
        }
    }

    if valid_heights.is_empty() {
        let mut closest_triangle = None;
        let mut closest_distance = f32::MAX;

        for &triangle_index in &candidates {
            let triangle = &navmesh.triangles[triangle_index];
            let dx = triangle.center.x - x;
            let dz = triangle.center.z - z;
            let distance = (dx * dx + dz * dz).sqrt();
            if distance < closest_distance {
                closest_distance = distance;
                closest_triangle = Some(triangle_index);
            }
        }

        if let Some(triangle_index) = closest_triangle
            && let Some(vertices) = navmesh.get_triangle_vertices(triangle_index)
        {
            valid_heights.push(barycentric_height(vertices, x, z));
        }
    }

    if valid_heights.is_empty() {
        return None;
    }

    if let Some(ref_y) = reference_y {
        valid_heights.into_iter().min_by(|a, b| {
            let diff_a = (*a - ref_y).abs();
            let diff_b = (*b - ref_y).abs();
            diff_a
                .partial_cmp(&diff_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    } else {
        valid_heights
            .into_iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    }
}

fn barycentric_height(vertices: [Vec3; 3], x: f32, z: f32) -> f32 {
    let v0 = vertices[0];
    let v1 = vertices[1];
    let v2 = vertices[2];

    let denom = (v1.z - v2.z) * (v0.x - v2.x) + (v2.x - v1.x) * (v0.z - v2.z);

    if denom.abs() < 0.0001 {
        return (v0.y + v1.y + v2.y) / 3.0;
    }

    let w0 = ((v1.z - v2.z) * (x - v2.x) + (v2.x - v1.x) * (z - v2.z)) / denom;
    let w1 = ((v2.z - v0.z) * (x - v2.x) + (v0.x - v2.x) * (z - v2.z)) / denom;
    let w2 = 1.0 - w0 - w1;

    let w0 = w0.clamp(0.0, 1.0);
    let w1 = w1.clamp(0.0, 1.0);
    let w2 = w2.clamp(0.0, 1.0);
    let sum = w0 + w1 + w2;

    (w0 * v0.y + w1 * v1.y + w2 * v2.y) / sum
}
