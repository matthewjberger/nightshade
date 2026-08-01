use nalgebra_glm::Vec3;

pub fn push_out(position: Vec3, radius: f32, blocks: &[(Vec3, Vec3)]) -> Vec3 {
    let mut moved = position;
    for (center, half) in blocks {
        let nearest = Vec3::new(
            moved.x.clamp(center.x - half.x, center.x + half.x),
            moved.y,
            moved.z.clamp(center.z - half.z, center.z + half.z),
        );
        let away = Vec3::new(moved.x - nearest.x, 0.0, moved.z - nearest.z);
        let distance = away.norm();
        if distance >= radius {
            continue;
        }
        if distance > 1.0e-4 {
            moved += away / distance * (radius - distance);
            continue;
        }
        let left = (moved.x - (center.x - half.x)).abs();
        let right = ((center.x + half.x) - moved.x).abs();
        let back = (moved.z - (center.z - half.z)).abs();
        let front = ((center.z + half.z) - moved.z).abs();
        let least = left.min(right).min(back).min(front);
        if least == left {
            moved.x = center.x - half.x - radius;
        } else if least == right {
            moved.x = center.x + half.x + radius;
        } else if least == back {
            moved.z = center.z - half.z - radius;
        } else {
            moved.z = center.z + half.z + radius;
        }
    }
    moved
}
