pub use crate::contract::monitor::*;

pub const DEFAULT_WINDOW_ICON: &[u8] = include_bytes!("../images/icon.png");

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct WindowRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub struct Window {
    pub handle: Option<std::sync::Arc<winit::window::Window>>,
    pub content_rect: Option<WindowRect>,
    pub unobstructed_rect: Option<WindowRect>,
    pub title: String,
    pub icon_bytes: Option<&'static [u8]>,
    pub should_exit: bool,
    pub close_requested: bool,
    pub cached_viewport_size: Option<(u32, u32)>,
    pub cached_scale_factor: f32,
    pub pointer_viewport_scale: f32,
    pub is_focused: bool,
    pub cursor_locked: bool,
    pub start_hidden: bool,
    pub min_window_size: Option<(u32, u32)>,
    pub use_fullscreen: bool,
    pub show_cursor: bool,
    pub initial_size: Option<(u32, u32)>,
    pub applied_min_window_size: Option<(u32, u32)>,
    pub applied_title: Option<String>,
    pub applied_icon_bytes: Option<&'static [u8]>,
    pub requested_cursor: Option<winit::window::CursorIcon>,
    pub applied_cursor: Option<winit::window::CursorIcon>,
    pub applied_cursor_locked: Option<bool>,
    pub applied_cursor_visible: Option<bool>,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            handle: None,
            content_rect: None,
            unobstructed_rect: None,
            title: "Nightshade".to_string(),
            icon_bytes: Some(DEFAULT_WINDOW_ICON),
            should_exit: false,
            close_requested: false,
            cached_viewport_size: None,
            cached_scale_factor: 1.0,
            pointer_viewport_scale: 1.0,
            is_focused: true,
            cursor_locked: false,
            start_hidden: false,
            min_window_size: None,
            use_fullscreen: false,
            show_cursor: true,
            initial_size: None,
            applied_min_window_size: None,
            applied_title: None,
            applied_icon_bytes: None,
            requested_cursor: None,
            applied_cursor: None,
            applied_cursor_locked: None,
            applied_cursor_visible: None,
        }
    }
}
