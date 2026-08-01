use blocker_blocks::data::blocks::BlockKind;

pub struct Plaque {
    pub x: i32,
    pub z: i32,
    pub layer: i32,
    pub text: &'static str,
}

const fn plaque(x: i32, z: i32, layer: i32, text: &'static str) -> Plaque {
    Plaque { x, z, layer, text }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SchematicName {
    Cottage,
    House,
    Lodge,
    Townhall,
    Well,
    Market,
    Farm,
    Oak,
    OakTall,
    Pine,
    FlowerRed,
    FlowerYellow,
    Deadbush,
    Boulder,
    Tower,
    Obelisk,
    Ruin,
    Henge,
    Lamp,
    Watchtower,
    Graveyard,
    Campfire,
    Ship,
    Lighthouse,
    Shrine,
    Keep,
    Barrow,
    Camp,
    Stonehouse,
    Redroof,
    Tallhouse,
}

pub struct Schematic {
    pub name: SchematicName,
    pub span: i32,
    pub sink: i32,
    pub foundation: bool,
    pub layers: &'static [&'static [&'static str]],
    pub plaques: &'static [Plaque],
}

const fn schematic(
    name: SchematicName,
    span: i32,
    sink: i32,
    foundation: bool,
    layers: &'static [&'static [&'static str]],
    plaques: &'static [Plaque],
) -> Schematic {
    Schematic {
        name,
        span,
        sink,
        foundation,
        layers,
        plaques,
    }
}

pub fn schematic_block(glyph: u8) -> Option<BlockKind> {
    Some(match glyph {
        b'P' => BlockKind::OakPlanks,
        b'L' => BlockKind::OakLog,
        b'C' => BlockKind::Cobblestone,
        b'S' => BlockKind::Stone,
        b'B' => BlockKind::Bricks,
        b'O' => BlockKind::Obsidian,
        b'G' => BlockKind::Glass,
        b'W' => BlockKind::WoolRed,
        b'w' => BlockKind::WoolWhite,
        b'u' => BlockKind::WoolBlue,
        b'D' => BlockKind::DoorSouth,
        b't' => BlockKind::TorchSouth,
        b'T' => BlockKind::Torch,
        b'g' => BlockKind::Glowstone,
        b'H' => BlockKind::Chest,
        b'K' => BlockKind::CraftingTable,
        b'F' => BlockKind::Campfire,
        b'l' => BlockKind::OakLeaves,
        b'a' => BlockKind::Water,
        b'R' => BlockKind::RedFlower,
        b'Y' => BlockKind::YellowFlower,
        b'e' => BlockKind::DeadBush,
        b'#' => BlockKind::Signpost,
        b'd' => BlockKind::Dirt,
        b'n' => BlockKind::Snow,
        b',' => BlockKind::Gravel,
        b'_' => BlockKind::Air,
        _ => return None,
    })
}

const COTTAGE: &[&[&str]] = &[
    &["7P", "7P", "7P", "7P", "7P", "7P", "7P"],
    &[
        "L2PD2PL", "P5.P", "P5.P", "P2.T2.P", "P.H.K.P", "P5.P", "L5PL",
    ],
    &["L2PD2PL", "P5.P", "G5.G", "P5.P", "G5.G", "P5.P", "LPGPGPL"],
    &["L5PL", "P5.P", "P5.P", "P5.P", "P5.P", "P5.P", "L5PL"],
    &["7L", "L5PL", "L5PL", "L5PL", "L5PL", "L5PL", "7L"],
    &["C6.", "7.", "7.", "7.", "7.", "7.", "7."],
    &["C6.", "7.", "7.", "7.", "7.", "7.", "7."],
];

const HOUSE: &[&[&str]] = &[
    &["9P", "9P", "9P", "9P", "9P", "9P", "9P", "9P", "9P"],
    &[
        "L3BD3BL", "B7.B", "B7.B", "B3.T3.B", "B7.B", "B.H3.K.B", "B7.B", "B7.B", "L7BL",
    ],
    &[
        "L3BD3BL", "B7.B", "G7.G", "B7.B", "G7.G", "B7.B", "G7.G", "B7.B", "LBG3BGBL",
    ],
    &[
        "L7BL", "B7.B", "B7.B", "B7.B", "B7.B", "B7.B", "B7.B", "B7.B", "L7BL",
    ],
    &[
        "9L", "L7PL", "L7PL", "L7PL", "L7PL", "L7PL", "L7PL", "L7PL", "9L",
    ],
    &[
        "9.", ".7L.", ".L5PL.", ".L5PL.", ".L5PL.", ".L5PL.", ".L5PL.", ".7L.", "9.",
    ],
    &["C8.", "9.", "9.", "9.", "9.", "9.", "9.", "9.", "9."],
];

const LODGE: &[&[&str]] = &[
    &["9P", "9P", "9P", "9P", "9P", "9P", "9P", "9P", "9P"],
    &[
        "4LD4L", "L7.L", "L7.L", "L3.F3.L", "L7.L", "L.H5.L", "L7.L", "L7.L", "9L",
    ],
    &[
        "4LD4L", "L7.L", "G7.G", "L7.L", "G7.G", "L7.L", "G7.G", "L7.L", "2LG3LG2L",
    ],
    &[
        "9L", "L7.L", "L7.L", "L7.L", "L7.L", "L7.L", "L7.L", "L7.L", "9L",
    ],
    &[
        "9L", "L7PL", "L7PL", "L7PL", "L7PL", "L7PL", "L7PL", "L7PL", "9L",
    ],
    &[
        "9.", ".7n.", ".7n.", ".7n.", ".7n.", ".7n.", ".7n.", ".7n.", "9.",
    ],
    &["C8.", "9.", "9.", "9.", "9.", "9.", "9.", "9.", "9."],
];

const TOWNHALL: &[&[&str]] = &[
    &[
        "11S", "11S", "11S", "11S", "11S", "11S", "11S", "11S", "11S", "11S", "11S",
    ],
    &[
        "L4P2D3PL", "P9.P", "P9.P", "P4.T4.P", "P9.P", "P9.P", "P9.P", "P9.P", "P4.H4.P", "P9.P",
        "L9PL",
    ],
    &[
        "L4P2D3PL",
        "P9.P",
        "2G7.2G",
        "P9.P",
        "W9.W",
        "P9.P",
        "2G7.2G",
        "P9.P",
        "W9.W",
        "P9.P",
        "LP2GP2GPGPL",
    ],
    &[
        "L9PL", "P9.P", "G9.G", "P9.P", "G9.G", "P9.P", "G9.G", "P9.P", "G9.G", "P9.P", "LPG5PGPL",
    ],
    &[
        "L9PL", "P9.P", "P9.P", "P9.P", "P9.P", "P9.P", "P9.P", "P9.P", "P9.P", "P9.P", "L9PL",
    ],
    &[
        "11L", "L9PL", "L9PL", "L9PL", "L9PL", "L9PL", "L9PL", "L9PL", "L9PL", "L9PL", "11L",
    ],
    &[
        "11.", ".9L.", ".L7PL.", ".L7PL.", ".L7PL.", ".L7PL.", ".L7PL.", ".L7PL.", ".L7PL.",
        ".9L.", "11.",
    ],
];

const MARKET: &[&[&str]] = &[
    &["5P", "5P", "5P", "5P", "5P"],
    &["L3.L", "5.", "2.H2.", "5.", "L3.L"],
    &["L3.L", "5.", "5.", "5.", "L3.L"],
    &["5P", "P3WP", "P3WP", "P3WP", "5P"],
];

const FARM: &[&[&str]] = &[&["7L", "L5dL", "LdadadL", "L5dL", "LdadadL", "L5dL", "7L"]];

const LAMP: &[&[&str]] = &[&["L"], &["L"], &["L"], &["T"]];

const WELL: &[&[&str]] = &[&["CCC", "CaC", "CCC"], &["C.C", "...", "C.C"]];

const OAK: &[&[&str]] = &[
    &[".....", ".....", "..L..", ".....", "....."],
    &[".....", ".....", "..L..", ".....", "....."],
    &[".....", ".....", "..L..", ".....", "....."],
    &[".....", ".....", "..L..", ".....", "....."],
    &["lllll", "lllll", "llLll", "lllll", "lllll"],
    &[".lll.", "lllll", "lllll", "lllll", ".lll."],
    &[".....", ".lll.", ".lll.", ".lll.", "....."],
];

const OAK_TALL: &[&[&str]] = &[
    &[".....", ".....", "..L..", ".....", "....."],
    &[".....", ".....", "..L..", ".....", "....."],
    &[".....", ".....", "..L..", ".....", "....."],
    &[".....", ".....", "..L..", ".....", "....."],
    &[".....", ".....", "..L..", ".....", "....."],
    &["lllll", "lllll", "llLll", "lllll", "lllll"],
    &["lllll", "lllll", "lllll", "lllll", "lllll"],
    &[".lll.", "lllll", "lllll", "lllll", ".lll."],
    &[".....", ".lll.", ".lll.", ".lll.", "....."],
];

const PINE: &[&[&str]] = &[
    &[".....", ".....", "..L..", ".....", "....."],
    &[".....", ".....", "..L..", ".....", "....."],
    &["..l..", ".lll.", "llLll", ".lll.", "..l.."],
    &[".....", "..l..", ".lLl.", "..l..", "....."],
    &[".....", ".lll.", ".lLl.", ".lll.", "....."],
    &[".....", "..l..", ".lll.", "..l..", "....."],
    &[".....", ".....", "..l..", ".....", "....."],
];

const FLOWER_RED: &[&[&str]] = &[&["R"]];
const FLOWER_YELLOW: &[&[&str]] = &[&["Y"]];
const DEADBUSH: &[&[&str]] = &[&["e"]];

const BOULDER: &[&[&str]] = &[&["SSS", "SSS", "SSS"], &[".S.", "SSS", ".S."]];

const TOWER: &[&[&str]] = &[
    &["SSSSS", "S...S", "S...S", "S...S", "SSSSS"],
    &["SSSSS", "S...S", "S...S", "S...S", "SSSSS"],
    &["SSSSS", "S...S", "S...S", "S...S", "SSSSS"],
    &["SS.SS", "S...S", "S...S", "S...S", "SSSSS"],
    &["SSSSS", "S...S", "S...S", "S...S", "SS.SS"],
    &["SS.SS", "S...S", ".....", "S...S", "S.SSS"],
    &[".S.S.", "S....", ".....", "....S", ".SS.."],
];

const OBELISK: &[&[&str]] = &[&["O"], &["O"], &["O"], &["O"], &["O"], &["O"]];

const WATCHTOWER: &[&[&str]] = &[
    &["5S", "S3.S", "S3.S", "S3.S", "5S"],
    &["2SD2S", "S3.S", "S3.S", "S3.S", "5S"],
    &["2SD2S", "S3.S", "S3.S", "S3.S", "5S"],
    &["2S.2S", "S3.S", "S3.S", "S3.S", "5S"],
    &["5S", "S3.S", "S3.S", "S3.S", "5S"],
    &["5L", "5L", "5L", "5L", "5L"],
    &["L.L.L", "L3.L", "L3.L", "L3.L", "L.L.L"],
    &["5.", "5.", "2.T2.", "5.", "5."],
];

const GRAVEYARD: &[&[&str]] = &[
    &["3L.3L", "L5dL", "L5dL", "L5dL", "L5dL", "L5dL", "7L"],
    &["7.", ".C.C.C.", "7.", ".C.C.C.", "7.", ".C.C.C.", "7."],
];

const CAMP: &[&[&str]] = &[
    &["L3.L", "5.", "2.F2.", "2.H2.", "L3.L"],
    &["L3.L", "5.", "5.", "5.", "L3.L"],
    &["5w", "5w", "5w", "5w", "5w"],
];

const RUIN: &[&[&str]] = &[
    &[
        "CCCCCCC", "C.....C", "C.....C", "C.....C", "C.....C", "C.....C", "CCCCCCC",
    ],
    &[
        "CC.C.CC", "C.....C", ".......", "C.....C", ".....C.", "C.....C", "CC.CC.C",
    ],
    &[
        "C...C..", ".....C.", ".......", "C......", ".......", ".C.....", "..C.C.C",
    ],
];

const HENGE: &[&[&str]] = &[
    &[
        ".......", ".O.O.O.", ".......", ".O.g.O.", ".......", ".O.O.O.", ".......",
    ],
    &[
        ".......", ".O.O.O.", ".......", ".O...O.", ".......", ".O.O.O.", ".......",
    ],
    &[
        ".......", ".O.O.O.", ".......", ".O...O.", ".......", ".O.O.O.", ".......",
    ],
    &[
        ".......", "..O.O..", ".......", ".......", ".......", "..O.O..", ".......",
    ],
];

const SHIP: &[&[&str]] = &[
    &[
        "....P....",
        "...PPP...",
        "..PPPPP..",
        "..PPPPP..",
        "..PPPPP..",
        "..PPPPP..",
        "..PPPPP..",
        "...PPP...",
        "....P....",
    ],
    &[
        "....P....",
        "...P.P...",
        "..P...P..",
        "..P.H.P..",
        "..P.L.P..",
        "..P...P..",
        "..P...P..",
        "...P.P...",
        "....P....",
    ],
    &[
        ".........",
        ".........",
        ".........",
        ".........",
        "....L....",
        ".........",
        ".........",
        ".........",
        ".........",
    ],
    &[
        ".........",
        ".........",
        ".........",
        ".........",
        "....L....",
        ".........",
        ".........",
        ".........",
        ".........",
    ],
    &[
        ".........",
        ".........",
        ".........",
        ".........",
        "..wwLww..",
        ".........",
        ".........",
        ".........",
        ".........",
    ],
    &[
        ".........",
        ".........",
        ".........",
        ".........",
        "..wwwww..",
        ".........",
        ".........",
        ".........",
        ".........",
    ],
    &[
        ".........",
        ".........",
        ".........",
        ".........",
        "....L....",
        ".........",
        ".........",
        ".........",
        ".........",
    ],
    &[
        ".........",
        ".........",
        ".........",
        ".........",
        "....W....",
        ".........",
        ".........",
        ".........",
        ".........",
    ],
];

const LIGHTHOUSE: &[&[&str]] = &[
    &["SSSSS", "SSSSS", "SSSSS", "SSSSS", "SSSSS"],
    &["CCCCC", "C...C", "C...C", "C...C", "CCCCC"],
    &["CCCCC", "C...C", "C...C", "C...C", "CCCCC"],
    &["CCCCC", "C...C", "C...C", "C...C", "CCCCC"],
    &["CCCCC", "C...C", "C...C", "C...C", "CCCCC"],
    &["CCCCC", "C...C", "C...C", "C...C", "CCCCC"],
    &["CGGGC", "G...G", "G...G", "G...G", "CGGGC"],
    &["WWWWW", "W...W", "W...W", "W...W", "WWWWW"],
    &["ggggg", "ggggg", "ggggg", "ggggg", "ggggg"],
    &["CCCCC", "CCCCC", "CCCCC", "CCCCC", "CCCCC"],
];

const SHRINE: &[&[&str]] = &[
    &[".....", ".SSS.", ".SSS.", ".SSS.", "....."],
    &[".....", ".t.t.", "..O..", ".t.t.", "....."],
    &[".....", ".....", "..O..", ".....", "....."],
    &[".....", ".....", "..g..", ".....", "....."],
];

const KEEP: &[&[&str]] = &[
    &[
        "15C", "15C", "15C", "15C", "15C", "15C", "15C", "15C", "15C", "15C", "15C", "15C", "15C",
        "15C", "15C",
    ],
    &[
        "15S",
        "S4.5S4.S",
        "S4.5S4.S",
        "S13.S",
        "S2.S7.S2.S",
        "S13.S",
        "ST11.TS",
        "S13.S",
        "ST11.TS",
        "S13.S",
        "S2.S7.S2.S",
        "S13.S",
        "S12.2S",
        "S13.S",
        "7SD7S",
    ],
    &[
        "3SG3SG3SG3S",
        "S5.gOg5.S",
        "S13.S",
        "S13.S",
        "S2.S7.S2.S",
        "S13.S",
        "S13.S",
        "S13.S",
        "S13.S",
        "S13.S",
        "S2.S7.S2.S",
        "S12.2S",
        "S13.S",
        "S13.S",
        "7SD7S",
    ],
    &[
        "15S",
        "S.g9.g.S",
        "S13.S",
        "S13.S",
        "S2.S7.S2.S",
        "S13.S",
        "S13.S",
        "S13.S",
        "S13.S",
        "S13.S",
        "S2.S7.S.2S",
        "S13.S",
        "S13.S",
        "S.g9.g.S",
        "15S",
    ],
    &[
        "15S", "S13PS", "S13PS", "S13PS", "S13PS", "S13PS", "S13PS", "S13PS", "S11P2_S", "S12P_S",
        "S13PS", "S13PS", "S13PS", "S13PS", "15S",
    ],
    &[
        "15S",
        "S13_S",
        "S_2H3_S3_K2_S",
        "S6_S6_S",
        "S_H4_S4_T_S",
        "S6_S6_S",
        "S6_S6_S",
        "S13_S",
        "S6_S6_S",
        "S6_S6_S",
        "S6_S6_S",
        "S6_S6_S",
        "S_T4_S3_H2_S",
        "S13_S",
        "15S",
    ],
    &[
        "3SG3SG3SG3S",
        "S13_S",
        "S6_S6_S",
        "G6_S6_G",
        "S6_S6_S",
        "S6_S6_S",
        "S6_S6_S",
        "G13_G",
        "S6_S6_S",
        "S6_S6_S",
        "S6_S6_S",
        "G6_S6_G",
        "S6_S6_S",
        "S13_S",
        "15S",
    ],
    &[
        "15S", "S13_S", "S6_S6_S", "S6_S6_S", "S6_S6_S", "S6_S6_S", "S6_S6_S", "S6_S6_S",
        "S6_S6_S", "S6_S6_S", "S6_S6_S", "S6_S6_S", "S6_S6_S", "S13_S", "15S",
    ],
    &[
        "15S", "15S", "15S", "15S", "15S", "15S", "15S", "15S", "15S", "15S", "15S", "15S", "15S",
        "15S", "15S",
    ],
    &[
        "15C", "C13.C", "C13.C", "C13.C", "C13.C", "C13.C", "C13.C", "C13.C", "C13.C", "C13.C",
        "C13.C", "C13.C", "C13.C", "C13.C", "15C",
    ],
    &[
        "C.C.C.C.C.C.C.C",
        ".T11.T.",
        "C13.C",
        "15.",
        "C13.C",
        "15.",
        "C13.C",
        "15.",
        "C13.C",
        "15.",
        "C13.C",
        "15.",
        "C13.C",
        ".T11.T.",
        "C.C.C.C.C.C.C.C",
    ],
];

const KEEP_PLAQUES: [Plaque; 3] = [
    plaque(
        7,
        3,
        1,
        "The banner of the watch hangs from the beam. Two threads for every name on the roll, one grey and one gold.",
    ),
    plaque(
        4,
        7,
        5,
        "The roll of the watch. Names go on at dawn and come off at dusk, and the sergeant keeps the pen.",
    ),
    plaque(
        10,
        7,
        5,
        "A map of the vale scratched on the table. The barrow is circled three times.",
    ),
];

const BARROW: &[&[&str]] = &[
    &[
        "31S", "31S", "31S", "31S", "31S", "31S", "31S", "31S", "31S", "31S", "31S", "31S", "31S",
        "31S", "31S", "31S", "31S", "31S", "31S", "31S", "31S", "31S", "31S", "31S", "31S", "31S",
        "31S", "31S", "31S", "31S", "31S",
    ],
    &[
        "31S",
        "11S2_5S2_11S",
        "11S2_5S2_11S",
        "11S9_11S",
        "11S_S5_S_11S",
        "11S9_11S",
        "11S_S_S_S_S_11S",
        "14S3_14S",
        "7S17_7S",
        "7S3_S3_S3_S3_S_7S",
        "7S17_7S",
        "7S17_7S",
        "7S3_S3_S3_S3_S_7S",
        "7S17_7S",
        "14S3_14S",
        "S6_2S13_2S6_S",
        "S_S2_S_2S_T9_T_2S_S2_S_S",
        "S12_e3_e12_S",
        "S14_F14_S",
        "S12_e3_e12_S",
        "S_S2_S_2S2_H10_2S_S2_S_S",
        "S6_2S13_2S6_S",
        "14S3_14S",
        "31S",
        "31S",
        "31S",
        "31S",
        "31S",
        "31S",
        "31S",
        "31S",
    ],
    &[
        "31S",
        "11S9_11S",
        "11S4_H4_11S",
        "11Sg7_g11S",
        "11S9_11S",
        "11S9_11S",
        "11S9_11S",
        "14S3_14S",
        "7S17_7S",
        "7S3_S3_S3_S3_S_7S",
        "7S17_7S",
        "7S17_7S",
        "7S3_S3_S3_S3_S_7S",
        "7S17_7S",
        "14S3_14S",
        "S6_2S13_2S6_S",
        "S6_2S13_2S6_S",
        "S29_S",
        "Sg27_gS",
        "S29_S",
        "S6_2S13_2S6_S",
        "S6_2S13_2S6_S",
        "14S3_14S",
        "14S3_14S",
        "14S3_14S",
        "31S",
        "16ST14S",
        "31S",
        "31S",
        "14ST16S",
        "31S",
    ],
    &[
        "31S",
        "11S9_11S",
        "11S9_11S",
        "11S9_11S",
        "11S9_11S",
        "11S9_11S",
        "11S9_11S",
        "14S3_14S",
        "7S_g3_g3_g3_g3_7S",
        "7S3_S3_S3_S3_S_7S",
        "7S17_7S",
        "7S17_7S",
        "7S3_S3_S3_S3_S_7S",
        "7S_g3_g3_g3_g3_7S",
        "14S3_14S",
        "S6_2S13_2S6_S",
        "S6_2S13_2S6_S",
        "S29_S",
        "S29_S",
        "S29_S",
        "S6_2S13_2S6_S",
        "S6_2S13_2S6_S",
        "14S3_14S",
        "14S3_14S",
        "14S3_14S",
        "14S3_14S",
        "14S3_14S",
        "31S",
        "31S",
        "31S",
        "31S",
    ],
    &[
        "31S", "31S", "31S", "15Sg15S", "31S", "31S", "31S", "31S", "31S", "31S", "31S", "31S",
        "31S", "31S", "31S", "31S", "31S", "31S", "31S", "31S", "31S", "31S", "31S", "14S3_14S",
        "14S3_14S", "14S3_14S", "14S3_14S", "14S3_14S", "14S3_14S", "31S", "31S",
    ],
    &[
        "15.S15.",
        "10.11S10.",
        "8.15S8.",
        "6.19S6.",
        "5.21S5.",
        "4.23S4.",
        "3.25S3.",
        "3.25S3.",
        "2.27S2.",
        "2.27S2.",
        ".29S.",
        ".29S.",
        ".29S.",
        ".29S.",
        ".29S.",
        "31S",
        ".29S.",
        ".29S.",
        ".29S.",
        ".29S.",
        ".29S.",
        "2.27S2.",
        "2.27S2.",
        "3.25S3.",
        "3.25S3.",
        "4.10S3_10S4.",
        "5.9S3_9S5.",
        "6.8S3_8S6.",
        "8.6S3_6S8.",
        "10.4S3_4S10.",
        "14.3_14.",
    ],
    &[
        "31.",
        "31.",
        "15.S15.",
        "10.11S10.",
        "9.13S9.",
        "7.17S7.",
        "6.19S6.",
        "5.21S5.",
        "5.21S5.",
        "4.23S4.",
        "3.25S3.",
        "3.25S3.",
        "3.25S3.",
        "3.25S3.",
        "3.25S3.",
        "2.27S2.",
        "3.25S3.",
        "3.25S3.",
        "3.25S3.",
        "3.25S3.",
        "3.25S3.",
        "4.23S4.",
        "5.21S5.",
        "5.21S5.",
        "6.19S6.",
        "7.17S7.",
        "9.13S9.",
        "10.4S3_4S10.",
        "14.3_14.",
        "14.3_14.",
        "14.3_14.",
    ],
    &[
        "31.", "31.", "31.", "31.", "15.S15.", "11.9S11.", "9.13S9.", "8.15S8.", "7.17S7.",
        "6.19S6.", "6.19S6.", "5.21S5.", "5.21S5.", "5.21S5.", "5.21S5.", "4.23S4.", "5.21S5.",
        "5.21S5.", "5.21S5.", "5.21S5.", "6.19S6.", "6.19S6.", "7.17S7.", "8.15S8.", "9.13S9.",
        "11.9S11.", "15.S15.", "31.", "31.", "14.3_14.", "14.3_14.",
    ],
    &[
        "31.",
        "31.",
        "31.",
        "31.",
        "31.",
        "31.",
        "15.S15.",
        "11.9S11.",
        "10.11S10.",
        "9.13S9.",
        "8.15S8.",
        "7.17S7.",
        "7.17S7.",
        "7.17S7.",
        "7.17S7.",
        "6.19S6.",
        "7.17S7.",
        "7.17S7.",
        "7.17S7.",
        "7.17S7.",
        "8.15S8.",
        "9.13S9.",
        "10.11S10.",
        "11.9S11.",
        "15.S15.",
        "31.",
        "31.",
        "31.",
        "31.",
        "31.",
        "31.",
    ],
    &[
        "31.",
        "31.",
        "31.",
        "31.",
        "31.",
        "31.",
        "31.",
        "31.",
        "15.S15.",
        "12.7S12.",
        "11.9S11.",
        "10.11S10.",
        "9.13S9.",
        "9.13S9.",
        "9.13S9.",
        "8.15S8.",
        "9.13S9.",
        "9.13S9.",
        "9.13S9.",
        "10.11S10.",
        "11.9S11.",
        "12.7S12.",
        "15.S15.",
        "31.",
        "31.",
        "31.",
        "31.",
        "31.",
        "31.",
        "31.",
        "31.",
    ],
    &[
        "31.",
        "31.",
        "31.",
        "31.",
        "31.",
        "31.",
        "31.",
        "31.",
        "31.",
        "31.",
        "15.S15.",
        "12.7S12.",
        "11.9S11.",
        "11.9S11.",
        "11.9S11.",
        "10.11S10.",
        "11.9S11.",
        "11.9S11.",
        "11.9S11.",
        "12.7S12.",
        "15.S15.",
        "31.",
        "31.",
        "31.",
        "31.",
        "31.",
        "31.",
        "31.",
        "31.",
        "31.",
        "31.",
    ],
    &[
        "31.", "31.", "31.", "31.", "31.", "31.", "31.", "31.", "31.", "31.", "31.", "31.",
        "15.S15.", "13.5S13.", "13.5S13.", "12.7S12.", "13.5S13.", "13.5S13.", "15.S15.", "31.",
        "31.", "31.", "31.", "31.", "31.", "31.", "31.", "31.", "31.", "31.", "31.",
    ],
];

const BARROW_PLAQUES: [Plaque; 6] = [
    plaque(
        15,
        28,
        5,
        "Rope, knotted to the post. Nobody untied it from the other end.",
    ),
    plaque(
        10,
        15,
        1,
        "Six bedrolls laid out along the wall. None of them slept in. The fire burned down on its own.",
    ),
    plaque(
        20,
        21,
        1,
        "Scratched into the stone: DAY 3. ALDIS HEARD THE COUNTING AGAIN. ORRIN SAYS IT IS WATER.",
    ),
    plaque(
        7,
        8,
        1,
        "Tally marks cover the wall from the floor to the roof. Somebody counted to six, again and again, and started over.",
    ),
    plaque(
        23,
        13,
        1,
        "A watch lantern, still full of oil, set down and never lit.",
    ),
    plaque(
        11,
        1,
        1,
        "Six stone slabs worn smooth. Six shapes pressed in the dust, and six sets of footprints walking out.",
    ),
];

const CAMP_HOLLOW: &[&[&str]] = &[
    &[
        "21,", "21,", "21,", "21,", "21,", "21,", "21,", "21,", "21,", "21,", "21,", "21,", "21,",
        "21,", "21,", "21,", "21,", "21,", "21,", "21,", "21,",
    ],
    &[
        "21.",
        "21.",
        "2.5w7.5w2.",
        "2.w15.w2.",
        "2.w.H13.w2.",
        "2.w15.w2.",
        "2.5w7.3wew2.",
        "10.L10.",
        "12.T8.",
        "5.T15.",
        ".e5.L2.F2.L5.e.",
        "21.",
        "8.T6.T5.",
        "10.L10.",
        "2.w3.w14.",
        "2.w3.w7.6L.",
        "2.w3.w14.",
        "2.w3.w9.H4.",
        "2.4we14.",
        "10.e10.",
        "21.",
    ],
    &[
        "21.",
        "21.",
        "2.5w7.5w2.",
        "2.w15.w2.",
        "2.w15.w2.",
        "2.w15.w2.",
        "2.5w7.5w2.",
        "21.",
        "21.",
        "21.",
        "21.",
        "21.",
        "21.",
        "21.",
        "2.w3.w14.",
        "2.w3.w14.",
        "2.w3.w7.6L.",
        "2.w3.w14.",
        "2.5w14.",
        "21.",
        "21.",
    ],
    &[
        "21.",
        "21.",
        "21.",
        "3.3w9.3w3.",
        "3.3w9.3w3.",
        "3.3w9.3w3.",
        "21.",
        "21.",
        "21.",
        "21.",
        "21.",
        "21.",
        "21.",
        "21.",
        "21.",
        "3.3w15.",
        "3.3w15.",
        "3.3w8.6L.",
        "14.6L.",
        "21.",
        "21.",
    ],
    &[
        "21.", "21.", "21.", "21.", "21.", "21.", "21.", "21.", "21.", "21.", "21.", "21.", "21.",
        "21.", "21.", "21.", "21.", "21.", "21.", "21.", "21.",
    ],
];

const CAMP_HOLLOW_PLAQUES: [Plaque; 3] = [
    plaque(
        9,
        12,
        1,
        "The watch roll, pinned under a stone. Six names. Five have a line through them, drawn in something that is not ink.",
    ),
    plaque(
        5,
        6,
        1,
        "Two packs, opened and packed again. Everything is here except the rope.",
    ),
    plaque(
        15,
        14,
        1,
        "A lean-to of fresh logs. Somebody built it this week, and built it well.",
    ),
];

const STONEHOUSE: &[&[&str]] = &[
    &["9C", "9C", "9C", "9C", "9C", "9C", "9C", "9C", "9C"],
    &[
        "C3SD3SC", "S7.S", "S7.S", "S3.T3.S", "S7.S", "S.K3.H.S", "S7.S", "S7.S", "C7SC",
    ],
    &[
        "C3SD3SC", "S7.S", "G7.G", "S7.S", "G7.G", "S7.S", "G7.G", "S7.S", "CSG3SGSC",
    ],
    &[
        "C7SC", "S7.S", "S7.S", "S7.S", "S7.S", "S7.S", "S7.S", "S7.S", "C7SC",
    ],
    &[
        "9u", "u7Cu", "u7Cu", "u7Cu", "u7Cu", "u7Cu", "u7Cu", "u7Cu", "9u",
    ],
    &[
        "9.", ".7u.", ".u5Cu.", ".u5Cu.", ".u5Cu.", ".u5Cu.", ".u5Cu.", ".7u.", "9.",
    ],
    &["C8.", "9.", "9.", "9.", "9.", "9.", "9.", "9.", "9."],
];

const REDROOF: &[&[&str]] = &[
    &["7C", "7C", "7C", "7C", "7C", "7C", "7C"],
    &[
        "B2CD2CB", "C5.C", "C5.C", "C2.T2.C", "C.H.K.C", "C5.C", "B5CB",
    ],
    &["B2CD2CB", "C5.C", "G5.G", "C5.C", "G5.G", "C5.C", "BCG3CB"],
    &["B5CB", "C5.C", "C5.C", "C5.C", "C5.C", "C5.C", "B5CB"],
    &["7W", "W5CW", "W5CW", "W5CW", "W5CW", "W5CW", "7W"],
    &[".5W.", ".5W.", ".5W.", ".5W.", ".5W.", ".5W.", ".5W."],
    &["C6.", "7.", "7.", "7.", "7.", "7.", "7."],
];

const TALLHOUSE: &[&[&str]] = &[
    &["7P", "7P", "7P", "7P", "7P", "7P", "7P"],
    &[
        "L2PD2PL", "P5.P", "P5.P", "P2.T2.P", "P.K3.P", "P5.P", "L5PL",
    ],
    &["L2PD2PL", "P5.P", "G5.G", "P5.P", "G5.G", "P5.P", "LPG3PL"],
    &["L5PL", "P5.P", "P5.P", "P5.P", "P5.P", "P5.P", "L5PL"],
    &["7P", "P5.P", "P5.P", "P5.P", "P5.P", "P5.P", "7P"],
    &["L5PL", "P5.P", "G5.G", "P.H3.P", "G5.G", "P5.P", "LPG3PL"],
    &["L5PL", "P5.P", "P5.P", "P5.P", "P5.P", "P5.P", "L5PL"],
    &["7w", "w5Pw", "w5Pw", "w5Pw", "w5Pw", "w5Pw", "7w"],
    &[".5w.", ".5w.", ".5w.", ".5w.", ".5w.", ".5w.", ".5w."],
    &["C6.", "7.", "7.", "7.", "7.", "7.", "7."],
];

pub const SCHEMATICS: [Schematic; 31] = [
    schematic(SchematicName::Cottage, 7, 0, true, COTTAGE, &[]),
    schematic(SchematicName::House, 9, 0, true, HOUSE, &[]),
    schematic(SchematicName::Lodge, 9, 0, true, LODGE, &[]),
    schematic(SchematicName::Townhall, 11, 0, true, TOWNHALL, &[]),
    schematic(SchematicName::Well, 3, 0, true, WELL, &[]),
    schematic(SchematicName::Market, 5, 0, true, MARKET, &[]),
    schematic(SchematicName::Farm, 7, 0, true, FARM, &[]),
    schematic(SchematicName::Oak, 5, -1, false, OAK, &[]),
    schematic(SchematicName::OakTall, 5, -1, false, OAK_TALL, &[]),
    schematic(SchematicName::Pine, 5, -1, false, PINE, &[]),
    schematic(SchematicName::FlowerRed, 1, -1, false, FLOWER_RED, &[]),
    schematic(
        SchematicName::FlowerYellow,
        1,
        -1,
        false,
        FLOWER_YELLOW,
        &[],
    ),
    schematic(SchematicName::Deadbush, 1, -1, false, DEADBUSH, &[]),
    schematic(SchematicName::Boulder, 3, 0, false, BOULDER, &[]),
    schematic(SchematicName::Tower, 5, 0, true, TOWER, &[]),
    schematic(SchematicName::Obelisk, 1, -1, false, OBELISK, &[]),
    schematic(SchematicName::Ruin, 7, 0, true, RUIN, &[]),
    schematic(SchematicName::Henge, 7, 0, true, HENGE, &[]),
    schematic(SchematicName::Lamp, 1, 0, false, LAMP, &[]),
    schematic(SchematicName::Watchtower, 5, 0, true, WATCHTOWER, &[]),
    schematic(SchematicName::Graveyard, 7, 0, true, GRAVEYARD, &[]),
    schematic(SchematicName::Campfire, 5, -1, true, CAMP, &[]),
    schematic(SchematicName::Ship, 9, 0, false, SHIP, &[]),
    schematic(SchematicName::Lighthouse, 5, 0, true, LIGHTHOUSE, &[]),
    schematic(SchematicName::Shrine, 5, 0, true, SHRINE, &[]),
    schematic(SchematicName::Keep, 15, 0, true, KEEP, &KEEP_PLAQUES),
    schematic(SchematicName::Barrow, 31, 4, true, BARROW, &BARROW_PLAQUES),
    schematic(
        SchematicName::Camp,
        21,
        0,
        true,
        CAMP_HOLLOW,
        &CAMP_HOLLOW_PLAQUES,
    ),
    schematic(SchematicName::Stonehouse, 9, 0, true, STONEHOUSE, &[]),
    schematic(SchematicName::Redroof, 7, 0, true, REDROOF, &[]),
    schematic(SchematicName::Tallhouse, 7, 0, true, TALLHOUSE, &[]),
];

pub const BARROW_SCHEMATIC: usize = SchematicName::Barrow as usize;
pub(super) const STONEHOUSE_SCHEMATIC: usize = SchematicName::Stonehouse as usize;
pub(super) const REDROOF_SCHEMATIC: usize = SchematicName::Redroof as usize;
pub(super) const TALLHOUSE_SCHEMATIC: usize = SchematicName::Tallhouse as usize;
pub const CAMP_SCHEMATIC: usize = SchematicName::Camp as usize;
pub const OAK_SCHEMATIC: usize = SchematicName::Oak as usize;
pub(super) const COTTAGE_SCHEMATIC: usize = SchematicName::Cottage as usize;
pub(super) const HOUSE_SCHEMATIC: usize = SchematicName::House as usize;
pub(super) const LODGE_SCHEMATIC: usize = SchematicName::Lodge as usize;
pub(super) const TOWNHALL_SCHEMATIC: usize = SchematicName::Townhall as usize;
pub(super) const WELL_SCHEMATIC: usize = SchematicName::Well as usize;
pub(super) const MARKET_SCHEMATIC: usize = SchematicName::Market as usize;
pub(super) const FARM_SCHEMATIC: usize = SchematicName::Farm as usize;
pub(super) const OAK_TALL_SCHEMATIC: usize = SchematicName::OakTall as usize;
pub(super) const PINE_SCHEMATIC: usize = SchematicName::Pine as usize;
pub(super) const FLOWER_RED_SCHEMATIC: usize = SchematicName::FlowerRed as usize;
pub(super) const FLOWER_YELLOW_SCHEMATIC: usize = SchematicName::FlowerYellow as usize;
pub(super) const DEADBUSH_SCHEMATIC: usize = SchematicName::Deadbush as usize;
pub(super) const BOULDER_SCHEMATIC: usize = SchematicName::Boulder as usize;
pub(super) const TOWER_SCHEMATIC: usize = SchematicName::Tower as usize;
pub(super) const OBELISK_SCHEMATIC: usize = SchematicName::Obelisk as usize;
pub(super) const RUIN_SCHEMATIC: usize = SchematicName::Ruin as usize;
pub(super) const HENGE_SCHEMATIC: usize = SchematicName::Henge as usize;
pub(super) const LAMP_SCHEMATIC: usize = SchematicName::Lamp as usize;
pub(super) const WATCHTOWER_SCHEMATIC: usize = SchematicName::Watchtower as usize;
pub(super) const GRAVEYARD_SCHEMATIC: usize = SchematicName::Graveyard as usize;
pub(super) const CAMPFIRE_SCHEMATIC: usize = SchematicName::Campfire as usize;
pub(super) const SHIP_SCHEMATIC: usize = SchematicName::Ship as usize;
pub(super) const LIGHTHOUSE_SCHEMATIC: usize = SchematicName::Lighthouse as usize;
pub(super) const SHRINE_SCHEMATIC: usize = SchematicName::Shrine as usize;
pub const KEEP_SCHEMATIC: usize = SchematicName::Keep as usize;

const fn schematics_line_up() -> bool {
    let mut index = 0;
    while index < SCHEMATICS.len() {
        if SCHEMATICS[index].name as usize != index {
            return false;
        }
        index += 1;
    }
    true
}

const _: () = assert!(schematics_line_up());
