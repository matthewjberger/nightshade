use nightshade::prelude::{Vec3, vec4};
use nightshade_ui_story::StoryStyle;

pub const TITLE: &str = "Candlewake";
pub const WINDOW: (u32, u32) = (1120, 880);

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 0.0, 0.0);
pub const CAMERA_BACK: f32 = 8.0;
pub const CAMERA_YAW: f32 = 0.0;
pub const CAMERA_PITCH: f32 = 0.0;

pub fn style() -> StoryStyle {
    StoryStyle {
        width: 940.0,
        lines: 18,
        columns: 80,
        links: 8,
        line_size: 17.0,
        background: vec4(0.04, 0.04, 0.06, 0.96),
        border: vec4(0.52, 0.44, 0.30, 0.8),
        line_tint: vec4(0.90, 0.88, 0.80, 1.0),
        ..StoryStyle::default()
    }
}
