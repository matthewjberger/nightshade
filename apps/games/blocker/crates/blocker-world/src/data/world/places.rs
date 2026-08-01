use super::nodes::{CAPITAL_SLOTS, HENGE_STAMP};
use super::schematics::{
    BARROW_SCHEMATIC, CAMP_SCHEMATIC, KEEP_SCHEMATIC, LIGHTHOUSE_SCHEMATIC, SCHEMATICS,
    SHRINE_SCHEMATIC,
};

pub struct Poi {
    pub at: [i32; 2],
    pub name: &'static str,
    pub schematic: usize,
}

const fn poi(at: [i32; 2], name: &'static str, schematic: usize) -> Poi {
    Poi {
        at,
        name,
        schematic,
    }
}

pub const POIS: [Poi; 9] = [
    poi([300, 300], "Northlight", LIGHTHOUSE_SCHEMATIC),
    poi([-420, 260], "Graywater Light", LIGHTHOUSE_SCHEMATIC),
    poi([180, -520], "Dawnlight", LIGHTHOUSE_SCHEMATIC),
    poi([-260, -360], "The Old Beacon", LIGHTHOUSE_SCHEMATIC),
    poi([140, 180], "The Quiet Altar", SHRINE_SCHEMATIC),
    poi([-190, 120], "Mossy Shrine", SHRINE_SCHEMATIC),
    poi([420, -160], "Wayshrine", SHRINE_SCHEMATIC),
    poi([-340, -140], "Ravenkeep", KEEP_SCHEMATIC),
    poi([360, 420], "Blackmoor Ruin", KEEP_SCHEMATIC),
];

pub struct Landmark {
    pub at: [i32; 2],
    pub node: usize,
    pub rot: u8,
}

const fn landmark(at: [i32; 2], node: usize, rot: u8) -> Landmark {
    Landmark { at, node, rot }
}

pub const LANDMARKS: [Landmark; 1] = [landmark([208, -164], HENGE_STAMP, 0)];

pub struct Capital {
    pub at: [i32; 2],
    pub name: &'static str,
}

const fn capital(at: [i32; 2], name: &'static str) -> Capital {
    Capital { at, name }
}

pub const CAPITALS: [Capital; 5] = [
    capital([0, 0], "Dawnhold"),
    capital([720, -560], "Ironmarch"),
    capital([-680, 520], "Stormcrest"),
    capital([560, 760], "Emberfort"),
    capital([-820, -700], "Frostmere"),
];

pub struct Barrow {
    pub name: &'static str,
    pub capital: &'static str,
}

const fn barrow(name: &'static str, capital: &'static str) -> Barrow {
    Barrow { name, capital }
}

pub const BARROW_OFFSET: [i32; 2] = [58, 96];

pub const BARROWS: [Barrow; 5] = [
    barrow("The Hollow Barrow", "Dawnhold"),
    barrow("Rustgrave Den", "Ironmarch"),
    barrow("Windwake Barrow", "Stormcrest"),
    barrow("Cinderhollow Den", "Emberfort"),
    barrow("Rimewatch Barrow", "Frostmere"),
];

const _: () = assert!(BARROWS.len() == CAPITALS.len());

pub struct Lost {
    pub name: &'static str,
    pub note: &'static str,
}

const fn lost(name: &'static str, note: &'static str) -> Lost {
    Lost { name, note }
}

pub const GATE_NOTICE_HEAD: &str = "The watch asks for word of";

pub const GATE_NOTICE_TAIL: &str = "Six went into the barrow to the south-east and none came back. Bring word to Sergeant Ovilda at the watch lodge, south-east quarter, inside the wall.";

pub const LOST: [Lost; 3] = [
    lost("Aldis Renn", "went down and did not come up"),
    lost("Sela Vance", "three weeks gone"),
    lost("Watchman Orrin", "sent after them"),
];

const _: () = assert!(LOST.len() == crate::theme::MARKER_OFFSETS.len());

pub struct Camp {
    pub name: &'static str,
}

const fn camp(name: &'static str) -> Camp {
    Camp { name }
}

pub const CAMP_OFFSET: [i32; 2] = [132, -18];

pub const CAMPS: [Camp; 5] = [
    camp("The Hollow Camp"),
    camp("Rustgrave Hollow"),
    camp("Windwake Hollow"),
    camp("Cinderhollow Copse"),
    camp("Rimewatch Copse"),
];

pub const CAMP_TREES: [[i32; 2]; 6] = [[-16, -12], [16, -12], [-17, 3], [17, 4], [-8, 16], [9, 16]];

const fn farthest(offset: [i32; 2]) -> i32 {
    if offset[0].abs() > offset[1].abs() {
        offset[0].abs()
    } else {
        offset[1].abs()
    }
}

const fn capital_spread() -> i32 {
    let mut spread = crate::data::world::CITY_WALL;
    let mut index = 0;
    while index < CAPITAL_SLOTS.len() {
        spread = farthest([spread, CAPITAL_SLOTS[index].dx]);
        spread = farthest([spread, CAPITAL_SLOTS[index].dz]);
        index += 1;
    }
    spread
}

pub const fn den_keeps_clear(reach: i32) -> i32 {
    capital_spread() + crate::theme::CITY_TERRACE_MARGIN + 3 + reach
}

const fn seat_clears_the_capital(offset: [i32; 2], reach: i32) -> bool {
    farthest(offset) >= den_keeps_clear(reach)
}

const _: () = assert!(seat_clears_the_capital(
    BARROW_OFFSET,
    crate::theme::DEN_CLAIM
));
const _: () = assert!(seat_clears_the_capital(
    CAMP_OFFSET,
    crate::theme::CAMP_REACH
));
const _: () = assert!(
    SCHEMATICS[BARROW_SCHEMATIC].span / 2 + crate::theme::BUILDING_APRON <= crate::theme::DEN_CLAIM
);
const _: () = assert!(
    SCHEMATICS[CAMP_SCHEMATIC].span / 2 + crate::theme::BUILDING_APRON <= crate::theme::CAMP_REACH
);
const _: () = assert!(CAMPS.len() == CAPITALS.len());
