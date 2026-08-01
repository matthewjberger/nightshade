use nalgebra_glm::{Vec2, Vec4};

pub const TITLE: &str = "Roundwood";
pub const TAGLINE: &str = "A village on a world that turns";
pub const PAUSED: &str = "Paused";

pub const ATLAS: &str = "roundwood";
pub const SHEET: &str = "tiny_farm";
pub const SHADOW: &str = "shadow";
pub const ATLAS_SIZE: u32 = 512;
pub const SHEET_BYTES: &[u8] = include_bytes!("../assets/tiny_farm/tilemap_packed.png");
pub const SHEET_COLUMNS: u32 = 12;
pub const SHEET_ROWS: u32 = 11;
pub const SHEET_TILES: usize = (SHEET_COLUMNS * SHEET_ROWS) as usize;
pub const TILE_TEXELS: u32 = 16;
pub const BLOCK_TEXELS: u32 = 16;

pub const ACROSS: u32 = 34;
pub const AROUND: u32 = 96;
pub const CIRCUMFERENCE: f32 = AROUND as f32;
pub const RADIUS: f32 = CIRCUMFERENCE / std::f32::consts::TAU;
pub const STEP_ANGLE: f32 = std::f32::consts::TAU / AROUND as f32;
pub const CAP_THICKNESS: f32 = 0.2;
pub const SEA_STRIPS: [f32; 9] = [2.0, 3.0, 4.0, 6.0, 8.0, 10.0, 13.0, 16.0, 20.0];
pub const SEA_LEVEL: f32 = 1.55;
pub const WATER_DEPTH: f32 = 0.5;

pub const WATER_HEIGHT: u32 = 1;
pub const LAND_HEIGHT: u32 = 2;

pub const WEST_SEA_END: u32 = 5;
pub const WEST_BEACH_END: u32 = 8;
pub const EAST_BEACH_START: u32 = 26;
pub const EAST_SEA_START: u32 = 29;

pub const ROAD_START: u32 = 15;
pub const ROAD_END: u32 = 18;
pub const RIVER_START: u32 = 58;
pub const RIVER_END: u32 = 61;

pub const PLAZA_COLUMN_START: u32 = 10;
pub const PLAZA_COLUMN_END: u32 = 15;
pub const PLAZA_ROW_START: u32 = 19;
pub const PLAZA_ROW_END: u32 = 27;

pub const POND_COLUMN: f32 = 22.5;
pub const POND_ROW: f32 = 40.5;
pub const POND_RADIUS: f32 = 3.4;
pub const POND_DECK_COLUMN_START: u32 = 18;
pub const POND_DECK_COLUMN_END: u32 = 20;
pub const POND_DECK_ROW: u32 = 40;

pub const DOCK_COLUMN_START: u32 = 26;
pub const DOCK_COLUMN_END: u32 = 33;
pub const DOCK_ROW_START: u32 = 33;
pub const DOCK_ROW_END: u32 = 35;

pub const HILL_COLUMN: f32 = 21.0;
pub const HILL_ROW: f32 = 74.0;
pub const HILL_RADIUS: f32 = 7.0;
pub const HILL_STEP: f32 = 2.4;
pub const HILL_RISE: f32 = 2.0;

pub const PLACES: [(f32, &str); 6] = [
    (10.0, "Pim's lane"),
    (23.0, "the plaza"),
    (40.0, "the pond"),
    (59.0, "the bridge"),
    (74.0, "the east hill"),
    (86.0, "the quiet mile"),
];
pub const WEST_STRAND: &str = "the west strand";
pub const EAST_STRAND: &str = "the east strand";
pub const THE_DOCK: &str = "the dock";

pub const START_COLUMN: f32 = 16.5;
pub const START_ROW: f32 = 30.5;

