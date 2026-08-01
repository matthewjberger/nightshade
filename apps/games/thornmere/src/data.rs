pub const FIRE: &str = "thornmere_fire";
pub const FROST: &str = "thornmere_frost";
pub const HOLY: &str = "thornmere_holy";
pub const LIFE: &str = "thornmere_life";
pub const ICE: &str = "thornmere_ice";
pub const SOOT: &str = "thornmere_soot";
pub const VIOLET: &str = "thornmere_violet";
pub mod assets;
pub mod chatter;
pub mod items;
pub mod save;

use nightshade::prelude::{Vec2, Vec3, Vec4};
use nightshade_ui_menu::prelude::MenuStyle;

pub const ZONE_SIZE: f32 = 300.0;
pub const ZONE_MARGIN: f32 = 3.0;
pub const TERRAIN_SUBDIVISIONS: u32 = 300;

pub const TOWN: usize = 0;
pub const FIELD: usize = 1;
pub const WOOD: usize = 2;
pub const FORT: usize = 3;
pub const BARROW: usize = 4;
pub const LAKE: usize = 5;
pub const FARM: usize = 6;
pub const CRYPT: usize = 7;
pub const MINE: usize = 8;
pub const HARBOR: usize = 9;
pub const DEEP: usize = 10;
pub const STARTER: usize = 11;

pub struct Place {
    pub name: &'static str,
    pub center: Vec2,
    pub radius: f32,
    pub height: f32,
}

pub const PLACES: [Place; 12] = [
    Place {
        name: "Thornmere",
        center: Vec2::new(0.0, -100.0),
        radius: 72.0,
        height: 0.0,
    },
    Place {
        name: "The Boar Field",
        center: Vec2::new(-20.0, -26.0),
        radius: 20.0,
        height: 0.0,
    },
    Place {
        name: "Wolf Wood",
        center: Vec2::new(-78.0, -6.0),
        radius: 32.0,
        height: 1.2,
    },
    Place {
        name: "Redharrow Fort",
        center: Vec2::new(88.0, -28.0),
        radius: 16.0,
        height: 0.0,
    },
    Place {
        name: "The Barrow",
        center: Vec2::new(64.0, 72.0),
        radius: 22.0,
        height: 6.0,
    },
    Place {
        name: "Mirror Lake",
        center: Vec2::new(-84.0, 84.0),
        radius: 26.0,
        height: -4.0,
    },
    Place {
        name: "Holt's Farm",
        center: Vec2::new(90.0, -98.0),
        radius: 16.0,
        height: 0.0,
    },
    Place {
        name: "The Sunken Crypt",
        center: Vec2::new(64.0, 76.0),
        radius: 11.0,
        height: 0.0,
    },
    Place {
        name: "Ironjaw Mine",
        center: Vec2::new(112.0, 116.0),
        radius: 24.0,
        height: 5.0,
    },
    Place {
        name: "Mirror Harbor",
        center: Vec2::new(-56.0, 76.0),
        radius: 10.0,
        height: 0.0,
    },
    Place {
        name: "Ironjaw Deep",
        center: Vec2::new(112.0, 120.0),
        radius: 10.0,
        height: 0.0,
    },
    Place {
        name: "Fallowmere Wood",
        center: Vec2::new(0.0, -8.0),
        radius: 44.0,
        height: 0.0,
    },
];

pub const WATER_LEVEL: f32 = -1.4;
pub const SWIM_DRAFT: f32 = 1.2;
pub const SWIM_FACTOR: f32 = 0.6;
pub const JUMP_SPEED: f32 = 7.5;
pub const GRAVITY: f32 = 20.0;

pub const ROADS: [&[Vec2]; 7] = [
    &[
        Vec2::new(0.0, -53.0),
        Vec2::new(1.5, -42.0),
        Vec2::new(-2.5, -30.0),
        Vec2::new(2.5, -18.0),
        Vec2::new(-1.5, -6.0),
        Vec2::new(0.0, 4.0),
        Vec2::new(0.0, 20.0),
    ],
    &[
        Vec2::new(-2.0, 6.0),
        Vec2::new(-18.0, 10.0),
        Vec2::new(-36.0, 2.0),
        Vec2::new(-60.0, -2.0),
    ],
    &[
        Vec2::new(2.0, -30.0),
        Vec2::new(40.0, -30.0),
        Vec2::new(72.0, -28.0),
    ],
    &[
        Vec2::new(6.0, -50.0),
        Vec2::new(46.0, -72.0),
        Vec2::new(74.0, -92.0),
    ],
    &[
        Vec2::new(2.0, 20.0),
        Vec2::new(30.0, 42.0),
        Vec2::new(54.0, 56.0),
    ],
    &[
        Vec2::new(74.0, 84.0),
        Vec2::new(92.0, 100.0),
        Vec2::new(104.0, 104.0),
    ],
    &[
        Vec2::new(-2.0, 20.0),
        Vec2::new(-30.0, 46.0),
        Vec2::new(-50.0, 68.0),
    ],
];

pub const ROAD_TILE_STEP: f32 = 3.0;
pub const ROAD_WIDTH: f32 = 3.6;

pub const BOARD_SPOT: Vec2 = Vec2::new(5.0, -73.0);
pub const GATE_SPOT: Vec2 = Vec2::new(0.0, -50.0);
pub const SQUARE_SPOT: Vec2 = Vec2::new(0.0, -80.0);
pub const PLAYER_START: Vec2 = Vec2::new(2.0, -68.0);
pub const GRAVEYARD: Vec2 = Vec2::new(-44.0, -105.0);

pub const CITY_MIN: Vec2 = Vec2::new(-60.0, -136.0);
pub const CITY_MAX: Vec2 = Vec2::new(60.0, -64.0);
pub const CITY_RISE: f32 = 0.6;
pub const CITY_SKIRT: f32 = 3.0;
pub const WALL_SCALE: f32 = 4.5;
pub const GATE_HALF: f32 = 4.5;
pub const MOAT_BANK: f32 = 2.0;
pub const MOAT_WIDTH: f32 = 9.0;
pub const MOAT_DEPTH: f32 = 5.5;
pub const CANAL_Z: f32 = -97.0;
pub const CANAL_HALF: f32 = 2.2;
pub const CANAL_DEPTH: f32 = 4.0;
pub const KEEP_MIN: Vec2 = Vec2::new(-20.0, -136.0);
pub const KEEP_MAX: Vec2 = Vec2::new(20.0, -110.0);
pub const KEEP_RISE: f32 = 2.4;
pub const PLAZA_MIN: Vec2 = Vec2::new(-12.0, -92.0);
pub const PLAZA_MAX: Vec2 = Vec2::new(12.0, -74.0);
pub const CANAL_BRIDGES: [f32; 3] = [-30.0, 0.0, 30.0];

pub const CITY_STREETS: [&[Vec2]; 12] = [
    &[Vec2::new(0.0, -64.0), Vec2::new(0.0, -109.0)],
    &[Vec2::new(-54.0, -70.0), Vec2::new(54.0, -70.0)],
    &[Vec2::new(-54.0, -86.0), Vec2::new(-12.0, -86.0)],
    &[Vec2::new(12.0, -86.0), Vec2::new(54.0, -86.0)],
    &[Vec2::new(-54.0, -112.0), Vec2::new(-22.0, -112.0)],
    &[Vec2::new(22.0, -112.0), Vec2::new(54.0, -112.0)],
    &[Vec2::new(-54.0, -128.0), Vec2::new(-22.0, -128.0)],
    &[Vec2::new(22.0, -128.0), Vec2::new(54.0, -128.0)],
    &[Vec2::new(-30.0, -70.0), Vec2::new(-30.0, -128.0)],
    &[Vec2::new(30.0, -70.0), Vec2::new(30.0, -128.0)],
    &[Vec2::new(-54.0, -70.0), Vec2::new(-54.0, -128.0)],
    &[Vec2::new(54.0, -70.0), Vec2::new(54.0, -128.0)],
];

pub struct Pit {
    pub min: Vec2,
    pub max: Vec2,
    pub depth: f32,
}

