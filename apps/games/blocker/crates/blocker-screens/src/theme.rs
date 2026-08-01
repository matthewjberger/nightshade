use nalgebra_glm::Vec4;

pub const CHEST_ROW: usize = 9;

pub const HELD_COLOR: Vec4 = Vec4::new(0.28, 0.24, 0.14, 1.0);

pub const HELD_BORDER: Vec4 = Vec4::new(1.0, 0.84, 0.45, 1.0);

pub const READY_COLOR: Vec4 = Vec4::new(0.62, 0.9, 0.55, 1.0);

pub const SETTINGS_HEIGHT: f32 = 397.0;

pub const TITLE_MENU_HEIGHT: f32 = 346.0;

pub const LOADING_HEIGHT: f32 = 268.0;

pub const TIP_SECONDS: f32 = 5.0;

pub const LOADING_BAR_HEIGHT: f32 = 18.0;

pub const LOADING_BAR_INSET: f32 = 3.0;

pub const LOADING_TRACK: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.10);

pub const NEW_WORLD_HEIGHT: f32 = 540.0;

pub const POCKET_HEIGHT: f32 = 560.0;

pub const CHEST_HEIGHT: f32 = 640.0;

pub const CRAFTING_HEIGHT: f32 = 560.0;

pub const SETUP_HEIGHT: f32 = 240.0;

pub const CHARACTER_WIDTH: f32 = 860.0;

pub const CHARACTER_HEIGHT: f32 = 640.0;

pub const CHARACTER_CAPTION: f32 = 48.0;

pub const CHARACTER_BARS_HEIGHT: f32 = 4.0 + 3.0 * (blocker_hud::theme::BAR_HEIGHT + 6.0);

pub const CHARACTER_GAP: f32 = 14.0;

pub const CLEAR_COLOR: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.0);

pub const COLUMN_COLOR: Vec4 = Vec4::new(0.08, 0.08, 0.11, 0.92);

pub const CHARACTER_STAND_WIDTH: f32 = 300.0;

pub const CHARACTER_STAT_WIDTH: f32 = 300.0;

pub const CHARACTER_SLAIN_WIDTH: f32 = 190.0;

pub const CHARACTER_NAME_WIDTH: f32 = 140.0;

pub const CHARACTER_SLAIN_NAME_WIDTH: f32 = 130.0;

pub const JOURNAL_HEIGHT: f32 = 600.0;

pub const DIALOGUE_HEIGHT: f32 = 600.0;

pub const DIALOGUE_WIDTH: f32 = 486.0;

pub const DIALOGUE_ROW_WIDTH: f32 = DIALOGUE_WIDTH - 36.0;

pub const DIALOGUE_BODY_HEIGHT: f32 = 306.0;

pub const DIALOGUE_CHOICES: usize = 4;

pub const DIALOGUE_LINE_CHARACTERS: usize = 56;

pub const DIALOGUE_PAGE_LINES: usize = 12;

pub const SHOP_HEIGHT: f32 = 500.0;

pub const MAP_HEIGHT: f32 = 644.0;

pub const MAP_FIELD: f32 = 430.0;

pub const MAP_MARKERS: usize = 24;

pub const MAP_SPANS: [f32; 3] = [160.0, 320.0, 640.0];

pub const MAP_START_ZOOM: usize = 1;

pub const MAP_PIXELS: u32 = 256;

pub const MAP_REPAINT_STEP: f32 = 48.0;

pub const MAP_BLANK: [u8; 4] = [41, 48, 41, 255];

pub const MAP_TONE: f32 = 0.88;

pub const MAP_DEPTH: f32 = 24.0;

pub const MAP_DEPTH_SHADE: f32 = 0.55;

pub const MAP_SHADE: f32 = 0.1;

pub const MAP_SHADE_LIMIT: f32 = 0.35;

pub const MAP_MARK_SIZE: f32 = 20.0;

pub const MAP_GLYPH_SIZE: f32 = 18.0;

pub const MAP_NOSE: f32 = 6.0;

pub const MAP_NOSE_REACH: f32 = 11.0;

pub const MAP_INK: Vec4 = Vec4::new(0.05, 0.04, 0.03, 1.0);

pub const MAP_INK_WIDTH: f32 = 1.5;

pub const MAP_DOT: f32 = 10.0;

pub const MAP_HERE_COLOR: Vec4 = Vec4::new(1.0, 0.95, 0.4, 1.0);

pub const MAP_FIELD_COLOR: Vec4 = Vec4::new(0.16, 0.19, 0.16, 1.0);

pub const ROW_ICON: f32 = 26.0;

pub const NAME_WIDTH: f32 = 214.0;

pub const NEEDS_WIDTH: f32 = 224.0;

pub const SETTING_NAME_WIDTH: f32 = 396.0;

pub const SEED_WIDTH: f32 = 96.0;

pub const JOURNAL_ROWS: usize = 10;

pub const SIGHT_RADII: [i32; 4] = [10, 14, 18, 22];

pub const MAP_QUEST_MARK: usize = 0;

pub const MAP_CITY_MARK: usize = 1;

pub const MAP_BARROW_MARK: usize = 2;

