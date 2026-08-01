use super::*;
use crate::components::layer::UiLayer;
use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};
use nightshade_paint::data::{DrawImage, DrawRect};
use nightshade_paint::data::{DrawText, Rect};
use nightshade_text::contract::components::TextProperties;

#[derive(Default)]
pub struct RetainedUiInputMirror {
    pub frame_chars: Vec<char>,
    pub frame_keys: Vec<(nightshade_platform::KeyCode, bool)>,
    pub ctrl_held: bool,
    pub shift_held: bool,
    pub scroll_delta: Vec2,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct RetainedUiTiming {
    pub current_time: f64,
    pub delta_time: f32,
}

#[derive(Default)]
pub struct RetainedUiDirty {
    pub layout_dirty: bool,
    pub render_dirty: bool,
    pub global_version: u64,
    pub last_compute_rect: Option<nightshade_paint::data::Rect>,
    pub last_compute_dpi: f32,
    pub last_compute_text_generation: u64,
    pub cached_node_rect_count: usize,
    pub cached_node_image_count: usize,
    pub cached_node_text_count: usize,
    pub rebuilds_owed: u8,
    pub node_versions: std::collections::HashMap<nightshade::ecs::Entity, u64>,
}

#[derive(Default)]
pub struct RetainedUiFrame {
    pub handed_off: bool,
    pub rects: Vec<DrawRect>,
    pub rect_entities: Vec<Option<nightshade::ecs::Entity>>,
    pub ui_images: Vec<DrawImage>,
    pub text_meshes: Vec<DrawText>,
    pub overlay_rects: Vec<DrawRect>,
    pub overlay_text: Vec<OverlayText>,
    pub events: Vec<UiEvent>,
    pub bubbled_events: Vec<BubbledUiEvent>,
}

#[derive(Default)]
pub struct RetainedUiRuntime {
    pub visible: bool,
    pub z_sorted_nodes: Vec<(nightshade::ecs::Entity, f32)>,
    pub blend_scratch: Vec<nightshade::ecs::Entity>,
    pub pending_image_uploads: Vec<UiImageLayerUpload>,
    pub taffy_region_rects:
        std::collections::HashMap<nightshade::ecs::Entity, nightshade_paint::data::Rect>,
    pub background_color: Option<Vec4>,
    pub next_focus_order: i32,
    pub next_tab_index: i32,
    pub custom_state_count: usize,
    pub clipboard_text: String,
    pub picking_grid: Option<PickingGrid>,
}

impl RetainedUiRuntime {}

impl RetainedUiFrame {
    pub fn draw_overlay_rect(&mut self, rect: DrawRect) {
        self.overlay_rects.push(rect);
    }

    pub fn draw_overlay_text(
        &mut self,
        text: &str,
        position: Vec2,
        properties: TextProperties,
        clip_rect: Option<Rect>,
        layer: UiLayer,
        z_index: i32,
    ) {
        self.overlay_text.push(OverlayText {
            text: text.to_string(),
            position,
            properties,
            clip_rect,
            layer,
            z_index,
        });
    }
}