pub const PITS: [Pit; 2] = [
    Pit {
        min: Vec2::new(52.0, 64.0),
        max: Vec2::new(76.0, 88.0),
        depth: 5.0,
    },
    Pit {
        min: Vec2::new(100.0, 108.0),
        max: Vec2::new(124.0, 132.0),
        depth: 6.0,
    },
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Interior {
    Home,
    Inn,
    Auction,
    Bank,
    Chapel,
    Keep,
    Smithy,
    Apothecary,
    Guard,
    Cabin,
    Warehouse,
}

pub const FURNITURE_SCALE: f32 = 2.8;

pub struct Block {
    pub spot: Vec2,
    pub yaw: f32,
    pub cells: usize,
    pub floors: usize,
    pub wood: bool,
    pub roof: &'static str,
    pub interior: Interior,
}

const NORTH: f32 = 0.0;
const SOUTH: f32 = std::f32::consts::PI;
const EAST: f32 = std::f32::consts::FRAC_PI_2;
const WEST: f32 = -std::f32::consts::FRAC_PI_2;

pub const AUCTION_HOUSE: Vec2 = Vec2::new(20.0, -78.0);
pub const BANK: Vec2 = Vec2::new(-20.0, -78.0);
pub const INN: Vec2 = Vec2::new(40.0, -78.0);
pub const APOTHECARY: Vec2 = Vec2::new(-38.0, -78.0);
pub const CATHEDRAL: Vec2 = Vec2::new(-42.0, -120.0);
pub const GUARD_HOUSE: Vec2 = Vec2::new(-24.0, -120.0);
pub const SMITHY: Vec2 = Vec2::new(24.0, -120.0);
pub const KEEP: Vec2 = Vec2::new(0.0, -124.0);
pub const STABLE: Vec2 = Vec2::new(48.0, -105.0);
pub const PARK: Vec2 = Vec2::new(48.0, -93.0);
pub const MAGE_TOWER: Vec2 = Vec2::new(-48.0, -93.0);

pub const CITY_BLOCKS: [Block; 18] = [
    Block {
        spot: AUCTION_HOUSE,
        yaw: WEST,
        cells: 3,
        floors: 3,
        wood: false,
        roof: "roof-high-point",
        interior: Interior::Auction,
    },
    Block {
        spot: BANK,
        yaw: EAST,
        cells: 3,
        floors: 2,
        wood: false,
        roof: "roof-point",
        interior: Interior::Bank,
    },
    Block {
        spot: INN,
        yaw: SOUTH,
        cells: 3,
        floors: 2,
        wood: true,
        roof: "roof-point",
        interior: Interior::Inn,
    },
    Block {
        spot: APOTHECARY,
        yaw: SOUTH,
        cells: 2,
        floors: 1,
        wood: false,
        roof: "roof-point",
        interior: Interior::Apothecary,
    },
    Block {
        spot: Vec2::new(-48.0, -78.0),
        yaw: SOUTH,
        cells: 2,
        floors: 2,
        wood: true,
        roof: "roof-point",
        interior: Interior::Home,
    },
    Block {
        spot: Vec2::new(19.0, -93.0),
        yaw: NORTH,
        cells: 2,
        floors: 1,
        wood: false,
        roof: "roof-point",
        interior: Interior::Home,
    },
    Block {
        spot: Vec2::new(38.0, -93.0),
        yaw: NORTH,
        cells: 2,
        floors: 2,
        wood: true,
        roof: "roof-point",
        interior: Interior::Home,
    },
    Block {
        spot: Vec2::new(-19.0, -93.0),
        yaw: NORTH,
        cells: 2,
        floors: 2,
        wood: false,
        roof: "roof-point",
        interior: Interior::Home,
    },
    Block {
        spot: Vec2::new(-38.0, -93.0),
        yaw: NORTH,
        cells: 2,
        floors: 1,
        wood: true,
        roof: "roof-point",
        interior: Interior::Home,
    },
    Block {
        spot: Vec2::new(19.0, -105.0),
        yaw: SOUTH,
        cells: 2,
        floors: 1,
        wood: true,
        roof: "roof-point",
        interior: Interior::Home,
    },
    Block {
        spot: Vec2::new(38.0, -105.0),
        yaw: SOUTH,
        cells: 2,
        floors: 2,
        wood: false,
        roof: "roof-point",
        interior: Interior::Home,
    },
    Block {
        spot: Vec2::new(-19.0, -105.0),
        yaw: SOUTH,
        cells: 2,
        floors: 2,
        wood: false,
        roof: "roof-point",
        interior: Interior::Home,
    },
    Block {
        spot: CATHEDRAL,
        yaw: NORTH,
        cells: 3,
        floors: 3,
        wood: false,
        roof: "roof-high-point",
        interior: Interior::Chapel,
    },
    Block {
        spot: GUARD_HOUSE,
        yaw: NORTH,
        cells: 2,
        floors: 2,
        wood: false,
        roof: "roof-point",
        interior: Interior::Guard,
    },
    Block {
        spot: SMITHY,
        yaw: NORTH,
        cells: 2,
        floors: 2,
        wood: false,
        roof: "roof-point",
        interior: Interior::Smithy,
    },
    Block {
        spot: Vec2::new(37.0, -120.0),
        yaw: NORTH,
        cells: 3,
        floors: 2,
        wood: true,
        roof: "roof-point",
        interior: Interior::Home,
    },
    Block {
        spot: Vec2::new(48.0, -120.0),
        yaw: NORTH,
        cells: 2,
        floors: 1,
        wood: true,
        roof: "roof-point",
        interior: Interior::Home,
    },
    Block {
        spot: KEEP,
        yaw: NORTH,
        cells: 4,
        floors: 3,
        wood: false,
        roof: "roof-high-point",
        interior: Interior::Keep,
    },
];

pub const GUARD_MODEL: &str = "character-soldier";
pub const GUARD_SCALE: f32 = 2.6;
pub const GUARD_SPEED: f32 = 3.6;
pub const GUARD_PAUSE_SECONDS: f32 = 2.5;
pub const GUARD_NAME: &str = "Thornmere Guard";
pub const GUARD_PLATE_COLOR: Vec4 = Vec4::new(0.70, 0.82, 1.0, 1.0);

pub const GUARD_POSTS: [(Vec2, f32); 8] = [
    (Vec2::new(-4.6, -66.5), SOUTH),
    (Vec2::new(4.6, -66.5), SOUTH),
    (Vec2::new(-4.6, -111.5), SOUTH),
    (Vec2::new(4.6, -111.5), SOUTH),
    (Vec2::new(-10.0, -76.0), SOUTH),
    (Vec2::new(10.0, -76.0), SOUTH),
    (Vec2::new(12.8, -71.0), WEST),
    (Vec2::new(-12.8, -71.0), EAST),
];

pub const GUARD_ROUTES: [&[Vec2]; 4] = [
    &[
        Vec2::new(3.0, -68.0),
        Vec2::new(3.0, -92.0),
        Vec2::new(-3.0, -92.0),
        Vec2::new(-3.0, -68.0),
    ],
    &[
        Vec2::new(51.0, -72.0),
        Vec2::new(51.0, -122.0),
        Vec2::new(33.0, -122.0),
        Vec2::new(33.0, -72.0),
    ],
    &[
        Vec2::new(-51.0, -72.0),
        Vec2::new(-33.0, -72.0),
        Vec2::new(-33.0, -122.0),
        Vec2::new(-51.0, -122.0),
    ],
    &[Vec2::new(-28.0, -100.2), Vec2::new(28.0, -100.2)],
];

pub const DOG_MODEL: &str = "animal-dog";
pub const DOG_SCALE: f32 = 2.2;
pub const DOG_LIFT: f32 = 0.46;
pub const DOG_SPEED: f32 = 4.2;
pub const DOG_ROUTES: [&[Vec2]; 2] = [
    &[
        Vec2::new(8.0, -76.0),
        Vec2::new(8.0, -90.0),
        Vec2::new(-8.0, -90.0),
        Vec2::new(-8.0, -76.0),
    ],
    &[
        Vec2::new(30.0, -100.0),
        Vec2::new(44.0, -113.0),
        Vec2::new(30.0, -126.0),
    ],
];

pub const HORSE_MODEL: &str = "animal-horse";
pub const HORSE_SCALE: f32 = 3.0;
pub const HORSE_LIFT: f32 = 0.93;
pub const HORSE_SPOTS: [(Vec2, f32); 4] = [
    (Vec2::new(45.0, -103.0), 0.4),
    (Vec2::new(48.0, -107.0), 2.6),
    (Vec2::new(51.0, -102.5), 1.4),
    (Vec2::new(51.5, -107.5), 3.6),
];

pub const BISON_MODEL: &str = "animal-bison";
pub const BISON_SCALE: f32 = 3.0;
pub const BISON_LIFT: f32 = 0.93;
pub const BISON_COUNT: usize = 6;

pub struct Townsfolk {
    pub name: &'static str,
    pub title: &'static str,
    pub model: &'static str,
    pub scale: f32,
    pub spot: Vec2,
    pub heading: f32,
}

pub const TOWNSFOLK: [Townsfolk; 6] = [
    Townsfolk {
        name: "Ysolde",
        title: "Flower Seller",
        model: "character-female-e",
        scale: 2.6,
        spot: Vec2::new(48.0, -87.0),
        heading: SOUTH,
    },
    Townsfolk {
        name: "Crier Tobben",
        title: "Town Crier",
        model: "character-male-c",
        scale: 2.6,
        spot: Vec2::new(4.0, -86.0),
        heading: SOUTH,
    },
    Townsfolk {
        name: "Rook",
        title: "Stablemaster",
        model: "character-male-e",
        scale: 2.6,
        spot: Vec2::new(48.0, -99.8),
        heading: NORTH,
    },
    Townsfolk {
        name: "Grocer Wenna",
        title: "Market Stall",
        model: "market-character-employee",
        scale: 2.6,
        spot: Vec2::new(-8.0, -88.5),
        heading: SOUTH,
    },
    Townsfolk {
        name: "Brother Cole",
        title: "Gravekeeper",
        model: "character-digger",
        scale: 2.9,
        spot: Vec2::new(-34.5, -102.0),
        heading: SOUTH,
    },
    Townsfolk {
        name: "Deckhand Mo",
        title: "Mirror Harbor",
        model: "character-female-f",
        scale: 2.6,
        spot: Vec2::new(-62.0, 80.0),
        heading: WEST,
    },
];

pub const CLOUD_COUNT: usize = 22;
pub const CLOUD_REACH: f32 = 210.0;
pub const CLOUD_FLOOR: f32 = 58.0;
pub const CLOUD_CEILING: f32 = 92.0;
pub const CLOUD_DRIFT: f32 = 1.6;
pub const BIRD_FLOCKS: usize = 3;
pub const BIRDS_PER_FLOCK: usize = 7;
pub const BIRD_HEIGHT: f32 = 34.0;
pub const BIRD_RADIUS: f32 = 26.0;
pub const BIRD_SPEED: f32 = 9.0;
pub const CLOUD_COLOR: [f32; 4] = [0.97, 0.97, 0.99, 1.0];
pub const FEATHER_COLOR: [f32; 4] = [0.16, 0.15, 0.17, 1.0];

pub const MAP_IMAGE: u32 = 384;
pub const MAP_PANEL: f32 = 720.0;
pub const MAP_ICON_CELL: u32 = 96;
pub const MAP_ATLAS: u32 = 512;
pub const MAP_MARKS: [(&str, usize); 11] = [
    ("castle", TOWN),
    ("treePines", WOOD),
    ("tent", FORT),
    ("graveyard", BARROW),
    ("lakeRound", LAKE),
    ("mill", FARM),
    ("skull", CRYPT),
    ("mine", MINE),
    ("dock", HARBOR),
    ("campfire", FIELD),
    ("house", STARTER),
];

pub const TREE_COUNT: usize = 130;
pub const MODELS_IN_FLIGHT: usize = 16;
pub const DRESS_BUDGET: usize = 48;
pub const SETTLE_FRAMES: u8 = 3;
pub const STARTER_TREE_COUNT: usize = 150;
pub const HAMLET: Vec2 = Vec2::new(10.0, 12.0);
pub const HAMLET_CLEARING: f32 = 22.0;
pub const WILD_TRIES: usize = 24;
pub const HAMLET_INN: Vec2 = Vec2::new(14.0, 14.0);
pub const HAMLET_HOUSE: Vec2 = Vec2::new(-12.0, 16.0);
pub const HAMLET_BARN: Vec2 = Vec2::new(16.0, 0.0);
pub const WOOD_TREE_COUNT: usize = 110;
pub const GRASS_COUNT: usize = 2200;
pub const FLOWER_COUNT: usize = 600;
pub const ROCK_COUNT: usize = 90;

pub const POPULATION: usize = 48;
pub const SIM_SPEED_MIN: f32 = 5.4;
pub const SIM_SPEED_MAX: f32 = 6.6;
pub const SIM_LEVEL_MIN: u32 = 2;
pub const SIM_LEVEL_MAX: u32 = 22;
pub const REST_MIN_SECONDS: f32 = 5.0;
pub const REST_MAX_SECONDS: f32 = 16.0;
pub const SIM_QUEST_KILLS: u32 = 6;
pub const SIM_QUEST_EXPERIENCE: u32 = 90;

pub const HEALTH_PER_LEVEL: f32 = 12.0;
pub const MANA_PER_LEVEL: f32 = 8.0;
pub const STRENGTH_PER_LEVEL: f32 = 2.0;
pub const MANA_REGEN: f32 = 6.0;
pub const REST_MANA_REGEN: f32 = 14.0;
pub const GLOBAL_COOLDOWN: f32 = 1.0;
pub const CRIT_CHANCE: f32 = 0.15;
pub const CRIT_MULTIPLIER: f32 = 1.5;
pub const SPRINT_MULTIPLIER: f32 = 1.6;
pub const CHILL_MULTIPLIER: f32 = 0.5;
pub const CHILL_SECONDS: f32 = 4.0;
pub const SELECT_RANGE: f32 = 30.0;
pub const HOVER_REFRESH_SECONDS: f32 = 0.2;
pub const APPROACH_STOP: f32 = 2.4;
pub const MELEE_REACH: f32 = 4.0;
pub const STANDOFF: f32 = 3.3;
pub const SEPARATION: f32 = 2.2;
pub const PLAYER_SPACE: f32 = 2.6;
pub const ATTACK_PERIOD: f32 = 1.6;
pub const MEND_RATE: f32 = 0.05;
pub const SINK_DEPTH: f32 = 60.0;
pub const GIVER_REACH: f32 = 3.5;
pub const PICKUP_REACH: f32 = 1.8;
pub const ARRIVE_REACH: f32 = 1.2;
pub const PLATE_HEIGHT: f32 = 2.2;
pub const PLATE_FONT_SIZE: f32 = 20.0;
pub const PLATE_RANGE: f32 = 38.0;
pub const BODY_RADIUS: f32 = 0.7;
pub const FLOATING_COUNT: usize = 40;
pub const FLOATING_SECONDS: f32 = 1.3;
pub const FLOATING_RISE: f32 = 1.4;
pub const CAMERA_DISTANCE: f32 = 9.0;
pub const CAMERA_PITCH: f32 = 0.35;

pub const DAY_SPEED: f32 = 0.008;
pub const SUN_FLOOR: f32 = 0.62;
pub const SUN_DISTANCE: f32 = 100.0;
pub const SUN_POWER: f32 = 3.5;
pub const MOON_DIRECTION: Vec3 = Vec3::new(0.3, 0.8, -0.4);
pub const MOON_POWER: f32 = 0.35;
pub const MOON_COLOR: Vec3 = Vec3::new(0.60, 0.70, 1.00);
pub const SUN_WARM: Vec3 = Vec3::new(1.0, 0.7, 0.4);
pub const SUN_WHITE: Vec3 = Vec3::new(1.0, 0.95, 0.8);
pub const START_HOUR: f32 = 9.5;
pub const NIGHT_EXPOSURE_EV: f32 = -1.6;
pub const OFFLINE_FRACTION_NIGHT: f32 = 0.55;
pub const OFFLINE_FRACTION_EVENING: f32 = 0.15;
pub const POPULATION_STEP_SECONDS: f32 = 2.5;

pub const CHAT_ROWS: usize = 7;
pub const CHAT_KEEP: usize = 160;
pub const LINE_GAP_SECONDS: f32 = 2.4;
pub const CHATTER_MIN_SECONDS: f32 = 16.0;
pub const CHATTER_MAX_SECONDS: f32 = 38.0;
pub const WHISPER_MIN_SECONDS: f32 = 45.0;
pub const WHISPER_MAX_SECONDS: f32 = 120.0;
pub const REPLY_MIN_SECONDS: f32 = 3.5;
pub const REPLY_MAX_SECONDS: f32 = 9.0;
pub const CAMP_LINE_CHANCE: f32 = 0.12;
pub const RETURN_LINE_CHANCE: f32 = 0.15;
pub const TURN_IN_LINE_CHANCE: f32 = 0.15;
pub const DEATH_THREAD_CHANCE: f32 = 0.6;
pub const DING_THREAD_CHANCE: f32 = 0.8;
pub const LOG_LINE_CHANCE: f32 = 0.5;

pub const ABILITY_COUNT: usize = 6;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Melee,
    Ranged,
    Chill,
    Heal,
    Cleave,
    Sprint,
}

