use nightshade::prelude::*;
use nightshade_platform::contract::window::{Window, WindowRect};
use nightshade_render::data::config::ViewportRect;
use nightshade_render::resources::viewport::{Viewport, ViewportExtent};

fn covers_whole_window(rect: &WindowRect, window: Option<(u32, u32)>) -> bool {
    let Some((width, height)) = window else {
        return false;
    };
    rect.x <= 0.5
        && rect.y <= 0.5
        && rect.width + 0.5 >= width as f32
        && rect.height + 0.5 >= height as f32
}

pub fn viewport_extent_system(window: Res<Window>, mut viewport: ResMut<Viewport>) {
    let rect = match viewport.extent {
        ViewportExtent::FullWindow => None,
        ViewportExtent::Unobstructed => window.unobstructed_rect,
        ViewportExtent::WindowContent => window.content_rect,
    };
    viewport.active_viewport_rect = rect
        .filter(|rect| !covers_whole_window(rect, window.cached_viewport_size))
        .map(|rect| ViewportRect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        });
}
