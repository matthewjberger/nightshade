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
pub const PLACE_COUNT: usize = 12;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Ground {
    Land,
    Water,
    High,
    Shore,
}

pub struct PlacePlan {
    pub name: &'static str,
    pub radius: f32,
    pub reach: f32,
    pub angle: f32,
    pub ground: Ground,
    pub anchor: usize,
}

pub const PLACES: [PlacePlan; PLACE_COUNT] = [
    PlacePlan {
        name: "Hollowmere",
        radius: 30.0,
        reach: 0.0,
        angle: 0.0,
        ground: Ground::Land,
        anchor: TOWN,
    },
    PlacePlan {
        name: "The Boar Field",
        radius: 20.0,
        reach: 58.0,
        angle: 0.0,
        ground: Ground::Land,
        anchor: TOWN,
    },
    PlacePlan {
        name: "Wolf Wood",
        radius: 32.0,
        reach: 100.0,
        angle: 1.2,
        ground: Ground::Land,
        anchor: TOWN,
    },
    PlacePlan {
        name: "Redharrow Fort",
        radius: 16.0,
        reach: 125.0,
        angle: -1.3,
        ground: Ground::Land,
        anchor: TOWN,
    },
    PlacePlan {
        name: "The Barrow",
        radius: 22.0,
        reach: 150.0,
        angle: 2.3,
        ground: Ground::Land,
        anchor: TOWN,
    },
    PlacePlan {
        name: "Mirror Lake",
        radius: 26.0,
        reach: 130.0,
        angle: 3.6,
        ground: Ground::Water,
        anchor: TOWN,
    },
    PlacePlan {
        name: "Holt's Farm",
        radius: 16.0,
        reach: 80.0,
        angle: -2.4,
        ground: Ground::Land,
        anchor: TOWN,
    },
    PlacePlan {
        name: "The Sunken Crypt",
        radius: 14.0,
        reach: 20.0,
        angle: 2.3,
        ground: Ground::Land,
        anchor: BARROW,
    },
    PlacePlan {
        name: "Ironjaw Mine",
        radius: 24.0,
        reach: 180.0,
        angle: 4.6,
        ground: Ground::High,
        anchor: TOWN,
    },
    PlacePlan {
        name: "Mirror Harbor",
        radius: 10.0,
        reach: 30.0,
        angle: 0.0,
        ground: Ground::Shore,
        anchor: LAKE,
    },
    PlacePlan {
        name: "Ironjaw Deep",
        radius: 10.0,
        reach: 16.0,
        angle: 4.6,
        ground: Ground::Land,
        anchor: MINE,
    },
    PlacePlan {
        name: "Fallowmere Wood",
        radius: 30.0,
        reach: 55.0,
        angle: 2.2,
        ground: Ground::Land,
        anchor: TOWN,
    },
];

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
