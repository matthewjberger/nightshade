#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum District {
    #[default]
    Downtown,
    Commercial,
    Industrial,
    Suburban,
}

pub const DOWNTOWN_REACH: i32 = 2;
pub const COMMERCIAL_REACH: i32 = 7;
pub const INDUSTRIAL_FROM: i32 = 8;
pub const INDUSTRIAL_SPAN: i32 = 6;

pub const DISTRICT_NAMES: [&str; 4] = ["Downtown", "Midtown", "The Works", "The Rows"];

pub const ROAD_STRAIGHT: &str = "road-straight";
pub const ROAD_CROSSROAD: &str = "road-crossroad";
pub const ROAD_CROSSING: &str = "road-crossing";
pub const LAMP_POST: &str = "light-curved";

pub const SIDEWALK_PROPS: [&str; 6] = [
    "furn-bench",
    "furn-trashcan",
    "furn-pottedPlant",
    "urban-detail-bench",
    "furn-cardboardBoxClosed",
    "furn-plantSmall2",
];
pub const CAFE_TABLES: [&str; 3] = ["furn-tableRound", "furn-tableCross", "furn-table"];
pub const CAFE_CHAIRS: [&str; 3] = ["furn-chair", "furn-chairRounded", "furn-stoolBar"];
pub const ALLEY_PROPS: [&str; 8] = [
    "urban-detail-dumpster-closed",
    "urban-detail-dumpster-open",
    "furn-cardboardBoxOpen",
    "furn-cardboardBoxClosed",
    "urban-pallet-small",
    "urban-planks",
    "furn-loungeSofa",
    "furn-bookcaseOpen",
];
pub const YARD_PROPS: [&str; 8] = [
    "furn-loungeChair",
    "furn-benchCushion",
    "furn-tableRound",
    "furn-chairRounded",
    "furn-pottedPlant",
    "furn-plantSmall1",
    "furn-trashcan",
    "furn-bear",
];
pub const CURB_PROPS: [&str; 3] = [
    "construction-cone",
    "construction-barrier",
    "construction-light",
];
pub const SITE_PROPS: [&str; 12] = [
    "construction-cone",
    "construction-barrier",
    "construction-light",
    "urban-detail-barrier-type-a",
    "urban-detail-barrier-type-b",
    "urban-detail-barrier-strong-type-a",
    "urban-detail-barrier-strong-damaged",
    "urban-detail-bricks-type-a",
    "urban-detail-block",
    "urban-detail-beam",
    "urban-pallet",
    "urban-planks",
];
pub const SCAFFOLDING: [&str; 2] = ["urban-scaffolding-structure", "urban-scaffolding-poles"];

pub const COMMERCIAL_BUILDINGS: [&str; 14] = [
    "commercial-building-a",
    "commercial-building-b",
    "commercial-building-c",
    "commercial-building-d",
    "commercial-building-e",
    "commercial-building-f",
    "commercial-building-g",
    "commercial-building-h",
    "commercial-building-i",
    "commercial-building-j",
    "commercial-building-k",
    "commercial-building-l",
    "commercial-building-m",
    "commercial-building-n",
];

pub const TOWERS: [&str; 5] = [
    "commercial-building-skyscraper-a",
    "commercial-building-skyscraper-b",
    "commercial-building-skyscraper-c",
    "commercial-building-skyscraper-d",
    "commercial-building-skyscraper-e",
];

pub const INDUSTRIAL_BUILDINGS: [&str; 20] = [
    "industrial-building-a",
    "industrial-building-b",
    "industrial-building-c",
    "industrial-building-d",
    "industrial-building-e",
    "industrial-building-f",
    "industrial-building-g",
    "industrial-building-h",
    "industrial-building-i",
    "industrial-building-j",
    "industrial-building-k",
    "industrial-building-l",
    "industrial-building-m",
    "industrial-building-n",
    "industrial-building-o",
    "industrial-building-p",
    "industrial-building-q",
    "industrial-building-r",
    "industrial-building-s",
    "industrial-building-t",
];

pub const INDUSTRIAL_PROPS: [&str; 4] = [
    "industrial-chimney-large",
    "industrial-chimney-medium",
    "industrial-chimney-small",
    "industrial-detail-tank",
];

pub const HOUSES: [&str; 21] = [
    "suburban-building-type-a",
    "suburban-building-type-b",
    "suburban-building-type-c",
    "suburban-building-type-d",
    "suburban-building-type-e",
    "suburban-building-type-f",
    "suburban-building-type-g",
    "suburban-building-type-h",
    "suburban-building-type-i",
    "suburban-building-type-j",
    "suburban-building-type-k",
    "suburban-building-type-l",
    "suburban-building-type-m",
    "suburban-building-type-n",
    "suburban-building-type-o",
    "suburban-building-type-p",
    "suburban-building-type-q",
    "suburban-building-type-r",
    "suburban-building-type-s",
    "suburban-building-type-t",
    "suburban-building-type-u",
];

pub const TREES: [&str; 2] = ["suburban-tree-large", "suburban-tree-small"];
pub const PLANTER: &str = "suburban-planter";
pub const FENCE: &str = "suburban-fence";

pub const CARS: [&str; 12] = [
    "car-sedan",
    "car-sedan-sports",
    "car-hatchback-sports",
    "car-suv",
    "car-suv-luxury",
    "car-van",
    "car-taxi",
    "car-delivery",
    "car-police",
    "car-ambulance",
    "car-race",
    "car-race-future",
];

pub const TRUCKS: [&str; 5] = [
    "car-truck",
    "car-truck-flat",
    "car-delivery-flat",
    "car-garbage-truck",
    "car-tractor",
];

pub const PEOPLE: [&str; 12] = [
    "character-male-a",
    "character-male-b",
    "character-male-c",
    "character-male-d",
    "character-male-e",
    "character-male-f",
    "character-female-a",
    "character-female-b",
    "character-female-c",
    "character-female-d",
    "character-female-e",
    "character-female-f",
];

pub const HERO: &str = "character-female-c";

pub const CLIP_NAMES: [&str; 7] = ["idle", "walk", "sprint", "sit", "drive", "die", "static"];
pub const CLIP_IDLE: usize = 0;
pub const CLIP_WALK: usize = 1;
pub const CLIP_SPRINT: usize = 2;
pub const CLIP_DRIVE: usize = 4;
pub const CLIP_DIE: usize = 5;
pub const CLIP_STATIC: usize = 6;

pub const TRAFFIC_PER_CELL: [usize; 4] = [3, 3, 2, 1];
pub const PEOPLE_PER_CELL: [usize; 4] = [8, 6, 2, 3];
pub const NIGHT_LIFE: [f32; 4] = [1.3, 1.0, 0.4, 0.5];
pub const DAY_LIFE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