pub struct Ability {
    pub name: &'static str,
    pub cooldown: f32,
    pub mana: f32,
    pub range: f32,
    pub power: f32,
    pub effect: Effect,
    pub icon: &'static str,
}

pub const ABILITY_TOTAL: usize = 30;

pub const ABILITIES: [Ability; ABILITY_TOTAL] = [
    Ability {
        name: "Heroic Strike",
        cooldown: 4.0,
        mana: 15.0,
        range: 4.2,
        power: 30.0,
        effect: Effect::Melee,
        icon: "fightFist",
    },
    Ability {
        name: "Fireball",
        cooldown: 5.0,
        mana: 25.0,
        range: 26.0,
        power: 40.0,
        effect: Effect::Ranged,
        icon: "star",
    },
    Ability {
        name: "Frostbolt",
        cooldown: 2.5,
        mana: 18.0,
        range: 26.0,
        power: 22.0,
        effect: Effect::Chill,
        icon: "diamond",
    },
    Ability {
        name: "Flash Heal",
        cooldown: 8.0,
        mana: 30.0,
        range: 0.0,
        power: 50.0,
        effect: Effect::Heal,
        icon: "plus",
    },
    Ability {
        name: "Whirlwind",
        cooldown: 10.0,
        mana: 30.0,
        range: 7.0,
        power: 24.0,
        effect: Effect::Cleave,
        icon: "return",
    },
    Ability {
        name: "Sprint",
        cooldown: 20.0,
        mana: 10.0,
        range: 0.0,
        power: 5.0,
        effect: Effect::Sprint,
        icon: "fastForward",
    },
    Ability {
        name: "Hamstring",
        cooldown: 6.0,
        mana: 12.0,
        range: 4.2,
        power: 14.0,
        effect: Effect::Chill,
        icon: "cross",
    },
    Ability {
        name: "Second Wind",
        cooldown: 12.0,
        mana: 25.0,
        range: 0.0,
        power: 40.0,
        effect: Effect::Heal,
        icon: "plus",
    },
    Ability {
        name: "Shield Bash",
        cooldown: 7.0,
        mana: 15.0,
        range: 4.2,
        power: 24.0,
        effect: Effect::Melee,
        icon: "locked",
    },
    Ability {
        name: "Charge",
        cooldown: 12.0,
        mana: 10.0,
        range: 0.0,
        power: 3.0,
        effect: Effect::Sprint,
        icon: "fastForward",
    },
    Ability {
        name: "Arcane Blast",
        cooldown: 3.0,
        mana: 20.0,
        range: 26.0,
        power: 28.0,
        effect: Effect::Ranged,
        icon: "power",
    },
    Ability {
        name: "Frost Nova",
        cooldown: 12.0,
        mana: 28.0,
        range: 6.0,
        power: 14.0,
        effect: Effect::Cleave,
        icon: "return",
    },
    Ability {
        name: "Ice Barrier",
        cooldown: 14.0,
        mana: 30.0,
        range: 0.0,
        power: 45.0,
        effect: Effect::Heal,
        icon: "locked",
    },
    Ability {
        name: "Blink",
        cooldown: 10.0,
        mana: 15.0,
        range: 0.0,
        power: 2.0,
        effect: Effect::Sprint,
        icon: "fastForward",
    },
    Ability {
        name: "Smite",
        cooldown: 2.5,
        mana: 16.0,
        range: 26.0,
        power: 24.0,
        effect: Effect::Ranged,
        icon: "star",
    },
    Ability {
        name: "Holy Fire",
        cooldown: 8.0,
        mana: 28.0,
        range: 26.0,
        power: 42.0,
        effect: Effect::Ranged,
        icon: "exclamation",
    },
    Ability {
        name: "Renew",
        cooldown: 4.0,
        mana: 18.0,
        range: 0.0,
        power: 24.0,
        effect: Effect::Heal,
        icon: "checkmark",
    },
    Ability {
        name: "Holy Nova",
        cooldown: 12.0,
        mana: 30.0,
        range: 7.0,
        power: 16.0,
        effect: Effect::Cleave,
        icon: "return",
    },
    Ability {
        name: "Fade",
        cooldown: 14.0,
        mana: 12.0,
        range: 0.0,
        power: 3.0,
        effect: Effect::Sprint,
        icon: "fastForward",
    },
    Ability {
        name: "Sinister",
        cooldown: 2.0,
        mana: 12.0,
        range: 4.2,
        power: 22.0,
        effect: Effect::Melee,
        icon: "fightFist",
    },
    Ability {
        name: "Backstab",
        cooldown: 8.0,
        mana: 20.0,
        range: 4.2,
        power: 48.0,
        effect: Effect::Melee,
        icon: "cross",
    },
    Ability {
        name: "Eviscerate",
        cooldown: 10.0,
        mana: 22.0,
        range: 4.2,
        power: 40.0,
        effect: Effect::Melee,
        icon: "warning",
    },
    Ability {
        name: "Cripple",
        cooldown: 6.0,
        mana: 14.0,
        range: 4.2,
        power: 12.0,
        effect: Effect::Chill,
        icon: "diamond",
    },
    Ability {
        name: "Fan of Knives",
        cooldown: 9.0,
        mana: 26.0,
        range: 7.0,
        power: 20.0,
        effect: Effect::Cleave,
        icon: "return",
    },
    Ability {
        name: "Aimed Shot",
        cooldown: 8.0,
        mana: 26.0,
        range: 26.0,
        power: 46.0,
        effect: Effect::Ranged,
        icon: "target",
    },
    Ability {
        name: "Arcane Shot",
        cooldown: 3.0,
        mana: 16.0,
        range: 26.0,
        power: 24.0,
        effect: Effect::Ranged,
        icon: "star",
    },
    Ability {
        name: "Concussion",
        cooldown: 6.0,
        mana: 16.0,
        range: 26.0,
        power: 16.0,
        effect: Effect::Chill,
        icon: "diamond",
    },
    Ability {
        name: "Multi-Shot",
        cooldown: 9.0,
        mana: 28.0,
        range: 12.0,
        power: 18.0,
        effect: Effect::Cleave,
        icon: "return",
    },
    Ability {
        name: "Mend",
        cooldown: 8.0,
        mana: 26.0,
        range: 0.0,
        power: 36.0,
        effect: Effect::Heal,
        icon: "plus",
    },
    Ability {
        name: "Cheetah",
        cooldown: 14.0,
        mana: 12.0,
        range: 0.0,
        power: 4.0,
        effect: Effect::Sprint,
        icon: "fastForward",
    },
];