pub const GRASS_TEXTURE: &str = "roundwood_grass";
pub const SAND_TEXTURE: &str = "roundwood_sand";
pub const DIRT_TEXTURE: &str = "roundwood_dirt";
pub const PATH_TEXTURE: &str = "roundwood_path";
pub const PLANK_TEXTURE: &str = "roundwood_plank";
pub const PLASTER_TEXTURE: &str = "roundwood_plaster";
pub const ROOF_RED_TEXTURE: &str = "roundwood_roof_red";
pub const ROOF_BLUE_TEXTURE: &str = "roundwood_roof_blue";
pub const ROOF_GREEN_TEXTURE: &str = "roundwood_roof_green";
pub const DOOR_TEXTURE: &str = "roundwood_door";
pub const WINDOW_TEXTURE: &str = "roundwood_window";
pub const AWNING_TEXTURE: &str = "roundwood_awning";

pub const GRASS_MATERIAL: &str = "roundwood_grass";
pub const SAND_MATERIAL: &str = "roundwood_sand";
pub const DIRT_MATERIAL: &str = "roundwood_dirt";
pub const PATH_MATERIAL: &str = "roundwood_path";
pub const PLANK_MATERIAL: &str = "roundwood_plank";
pub const PLASTER_MATERIAL: &str = "roundwood_plaster";
pub const ROOF_RED_MATERIAL: &str = "roundwood_roof_red";
pub const ROOF_BLUE_MATERIAL: &str = "roundwood_roof_blue";
pub const ROOF_GREEN_MATERIAL: &str = "roundwood_roof_green";
pub const DOOR_MATERIAL: &str = "roundwood_door";
pub const WINDOW_MATERIAL: &str = "roundwood_window";
pub const AWNING_MATERIAL: &str = "roundwood_awning";
pub const WATER_MATERIAL: &str = "roundwood_water";
pub const TIMBER_MATERIAL: &str = "roundwood_timber";
pub const LAMP_MATERIAL: &str = "roundwood_lamp";

pub const GRASS_COLOR: [u8; 4] = [110, 176, 92, 255];
pub const GRASS_DARK: [u8; 4] = [92, 156, 78, 255];
pub const GRASS_LIGHT: [u8; 4] = [134, 196, 108, 255];
pub const SAND_COLOR: [u8; 4] = [230, 210, 162, 255];
pub const SAND_DARK: [u8; 4] = [210, 188, 142, 255];
pub const SAND_LIGHT: [u8; 4] = [244, 230, 190, 255];
pub const DIRT_COLOR: [u8; 4] = [138, 104, 74, 255];
pub const DIRT_DARK: [u8; 4] = [114, 84, 58, 255];
pub const DIRT_LIGHT: [u8; 4] = [158, 122, 90, 255];
pub const PATH_COLOR: [u8; 4] = [188, 162, 126, 255];
pub const PATH_DARK: [u8; 4] = [162, 136, 104, 255];
pub const PATH_LIGHT: [u8; 4] = [206, 184, 152, 255];
pub const PLANK_COLOR: [u8; 4] = [166, 122, 78, 255];
pub const PLANK_DARK: [u8; 4] = [134, 96, 58, 255];
pub const PLANK_SEAM: [u8; 4] = [104, 72, 44, 255];
pub const PLASTER_COLOR: [u8; 4] = [238, 228, 208, 255];
pub const PLASTER_DARK: [u8; 4] = [216, 204, 184, 255];
pub const PLASTER_LIGHT: [u8; 4] = [250, 244, 230, 255];
pub const ROOF_RED_COLOR: [u8; 4] = [188, 86, 74, 255];
pub const ROOF_RED_DARK: [u8; 4] = [150, 62, 54, 255];
pub const ROOF_BLUE_COLOR: [u8; 4] = [94, 128, 180, 255];
pub const ROOF_BLUE_DARK: [u8; 4] = [72, 100, 148, 255];
pub const ROOF_GREEN_COLOR: [u8; 4] = [98, 148, 108, 255];
pub const ROOF_GREEN_DARK: [u8; 4] = [74, 118, 86, 255];
pub const DOOR_COLOR: [u8; 4] = [142, 96, 62, 255];
pub const DOOR_DARK: [u8; 4] = [96, 62, 40, 255];
pub const DOOR_KNOB: [u8; 4] = [232, 200, 108, 255];
pub const WINDOW_FRAME: [u8; 4] = [116, 84, 56, 255];
pub const WINDOW_GLASS: [u8; 4] = [138, 186, 208, 255];
pub const WINDOW_GLASS_LIGHT: [u8; 4] = [196, 226, 238, 255];
pub const AWNING_COLOR: [u8; 4] = [232, 226, 214, 255];
pub const AWNING_STRIPE: [u8; 4] = [206, 92, 84, 255];

