use nalgebra_glm::{Vec2, Vec4};

pub const TITLE: &str = "Node graph";

pub const CLEAR: [f32; 4] = [0.07, 0.08, 0.10, 1.0];

pub const SOURCE_PLACE: Vec2 = Vec2::new(120.0, 140.0);
pub const SHAPER_PLACE: Vec2 = Vec2::new(400.0, 100.0);
pub const MIXER_PLACE: Vec2 = Vec2::new(400.0, 300.0);
pub const SINK_PLACE: Vec2 = Vec2::new(680.0, 200.0);

pub const SOURCE_TINT: Vec4 = Vec4::new(0.28, 0.46, 0.66, 1.0);
pub const SHAPER_TINT: Vec4 = Vec4::new(0.52, 0.34, 0.62, 1.0);
pub const MIXER_TINT: Vec4 = Vec4::new(0.30, 0.54, 0.42, 1.0);
pub const SINK_TINT: Vec4 = Vec4::new(0.66, 0.38, 0.30, 1.0);
pub const FRESH_TINT: Vec4 = Vec4::new(0.46, 0.44, 0.50, 1.0);

pub const FRESH_INLETS: &[&str] = &["a", "b"];
pub const FRESH_OUTLETS: &[&str] = &["out"];
pub const FRESH_TITLE: &str = "Node";
pub const FRESH_NOTE: &str = "a fresh node";

pub const SHAPE_LOW: f32 = 0.0;
pub const SHAPE_HIGH: f32 = 4.0;
pub const SHAPE_START: f32 = 1.0;
pub const SHAPE_SHARE: f32 = 0.01;

pub fn note_of(amount: f32) -> String {
    format!("amount  {amount:.2}")
}
