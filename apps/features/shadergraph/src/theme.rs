use nalgebra_glm::{Vec2, Vec4};

pub const TITLE: &str = "Shader graph";

pub const CLEAR: [f32; 4] = [0.02, 0.02, 0.03, 1.0];

pub const PALETTE_TITLE: &str = "Nodes";
pub const PALETTE_WIDTH: f32 = 262.0;
pub const GRAPH_TITLE: &str = "Graph";
pub const GRAPH_HEIGHT: f32 = 460.0;
pub const PANEL_PADDING: f32 = 10.0;
pub const PANEL_SPACING: f32 = 6.0;
pub const SHELF_HEIGHT: f32 = 320.0;
pub const KIND_WIDTH: f32 = 108.0;
pub const BUTTON_SPACING: f32 = 4.0;
pub const OPEN_FAMILIES: usize = 2;

pub const PRESET_HEADING: &str = "Preset";
pub const PICTURE_HEADING: &str = "Pictures";
pub const PLACE_HEADING: &str = "Place a node";

pub const PLUCK_LABEL: &str = "Open a picture";
pub const WRITE_LABEL: &str = "Write the WGSL";
pub const FACE_NAME: &str = "awesomeface.png";
pub const STONE_NAME: &str = "cobblestone.png";
pub const FACE_SLOT: u32 = 0;
pub const STONE_SLOT: u32 = 1;

pub const DRAG_SHARE: f32 = 0.004;
pub const STEP_REACH: f32 = 26.0;
pub const TAP_REACH: f32 = 4.0;

pub const PRESET_CORNER: Vec2 = Vec2::new(60.0, 40.0);
pub const FRESH_PLACE: Vec2 = Vec2::new(120.0, 60.0);
pub const FRESH_STEP: Vec2 = Vec2::new(26.0, 32.0);

pub const GRID_TINT: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.07);
pub const BODY_TINT: Vec4 = Vec4::new(0.10, 0.11, 0.14, 0.96);
pub const EDGE_TINT: Vec4 = Vec4::new(0.02, 0.02, 0.03, 1.0);
pub const WIRE_TINT: Vec4 = Vec4::new(0.82, 0.88, 0.96, 0.94);
