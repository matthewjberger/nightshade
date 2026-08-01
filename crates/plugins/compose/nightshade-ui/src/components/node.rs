use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};

use crate::components::layer::UiLayer;
use crate::components::layout_types::{FlowLayout, GridLayout, UiLayoutType};
use crate::data::state::STATE_COUNT;
use crate::data::units::UiValue;
use nightshade_paint::data::Rect;
use nightshade_text::contract::data::text_data::{TextAlignment, VerticalAlignment};

use super::{AutoSizeMode, TextOverflow, UiDepthMode, UiResponsive};

#[derive(Clone, Debug)]
pub struct UiLayoutNode {
    pub base_layout: Option<UiLayoutType>,
    pub flow_layout: Option<FlowLayout>,
    pub responsive_flow: Option<crate::components::layout_types::ResponsiveFlowOverride>,
    pub responsive: Option<UiResponsive>,
    pub grid_layout: Option<GridLayout>,
    pub depth: UiDepthMode,
    pub font_size: Option<f32>,
    pub clip_content: bool,
    pub pointer_events: bool,
    pub visible: bool,
    pub flow_child_size: Option<UiValue<Vec2>>,
    pub flex_grow: Option<f32>,
    pub flex_shrink: Option<f32>,
    pub min_size: Option<Vec2>,
    pub max_size: Option<Vec2>,
    pub z_index: Option<i32>,
    pub auto_size: AutoSizeMode,
    pub auto_size_padding: Vec2,
    pub rotation: f32,
    pub computed_rotation: f32,
    pub computed_pivot: Vec2,
    pub computed_rect: Rect,
    pub computed_depth: f32,
    pub computed_font_scale: f32,
    pub computed_clip_rect: Option<Rect>,
    pub layer: Option<UiLayer>,
    pub computed_layer: Option<UiLayer>,
    pub animation: Option<UiNodeAnimation>,
    pub scroll_offset: Vec2,
}

impl Default for UiLayoutNode {
    fn default() -> Self {
        Self {
            base_layout: None,
            flow_layout: None,
            responsive_flow: None,
            responsive: None,
            grid_layout: None,
            depth: UiDepthMode::default(),
            font_size: None,
            clip_content: false,
            pointer_events: false,
            visible: true,
            flow_child_size: None,
            flex_grow: None,
            flex_shrink: None,
            min_size: None,
            max_size: None,
            z_index: None,
            auto_size: AutoSizeMode::None,
            auto_size_padding: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            computed_rotation: 0.0,
            computed_pivot: Vec2::new(0.0, 0.0),
            computed_rect: Rect::default(),
            computed_depth: 0.0,
            computed_font_scale: 1.0,
            computed_clip_rect: None,
            layer: None,
            computed_layer: None,
            animation: None,
            scroll_offset: Vec2::new(0.0, 0.0),
        }
    }
}

impl UiLayoutNode {}

#[derive(Clone, Copy, Debug)]
pub struct UiNodeColor {
    pub colors: [Option<Vec4>; STATE_COUNT],
    pub computed_color: Vec4,
}

impl Default for UiNodeColor {
    fn default() -> Self {
        Self {
            colors: [None; STATE_COUNT],
            computed_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum UiNodeContent {
    #[default]
    None,
    Rect {
        corner_radius: f32,
        border_width: f32,
        border_color: Vec4,
    },
    Text {
        text_slot: usize,
        font_index: usize,
        font_size_override: Option<f32>,
        outline_color: Vec4,
        outline_width: f32,
        alignment: TextAlignment,
        vertical_alignment: VerticalAlignment,
        overflow: TextOverflow,
        monospace_width: Option<f32>,
        font_kind: nightshade_text::contract::data::text_data::FontKind,
    },
    Image {
        texture_index: u32,
        uv_min: Vec2,
        uv_max: Vec2,
    },
}

#[derive(Clone, Copy, Debug, Default)]
pub struct UiNodeShadow {
    pub color: Vec4,
    pub offset: Vec2,
    pub blur: f32,
    pub spread: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct UiNodeEffect {
    pub kind: u32,
    pub params: [f32; 4],
}

#[derive(Clone, Copy, Debug, Default)]
pub struct UiNodeShadowStates {
    pub shadows: [Option<UiNodeShadow>; crate::data::state::STATE_COUNT],
}

#[derive(Clone, Copy, Debug, Default)]
pub struct UiNodeTransformStates {
    pub offsets: [Option<Vec2>; crate::data::state::STATE_COUNT],
    pub scales: [Option<f32>; crate::data::state::STATE_COUNT],
    pub radii: [Option<f32>; crate::data::state::STATE_COUNT],
}

#[derive(Clone, Copy, Debug)]
pub struct UiNodeBlendedTransform {
    pub offset: Vec2,
    pub scale: f32,
    pub radius_override: Option<f32>,
}

impl Default for UiNodeBlendedTransform {
    fn default() -> Self {
        Self {
            offset: Vec2::zeros(),
            scale: 1.0,
            radius_override: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiAnimationType {
    Fade,
    SlideLeft,
    SlideRight,
    SlideUp,
    SlideDown,
    Scale,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiAnimationPhase {
    Idle,
    IntroPlaying,
    OutroPlaying,
    OutroComplete,
}

#[derive(Clone, Copy, Debug)]
pub struct UiNodeAnimation {
    pub intro: Option<UiAnimationType>,
    pub outro: Option<UiAnimationType>,
    pub duration: f32,
    pub progress: f32,
    pub phase: UiAnimationPhase,
}
