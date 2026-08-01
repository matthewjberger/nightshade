use nightshade::prelude::{Vec3, Vec4};

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Need {
    #[default]
    Energy,
    Hunger,
    Hygiene,
    Fun,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Category {
    #[default]
    Build,
    Buy,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Placement {
    #[default]
    Floor,
    Cell,
    Edge,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kit {
    #[default]
    Furniture,
    Building,
}

pub const CATEGORIES: [Category; 2] = [Category::Build, Category::Buy];

pub fn category_name(category: Category) -> &'static str {
    match category {
        Category::Build => "Build",
        Category::Buy => "Buy",
    }
}

pub const NEEDS: [Need; 4] = [Need::Energy, Need::Hunger, Need::Hygiene, Need::Fun];

pub const NEED_DECAY: [f32; 4] = [0.8, 1.1, 0.65, 0.95];

pub const NEED_COLOR: [Vec4; 4] = [
    Vec4::new(0.55, 0.72, 1.0, 1.0),
    Vec4::new(1.0, 0.72, 0.32, 1.0),
    Vec4::new(0.48, 0.88, 0.78, 1.0),
    Vec4::new(0.92, 0.60, 0.90, 1.0),
];

pub fn need_index(need: Need) -> usize {
    match need {
        Need::Energy => 0,
        Need::Hunger => 1,
        Need::Hygiene => 2,
        Need::Fun => 3,
    }
}

pub fn need_name(need: Need) -> &'static str {
    match need {
        Need::Energy => "Rest",
        Need::Hunger => "Food",
        Need::Hygiene => "Clean",
        Need::Fun => "Fun",
    }
}

pub struct Piece {
    pub label: &'static str,
    pub bytes: &'static [u8],
    pub kit: Kit,
    pub category: Category,
    pub placement: Placement,
    pub need: Option<Need>,
    pub price: u32,
    pub fill_rate: f32,
    pub blocks: bool,
    pub sits: bool,
}

pub const BUILDING_COLORMAP: &[u8] = include_bytes!("../assets/building/colormap.png");
pub const CHARACTER_COLORMAP: &[u8] = include_bytes!("../assets/character/colormap.png");
pub const COLORMAP_URI: &str = "Textures/colormap.png";
pub const CHARACTER_MODEL: &[u8] = include_bytes!("../assets/character/character-female-a.glb");

pub const CATALOG: &[Piece] = &[
    Piece {
        label: "Floor",
        bytes: include_bytes!("../assets/building/floor.glb"),
        kit: Kit::Building,
        category: Category::Build,
        placement: Placement::Floor,
        need: None,
        price: 30,
        fill_rate: 0.0,
        blocks: false,
        sits: false,
    },
    Piece {
        label: "Wall",
        bytes: include_bytes!("../assets/building/wall.glb"),
        kit: Kit::Building,
        category: Category::Build,
        placement: Placement::Edge,
        need: None,
        price: 70,
        fill_rate: 0.0,
        blocks: true,
        sits: false,
    },
    Piece {
        label: "Door",
        bytes: include_bytes!("../assets/building/wall-doorway-square.glb"),
        kit: Kit::Building,
        category: Category::Build,
        placement: Placement::Edge,
        need: None,
        price: 110,
        fill_rate: 0.0,
        blocks: false,
        sits: false,
    },
    Piece {
        label: "Window",
        bytes: include_bytes!("../assets/building/wall-window-square.glb"),
        kit: Kit::Building,
        category: Category::Build,
        placement: Placement::Edge,
        need: None,
        price: 130,
        fill_rate: 0.0,
        blocks: true,
        sits: false,
    },
    Piece {
        label: "Bed",
        bytes: include_bytes!("../assets/furniture/bedSingle.glb"),
        kit: Kit::Furniture,
        category: Category::Buy,
        placement: Placement::Cell,
        need: Some(Need::Energy),
        price: 240,
        fill_rate: 16.0,
        blocks: true,
        sits: true,
    },
    Piece {
        label: "Fridge",
        bytes: include_bytes!("../assets/furniture/kitchenFridge.glb"),
        kit: Kit::Furniture,
        category: Category::Buy,
        placement: Placement::Cell,
        need: Some(Need::Hunger),
        price: 300,
        fill_rate: 22.0,
        blocks: true,
        sits: false,
    },
    Piece {
        label: "Stove",
        bytes: include_bytes!("../assets/furniture/kitchenStove.glb"),
        kit: Kit::Furniture,
        category: Category::Buy,
        placement: Placement::Cell,
        need: Some(Need::Hunger),
        price: 360,
        fill_rate: 28.0,
        blocks: true,
        sits: false,
    },
    Piece {
        label: "Bath",
        bytes: include_bytes!("../assets/furniture/bathtub.glb"),
        kit: Kit::Furniture,
        category: Category::Buy,
        placement: Placement::Cell,
        need: Some(Need::Hygiene),
        price: 380,
        fill_rate: 20.0,
        blocks: true,
        sits: true,
    },
    Piece {
        label: "Toilet",
        bytes: include_bytes!("../assets/furniture/toilet.glb"),
        kit: Kit::Furniture,
        category: Category::Buy,
        placement: Placement::Cell,
        need: Some(Need::Hygiene),
        price: 120,
        fill_rate: 12.0,
        blocks: true,
        sits: true,
    },
    Piece {
        label: "TV",
        bytes: include_bytes!("../assets/furniture/televisionModern.glb"),
        kit: Kit::Furniture,
        category: Category::Buy,
        placement: Placement::Cell,
        need: Some(Need::Fun),
        price: 220,
        fill_rate: 18.0,
        blocks: true,
        sits: false,
    },
    Piece {
        label: "Sofa",
        bytes: include_bytes!("../assets/furniture/loungeSofa.glb"),
        kit: Kit::Furniture,
        category: Category::Buy,
        placement: Placement::Cell,
        need: Some(Need::Fun),
        price: 280,
        fill_rate: 13.0,
        blocks: true,
        sits: true,
    },
    Piece {
        label: "Desk",
        bytes: include_bytes!("../assets/furniture/desk.glb"),
        kit: Kit::Furniture,
        category: Category::Buy,
        placement: Placement::Cell,
        need: Some(Need::Fun),
        price: 200,
        fill_rate: 10.0,
        blocks: true,
        sits: true,
    },
];

pub const STARTING_MONEY: u32 = 1200;
pub const WAGE_CONTENT: f32 = 26.0;
pub const WAGE_STRAINED: f32 = 3.0;
pub const CONTENT_LINE: f32 = 30.0;
pub const INDOOR_BONUS: f32 = 0.5;

pub const NEED_FULL: f32 = 100.0;
pub const NEED_SATISFIED: f32 = 92.0;

pub const COLUMNS: usize = 12;
pub const ROWS: usize = 9;
pub const GRID: f32 = 2.0;
pub const LOT_HALF_WIDTH: f32 = COLUMNS as f32 * GRID * 0.5;
pub const LOT_HALF_DEPTH: f32 = ROWS as f32 * GRID * 0.5;
pub const FLOOR_DROP: f32 = -0.1;
pub const GRASS_DROP: f32 = -0.12;

pub const DWELLER_SPEED: f32 = 2.4;
pub const DWELLER_TURN: f32 = 9.0;
pub const DWELLER_REACH: f32 = 0.22;
pub const DWELLER_SCALE: f32 = 2.1;
pub const DWELLER_START_CELL: (usize, usize) = (1, 7);

pub const CLIP_IDLE: &str = "idle";
pub const CLIP_WALK: &str = "walk";
pub const CLIP_SIT: &str = "sit";

pub const CAMERA_FOCUS: [f32; 3] = [0.0, 0.5, 0.0];
pub const CAMERA_RADIUS: f32 = 22.0;
pub const CAMERA_YAW: f32 = std::f32::consts::FRAC_PI_4;
pub const CAMERA_PITCH: f32 = 0.9;
pub const CAMERA_ZOOM_NEAR: f32 = 6.0;
pub const CAMERA_ZOOM_FAR: f32 = 46.0;

pub const SUN_STRENGTH: f32 = 2.6;
pub const SUN_COLOR: [f32; 3] = [1.0, 0.96, 0.88];
pub const EXPOSURE_EV: f32 = -0.6;
pub const SSAO_STRENGTH: f32 = 0.85;

pub const PARKED: Vec3 = Vec3::new(0.0, -80.0, 0.0);
pub const PAD_LIFT: f32 = 0.02;
pub const PAD_THICKNESS: f32 = 0.16;
pub const CLIP_BLEND: f32 = 0.18;
pub const MAXMONEY_GRANT: u32 = 50_000;

pub const GROUND_MATERIAL: &str = "hearthside_ground";
pub const LOT_MATERIAL: &str = "hearthside_lot";
pub const GHOST_CLEAR_MATERIAL: &str = "hearthside_ghost_clear";
pub const GHOST_BLOCKED_MATERIAL: &str = "hearthside_ghost_blocked";

pub const GROUND_COLOR: [f32; 4] = [0.34, 0.46, 0.27, 1.0];
pub const LOT_COLOR: [f32; 4] = [0.47, 0.60, 0.35, 1.0];
pub const GHOST_CLEAR_COLOR: [f32; 4] = [0.40, 0.95, 0.55, 0.50];
pub const GHOST_BLOCKED_COLOR: [f32; 4] = [0.95, 0.35, 0.35, 0.50];

pub const PANEL_BACKGROUND: Vec4 = Vec4::new(0.05, 0.05, 0.08, 0.82);
pub const PANEL_BORDER: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.12);
pub const TEXT_PRIMARY: Vec4 = Vec4::new(0.96, 0.96, 0.98, 1.0);
pub const TEXT_DIM: Vec4 = Vec4::new(0.70, 0.72, 0.80, 1.0);
pub const TRACK_COLOR: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.14);

