use super::resources::{NavMeshConnection, NavMeshTriangle, NavMeshWorld, SpatialHash};
#[cfg(feature = "physics")]
use crate::ecs::world::World;
use glam::{UVec3, Vec3A};
use rerecast::{
    AreaType, BuildContoursFlags, ConfigBuilder, DetailNavmesh, HeightfieldBuilder, PolygonNavmesh,
    TriMesh,
};

#[derive(Debug, Clone)]
pub struct RecastNavMeshConfig {
    pub agent_radius: f32,
    pub agent_height: f32,
    pub cell_size_fraction: f32,
    pub cell_height_fraction: f32,
    pub walkable_climb: f32,
    pub walkable_slope_angle: f32,
    pub min_region_size: u16,
    pub merge_region_size: u16,
    pub max_simplification_error: f32,
    pub edge_max_len_factor: u16,
    pub max_vertices_per_polygon: u16,
    pub detail_sample_dist: f32,
    pub detail_sample_max_error: f32,
}

impl Default for RecastNavMeshConfig {
    fn default() -> Self {
        Self {
            agent_radius: 0.4,
            agent_height: 1.8,
            cell_size_fraction: 3.0,
            cell_height_fraction: 6.0,
            walkable_climb: 0.4,
            walkable_slope_angle: std::f32::consts::FRAC_PI_4,
            min_region_size: 8,
            merge_region_size: 20,
            max_simplification_error: 1.3,
            edge_max_len_factor: 12,
            max_vertices_per_polygon: 6,
            detail_sample_dist: 6.0,
            detail_sample_max_error: 1.0,
        }
    }
}

pub fn generate_navmesh_recast(
    vertices: &[[f32; 3]],
    indices: &[[u32; 3]],
    config: &RecastNavMeshConfig,
) -> Option<NavMeshWorld> {
    let mut trimesh = TriMesh::default();

    for vertex in vertices {
        trimesh
            .vertices
            .push(Vec3A::new(vertex[0], vertex[1], vertex[2]));
    }

    for triangle in indices {
        trimesh
            .indices
            .push(UVec3::new(triangle[0], triangle[1], triangle[2]));
        trimesh.area_types.push(AreaType::DEFAULT_WALKABLE);
    }

    if trimesh.indices.is_empty() {
        return None;
    }

    let aabb = trimesh.compute_aabb()?;

    let recast_config = ConfigBuilder {
        agent_radius: config.agent_radius,
        agent_height: config.agent_height,
        cell_size_fraction: config.cell_size_fraction,
        cell_height_fraction: config.cell_height_fraction,
        walkable_climb: config.walkable_climb,
        walkable_slope_angle: config.walkable_slope_angle,
        min_region_size: config.min_region_size,
        merge_region_size: config.merge_region_size,
        max_simplification_error: config.max_simplification_error,
        edge_max_len_factor: config.edge_max_len_factor,
        max_vertices_per_polygon: config.max_vertices_per_polygon,
        detail_sample_dist: config.detail_sample_dist,
        detail_sample_max_error: config.detail_sample_max_error,
        aabb,
        ..Default::default()
    }
    .build();

    trimesh.mark_walkable_triangles(recast_config.walkable_slope_angle);

    let heightfield_builder = HeightfieldBuilder {
        aabb,
        cell_size: recast_config.cell_size,
        cell_height: recast_config.cell_height,
    };

    let mut heightfield = heightfield_builder.build().ok()?;

    heightfield
        .rasterize_triangles(&trimesh, recast_config.walkable_climb)
        .ok()?;
    heightfield.filter_low_hanging_walkable_obstacles(recast_config.walkable_climb);
    heightfield.filter_ledge_spans(recast_config.walkable_height, recast_config.walkable_climb);
    heightfield.filter_walkable_low_height_spans(recast_config.walkable_height);

    let mut compact = heightfield
        .into_compact(recast_config.walkable_height, recast_config.walkable_climb)
        .ok()?;
    compact.erode_walkable_area(recast_config.walkable_radius);
    compact.build_distance_field();
    compact
        .build_regions(
            recast_config.border_size,
            recast_config.min_region_area,
            recast_config.merge_region_area,
        )
        .ok()?;

    let contours = compact.build_contours(
        recast_config.max_simplification_error,
        recast_config.max_edge_len,
        BuildContoursFlags::TESSELLATE_SOLID_WALL_EDGES,
    );

    let polygon_mesh = contours
        .into_polygon_mesh(recast_config.max_vertices_per_polygon)
        .ok()?;

    let detail_mesh = DetailNavmesh::new(
        &polygon_mesh,
        &compact,
        config.detail_sample_dist,
        config.detail_sample_max_error,
    )
    .ok()?;

    Some(convert_detail_mesh_to_navmesh(&polygon_mesh, &detail_mesh))
}

