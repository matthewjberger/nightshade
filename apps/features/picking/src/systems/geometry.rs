use crate::theme::{DEBUG_VOLUME_COLOR, DEBUG_VOLUME_SEGMENTS};
use nightshade::prelude::*;
use nightshade_render::components::lines::{Line, Lines};

pub fn sphere_wireframe(radius: f32, color: Vec4, segments: u32) -> Vec<Line> {
    let mut lines = Vec::new();
    for i in 0..segments {
        let phi = std::f32::consts::PI * (i as f32 / segments as f32);
        let y = radius * phi.cos();
        let circle_radius = radius * phi.sin();
        for j in 0..segments {
            let theta1 = 2.0 * std::f32::consts::PI * (j as f32 / segments as f32);
            let theta2 = 2.0 * std::f32::consts::PI * ((j + 1) as f32 / segments as f32);
            lines.push(Line {
                start: Vec3::new(
                    circle_radius * theta1.cos(),
                    y,
                    circle_radius * theta1.sin(),
                ),
                end: Vec3::new(
                    circle_radius * theta2.cos(),
                    y,
                    circle_radius * theta2.sin(),
                ),
                color,
            });
        }
    }
    for i in 0..segments {
        let theta = 2.0 * std::f32::consts::PI * (i as f32 / segments as f32);
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();
        for j in 0..segments {
            let phi1 = std::f32::consts::PI * (j as f32 / segments as f32);
            let phi2 = std::f32::consts::PI * ((j + 1) as f32 / segments as f32);
            lines.push(Line {
                start: Vec3::new(
                    radius * phi1.sin() * cos_theta,
                    radius * phi1.cos(),
                    radius * phi1.sin() * sin_theta,
                ),
                end: Vec3::new(
                    radius * phi2.sin() * cos_theta,
                    radius * phi2.cos(),
                    radius * phi2.sin() * sin_theta,
                ),
                color,
            });
        }
    }
    lines
}

pub fn preview_marker(position: Vec3, normal: Vec3, size: f32) -> Lines {
    let normal = if normal.norm() > 1e-4 {
        nalgebra_glm::normalize(&normal)
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    };
    let helper = if normal.x.abs() < 0.9 {
        Vec3::new(1.0, 0.0, 0.0)
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    };
    let tangent = nalgebra_glm::normalize(&nalgebra_glm::cross(&normal, &helper));
    let bitangent = nalgebra_glm::cross(&normal, &tangent);
    let half = size * 0.5;
    let color = vec4(0.0, 1.0, 0.0, 1.0);
    let normal_color = vec4(1.0, 1.0, 0.0, 1.0);

    let corners = [
        position + tangent * half + bitangent * half,
        position - tangent * half + bitangent * half,
        position - tangent * half - bitangent * half,
        position + tangent * half - bitangent * half,
    ];
    let mut lines_vec = Vec::with_capacity(5);
    for i in 0..4 {
        lines_vec.push(Line {
            start: corners[i],
            end: corners[(i + 1) % 4],
            color,
        });
    }
    lines_vec.push(Line {
        start: position,
        end: position + normal * size,
        color: normal_color,
    });
    Lines::new(lines_vec)
}

pub fn debug_volume_wireframe(radius: f32) -> Vec<Line> {
    sphere_wireframe(radius, DEBUG_VOLUME_COLOR, DEBUG_VOLUME_SEGMENTS)
}
