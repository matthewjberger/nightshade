#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ToolKind {
    Pickaxe,
    Shovel,
    Axe,
    Sword,
    Glider,
}

impl ToolKind {
    pub fn index(self) -> u8 {
        self as u8
    }

    pub fn from_index(index: u8) -> Option<ToolKind> {
        TOOLS.get(index as usize).map(|row| row.kind)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ToolInfo {
    pub kind: ToolKind,
    pub name: &'static str,
    pub speed: f32,
    pub art: [&'static str; 16],
}

pub const TOOLS: [ToolInfo; 5] = [
    ToolInfo {
        kind: ToolKind::Pickaxe,
        name: "Pickaxe",
        speed: 6.0,
        art: [
            "................",
            "...iIIIIIIi.....",
            ".iII......IIi...",
            "iId...bB...dIi..",
            "d.....bB.....d..",
            "......bB........",
            "......bB........",
            "......bB........",
            "......bB........",
            "......bB........",
            "......bB........",
            "......bB........",
            "......bB........",
            "......BB........",
            "................",
            "................",
        ],
    },
    ToolInfo {
        kind: ToolKind::Shovel,
        name: "Shovel",
        speed: 7.0,
        art: [
            "................",
            "................",
            "................",
            ".....dIId.......",
            "....IIIIII......",
            "...iIIIIIIi.....",
            "...iIIIIIIi.....",
            "...iIIIIIIi.....",
            "...iIIIIIIi.....",
            "....ddIIdd......",
            "......bB........",
            "......bB........",
            "......bB........",
            "......bB........",
            "......bB........",
            "................",
        ],
    },
    ToolInfo {
        kind: ToolKind::Axe,
        name: "Axe",
        speed: 5.0,
        art: [
            "................",
            "......bB........",
            "......bBdd......",
            "......bBdIIi....",
            "......bBdIIIi...",
            "......bBdIIIIi..",
            "......bBdIIIIi..",
            "......bBdIIIi...",
            "......bBdIi.....",
            "......bB........",
            "......bB........",
            "......bB........",
            "......bB........",
            "......BB........",
            "................",
            "................",
        ],
    },
    ToolInfo {
        kind: ToolKind::Sword,
        name: "Sword",
        speed: 2.0,
        art: [
            ".......i........",
            "......iIi.......",
            ".....iIIId......",
            ".....iIIId......",
            ".....iIIId......",
            ".....iIIId......",
            ".....iIIId......",
            ".....iIIId......",
            ".....iIIId......",
            ".....iIIId......",
            ".....iIIId......",
            "...yyyyyyyyy....",
            "......BbB.......",
            "......BbB.......",
            ".....yYYYy......",
            "................",
        ],
    },
    ToolInfo {
        kind: ToolKind::Glider,
        name: "Hang Glider",
        speed: 1.0,
        art: [
            "................",
            "................",
            ".dwwwwwwwwwwwwd.",
            "..dwwwwwwwwwwd..",
            "...dwwwwwwwwd...",
            "....dwwwwwwd....",
            ".....dwwwwd.....",
            "......dwwd......",
            ".......dd.......",
            ".......BB.......",
            "......uuuu......",
            ".......uu.......",
            "......u..u......",
            "................",
            "................",
            "................",
        ],
    },
];

impl ToolKind {
    pub fn name(self) -> &'static str {
        TOOLS[self as usize].name
    }

    pub fn speed(self) -> f32 {
        TOOLS[self as usize].speed
    }

    pub fn art(self) -> [&'static str; 16] {
        TOOLS[self as usize].art
    }
}

const fn tools_line_up() -> bool {
    let mut index = 0;
    while index < TOOLS.len() {
        if TOOLS[index].kind as usize != index {
            return false;
        }
        index += 1;
    }
    true
}

const _: () = assert!(
    tools_line_up(),
    "a TOOLS row does not sit at the index of its ToolKind"
);
