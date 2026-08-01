use crate::theme;
use nalgebra_glm::{Vec3, lerp};
use nightshade_render::data::geometry::{Mesh, Vertex};
use std::f32::consts::{PI, TAU};

pub fn boat_mesh() -> Mesh {
    let plank = Vec3::new(0.42, 0.28, 0.16);
    let dark = Vec3::new(0.28, 0.18, 0.10);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let hull = [
        (Vec3::new(-0.7, 0.0, -1.5), Vec3::new(1.4, 0.18, 3.0), plank),
        (Vec3::new(-0.7, 0.0, -1.5), Vec3::new(0.16, 0.55, 3.0), dark),
        (Vec3::new(0.54, 0.0, -1.5), Vec3::new(0.16, 0.55, 3.0), dark),
        (Vec3::new(-0.7, 0.0, -1.5), Vec3::new(1.4, 0.55, 0.16), dark),
        (Vec3::new(-0.7, 0.0, 1.34), Vec3::new(1.4, 0.55, 0.16), dark),
        (Vec3::new(-0.7, 0.3, -0.2), Vec3::new(1.4, 0.14, 0.5), plank),
    ];
    for (corner, span, color) in hull {
        add_box(&mut vertices, &mut indices, corner, span, color);
    }
    Mesh::new(vertices, indices)
}

pub fn glider_mesh() -> Mesh {
    let crimson = Vec3::new(0.78, 0.16, 0.14);
    let cream = Vec3::new(0.95, 0.89, 0.76);
    let amber = Vec3::new(0.97, 0.58, 0.12);
    let tube = Vec3::new(0.24, 0.25, 0.28);
    let wire = Vec3::new(0.7, 0.72, 0.75);
    let grip = Vec3::new(0.07, 0.07, 0.08);
    let stripes = [
        cream, crimson, amber, crimson, cream, crimson, amber, crimson,
    ];
    let nose = Vec3::new(0.0, 0.1, -1.55);
    let tail = Vec3::new(0.0, 0.02, 0.5);
    let tips = [Vec3::new(-2.4, 0.0, 0.7), Vec3::new(2.4, 0.0, 0.7)];
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let chords = 5;
    for tip in tips {
        let sail = |span: f32, chord: f32| {
            let leading = lerp(&nose, &tip, span);
            let sweep = Vec3::new(0.0, 0.0, -0.28 * (span * PI).sin());
            let trailing = lerp(&tail, &tip, span) + sweep;
            let billow = 0.16 * (chord * PI).sin() * (1.0 - 0.7 * span);
            lerp(&leading, &trailing, chord) + Vec3::new(0.0, billow, 0.0)
        };
        for (span_index, color) in stripes.iter().enumerate() {
            let inner = span_index as f32 / stripes.len() as f32;
            let outer = (span_index + 1) as f32 / stripes.len() as f32;
            for chord_index in 0..chords {
                let front = chord_index as f32 / chords as f32;
                let back = (chord_index + 1) as f32 / chords as f32;
                add_sheet(
                    &mut vertices,
                    &mut indices,
                    [
                        sail(inner, front),
                        sail(outer, front),
                        sail(outer, back),
                        sail(inner, back),
                    ],
                    *color,
                );
            }
        }
        add_rod(
            &mut vertices,
            &mut indices,
            nose - Vec3::new(0.0, 0.03, 0.0),
            tip - Vec3::new(0.0, 0.03, 0.0),
            0.035,
            tube,
        );
    }
    let below = Vec3::new(0.0, 0.04, 0.0);
    add_rod(
        &mut vertices,
        &mut indices,
        nose - below,
        tail - below,
        0.03,
        tube,
    );
    add_rod(
        &mut vertices,
        &mut indices,
        lerp(&nose, &tips[0], 0.45) - below,
        lerp(&nose, &tips[1], 0.45) - below,
        0.025,
        tube,
    );
    let king_post = Vec3::new(0.0, 0.5, -0.3);
    add_rod(
        &mut vertices,
        &mut indices,
        Vec3::new(0.0, 0.1, -0.3),
        king_post,
        0.025,
        tube,
    );
    let apex = Vec3::new(0.0, -0.04, -0.3);
    let bar_middle = Vec3::new(0.0, -0.68, -0.4);
    let sides = [-1.0_f32, 1.0];
    let shoulders = sides.map(|side| Vec3::new(0.85 * side, -0.4, -0.36));
    let corners = sides.map(|side| Vec3::new(1.0 * side, -0.6, -0.4));
    let bar_ends = sides.map(|side| Vec3::new(0.8 * side, -0.64, -0.4));
    let bend = |side: usize, step: usize| {
        bezier(
            shoulders[side],
            corners[side],
            bar_ends[side],
            step as f32 / 6.0,
        )
    };
    let mut frame = vec![apex];
    frame.extend((0..=6).map(|step| bend(0, step)));
    frame
        .extend((1..8).map(|step| bezier(bar_ends[0], bar_middle, bar_ends[1], step as f32 / 8.0)));
    frame.extend((0..=6).rev().map(|step| bend(1, step)));
    frame.push(apex);
    for pair in frame.windows(2) {
        add_rod(&mut vertices, &mut indices, pair[0], pair[1], 0.028, tube);
    }
    for (from, to) in [(0.06, 0.2), (0.8, 0.94)] {
        add_rod(
            &mut vertices,
            &mut indices,
            bezier(bar_ends[0], bar_middle, bar_ends[1], from),
            bezier(bar_ends[0], bar_middle, bar_ends[1], to),
            0.042,
            grip,
        );
    }
    for anchor in [nose, tail, tips[0], tips[1]] {
        add_rod(&mut vertices, &mut indices, king_post, anchor, 0.008, wire);
        for corner in corners {
            add_rod(&mut vertices, &mut indices, corner, anchor, 0.008, wire);
        }
    }
    Mesh::new(vertices, indices)
}

