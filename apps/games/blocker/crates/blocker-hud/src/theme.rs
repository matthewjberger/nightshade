use nalgebra_glm::Vec4;

pub const TITLE: &str = "Blocker";

pub const TOOL_SHEET_WIDTH: u32 = 512;

pub const TOOL_SHEET_HEIGHT: u32 = 128;

pub const FRAME_WIDTH: f32 = 232.0;

pub const FRAME_HEIGHT: f32 = 128.0;

pub const FRAME_MARGIN: f32 = 16.0;

pub const FRAME_GAP: f32 = 12.0;

pub const TOAST_WIDTH: f32 = 380.0;

pub const TOAST_HEIGHT: f32 = 44.0;

pub const TOAST_TOP: f32 = 90.0;

pub const BAR_HEIGHT: f32 = 22.0;

pub const XP_COLOR: Vec4 = Vec4::new(0.62, 0.28, 0.86, 1.0);

pub const BAR_BORDER: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.85);

pub const BAR_BACKGROUND: Vec4 = Vec4::new(0.05, 0.05, 0.07, 0.92);

pub const HEALTH_COLOR: Vec4 = Vec4::new(0.16, 0.72, 0.24, 1.0);

pub const FOOD_COLOR: Vec4 = Vec4::new(0.92, 0.66, 0.24, 1.0);

pub const PANEL_BACKGROUND: Vec4 = Vec4::new(0.04, 0.04, 0.06, 0.72);

pub const UNIT_NAME_COLOR: Vec4 = Vec4::new(1.0, 0.94, 0.72, 1.0);

pub const TEXT_BRIGHT: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);

pub const SLOT_SIZE: f32 = 52.0;

pub const SLOT_GAP: f32 = 4.0;

pub const ICON_SIZE: f32 = 34.0;

pub const HOTBAR_BOTTOM_MARGIN: f32 = 18.0;

pub const SLOT_COLOR: Vec4 = Vec4::new(0.08, 0.08, 0.1, 0.72);

pub const SLOT_BORDER: Vec4 = Vec4::new(0.45, 0.45, 0.5, 0.9);

pub const SLOT_SELECTED_COLOR: Vec4 = Vec4::new(0.25, 0.25, 0.28, 0.9);

pub const SLOT_SELECTED_BORDER: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);

pub const TEXT_COLOR: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);

pub const CROSSHAIR_COLOR: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.85);

pub const TOAST_SECONDS: f32 = 2.5;

pub const PANEL_COLOR: Vec4 = Vec4::new(0.07, 0.06, 0.09, 0.96);

pub const BACKDROP_COLOR: Vec4 = Vec4::new(0.02, 0.02, 0.05, 0.72);

pub const PANEL_BORDER: Vec4 = Vec4::new(0.85, 0.7, 0.45, 0.9);

pub const FRAME_IMAGE: &str = "panel-border-000";

pub const FRAME_CORNER: f32 = 16.0;

pub const FRAME_TINT: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);

pub const CHAT_WIDTH: f32 = 464.0;

pub const CHAT_TAB_HEIGHT: f32 = 34.0;

pub const CHAT_TAB_GAP: f32 = 4.0;

pub const CHAT_GAP: f32 = 6.0;

pub const CHAT_ENTRY_HEIGHT: f32 = 28.0;

pub const CHAT_UNREAD_COLOR: Vec4 = Vec4::new(1.0, 0.55, 0.2, 1.0);

pub const CHANNEL_COLORS: [Vec4; blocker_chat::data::chat::CHANNELS] = [
    Vec4::new(1.0, 0.78, 0.78, 1.0),
    Vec4::new(1.0, 0.72, 0.60, 1.0),
    Vec4::new(0.82, 0.95, 0.68, 1.0),
    Vec4::new(1.0, 0.55, 1.0, 1.0),
    Vec4::new(1.0, 1.0, 1.0, 1.0),
    Vec4::new(1.0, 1.0, 0.45, 1.0),
];

pub const CHAT_LINE: f32 = 20.0;

pub const CHAT_CHARACTERS: usize = 58;

pub const CHAT_MARGIN: f32 = 16.0;

pub const QUEST_ROWS: usize = 5;

pub const TITLE_COLOR: Vec4 = Vec4::new(0.16, 0.13, 0.09, 1.0);

pub const FOOTER_COLOR: Vec4 = Vec4::new(0.11, 0.1, 0.13, 1.0);

pub const ROW_COLOR: Vec4 = Vec4::new(0.15, 0.14, 0.18, 0.75);

pub const ROW_ALT_COLOR: Vec4 = Vec4::new(0.11, 0.1, 0.14, 0.75);

pub const CELL_COLOR: Vec4 = Vec4::new(0.13, 0.12, 0.16, 0.95);

pub const CELL_BORDER: Vec4 = Vec4::new(0.42, 0.4, 0.46, 0.95);

pub const ACCENT_COLOR: Vec4 = Vec4::new(1.0, 0.82, 0.45, 1.0);

pub const DIM_TEXT_COLOR: Vec4 = Vec4::new(0.7, 0.66, 0.6, 1.0);

pub const HEADING_SIZE: f32 = 24.0;