pub const MAP_CAMP_MARK: usize = 3;

pub const MAP_PLACE_MARK: usize = 4;

pub const MAP_VILLAGE_MARK: usize = 5;

pub const MAP_GLYPHS: [(&str, Vec4); 6] = [
    ("!", blocker_hud::theme::COMPASS_QUEST),
    ("#", blocker_hud::theme::COMPASS_CITY),
    ("+", blocker_hud::theme::COMPASS_BARROW),
    ("^", blocker_hud::theme::COMPASS_CAMP),
    ("*", blocker_hud::theme::COMPASS_PLACE),
    ("o", blocker_hud::theme::MAP_VILLAGE_COLOR),
];

pub const MAP_LABELS: usize = 8;

pub const MAP_LABEL_WIDTH: f32 = 148.0;

pub const MAP_LABEL_LIFTS: [f32; 6] = [-13.0, 15.0, -26.0, 28.0, -39.0, 41.0];

pub const MAP_LABEL_STEP: f32 = 13.0;

pub const MAP_LEGEND: [&str; 6] = ["errand", "city", "barrow", "camp", "place", "village"];

pub const MAP_LEGEND_HEIGHT: f32 = 22.0;

pub const MAP_BAR_BLOCKS: [f32; 3] = [40.0, 80.0, 160.0];

pub const MAP_BAR_HEIGHT: f32 = 4.0;

pub const MAP_WINDS: [&str; 8] = [
    "north",
    "north-east",
    "east",
    "south-east",
    "south",
    "south-west",
    "west",
    "north-west",
];

pub const MAP_HERE_EDGE: Vec4 = Vec4::new(0.1, 0.09, 0.06, 1.0);

pub const JOURNAL_LIST_WIDTH: f32 = 250.0;

pub const JOURNAL_DETAIL_WIDTH: f32 = 366.0;

const fn body_column(height: f32) -> f32 {
    height - blocker_hud::theme::TITLE_HEIGHT - blocker_hud::theme::FOOTER_HEIGHT - 20.0
}

const fn journal_fits() -> bool {
    let rows = JOURNAL_ROWS as f32
        * (blocker_hud::theme::ACTION_HEIGHT + 6.0 + blocker_hud::theme::ROW_GAP);
    let detail = 22.0 + 22.0 + 132.0 + 92.0 + 22.0 + 22.0 + 30.0 + 16.0;
    let wide =
        JOURNAL_LIST_WIDTH + JOURNAL_DETAIL_WIDTH + 10.0 + blocker_hud::theme::BODY_PADDING * 2.0;
    rows <= body_column(JOURNAL_HEIGHT)
        && detail <= body_column(JOURNAL_HEIGHT)
        && wide <= blocker_hud::theme::MENU_WIDTH
}

const fn dialogue_fits() -> bool {
    let rows = DIALOGUE_CHOICES as f32 * (blocker_hud::theme::ACTION_HEIGHT + 6.0);
    let stack = DIALOGUE_BODY_HEIGHT
        + 44.0
        + rows
        + blocker_hud::theme::ROW_GAP * 5.0
        + blocker_hud::theme::BODY_PADDING * 2.0;
    stack <= DIALOGUE_HEIGHT - blocker_hud::theme::TITLE_HEIGHT
}

const fn map_fits() -> bool {
    MAP_FIELD
        + blocker_hud::theme::BODY_PADDING
        + 8.0
        + MAP_LEGEND_HEIGHT
        + blocker_hud::theme::ROW_HEIGHT
        <= body_column(MAP_HEIGHT) + 20.0
        && MAP_BAR_BLOCKS.len() == MAP_SPANS.len()
        && MAP_LEGEND.len() == MAP_GLYPHS.len()
        && MAP_FIELD <= blocker_hud::theme::MENU_WIDTH - blocker_hud::theme::BODY_PADDING * 2.0
}

const _: () = assert!(journal_fits());
const _: () = assert!(dialogue_fits());
const fn character_fits() -> bool {
    let wide = CHARACTER_STAND_WIDTH
        + CHARACTER_STAT_WIDTH
        + CHARACTER_SLAIN_WIDTH
        + CHARACTER_GAP * 2.0
        + blocker_hud::theme::BODY_PADDING * 2.0;
    let rows = 7.0 * (blocker_hud::theme::ROW_HEIGHT + blocker_hud::theme::ROW_GAP)
        + 12.0
        + CHARACTER_BARS_HEIGHT
        + blocker_hud::theme::ROW_GAP;
    let slain = blocker_hud::theme::SMALL_SIZE
        + 10.0
        + blocker_hud::theme::ROW_GAP
        + 7.0 * (blocker_hud::theme::ROW_HEIGHT + blocker_hud::theme::ROW_GAP)
        + 12.0;
    let stand = 352.0 + 32.0 + CHARACTER_CAPTION;
    let column = body_column(CHARACTER_HEIGHT);
    wide <= CHARACTER_WIDTH && rows <= column && slain <= column && stand <= column
}

const _: () = assert!(character_fits());

const _: () = assert!(map_fits());

pub const DIALOGUE_LEFT: f32 = 40.0;

pub const DIALOGUE_TOP: f32 = 15.0;
