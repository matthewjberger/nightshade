use nalgebra_glm::Vec2;

pub fn inside(pen: Vec2, spot: Vec2) -> bool {
    spot.x.abs() <= pen.x && spot.y.abs() <= pen.y
}

pub fn held(pen: Vec2, spot: Vec2) -> Vec2 {
    Vec2::new(spot.x.clamp(-pen.x, pen.x), spot.y.clamp(-pen.y, pen.y))
}