pub struct Calling {
    pub name: &'static str,
    pub blurb: &'static str,
    pub model: &'static str,
    pub health: f32,
    pub mana: f32,
    pub strength: f32,
    pub speed: f32,
    pub accent: Vec4,
    pub frame: usize,
    pub abilities: [usize; ABILITY_COUNT],
}

pub const CALLING_COUNT: usize = 5;

pub const CALLINGS: [Calling; CALLING_COUNT] = [
    Calling {
        name: "Warrior",
        blurb: "Steel and stubbornness. Hits hard and stands longer.",
        model: "character-male-b",
        health: 160.0,
        mana: 60.0,
        strength: 14.0,
        speed: 6.6,
        accent: Vec4::new(0.88, 0.48, 0.32, 0.95),
        frame: 0,
        abilities: [0, 4, 6, 7, 8, 9],
    },
    Calling {
        name: "Mage",
        blurb: "Fire and frost from a distance. Fragile and furious.",
        model: "character-female-a",
        health: 95.0,
        mana: 150.0,
        strength: 8.0,
        speed: 6.8,
        accent: Vec4::new(0.50, 0.68, 1.00, 0.95),
        frame: 1,
        abilities: [1, 2, 10, 11, 12, 13],
    },
    Calling {
        name: "Priest",
        blurb: "Light that heals and light that burns.",
        model: "character-female-b",
        health: 105.0,
        mana: 140.0,
        strength: 9.0,
        speed: 6.8,
        accent: Vec4::new(0.96, 0.92, 0.72, 0.95),
        frame: 2,
        abilities: [14, 15, 3, 16, 17, 18],
    },
    Calling {
        name: "Rogue",
        blurb: "Quick blades and quicker feet. In, and out again.",
        model: "character-male-d",
        health: 115.0,
        mana: 100.0,
        strength: 12.0,
        speed: 7.4,
        accent: Vec4::new(0.88, 0.82, 0.38, 0.95),
        frame: 3,
        abilities: [19, 20, 21, 22, 23, 5],
    },
    Calling {
        name: "Hunter",
        blurb: "Arrows from the tree line. Nothing gets close.",
        model: "character-female-c",
        health: 120.0,
        mana: 110.0,
        strength: 11.0,
        speed: 7.0,
        accent: Vec4::new(0.56, 0.86, 0.48, 0.95),
        frame: 4,
        abilities: [24, 25, 26, 27, 28, 29],
    },
];

pub struct Sheet {
    pub title: String,
    pub blurb: String,
    pub stats: [String; 4],
    pub abilities: [String; ABILITY_COUNT],
}

pub const STAGE_SPOT: Vec2 = Vec2::new(-10.0, 6.0);
pub const STAGE_DISTANCE: f32 = 5.5;
pub const STAGE_PITCH: f32 = 0.12;
pub const MENU_WIDTH: f32 = 520.0;
pub const MENU_HEIGHT: f32 = 760.0;
pub const MENU_LEFT: f32 = 3.0;
pub const MENU_STYLE: MenuStyle = MenuStyle {
    background: Vec4::new(0.06, 0.05, 0.04, 0.93),
    border: Vec4::new(0.90, 0.78, 0.50, 0.65),
    corner_radius: 10.0,
    border_width: 1.5,
    padding: 20.0,
    spacing: 8.0,
    visible: false,
    heading_size: 36.0,
    heading_color: Vec4::new(0.95, 0.85, 0.58, 1.0),
    centered: true,
    row_size: 16.0,
    row_color: Vec4::new(0.86, 0.83, 0.76, 1.0),
    outline: Vec4::new(0.0, 0.0, 0.0, 0.0),
    outline_width: 0.0,
};
pub const FRAME_IMAGES: [&str; CALLING_COUNT] = [
    "panel-border-000",
    "panel-border-004",
    "panel-border-010",
    "panel-border-018",
    "panel-border-022",
];

pub const NORMAL: u8 = 0;
pub const ELITE: u8 = 1;

pub struct Spawn {
    pub name: &'static str,
    pub level: u32,
    pub health: f32,
    pub strength: f32,
    pub speed: f32,
    pub aggro: f32,
    pub leash: f32,
    pub experience: u32,
    pub coins: u32,
    pub color: [f32; 4],
    pub scale: f32,
    pub biped: bool,
    pub rank: u8,
    pub respawn: f32,
    pub count: usize,
    pub place: usize,
    pub model: Option<&'static str>,
    pub model_scale: f32,
}

