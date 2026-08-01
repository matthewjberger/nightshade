use nightshade::prelude::Vec2;
use nightshade_platform::prelude::Window;

pub fn to_units(pixels: f32, window: &Window) -> f32 {
    pixels / scale(window)
}

pub fn point_to_units(pixels: Vec2, window: &Window) -> Vec2 {
    pixels / scale(window)
}

pub fn to_pixels(units: f32, window: &Window) -> f32 {
    units * scale(window)
}

fn scale(window: &Window) -> f32 {
    window.cached_scale_factor.max(0.0001)
}
