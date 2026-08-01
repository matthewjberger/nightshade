use crate::data::places::{
    BARROW, CRYPT, DEEP, FARM, FIELD, FORT, HARBOR, MINE, STARTER, TOWN, WOOD,
};
use nalgebra_glm::Vec2;

const NORTH: f32 = 0.0;
const SOUTH: f32 = std::f32::consts::PI;
const EAST: f32 = std::f32::consts::FRAC_PI_2;
const WEST: f32 = -std::f32::consts::FRAC_PI_2;

pub const GUARD_MODEL: &str = "character-soldier";
pub const GUARD_NAME: &str = "Hollowmere Guard";
pub const DOG_MODEL: &str = "animal-dog";
pub const BISON_MODEL: &str = "animal-bison";

pub const GUARD_POSTS: [(Vec2, f32); 4] = [
    (Vec2::new(-3.0, 12.0), SOUTH),
    (Vec2::new(3.0, 12.0), SOUTH),
    (Vec2::new(-9.0, -4.0), EAST),
    (Vec2::new(9.0, -4.0), WEST),
];

pub const GUARD_ROUTES: [&[Vec2]; 2] = [
    &[
        Vec2::new(14.0, 14.0),
        Vec2::new(14.0, -14.0),
        Vec2::new(-14.0, -14.0),
        Vec2::new(-14.0, 14.0),
    ],
    &[
        Vec2::new(22.0, 0.0),
        Vec2::new(0.0, 22.0),
        Vec2::new(-22.0, 0.0),
        Vec2::new(0.0, -22.0),
    ],
];

pub const DOG_ROUTES: [&[Vec2]; 1] = [&[
    Vec2::new(8.0, 4.0),
    Vec2::new(8.0, -8.0),
    Vec2::new(-8.0, -8.0),
    Vec2::new(-8.0, 4.0),
]];

pub struct Townsfolk {
    pub name: &'static str,
    pub title: &'static str,
    pub model: &'static str,
    pub scale: f32,
    pub place: usize,
    pub offset: Vec2,
    pub heading: f32,
}

pub const TOWNSFOLK: [Townsfolk; 6] = [
    Townsfolk {
        name: "Ysolde",
        title: "Flower Seller",
        model: "character-female-e",
        scale: 2.6,
        place: TOWN,
        offset: Vec2::new(5.0, 4.0),
        heading: SOUTH,
    },
    Townsfolk {
        name: "Crier Tobben",
        title: "Town Crier",
        model: "character-male-c",
        scale: 2.6,
        place: TOWN,
        offset: Vec2::new(-4.0, 3.0),
        heading: SOUTH,
    },
    Townsfolk {
        name: "Rook",
        title: "Stablemaster",
        model: "character-male-e",
        scale: 2.6,
        place: FARM,
        offset: Vec2::new(7.0, 5.0),
        heading: NORTH,
    },
    Townsfolk {
        name: "Grocer Wenna",
        title: "Market Stall",
        model: "market-character-employee",
        scale: 2.6,
        place: TOWN,
        offset: Vec2::new(-7.0, 12.0),
        heading: SOUTH,
    },
    Townsfolk {
        name: "Brother Cole",
        title: "Gravekeeper",
        model: "character-digger",
        scale: 2.9,
        place: TOWN,
        offset: Vec2::new(-16.0, 12.0),
        heading: SOUTH,
    },
    Townsfolk {
        name: "Deckhand Mo",
        title: "Mirror Harbor",
        model: "character-female-f",
        scale: 2.6,
        place: HARBOR,
        offset: Vec2::new(-4.0, 3.0),
        heading: WEST,
    },
];

pub struct Landmark {
    pub model: &'static str,
    pub place: usize,
    pub offset: Vec2,
    pub yaw: f32,
    pub scale: f32,
}

const fn mark(model: &'static str, place: usize, x: f32, z: f32, yaw: f32, scale: f32) -> Landmark {
    Landmark {
        model,
        place,
        offset: Vec2::new(x, z),
        yaw,
        scale,
    }
}