pub const TIMBER_COLOR: [f32; 4] = [0.42, 0.3, 0.2, 1.0];
pub const LAMP_COLOR: [f32; 4] = [1.0, 0.86, 0.6, 1.0];
pub const LAMP_GLOW: [f32; 3] = [1.6, 1.2, 0.6];
pub const WATER_COLOR: [f32; 4] = [0.24, 0.54, 0.74, 0.76];

pub const PINE_TOP: u32 = 3;
pub const PINE_BOTTOM: u32 = 15;
pub const BARE_TREE: [u32; 2] = [2, 14];
pub const SMALL_TREE: u32 = 27;
pub const BUSH: u32 = 39;
pub const BERRY_BUSH: u32 = 78;
pub const FLOWER: u32 = 83;
pub const GRASS_TUFT: u32 = 80;
pub const ROCKS: u32 = 77;
pub const ROCK_PILE: u32 = 89;
pub const HAY: u32 = 96;
pub const HAY_BALE: u32 = 97;
pub const BARREL: u32 = 85;
pub const SACK: u32 = 74;
pub const BASKET: u32 = 75;
pub const CRATE: u32 = 76;
pub const PLAYER_TILE: u32 = 108;
pub const CHICKEN_TILE: u32 = 122;
pub const SHEEP_TILE: u32 = 120;
pub const COW_TILE: u32 = 121;

pub const BERRY_TILE: u32 = 44;
pub const MUSHROOM_TILE: u32 = 79;
pub const WOOD_TILE: u32 = 26;
pub const STONE_TILE: u32 = 77;

pub const VILLAGER_TEXELS: u32 = 16;

pub const SHELL_SHAPE: [&str; 16] = [
    "................",
    "................",
    "................",
    "................",
    "................",
    "......oooo......",
    ".....ooFFoo.....",
    "....oFfFFfFo....",
    "...oFfFffFfFo...",
    "..oFfFfFfFfFFo..",
    "..oFFfFfFfFFFo..",
    "...oFFFFFFFFo...",
    "....oooooooo....",
    "................",
    "................",
    "................",
];

pub const FISH_SHAPE: [&str; 16] = [
    "................",
    "................",
    "................",
    "................",
    "......oooo......",
    "....ooFFFFoo..oo",
    "...oFFFFFFFFooFo",
    "..oFwFFFFFFFFFFo",
    "..oFweFFFFFFFFFo",
    "..oFwFFFFFFFFoFo",
    "...oFFFFFFFFoooo",
    "....ooFFFFoo....",
    "......oooo......",
    "................",
    "................",
    "................",
];

pub const SHELL_COAT: [u8; 4] = [244, 212, 206, 255];
pub const SHELL_LIGHT: [u8; 4] = [255, 240, 236, 255];
pub const FISH_COAT: [u8; 4] = [126, 178, 214, 255];
pub const FISH_LIGHT: [u8; 4] = [206, 232, 244, 255];

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Item {
    #[default]
    Berry,
    Mushroom,
    Shell,
    Fish,
    Wood,
    Stone,
}

impl Item {
    pub const ALL: [Item; 6] = [
        Item::Berry,
        Item::Mushroom,
        Item::Shell,
        Item::Fish,
        Item::Wood,
        Item::Stone,
    ];

    pub fn index(self) -> usize {
        match self {
            Item::Berry => 0,
            Item::Mushroom => 1,
            Item::Shell => 2,
            Item::Fish => 3,
            Item::Wood => 4,
            Item::Stone => 5,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Item::Berry => "berries",
            Item::Mushroom => "mushrooms",
            Item::Shell => "shells",
            Item::Fish => "fish",
            Item::Wood => "wood",
            Item::Stone => "stones",
        }
    }

    pub fn price(self) -> u32 {
        match self {
            Item::Berry => 12,
            Item::Mushroom => 18,
            Item::Shell => 25,
            Item::Fish => 40,
            Item::Wood => 6,
            Item::Stone => 8,
        }
    }
}

