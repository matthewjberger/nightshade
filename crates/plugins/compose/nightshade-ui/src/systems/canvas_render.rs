use nalgebra_glm::Vec4;

use crate::components::CanvasCommand;
use crate::components::layer::UiLayer;
use nightshade_paint::data::DrawRect;
use nightshade_paint::data::DrawText;
use nightshade_paint::data::Rect;
use nightshade_text::contract::data::text_data::{TextAlignment, TextMesh, VerticalAlignment};

struct CanvasEmit {
    entity: nightshade::ecs::Entity,
    commands: Vec<CanvasCommand>,
    origin: nalgebra_glm::Vec2,
    clip_rect: Option<Rect>,
    layer: UiLayer,
    z_index: i32,
    hit_test_enabled: bool,
}

fn push_canvas_line(
    sync: &mut crate::systems::surface::RenderSync<'_>,
    canvas: &CanvasEmit,
    from: nalgebra_glm::Vec2,
    to: nalgebra_glm::Vec2,
    thickness: f32,
    color: Vec4,
) {
    let delta = to - from;
    let length = delta.magnitude();
    if length < 0.001 {
        return;
    }
    let midpoint = (from + to) * 0.5;
    let angle = delta.y.atan2(delta.x);
    let rect_position = midpoint - nalgebra_glm::Vec2::new(length * 0.5, thickness * 0.5);
    sync.frame.rects.push(DrawRect {
        position: rect_position,
        size: nalgebra_glm::Vec2::new(length, thickness),
        color,
        corner_radius: 0.0,
        border_width: 0.0,
        border_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
        rotation: angle,
        clip_rect: canvas.clip_rect,
        layer: canvas.layer as u8,
        z_index: canvas.z_index + 1,
        shadow: None,
        effect_kind: 0,
        effect_params: [0.0; 4],
        quad_corners: None,
    });
}