pub const LANDMARKS: [Landmark; 46] = [
    mark("fountain-round", TOWN, 0.0, 0.0, 0.0, 3.2),
    mark("lantern", TOWN, 4.0, 4.0, 0.0, 3.0),
    mark("lantern", TOWN, -4.0, 4.0, 0.0, 3.0),
    mark("lantern", TOWN, 4.0, -4.0, 0.0, 3.0),
    mark("lantern", TOWN, -4.0, -4.0, 0.0, 3.0),
    mark("cart", TOWN, 9.0, 12.0, 0.6, 3.0),
    mark("stall-red", TOWN, -9.0, 14.0, SOUTH, 3.0),
    mark("gravestone-cross", TOWN, -17.0, 15.0, 0.0, 2.8),
    mark("gravestone-round", TOWN, -19.0, 13.0, 0.4, 2.8),
    mark("gravestone-broken", TOWN, -14.0, 16.0, -0.3, 2.8),
    mark("iron-fence", TOWN, -16.0, 10.0, 0.0, 2.8),
    mark("campfire-pit", FIELD, 0.0, 0.0, 0.0, 3.0),
    mark("hay-bale", FIELD, 6.0, 3.0, 0.5, 3.0),
    mark("fence", FIELD, -8.0, 2.0, 0.0, 3.0),
    mark("tent", STARTER, 0.0, 3.0, SOUTH, 3.0),
    mark("campfire-stand", STARTER, 4.0, 5.0, 0.0, 3.0),
    mark("signpost", STARTER, -4.0, 8.0, 0.4, 3.0),
    mark("tree-log", STARTER, 6.0, 1.0, 1.2, 3.0),
    mark("stump_old", WOOD, 3.0, 2.0, 0.0, 3.0),
    mark("rock_largeA", WOOD, -6.0, 8.0, 0.8, 3.0),
    mark("mushroom_redGroup", WOOD, 8.0, -6.0, 0.0, 3.0),
    mark(
        "castle-tower-square-top-roof-high",
        FORT,
        0.0,
        0.0,
        0.0,
        3.4,
    ),
    mark("castle-wall", FORT, 6.0, 0.0, EAST, 3.4),
    mark("castle-wall", FORT, -6.0, 0.0, EAST, 3.4),
    mark("banner-red", FORT, 3.0, 6.0, 0.0, 3.0),
    mark("tent", FORT, -7.0, 8.0, NORTH, 3.0),
    mark("tent", FORT, 8.0, -8.0, WEST, 3.0),
    mark("campfire-pit", FORT, 0.0, -7.0, 0.0, 3.0),
    mark("crypt-large", BARROW, 0.0, 0.0, SOUTH, 3.6),
    mark("gravestone-wide", BARROW, 8.0, 6.0, 0.3, 3.0),
    mark("gravestone-roof", BARROW, -9.0, 5.0, -0.5, 3.0),
    mark("gravestone-decorative", BARROW, 6.0, -9.0, 0.9, 3.0),
    mark("pillar-obelisk", BARROW, -7.0, -8.0, 0.0, 3.0),
    mark("candle-multiple", BARROW, 2.0, 4.0, 0.0, 3.0),
    mark("crypt-door", CRYPT, 0.0, 0.0, SOUTH, 3.6),
    mark("altar-stone", CRYPT, 0.0, -5.0, 0.0, 3.0),
    mark("coffin", CRYPT, 5.0, 3.0, 0.7, 3.0),
    mark("wood-structure-high", MINE, 0.0, 0.0, 0.0, 3.4),
    mark("detail-crystal-large", MINE, 6.0, 5.0, 0.0, 3.0),
    mark("detail-rocks-large", MINE, -7.0, 6.0, 0.6, 3.0),
    mark("wood-structure", DEEP, 0.0, 0.0, 0.0, 3.4),
    mark(
        "pirate-structure-platform-dock",
        HARBOR,
        0.0,
        -4.0,
        0.0,
        3.4,
    ),
    mark("pirate-boat-row-small", HARBOR, 5.0, -8.0, 1.0, 3.0),
    mark("pirate-barrel", HARBOR, -3.0, 2.0, 0.0, 3.0),
    mark("windmill", FARM, -8.0, 0.0, 0.0, 3.4),
    mark("hay-bale-bundled", FARM, 5.0, -5.0, 0.4, 3.0),
];