fn convert_detail_mesh_to_navmesh(
    polygon_mesh: &PolygonNavmesh,
    detail_mesh: &DetailNavmesh,
) -> NavMeshWorld {
    let mut navmesh = NavMeshWorld::new();
    navmesh.spatial_hash = SpatialHash::new(2.0);

    for vertex in &detail_mesh.vertices {
        navmesh
            .vertices
            .push(nalgebra_glm::vec3(vertex.x, vertex.y, vertex.z));
    }

    let mut triangle_to_polygon: Vec<usize> = Vec::new();
    let mut polygon_first_triangle: Vec<usize> = Vec::new();

    for (polygon_index, submesh) in detail_mesh.meshes.iter().enumerate() {
        polygon_first_triangle.push(navmesh.triangles.len());

        let base_vertex = submesh.base_vertex_index as usize;
        let triangle_count = submesh.triangle_count as usize;
        let base_triangle = submesh.base_triangle_index as usize;

        for triangle_offset in 0..triangle_count {
            let tri_index = base_triangle + triangle_offset;
            let tri_data = detail_mesh.triangles[tri_index];

            let vertex_indices = [
                base_vertex + (tri_data[0] as usize),
                base_vertex + (tri_data[1] as usize),
                base_vertex + (tri_data[2] as usize),
            ];

            let triangle = NavMeshTriangle::new(vertex_indices, &navmesh.vertices);
            navmesh.triangles.push(triangle);
            triangle_to_polygon.push(polygon_index);
        }
    }

    let max_verts = polygon_mesh.max_vertices_per_polygon as usize;

    for (triangle_index, &polygon_index) in triangle_to_polygon.iter().enumerate() {
        let poly_start = polygon_index * max_verts;
        let mut connections: Vec<NavMeshConnection> = Vec::new();

        for edge_offset in 0..max_verts {
            let neighbor_raw = polygon_mesh.polygon_neighbors[poly_start + edge_offset];
            if neighbor_raw == PolygonNavmesh::NO_CONNECTION || neighbor_raw & 0x8000 != 0 {
                continue;
            }

            let neighbor_polygon = neighbor_raw as usize;
            if neighbor_polygon >= polygon_first_triangle.len() {
                continue;
            }

            let neighbor_first_tri = polygon_first_triangle[neighbor_polygon];
            let neighbor_tri_count = if neighbor_polygon + 1 < polygon_first_triangle.len() {
                polygon_first_triangle[neighbor_polygon + 1] - neighbor_first_tri
            } else {
                navmesh.triangles.len() - neighbor_first_tri
            };

            for neighbor_tri_offset in 0..neighbor_tri_count {
                let neighbor_tri_index = neighbor_first_tri + neighbor_tri_offset;
                if neighbor_tri_index == triangle_index {
                    continue;
                }

                let already_connected = connections
                    .iter()
                    .any(|c| c.target_triangle == neighbor_tri_index);
                if already_connected {
                    continue;
                }

                let cost = nalgebra_glm::distance(
                    &navmesh.triangles[triangle_index].center,
                    &navmesh.triangles[neighbor_tri_index].center,
                );

                connections.push(NavMeshConnection {
                    target_triangle: neighbor_tri_index,
                    shared_edge: 0,
                    cost,
                });
            }
        }

        if !connections.is_empty() {
            navmesh.adjacency.insert(triangle_index, connections);
        }
    }

    for triangle_index in 0..navmesh.triangles.len() {
        for other_index in (triangle_index + 1)..navmesh.triangles.len() {
            if triangle_to_polygon[triangle_index] == triangle_to_polygon[other_index] {
                let cost = nalgebra_glm::distance(
                    &navmesh.triangles[triangle_index].center,
                    &navmesh.triangles[other_index].center,
                );

                navmesh
                    .adjacency
                    .entry(triangle_index)
                    .or_default()
                    .push(NavMeshConnection {
                        target_triangle: other_index,
                        shared_edge: 0,
                        cost,
                    });

                navmesh
                    .adjacency
                    .entry(other_index)
                    .or_default()
                    .push(NavMeshConnection {
                        target_triangle: triangle_index,
                        shared_edge: 0,
                        cost,
                    });
            }
        }
    }

    for (triangle_index, triangle) in navmesh.triangles.iter().enumerate() {
        let vertex_a = navmesh.vertices[triangle.vertex_indices[0]];
        let vertex_b = navmesh.vertices[triangle.vertex_indices[1]];
        let vertex_c = navmesh.vertices[triangle.vertex_indices[2]];

        let min_x = vertex_a.x.min(vertex_b.x).min(vertex_c.x);
        let max_x = vertex_a.x.max(vertex_b.x).max(vertex_c.x);
        let min_z = vertex_a.z.min(vertex_b.z).min(vertex_c.z);
        let max_z = vertex_a.z.max(vertex_b.z).max(vertex_c.z);

        navmesh.spatial_hash.insert_with_bounds(
            triangle_index,
            nalgebra_glm::vec3(min_x, 0.0, min_z),
            nalgebra_glm::vec3(max_x, 0.0, max_z),
        );
    }

    navmesh
}

