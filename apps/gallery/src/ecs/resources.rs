use nightshade::prelude::Entity;

#[derive(Clone, Debug, Default)]
pub struct GalleryState {
    pub nav_buttons: Vec<Entity>,
    pub nav_indicators: Vec<Entity>,
    pub nav_labels: Vec<Entity>,
    pub screen_roots: Vec<Entity>,
    pub virtual_list_text_nodes: Vec<Entity>,
    pub nav_y_offsets: Vec<f32>,
    pub active_screen: usize,
    pub progress_value: f32,
    pub color_blend_time: f32,
    pub effects_progress_value: f32,
    pub effects_spinner_angle: f32,
    pub submit_count: u32,
    pub nav_slider_y: f32,
    pub nav_slider_target_y: f32,
}
