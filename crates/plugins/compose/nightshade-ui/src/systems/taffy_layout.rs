use nalgebra_glm::Vec2;
use taffy::prelude::{FromFlex, FromLength, FromPercent, TaffyAuto, minmax, repeat};
use taffy::{
    AlignItems, AvailableSpace, Dimension, Display, FlexDirection, FlexWrap, GridTrackRepetition,
    JustifyContent, LengthPercentage, MaxTrackSizingFunction, MinMax, MinTrackSizingFunction,
    NonRepeatedTrackSizingFunction, Size, Style, TrackSizingFunction,
};

use crate::components::layout_types::{FlowAlignment, FlowDirection, FlowLayout, GridLayout};
use crate::components::{AutoSizeMode, UiLayoutNode};
use crate::data::units::UiValue;

fn mix_dim(absolute: f32, relative_pct: f32, parent: f32) -> Dimension {
    if relative_pct.abs() < f32::EPSILON {
        return Dimension::from_length(absolute);
    }
    if absolute.abs() < f32::EPSILON {
        return Dimension::from_percent(relative_pct * 0.01);
    }
    Dimension::from_length(absolute + relative_pct * 0.01 * parent)
}

fn vec_dimension(
    value: &UiValue<Vec2>,
    parent: Vec2,
    absolute_scale: f32,
    auto: AutoSizeMode,
) -> Size<Dimension> {
    let absolute = value.absolute.unwrap_or(Vec2::new(0.0, 0.0)) * absolute_scale;
    let relative_pct = value.relative.unwrap_or(Vec2::new(0.0, 0.0));

    let width = match auto {
        AutoSizeMode::Width | AutoSizeMode::Both => Dimension::AUTO,
        _ => mix_dim(absolute.x, relative_pct.x, parent.x),
    };
    let height = match auto {
        AutoSizeMode::Height | AutoSizeMode::Both => Dimension::AUTO,
        _ => mix_dim(absolute.y, relative_pct.y, parent.y),
    };
    Size { width, height }
}

pub fn apply_flow_container(style: &mut Style, flow: &FlowLayout, absolute_scale: f32) {
    let padding_units = LengthPercentage::from_length(flow.padding * absolute_scale);
    let gap = LengthPercentage::from_length(flow.spacing * absolute_scale);

    style.display = Display::Flex;
    style.flex_direction = match flow.direction {
        FlowDirection::Vertical => FlexDirection::Column,
        FlowDirection::Horizontal => FlexDirection::Row,
    };
    style.flex_wrap = if flow.wrap {
        FlexWrap::Wrap
    } else {
        FlexWrap::NoWrap
    };
    style.gap = Size {
        width: gap,
        height: gap,
    };
    style.padding = taffy::Rect {
        left: padding_units,
        right: padding_units,
        top: padding_units,
        bottom: padding_units,
    };
    style.justify_content = Some(match flow.alignment {
        FlowAlignment::Start => JustifyContent::FlexStart,
        FlowAlignment::Center => JustifyContent::Center,
        FlowAlignment::End => JustifyContent::FlexEnd,
    });
    style.align_items = Some(match flow.cross_alignment {
        FlowAlignment::Start => AlignItems::FlexStart,
        FlowAlignment::Center => AlignItems::Center,
        FlowAlignment::End => AlignItems::FlexEnd,
    });
}

fn apply_grid_container(style: &mut Style, grid: &GridLayout, absolute_scale: f32) {
    let padding_units = LengthPercentage::from_length(grid.padding * absolute_scale);
    let column_gap = LengthPercentage::from_length(grid.column_spacing * absolute_scale);
    let row_gap = LengthPercentage::from_length(grid.row_spacing * absolute_scale);

    style.display = Display::Grid;
    style.grid_template_columns = match grid.min_column_width {
        Some(min_width) => {
            let track: NonRepeatedTrackSizingFunction = minmax(
                MinTrackSizingFunction::from_length(min_width * absolute_scale),
                MaxTrackSizingFunction::from_flex(1.0f32),
            );
            vec![repeat(GridTrackRepetition::AutoFill, vec![track])]
        }
        None => (0..grid.columns.max(1))
            .map(|_| {
                TrackSizingFunction::from(MinMax {
                    min: MinTrackSizingFunction::from_length(0.0f32),
                    max: MaxTrackSizingFunction::from_flex(1.0f32),
                })
            })
            .collect(),
    };
    style.grid_auto_rows = vec![NonRepeatedTrackSizingFunction::from_length(
        grid.row_height * absolute_scale,
    )];
    style.gap = Size {
        width: column_gap,
        height: row_gap,
    };
    style.padding = taffy::Rect {
        left: padding_units,
        right: padding_units,
        top: padding_units,
        bottom: padding_units,
    };
}

