//! Built-in mesh primitive generators.

use crate::geometry::{Mesh, Vertex};
use nalgebra_glm::vec3;
use std::f32::consts::PI;

/// Creates a unit cube centered at the origin (side length 1).
pub fn create_cube_mesh() -> Mesh {
    let positions = vec![
        vec3(-0.5, -0.5, 0.5),
        vec3(0.5, -0.5, 0.5),
        vec3(0.5, 0.5, 0.5),
        vec3(-0.5, 0.5, 0.5),
        vec3(-0.5, -0.5, -0.5),
        vec3(-0.5, 0.5, -0.5),
        vec3(0.5, 0.5, -0.5),
        vec3(0.5, -0.5, -0.5),
        vec3(0.5, -0.5, -0.5),
        vec3(0.5, 0.5, -0.5),
        vec3(0.5, 0.5, 0.5),
        vec3(0.5, -0.5, 0.5),
        vec3(-0.5, -0.5, -0.5),
        vec3(-0.5, -0.5, 0.5),
        vec3(-0.5, 0.5, 0.5),
        vec3(-0.5, 0.5, -0.5),
        vec3(-0.5, 0.5, -0.5),
        vec3(-0.5, 0.5, 0.5),
        vec3(0.5, 0.5, 0.5),
        vec3(0.5, 0.5, -0.5),
        vec3(-0.5, -0.5, -0.5),
        vec3(0.5, -0.5, -0.5),
        vec3(0.5, -0.5, 0.5),
        vec3(-0.5, -0.5, 0.5),
    ];

    let normals = vec![
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 0.0, -1.0),
        vec3(0.0, 0.0, -1.0),
        vec3(0.0, 0.0, -1.0),
        vec3(0.0, 0.0, -1.0),
        vec3(1.0, 0.0, 0.0),
        vec3(1.0, 0.0, 0.0),
        vec3(1.0, 0.0, 0.0),
        vec3(1.0, 0.0, 0.0),
        vec3(-1.0, 0.0, 0.0),
        vec3(-1.0, 0.0, 0.0),
        vec3(-1.0, 0.0, 0.0),
        vec3(-1.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, -1.0, 0.0),
        vec3(0.0, -1.0, 0.0),
        vec3(0.0, -1.0, 0.0),
        vec3(0.0, -1.0, 0.0),
    ];

    let uvs = vec![
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
        [0.0, 0.0],
        [0.0, 1.0],
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
        [0.0, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
        [0.0, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
        [0.0, 0.0],
    ];

    let indices = vec![
        0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4, 8, 9, 10, 10, 11, 8, 12, 13, 14, 14, 15, 12, 16, 17,
        18, 18, 19, 16, 20, 21, 22, 22, 23, 20,
    ];

    let vertices = positions
        .into_iter()
        .zip(normals)
        .zip(uvs)
        .map(|((pos, norm), uv)| Vertex::with_tex_coords(pos, norm, uv))
        .collect();

    Mesh::new(vertices, indices)
}

/// Creates a UV sphere with the given radius and segment count.
pub fn create_sphere_mesh(radius: f32, segments: u32) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=segments {
        let lat = PI * (i as f32) / segments as f32;
        let v = i as f32 / segments as f32;

        for j in 0..=segments {
            let lon = 2.0 * PI * (j as f32) / segments as f32;
            let u = j as f32 / segments as f32;

            let x = radius * lat.sin() * lon.cos();
            let y = radius * lat.cos();
            let z = radius * lat.sin() * lon.sin();

            let pos = vec3(x, y, z);
            let normal = nalgebra_glm::normalize(&pos);

            vertices.push(Vertex::with_tex_coords(pos, normal, [u, v]));
        }
    }

    for i in 0..segments {
        for j in 0..segments {
            let first = i * (segments + 1) + j;
            let second = first + segments + 1;

            indices.extend_from_slice(&[first, first + 1, second, second, first + 1, second + 1]);
        }
    }

    Mesh::new(vertices, indices)
}

