use nalgebra_glm;

use nalgebra_glm::Vec2;

use crate::data::units::UiValue;

use nightshade_paint::data::Anchor;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ScalingMode {
    #[default]
    Fit,
    Fill,
    HorizontalFill,
    VerticalFill,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FlowDirection {
    #[default]
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FlowAlignment {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug)]
pub struct FlowLayout {
    pub direction: FlowDirection,
    pub padding: f32,
    pub spacing: f32,
    pub alignment: FlowAlignment,
    pub cross_alignment: FlowAlignment,
    pub wrap: bool,
}

impl FlowLayout {
    pub fn stack() -> Self {
        Self {
            direction: FlowDirection::Vertical,
            padding: 0.0,
            spacing: 0.0,
            alignment: FlowAlignment::Start,
            cross_alignment: FlowAlignment::Start,
            wrap: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum UiLayoutType {
    Boundary(BoundaryLayout),
    Window(WindowLayout),
    Solid(SolidLayout),
}

#[derive(Clone, Copy, Debug)]
pub struct BoundaryLayout {
    pub position_1: UiValue<Vec2>,
    pub position_2: UiValue<Vec2>,
}

#[derive(Clone, Copy, Debug)]
pub struct WindowLayout {
    pub position: UiValue<Vec2>,
    pub size: UiValue<Vec2>,
    pub anchor: Anchor,
}

#[derive(Clone, Copy, Debug)]
pub struct SolidLayout {
    pub size: UiValue<Vec2>,
    pub scaling: ScalingMode,
    pub alignment: Vec2,
}

#[derive(Clone, Copy, Debug)]
pub struct ResponsiveFlowOverride {
    pub max_width: f32,
    pub direction: FlowDirection,
}

#[derive(Clone, Copy, Debug)]
pub struct GridLayout {
    pub columns: usize,
    pub row_height: f32,
    pub padding: f32,
    pub column_spacing: f32,
    pub row_spacing: f32,
    pub min_column_width: Option<f32>,
}