pub const MARGIN: f32 = 16.0;
pub const BAR_WIDTH: f32 = 220.0;
pub const BAR_HEIGHT: f32 = 16.0;
pub const ROW_HEIGHT: f32 = 30.0;
pub const LABEL_SIZE: f32 = 16.0;
pub const MONEY_SIZE: f32 = 26.0;
pub const PANEL_WIDTH: f32 = BAR_WIDTH + 150.0;
pub const MONEY_PANEL_HEIGHT: f32 = 76.0;
pub const NEEDS_PANEL_HEIGHT: f32 = ROW_HEIGHT * 4.0 + 46.0;
pub const BAR_LEFT: f32 = 76.0;
pub const READOUT_LEFT: f32 = BAR_LEFT + BAR_WIDTH + 8.0;
pub const PALETTE_SLOT_WIDTH: f32 = 118.0;
pub const PALETTE_SLOT_HEIGHT: f32 = 40.0;
pub const PALETTE_GAP: f32 = 8.0;
pub const TAB_WIDTH: f32 = 96.0;
pub const TAB_HEIGHT: f32 = 30.0;
pub const PALETTE_HEIGHT: f32 = PALETTE_SLOT_HEIGHT + TAB_HEIGHT + 28.0;
pub const STATUS_WIDTH: f32 = 860.0;
pub const STATUS_HEIGHT: f32 = 32.0;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    SlotOne,
    SlotTwo,
    SlotThree,
    SlotFour,
    SlotFive,
    SlotSix,
    SlotSeven,
    SlotEight,
    SwitchCategory,
    Rotate,
    Place,
    Quit,
}

