use crate::theme;
use nalgebra_glm::Vec3;
use nightshade_render::data::geometry::{Mesh, Vertex};

pub fn hands_ladder() -> Vec<usize> {
    let most = std::thread::available_parallelism()
        .map(|count| count.get().saturating_sub(1))
        .unwrap_or(1)
        .max(1);
    let mut ladder = Vec::new();
    let mut step = 1;
    while step < most {
        ladder.push(step);
        step *= 2;
    }
    ladder.push(most);
    ladder
}

pub fn stand_place(slot: usize) -> Vec3 {
    let across = theme::ACROSS;
    let column = (slot % across) as f32;
    let row = (slot / across) as f32;
    let half = (across as f32 - 1.0) * 0.5;
    Vec3::new(
        (column - half) * theme::STAND_SPAN,
        0.0,
        (row - half) * theme::STAND_SPAN,
    )
}

pub fn statue_name(slot: usize) -> String {
    format!("workers_statue_{slot}")
}

pub fn floor_mesh() -> Mesh {
    let reach = theme::ACROSS as f32 * theme::STAND_SPAN * 0.72;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    face(
        &mut vertices,
        &mut indices,
        Vec3::new(-reach, 0.0, -reach),
        Vec3::new(0.0, 0.0, reach * 2.0),
        Vec3::new(reach * 2.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(1.0, 1.0, 1.0),
    );
    Mesh::new(vertices, indices)
}

pub fn box_mesh(wide: f32, tall: f32) -> Mesh {
    let half = wide * 0.5;
    let white = Vec3::new(1.0, 1.0, 1.0);
    let across = Vec3::new(wide, 0.0, 0.0);
    let along = Vec3::new(0.0, 0.0, wide);
    let up = Vec3::new(0.0, tall, 0.0);
    let low = Vec3::new(-half, 0.0, -half);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let sides = [
        (low + up, along, across, Vec3::new(0.0, 1.0, 0.0)),
        (low, across, along, Vec3::new(0.0, -1.0, 0.0)),
        (low + across, up, along, Vec3::new(1.0, 0.0, 0.0)),
        (low, along, up, Vec3::new(-1.0, 0.0, 0.0)),
        (low + along, across, up, Vec3::new(0.0, 0.0, 1.0)),
        (low, up, across, Vec3::new(0.0, 0.0, -1.0)),
    ];
    for (origin, first, second, normal) in sides {
        face(
            &mut vertices,
            &mut indices,
            origin,
            first,
            second,
            normal,
            white,
        );
    }
    Mesh::new(vertices, indices)
}

fn face(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    origin: Vec3,
    across: Vec3,
    along: Vec3,
    normal: Vec3,
    color: Vec3,
) {
    let base = vertices.len() as u32;
    let corners = [
        origin,
        origin + across,
        origin + across + along,
        origin + along,
    ];
    for corner in corners {
        let mut vertex = Vertex::new(corner, normal);
        vertex.color = [color.x, color.y, color.z, 1.0];
        vertices.push(vertex);
    }
    indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
}
