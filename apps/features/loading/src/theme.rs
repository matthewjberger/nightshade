use nightshade::prelude::{Vec3, Vec4};

pub const TITLE: &str = "Loading";
pub const HEADING: &str = "reading";
pub const RELOAD: &str = "read them again";

pub const SHEET: &[u8] = include_bytes!("../assets/sheet.png");
pub const FOOTSTEP: &[u8] = include_bytes!("../assets/footstep.ogg");
pub const FOOTSTEP_NAME: &str = "footstep";

pub const CAMERA_TARGET: Vec3 = Vec3::new(1.0, 0.8, 0.0);
pub const CAMERA_DISTANCE: f32 = 14.0;
pub const CAMERA_YAW: f32 = 0.5;
pub const CAMERA_PITCH: f32 = -0.2;

pub const CARD_BACKGROUND: Vec4 = Vec4::new(0.04, 0.04, 0.07, 0.92);
pub const CARD_BORDER: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.14);
pub const TRACK: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.10);
pub const FILL: Vec4 = Vec4::new(0.45, 0.92, 0.50, 1.0);
pub const TEXT: Vec4 = Vec4::new(0.92, 0.93, 0.96, 1.0);
pub const TEXT_DIM: Vec4 = Vec4::new(0.70, 0.72, 0.80, 1.0);

pub const CARD_WIDTH: f32 = 560.0;
pub const CARD_HEIGHT: f32 = 220.0;
pub const TITLE_SIZE: f32 = 40.0;
pub const COUNT_SIZE: f32 = 20.0;
pub const NAME_SIZE: f32 = 18.0;
pub const BAR_WIDTH: f32 = CARD_WIDTH - 96.0;
pub const BAR_HEIGHT: f32 = 18.0;
pub const BAR_INSET: f32 = 3.0;

pub const MARGIN: f32 = 16.0;
pub const BUTTON_WIDTH: f32 = 200.0;
pub const BUTTON_HEIGHT: f32 = 34.0;
pub const PICTURE_SIDE: f32 = 160.0;