impl Ask {
    pub const SLOTS: [Ask; 8] = [
        Ask::SlotOne,
        Ask::SlotTwo,
        Ask::SlotThree,
        Ask::SlotFour,
        Ask::SlotFive,
        Ask::SlotSix,
        Ask::SlotSeven,
        Ask::SlotEight,
    ];
}

impl nightshade_input::Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::SlotOne,
            Self::SlotTwo,
            Self::SlotThree,
            Self::SlotFour,
            Self::SlotFive,
            Self::SlotSix,
            Self::SlotSeven,
            Self::SlotEight,
            Self::SwitchCategory,
            Self::Rotate,
            Self::Place,
            Self::Quit,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::SlotOne => "slot one",
            Self::SlotTwo => "slot two",
            Self::SlotThree => "slot three",
            Self::SlotFour => "slot four",
            Self::SlotFive => "slot five",
            Self::SlotSix => "slot six",
            Self::SlotSeven => "slot seven",
            Self::SlotEight => "slot eight",
            Self::SwitchCategory => "switch category",
            Self::Rotate => "rotate",
            Self::Place => "place",
            Self::Quit => "quit",
        }
    }

    fn default_bindings(&self) -> &'static [nightshade_input::Binding] {
        use nightshade_input::{Binding, Pointer};
        use nightshade_platform::KeyCode;
        match self {
            Self::SlotOne => &[Binding::Key(KeyCode::Digit1)],
            Self::SlotTwo => &[Binding::Key(KeyCode::Digit2)],
            Self::SlotThree => &[Binding::Key(KeyCode::Digit3)],
            Self::SlotFour => &[Binding::Key(KeyCode::Digit4)],
            Self::SlotFive => &[Binding::Key(KeyCode::Digit5)],
            Self::SlotSix => &[Binding::Key(KeyCode::Digit6)],
            Self::SlotSeven => &[Binding::Key(KeyCode::Digit7)],
            Self::SlotEight => &[Binding::Key(KeyCode::Digit8)],
            Self::SwitchCategory => &[Binding::Key(KeyCode::Tab)],
            Self::Rotate => &[Binding::Key(KeyCode::KeyR)],
            Self::Place => &[Binding::Pointer(Pointer::Left)],
            Self::Quit => &[Binding::Key(KeyCode::Escape)],
        }
    }
}
