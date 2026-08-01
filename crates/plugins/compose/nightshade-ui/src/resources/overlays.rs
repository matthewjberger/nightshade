use super::*;
use crate::components::layer::UiLayer;
use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};
use nightshade_paint::data::Rect;
use nightshade_text::contract::components::TextProperties;

#[derive(Default, Debug)]
pub struct RetainedUiOverlays {
    pub tooltip_state: Option<TooltipActiveState>,
    pub popup_entities: Vec<nightshade::ecs::Entity>,
    pub toast_container: Option<nightshade::ecs::Entity>,
    pub toast_entries: Vec<ToastEntry>,
    pub toast_corner: ToastCorner,
    pub dock_indicator_active: bool,
    pub dock_indicator_panel: Option<nightshade::ecs::Entity>,
    pub active_context_menu: Option<nightshade::ecs::Entity>,
    pub active_modal: Option<nightshade::ecs::Entity>,
    pub focus_ring_visible: bool,
}

#[derive(Clone, Debug)]
pub struct TopProgressBar {
    pub value: Option<f32>,
    pub indeterminate: bool,
    pub phase: f32,
    pub height: f32,
    pub color: Option<Vec4>,
}

impl Default for TopProgressBar {
    fn default() -> Self {
        Self {
            value: None,
            indeterminate: false,
            phase: 0.0,
            height: 2.0,
            color: None,
        }
    }
}

impl TopProgressBar {
    pub fn set(&mut self, value: f32) {
        self.value = Some(value.clamp(0.0, 1.0));
        self.indeterminate = false;
    }

    pub fn set_indeterminate(&mut self) {
        self.value = None;
        self.indeterminate = true;
    }

    pub fn clear(&mut self) {
        self.value = None;
        self.indeterminate = false;
    }

    pub fn is_active(&self) -> bool {
        self.value.is_some() || self.indeterminate
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ToastSeverity {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ToastCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    #[default]
    BottomRight,
}

#[derive(Clone, Debug)]
pub struct ToastEntry {
    pub entity: nightshade::ecs::Entity,
    pub spawn_time: f64,
    pub duration: f32,
    pub dismissing: bool,
}

pub struct DrawRectParams {
    pub position: Vec2,
    pub size: Vec2,
    pub color: Vec4,
    pub corner_radius: f32,
    pub border_width: f32,
    pub border_color: Vec4,
    pub layer: UiLayer,
    pub z_index: i32,
}

pub struct OverlayText {
    pub text: String,
    pub position: Vec2,
    pub properties: TextProperties,
    pub clip_rect: Option<Rect>,
    pub layer: UiLayer,
    pub z_index: i32,
}
