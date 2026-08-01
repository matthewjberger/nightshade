use nightshade_text::contract::data::text_data::FontKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IconGlyph {
    pub font_kind: FontKind,
    pub codepoint: char,
}

impl IconGlyph {
    pub const fn material(codepoint: char) -> Self {
        Self {
            font_kind: FontKind::IconsMaterial,
            codepoint,
        }
    }

    pub const fn lucide(codepoint: char) -> Self {
        Self {
            font_kind: FontKind::IconsLucide,
            codepoint,
        }
    }
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum IconSet {
    #[default]
    Material,
    Lucide,
}

pub struct IconLookup {
    pub home: IconGlyph,
    pub settings: IconGlyph,
    pub search: IconGlyph,
    pub close: IconGlyph,
    pub menu: IconGlyph,
    pub add: IconGlyph,
    pub remove: IconGlyph,
    pub delete: IconGlyph,
    pub refresh: IconGlyph,
    pub shuffle: IconGlyph,
    pub folder: IconGlyph,
    pub folder_open: IconGlyph,
    pub file: IconGlyph,
    pub videocam: IconGlyph,
    pub lightbulb: IconGlyph,
    pub mesh: IconGlyph,
    pub tune: IconGlyph,
    pub visibility: IconGlyph,
    pub visibility_off: IconGlyph,
    pub crop_free: IconGlyph,
    pub grid: IconGlyph,
    pub palette: IconGlyph,
    pub terrain: IconGlyph,
    pub public: IconGlyph,
    pub photo: IconGlyph,
    pub collections: IconGlyph,
    pub paintbrush: IconGlyph,
    pub view_quilt: IconGlyph,
    pub arrow_forward: IconGlyph,
    pub arrow_back: IconGlyph,
    pub arrow_drop_down: IconGlyph,
    pub arrow_drop_up: IconGlyph,
    pub chevron_left: IconGlyph,
    pub chevron_right: IconGlyph,
    pub play_arrow: IconGlyph,
    pub pause: IconGlyph,
    pub stop: IconGlyph,
    pub check: IconGlyph,
    pub error_outline: IconGlyph,
    pub warning: IconGlyph,
    pub info: IconGlyph,
    pub speed: IconGlyph,
    pub timer: IconGlyph,
    pub token: IconGlyph,
    pub format_list_bulleted: IconGlyph,
    pub account_tree: IconGlyph,
    pub table_chart: IconGlyph,
    pub sliders: IconGlyph,
    pub keyboard: IconGlyph,
    pub text_fields: IconGlyph,
    pub shopping_basket: IconGlyph,
    pub toggle_on: IconGlyph,
    pub auto_awesome: IconGlyph,
    pub open_with: IconGlyph,
    pub rotate_left: IconGlyph,
    pub zoom_out_map: IconGlyph,
    pub language: IconGlyph,
}

impl IconLookup {
    pub const fn for_set(set: IconSet) -> &'static Self {
        match set {
            IconSet::Material => &MATERIAL_LOOKUP,
            IconSet::Lucide => &LUCIDE_LOOKUP,
        }
    }
}

const MATERIAL_LOOKUP: IconLookup = IconLookup {
    home: material::HOME,
    settings: material::SETTINGS,
    search: material::SEARCH,
    close: material::CLOSE,
    menu: material::MENU,
    add: material::ADD,
    remove: material::REMOVE,
    delete: material::DELETE,
    refresh: material::REFRESH,
    shuffle: material::SHUFFLE,
    folder: material::FOLDER,
    folder_open: material::FOLDER_OPEN,
    file: material::INSERT_DRIVE_FILE,
    videocam: material::VIDEOCAM,
    lightbulb: material::LIGHTBULB,
    mesh: material::VIEW_IN_AR,
    tune: material::TUNE,
    visibility: material::VISIBILITY,
    visibility_off: material::VISIBILITY_OFF,
    crop_free: material::CROP_FREE,
    grid: material::GRID_ON,
    palette: material::PALETTE,
    terrain: material::TERRAIN,
    public: material::PUBLIC,
    photo: material::PHOTO,
    collections: material::COLLECTIONS,
    paintbrush: material::FORMAT_PAINT,
    view_quilt: material::VIEW_QUILT,
    arrow_forward: material::ARROW_FORWARD,
    arrow_back: material::ARROW_BACK,
    arrow_drop_down: material::ARROW_DROP_DOWN,
    arrow_drop_up: material::ARROW_DROP_UP,
    chevron_left: material::CHEVRON_LEFT,
    chevron_right: material::CHEVRON_RIGHT,
    play_arrow: material::PLAY_ARROW,
    pause: material::PAUSE,
    stop: material::STOP,
    check: material::CHECK,
    error_outline: material::ERROR_OUTLINE,
    warning: material::WARNING,
    info: material::INFO,
    speed: material::SPEED,
    timer: material::TIMER,
    token: material::TOKEN,
    format_list_bulleted: material::FORMAT_LIST_BULLETED,
    account_tree: material::ACCOUNT_TREE,
    table_chart: material::TABLE_CHART,
    sliders: material::SLIDERS,
    keyboard: material::KEYBOARD,
    text_fields: material::TEXT_FIELDS,
    shopping_basket: material::SHOPPING_BASKET,
    toggle_on: material::TOGGLE_ON,
    auto_awesome: material::AUTO_AWESOME,
    open_with: material::OPEN_WITH,
    rotate_left: material::ROTATE_LEFT,
    zoom_out_map: material::ZOOM_OUT_MAP,
    language: material::LANGUAGE,
};