pub const SPAWNS: [Spawn; 17] = [
    Spawn {
        name: "Boar",
        level: 2,
        health: 45.0,
        strength: 5.0,
        speed: 4.2,
        aggro: 0.0,
        leash: 30.0,
        experience: 20,
        coins: 18,
        color: [0.36, 0.24, 0.16, 1.0],
        scale: 1.0,
        biped: false,
        rank: NORMAL,
        respawn: 10.0,
        count: 22,
        place: FIELD,
        model: None,
        model_scale: 1.0,
    },
    Spawn {
        name: "Old Tusk",
        level: 6,
        health: 420.0,
        strength: 14.0,
        speed: 4.6,
        aggro: 6.0,
        leash: 34.0,
        experience: 220,
        coins: 400,
        color: [0.22, 0.14, 0.10, 1.0],
        scale: 1.9,
        biped: false,
        rank: ELITE,
        respawn: 240.0,
        count: 1,
        place: FIELD,
        model: None,
        model_scale: 1.0,
    },
    Spawn {
        name: "Grey Wolf",
        level: 5,
        health: 70.0,
        strength: 9.0,
        speed: 6.0,
        aggro: 12.0,
        leash: 36.0,
        experience: 40,
        coins: 35,
        color: [0.46, 0.46, 0.48, 1.0],
        scale: 0.9,
        biped: false,
        rank: NORMAL,
        respawn: 14.0,
        count: 16,
        place: WOOD,
        model: None,
        model_scale: 1.0,
    },
    Spawn {
        name: "Toll Bandit",
        level: 8,
        health: 120.0,
        strength: 13.0,
        speed: 5.5,
        aggro: 11.0,
        leash: 24.0,
        experience: 65,
        coins: 90,
        color: [0.55, 0.16, 0.14, 1.0],
        scale: 1.0,
        biped: true,
        rank: NORMAL,
        respawn: 18.0,
        count: 10,
        place: FORT,
        model: Some("character-orc"),
        model_scale: 2.4,
    },
    Spawn {
        name: "Barrow Skeleton",
        level: 11,
        health: 170.0,
        strength: 17.0,
        speed: 5.0,
        aggro: 10.0,
        leash: 26.0,
        experience: 95,
        coins: 120,
        color: [0.86, 0.84, 0.76, 1.0],
        scale: 1.05,
        biped: true,
        rank: NORMAL,
        respawn: 20.0,
        count: 10,
        place: BARROW,
        model: Some("character-skeleton"),
        model_scale: 3.2,
    },
    Spawn {
        name: "Grave Shambler",
        level: 9,
        health: 150.0,
        strength: 15.0,
        speed: 3.4,
        aggro: 9.0,
        leash: 24.0,
        experience: 80,
        coins: 95,
        color: [0.40, 0.55, 0.35, 1.0],
        scale: 1.0,
        biped: true,
        rank: NORMAL,
        respawn: 22.0,
        count: 8,
        place: BARROW,
        model: Some("character-zombie"),
        model_scale: 3.1,
    },
    Spawn {
        name: "Barrow Wisp",
        level: 12,
        health: 110.0,
        strength: 20.0,
        speed: 6.4,
        aggro: 13.0,
        leash: 28.0,
        experience: 110,
        coins: 140,
        color: [0.75, 0.85, 1.0, 1.0],
        scale: 1.0,
        biped: true,
        rank: NORMAL,
        respawn: 26.0,
        count: 6,
        place: BARROW,
        model: Some("character-ghost"),
        model_scale: 2.8,
    },
    Spawn {
        name: "The Gravelord",
        level: 15,
        health: 1200.0,
        strength: 30.0,
        speed: 4.8,
        aggro: 9.0,
        leash: 18.0,
        experience: 700,
        coins: 1500,
        color: [0.18, 0.16, 0.20, 1.0],
        scale: 1.7,
        biped: true,
        rank: ELITE,
        respawn: 300.0,
        count: 1,
        place: CRYPT,
        model: Some("character-vampire"),
        model_scale: 3.6,
    },
    Spawn {
        name: "Bandit Marksman",
        level: 9,
        health: 130.0,
        strength: 14.0,
        speed: 5.6,
        aggro: 12.0,
        leash: 26.0,
        experience: 75,
        coins: 100,
        color: [0.50, 0.30, 0.18, 1.0],
        scale: 1.0,
        biped: true,
        rank: NORMAL,
        respawn: 20.0,
        count: 8,
        place: FORT,
        model: Some("blocky-character-b"),
        model_scale: 0.25,
    },
    Spawn {
        name: "Red Harrow",
        level: 11,
        health: 900.0,
        strength: 26.0,
        speed: 5.2,
        aggro: 10.0,
        leash: 18.0,
        experience: 520,
        coins: 6000,
        color: [0.70, 0.12, 0.10, 1.0],
        scale: 1.4,
        biped: true,
        rank: ELITE,
        respawn: 300.0,
        count: 1,
        place: FORT,
        model: Some("character-human"),
        model_scale: 3.2,
    },
    Spawn {
        name: "Bone Warden",
        level: 13,
        health: 210.0,
        strength: 20.0,
        speed: 5.0,
        aggro: 10.0,
        leash: 20.0,
        experience: 120,
        coins: 150,
        color: [0.80, 0.78, 0.70, 1.0],
        scale: 1.1,
        biped: true,
        rank: NORMAL,
        respawn: 24.0,
        count: 8,
        place: CRYPT,
        model: Some("character-skeleton"),
        model_scale: 3.4,
    },
    Spawn {
        name: "Crypt Fiend",
        level: 14,
        health: 160.0,
        strength: 24.0,
        speed: 6.6,
        aggro: 12.0,
        leash: 20.0,
        experience: 135,
        coins: 170,
        color: [0.60, 0.70, 0.95, 1.0],
        scale: 1.0,
        biped: true,
        rank: NORMAL,
        respawn: 28.0,
        count: 4,
        place: CRYPT,
        model: Some("character-ghost"),
        model_scale: 3.0,
    },
    Spawn {
        name: "Kobold Digger",
        level: 10,
        health: 140.0,
        strength: 15.0,
        speed: 5.4,
        aggro: 11.0,
        leash: 24.0,
        experience: 85,
        coins: 110,
        color: [0.60, 0.42, 0.22, 1.0],
        scale: 1.0,
        biped: true,
        rank: NORMAL,
        respawn: 20.0,
        count: 10,
        place: MINE,
        model: Some("blocky-character-h"),
        model_scale: 0.24,
    },
    Spawn {
        name: "Kobold Slinger",
        level: 11,
        health: 120.0,
        strength: 18.0,
        speed: 5.8,
        aggro: 13.0,
        leash: 24.0,
        experience: 95,
        coins: 125,
        color: [0.55, 0.50, 0.25, 1.0],
        scale: 1.0,
        biped: true,
        rank: NORMAL,
        respawn: 22.0,
        count: 6,
        place: MINE,
        model: Some("blocky-character-j"),
        model_scale: 0.24,
    },
    Spawn {
        name: "Foreman Grask",
        level: 16,
        health: 1400.0,
        strength: 32.0,
        speed: 4.6,
        aggro: 9.0,
        leash: 16.0,
        experience: 800,
        coins: 9000,
        color: [0.30, 0.45, 0.20, 1.0],
        scale: 1.6,
        biped: true,
        rank: ELITE,
        respawn: 320.0,
        count: 1,
        place: DEEP,
        model: Some("character-orc"),
        model_scale: 3.6,
    },
    Spawn {
        name: "Young Wolf",
        level: 1,
        health: 30.0,
        strength: 3.0,
        speed: 5.4,
        aggro: 5.0,
        leash: 30.0,
        experience: 14,
        coins: 8,
        color: [0.55, 0.52, 0.50, 1.0],
        scale: 0.75,
        biped: false,
        rank: NORMAL,
        respawn: 12.0,
        count: 16,
        place: STARTER,
        model: None,
        model_scale: 1.0,
    },
    Spawn {
        name: "Kobold Vermin",
        level: 2,
        health: 40.0,
        strength: 4.0,
        speed: 4.8,
        aggro: 5.5,
        leash: 26.0,
        experience: 18,
        coins: 12,
        color: [0.62, 0.48, 0.30, 1.0],
        scale: 1.0,
        biped: true,
        rank: NORMAL,
        respawn: 14.0,
        count: 12,
        place: STARTER,
        model: Some("blocky-character-c"),
        model_scale: 0.22,
    },
];

pub struct HuntData {
    pub prey: &'static str,
    pub place: usize,
    pub giver: usize,
    pub min_level: u32,
}

pub const HUNTS: [HuntData; 11] = [
    HuntData {
        prey: "Young Wolf",
        place: STARTER,
        giver: 12,
        min_level: 0,
    },
    HuntData {
        prey: "Boar",
        place: FIELD,
        giver: 0,
        min_level: 3,
    },
    HuntData {
        prey: "Grey Wolf",
        place: WOOD,
        giver: 1,
        min_level: 5,
    },
    HuntData {
        prey: "Toll Bandit",
        place: FORT,
        giver: 2,
        min_level: 9,
    },
    HuntData {
        prey: "Bandit Marksman",
        place: FORT,
        giver: 2,
        min_level: 10,
    },
    HuntData {
        prey: "Grave Shambler",
        place: BARROW,
        giver: 3,
        min_level: 11,
    },
    HuntData {
        prey: "Kobold Digger",
        place: MINE,
        giver: 11,
        min_level: 12,
    },
    HuntData {
        prey: "Barrow Skeleton",
        place: BARROW,
        giver: 3,
        min_level: 13,
    },
    HuntData {
        prey: "Bone Warden",
        place: CRYPT,
        giver: 3,
        min_level: 15,
    },
    HuntData {
        prey: "Barrow Wisp",
        place: BARROW,
        giver: 3,
        min_level: 16,
    },
    HuntData {
        prey: "Kobold Slinger",
        place: MINE,
        giver: 11,
        min_level: 18,
    },
];

pub struct GiverData {
    pub name: &'static str,
    pub title: &'static str,
    pub spot: Vec2,
    pub board: bool,
    pub model: &'static str,
    pub model_scale: f32,
}

pub const GIVERS: [GiverData; 13] = [
    GiverData {
        name: "Notice Board",
        title: "Thornmere",
        spot: BOARD_SPOT,
        board: true,
        model: "signpost",
        model_scale: 4.2,
    },
    GiverData {
        name: "Marren",
        title: "Hunter",
        spot: Vec2::new(-46.0, -4.0),
        board: false,
        model: "character-female-c",
        model_scale: 2.6,
    },
    GiverData {
        name: "Captain Idris",
        title: "City Watch",
        spot: Vec2::new(-6.0, -74.0),
        board: false,
        model: "character-soldier",
        model_scale: 2.8,
    },
    GiverData {
        name: "Sister Ael",
        title: "Priestess of the Light",
        spot: Vec2::new(-42.0, -112.4),
        board: false,
        model: "character-female-a",
        model_scale: 2.6,
    },
    GiverData {
        name: "Wynne",
        title: "Apothecary",
        spot: Vec2::new(-38.0, -84.0),
        board: false,
        model: "character-female-d",
        model_scale: 2.6,
    },
    GiverData {
        name: "Bram",
        title: "Innkeeper",
        spot: Vec2::new(40.0, -86.0),
        board: false,
        model: "character-male-d",
        model_scale: 2.6,
    },
    GiverData {
        name: "Holt",
        title: "Farmer",
        spot: Vec2::new(88.0, -96.0),
        board: false,
        model: "character-digger",
        model_scale: 2.9,
    },
    GiverData {
        name: "Fenn",
        title: "Auctioneer",
        spot: Vec2::new(12.4, -78.0),
        board: false,
        model: "character-male-b",
        model_scale: 2.6,
    },
    GiverData {
        name: "Orla",
        title: "Banker",
        spot: Vec2::new(-12.4, -78.0),
        board: false,
        model: "character-female-b",
        model_scale: 2.6,
    },
    GiverData {
        name: "Garron",
        title: "Blacksmith",
        spot: Vec2::new(24.0, -114.6),
        board: false,
        model: "character-male-e",
        model_scale: 2.6,
    },
    GiverData {
        name: "Pell",
        title: "Dockmaster",
        spot: Vec2::new(-58.0, 76.0),
        board: false,
        model: "character-male-f",
        model_scale: 2.6,
    },
    GiverData {
        name: "Mabb",
        title: "Prospector",
        spot: Vec2::new(100.0, 103.0),
        board: false,
        model: "character-male-c",
        model_scale: 2.6,
    },
    GiverData {
        name: "Rosalind",
        title: "Fallowmere Inn",
        spot: Vec2::new(8.6, 14.0),
        board: false,
        model: "character-female-e",
        model_scale: 2.6,
    },
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Objective {
    Kill { prey: &'static str, count: u32 },
    Gather { pickup: usize, count: u32 },
    Deliver { giver: usize },
    Discover { place: usize },
}

pub struct Quest {
    pub title: &'static str,
    pub story: &'static str,
    pub giver: usize,
    pub objective: Objective,
    pub experience: u32,
    pub coins: u32,
    pub repeatable: bool,
    pub accept: &'static [&'static str],
    pub complete: &'static [&'static str],
}