#[cfg(feature = "physics")]
pub fn generate_navmesh_from_world(world: &mut World, config: &RecastNavMeshConfig) {
    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<[u32; 3]> = Vec::new();

    for (_, collider) in world.resources.physics.collider_set.iter() {
        let parent_handle = collider.parent();
        let is_static = if let Some(parent_handle) = parent_handle {
            world
                .resources
                .physics
                .rigid_body_set
                .get(parent_handle)
                .is_some_and(|rb| !rb.is_dynamic())
        } else {
            true
        };

        if !is_static {
            continue;
        }

        let position = collider.position();
        let shape = collider.shape();

        if let Some(trimesh) = shape.as_trimesh() {
            let base_index = vertices.len() as u32;
            for vertex in trimesh.vertices() {
                let world_vertex = position * vertex;
                vertices.push([world_vertex.x, world_vertex.y, world_vertex.z]);
            }
            for triangle in trimesh.indices() {
                indices.push([
                    base_index + triangle[0],
                    base_index + triangle[1],
                    base_index + triangle[2],
                ]);
            }
        } else if let Some(cuboid) = shape.as_cuboid() {
            let half = cuboid.half_extents;
            let base_index = vertices.len() as u32;

            let corners = [
                rapier3d::na::Point3::new(-half.x, -half.y, -half.z),
                rapier3d::na::Point3::new(half.x, -half.y, -half.z),
                rapier3d::na::Point3::new(half.x, half.y, -half.z),
                rapier3d::na::Point3::new(-half.x, half.y, -half.z),
                rapier3d::na::Point3::new(-half.x, -half.y, half.z),
                rapier3d::na::Point3::new(half.x, -half.y, half.z),
                rapier3d::na::Point3::new(half.x, half.y, half.z),
                rapier3d::na::Point3::new(-half.x, half.y, half.z),
            ];

            for corner in &corners {
                let world_vertex = position * corner;
                vertices.push([world_vertex.x, world_vertex.y, world_vertex.z]);
            }

            let cube_triangles: [[u32; 3]; 12] = [
                [0, 1, 2],
                [0, 2, 3],
                [4, 6, 5],
                [4, 7, 6],
                [0, 4, 5],
                [0, 5, 1],
                [2, 6, 7],
                [2, 7, 3],
                [0, 3, 7],
                [0, 7, 4],
                [1, 5, 6],
                [1, 6, 2],
            ];

            for triangle in &cube_triangles {
                indices.push([
                    base_index + triangle[0],
                    base_index + triangle[1],
                    base_index + triangle[2],
                ]);
            }
        }
    }

    if vertices.is_empty() || indices.is_empty() {
        return;
    }

    if let Some(navmesh) = generate_navmesh_recast(&vertices, &indices, config) {
        world.resources.navmesh = navmesh;
    }
}