pub const PICKUP_REACH: f32 = 0.9;
pub const PICKUP_HOP: f32 = 0.55;
pub const PICKUP_HOP_RATE: f32 = 3.4;
pub const PICKUP_SCALE: f32 = 0.55;
pub const ACT_REACH: f32 = 1.9;
pub const SHAKE_TIME: f32 = 0.6;
pub const SHAKE_LEAN: f32 = 0.16;
pub const BERRY_YIELD: u32 = 2;
pub const BERRY_RIPEN: f32 = 90.0;
pub const WOOD_CHANCE: u32 = 55;
pub const TREE_REST: f32 = 40.0;
pub const ROCK_REST: f32 = 50.0;
pub const SHELL_COUNT: u32 = 16;
pub const SHELL_RETURN: f32 = 30.0;
pub const MUSHROOM_COUNT: u32 = 14;

pub const CAST_RANGE: f32 = 2.6;
pub const BITE_WAIT_MIN: f32 = 2.0;
pub const BITE_WAIT_MAX: f32 = 7.0;
pub const BITE_WINDOW: f32 = 1.1;
pub const BOBBER_TILE: u32 = 72;

pub const TOAST_TIME: f32 = 2.8;

pub const CAT_SHAPE: [&str; 16] = [
    "................",
    "..oo........oo..",
    ".oFo........oFo.",
    ".oFFooooooooFFo.",
    ".oFFFFFFFFFFFFo.",
    ".oFFFFFFFFFFFFo.",
    ".oFweFFFFFFweFo.",
    ".oFFFFFnnFFFFFo.",
    ".oFFFFFffFFFFFo.",
    "..oFFFFFFFFFFo..",
    "...oooooooooo...",
    "...osssssssso...",
    "..oFssssssssFo..",
    "..oFssssssssFo..",
    "...osssssssso...",
    "....obb..bbo....",
];

pub const DOG_SHAPE: [&str; 16] = [
    "................",
    "................",
    "...oooooooooo...",
    "..oFFFFFFFFFFo..",
    ".ooFFFFFFFFFFoo.",
    "oFFoFFFFFFFFoFFo",
    "oFFoFweFFweFoFFo",
    "oFFoFFFnnFFFoFFo",
    ".ooFFFFffFFFFoo.",
    "..oFFFFFFFFFFo..",
    "...oooooooooo...",
    "...osssssssso...",
    "..oFssssssssFo..",
    "..oFssssssssFo..",
    "...osssssssso...",
    "....obb..bbo....",
];

pub const BIRD_SHAPE: [&str; 16] = [
    "................",
    ".......ff.......",
    "......ofFo......",
    "...ooooFFoooo...",
    "..oFFFFFFFFFFo..",
    "..oFFFFFFFFFFo..",
    "..oFweFFFFweFo..",
    "..oFFFnnnnFFFo..",
    "..oFFFFnnFFFFo..",
    "...oFFFFFFFFo...",
    "...oooooooooo...",
    "...osssssssso...",
    "..oFssssssssFo..",
    "..oFssssssssFo..",
    "...osssssssso...",
    "....onn..nno....",
];

pub const BEAR_SHAPE: [&str; 16] = [
    "................",
    "..ooo......ooo..",
    ".oFFFo....oFFFo.",
    ".oFffFooooFffFo.",
    "..oFFFFFFFFFFo..",
    ".oFFFFFFFFFFFFo.",
    ".oFweFFFFFFweFo.",
    ".oFFFFFnnFFFFFo.",
    ".oFFFFffffFFFFo.",
    "..oFFFFFFFFFFo..",
    "...oooooooooo...",
    "...osssssssso...",
    "..oFssssssssFo..",
    "..oFssssssssFo..",
    "...osssssssso...",
    "....obb..bbo....",
];

pub const VILLAGER_OUTLINE: [u8; 4] = [48, 36, 42, 255];
pub const VILLAGER_EYE: [u8; 4] = [48, 36, 42, 255];
pub const VILLAGER_WHITE: [u8; 4] = [250, 250, 246, 255];
pub const VILLAGER_NOSE: [u8; 4] = [232, 142, 118, 255];
pub const VILLAGER_BOOT: [u8; 4] = [92, 68, 58, 255];

