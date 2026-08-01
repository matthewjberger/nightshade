use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};

use crate::components::*;

pub fn ui_canvas_hit_test(
    world: &nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
    position: Vec2,
) -> Option<u32> {
    if let Some(data) = world.get::<crate::components::UiCanvasData>(entity) {
        for &(command_id, ref rect) in data.command_bounds.iter().rev() {
            if rect.contains(position) {
                return Some(command_id);
            }
        }
    }
    None
}

pub fn ui_canvas_tag(
    world: &mut nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
    id: u32,
) {
    let Some(data) = world.get_mut::<crate::components::UiCanvasData>(entity) else {
        return;
    };
    let Some(command) = data.commands.last_mut() else {
        return;
    };
    let slot = match command {
        CanvasCommand::Rect { id, .. }
        | CanvasCommand::Circle { id, .. }
        | CanvasCommand::Line { id, .. }
        | CanvasCommand::Text { id, .. }
        | CanvasCommand::Polyline { id, .. }
        | CanvasCommand::RectStroke { id, .. }
        | CanvasCommand::CircleStroke { id, .. }
        | CanvasCommand::Arc { id, .. }
        | CanvasCommand::QuadraticBezier { id, .. }
        | CanvasCommand::CubicBezier { id, .. } => id,
    };
    *slot = Some(id);
}

pub fn ui_canvas_clear(world: &mut nightshade::ecs::Storage, entity: nightshade::ecs::Entity) {
    if let Some(data) = world.get_mut::<crate::components::UiCanvasData>(entity) {
        data.commands.clear();
    }
}

pub fn ui_canvas_rect(
    world: &mut nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
    position: Vec2,
    size: Vec2,
    color: Vec4,
    corner_radius: f32,
) {
    if let Some(data) = world.get_mut::<crate::components::UiCanvasData>(entity) {
        data.commands.push(CanvasCommand::Rect {
            position,
            size,
            color,
            corner_radius,
            id: None,
        });
    }
}

pub fn ui_canvas_circle(
    world: &mut nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
    center: Vec2,
    radius: f32,
    color: Vec4,
) {
    if let Some(data) = world.get_mut::<crate::components::UiCanvasData>(entity) {
        data.commands.push(CanvasCommand::Circle {
            center,
            radius,
            color,
            id: None,
        });
    }
}

pub fn ui_canvas_line(
    world: &mut nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
    from: Vec2,
    to: Vec2,
    thickness: f32,
    color: Vec4,
) {
    if let Some(data) = world.get_mut::<crate::components::UiCanvasData>(entity) {
        data.commands.push(CanvasCommand::Line {
            from,
            to,
            thickness,
            color,
            id: None,
        });
    }
}

pub fn ui_canvas_text(
    world: &mut nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
    text: &str,
    position: Vec2,
    font_size: f32,
    color: Vec4,
) {
    if let Some(data) = world.get_mut::<crate::components::UiCanvasData>(entity) {
        data.commands.push(CanvasCommand::Text {
            text: text.to_string(),
            position,
            font_size,
            color,
            id: None,
        });
    }
}

pub fn ui_canvas_polyline(
    world: &mut nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
    points: &[Vec2],
    thickness: f32,
    color: Vec4,
    closed: bool,
) {
    if let Some(data) = world.get_mut::<crate::components::UiCanvasData>(entity) {
        data.commands.push(CanvasCommand::Polyline {
            points: points.to_vec(),
            thickness,
            color,
            closed,
            id: None,
        });
    }
}

pub fn ui_canvas_rect_stroke(
    world: &mut nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
    position: Vec2,
    size: Vec2,
    color: Vec4,
    thickness: f32,
    corner_radius: f32,
) {
    if let Some(data) = world.get_mut::<crate::components::UiCanvasData>(entity) {
        data.commands.push(CanvasCommand::RectStroke {
            position,
            size,
            color,
            thickness,
            corner_radius,
            id: None,
        });
    }
}

pub fn ui_canvas_arc(
    world: &mut nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
    center: Vec2,
    radius: f32,
    angle_range: (f32, f32),
    thickness: f32,
    color: Vec4,
) {
    if let Some(data) = world.get_mut::<crate::components::UiCanvasData>(entity) {
        data.commands.push(CanvasCommand::Arc {
            center,
            radius,
            start_angle: angle_range.0,
            end_angle: angle_range.1,
            thickness,
            color,
            id: None,
        });
    }
}

pub fn ui_canvas_quadratic_bezier(
    world: &mut nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
    start: Vec2,
    control: Vec2,
    end: Vec2,
    thickness: f32,
    color: Vec4,
) {
    if let Some(data) = world.get_mut::<crate::components::UiCanvasData>(entity) {
        data.commands.push(CanvasCommand::QuadraticBezier {
            start,
            control,
            end,
            thickness,
            color,
            id: None,
        });
    }
}
