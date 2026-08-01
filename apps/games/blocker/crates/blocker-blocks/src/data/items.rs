use crate::data::blocks::BlockKind;
use crate::data::crops::CropKind;
use crate::data::tools::ToolKind;
use crate::theme;
use serde::{Deserialize, Serialize};

pub const CHEST_ART: [((u32, u32), [&str; 16]); 2] = [
    (
        theme::CHEST_TOP_CELL,
        [
            "BBBBBBBBBBBBBBBB",
            "BbbbbbbbbbbbbbbB",
            "BbttttttttttttbB",
            "BbtbbbbbbbbbbtbB",
            "BbtbBBBBBBBBbtbB",
            "BbtbBbbbbbbBbtbB",
            "BbtbBbbbbbbBbtbB",
            "BbtbBbbbbbbBbtbB",
            "BbtbBbbbbbbBbtbB",
            "BbtbBbbbbbbBbtbB",
            "BbtbBbbbbbbBbtbB",
            "BbtbBBBBBBBBbtbB",
            "BbtbbbbbbbbbbtbB",
            "BbttttttttttttbB",
            "BbbbbbbbbbbbbbbB",
            "BBBBBBBBBBBBBBBB",
        ],
    ),
    (
        theme::CHEST_SIDE_CELL,
        [
            "BBBBBBBBBBBBBBBB",
            "BttttttttttttttB",
            "BtbbbbbbbbbbbbtB",
            "BtbbbbbbbbbbbbtB",
            "BtbbbbbbbbbbbbtB",
            "BBBBBBBYYBBBBBBB",
            "BtttttbyybbtttbB",
            "BtbbbbbyybbbbbtB",
            "BtbbbbbYYbbbbbtB",
            "BtbbbbbbbbbbbbtB",
            "BtbbbbbbbbbbbbtB",
            "BtbbbbbbbbbbbbtB",
            "BtbbbbbbbbbbbbtB",
            "BtbbbbbbbbbbbbtB",
            "BttttttttttttttB",
            "BBBBBBBBBBBBBBBB",
        ],
    ),
];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[serde(into = "[u8; 2]", try_from = "[u8; 2]")]
pub enum Item {
    Block(BlockKind),
    Tool(ToolKind),
    Firework,
    Seed(CropKind),
    Crop(CropKind),
    Bucket,
    WaterBucket,
    Boat,
}

impl Item {
    pub fn code(self) -> [u8; 2] {
        match self {
            Item::Block(kind) => [0, kind.index()],
            Item::Tool(tool) => [1, tool.index()],
            Item::Firework => [2, 0],
            Item::Seed(crop) => [3, crop.index() as u8],
            Item::Crop(crop) => [4, crop.index() as u8],
            Item::Bucket => [5, 0],
            Item::WaterBucket => [6, 0],
            Item::Boat => [7, 0],
        }
    }

    pub fn from_code(code: [u8; 2]) -> Option<Item> {
        match code[0] {
            0 => Some(Item::Block(BlockKind::from_index(code[1]))),
            1 => ToolKind::from_index(code[1]).map(Item::Tool),
            2 => Some(Item::Firework),
            3 => CropKind::from_index(code[1]).map(Item::Seed),
            4 => CropKind::from_index(code[1]).map(Item::Crop),
            5 => Some(Item::Bucket),
            6 => Some(Item::WaterBucket),
            7 => Some(Item::Boat),
            _ => None,
        }
    }

    pub fn kind(self) -> u32 {
        let code = self.code();
        1 + ((code[0] as u32) << 8 | code[1] as u32)
    }

    pub fn from_kind(kind: u32) -> Option<Item> {
        let value = kind.checked_sub(1)?;
        Item::from_code([(value >> 8) as u8, value as u8])
    }

