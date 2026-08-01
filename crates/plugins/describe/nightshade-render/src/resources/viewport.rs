#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportExtent {
    FullWindow,
    #[default]
    Unobstructed,
    WindowContent,
}

#[derive(Default)]
pub struct Viewport {
    pub active_viewport_rect: Option<crate::data::config::ViewportRect>,
    pub extent: ViewportExtent,
    pub camera_tile_rects:
        std::collections::HashMap<nightshade::ecs::Entity, crate::data::config::ViewportRect>,
    pub force_render_cameras: std::collections::HashSet<nightshade::ecs::Entity>,
    pub camera_tile_render_iteration: u32,
    pub required_cameras: Vec<nightshade::ecs::Entity>,
    pub required_camera_sizes: Vec<(u32, u32)>,
    pub viewport_texture_sizes: Vec<(u32, u32)>,
    pub create_renderer: bool,
}
