use crate::data::plane::Plane;
use nalgebra_glm::{Vec2, Vec3};

pub fn flatten(plane: Plane, spot: Vec3) -> Vec2 {
    match plane {
        Plane::Ground => Vec2::new(spot.x, spot.z),
        Plane::Side => Vec2::new(spot.x, spot.y),
    }
}

pub fn raise(plane: Plane, flat: Vec2) -> Vec3 {
    match plane {
        Plane::Ground => Vec3::new(flat.x, 0.0, flat.y),
        Plane::Side => Vec3::new(flat.x, flat.y, 0.0),
    }
}

pub fn across(plane: Plane, from: Vec3, to: Vec3) -> f32 {
    (flatten(plane, to) - flatten(plane, from)).magnitude()
}

pub fn lift(plane: Plane, spot: Vec3) -> f32 {
    match plane {
        Plane::Ground => spot.y,
        Plane::Side => spot.z,
    }
}