    pub fn name(self) -> &'static str {
        match self {
            Item::Block(kind) => kind.info().name,
            Item::Tool(tool) => tool.name(),
            Item::Firework => "Firework",
            Item::Boat => "Boat",
            Item::Bucket => "Bucket",
            Item::WaterBucket => "Water Bucket",
            Item::Seed(crop) => crop.info().seed_name,
            Item::Crop(crop) => crop.info().name,
        }
    }

    pub fn max_stack(self) -> u32 {
        match self {
            Item::Tool(_) => 1,
            _ => 64,
        }
    }

    pub fn art_index(self) -> Option<usize> {
        match self {
            Item::Tool(tool) => Some(match tool {
                ToolKind::Pickaxe => 0,
                ToolKind::Shovel => 1,
                ToolKind::Axe => 2,
                ToolKind::Sword => 3,
                ToolKind::Glider => 12,
            }),
            Item::Firework => Some(4),
            Item::Seed(crop) => Some(5 + crop.index()),
            Item::Crop(crop) => Some(7 + crop.index()),
            Item::Bucket => Some(9),
            Item::WaterBucket => Some(10),
            Item::Boat => Some(11),
            Item::Block(_) => None,
        }
    }

    pub fn art(self) -> Option<[&'static str; 16]> {
        match self {
            Item::Tool(tool) => Some(tool.art()),
            Item::Firework => Some([
                "................",
                "..........rr....",
                ".........rwwr...",
                "........rwwwwr..",
                "........rwuuwr..",
                "........rwuuwr..",
                "........rwwwwr..",
                "........rrrrrr..",
                ".......bBrrrrBb.",
                "......bB..yy..B.",
                ".....bB...yy....",
                "....bB....yy....",
                "...bB....yooy...",
                "..bB.....oooo...",
                ".BB.......oo....",
                "................",
            ]),
            Item::Boat => Some([
                "................",
                "................",
                "................",
                "................",
                ".....TT.........",
                ".....TT.........",
                ".B...TT......B..",
                ".BB..TT.....BB..",
                ".BbB.TT....BbB..",
                ".BbbBBBBBBBbbB..",
                ".BbbbbbbbbbbbB..",
                ".BBbbbbbbbbbBB..",
                "..BBBBBBBBBBB...",
                "................",
                "................",
                "................",
            ]),
            Item::Bucket => Some([
                "................",
                "................",
                "................",
                "..II........II..",
                "..II........II..",
                "..iIIIIIIIIIIi..",
                "..iIddddddddIi..",
                "..iIddddddddIi..",
                "...iIdddddddIi..",
                "...iIddddddIi...",
                "....iIdddddIi...",
                "....iIIIIIIIi...",
                ".....iiiiiii....",
                "................",
                "................",
                "................",
            ]),
            Item::WaterBucket => Some([
                "................",
                "................",
                "................",
                "..II........II..",
                "..II........II..",
                "..iIIIIIIIIIIi..",
                "..iIuuuuuuuuIi..",
                "..iIuuuuuuuuIi..",
                "...iIuuuuuuuIi..",
                "...iIuuuuuuIi...",
                "....iIuuuuuIi...",
                "....iIIIIIIIi...",
                ".....iiiiiii....",
                "................",
                "................",
                "................",
            ]),
            Item::Seed(CropKind::Carrot) => Some([
                "................",
                "................",
                ".....tttttt.....",
                "....tTttttTt....",
                "...tTttttttTt...",
                "...tttooootTt...",
                "...ttoooooott...",
                "...ttoooooott...",
                "...tttooootTt...",
                "...tTttttttTt...",
                "....tTttttTt....",
                ".....tttttt.....",
                "................",
                "................",
                "................",
                "................",
            ]),
            Item::Seed(CropKind::Wheat) => Some([
                "................",
                "................",
                ".....tttttt.....",
                "....tTttttTt....",
                "...tTttttttTt...",
                "...tttyyyytTt...",
                "...ttyyyyyytt...",
                "...ttyyyyyytt...",
                "...tttyyyytTt...",
                "...tTttttttTt...",
                "....tTttttTt....",
                ".....tttttt.....",
                "................",
                "................",
                "................",
                "................",
            ]),
            Item::Crop(CropKind::Carrot) => Some([
                "................",
                "..........gg....",
                ".........gGg....",
                "........gGgg....",
                ".......gGg......",
                "......ooGg......",
                ".....oooo.......",
                "....ooooo.......",
                "....oooo........",
                "...oooo.........",
                "...ooo..........",
                "..ooo...........",
                "..oo............",
                ".oo.............",
                "................",
                "................",
            ]),
            Item::Crop(CropKind::Wheat) => Some([
                "................",
                "..........yy....",
                ".........yYy....",
                "........yYyy....",
                ".......yYy......",
                "......yYyy......",
                ".....yYy........",
                "....yYyy........",
                "...yYyY.........",
                "...tYt..........",
                "..tT............",
                "..tT............",
                ".tT.............",
                ".tT.............",
                "................",
                "................",
            ]),
            Item::Block(_) => None,
        }
    }

    pub fn mesh_name(self) -> String {
        match self {
            Item::Block(kind) => format!("voxels_item_block_{}", kind.index()),
            _ => format!("voxels_item_art_{}", self.art_index().unwrap_or(0)),
        }
    }

    pub fn material_name(self) -> &'static str {
        match self {
            Item::Block(kind) if kind.info().transparent => {
                crate::data::atlas::TRANSPARENT_MATERIAL
            }
            Item::Block(kind) if kind.info().cutout => crate::data::atlas::CUTOUT_MATERIAL,
            Item::Block(_) => theme::SOLID_MATERIAL,
            _ => crate::data::atlas::FLAT_MATERIAL,
        }
    }
}

