use nightshade::prelude::*;

#[derive(Default, Clone, Copy)]
pub struct UiHandles {
    pub panel: Entity,
    pub heading: Entity,
    pub width_slider: Entity,
    pub smoothing_slider: Entity,
    pub red_slider: Entity,
    pub green_slider: Entity,
    pub blue_slider: Entity,
    pub scale_slider: Entity,
    pub billboard_toggle: Entity,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Style {
    pub outline_width: f32,
    pub smoothing: f32,
    pub outline_color: Vec4,
    pub scale: f32,
    pub billboard: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            outline_width: 2.0,
            smoothing: 0.0,
            outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
            scale: 1.0,
            billboard: true,
        }
    }
}

#[derive(Default)]
pub struct TextStyleResources {
    pub handles: UiHandles,
    pub style: Style,
    pub applied: Option<Style>,
}
