use nightshade::prelude::*;

pub const BOX_EDGES: [(usize, usize); 12] = [
    (0, 1),
    (1, 3),
    (3, 2),
    (2, 0),
    (4, 5),
    (5, 7),
    (7, 6),
    (6, 4),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
];

pub fn ring(
    origin: Vec3,
    first: Vec3,
    second: Vec3,
    radius: f32,
    color: Vec4,
) -> Vec<(Vec3, Vec3, Vec4)> {
    const STEPS: usize = 48;
    let mut lines = Vec::with_capacity(STEPS);
    let mut previous = origin + first * radius;
    for step in 1..=STEPS {
        let angle = step as f32 / STEPS as f32 * std::f32::consts::TAU;
        let point = origin + first * (radius * angle.cos()) + second * (radius * angle.sin());
        lines.push((previous, point, color));
        previous = point;
    }
    lines
}
