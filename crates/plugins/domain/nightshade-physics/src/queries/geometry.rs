use crate::resources::PhysicsWorld;

pub fn static_collider_geometry(physics: &PhysicsWorld) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<[u32; 3]> = Vec::new();

    for (_, collider) in physics.collider_set.iter() {
        let is_static = match collider.parent() {
            Some(parent) => physics
                .rigid_body_set
                .get(parent)
                .is_some_and(|body| !body.is_dynamic()),
            None => true,
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
            const CUBE_TRIANGLES: [[u32; 3]; 12] = [
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
            for triangle in &CUBE_TRIANGLES {
                indices.push([
                    base_index + triangle[0],
                    base_index + triangle[1],
                    base_index + triangle[2],
                ]);
            }
        }
    }

    (vertices, indices)
}
