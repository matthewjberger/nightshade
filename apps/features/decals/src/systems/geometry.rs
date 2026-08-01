use crate::theme::{PREVIEW_COLOR, PREVIEW_NORMAL_COLOR};
use nightshade::prelude::*;
use nightshade_render::components::lines::{Line, Lines};

pub fn rotation_from_normal(normal: Vec3) -> Quat {
    let forward = Vec3::new(0.0, 0.0, -1.0);
    let normal = nalgebra_glm::normalize(&normal);

    if (normal - forward).norm() < 0.001 {
        return Quat::identity();
    }

    if (normal + forward).norm() < 0.001 {
        return nalgebra_glm::quat_angle_axis(std::f32::consts::PI, &Vec3::new(0.0, 1.0, 0.0));
    }

    let axis = nalgebra_glm::normalize(&nalgebra_glm::cross(&forward, &normal));
    let angle = nalgebra_glm::dot(&forward, &normal).acos();

    nalgebra_glm::quat_angle_axis(angle, &axis)
}

pub fn decal_preview_lines(position: Vec3, normal: Vec3, size: f32) -> Lines {
    let half_size = size / 2.0;
    let depth = size;

    let rotation = nalgebra_glm::quat_to_mat3(&rotation_from_normal(normal));
    let transform_point = |local: Vec3| -> Vec3 { position + rotation * local };

    let front: Vec<Vec3> = [
        Vec3::new(-half_size, -half_size, 0.0),
        Vec3::new(half_size, -half_size, 0.0),
        Vec3::new(half_size, half_size, 0.0),
        Vec3::new(-half_size, half_size, 0.0),
    ]
    .into_iter()
    .map(transform_point)
    .collect();

    let back: Vec<Vec3> = [
        Vec3::new(-half_size, -half_size, depth),
        Vec3::new(half_size, -half_size, depth),
        Vec3::new(half_size, half_size, depth),
        Vec3::new(-half_size, half_size, depth),
    ]
    .into_iter()
    .map(transform_point)
    .collect();

    let color = Vec4::from(PREVIEW_COLOR);
    let mut lines = Vec::with_capacity(13);

    for index in 0..4 {
        let next = (index + 1) % 4;
        lines.push(Line {
            start: front[index],
            end: front[next],
            color,
        });
        lines.push(Line {
            start: back[index],
            end: back[next],
            color,
        });
        lines.push(Line {
            start: front[index],
            end: back[index],
            color,
        });
    }

    lines.push(Line {
        start: position,
        end: transform_point(Vec3::new(0.0, 0.0, depth * 2.0)),
        color: Vec4::from(PREVIEW_NORMAL_COLOR),
    });

    Lines::new(lines)
}
