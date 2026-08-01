use nalgebra_glm::Vec3;
use nightshade_plane::Plane;
use nightshade_plane::queries::{across, flatten};

pub fn in_arc(
    plane: Plane,
    from: Vec3,
    facing: Vec3,
    target: Vec3,
    reach: f32,
    half_angle: f32,
) -> bool {
    let distance = across(plane, from, target);
    if distance > reach || distance <= f32::EPSILON {
        return false;
    }
    let toward = flatten(plane, target) - flatten(plane, from);
    let ahead = flatten(plane, facing);
    let forward = ahead.magnitude();
    if forward <= f32::EPSILON {
        return false;
    }
    let dot = toward.dot(&ahead) / (distance * forward);
    dot.clamp(-1.0, 1.0).acos() <= half_angle
}

pub fn facing_from_yaw(yaw: f32) -> Vec3 {
    Vec3::new(-yaw.sin(), 0.0, -yaw.cos())
}