pub(super) fn emit_canvas_commands(
    sync: &mut crate::systems::surface::RenderSync<'_>,
    storage: &mut nightshade::ecs::Storage,
    dpi_scale: f32,
) {
    let canvas_entities: Vec<nightshade::ecs::Entity> = storage
        .query_entities::<(crate::components::UiCanvasData,)>()
        .collect();

    let mut canvases = Vec::new();

    for entity in canvas_entities {
        if !crate::commands::widgets::world_layout::ui_chain_visible(
            &mut |entity| {
                storage
                    .get::<crate::components::UiLayoutNode>(entity)
                    .map(|node| node.visible)
            },
            &mut |entity| {
                storage
                    .get::<nightshade::ecs::ChildOf>(entity)
                    .map(|child_of| child_of.0)
            },
            entity,
        ) {
            continue;
        }

        let (origin, clip_rect, layer, z_index) =
            if let Some(node) = storage.get::<crate::components::UiLayoutNode>(entity) {
                let node_layer = node.computed_layer.unwrap_or(UiLayer::Background);
                let depth_z_index = (node.computed_depth * 100.0) as i32;
                (
                    node.computed_rect.min,
                    node.computed_clip_rect,
                    node_layer,
                    depth_z_index,
                )
            } else {
                continue;
            };

        let (commands, hit_test_enabled) =
            if let Some(data) = storage.get::<crate::components::UiCanvasData>(entity) {
                (data.commands.clone(), data.hit_test_enabled)
            } else {
                continue;
            };

        canvases.push(CanvasEmit {
            entity,
            commands,
            origin,
            clip_rect,
            layer,
            z_index,
            hit_test_enabled,
        });
    }

    for canvas in canvases {
        let mut bounds: Vec<(u32, Rect)> = Vec::new();

        for command in &canvas.commands {
            let command_id = match command {
                CanvasCommand::Rect { id, .. }
                | CanvasCommand::Circle { id, .. }
                | CanvasCommand::Line { id, .. }
                | CanvasCommand::Text { id, .. }
                | CanvasCommand::Polyline { id, .. }
                | CanvasCommand::RectStroke { id, .. }
                | CanvasCommand::CircleStroke { id, .. }
                | CanvasCommand::Arc { id, .. }
                | CanvasCommand::QuadraticBezier { id, .. }
                | CanvasCommand::CubicBezier { id, .. } => *id,
            };

            let command_bounds: Option<Rect> = match command {
                CanvasCommand::Rect {
                    position,
                    size,
                    color,
                    corner_radius,
                    ..
                } => {
                    sync.frame.rects.push(DrawRect {
                        position: canvas.origin + position,
                        size: *size,
                        color: *color,
                        corner_radius: *corner_radius,
                        border_width: 0.0,
                        border_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
                        rotation: 0.0,
                        clip_rect: canvas.clip_rect,
                        layer: canvas.layer as u8,
                        z_index: canvas.z_index + 1,
                        shadow: None,
                        effect_kind: 0,
                        effect_params: [0.0; 4],
                        quad_corners: None,
                    });
                    Some(Rect::from_min_max(*position, *position + *size))
                }
                CanvasCommand::Circle {
                    center,
                    radius,
                    color,
                    ..
                } => {
                    let top_left =
                        canvas.origin + center - nalgebra_glm::Vec2::new(*radius, *radius);
                    let diameter = *radius * 2.0;
                    sync.frame.rects.push(DrawRect {
                        position: top_left,
                        size: nalgebra_glm::Vec2::new(diameter, diameter),
                        color: *color,
                        corner_radius: *radius,
                        border_width: 0.0,
                        border_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
                        rotation: 0.0,
                        clip_rect: canvas.clip_rect,
                        layer: canvas.layer as u8,
                        z_index: canvas.z_index + 1,
                        shadow: None,
                        effect_kind: 0,
                        effect_params: [0.0; 4],
                        quad_corners: None,
                    });
                    let offset = nalgebra_glm::Vec2::new(*radius, *radius);
                    Some(Rect::from_min_max(center - offset, center + offset))
                }
                CanvasCommand::Line {
                    from,
                    to,
                    thickness,
                    color,
                    ..
                } => {
                    let world_from = canvas.origin + from;
                    let world_to = canvas.origin + to;
                    let delta = world_to - world_from;
                    let length = delta.magnitude();
                    if length < 0.001 {
                        continue;
                    }
                    let midpoint = (world_from + world_to) * 0.5;
                    let angle = delta.y.atan2(delta.x);
                    let rect_position =
                        midpoint - nalgebra_glm::Vec2::new(length * 0.5, *thickness * 0.5);
                    sync.frame.rects.push(DrawRect {
                        position: rect_position,
                        size: nalgebra_glm::Vec2::new(length, *thickness),
                        color: *color,
                        corner_radius: 0.0,
                        border_width: 0.0,
                        border_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
                        rotation: angle,
                        clip_rect: canvas.clip_rect,
                        layer: canvas.layer as u8,
                        z_index: canvas.z_index + 1,
                        shadow: None,
                        effect_kind: 0,
                        effect_params: [0.0; 4],
                        quad_corners: None,
                    });
                    let min_x = from.x.min(to.x) - *thickness * 0.5;
                    let min_y = from.y.min(to.y) - *thickness * 0.5;
                    let max_x = from.x.max(to.x) + *thickness * 0.5;
                    let max_y = from.y.max(to.y) + *thickness * 0.5;
                    Some(Rect::from_min_max(
                        nalgebra_glm::Vec2::new(min_x, min_y),
                        nalgebra_glm::Vec2::new(max_x, max_y),
                    ))
                }
                CanvasCommand::Text {
                    text,
                    position,
                    font_size,
                    color,
                    ..
                } => {
                    let physical_size = *font_size * dpi_scale;
                    sync.frame.text_meshes.push(DrawText {
                        entity: None,
                        text: text.clone(),
                        mesh: TextMesh::default(),
                        position: canvas.origin + position,
                        color: *color,
                        outline_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
                        outline_width: 0.0,
                        font_size: physical_size,
                        alignment: TextAlignment::Left,
                        vertical_alignment: VerticalAlignment::Top,
                        monospace: false,
                        font_kind: nightshade_text::contract::data::text_data::FontKind::Default,
                        wrap_width: None,
                        clip_rect: canvas.clip_rect,
                        layer: canvas.layer as u8,
                        z_index: canvas.z_index + 2,
                        rotation: 0.0,
                        pivot: nalgebra_glm::Vec2::new(0.0, 0.0),
                        character_colors: None,
                        character_background_colors: None,
                    });
                    None
                }
                CanvasCommand::Polyline {
                    points,
                    thickness,
                    color,
                    closed,
                    ..
                } => {
                    let segment_count = if *closed {
                        points.len()
                    } else {
                        points.len().saturating_sub(1)
                    };
                    for segment_index in 0..segment_count {
                        let from = canvas.origin + points[segment_index];
                        let to = canvas.origin + points[(segment_index + 1) % points.len()];
                        push_canvas_line(sync, &canvas, from, to, *thickness, *color);
                    }
                    if !points.is_empty() {
                        let mut min = points[0];
                        let mut max = points[0];
                        for point in points {
                            min.x = min.x.min(point.x);
                            min.y = min.y.min(point.y);
                            max.x = max.x.max(point.x);
                            max.y = max.y.max(point.y);
                        }
                        Some(Rect::from_min_max(min, max))
                    } else {
                        None
                    }
                }
                CanvasCommand::RectStroke {
                    position,
                    size,
                    color,
                    thickness,
                    ..
                } => {
                    let origin = canvas.origin + position;
                    let corners = [
                        origin,
                        origin + nalgebra_glm::Vec2::new(size.x, 0.0),
                        origin + *size,
                        origin + nalgebra_glm::Vec2::new(0.0, size.y),
                    ];
                    for edge_index in 0..4 {
                        let from = corners[edge_index];
                        let to = corners[(edge_index + 1) % 4];
                        push_canvas_line(sync, &canvas, from, to, *thickness, *color);
                    }
                    Some(Rect::from_min_max(*position, *position + *size))
                }
                CanvasCommand::CircleStroke {
                    center,
                    radius,
                    color,
                    thickness,
                    ..
                } => {
                    let segment_count = ((*radius * 2.0) as usize).max(16);
                    let world_center = canvas.origin + center;
                    for segment_index in 0..segment_count {
                        let angle_a =
                            (segment_index as f32 / segment_count as f32) * std::f32::consts::TAU;
                        let angle_b = ((segment_index + 1) as f32 / segment_count as f32)
                            * std::f32::consts::TAU;
                        let from = world_center
                            + nalgebra_glm::Vec2::new(
                                angle_a.cos() * *radius,
                                angle_a.sin() * *radius,
                            );
                        let to = world_center
                            + nalgebra_glm::Vec2::new(
                                angle_b.cos() * *radius,
                                angle_b.sin() * *radius,
                            );
                        push_canvas_line(sync, &canvas, from, to, *thickness, *color);
                    }
                    let offset = nalgebra_glm::Vec2::new(*radius, *radius);
                    Some(Rect::from_min_max(center - offset, center + offset))
                }
                CanvasCommand::Arc {
                    center,
                    radius,
                    start_angle,
                    end_angle,
                    thickness,
                    color,
                    ..
                } => {
                    let arc_span = end_angle - start_angle;
                    let segment_count = ((arc_span.abs() / std::f32::consts::TAU * (*radius * 2.0))
                        as usize)
                        .max(8);
                    let world_center = canvas.origin + center;
                    for segment_index in 0..segment_count {
                        let angle_a =
                            *start_angle + arc_span * (segment_index as f32 / segment_count as f32);
                        let angle_b = *start_angle
                            + arc_span * ((segment_index + 1) as f32 / segment_count as f32);
                        let from = world_center
                            + nalgebra_glm::Vec2::new(
                                angle_a.cos() * *radius,
                                angle_a.sin() * *radius,
                            );
                        let to = world_center
                            + nalgebra_glm::Vec2::new(
                                angle_b.cos() * *radius,
                                angle_b.sin() * *radius,
                            );
                        push_canvas_line(sync, &canvas, from, to, *thickness, *color);
                    }
                    let offset = nalgebra_glm::Vec2::new(*radius, *radius);
                    Some(Rect::from_min_max(center - offset, center + offset))
                }
                CanvasCommand::QuadraticBezier {
                    start,
                    control,
                    end,
                    thickness,
                    color,
                    ..
                } => {
                    let arc_length_estimate = {
                        let chord = (end - start).magnitude();
                        let control_net =
                            (control - start).magnitude() + (end - control).magnitude();
                        (chord + control_net) * 0.5
                    };
                    let target_segment_length = (3.0 / (1.0 + thickness * 0.12)).max(0.6);
                    let segment_count = (arc_length_estimate / target_segment_length)
                        .ceil()
                        .max(8.0) as usize;
                    let overlap = (thickness * 0.5).min(2.5);
                    for segment_index in 0..segment_count {
                        let t0 = segment_index as f32 / segment_count as f32;
                        let t1 = (segment_index + 1) as f32 / segment_count as f32;
                        let one_minus_t0 = 1.0 - t0;
                        let one_minus_t1 = 1.0 - t1;
                        let from = canvas.origin
                            + start * (one_minus_t0 * one_minus_t0)
                            + control * (2.0 * one_minus_t0 * t0)
                            + end * (t0 * t0);
                        let to = canvas.origin
                            + start * (one_minus_t1 * one_minus_t1)
                            + control * (2.0 * one_minus_t1 * t1)
                            + end * (t1 * t1);
                        let delta = to - from;
                        let length = delta.magnitude();
                        if length < 0.001 {
                            continue;
                        }
                        let midpoint = (from + to) * 0.5;
                        let angle = delta.y.atan2(delta.x);
                        let extended_length = length + overlap;
                        let rect_position = midpoint
                            - nalgebra_glm::Vec2::new(extended_length * 0.5, *thickness * 0.5);
                        sync.frame.rects.push(DrawRect {
                            position: rect_position,
                            size: nalgebra_glm::Vec2::new(extended_length, *thickness),
                            color: *color,
                            corner_radius: 0.0,
                            border_width: 0.0,
                            border_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
                            rotation: angle,
                            clip_rect: canvas.clip_rect,
                            layer: canvas.layer as u8,
                            z_index: canvas.z_index + 1,
                            shadow: None,
                            effect_kind: 0,
                            effect_params: [0.0; 4],
                            quad_corners: None,
                        });
                    }
                    let min_x = start.x.min(control.x).min(end.x);
                    let min_y = start.y.min(control.y).min(end.y);
                    let max_x = start.x.max(control.x).max(end.x);
                    let max_y = start.y.max(control.y).max(end.y);
                    Some(Rect::from_min_max(
                        nalgebra_glm::Vec2::new(min_x, min_y),
                        nalgebra_glm::Vec2::new(max_x, max_y),
                    ))
                }
                CanvasCommand::CubicBezier {
                    start,
                    control1,
                    control2,
                    end,
                    thickness,
                    color,
                    ..
                } => {
                    let arc_length_estimate = {
                        let chord = (end - start).magnitude();
                        let control_net = (control1 - start).magnitude()
                            + (control2 - control1).magnitude()
                            + (end - control2).magnitude();
                        (chord + control_net) * 0.5
                    };
                    let segment_count = (arc_length_estimate / 4.0).ceil().max(4.0) as usize;
                    for segment_index in 0..segment_count {
                        let t0 = segment_index as f32 / segment_count as f32;
                        let t1 = (segment_index + 1) as f32 / segment_count as f32;
                        let one_minus_t0 = 1.0 - t0;
                        let one_minus_t1 = 1.0 - t1;
                        let from = canvas.origin
                            + start * (one_minus_t0 * one_minus_t0 * one_minus_t0)
                            + control1 * (3.0 * one_minus_t0 * one_minus_t0 * t0)
                            + control2 * (3.0 * one_minus_t0 * t0 * t0)
                            + end * (t0 * t0 * t0);
                        let to = canvas.origin
                            + start * (one_minus_t1 * one_minus_t1 * one_minus_t1)
                            + control1 * (3.0 * one_minus_t1 * one_minus_t1 * t1)
                            + control2 * (3.0 * one_minus_t1 * t1 * t1)
                            + end * (t1 * t1 * t1);
                        push_canvas_line(sync, &canvas, from, to, *thickness, *color);
                    }
                    let min_x = start.x.min(control1.x).min(control2.x).min(end.x);
                    let min_y = start.y.min(control1.y).min(control2.y).min(end.y);
                    let max_x = start.x.max(control1.x).max(control2.x).max(end.x);
                    let max_y = start.y.max(control1.y).max(control2.y).max(end.y);
                    Some(Rect::from_min_max(
                        nalgebra_glm::Vec2::new(min_x, min_y),
                        nalgebra_glm::Vec2::new(max_x, max_y),
                    ))
                }
            };

            if canvas.hit_test_enabled
                && let Some(cid) = command_id
                && let Some(rect) = command_bounds
            {
                bounds.push((cid, rect));
            }
        }

        if canvas.hit_test_enabled
            && let Some(data) = storage.get_mut::<crate::components::UiCanvasData>(canvas.entity)
        {
            data.command_bounds = bounds;
        }
    }
}