/// Creates a horizontal plane (Y-up) with the given size.
pub fn create_plane_mesh(size: f32) -> Mesh {
    let half_size = size / 2.0;
    let positions = vec![
        vec3(-half_size, 0.0, -half_size),
        vec3(half_size, 0.0, -half_size),
        vec3(half_size, 0.0, half_size),
        vec3(-half_size, 0.0, half_size),
    ];

    let uvs = vec![[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];

    let normal = vec3(0.0, 1.0, 0.0);
    let vertices = positions
        .into_iter()
        .zip(uvs)
        .map(|(pos, uv)| Vertex::with_tex_coords(pos, normal, uv))
        .collect();

    let indices = vec![0, 2, 1, 2, 0, 3];

    Mesh::new(vertices, indices)
}

/// Creates a subdivided horizontal plane for terrain or deformation.
pub fn create_subdivided_plane_mesh(size: f32, subdivisions: u32) -> Mesh {
    let half = size / 2.0;
    let step = size / subdivisions as f32;
    let uv_step = 1.0 / subdivisions as f32;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let normal = vec3(0.0, 1.0, 0.0);

    for row in 0..=subdivisions {
        for col in 0..=subdivisions {
            let x = -half + col as f32 * step;
            let z = -half + row as f32 * step;
            let u = col as f32 * uv_step;
            let v = row as f32 * uv_step;

            vertices.push(Vertex::with_tex_coords(
                vec3(x, 0.0, z),
                normal,
                [u, 1.0 - v],
            ));
        }
    }

    let verts_per_row = subdivisions + 1;
    for row in 0..subdivisions {
        for col in 0..subdivisions {
            let top_left = row * verts_per_row + col;
            let top_right = top_left + 1;
            let bottom_left = top_left + verts_per_row;
            let bottom_right = bottom_left + 1;

            indices.push(top_left);
            indices.push(bottom_right);
            indices.push(top_right);

            indices.push(bottom_right);
            indices.push(top_left);
            indices.push(bottom_left);
        }
    }

    Mesh::new(vertices, indices)
}

/// Creates a cylinder along the Y axis with caps.
pub fn create_cylinder_mesh(radius: f32, height: f32, segments: u32) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let half_height = height / 2.0;

    for i in 0..=segments {
        let angle = 2.0 * PI * (i as f32) / segments as f32;
        let u = i as f32 / segments as f32;
        let x = radius * angle.cos();
        let z = radius * angle.sin();

        vertices.push(Vertex::with_tex_coords(
            vec3(x, half_height, z),
            vec3(angle.cos(), 0.0, angle.sin()),
            [u, 0.0],
        ));
        vertices.push(Vertex::with_tex_coords(
            vec3(x, -half_height, z),
            vec3(angle.cos(), 0.0, angle.sin()),
            [u, 1.0],
        ));
    }

    for i in 0..segments {
        let start = 2 * i;
        indices.extend_from_slice(&[start, start + 2, start + 1, start + 1, start + 2, start + 3]);
    }

    let top_center_idx = vertices.len() as u32;
    vertices.push(Vertex::with_tex_coords(
        vec3(0.0, half_height, 0.0),
        vec3(0.0, 1.0, 0.0),
        [0.5, 0.5],
    ));
    vertices.push(Vertex::with_tex_coords(
        vec3(0.0, -half_height, 0.0),
        vec3(0.0, -1.0, 0.0),
        [0.5, 0.5],
    ));

    for i in 0..segments {
        let angle = 2.0 * PI * (i as f32) / segments as f32;
        let u = angle.cos() * 0.5 + 0.5;
        let v = angle.sin() * 0.5 + 0.5;
        let x = radius * angle.cos();
        let z = radius * angle.sin();

        vertices.push(Vertex::with_tex_coords(
            vec3(x, half_height, z),
            vec3(0.0, 1.0, 0.0),
            [u, v],
        ));
        vertices.push(Vertex::with_tex_coords(
            vec3(x, -half_height, z),
            vec3(0.0, -1.0, 0.0),
            [u, v],
        ));
    }

    for i in 0..segments {
        let top_idx = top_center_idx + 2 + (i * 2);
        let next_top_idx = top_center_idx + 2 + (((i + 1) % segments) * 2);
        let bottom_idx = top_center_idx + 3 + (i * 2);
        let next_bottom_idx = top_center_idx + 3 + (((i + 1) % segments) * 2);

        indices.extend_from_slice(&[top_center_idx, next_top_idx, top_idx]);
        indices.extend_from_slice(&[top_center_idx + 1, bottom_idx, next_bottom_idx]);
    }

    Mesh::new(vertices, indices)
}