pub const LAYER_SHADOW: i32 = 0;
pub const LAYER_STANDING: i32 = 1;
pub const SHADOW_ALPHA: f32 = 0.3;
pub const SHADOW_LIFT: f32 = 0.03;
pub const SHADOW_FORWARD: f32 = 0.14;

pub const WALK_SPEED: f32 = 4.4;
pub const STRIDE_RATE: f32 = 11.0;
pub const STRIDE_BOUNCE: f32 = 0.09;
pub const STEP_SMOOTHING: f32 = 14.0;
pub const NEIGHBOR_SPEED: f32 = 1.5;
pub const NEIGHBOR_REACH: f32 = 4.0;
pub const NEIGHBOR_REST: f32 = 2.6;
pub const ANIMAL_SPEED: f32 = 1.1;
pub const ANIMAL_REACH: f32 = 3.5;
pub const ANIMAL_REST: f32 = 3.4;
pub const ARRIVE_RANGE: f32 = 0.35;

pub const CAMERA_HEIGHT: f32 = 8.0;
pub const CAMERA_REACH: f32 = 12.0;
pub const CAMERA_LOOK_LIFT: f32 = 1.2;
pub const CAMERA_FOLLOW: f32 = 8.0;
pub const CAMERA_FOV_DEGREES: f32 = 34.0;
pub const CAMERA_NEAR: f32 = 0.3;
pub const CAMERA_FAR: f32 = 140.0;

pub const CLEAR_FOG: [f32; 3] = [0.88, 0.66, 0.5];
pub const RAIN_FOG: [f32; 3] = [0.6, 0.58, 0.66];
pub const FOG_START: f32 = 34.0;
pub const FOG_END: f32 = 96.0;
pub const RAIN_FOG_START: f32 = 18.0;
pub const RAIN_FOG_END: f32 = 52.0;
pub const HOUR: f32 = 16.0;
pub const DAY_START: f32 = 15.0;
pub const DAY_END: f32 = 19.5;
pub const HOUR_SPEED: f32 = 0.06;
pub const SUN_SHADOW_SOFTNESS: f32 = 0.6;
pub const SUN_STRENGTH: f32 = 3.4;
pub const SUN_WARM: [f32; 3] = [1.0, 0.78, 0.56];
pub const SUN_WHITE: [f32; 3] = [1.0, 0.97, 0.9];
pub const BLOOM_INTENSITY: f32 = 0.06;

pub const WEATHER_SPELL: f32 = 40.0;
pub const CLOUD_SHARE: u32 = 34;
pub const RAIN_SHARE: u32 = 20;
pub const WEATHER_FADE: f32 = 0.35;
pub const RAIN_RATE: f32 = 900.0;
pub const RAIN_LIFT: f32 = 7.0;

pub const DAY_TINTS: [(f32, [f32; 3]); 8] = [
    (0.0, [0.36, 0.42, 0.66]),
    (5.0, [0.36, 0.42, 0.66]),
    (7.0, [0.94, 0.78, 0.72]),
    (9.0, [1.0, 1.0, 1.0]),
    (17.0, [1.0, 1.0, 1.0]),
    (19.0, [1.0, 0.76, 0.58]),
    (21.0, [0.36, 0.42, 0.66]),
    (24.0, [0.36, 0.42, 0.66]),
];

pub const TALK_RANGE: f32 = 1.8;
pub const SWAY_STRENGTH: f32 = 0.05;
pub const SWAY_RATE: f32 = 1.1;

pub const PANEL_BACKGROUND: Vec4 = Vec4::new(0.06, 0.08, 0.12, 0.78);
pub const PANEL_BORDER: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.16);
pub const TEXT_BRIGHT: Vec4 = Vec4::new(0.97, 0.97, 1.0, 1.0);
pub const TEXT_DIM: Vec4 = Vec4::new(0.78, 0.82, 0.9, 1.0);
pub const TEXT_WARM: Vec4 = Vec4::new(1.0, 0.86, 0.56, 1.0);
pub const VEIL: Vec4 = Vec4::new(0.03, 0.05, 0.09, 0.58);

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum Ground {
    Water,
    Sand,
    #[default]
    Grass,
    Path,
    Plank,
}