pub const QUESTS: [Quest; 25] = [
    Quest {
        title: "Boar Trouble",
        story: "The field feeds the city. The boars are eating the field.",
        giver: 0,
        objective: Objective::Kill {
            prey: "Boar",
            count: 6,
        },
        experience: 70,
        coins: 350,
        repeatable: true,
        accept: &[
            "The boars in the field beyond the moat are in the crops again. Six boars, one bounty, no questions.",
            "Bring nothing back. The field will know.",
        ],
        complete: &[
            "The field is quieter. The bounty is paid. The boars will be back by morning, and so will the bounty.",
        ],
    },
    Quest {
        title: "Teeth in the Wood",
        story: "Marren has lost three sheep to a pack grown bold.",
        giver: 1,
        objective: Objective::Kill {
            prey: "Grey Wolf",
            count: 5,
        },
        experience: 160,
        coins: 900,
        repeatable: false,
        accept: &[
            "Three sheep this week. Three. The pack has been bold since the old alpha died.",
            "Thin them out for me. Five should send the rest deeper into the wood.",
            "Watch the stream crossing. They hunt in pairs there.",
        ],
        complete: &[
            "Five pelts and not a scratch on you? You will do.",
            "Here. This is what the city pays for wolf work. It is not enough. It never is.",
        ],
    },
    Quest {
        title: "The Toll Road",
        story: "Bandits from Redharrow Fort tax the east road as if they owned it.",
        giver: 2,
        objective: Objective::Kill {
            prey: "Toll Bandit",
            count: 4,
        },
        experience: 240,
        coins: 1400,
        repeatable: false,
        accept: &[
            "Bandits out of Redharrow Fort are charging a toll on the east road. My road.",
            "I have twenty guards and half of them are on the walls. Go and remind the fort whose road it is.",
            "Four of them should carry the message to the rest.",
        ],
        complete: &[
            "Four fewer thieves. The carts will roll again by morning.",
            "Take this. And do not tell the mayor how much it was.",
        ],
    },
    Quest {
        title: "What Sleeps in the Barrow",
        story: "The dead of the barrow walk, and something under the hill wakes them.",
        giver: 3,
        objective: Objective::Kill {
            prey: "Barrow Skeleton",
            count: 6,
        },
        experience: 360,
        coins: 2000,
        repeatable: false,
        accept: &[
            "The dead in the barrow do not rest. Something wakes them and sets them walking.",
            "Put six of them back in the ground. Gently, if you can manage it.",
            "And do not read anything carved on the stones. I mean that.",
        ],
        complete: &[
            "You smell of the barrow. Good. That means you were truly there.",
            "The Light keeps its own accounts. Here is the city's part.",
        ],
    },
    Quest {
        title: "Bitter Roots",
        story: "Wynne needs bitterroot for a salve, and it grows only where the wolves walk.",
        giver: 4,
        objective: Objective::Gather {
            pickup: 0,
            count: 5,
        },
        experience: 130,
        coins: 600,
        repeatable: false,
        accept: &[
            "Bitterroot only grows where the wolves walk, which is the whole wood.",
            "Five bundles, roots and all. Mind the leaves, they stain everything they touch.",
        ],
        complete: &[
            "Fresh. Mostly. These will do for the salve.",
            "If your hands itch tonight, that is the bitterroot. It passes. Usually.",
        ],
    },
    Quest {
        title: "A Letter Downhill",
        story: "Bram wants his money from Holt and would rather not ask in person.",
        giver: 5,
        objective: Objective::Deliver { giver: 6 },
        experience: 100,
        coins: 400,
        repeatable: false,
        accept: &[
            "Holt owes me for two barrels and a bed. This letter reminds him, politely.",
            "He is at the farm east of the moat. Do not read it. It is not that polite.",
        ],
        complete: &[
            "Bram sent you? Tell him the barrels were half water and the bed had fleas.",
            "Here. That is for the walk, not for Bram.",
        ],
    },
    Quest {
        title: "Old Tusk",
        story: "One boar has gored twelve hunters. The board has stopped counting.",
        giver: 0,
        objective: Objective::Kill {
            prey: "Old Tusk",
            count: 1,
        },
        experience: 450,
        coins: 5000,
        repeatable: false,
        accept: &[
            "A bounty on Old Tusk, the boar that gores the field and the hunters in it.",
            "Twelve hunters have tried. The board does not say how many came back.",
        ],
        complete: &["The bounty is yours. The field is quiet for the first time in a year."],
    },
    Quest {
        title: "The Gravelord",
        story: "Whatever rules the Sunken Crypt sends the dead walking closer each night.",
        giver: 3,
        objective: Objective::Kill {
            prey: "The Gravelord",
            count: 1,
        },
        experience: 900,
        coins: 12000,
        repeatable: false,
        accept: &[
            "Something rules the crypt under the barrow. The dead call it the Gravelord, when they call it anything.",
            "I would not send you. But nobody else will go down those stairs, and the dead are walking closer each night.",
        ],
        complete: &[
            "It is done? Then the Light is kinder than I feared.",
            "Rest now. You have earned more than coin, though coin is what I have.",
        ],
    },
    Quest {
        title: "Lights on the Barrow Road",
        story: "Travelers follow lights on the barrow road and do not come back.",
        giver: 3,
        objective: Objective::Kill {
            prey: "Barrow Wisp",
            count: 4,
        },
        experience: 320,
        coins: 1800,
        repeatable: false,
        accept: &[
            "Travelers see lights on the barrow road at night and follow them. They do not come back.",
            "The lights are the dead who forgot their graves. Four of them, and the road is dark again.",
        ],
        complete: &[
            "Four lights out. The road will be dark tonight, and dark is safer than what was there.",
        ],
    },
    Quest {
        title: "Redharrow's Toll",
        story: "Marksmen on the fort wall pick off carters before they reach the gate.",
        giver: 2,
        objective: Objective::Kill {
            prey: "Bandit Marksman",
            count: 6,
        },
        experience: 300,
        coins: 1600,
        repeatable: false,
        accept: &[
            "The fort has marksmen on the palisade now. Carters get an arrow before they get a word.",
            "Six of them down and the wall is a wall again. Go in fast, they do not miss twice.",
        ],
        complete: &[
            "Six. The carters will breathe on the east road again.",
            "The Watch pays for arrows it did not have to catch.",
        ],
    },
    Quest {
        title: "The Red Harrow",
        story: "Every bandit on the east road answers to one woman behind the palisade.",
        giver: 2,
        objective: Objective::Kill {
            prey: "Red Harrow",
            count: 1,
        },
        experience: 700,
        coins: 8000,
        repeatable: false,
        accept: &[
            "They call her the Red Harrow, and every thief on the east road answers to her.",
            "She sits in the middle of that fort like a spider. Cut the web and the flies scatter.",
        ],
        complete: &[
            "The Harrow is down? Then the east road is the city's again.",
            "This is from the treasury. The rest is from me.",
        ],
    },
    Quest {
        title: "Wardens of the Deep",
        story: "Armored dead guard the crypt stairs so that nothing living reaches the Gravelord.",
        giver: 3,
        objective: Objective::Kill {
            prey: "Bone Warden",
            count: 6,
        },
        experience: 420,
        coins: 2400,
        repeatable: false,
        accept: &[
            "The crypt has wardens now. Old soldiers in old iron, and they do not sleep.",
            "Six of them stand between the stairs and the throne. Clear the stairs and the Light can follow you down.",
        ],
        complete: &[
            "The wardens are broken. The stairs are open.",
            "Whatever waits below has fewer friends tonight.",
        ],
    },
    Quest {
        title: "The Crypt Door",
        story: "Nobody has mapped the stairs under the barrow and lived to draw them.",
        giver: 3,
        objective: Objective::Discover { place: CRYPT },
        experience: 180,
        coins: 800,
        repeatable: false,
        accept: &[
            "There is a door under the barrow. Sunken steps, a gate, and a cold that does not belong to the weather.",
            "Find it. Look at it. Come back and tell me it is real, so I stop hoping it is not.",
        ],
        complete: &[
            "So it is real. I had prayed the diggers were drunk.",
            "Thank you. That is the worst news I have been glad to hear.",
        ],
    },
    Quest {
        title: "Ironjaw Diggers",
        story: "Kobolds took the mine in a night, and the city's iron with it.",
        giver: 11,
        objective: Objective::Kill {
            prey: "Kobold Digger",
            count: 8,
        },
        experience: 380,
        coins: 2100,
        repeatable: false,
        accept: &[
            "Kobolds came up through the deep shaft one night and the mine was theirs by morning.",
            "Eight diggers dead and the rest will lose their nerve. They are brave in a crowd and nowhere else.",
        ],
        complete: &[
            "Eight. Good. The picks can go back to people who know which end to hold.",
            "The company pays. Not well, but it pays.",
        ],
    },
    Quest {
        title: "The Foreman",
        story: "An orc calling himself Foreman Grask runs the kobolds from the bottom of the pit.",
        giver: 11,
        objective: Objective::Kill {
            prey: "Foreman Grask",
            count: 1,
        },
        experience: 850,
        coins: 10000,
        repeatable: false,
        accept: &[
            "The kobolds have a foreman. An orc, big as a cart, who calls the deep shaft his.",
            "He sits at the bottom of the pit with the best ore and the worst temper. Go and fire him.",
        ],
        complete: &[
            "Grask is dead? Then Ironjaw is a mine again and not a warren.",
            "Here. Buy something that is not a bandage.",
        ],
    },
    Quest {
        title: "Slingers in the Shaft",
        story: "Kobold slingers on the pit ledges stone anyone who climbs down.",
        giver: 11,
        objective: Objective::Kill {
            prey: "Kobold Slinger",
            count: 5,
        },
        experience: 340,
        coins: 1900,
        repeatable: false,
        accept: &[
            "The slingers sit on the ledges and stone anyone who climbs down. Five of them, by my count.",
            "Knock them off the ledges and the miners can work the upper seam again.",
        ],
        complete: &["Five slingers down. The upper seam is quiet. I could kiss you. I will not."],
    },
    Quest {
        title: "The Morning Catch",
        story: "The harbor nets came in empty and the inn expects fish by noon.",
        giver: 10,
        objective: Objective::Gather {
            pickup: 1,
            count: 5,
        },
        experience: 150,
        coins: 700,
        repeatable: true,
        accept: &[
            "The nets came in empty and Bram expects trout by noon.",
            "The trout beach themselves in the reeds along the shore. Five will fill a basket.",
        ],
        complete: &["Five trout. Bram will complain about the size, but he will pay."],
    },
    Quest {
        title: "Fish for the Inn",
        story: "The harbor's catch has to reach the inn while it is still a catch.",
        giver: 10,
        objective: Objective::Deliver { giver: 5 },
        experience: 120,
        coins: 500,
        repeatable: false,
        accept: &[
            "Take this basket to Bram at the inn. Trade Square, east side, the one with the barrels.",
            "Walk fast. Fish do not improve with the road.",
        ],
        complete: &[
            "From Pell? Smell that. That is a fresh trout, which means Pell finally hired a deckhand.",
            "Here. For the walk, and for not eating one.",
        ],
    },
    Quest {
        title: "Lot Forty-One",
        story: "The auction house sold a lot it should not have, and the bank must hear it first.",
        giver: 7,
        objective: Objective::Deliver { giver: 8 },
        experience: 110,
        coins: 450,
        repeatable: false,
        accept: &[
            "Lot forty-one sold this morning. It should not have. The seller did not own it.",
            "These papers go to Orla at the bank, across the square. Before the buyer gets there.",
        ],
        complete: &[
            "From Fenn? Let me see. Ah. Lot forty-one. Yes, that is going to be a long afternoon.",
            "Here. The bank pays its couriers, even the ones it did not hire.",
        ],
    },
    Quest {
        title: "Coin for the Watch",
        story: "The Watch payroll crosses the square once a month, and once a month it needs a guard.",
        giver: 8,
        objective: Objective::Deliver { giver: 2 },
        experience: 110,
        coins: 450,
        repeatable: false,
        accept: &[
            "The Watch payroll. It crosses the square once a month and once a month I lose sleep over it.",
            "Captain Idris is by the notice board. Put this in her hand and nobody else's.",
        ],
        complete: &[
            "The payroll. Good. Twenty guards were about to become twenty problems.",
            "Take a share. Orla counts it, but I sign for it.",
        ],
    },
    Quest {
        title: "Iron from the Deep",
        story: "Garron's forge is cold until someone brings ore out of a mine full of kobolds.",
        giver: 9,
        objective: Objective::Gather {
            pickup: 2,
            count: 4,
        },
        experience: 260,
        coins: 1500,
        repeatable: true,
        accept: &[
            "The forge is cold. No ore comes down from Ironjaw since the kobolds took it.",
            "Four lumps of raw iron from the pit and I can arm the Watch again. Mind the slingers.",
        ],
        complete: &["Iron. Real iron. Stand back, this is going to be loud for a week."],
    },
    Quest {
        title: "Where the Iron Comes From",
        story: "A smith should see the mine his iron comes from, or at least send someone who can run.",
        giver: 9,
        objective: Objective::Discover { place: MINE },
        experience: 160,
        coins: 700,
        repeatable: false,
        accept: &[
            "Ironjaw is up the northeast road past the barrow. I have never seen it. Smiths do not travel.",
            "Go and look at it for me. Tell me if the headframe still stands.",
        ],
        complete: &["Still standing? Then there is hope for the seam. And for my forge."],
    },
    Quest {
        title: "The Harbor Road",
        story: "The city wants to know the northwest road still reaches the harbor.",
        giver: 0,
        objective: Objective::Discover { place: HARBOR },
        experience: 120,
        coins: 500,
        repeatable: false,
        accept: &[
            "The council asks that a traveler walk the northwest road to Mirror Harbor and report that it is open.",
            "Pell the dockmaster will confirm your arrival. He will also try to sell you a boat.",
        ],
        complete: &[
            "The board notes the harbor road is open. The council is relieved, or claims to be.",
        ],
    },
    Quest {
        title: "Wolves at the Door",
        story: "The young wolves of Fallowmere Wood have started taking chickens from the inn yard.",
        giver: 12,
        objective: Objective::Kill {
            prey: "Young Wolf",
            count: 6,
        },
        experience: 60,
        coins: 250,
        repeatable: true,
        accept: &[
            "The wolves are pups still, but pups with teeth. Two chickens this week and a boot.",
            "Six of them thinned out and the rest will keep to the deep wood. Stay near the road at first.",
        ],
        complete: &[
            "Six. Good. The chickens can go back to being loud instead of missing.",
            "Here. It is inn coin, so do not spend it anywhere but here.",
        ],
    },
    Quest {
        title: "Candles in the Roots",
        story: "Kobold vermin dig under the old oaks at night and carry off anything that burns.",
        giver: 12,
        objective: Objective::Kill {
            prey: "Kobold Vermin",
            count: 5,
        },
        experience: 80,
        coins: 320,
        repeatable: false,
        accept: &[
            "Kobolds. Vermin, the little kind, with candles on their heads and no sense at all.",
            "They dig under the oaks west of the road and steal lamp oil. Five of them and the wood is quiet again.",
        ],
        complete: &[
            "You smell of tallow. Five candles out. The oaks will thank you when they can.",
        ],
    },
];