/// Creates a cone along the Y axis with a base cap.
pub fn create_cone_mesh(radius: f32, height: f32, segments: u32) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let half_height = height / 2.0;

    vertices.push(Vertex::with_tex_coords(
        vec3(0.0, half_height, 0.0),
        vec3(0.0, 1.0, 0.0),
        [0.5, 0.5],
    ));

    for i in 0..segments {
        let angle = 2.0 * PI * (i as f32) / segments as f32;
        let u = i as f32 / segments as f32;
        let x = radius * angle.cos();
        let z = radius * angle.sin();

        let side_normal = vec3(angle.cos(), 0.5, angle.sin());
        vertices.push(Vertex::with_tex_coords(
            vec3(x, -half_height, z),
            nalgebra_glm::normalize(&side_normal),
            [u, 1.0],
        ));
    }

    for i in 1..=segments {
        let next_index = if i == segments { 1 } else { i + 1 };
        indices.extend_from_slice(&[0, next_index, i]);
    }

    let base_center_idx = vertices.len() as u32;
    vertices.push(Vertex::with_tex_coords(
        vec3(0.0, -half_height, 0.0),
        vec3(0.0, -1.0, 0.0),
        [0.5, 0.5],
    ));

    for i in 0..segments {
        let angle = 2.0 * PI * (i as f32) / segments as f32;
        let u = angle.cos() * 0.5 + 0.5;
        let v = angle.sin() * 0.5 + 0.5;
        let x = radius * angle.cos();
        let z = radius * angle.sin();
        vertices.push(Vertex::with_tex_coords(
            vec3(x, -half_height, z),
            vec3(0.0, -1.0, 0.0),
            [u, v],
        ));
    }

    for i in 0..segments {
        let base_vertex = base_center_idx + 1 + i;
        let next_vertex = base_center_idx + 1 + ((i + 1) % segments);
        indices.extend_from_slice(&[base_center_idx, base_vertex, next_vertex]);
    }

    Mesh::new(vertices, indices)
}

/// Creates a torus (donut shape) in the XZ plane.
pub fn create_torus_mesh(
    major_radius: f32,
    minor_radius: f32,
    major_segments: u32,
    minor_segments: u32,
) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=major_segments {
        let major_angle = 2.0 * PI * (i as f32) / major_segments as f32;
        let u = i as f32 / major_segments as f32;
        let center_x = major_radius * major_angle.cos();
        let center_z = major_radius * major_angle.sin();

        for j in 0..=minor_segments {
            let minor_angle = 2.0 * PI * (j as f32) / minor_segments as f32;
            let v = j as f32 / minor_segments as f32;

            let x = minor_angle.cos() * minor_radius;
            let y = minor_angle.sin() * minor_radius;

            let pos = vec3(
                center_x + x * major_angle.cos(),
                y,
                center_z + x * major_angle.sin(),
            );

            let normal = vec3(
                major_angle.cos() * minor_angle.cos(),
                minor_angle.sin(),
                major_angle.sin() * minor_angle.cos(),
            );

            vertices.push(Vertex::with_tex_coords(
                pos,
                nalgebra_glm::normalize(&normal),
                [u, v],
            ));
        }
    }

    for i in 0..major_segments {
        for j in 0..minor_segments {
            let current = i * (minor_segments + 1) + j;
            let next_major = ((i + 1) % major_segments) * (minor_segments + 1) + j;

            indices.extend_from_slice(&[
                current,
                current + 1,
                next_major,
                next_major,
                current + 1,
                next_major + 1,
            ]);
        }
    }

    Mesh::new(vertices, indices)
}