fn bezier(start: Vec3, control: Vec3, end: Vec3, along: f32) -> Vec3 {
    lerp(
        &lerp(&start, &control, along),
        &lerp(&control, &end, along),
        along,
    )
}

fn add_sheet(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>, corners: [Vec3; 4], color: Vec3) {
    for triangle in [
        [corners[0], corners[1], corners[2]],
        [corners[0], corners[2], corners[3]],
    ] {
        let cross = (triangle[1] - triangle[0]).cross(&(triangle[2] - triangle[0]));
        if cross.norm() < 1e-6 {
            continue;
        }
        let normal = cross.normalize();
        let upper = if normal.y >= 0.0 { normal } else { -normal };
        add_triangle(vertices, indices, triangle, upper, color);
        add_triangle(vertices, indices, triangle, -upper, color * 0.72);
    }
}

fn add_rod(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    start: Vec3,
    end: Vec3,
    radius: f32,
    color: Vec3,
) {
    let direction = end - start;
    if direction.norm() < 1e-6 {
        return;
    }
    let axis = direction.normalize();
    let helper = if axis.y.abs() < 0.9 {
        Vec3::y()
    } else {
        Vec3::x()
    };
    let first = axis.cross(&helper).normalize();
    let second = axis.cross(&first);
    let sides = 6;
    for side in 0..sides {
        let near = side as f32 / sides as f32 * TAU;
        let far = (side + 1) as f32 / sides as f32 * TAU;
        let rim_near = first * near.cos() + second * near.sin();
        let rim_far = first * far.cos() + second * far.sin();
        let normal = (rim_near + rim_far).normalize();
        let quad = [
            start + rim_near * radius,
            end + rim_near * radius,
            end + rim_far * radius,
            start + rim_far * radius,
        ];
        add_triangle(
            vertices,
            indices,
            [quad[0], quad[1], quad[2]],
            normal,
            color,
        );
        add_triangle(
            vertices,
            indices,
            [quad[0], quad[2], quad[3]],
            normal,
            color,
        );
    }
}

fn add_triangle(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    triangle: [Vec3; 3],
    normal: Vec3,
    color: Vec3,
) {
    let [first, second, third] = triangle;
    let ordered = if (second - first).cross(&(third - first)).dot(&normal) >= 0.0 {
        [first, second, third]
    } else {
        [first, third, second]
    };
    let base = vertices.len() as u32;
    for point in ordered {
        vertices.push(painted(point, normal, color));
    }
    indices.extend([base, base + 1, base + 2]);
}

fn painted(point: Vec3, normal: Vec3, color: Vec3) -> Vertex {
    Vertex::with_all(
        point,
        normal,
        [0.0, 0.0],
        [0.0, 1.0],
        [1.0, 0.0, 0.0, 1.0],
        [color.x, color.y, color.z, 1.0],
    )
}

fn add_box(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    corner: Vec3,
    span: Vec3,
    color: Vec3,
) {
    let across = Vec3::new(span.x, 0.0, 0.0);
    let up = Vec3::new(0.0, span.y, 0.0);
    let along = Vec3::new(0.0, 0.0, span.z);
    let faces = [
        (corner + up, along, across, Vec3::new(0.0, 1.0, 0.0)),
        (corner, across, along, Vec3::new(0.0, -1.0, 0.0)),
        (corner + across, up, along, Vec3::new(1.0, 0.0, 0.0)),
        (corner, along, up, Vec3::new(-1.0, 0.0, 0.0)),
        (corner + along, across, up, Vec3::new(0.0, 0.0, 1.0)),
        (corner, up, across, Vec3::new(0.0, 0.0, -1.0)),
    ];
    for (origin, first, second, normal) in faces {
        let base = vertices.len() as u32;
        for point in [
            origin,
            origin + first,
            origin + first + second,
            origin + second,
        ] {
            vertices.push(painted(point, normal, color));
        }
        indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

pub fn player_box(position: Vec3) -> (Vec3, Vec3) {
    (
        Vec3::new(
            position.x - theme::PLAYER_HALF_WIDTH,
            position.y,
            position.z - theme::PLAYER_HALF_WIDTH,
        ),
        Vec3::new(
            position.x + theme::PLAYER_HALF_WIDTH,
            position.y + theme::PLAYER_HEIGHT,
            position.z + theme::PLAYER_HALF_WIDTH,
        ),
    )
}