const LUCIDE_LOOKUP: IconLookup = IconLookup {
    home: lucide::HOUSE,
    settings: lucide::SETTINGS,
    search: lucide::SEARCH,
    close: lucide::X,
    menu: lucide::MENU,
    add: lucide::PLUS,
    remove: lucide::MINUS,
    delete: lucide::TRASH,
    refresh: lucide::REFRESH,
    shuffle: lucide::SHUFFLE,
    folder: lucide::FOLDER,
    folder_open: lucide::FOLDER_OPEN,
    file: lucide::FILE,
    videocam: lucide::VIDEO,
    lightbulb: lucide::LIGHTBULB,
    mesh: lucide::BOX,
    tune: lucide::SLIDERS_HORIZONTAL,
    visibility: lucide::EYE,
    visibility_off: lucide::EYE_OFF,
    crop_free: lucide::MAXIMIZE,
    grid: lucide::GRID,
    palette: lucide::PALETTE,
    terrain: lucide::MOUNTAIN,
    public: lucide::GLOBE,
    photo: lucide::IMAGE,
    collections: lucide::IMAGES,
    paintbrush: lucide::PAINTBRUSH,
    view_quilt: lucide::LAYOUT_GRID,
    arrow_forward: lucide::ARROW_RIGHT,
    arrow_back: lucide::ARROW_LEFT,
    arrow_drop_down: lucide::CHEVRON_DOWN,
    arrow_drop_up: lucide::CHEVRON_UP,
    chevron_left: lucide::CHEVRON_LEFT,
    chevron_right: lucide::CHEVRON_RIGHT,
    play_arrow: lucide::PLAY,
    pause: lucide::PAUSE,
    stop: lucide::SQUARE,
    check: lucide::CHECK,
    error_outline: lucide::CIRCLE_ALERT,
    warning: lucide::TRIANGLE_ALERT,
    info: lucide::INFO,
    speed: lucide::GAUGE,
    timer: lucide::TIMER,
    token: lucide::COINS,
    format_list_bulleted: lucide::LIST,
    account_tree: lucide::NETWORK,
    table_chart: lucide::TABLE,
    sliders: lucide::SLIDERS_VERTICAL,
    keyboard: lucide::KEYBOARD,
    text_fields: lucide::TYPE,
    shopping_basket: lucide::SHOPPING_BASKET,
    toggle_on: lucide::TOGGLE_RIGHT,
    auto_awesome: lucide::SPARKLES,
    open_with: lucide::MAXIMIZE,
    rotate_left: lucide::REFRESH,
    zoom_out_map: lucide::MAXIMIZE,
    language: lucide::GLOBE,
};

pub mod material {
    use super::IconGlyph;

    pub const HOME: IconGlyph = IconGlyph::material('\u{e88a}');
    pub const SETTINGS: IconGlyph = IconGlyph::material('\u{e8b8}');
    pub const SEARCH: IconGlyph = IconGlyph::material('\u{e8b6}');
    pub const CLOSE: IconGlyph = IconGlyph::material('\u{e5cd}');
    pub const MENU: IconGlyph = IconGlyph::material('\u{e5d2}');
    pub const ADD: IconGlyph = IconGlyph::material('\u{e145}');
    pub const REMOVE: IconGlyph = IconGlyph::material('\u{e15b}');
    pub const DELETE: IconGlyph = IconGlyph::material('\u{e872}');
    pub const REFRESH: IconGlyph = IconGlyph::material('\u{e5d5}');
    pub const SHUFFLE: IconGlyph = IconGlyph::material('\u{e043}');
    pub const FOLDER: IconGlyph = IconGlyph::material('\u{e2c7}');
    pub const FOLDER_OPEN: IconGlyph = IconGlyph::material('\u{e2c8}');
    pub const INSERT_DRIVE_FILE: IconGlyph = IconGlyph::material('\u{e24d}');
    pub const CODE: IconGlyph = IconGlyph::material('\u{e86f}');