pub fn flow_to_style(
    node: &UiLayoutNode,
    flow: &FlowLayout,
    parent_size: Vec2,
    absolute_scale: f32,
) -> Style {
    let mut style = Style {
        size: Size {
            width: Dimension::from_length(parent_size.x),
            height: Dimension::from_length(parent_size.y),
        },
        ..Default::default()
    };
    apply_flow_container(&mut style, flow, absolute_scale);
    apply_size_constraints(&mut style, node, absolute_scale);
    style
}

pub fn grid_to_style(
    node: &UiLayoutNode,
    grid: &GridLayout,
    parent_size: Vec2,
    absolute_scale: f32,
) -> Style {
    let mut style = Style {
        size: Size {
            width: Dimension::from_length(parent_size.x),
            height: Dimension::from_length(parent_size.y),
        },
        ..Default::default()
    };
    apply_grid_container(&mut style, grid, absolute_scale);
    apply_size_constraints(&mut style, node, absolute_scale);
    style
}

pub fn child_to_style(node: &UiLayoutNode, parent_size: Vec2, absolute_scale: f32) -> Style {
    let is_container = node.flow_layout.is_some() || node.grid_layout.is_some();

    let mut style = if let Some(child_size) = &node.flow_child_size {
        let mut size = vec_dimension(child_size, parent_size, absolute_scale, node.auto_size);
        if is_container {
            if let Dimension::Length(value) = size.width
                && value.abs() < f32::EPSILON
            {
                size.width = Dimension::AUTO;
            }
            if let Dimension::Length(value) = size.height
                && value.abs() < f32::EPSILON
            {
                size.height = Dimension::AUTO;
            }
        }
        Style {
            size,
            ..Default::default()
        }
    } else if matches!(node.auto_size, AutoSizeMode::Both) || is_container {
        Style {
            size: Size {
                width: Dimension::AUTO,
                height: Dimension::AUTO,
            },
            ..Default::default()
        }
    } else {
        Style::default()
    };

    if let Some(grow) = node.flex_grow {
        style.flex_grow = grow;
    }
    style.flex_shrink = node.flex_shrink.unwrap_or(0.0);

    apply_size_constraints(&mut style, node, absolute_scale);

    if let Some(flow) = &node.flow_layout {
        apply_flow_container(&mut style, flow, absolute_scale);
    } else if let Some(grid) = &node.grid_layout {
        apply_grid_container(&mut style, grid, absolute_scale);
        if grid.min_column_width.is_some() && matches!(style.size.width, Dimension::Auto) {
            style.size.width = Dimension::from_percent(1.0f32);
        }
    }

    if !node.visible {
        style.display = Display::None;
    }

    style
}

fn apply_size_constraints(style: &mut Style, node: &UiLayoutNode, absolute_scale: f32) {
    if let Some(min) = node.min_size {
        style.min_size = Size {
            width: Dimension::from_length(min.x * absolute_scale),
            height: Dimension::from_length(min.y * absolute_scale),
        };
    }
    if let Some(max) = node.max_size {
        style.max_size = Size {
            width: Dimension::from_length(max.x * absolute_scale),
            height: Dimension::from_length(max.y * absolute_scale),
        };
    }
}

pub fn available_size_for(parent: Vec2) -> Size<AvailableSpace> {
    Size {
        width: AvailableSpace::Definite(parent.x),
        height: AvailableSpace::Definite(parent.y),
    }
}