pub const ART_ITEMS: [Item; 13] = [
    Item::Tool(ToolKind::Pickaxe),
    Item::Tool(ToolKind::Shovel),
    Item::Tool(ToolKind::Axe),
    Item::Tool(ToolKind::Sword),
    Item::Firework,
    Item::Seed(CropKind::Carrot),
    Item::Seed(CropKind::Wheat),
    Item::Crop(CropKind::Carrot),
    Item::Crop(CropKind::Wheat),
    Item::Bucket,
    Item::WaterBucket,
    Item::Boat,
    Item::Tool(ToolKind::Glider),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ItemStack {
    pub item: Item,
    pub count: u32,
}

pub fn starting_inventory() -> [Option<ItemStack>; crate::data::items::INVENTORY_SLOTS] {
    let stack = |item, count| Some(ItemStack { item, count });
    let given = [
        stack(Item::Tool(ToolKind::Pickaxe), 1),
        stack(Item::Tool(ToolKind::Shovel), 1),
        stack(Item::Tool(ToolKind::Axe), 1),
        stack(Item::Tool(ToolKind::Sword), 1),
        stack(Item::Tool(ToolKind::Glider), 1),
        stack(Item::Block(BlockKind::Torch), 16),
        stack(Item::Block(BlockKind::OakPlanks), 32),
        stack(Item::Boat, 1),
        stack(Item::Seed(CropKind::Carrot), 4),
        stack(Item::WaterBucket, 1),
        stack(Item::Block(BlockKind::Glass), 16),
    ];
    let mut slots = [None; crate::data::items::INVENTORY_SLOTS];
    slots[..given.len()].copy_from_slice(&given);
    slots
}

#[derive(Clone, Copy, Debug)]
pub struct Recipe {
    pub output: Item,
    pub count: u32,
    pub inputs: &'static [(Item, u32)],
}

pub const RECIPES: [Recipe; 17] = [
    Recipe {
        output: Item::Boat,
        count: 1,
        inputs: &[(Item::Block(BlockKind::OakPlanks), 5)],
    },
    Recipe {
        output: Item::Block(BlockKind::Chest),
        count: 1,
        inputs: &[(Item::Block(BlockKind::OakPlanks), 8)],
    },
    Recipe {
        output: Item::Bucket,
        count: 1,
        inputs: &[(Item::Block(BlockKind::IronOre), 3)],
    },
    Recipe {
        output: Item::Block(BlockKind::DoorNorth),
        count: 1,
        inputs: &[(Item::Block(BlockKind::OakPlanks), 6)],
    },
    Recipe {
        output: Item::Block(BlockKind::Torch),
        count: 4,
        inputs: &[
            (Item::Block(BlockKind::OakPlanks), 1),
            (Item::Block(BlockKind::CoalOre), 1),
        ],
    },
    Recipe {
        output: Item::Block(BlockKind::OakPlanks),
        count: 4,
        inputs: &[(Item::Block(BlockKind::OakLog), 1)],
    },
    Recipe {
        output: Item::Block(BlockKind::CraftingTable),
        count: 1,
        inputs: &[(Item::Block(BlockKind::OakPlanks), 4)],
    },
    Recipe {
        output: Item::Block(BlockKind::Signpost),
        count: 1,
        inputs: &[
            (Item::Block(BlockKind::OakPlanks), 2),
            (Item::Block(BlockKind::OakLog), 1),
        ],
    },
    Recipe {
        output: Item::Block(BlockKind::Campfire),
        count: 1,
        inputs: &[
            (Item::Block(BlockKind::Cobblestone), 4),
            (Item::Block(BlockKind::OakLog), 2),
        ],
    },
    Recipe {
        output: Item::Block(BlockKind::Glass),
        count: 1,
        inputs: &[(Item::Block(BlockKind::Sand), 2)],
    },
    Recipe {
        output: Item::Block(BlockKind::Bricks),
        count: 1,
        inputs: &[(Item::Block(BlockKind::Cobblestone), 4)],
    },
    Recipe {
        output: Item::Firework,
        count: 2,
        inputs: &[
            (Item::Block(BlockKind::Gravel), 1),
            (Item::Block(BlockKind::Glowstone), 1),
            (Item::Block(BlockKind::OakPlanks), 1),
        ],
    },
    Recipe {
        output: Item::Block(BlockKind::WoolRed),
        count: 1,
        inputs: &[
            (Item::Block(BlockKind::WoolWhite), 1),
            (Item::Block(BlockKind::RedFlower), 1),
        ],
    },
    Recipe {
        output: Item::Tool(ToolKind::Pickaxe),
        count: 1,
        inputs: &[
            (Item::Block(BlockKind::Cobblestone), 3),
            (Item::Block(BlockKind::OakPlanks), 2),
        ],
    },
    Recipe {
        output: Item::Tool(ToolKind::Shovel),
        count: 1,
        inputs: &[
            (Item::Block(BlockKind::Cobblestone), 1),
            (Item::Block(BlockKind::OakPlanks), 2),
        ],
    },
    Recipe {
        output: Item::Tool(ToolKind::Axe),
        count: 1,
        inputs: &[
            (Item::Block(BlockKind::Cobblestone), 3),
            (Item::Block(BlockKind::OakPlanks), 2),
        ],
    },
    Recipe {
        output: Item::Seed(CropKind::Wheat),
        count: 2,
        inputs: &[(Item::Crop(CropKind::Wheat), 1)],
    },
];

impl From<Item> for [u8; 2] {
    fn from(value: Item) -> Self {
        value.code()
    }
}

impl TryFrom<[u8; 2]> for Item {
    type Error = &'static str;

    fn try_from(code: [u8; 2]) -> Result<Self, Self::Error> {
        Item::from_code(code).ok_or("unknown item")
    }
}

pub const HOTBAR_SLOTS: usize = 9;

pub const INVENTORY_ROWS: usize = 4;

pub const INVENTORY_SLOTS: usize = HOTBAR_SLOTS * INVENTORY_ROWS;
