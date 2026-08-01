use crate::data::wire::{Body, PeerId};
use crate::theme;
use nalgebra_glm::Vec3;
use std::collections::VecDeque;

pub fn bodies_at(history: &VecDeque<(u32, Vec<Body>)>, tick: u32) -> Option<&Vec<Body>> {
    let mut found: Option<&(u32, Vec<Body>)> = None;
    for entry in history {
        if entry.0 <= tick && found.map(|best| best.0 <= entry.0).unwrap_or(true) {
            found = Some(entry);
        }
    }
    found.or_else(|| history.front()).map(|entry| &entry.1)
}

pub fn tag_target(
    shooter: PeerId,
    origin: Vec3,
    direction: Vec3,
    bodies: &[Body],
) -> Option<(PeerId, Vec3)> {
    if direction.norm() < 1e-4 {
        return None;
    }
    let direction = direction.normalize();
    let mut nearest: Option<(f32, PeerId)> = None;

    for body in bodies {
        if body.peer == shooter {
            continue;
        }
        let foot = Vec3::new(body.position[0], body.position[1], body.position[2]);
        let Some(distance) = ray_meets_body(origin, direction, foot) else {
            continue;
        };
        if nearest.map(|(near, _)| distance < near).unwrap_or(true) {
            nearest = Some((distance, body.peer));
        }
    }
    nearest.map(|(distance, peer)| (peer, origin + direction * distance))
}

fn ray_meets_body(origin: Vec3, direction: Vec3, foot: Vec3) -> Option<f32> {
    let low = foot + Vec3::new(0.0, theme::TAG_LOW, 0.0);
    let axis = Vec3::new(0.0, theme::TAG_HIGH - theme::TAG_LOW, 0.0);
    let between = origin - low;
    let axis_squared = axis.dot(&axis);
    let along = direction.dot(&axis);
    let forward_term = direction.dot(&between);
    let axis_term = axis.dot(&between);
    let denominator = axis_squared - along * along;
    let height = if denominator.abs() > 1e-5 {
        ((axis_term - along * forward_term) / denominator).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let distance = height * along - forward_term;
    if distance < 0.0 || distance > theme::TAG_RANGE {
        return None;
    }
    let on_ray = origin + direction * distance;
    let on_body = low + axis * height;
    if (on_ray - on_body).norm() > theme::TAG_RADIUS {
        return None;
    }
    Some(distance)
}