impl Ground {
    pub fn material(self) -> &'static str {
        match self {
            Ground::Water | Ground::Sand => SAND_MATERIAL,
            Ground::Grass => GRASS_MATERIAL,
            Ground::Path => PATH_MATERIAL,
            Ground::Plank => PLANK_MATERIAL,
        }
    }

    pub fn walkable(self) -> bool {
        self != Ground::Water
    }
}

pub struct Placement {
    pub spot: Vec2,
    pub lift: f32,
    pub shadow: Option<Vec2>,
    pub scale: f32,
}

impl Placement {
    pub fn new(spot: Vec2, lift: f32) -> Self {
        Self {
            spot,
            lift,
            shadow: None,
            scale: 1.0,
        }
    }

    pub fn with_shadow(mut self, size: Vec2) -> Self {
        self.shadow = Some(size);
        self
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
}

pub struct Cottage {
    pub column: u32,
    pub row: u32,
    pub wall: &'static str,
    pub roof: &'static str,
    pub door_west: bool,
    pub awning: bool,
}

pub const COTTAGE_WIDTH: u32 = 5;
pub const COTTAGE_DEPTH: u32 = 4;
pub const COTTAGE_WALL_LEVELS: u32 = 2;

pub const COTTAGES: [Cottage; 7] = [
    Cottage {
        column: 20,
        row: 8,
        wall: PLASTER_MATERIAL,
        roof: ROOF_RED_MATERIAL,
        door_west: true,
        awning: false,
    },
    Cottage {
        column: 9,
        row: 36,
        wall: PLANK_MATERIAL,
        roof: ROOF_BLUE_MATERIAL,
        door_west: false,
        awning: false,
    },
    Cottage {
        column: 20,
        row: 48,
        wall: PLASTER_MATERIAL,
        roof: ROOF_GREEN_MATERIAL,
        door_west: true,
        awning: false,
    },
    Cottage {
        column: 9,
        row: 66,
        wall: PLASTER_MATERIAL,
        roof: ROOF_RED_MATERIAL,
        door_west: false,
        awning: false,
    },
    Cottage {
        column: 20,
        row: 80,
        wall: PLANK_MATERIAL,
        roof: ROOF_BLUE_MATERIAL,
        door_west: true,
        awning: false,
    },
    Cottage {
        column: 20,
        row: 19,
        wall: PLASTER_MATERIAL,
        roof: ROOF_RED_MATERIAL,
        door_west: true,
        awning: true,
    },
    Cottage {
        column: 20,
        row: 25,
        wall: PLASTER_MATERIAL,
        roof: ROOF_GREEN_MATERIAL,
        door_west: true,
        awning: true,
    },
];

pub const SHOP_SIGNS: [(usize, u32); 2] = [(5, BASKET), (6, CRATE)];

pub struct Neighbor {
    pub name: &'static str,
    pub shape: usize,
    pub coat: [u8; 4],
    pub belly: [u8; 4],
    pub shirt: [u8; 4],
    pub column: f32,
    pub row: f32,
    pub lines: [&'static str; 3],
}

pub const NEIGHBORS: [Neighbor; 6] = [
    Neighbor {
        name: "Pim",
        shape: 0,
        coat: [216, 158, 96, 255],
        belly: [244, 226, 196, 255],
        shirt: [92, 148, 196, 255],
        column: 17.5,
        row: 10.5,
        lines: [
            "Walk north long enough and you end up back at my door.",
            "I have tested it. Twice. It takes most of a morning.",
            "Some folks call that a small world. I call it a friendly one.",
        ],
    },
    Neighbor {
        name: "Marla",
        shape: 1,
        coat: [148, 148, 164, 255],
        belly: [236, 236, 240, 255],
        shirt: [214, 122, 108, 255],
        column: 13.5,
        row: 34.5,
        lines: [
            "The bridge is the only crossing, so I sweep it myself.",
            "Rain came through last night and left the planks shining.",
            "Mind the middle board. It sings when you step on it.",
        ],
    },
    Neighbor {
        name: "Bo",
        shape: 3,
        coat: [166, 124, 92, 255],
        belly: [230, 202, 166, 255],
        shirt: [122, 178, 120, 255],
        column: 17.5,
        row: 50.5,
        lines: [
            "Morning. The hill up ahead is the best seat in the village.",
            "You can watch the whole road bend away from up there.",
            "Bring an apple. Views are better with an apple.",
        ],
    },
    Neighbor {
        name: "Juniper",
        shape: 2,
        coat: [232, 196, 108, 255],
        belly: [248, 232, 178, 255],
        shirt: [154, 122, 198, 255],
        column: 13.5,
        row: 67.5,
        lines: [
            "New here? Then the dock first. Everyone starts at the dock.",
            "The sea runs along both sides of us, east and west.",
            "Nothing past it but weather.",
        ],
    },
    Neighbor {
        name: "Tam",
        shape: 0,
        coat: [96, 96, 108, 255],
        belly: [214, 214, 220, 255],
        shirt: [226, 186, 106, 255],
        column: 12.5,
        row: 23.5,
        lines: [
            "The plaza is where we meet when the lanterns go up.",
            "Stand by the tree and look down the road. Go on.",
            "The ground leans away from you at both ends. I love that.",
        ],
    },
    Neighbor {
        name: "Wick",
        shape: 3,
        coat: [124, 100, 86, 255],
        belly: [206, 186, 160, 255],
        shirt: [110, 178, 172, 255],
        column: 17.5,
        row: 82.5,
        lines: [
            "I moved my cottage off the plaza for the quiet.",
            "Got the quiet. Lost the gossip. A fair trade most days.",
            "Come back this evening and I will have tea on.",
        ],
    },
];

pub const TOWN_TREE: Vec2 = Vec2::new(12.5, 23.5);
pub const TOWN_TREE_SCALE: f32 = 1.9;
pub const BOARD: Vec2 = Vec2::new(14.5, 21.5);

pub const CHICKEN_SPOTS: [Vec2; 5] = [
    Vec2::new(11.5, 28.5),
    Vec2::new(12.5, 29.5),
    Vec2::new(13.5, 30.5),
    Vec2::new(10.5, 30.5),
    Vec2::new(12.5, 27.5),
];

pub const SHEEP_SPOTS: [Vec2; 3] = [
    Vec2::new(22.5, 72.5),
    Vec2::new(24.5, 74.5),
    Vec2::new(21.5, 76.5),
];

pub const COW_SPOTS: [Vec2; 2] = [Vec2::new(11.5, 54.5), Vec2::new(13.5, 56.5)];

pub const LANTERN_SPOTS: [Vec2; 8] = [
    Vec2::new(10.5, 19.5),
    Vec2::new(14.5, 19.5),
    Vec2::new(10.5, 26.5),
    Vec2::new(14.5, 26.5),
    Vec2::new(14.5, 40.5),
    Vec2::new(14.5, 62.5),
    Vec2::new(14.5, 6.5),
    Vec2::new(14.5, 86.5),
];

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Walk,
    Interact,
    Advance,
    Leave,
    Pause,
}

impl nightshade_input::Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::Walk,
            Self::Interact,
            Self::Advance,
            Self::Leave,
            Self::Pause,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Walk => "walk",
            Self::Interact => "interact",
            Self::Advance => "advance",
            Self::Leave => "leave",
            Self::Pause => "pause",
        }
    }

    fn default_bindings(&self) -> &'static [nightshade_input::Binding] {
        use nightshade_input::Binding;
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
            ],
            Self::Interact => &[
                Binding::Key(KeyCode::Space),
                Binding::Key(KeyCode::KeyE),
                Binding::Key(KeyCode::Enter),
            ],
            Self::Advance => &[
                Binding::Key(KeyCode::Space),
                Binding::Key(KeyCode::KeyE),
                Binding::Key(KeyCode::Enter),
                Binding::Key(KeyCode::Escape),
            ],
            Self::Leave => &[Binding::Key(KeyCode::Escape)],
            Self::Pause => &[Binding::Key(KeyCode::Escape)],
        }
    }
}