    pub const VIDEOCAM: IconGlyph = IconGlyph::material('\u{e04b}');
    pub const LIGHTBULB: IconGlyph = IconGlyph::material('\u{e0f0}');
    pub const VIEW_IN_AR: IconGlyph = IconGlyph::material('\u{e9fe}');
    pub const TUNE: IconGlyph = IconGlyph::material('\u{e429}');
    pub const VISIBILITY: IconGlyph = IconGlyph::material('\u{e8f4}');
    pub const VISIBILITY_OFF: IconGlyph = IconGlyph::material('\u{e8f5}');
    pub const CROP_FREE: IconGlyph = IconGlyph::material('\u{e3c2}');
    pub const GRID_ON: IconGlyph = IconGlyph::material('\u{e3ec}');
    pub const PALETTE: IconGlyph = IconGlyph::material('\u{e40a}');
    pub const TERRAIN: IconGlyph = IconGlyph::material('\u{e564}');
    pub const PUBLIC: IconGlyph = IconGlyph::material('\u{e80b}');
    pub const PHOTO: IconGlyph = IconGlyph::material('\u{e410}');
    pub const COLLECTIONS: IconGlyph = IconGlyph::material('\u{e431}');
    pub const TEXTURE: IconGlyph = IconGlyph::material('\u{e421}');
    pub const FORMAT_PAINT: IconGlyph = IconGlyph::material('\u{e243}');
    pub const TV: IconGlyph = IconGlyph::material('\u{e333}');
    pub const VIEW_QUILT: IconGlyph = IconGlyph::material('\u{e8f9}');

    pub const ARROW_FORWARD: IconGlyph = IconGlyph::material('\u{e5c8}');
    pub const ARROW_BACK: IconGlyph = IconGlyph::material('\u{e5c4}');
    pub const ARROW_DROP_DOWN: IconGlyph = IconGlyph::material('\u{e5c5}');
    pub const ARROW_DROP_UP: IconGlyph = IconGlyph::material('\u{e5c7}');
    pub const CHEVRON_LEFT: IconGlyph = IconGlyph::material('\u{e5cb}');
    pub const CHEVRON_RIGHT: IconGlyph = IconGlyph::material('\u{e5cc}');
    pub const PLAY_ARROW: IconGlyph = IconGlyph::material('\u{e037}');
    pub const PAUSE: IconGlyph = IconGlyph::material('\u{e034}');
    pub const STOP: IconGlyph = IconGlyph::material('\u{e047}');

    pub const CHECK: IconGlyph = IconGlyph::material('\u{e5ca}');
    pub const ERROR_OUTLINE: IconGlyph = IconGlyph::material('\u{e001}');
    pub const WARNING: IconGlyph = IconGlyph::material('\u{e002}');
    pub const INFO: IconGlyph = IconGlyph::material('\u{e88e}');

    pub const SPEED: IconGlyph = IconGlyph::material('\u{e9e4}');
    pub const TIMER: IconGlyph = IconGlyph::material('\u{e425}');
    pub const TOKEN: IconGlyph = IconGlyph::material('\u{ea25}');

    pub const BRUSH: IconGlyph = IconGlyph::material('\u{e3ae}');
    pub const FORMAT_LIST_BULLETED: IconGlyph = IconGlyph::material('\u{e241}');
    pub const ACCOUNT_TREE: IconGlyph = IconGlyph::material('\u{e97a}');
    pub const TABLE_CHART: IconGlyph = IconGlyph::material('\u{e265}');
    pub const SLIDERS: IconGlyph = IconGlyph::material('\u{e429}');
    pub const KEYBOARD: IconGlyph = IconGlyph::material('\u{e312}');
    pub const TEXT_FIELDS: IconGlyph = IconGlyph::material('\u{e262}');
    pub const SHOPPING_BASKET: IconGlyph = IconGlyph::material('\u{e8cb}');
    pub const TOGGLE_ON: IconGlyph = IconGlyph::material('\u{e9f6}');
    pub const AUTO_AWESOME: IconGlyph = IconGlyph::material('\u{e65f}');
    pub const OPEN_WITH: IconGlyph = IconGlyph::material('\u{e89f}');
    pub const ROTATE_LEFT: IconGlyph = IconGlyph::material('\u{e419}');
    pub const ZOOM_OUT_MAP: IconGlyph = IconGlyph::material('\u{e56b}');
    pub const LANGUAGE: IconGlyph = IconGlyph::material('\u{e894}');
}

pub mod lucide {
    use super::IconGlyph;