pub struct PickupKind {
    pub name: &'static str,
    pub color: [f32; 4],
    pub place: usize,
    pub count: usize,
    pub regrow: f32,
    pub model: &'static str,
    pub model_scale: f32,
}

pub const PICKUPS: [PickupKind; 3] = [
    PickupKind {
        name: "Bitterroot",
        color: [0.72, 0.85, 0.30, 1.0],
        place: WOOD,
        count: 14,
        regrow: 40.0,
        model: "crop_turnip",
        model_scale: 2.4,
    },
    PickupKind {
        name: "Lake Trout",
        color: [0.55, 0.80, 0.95, 1.0],
        place: HARBOR,
        count: 8,
        regrow: 35.0,
        model: "food-fish",
        model_scale: 2.6,
    },
    PickupKind {
        name: "Iron Ore",
        color: [0.80, 0.55, 0.35, 1.0],
        place: MINE,
        count: 10,
        regrow: 45.0,
        model: "resource-stone",
        model_scale: 2.4,
    },
];

pub const GRASS_COLOR: [f32; 4] = [0.33, 0.58, 0.20, 1.0];
pub const TUFT_COLOR: [f32; 4] = [0.40, 0.66, 0.24, 1.0];
pub const STONE_COLOR: [f32; 4] = [0.50, 0.50, 0.52, 1.0];
pub const WATER_COLOR: [f32; 4] = [0.18, 0.38, 0.55, 1.0];
pub const FIRE_COLOR: [f32; 4] = [1.0, 0.55, 0.15, 1.0];
pub const LANTERN_COLOR: [f32; 4] = [1.0, 0.80, 0.45, 1.0];
pub const FLOWER_COLORS: [[f32; 4]; 3] = [
    [0.95, 0.85, 0.30, 1.0],
    [0.90, 0.40, 0.55, 1.0],
    [0.85, 0.90, 0.95, 1.0],
];

pub struct ClassData {
    pub name: &'static str,
    pub plate: Vec4,
    pub role: &'static str,
}

pub const CLASSES: [ClassData; 6] = [
    ClassData {
        name: "Warrior",
        plate: Vec4::new(0.90, 0.62, 0.45, 1.0),
        role: "tank",
    },
    ClassData {
        name: "Mage",
        plate: Vec4::new(0.55, 0.78, 1.00, 1.0),
        role: "dps",
    },
    ClassData {
        name: "Hunter",
        plate: Vec4::new(0.70, 0.90, 0.45, 1.0),
        role: "dps",
    },
    ClassData {
        name: "Priest",
        plate: Vec4::new(1.00, 1.00, 0.95, 1.0),
        role: "healer",
    },
    ClassData {
        name: "Rogue",
        plate: Vec4::new(1.00, 0.95, 0.45, 1.0),
        role: "dps",
    },
    ClassData {
        name: "Druid",
        plate: Vec4::new(1.00, 0.60, 0.25, 1.0),
        role: "healer",
    },
];

pub const PLAYER_PLATE_COLOR: Vec4 = Vec4::new(0.55, 1.0, 0.65, 1.0);
pub const GIVER_PLATE_COLOR: Vec4 = Vec4::new(1.0, 0.92, 0.45, 1.0);
pub const MONSTER_PLATE_COLOR: Vec4 = Vec4::new(1.0, 0.45, 0.40, 1.0);
pub const ELITE_PLATE_COLOR: Vec4 = Vec4::new(1.0, 0.75, 0.30, 1.0);

pub const GUILDS: [&str; 10] = [
    "Thornwatch",
    "Emberkin",
    "Vale Runners",
    "Boar Hunters",
    "Nightbloom",
    "Grey Company",
    "Stormhollow",
    "Ashen Oath",
    "Lantern Guard",
    "Barrow Kings",
];

pub const NAME_STARTS: [&str; 30] = [
    "Ael", "Bran", "Cor", "Dun", "El", "Fen", "Gar", "Hal", "Isk", "Jor", "Kel", "Lor", "Mor",
    "Nim", "Or", "Pel", "Quen", "Ren", "Syl", "Tor", "Ul", "Var", "Wyn", "Zel", "Thal", "Ost",
    "Grim", "Vel", "Ash", "Ryn",
];

pub const NAME_ENDS: [&str; 14] = [
    "a", "ien", "dor", "is", "wen", "ric", "ar", "eth", "on", "ia", "us", "yn", "gar", "leth",
];

pub const VOICE_COUNT: usize = 5;