pub const BODY_SIZE: f32 = 18.0;

pub const SMALL_SIZE: f32 = 15.0;

pub const MENU_WIDTH: f32 = 660.0;

pub const MAP_VILLAGE_COLOR: Vec4 = Vec4::new(0.55, 0.85, 1.0, 1.0);

pub const TITLE_HEIGHT: f32 = 48.0;

pub const FOOTER_HEIGHT: f32 = 58.0;

pub const BODY_PADDING: f32 = 16.0;

pub const ROW_HEIGHT: f32 = 38.0;

pub const ROW_GAP: f32 = 4.0;

pub const ROW_PADDING: f32 = 8.0;

pub const ROW_SPACING: f32 = 8.0;

pub const ROW_WIDTH: f32 = MENU_WIDTH - 60.0;

pub const ACTION_WIDTH: f32 = 88.0;

pub const ACTION_HEIGHT: f32 = 30.0;

pub const CELL_SIZE: f32 = 58.0;

pub const CELL_ICON: f32 = 36.0;

pub const CELL_GAP: f32 = 8.0;

pub const COUNT_SIZE: f32 = 14.0;

pub const RATE_CARD_WIDTH: f32 = 236.0;

pub const RATE_GOOD_COLOR: Vec4 = Vec4::new(0.4, 1.0, 0.5, 1.0);

pub const RATE_FAIR_COLOR: Vec4 = Vec4::new(1.0, 0.85, 0.3, 1.0);

pub const RATE_POOR_COLOR: Vec4 = Vec4::new(1.0, 0.36, 0.3, 1.0);

pub const RATE_GOOD: f32 = 110.0;

pub const RATE_FAIR: f32 = 55.0;

pub const STATUS_WIDTH: f32 = 460.0;

pub const STATUS_HEIGHT: f32 = 34.0;

pub const TARGET_HEIGHT: f32 = 62.0;

pub const THREAT_RANGE: f32 = 32.0;

pub const FOLK_HOVER_REACH: f32 = 5.5;

pub const FOLK_HOVER_RADIUS: f32 = 0.65;

pub const FOLK_HEAD_CENTER: f32 = 0.9;

pub const FOLK_OUTLINE_COLOR: [f32; 4] = [1.0, 0.45, 0.0, 1.0];

pub const COMPASS_TOP: f32 = 14.0;

pub const COMPASS_WIDTH: f32 = 460.0;

pub const COMPASS_HEIGHT: f32 = 46.0;

pub const COMPASS_FIELD: f32 = 150.0;

pub const COMPASS_RANGE: f32 = 900.0;

pub const COMPASS_MARKS: usize = 10;

pub const COMPASS_DOT: f32 = 18.0;

pub const COMPASS_TICK_ROW: f32 = -12.0;

pub const COMPASS_MARK_ROW: f32 = 11.0;

pub const COMPASS_READOUT_WIDTH: f32 = 320.0;

pub const COMPASS_READOUT_HEIGHT: f32 = 24.0;

pub const COMPASS_TICK_WIDTH: f32 = 26.0;

pub const COMPASS_COLOR: Vec4 = Vec4::new(0.05, 0.05, 0.07, 0.55);

pub const COMPASS_BORDER: Vec4 = Vec4::new(0.85, 0.7, 0.45, 0.5);

pub const COMPASS_TEXT: Vec4 = Vec4::new(0.95, 0.9, 0.78, 0.95);

pub const COMPASS_NOTCH: Vec4 = Vec4::new(0.98, 0.86, 0.52, 0.9);

pub const COMPASS_PLACE: Vec4 = Vec4::new(0.72, 0.78, 0.9, 0.85);

pub const COMPASS_QUEST: Vec4 = Vec4::new(0.98, 0.82, 0.25, 1.0);

pub const COMPASS_CITY: Vec4 = Vec4::new(0.72, 0.86, 0.98, 1.0);

pub const COMPASS_BARROW: Vec4 = Vec4::new(0.82, 0.62, 0.92, 1.0);

pub const COMPASS_CAMP: Vec4 = Vec4::new(0.95, 0.62, 0.45, 1.0);

pub const COMPASS_QUEST_MARK: usize = 0;

pub const COMPASS_CITY_MARK: usize = 1;

pub const COMPASS_BARROW_MARK: usize = 2;

pub const COMPASS_CAMP_MARK: usize = 3;

pub const COMPASS_PLACE_MARK: usize = 4;

pub const COMPASS_GLYPHS: [(&str, Vec4); 5] = [
    ("!", COMPASS_QUEST),
    ("#", COMPASS_CITY),
    ("+", COMPASS_BARROW),
    ("^", COMPASS_CAMP),
    ("*", COMPASS_PLACE),
];

pub const COMPASS_POINTS: [(&str, f32); 8] = [
    ("S", 0.0),
    ("SE", 45.0),
    ("E", 90.0),
    ("NE", 135.0),
    ("N", 180.0),
    ("NW", 225.0),
    ("W", 270.0),
    ("SW", 315.0),
];

pub const SIGHT_LABELS: [&str; 4] = ["Near", "Middle", "Far", "Furthest"];

pub const SIGHT_DEFAULT: usize = 1;
