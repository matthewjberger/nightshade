use super::pathfinding::PathfindingAlgorithm;
use nalgebra_glm::Vec3;
use std::collections::HashMap;

pub struct NavMeshWorld {
    pub vertices: Vec<Vec3>,
    pub triangles: Vec<NavMeshTriangle>,
    pub edges: Vec<NavMeshEdge>,
    pub adjacency: HashMap<usize, Vec<NavMeshConnection>>,
    pub spatial_hash: SpatialHash,
    pub debug_draw: bool,
    pub debug_entity: Option<freecs::Entity>,
    pub algorithm: PathfindingAlgorithm,
}

impl Default for NavMeshWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl NavMeshWorld {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            triangles: Vec::new(),
            edges: Vec::new(),
            adjacency: HashMap::new(),
            spatial_hash: SpatialHash::new(2.0),
            debug_draw: false,
            debug_entity: None,
            algorithm: PathfindingAlgorithm::default(),
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.triangles.clear();
        self.edges.clear();
        self.adjacency.clear();
        self.spatial_hash.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.triangles.is_empty()
    }

    pub fn get_triangle(&self, index: usize) -> Option<&NavMeshTriangle> {
        self.triangles.get(index)
    }

    pub fn get_vertex(&self, index: usize) -> Option<Vec3> {
        self.vertices.get(index).copied()
    }

    pub fn get_triangle_vertices(&self, triangle_index: usize) -> Option<[Vec3; 3]> {
        let triangle = self.triangles.get(triangle_index)?;
        Some([
            self.vertices[triangle.vertex_indices[0]],
            self.vertices[triangle.vertex_indices[1]],
            self.vertices[triangle.vertex_indices[2]],
        ])
    }

    pub fn get_neighbors(&self, triangle_index: usize) -> &[NavMeshConnection] {
        self.adjacency
            .get(&triangle_index)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

#[derive(Debug, Clone)]
pub struct NavMeshTriangle {
    pub vertex_indices: [usize; 3],
    pub center: Vec3,
    pub normal: Vec3,
    pub area: f32,
    pub edge_indices: [usize; 3],
}

impl NavMeshTriangle {
    pub fn new(vertex_indices: [usize; 3], vertices: &[Vec3]) -> Self {
        let vertex_a = vertices[vertex_indices[0]];
        let vertex_b = vertices[vertex_indices[1]];
        let vertex_c = vertices[vertex_indices[2]];

        let center = (vertex_a + vertex_b + vertex_c) / 3.0;

        let edge_ab = vertex_b - vertex_a;
        let edge_ac = vertex_c - vertex_a;
        let cross = nalgebra_glm::cross(&edge_ab, &edge_ac);
        let area = nalgebra_glm::length(&cross) * 0.5;
        let normal = if area > 0.0 {
            nalgebra_glm::normalize(&cross)
        } else {
            nalgebra_glm::vec3(0.0, 1.0, 0.0)
        };

        Self {
            vertex_indices,
            center,
            normal,
            area,
            edge_indices: [0, 0, 0],
        }
    }
}

#[derive(Debug, Clone)]
pub struct NavMeshEdge {
    pub vertex_indices: [usize; 2],
    pub triangle_indices: [Option<usize>; 2],
    pub midpoint: Vec3,
    pub length: f32,
}

impl NavMeshEdge {
    pub fn new(vertex_index_a: usize, vertex_index_b: usize, vertices: &[Vec3]) -> Self {
        let sorted = if vertex_index_a < vertex_index_b {
            [vertex_index_a, vertex_index_b]
        } else {
            [vertex_index_b, vertex_index_a]
        };

        let vertex_a = vertices[sorted[0]];
        let vertex_b = vertices[sorted[1]];
        let midpoint = (vertex_a + vertex_b) * 0.5;
        let length = nalgebra_glm::distance(&vertex_a, &vertex_b);

        Self {
            vertex_indices: sorted,
            triangle_indices: [None, None],
            midpoint,
            length,
        }
    }

    pub fn key(&self) -> (usize, usize) {
        (self.vertex_indices[0], self.vertex_indices[1])
    }

    pub fn is_shared(&self) -> bool {
        self.triangle_indices[0].is_some() && self.triangle_indices[1].is_some()
    }

    pub fn get_other_triangle(&self, triangle_index: usize) -> Option<usize> {
        if self.triangle_indices[0] == Some(triangle_index) {
            self.triangle_indices[1]
        } else if self.triangle_indices[1] == Some(triangle_index) {
            self.triangle_indices[0]
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct NavMeshConnection {
    pub target_triangle: usize,
    pub shared_edge: usize,
    pub cost: f32,
}

#[derive(Debug, Clone)]
pub struct SpatialHash {
    pub cell_size: f32,
    pub cells: HashMap<(i32, i32), Vec<usize>>,
}

impl SpatialHash {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }

    pub fn cell_coords(&self, position: Vec3) -> (i32, i32) {
        let cell_x = (position.x / self.cell_size).floor() as i32;
        let cell_z = (position.z / self.cell_size).floor() as i32;
        (cell_x, cell_z)
    }

    pub fn insert(&mut self, triangle_index: usize, center: Vec3) {
        let coords = self.cell_coords(center);
        self.cells.entry(coords).or_default().push(triangle_index);
    }

    pub fn insert_with_bounds(
        &mut self,
        triangle_index: usize,
        min_bounds: Vec3,
        max_bounds: Vec3,
    ) {
        let min_cell = self.cell_coords(min_bounds);
        let max_cell = self.cell_coords(max_bounds);

        for cell_x in min_cell.0..=max_cell.0 {
            for cell_z in min_cell.1..=max_cell.1 {
                self.cells
                    .entry((cell_x, cell_z))
                    .or_default()
                    .push(triangle_index);
            }
        }
    }

    pub fn query(&self, position: Vec3) -> &[usize] {
        let coords = self.cell_coords(position);
        self.cells.get(&coords).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn query_radius(&self, position: Vec3, radius: f32) -> Vec<usize> {
        let min_pos = position - nalgebra_glm::vec3(radius, 0.0, radius);
        let max_pos = position + nalgebra_glm::vec3(radius, 0.0, radius);

        let min_cell = self.cell_coords(min_pos);
        let max_cell = self.cell_coords(max_pos);

        let mut result = Vec::new();

        for cell_x in min_cell.0..=max_cell.0 {
            for cell_z in min_cell.1..=max_cell.1 {
                if let Some(triangles) = self.cells.get(&(cell_x, cell_z)) {
                    for &triangle_index in triangles {
                        if !result.contains(&triangle_index) {
                            result.push(triangle_index);
                        }
                    }
                }
            }
        }

        result
    }
}