pub const SOUND_RATE: u32 = 32000;
pub const SOUND_FIREBALL: &str = "thornmere-fireball";
pub const SOUND_WHIRL: &str = "thornmere-whirl";
pub const SOUND_SPRINT: &str = "thornmere-sprint";
pub const SOUND_FROST: &str = "interface/glass_001";
pub const SOUND_HEAL: &str = "interface/confirmation_003";
pub const SOUND_LEVEL: &str = "jingles/jingles-pizzicato_16";
pub const SOUND_ACCEPT: &str = "rpg/bookFlip1";
pub const SOUND_DONE: &str = "jingles/jingles-pizzicato_05";
pub const SOUND_OBJECTIVE: &str = "interface/tick_001";
pub const SOUND_DISCOVER: &str = "jingles/jingles-pizzicato_10";
pub const SOUND_AGGRO: &str = "interface/bong_001";
pub const SOUND_DEATH: &str = "jingles/jingles-pizzicato_14";
pub const SOUND_DENY: &str = "interface/error_001";
pub const SOUND_WHISPER: &str = "interface/switch_001";
pub const SOUND_SLOT: &str = "ui/click1";
pub const SOUND_JUMP: &str = "rpg/cloth1";
pub const SOUND_ELITE_UP: &str = "rpg/doorOpen_1";
pub const SOUND_ZONE_BED: &str = "loops/Retro-Mystic";
pub const SOUND_NIGHT_BED: &str = "loops/Sad-Town";
pub const SOUND_BATTLE_BED: &str = "loops/Drumming-Sticks";
pub const SWING_CLIPS: [&str; 4] = [
    "swords/sword1",
    "swords/sword2",
    "swords/sword3",
    "swords/sword4",
];
pub const HIT_CLIPS: [&str; 3] = [
    "impact/impactPunch_heavy_000",
    "impact/impactPunch_heavy_001",
    "impact/impactPunch_heavy_002",
];
pub const CRIT_CLIPS: [&str; 3] = [
    "swords/swordMetal1",
    "swords/swordMetal2",
    "swords/swordMetal3",
];
pub const HURT_CLIPS: [&str; 2] = ["impact/impactSoft_heavy_000", "impact/impactSoft_heavy_001"];
pub const KILL_CLIPS: [&str; 3] = [
    "swords/hitHelmet1",
    "swords/hitHelmet2",
    "swords/hitHelmet3",
];
pub const COIN_CLIPS: [&str; 2] = ["rpg/handleCoins", "rpg/handleCoins2"];
pub const TARGET_CLIPS: [&str; 3] = ["rpg/drawKnife1", "rpg/drawKnife2", "rpg/drawKnife3"];
pub const GATHER_CLIPS: [&str; 3] = ["rpg/cloth1", "rpg/cloth2", "rpg/cloth3"];
pub const STEP_CLIPS: [&str; 5] = [
    "impact/footstep_grass_000",
    "impact/footstep_grass_001",
    "impact/footstep_grass_002",
    "impact/footstep_grass_003",
    "impact/footstep_grass_004",
];
pub const SOUND_FAINT: f32 = -20.0;
pub const SOUND_QUIET: f32 = -12.0;
pub const SOUND_PLAIN: f32 = -6.0;
pub const SOUND_LOUD: f32 = 0.0;
pub const MUSIC_LEVEL: f32 = -11.0;
pub const DRUM_LEVEL: f32 = -8.0;

pub const SHAKE_AMOUNT: f32 = 0.32;
pub const SHAKE_DECAY: f32 = 2.4;
pub const HIT_TRAUMA: f32 = 0.18;
pub const CRIT_TRAUMA: f32 = 0.42;
pub const HURT_TRAUMA: f32 = 0.30;
pub const KILL_TRAUMA: f32 = 0.55;
pub const ELITE_KILL_TRAUMA: f32 = 1.0;
pub const HURT_FLASH_SECONDS: f32 = 0.45;
pub const GREY_SECONDS: f32 = 4.0;
pub const LOW_HEALTH_FRACTION: f32 = 0.3;
pub const COMBAT_LINGER: f32 = 5.0;
pub const BANNER_SECONDS: f32 = 3.4;
pub const BANNER_FADE: f32 = 0.9;
pub const SLOT_FLASH_SECONDS: f32 = 0.22;
pub const PROJECTILE_SPEED: f32 = 26.0;
pub const PROJECTILE_HEIGHT: f32 = 1.2;
pub const STRIKE_SECONDS: f32 = 0.7;
pub const FLINCH_SECONDS: f32 = 0.28;
pub const DYING_SECONDS: f32 = 0.9;
pub const RISING_SECONDS: f32 = 0.7;
pub const WALK_SWING: f32 = 0.6;
pub const WALK_CYCLE: f32 = 4.2;
pub const STEP_STRIDE: f32 = 1.9;
pub const MINIMAP_SIZE: f32 = 176.0;
pub const MINIMAP_RANGE: f32 = 55.0;
pub const MINIMAP_DOTS: usize = 56;
pub const MINIMAP_STEP: f32 = 0.1;
pub const FLOATING_POP: f32 = 1.5;
pub const CRIT_POP: f32 = 2.3;

pub const FROST_COLOR: [f32; 4] = [0.55, 0.80, 1.0, 1.0];

pub const MODEL_FACING: f32 = std::f32::consts::PI;
pub const FOLK_SCALE: f32 = 2.6;
pub const HOUSE_CELL: f32 = 4.0;
pub const TREE_SCALE: f32 = 3.6;
pub const CLASS_MODELS: [(&str, &str); 6] = [
    ("character-male-b", "character-female-b"),
    ("character-male-c", "character-female-d"),
    ("character-male-e", "character-female-c"),
    ("character-male-f", "character-female-a"),
    ("character-male-a", "character-female-e"),
    ("character-male-d", "character-female-f"),
];
pub const RING_MODEL: &str = "selection-a";
pub const CLIP_NAMES: [&str; 8] = [
    "idle",
    "walk",
    "sprint",
    "attack-melee-right",
    "die",
    "sit",
    "pick-up",
    "static",
];
pub const CLIP_IDLE: usize = 0;
pub const CLIP_WALK: usize = 1;
pub const CLIP_SPRINT: usize = 2;
pub const CLIP_ATTACK: usize = 3;
pub const CLIP_DIE: usize = 4;
pub const CLIP_SIT: usize = 5;
pub const CLIP_STATIC: usize = 7;
pub const ICON_CELL: u32 = 128;
pub const FRAME_BANDS: f32 = 3.0;
pub const FRAME_CORNER: f32 = 18.0;
pub const ICON_ATLAS: u32 = 1024;
pub const BUSH_COUNT: usize = 140;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Shape {
    #[default]
    Blast,
    Shard,
    Slash,
    Whirl,
    Halo,
    Patch,
    Lamp,
}

pub const HOLY_COLOR: [f32; 4] = [1.0, 0.88, 0.45, 1.0];
pub const LIFE_COLOR: [f32; 4] = [0.55, 1.0, 0.55, 1.0];
pub const ICE_COLOR: [f32; 4] = [0.80, 0.92, 1.0, 1.0];
pub const SOOT_COLOR: [f32; 4] = [0.08, 0.06, 0.05, 1.0];
pub const VIOLET_COLOR: [f32; 4] = [0.75, 0.45, 1.0, 1.0];

pub const MOAT_BRIDGE_HALF_WIDTH: f32 = 2.4;
pub const CANAL_BRIDGE_HALF_WIDTH: f32 = 1.6;
pub const CANAL_BRIDGE_HALF_SPAN: f32 = 3.2;
pub const SUN_SHADOW_BIAS: f32 = 0.02;
pub const SUN_SHADOW_NORMAL_BIAS: f32 = 2.2;

pub const TREE_GAP: f32 = 9.0;
pub const WOOD_TREE_GAP: f32 = 5.5;
pub const STARTER_TREE_GAP: f32 = 6.5;
pub const SCATTER_GAP: f32 = 4.5;
pub const MOB_GAP: f32 = 9.0;
pub const FOLK_GAP: f32 = 3.2;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Walk,
    Jump,
    SelectTarget,
    AbilityOne,
    AbilityTwo,
    AbilityThree,
    AbilityFour,
    AbilityFive,
    AbilitySix,
    Sheet,
    Bag,
    Log,
    Map,
    Confirm,
}

pub const ABILITY_ASKS: [Ask; 6] = [
    Ask::AbilityOne,
    Ask::AbilityTwo,
    Ask::AbilityThree,
    Ask::AbilityFour,
    Ask::AbilityFive,
    Ask::AbilitySix,
];

impl nightshade_input::Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::Walk,
            Self::Jump,
            Self::SelectTarget,
            Self::AbilityOne,
            Self::AbilityTwo,
            Self::AbilityThree,
            Self::AbilityFour,
            Self::AbilityFive,
            Self::AbilitySix,
            Self::Sheet,
            Self::Bag,
            Self::Log,
            Self::Map,
            Self::Confirm,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Walk => "walk",
            Self::Jump => "jump",
            Self::SelectTarget => "select target",
            Self::AbilityOne => "ability one",
            Self::AbilityTwo => "ability two",
            Self::AbilityThree => "ability three",
            Self::AbilityFour => "ability four",
            Self::AbilityFive => "ability five",
            Self::AbilitySix => "ability six",
            Self::Sheet => "sheet",
            Self::Bag => "bag",
            Self::Log => "log",
            Self::Map => "map",
            Self::Confirm => "confirm",
        }
    }

    fn default_bindings(&self) -> &'static [nightshade_input::Binding] {
        use nightshade_gamepad::contract::GamepadButton;
        use nightshade_input::{Binding, Stick};
        use nightshade_platform::KeyCode;
        match self {
            Self::Walk => &[
                Binding::Keys {
                    up: KeyCode::KeyW,
                    down: KeyCode::KeyS,
                    left: KeyCode::KeyA,
                    right: KeyCode::KeyD,
                },
                Binding::Keys {
                    up: KeyCode::ArrowUp,
                    down: KeyCode::ArrowDown,
                    left: KeyCode::ArrowLeft,
                    right: KeyCode::ArrowRight,
                },
                Binding::Stick(Stick::Left),
            ],
            Self::Jump => &[
                Binding::Key(KeyCode::Space),
                Binding::Pad(GamepadButton::South),
            ],
            Self::SelectTarget => &[Binding::Key(KeyCode::Tab)],
            Self::AbilityOne => &[Binding::Key(KeyCode::Digit1)],
            Self::AbilityTwo => &[Binding::Key(KeyCode::Digit2)],
            Self::AbilityThree => &[Binding::Key(KeyCode::Digit3)],
            Self::AbilityFour => &[Binding::Key(KeyCode::Digit4)],
            Self::AbilityFive => &[Binding::Key(KeyCode::Digit5)],
            Self::AbilitySix => &[Binding::Key(KeyCode::Digit6)],
            Self::Sheet => &[Binding::Key(KeyCode::KeyC)],
            Self::Bag => &[Binding::Key(KeyCode::KeyB)],
            Self::Log => &[Binding::Key(KeyCode::KeyL)],
            Self::Map => &[Binding::Key(KeyCode::KeyM)],
            Self::Confirm => &[
                Binding::Key(KeyCode::Enter),
                Binding::Pad(GamepadButton::South),
            ],
        }
    }
}
