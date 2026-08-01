use crate::data;
use crate::resources::Floor;
use nalgebra_glm::Vec2;
use nightshade_platform::prelude::Window;
use nightshade_render::resources::viewport::Viewport;

pub fn cell_center(column: u32, row: u32) -> Vec2 {
    Vec2::new(column as f32 + 0.5, row as f32 + 0.5)
}

pub fn solid_at(floor: &Floor, point: Vec2) -> bool {
    if point.x < 0.0 || point.y < 0.0 {
        return true;
    }
    let column = point.x as u32;
    let row = point.y as u32;
    if column >= floor.columns || row >= floor.rows {
        return true;
    }
    floor.solid[(row * floor.columns + column) as usize]
}

pub fn blocked(floor: &Floor, position: Vec2, radius: f32) -> bool {
    let corners = [
        Vec2::new(position.x - radius, position.y - radius),
        Vec2::new(position.x + radius, position.y - radius),
        Vec2::new(position.x - radius, position.y + radius),
        Vec2::new(position.x + radius, position.y + radius),
    ];
    corners.iter().any(|corner| solid_at(floor, *corner))
}

pub fn clear_line(floor: &Floor, from: Vec2, to: Vec2) -> bool {
    let span = to - from;
    let distance = span.magnitude();
    if distance <= f32::EPSILON {
        return true;
    }
    let steps = (distance / 0.35).ceil() as u32;
    let step = span / steps as f32;
    (1..steps).all(|index| !solid_at(floor, from + step * index as f32))
}

pub fn distance_to_step(from: Vec2, to: Vec2, point: Vec2) -> f32 {
    let span = to - from;
    let length_squared = span.magnitude_squared();
    if length_squared <= f32::EPSILON {
        return (point - from).magnitude();
    }
    let share = ((point - from).dot(&span) / length_squared).clamp(0.0, 1.0);
    (point - (from + span * share)).magnitude()
}

pub fn viewport_rectangle(viewport: &Viewport, window: &Window) -> Option<(Vec2, Vec2)> {
    if let Some(rect) = viewport.active_viewport_rect
        && rect.width > 0.0
        && rect.height > 0.0
    {
        return Some((
            Vec2::new(rect.x, rect.y),
            Vec2::new(rect.width, rect.height),
        ));
    }
    match window.cached_viewport_size {
        Some((width, height)) if width > 0 && height > 0 => {
            Some((Vec2::zeros(), Vec2::new(width as f32, height as f32)))
        }
        _ => None,
    }
}

pub fn half_height(size: Vec2) -> f32 {
    (size.y / data::PIXELS_PER_UNIT / 2.0).max(1.0)
}

pub fn pointer_world_position(origin: Vec2, size: Vec2, camera: Vec2, pointer: Vec2) -> Vec2 {
    let normalized = Vec2::new(
        (pointer.x - origin.x) / size.x * 2.0 - 1.0,
        1.0 - (pointer.y - origin.y) / size.y * 2.0,
    );
    let half_height = half_height(size);
    let half_width = half_height * size.x / size.y;
    camera + Vec2::new(normalized.x * half_width, normalized.y * half_height)
}
