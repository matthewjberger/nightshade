use nalgebra_glm::Vec2;
use nightshade_paint::data::{Anchor, Rect};

use crate::components::layout_types::{
    BoundaryLayout, ScalingMode, SolidLayout, UiLayoutType, WindowLayout,
};
use crate::data::units::UiEvalContext;

pub fn compute_layout_rect(layout: &UiLayoutType, context: &UiEvalContext, parent: &Rect) -> Rect {
    match layout {
        UiLayoutType::Boundary(boundary) => compute_boundary(boundary, context, parent),
        UiLayoutType::Window(window) => compute_window(window, context, parent),
        UiLayoutType::Solid(solid) => compute_solid(solid, context, parent),
    }
}

fn anchor_offset(anchor: Anchor, size: Vec2) -> Vec2 {
    match anchor {
        Anchor::TopLeft => Vec2::new(0.0, 0.0),
        Anchor::TopCenter => Vec2::new(size.x * 0.5, 0.0),
        Anchor::TopRight => Vec2::new(size.x, 0.0),
        Anchor::CenterLeft => Vec2::new(0.0, size.y * 0.5),
        Anchor::Center => Vec2::new(size.x * 0.5, size.y * 0.5),
        Anchor::CenterRight => Vec2::new(size.x, size.y * 0.5),
        Anchor::BottomLeft => Vec2::new(0.0, size.y),
        Anchor::BottomCenter => Vec2::new(size.x * 0.5, size.y),
        Anchor::BottomRight => Vec2::new(size.x, size.y),
    }
}

fn compute_boundary(layout: &BoundaryLayout, context: &UiEvalContext, parent: &Rect) -> Rect {
    let point_1 = layout.position_1.evaluate(context);
    let point_2 = layout.position_2.evaluate(context);

    let min = Vec2::new(
        parent.min.x + point_1.x.min(point_2.x),
        parent.min.y + point_1.y.min(point_2.y),
    );
    let max = Vec2::new(
        parent.min.x + point_1.x.max(point_2.x),
        parent.min.y + point_1.y.max(point_2.y),
    );

    Rect::from_min_max(min, max)
}

fn compute_window(layout: &WindowLayout, context: &UiEvalContext, parent: &Rect) -> Rect {
    let position = layout.position.evaluate(context);
    let size = layout.size.evaluate(context);
    let offset = anchor_offset(layout.anchor, size);

    let top_left = Vec2::new(
        parent.min.x + position.x - offset.x,
        parent.min.y + position.y - offset.y,
    );

    Rect::from_min_max(top_left, top_left + size)
}

fn compute_solid(layout: &SolidLayout, context: &UiEvalContext, parent: &Rect) -> Rect {
    if parent.width() < f32::EPSILON || parent.height() < f32::EPSILON {
        return Rect::from_min_max(parent.min, parent.min);
    }

    let aspect_ratio_raw = layout.size.evaluate(context);
    let aspect = if aspect_ratio_raw.y.abs() > f32::EPSILON {
        aspect_ratio_raw.x / aspect_ratio_raw.y
    } else {
        1.0
    };

    let aspect = if aspect.abs() < f32::EPSILON {
        1.0
    } else {
        aspect
    };

    let parent_width = parent.width();
    let parent_height = parent.height();

    let (width, height) = match layout.scaling {
        ScalingMode::Fit => {
            let scale = (parent_width / aspect).min(parent_height);
            (scale * aspect, scale)
        }
        ScalingMode::Fill => {
            let scale = (parent_width / aspect).max(parent_height);
            (scale * aspect, scale)
        }
        ScalingMode::HorizontalFill => (parent_width, parent_width / aspect),
        ScalingMode::VerticalFill => (parent_height * aspect, parent_height),
    };

    if width.is_nan() || height.is_nan() {
        return Rect::from_min_max(parent.min, parent.min);
    }

    let remaining_x = parent_width - width;
    let remaining_y = parent_height - height;

    let offset_x = remaining_x * (layout.alignment.x + 1.0) * 0.5;
    let offset_y = remaining_y * (layout.alignment.y + 1.0) * 0.5;

    let top_left = Vec2::new(parent.min.x + offset_x, parent.min.y + offset_y);

    Rect::from_min_max(top_left, Vec2::new(top_left.x + width, top_left.y + height))
}