    pub const HOUSE: IconGlyph = IconGlyph::lucide('\u{e0f5}');
    pub const SETTINGS: IconGlyph = IconGlyph::lucide('\u{e154}');
    pub const SEARCH: IconGlyph = IconGlyph::lucide('\u{e151}');
    pub const X: IconGlyph = IconGlyph::lucide('\u{e1b2}');
    pub const MENU: IconGlyph = IconGlyph::lucide('\u{e115}');
    pub const PLUS: IconGlyph = IconGlyph::lucide('\u{e13d}');
    pub const MINUS: IconGlyph = IconGlyph::lucide('\u{e11c}');
    pub const TRASH: IconGlyph = IconGlyph::lucide('\u{e18e}');
    pub const REFRESH: IconGlyph = IconGlyph::lucide('\u{e145}');
    pub const SHUFFLE: IconGlyph = IconGlyph::lucide('\u{e15e}');
    pub const FOLDER: IconGlyph = IconGlyph::lucide('\u{e0d7}');
    pub const FOLDER_OPEN: IconGlyph = IconGlyph::lucide('\u{e247}');
    pub const FILE: IconGlyph = IconGlyph::lucide('\u{e0c0}');

    pub const VIDEO: IconGlyph = IconGlyph::lucide('\u{e1a5}');
    pub const LIGHTBULB: IconGlyph = IconGlyph::lucide('\u{e1c2}');
    pub const BOX: IconGlyph = IconGlyph::lucide('\u{e061}');
    pub const SLIDERS_HORIZONTAL: IconGlyph = IconGlyph::lucide('\u{e29a}');
    pub const EYE: IconGlyph = IconGlyph::lucide('\u{e0ba}');
    pub const EYE_OFF: IconGlyph = IconGlyph::lucide('\u{e0bb}');
    pub const MAXIMIZE: IconGlyph = IconGlyph::lucide('\u{e112}');
    pub const GRID: IconGlyph = IconGlyph::lucide('\u{e0e9}');
    pub const PALETTE: IconGlyph = IconGlyph::lucide('\u{e1dd}');
    pub const MOUNTAIN: IconGlyph = IconGlyph::lucide('\u{e231}');
    pub const GLOBE: IconGlyph = IconGlyph::lucide('\u{e0e8}');
    pub const IMAGE: IconGlyph = IconGlyph::lucide('\u{e0f6}');
    pub const IMAGES: IconGlyph = IconGlyph::lucide('\u{e5c4}');
    pub const PAINTBRUSH: IconGlyph = IconGlyph::lucide('\u{e2e7}');
    pub const LAYOUT_GRID: IconGlyph = IconGlyph::lucide('\u{e0ff}');

    pub const ARROW_RIGHT: IconGlyph = IconGlyph::lucide('\u{e049}');
    pub const ARROW_LEFT: IconGlyph = IconGlyph::lucide('\u{e048}');
    pub const CHEVRON_DOWN: IconGlyph = IconGlyph::lucide('\u{e06d}');
    pub const CHEVRON_UP: IconGlyph = IconGlyph::lucide('\u{e070}');
    pub const CHEVRON_LEFT: IconGlyph = IconGlyph::lucide('\u{e06e}');
    pub const CHEVRON_RIGHT: IconGlyph = IconGlyph::lucide('\u{e06f}');
    pub const PLAY: IconGlyph = IconGlyph::lucide('\u{e13c}');
    pub const PAUSE: IconGlyph = IconGlyph::lucide('\u{e12e}');
    pub const SQUARE: IconGlyph = IconGlyph::lucide('\u{e167}');

    pub const CHECK: IconGlyph = IconGlyph::lucide('\u{e06c}');
    pub const CIRCLE_ALERT: IconGlyph = IconGlyph::lucide('\u{e077}');
    pub const TRIANGLE_ALERT: IconGlyph = IconGlyph::lucide('\u{e193}');
    pub const INFO: IconGlyph = IconGlyph::lucide('\u{e0f9}');

    pub const GAUGE: IconGlyph = IconGlyph::lucide('\u{e1bf}');
    pub const TIMER: IconGlyph = IconGlyph::lucide('\u{e1e0}');
    pub const COINS: IconGlyph = IconGlyph::lucide('\u{e097}');

    pub const LIST: IconGlyph = IconGlyph::lucide('\u{e106}');
    pub const NETWORK: IconGlyph = IconGlyph::lucide('\u{e125}');
    pub const TABLE: IconGlyph = IconGlyph::lucide('\u{e17d}');
    pub const SLIDERS_VERTICAL: IconGlyph = IconGlyph::lucide('\u{e162}');
    pub const KEYBOARD: IconGlyph = IconGlyph::lucide('\u{e284}');
    pub const TYPE: IconGlyph = IconGlyph::lucide('\u{e198}');
    pub const SHOPPING_BASKET: IconGlyph = IconGlyph::lucide('\u{e4ea}');
    pub const TOGGLE_RIGHT: IconGlyph = IconGlyph::lucide('\u{e18c}');
    pub const SPARKLES: IconGlyph = IconGlyph::lucide('\u{e412}');
}
