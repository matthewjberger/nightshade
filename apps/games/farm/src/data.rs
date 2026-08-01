use crate::theme;
use nalgebra_glm::Vec2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Item {
    #[default]
    CarrotSeed,
    EggplantSeed,
    TomatoSeed,
    WheatSeed,
    Carrot,
    Eggplant,
    Tomato,
    Wheat,
    Egg,
    Milk,
    Wood,
    Stone,
    Fish,
    Bread,
    Berry,
    Ore,
}

impl Item {
    pub const ALL: [Item; 16] = [
        Item::CarrotSeed,
        Item::EggplantSeed,
        Item::TomatoSeed,
        Item::WheatSeed,
        Item::Carrot,
        Item::Eggplant,
        Item::Tomato,
        Item::Wheat,
        Item::Egg,
        Item::Milk,
        Item::Wood,
        Item::Stone,
        Item::Fish,
        Item::Bread,
        Item::Berry,
        Item::Ore,
    ];

    pub fn index(self) -> usize {
        Item::ALL.iter().position(|item| *item == self).unwrap_or(0)
    }

    pub fn name(self) -> &'static str {
        match self {
            Item::CarrotSeed => "Carrot seeds",
            Item::EggplantSeed => "Eggplant seeds",
            Item::TomatoSeed => "Tomato seeds",
            Item::WheatSeed => "Wheat seeds",
            Item::Carrot => "Carrot",
            Item::Eggplant => "Eggplant",
            Item::Tomato => "Tomato",
            Item::Wheat => "Wheat",
            Item::Egg => "Egg",
            Item::Milk => "Milk",
            Item::Wood => "Wood",
            Item::Stone => "Stone",
            Item::Fish => "Fish",
            Item::Bread => "Bread",
            Item::Berry => "Berries",
            Item::Ore => "Ore",
        }
    }

    pub fn icon(self) -> u32 {
        match self {
            Item::CarrotSeed => theme::CARROT_SEED_ICON,
            Item::EggplantSeed => theme::EGGPLANT_SEED_ICON,
            Item::TomatoSeed => theme::TOMATO_SEED_ICON,
            Item::WheatSeed => theme::WHEAT_SEED_ICON,
            Item::Carrot => theme::CARROT_STAGES[3],
            Item::Eggplant => theme::EGGPLANT_STAGES[3],
            Item::Tomato => theme::TOMATO_STAGES[3],
            Item::Wheat => theme::WHEAT_STAGES[3],
            Item::Egg => theme::EGG,
            Item::Milk => theme::MILK_PAIL,
            Item::Wood => theme::WOOD_ICON,
            Item::Stone => theme::ROCK_PILE,
            Item::Fish => theme::FISH_ICON,
            Item::Bread => theme::BREAD_ICON,
            Item::Berry => theme::BERRY_BUSH,
            Item::Ore => theme::ORE_ICON,
        }
    }

    pub fn food(self) -> f32 {
        match self {
            Item::Bread => 34.0,
            Item::Fish => 26.0,
            Item::Milk => 20.0,
            Item::Eggplant | Item::Tomato => 18.0,
            Item::Egg => 14.0,
            Item::Carrot => 12.0,
            Item::Berry => 8.0,
            Item::Wheat => 6.0,
            _ => 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Tool {
    #[default]
    Hoe,
    WateringCan,
    Axe,
    Shovel,
    Bucket,
    FishingRod,
}

impl Tool {
    pub const ALL: [Tool; 6] = [
        Tool::Hoe,
        Tool::WateringCan,
        Tool::Axe,
        Tool::Shovel,
        Tool::Bucket,
        Tool::FishingRod,
    ];

    pub fn index(self) -> usize {
        Tool::ALL.iter().position(|tool| *tool == self).unwrap_or(0)
    }

    pub fn name(self) -> &'static str {
        match self {
            Tool::Hoe => "Hoe",
            Tool::WateringCan => "Watering can",
            Tool::Axe => "Axe",
            Tool::Shovel => "Shovel",
            Tool::Bucket => "Bucket",
            Tool::FishingRod => "Fishing rod",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            Tool::Hoe => "turns grass into a plot",
            Tool::WateringCan => "waters a plot for the day",
            Tool::Axe => "fells a tree for wood",
            Tool::Shovel => "digs up rocks for stone",
            Tool::Bucket => "milks a cow, once a day",
            Tool::FishingRod => "casts from the water's edge",
        }
    }

    pub fn effort(self) -> f32 {
        match self {
            Tool::Axe | Tool::Shovel => 9.0,
            Tool::Hoe => 7.0,
            Tool::Bucket => 4.0,
            Tool::WateringCan => 3.0,
            Tool::FishingRod => 2.0,
        }
    }

    pub fn icon(self) -> u32 {
        match self {
            Tool::Hoe => theme::HOE_ICON,
            Tool::WateringCan => theme::WATERING_CAN_ICON,
            Tool::Axe => theme::AXE_ICON,
            Tool::Shovel => theme::SHOVEL_ICON,
            Tool::Bucket => theme::BUCKET_ICON,
            Tool::FishingRod => theme::ROD_ICON,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Tool(Tool),
    Seed(CropKind),
}

impl Slot {
    pub const ALL: [Slot; 10] = [
        Slot::Tool(Tool::Hoe),
        Slot::Tool(Tool::WateringCan),
        Slot::Tool(Tool::Axe),
        Slot::Tool(Tool::Shovel),
        Slot::Tool(Tool::Bucket),
        Slot::Tool(Tool::FishingRod),
        Slot::Seed(CropKind::Carrot),
        Slot::Seed(CropKind::Eggplant),
        Slot::Seed(CropKind::Tomato),
        Slot::Seed(CropKind::Wheat),
    ];

    pub fn icon(self) -> u32 {
        match self {
            Slot::Tool(tool) => tool.icon(),
            Slot::Seed(kind) => kind.seed().icon(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ware {
    pub item: Item,
    pub price: u32,
    pub sells: bool,
}

impl Ware {
    const fn sells(item: Item, price: u32) -> Self {
        Self {
            item,
            price,
            sells: true,
        }
    }

    const fn buys(item: Item, price: u32) -> Self {
        Self {
            item,
            price,
            sells: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ShopKind {
    #[default]
    Barn,
    Forge,
    Bakery,
}

impl ShopKind {
    pub const ALL: [ShopKind; 3] = [ShopKind::Barn, ShopKind::Forge, ShopKind::Bakery];

    pub fn index(self) -> usize {
        ShopKind::ALL
            .iter()
            .position(|kind| *kind == self)
            .unwrap_or(0)
    }

    pub fn title(self) -> &'static str {
        match self {
            ShopKind::Barn => "Bram's Barn Store",
            ShopKind::Forge => "Wren's Forge",
            ShopKind::Bakery => "Lina's Oven",
        }
    }

    pub fn wares(self) -> &'static [Ware] {
        match self {
            ShopKind::Barn => &BARN_WARES,
            ShopKind::Forge => &FORGE_WARES,
            ShopKind::Bakery => &BAKERY_WARES,
        }
    }
}

const BARN_WARES: [Ware; 14] = [
    Ware::buys(Item::Berry, 5),
    Ware::sells(Item::CarrotSeed, 4),
    Ware::sells(Item::EggplantSeed, 6),
    Ware::sells(Item::TomatoSeed, 7),
    Ware::sells(Item::WheatSeed, 3),
    Ware::buys(Item::Carrot, 11),
    Ware::buys(Item::Eggplant, 15),
    Ware::buys(Item::Tomato, 17),
    Ware::buys(Item::Wheat, 8),
    Ware::buys(Item::Egg, 6),
    Ware::buys(Item::Milk, 12),
    Ware::buys(Item::Wood, 5),
    Ware::buys(Item::Stone, 4),
    Ware::buys(Item::Fish, 14),
];

const FORGE_WARES: [Ware; 3] = [
    Ware::sells(Item::Bread, 14),
    Ware::buys(Item::Stone, 6),
    Ware::buys(Item::Wood, 6),
];

const BAKERY_WARES: [Ware; 5] = [
    Ware::buys(Item::Berry, 8),
    Ware::sells(Item::Bread, 10),
    Ware::buys(Item::Wheat, 12),
    Ware::buys(Item::Egg, 9),
    Ware::buys(Item::Milk, 16),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CropKind {
    #[default]
    Carrot,
    Eggplant,
    Tomato,
    Wheat,
}

impl CropKind {
    pub const ALL: [CropKind; 4] = [
        CropKind::Carrot,
        CropKind::Eggplant,
        CropKind::Tomato,
        CropKind::Wheat,
    ];

    pub fn index(self) -> usize {
        CropKind::ALL
            .iter()
            .position(|kind| *kind == self)
            .unwrap_or(0)
    }

    pub fn name(self) -> &'static str {
        match self {
            CropKind::Carrot => "carrot",
            CropKind::Eggplant => "eggplant",
            CropKind::Tomato => "tomato",
            CropKind::Wheat => "wheat",
        }
    }

    pub fn seed(self) -> Item {
        match self {
            CropKind::Carrot => Item::CarrotSeed,
            CropKind::Eggplant => Item::EggplantSeed,
            CropKind::Tomato => Item::TomatoSeed,
            CropKind::Wheat => Item::WheatSeed,
        }
    }

    pub fn harvest(self) -> Item {
        match self {
            CropKind::Carrot => Item::Carrot,
            CropKind::Eggplant => Item::Eggplant,
            CropKind::Tomato => Item::Tomato,
            CropKind::Wheat => Item::Wheat,
        }
    }

    pub fn stages(self) -> [u32; 4] {
        match self {
            CropKind::Carrot => theme::CARROT_STAGES,
            CropKind::Eggplant => theme::EGGPLANT_STAGES,
            CropKind::Tomato => theme::TOMATO_STAGES,
            CropKind::Wheat => theme::WHEAT_STAGES,
        }
    }

    pub fn grow_hours(self) -> f32 {
        match self {
            CropKind::Carrot => 14.0,
            CropKind::Eggplant => 22.0,
            CropKind::Tomato => 26.0,
            CropKind::Wheat => 10.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Errand {
    Carrots,
    Wheat,
    Fish,
    Wood,
    Egg,
    Stone,
    Bread,
}

impl Errand {
    pub const ALL: [Errand; 7] = [
        Errand::Carrots,
        Errand::Wheat,
        Errand::Fish,
        Errand::Wood,
        Errand::Egg,
        Errand::Stone,
        Errand::Bread,
    ];

    pub fn index(self) -> usize {
        Errand::ALL
            .iter()
            .position(|errand| *errand == self)
            .unwrap_or(0)
    }

    pub fn asker(self) -> Person {
        match self {
            Errand::Carrots => Person::Mira,
            Errand::Wheat => Person::Lina,
            Errand::Fish => Person::Tam,
            Errand::Wood => Person::Nell,
            Errand::Egg => Person::Pip,
            Errand::Stone => Person::Wren,
            Errand::Bread => Person::Juno,
        }
    }

    pub fn wants(self) -> (Item, u32) {
        match self {
            Errand::Carrots => (Item::Carrot, 3),
            Errand::Wheat => (Item::Wheat, 5),
            Errand::Fish => (Item::Fish, 3),
            Errand::Wood => (Item::Wood, 10),
            Errand::Egg => (Item::Egg, 1),
            Errand::Stone => (Item::Stone, 6),
            Errand::Bread => (Item::Bread, 2),
        }
    }

    pub fn reward(self) -> u32 {
        match self {
            Errand::Carrots => 60,
            Errand::Wheat => 50,
            Errand::Fish => 70,
            Errand::Wood => 80,
            Errand::Egg => 10,
            Errand::Stone => 45,
            Errand::Bread => 40,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Person {
    #[default]
    Bram,
    Mira,
    Wren,
    Lina,
    Tam,
    Pip,
    Osk,
    Dov,
    Nell,
    Juno,
}

impl Person {
    pub const ALL: [Person; 10] = [
        Person::Bram,
        Person::Mira,
        Person::Wren,
        Person::Lina,
        Person::Tam,
        Person::Pip,
        Person::Osk,
        Person::Dov,
        Person::Nell,
        Person::Juno,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Person::Bram => theme::BRAM,
            Person::Mira => theme::MIRA,
            Person::Wren => theme::WREN,
            Person::Lina => theme::LINA,
            Person::Tam => theme::TAM,
            Person::Pip => theme::PIP,
            Person::Osk => theme::OSK,
            Person::Dov => theme::DOV,
            Person::Nell => theme::NELL,
            Person::Juno => theme::JUNO,
        }
    }

    pub fn tile(self) -> u32 {
        match self {
            Person::Bram => theme::BRAM_TILE,
            Person::Mira => theme::MIRA_TILE,
            Person::Wren => theme::WREN_TILE,
            Person::Lina => theme::LINA_TILE,
            Person::Tam => theme::TAM_TILE,
            Person::Pip => theme::PIP_TILE,
            Person::Osk => theme::OSK_TILE,
            Person::Dov => theme::DOV_TILE,
            Person::Nell => theme::NELL_TILE,
            Person::Juno => theme::JUNO_TILE,
        }
    }

    pub fn tint(self) -> [f32; 4] {
        match self {
            Person::Bram => [1.0, 1.0, 1.0, 1.0],
            Person::Mira => [0.82, 0.92, 1.0, 1.0],
            Person::Wren => [0.82, 0.82, 0.9, 1.0],
            Person::Lina => [1.0, 0.88, 0.82, 1.0],
            Person::Tam => [0.78, 0.9, 0.85, 1.0],
            Person::Pip => [1.0, 0.95, 0.7, 1.0],
            Person::Osk => [0.9, 0.9, 0.78, 1.0],
            Person::Dov => [0.88, 1.0, 0.85, 1.0],
            Person::Nell => [1.0, 0.85, 0.95, 1.0],
            Person::Juno => [0.85, 0.8, 1.0, 1.0],
        }
    }

    pub fn size(self) -> f32 {
        match self {
            Person::Pip => 0.72,
            _ => 1.0,
        }
    }

    pub fn home(self) -> Option<HouseKind> {
        match self {
            Person::Bram | Person::Dov => Some(HouseKind::Barn),
            Person::Mira | Person::Osk => Some(HouseKind::Elder),
            Person::Wren => Some(HouseKind::Forge),
            Person::Lina | Person::Pip => Some(HouseKind::Bakery),
            Person::Tam => Some(HouseKind::Fisher),
            Person::Nell => Some(HouseKind::Carpenter),
            Person::Juno => Some(HouseKind::Tavern),
        }
    }

    pub fn post(self) -> Vec2 {
        match self {
            Person::Bram => Vec2::new(9.5, -9.5),
            Person::Mira => Vec2::new(-4.5, 0.5),
            Person::Wren => Vec2::new(-2.5, -9.5),
            Person::Lina => Vec2::new(-7.5, -9.5),
            Person::Tam => Vec2::new(12.5, 7.0),
            Person::Pip => Vec2::new(-2.0, -3.0),
            Person::Osk => Vec2::new(1.5, -6.5),
            Person::Dov => Vec2::new(7.0, -6.0),
            Person::Nell => Vec2::new(15.5, -9.5),
            Person::Juno => Vec2::new(1.0, 11.0),
        }
    }

    pub fn midday_post(self) -> Vec2 {
        match self {
            Person::Bram => theme::MARKET,
            Person::Mira => Vec2::new(-6.5, -2.5),
            Person::Wren => Vec2::new(-2.5, -9.5),
            Person::Lina => theme::MARKET + Vec2::new(-2.0, 0.0),
            Person::Tam => Vec2::new(13.5, 5.5),
            Person::Pip => Vec2::new(5.5, -7.5),
            Person::Osk => theme::CHAPEL_FRONT,
            Person::Dov => Vec2::new(17.5, -8.5),
            Person::Nell => Vec2::new(16.5, -8.5),
            Person::Juno => theme::MARKET + Vec2::new(2.0, 0.0),
        }
    }

    pub fn evening_post(self) -> Vec2 {
        match self {
            Person::Bram => Vec2::new(9.5, -9.5),
            Person::Mira => theme::TAVERN_FRONT + Vec2::new(1.5, 0.0),
            Person::Wren => theme::TAVERN_FRONT,
            Person::Lina => theme::TAVERN_FRONT + Vec2::new(3.0, 0.0),
            Person::Tam => Vec2::new(12.5, 7.0),
            Person::Pip => Vec2::new(-7.5, -9.5),
            Person::Osk => Vec2::new(1.5, -6.5),
            Person::Dov => Vec2::new(7.0, -6.0),
            Person::Nell => theme::TAVERN_FRONT + Vec2::new(0.0, 1.0),
            Person::Juno => Vec2::new(-19.5, -9.5),
        }
    }

    pub fn post_at(self, hour: f32) -> Vec2 {
        if hour >= theme::EVENING_HOUR {
            self.evening_post()
        } else if hour >= theme::MIDDAY_HOUR {
            self.midday_post()
        } else {
            self.post()
        }
    }

    pub fn reach(self) -> f32 {
        match self {
            Person::Pip => 5.0,
            Person::Mira | Person::Juno => 2.5,
            Person::Dov => 3.0,
            _ => 1.2,
        }
    }

    pub fn speed(self) -> f32 {
        match self {
            Person::Pip => 2.4,
            Person::Osk => 0.6,
            _ => 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeKind {
    Pine,
    Dead,
    Small,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum WallStyle {
    #[default]
    Plaster,
    Planks,
    RedPlanks,
    Stone,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RoofStyle {
    #[default]
    Red,
    Dark,
    Green,
}

impl WallStyle {
    pub fn material(self) -> &'static str {
        match self {
            WallStyle::Plaster => theme::PLASTER_MATERIAL,
            WallStyle::Planks => theme::PLANK_MATERIAL,
            WallStyle::RedPlanks => theme::RED_PLANK_MATERIAL,
            WallStyle::Stone => theme::STONE_WALL_MATERIAL,
        }
    }
}

impl RoofStyle {
    pub fn material(self) -> &'static str {
        match self {
            RoofStyle::Red => theme::ROOF_RED_MATERIAL,
            RoofStyle::Dark => theme::ROOF_DARK_MATERIAL,
            RoofStyle::Green => theme::ROOF_GREEN_MATERIAL,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum HouseKind {
    #[default]
    Farmhouse,
    Bakery,
    Forge,
    Elder,
    Barn,
    Carpenter,
    Fisher,
    Tavern,
    Chapel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Furnishing {
    Plain,
    Bed,
    Counter,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Piece {
    pub column: u32,
    pub row: u32,
    pub tile: u32,
    pub span: u32,
    pub role: Furnishing,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Room {
    pub columns: u32,
    pub rows: u32,
    pub floor: &'static str,
    pub pieces: &'static [Piece],
}

const fn piece(column: u32, row: u32, tile: u32) -> Piece {
    Piece {
        column,
        row,
        tile,
        span: 1,
        role: Furnishing::Plain,
    }
}

const fn bed(column: u32, row: u32, tile: u32) -> Piece {
    Piece {
        column,
        row,
        tile,
        span: 2,
        role: Furnishing::Bed,
    }
}

const fn counter(column: u32, row: u32, tile: u32) -> Piece {
    Piece {
        column,
        row,
        tile,
        span: 1,
        role: Furnishing::Counter,
    }
}

const FARMHOUSE_PIECES: [Piece; 4] = [
    bed(1, 1, theme::WOOD_BED_TILE),
    piece(5, 1, theme::CRATE_TILE),
    piece(1, 3, theme::BARREL_TILE),
    piece(4, 3, theme::BASKET_TILE),
];

const BAKERY_PIECES: [Piece; 5] = [
    counter(3, 1, theme::CRATE_TILE),
    piece(1, 1, theme::SACK_TILE),
    piece(1, 2, theme::SACK_TILE),
    piece(5, 1, theme::BARREL_TILE),
    bed(4, 3, theme::YELLOW_BED_TILE),
];

const FORGE_PIECES: [Piece; 4] = [
    counter(4, 1, theme::ANVIL_TILE),
    bed(1, 1, theme::GREY_BED_TILE),
    piece(1, 3, theme::BARREL_TILE),
    piece(5, 3, theme::CRATE_TILE),
];

const ELDER_PIECES: [Piece; 3] = [
    bed(1, 1, theme::BLUE_BED_TILE),
    piece(3, 3, theme::BASKET_TILE),
    piece(1, 3, theme::CRATE_TILE),
];

const BARN_PIECES: [Piece; 5] = [
    piece(1, 1, theme::PAIL_TILE),
    piece(2, 1, theme::WATER_PAIL_TILE),
    piece(5, 1, theme::HAY_SACK_TILE),
    piece(5, 2, theme::BASKET_TILE),
    counter(3, 3, theme::CRATE_TILE),
];

const TAVERN_PIECES: [Piece; 6] = [
    counter(3, 3, theme::BARREL_TILE),
    piece(1, 1, theme::BARREL_TILE),
    piece(2, 1, theme::BARREL_TILE),
    piece(5, 1, theme::CRATE_TILE),
    bed(5, 3, theme::YELLOW_BED_TILE),
    piece(1, 3, theme::SACK_TILE),
];

const CHAPEL_PIECES: [Piece; 3] = [
    piece(2, 1, theme::BASKET_TILE),
    piece(1, 3, theme::CRATE_TILE),
    piece(3, 3, theme::CRATE_TILE),
];

const CARPENTER_PIECES: [Piece; 4] = [
    piece(1, 1, theme::CRATE_TILE),
    piece(2, 1, theme::BASKET_TILE),
    bed(3, 3, theme::WOOD_BED_TILE),
    piece(1, 3, theme::BARREL_TILE),
];

const FISHER_PIECES: [Piece; 4] = [
    piece(1, 1, theme::BARREL_TILE),
    piece(1, 3, theme::CRATE_TILE),
    piece(3, 1, theme::SACK_TILE),
    bed(3, 3, theme::YELLOW_BED_TILE),
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct House {
    pub kind: HouseKind,
    pub name: &'static str,
    pub origin: Vec2,
    pub width: u32,
    pub depth: u32,
    pub height: u32,
    pub wall: WallStyle,
    pub roof: RoofStyle,
    pub door: u32,
    pub door_width: u32,
    pub windows: &'static [u32],
    pub chimney: bool,
    pub shop: Option<ShopKind>,
}

impl HouseKind {
    pub const ALL: [HouseKind; 9] = [
        HouseKind::Farmhouse,
        HouseKind::Bakery,
        HouseKind::Forge,
        HouseKind::Elder,
        HouseKind::Barn,
        HouseKind::Carpenter,
        HouseKind::Fisher,
        HouseKind::Tavern,
        HouseKind::Chapel,
    ];

    pub fn room(self) -> Room {
        match self {
            HouseKind::Farmhouse => Room {
                columns: 7,
                rows: 5,
                floor: theme::PLANK_MATERIAL,
                pieces: &FARMHOUSE_PIECES,
            },
            HouseKind::Bakery => Room {
                columns: 7,
                rows: 5,
                floor: theme::PLANK_MATERIAL,
                pieces: &BAKERY_PIECES,
            },
            HouseKind::Forge => Room {
                columns: 7,
                rows: 5,
                floor: theme::STONE_MATERIAL,
                pieces: &FORGE_PIECES,
            },
            HouseKind::Elder => Room {
                columns: 5,
                rows: 5,
                floor: theme::PLANK_MATERIAL,
                pieces: &ELDER_PIECES,
            },
            HouseKind::Barn => Room {
                columns: 7,
                rows: 5,
                floor: theme::DIRT_MATERIAL,
                pieces: &BARN_PIECES,
            },
            HouseKind::Carpenter => Room {
                columns: 6,
                rows: 5,
                floor: theme::PLANK_MATERIAL,
                pieces: &CARPENTER_PIECES,
            },
            HouseKind::Fisher => Room {
                columns: 6,
                rows: 5,
                floor: theme::PLANK_MATERIAL,
                pieces: &FISHER_PIECES,
            },
            HouseKind::Tavern => Room {
                columns: 7,
                rows: 5,
                floor: theme::PLANK_MATERIAL,
                pieces: &TAVERN_PIECES,
            },
            HouseKind::Chapel => Room {
                columns: 5,
                rows: 5,
                floor: theme::STONE_MATERIAL,
                pieces: &CHAPEL_PIECES,
            },
        }
    }

    pub fn house(self) -> House {
        match self {
            HouseKind::Farmhouse => House {
                kind: self,
                name: "Home",
                origin: Vec2::new(-15.0, -14.0),
                width: 4,
                depth: 3,
                height: 2,
                wall: WallStyle::Plaster,
                roof: RoofStyle::Red,
                door: 1,
                door_width: 1,
                windows: &[3],
                chimney: true,
                shop: None,
            },
            HouseKind::Bakery => House {
                kind: self,
                name: "Lina's Oven",
                origin: Vec2::new(-9.0, -14.0),
                width: 3,
                depth: 3,
                height: 2,
                wall: WallStyle::Planks,
                roof: RoofStyle::Green,
                door: 1,
                door_width: 1,
                windows: &[0, 2],
                chimney: true,
                shop: Some(ShopKind::Bakery),
            },
            HouseKind::Forge => House {
                kind: self,
                name: "Wren's Forge",
                origin: Vec2::new(-4.0, -14.0),
                width: 4,
                depth: 3,
                height: 2,
                wall: WallStyle::Stone,
                roof: RoofStyle::Dark,
                door: 0,
                door_width: 1,
                windows: &[2],
                chimney: true,
                shop: Some(ShopKind::Forge),
            },
            HouseKind::Elder => House {
                kind: self,
                name: "Osk's house",
                origin: Vec2::new(2.0, -14.0),
                width: 3,
                depth: 3,
                height: 2,
                wall: WallStyle::Plaster,
                roof: RoofStyle::Red,
                door: 1,
                door_width: 1,
                windows: &[0, 2],
                chimney: false,
                shop: None,
            },
            HouseKind::Barn => House {
                kind: self,
                name: "Bram's Barn",
                origin: Vec2::new(7.0, -14.0),
                width: 5,
                depth: 3,
                height: 3,
                wall: WallStyle::RedPlanks,
                roof: RoofStyle::Dark,
                door: 1,
                door_width: 2,
                windows: &[0, 4],
                chimney: false,
                shop: Some(ShopKind::Barn),
            },
            HouseKind::Carpenter => House {
                kind: self,
                name: "Nell's workshop",
                origin: Vec2::new(14.0, -14.0),
                width: 3,
                depth: 3,
                height: 2,
                wall: WallStyle::Planks,
                roof: RoofStyle::Red,
                door: 1,
                door_width: 1,
                windows: &[2],
                chimney: false,
                shop: None,
            },
            HouseKind::Tavern => House {
                kind: self,
                name: "The Kettle",
                origin: Vec2::new(-21.0, -14.0),
                width: 4,
                depth: 3,
                height: 2,
                wall: WallStyle::Planks,
                roof: RoofStyle::Dark,
                door: 1,
                door_width: 1,
                windows: &[0, 3],
                chimney: true,
                shop: None,
            },
            HouseKind::Chapel => House {
                kind: self,
                name: "The chapel",
                origin: Vec2::new(-11.0, -14.0),
                width: 2,
                depth: 3,
                height: 3,
                wall: WallStyle::Stone,
                roof: RoofStyle::Green,
                door: 0,
                door_width: 1,
                windows: &[1],
                chimney: false,
                shop: None,
            },
            HouseKind::Fisher => House {
                kind: self,
                name: "Tam's hut",
                origin: Vec2::new(13.0, 9.0),
                width: 3,
                depth: 2,
                height: 2,
                wall: WallStyle::Stone,
                roof: RoofStyle::Green,
                door: 1,
                door_width: 1,
                windows: &[0],
                chimney: true,
                shop: None,
            },
        }
    }

    pub fn door_spot(self) -> Vec2 {
        let house = self.house();
        Vec2::new(
            house.origin.x + house.door as f32 + house.door_width as f32 / 2.0,
            house.origin.y + house.depth as f32 + 0.5,
        )
    }
}

pub const ROD_PIXELS: [&str; 16] = [
    "..............k.",
    ".............kwk",
    "............kw.k",
    "...........kw...",
    "..........kw....",
    ".........kw.....",
    "........kw......",
    ".......kw.......",
    "......kw........",
    ".....kw.........",
    "....kw..........",
    "...kw...........",
    "..kw............",
    ".kbk............",
    "kbbk............",
    ".kk.............",
];

pub const WOOD_PIXELS: [&str; 16] = [
    "................",
    "................",
    "................",
    "....kkkkkkkkk...",
    "...kbbbbbbbbbkk.",
    "..kbllllllllbolk",
    "..kbllllllllbolk",
    "..kbbbbbbbbbbolk",
    "..kbllllllllbolk",
    "..kbllllllllbkk.",
    "...kbbbbbbbbbk..",
    "....kkkkkkkkk...",
    "................",
    "................",
    "................",
    "................",
];

pub const FISH_PIXELS: [&str; 16] = [
    "................",
    "................",
    "................",
    "......kkkk......",
    ".....kBBBBk..k..",
    "....kBBwBBBkkBk.",
    "...kBBBBBBBBBBk.",
    "...kBBBBBBBBBk..",
    "....kBBBBBBkkBk.",
    ".....kBBBBk..k..",
    "......kkkk......",
    "................",
    "................",
    "................",
    "................",
    "................",
];

pub const BREAD_PIXELS: [&str; 16] = [
    "................",
    "................",
    "................",
    "................",
    ".....kkkkkk.....",
    "...kkoooooookk..",
    "..koolloolloook.",
    "..kooooooooooook",
    "..kooooooooooook",
    "..kbbbbbbbbbbbk.",
    "...kkkkkkkkkkk..",
    "................",
    "................",
    "................",
    "................",
    "................",
];

pub const ORE_PIXELS: [&str; 16] = [
    "................",
    "................",
    "................",
    ".......kkk......",
    "......kgggkk....",
    ".....kggyggk....",
    "....kkgggyggk...",
    "...kggkgggggk...",
    "...kgygkgggkk...",
    "...kgggkkgkk....",
    "...kggggggk.....",
    "....kkkkkk......",
    "................",
    "................",
    "................",
    "................",
];

pub const CUSTOM_ICONS: [&[&str; 16]; 5] = [
    &ROD_PIXELS,
    &WOOD_PIXELS,
    &FISH_PIXELS,
    &BREAD_PIXELS,
    &ORE_PIXELS,
];

pub fn palette(character: char) -> Option<[u8; 4]> {
    match character {
        'k' => Some([38, 26, 30, 255]),
        'b' => Some([124, 82, 48, 255]),
        'l' => Some([196, 150, 96, 255]),
        'o' => Some([222, 160, 84, 255]),
        'w' => Some([240, 240, 240, 255]),
        'B' => Some([86, 150, 220, 255]),
        'g' => Some([128, 136, 152, 255]),
        'y' => Some([228, 196, 108, 255]),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum WeatherKind {
    #[default]
    Clear,
    Cloudy,
    Fog,
    Wind,
    Rain,
    Storm,
}

impl WeatherKind {
    pub fn name(self) -> &'static str {
        match self {
            WeatherKind::Clear => "Clear",
            WeatherKind::Cloudy => "Overcast",
            WeatherKind::Fog => "Fog",
            WeatherKind::Wind => "Windy",
            WeatherKind::Rain => "Rain",
            WeatherKind::Storm => "Storm",
        }
    }

    pub fn rains(self) -> bool {
        matches!(self, WeatherKind::Rain | WeatherKind::Storm)
    }

    pub fn blows(self) -> bool {
        matches!(self, WeatherKind::Wind | WeatherKind::Storm)
    }

    pub fn clouds(self) -> bool {
        matches!(
            self,
            WeatherKind::Cloudy | WeatherKind::Rain | WeatherKind::Storm
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Zone {
    #[default]
    Farm,
    Village,
    Pen,
    Forest,
    Quarry,
    Lake,
    Meadow,
    Cliffs,
}

impl Zone {
    pub fn name(self) -> &'static str {
        match self {
            Zone::Farm => "The Farm",
            Zone::Village => "The Village",
            Zone::Pen => "The Pen",
            Zone::Forest => "The Old Forest",
            Zone::Quarry => "The Quarry",
            Zone::Lake => "The Lakeshore",
            Zone::Meadow => "The Meadow",
            Zone::Cliffs => "The Cliffs",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EnemyKind {
    #[default]
    Slime,
    Rat,
    Bat,
    Spider,
    Wolf,
    Ghost,
    Crab,
    Mimic,
}

impl EnemyKind {
    pub const ROSTER: [EnemyKind; 7] = [
        EnemyKind::Slime,
        EnemyKind::Rat,
        EnemyKind::Bat,
        EnemyKind::Spider,
        EnemyKind::Wolf,
        EnemyKind::Ghost,
        EnemyKind::Crab,
    ];

    pub fn name(self) -> &'static str {
        match self {
            EnemyKind::Slime => "slime",
            EnemyKind::Rat => "cave rat",
            EnemyKind::Bat => "bat",
            EnemyKind::Spider => "spider",
            EnemyKind::Wolf => "hollow wolf",
            EnemyKind::Ghost => "wisp",
            EnemyKind::Crab => "ember crab",
            EnemyKind::Mimic => "mimic",
        }
    }

    pub fn tile(self) -> u32 {
        match self {
            EnemyKind::Slime => theme::SLIME_TILE,
            EnemyKind::Rat => theme::RAT_TILE,
            EnemyKind::Bat => theme::BAT_TILE,
            EnemyKind::Spider => theme::SPIDER_TILE,
            EnemyKind::Wolf => theme::WOLF_TILE,
            EnemyKind::Ghost => theme::GHOST_TILE,
            EnemyKind::Crab => theme::CRAB_TILE,
            EnemyKind::Mimic => theme::MIMIC_TILE,
        }
    }

    pub fn hp(self) -> u32 {
        match self {
            EnemyKind::Slime => 3,
            EnemyKind::Rat => 3,
            EnemyKind::Bat => 2,
            EnemyKind::Spider => 4,
            EnemyKind::Wolf => 6,
            EnemyKind::Ghost => 5,
            EnemyKind::Crab => 12,
            EnemyKind::Mimic => 9,
        }
    }

    pub fn damage(self) -> u32 {
        match self {
            EnemyKind::Slime | EnemyKind::Rat | EnemyKind::Bat => 1,
            EnemyKind::Spider | EnemyKind::Ghost => 2,
            EnemyKind::Wolf | EnemyKind::Mimic => 3,
            EnemyKind::Crab => 4,
        }
    }

    pub fn speed(self) -> f32 {
        match self {
            EnemyKind::Slime => 1.2,
            EnemyKind::Rat => 2.4,
            EnemyKind::Bat => 3.2,
            EnemyKind::Spider => 2.0,
            EnemyKind::Wolf => 2.8,
            EnemyKind::Ghost => 1.6,
            EnemyKind::Crab => 1.4,
            EnemyKind::Mimic => 1.8,
        }
    }

    pub fn xp(self) -> u32 {
        match self {
            EnemyKind::Slime | EnemyKind::Rat | EnemyKind::Bat => 2,
            EnemyKind::Spider => 3,
            EnemyKind::Wolf | EnemyKind::Ghost => 5,
            EnemyKind::Crab => 14,
            EnemyKind::Mimic => 8,
        }
    }

    pub fn min_depth(self) -> u32 {
        match self {
            EnemyKind::Slime | EnemyKind::Rat => 1,
            EnemyKind::Bat => 2,
            EnemyKind::Spider => 3,
            EnemyKind::Wolf => 4,
            EnemyKind::Ghost => 6,
            EnemyKind::Crab => 5,
            EnemyKind::Mimic => 1,
        }
    }

    pub fn size(self) -> f32 {
        match self {
            EnemyKind::Crab => 1.3,
            EnemyKind::Bat | EnemyKind::Rat => 0.8,
            _ => 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Weapon {
    #[default]
    Fists,
    Dagger,
    Sword,
    Axe,
    Hammer,
}

impl Weapon {
    pub const ALL: [Weapon; 5] = [
        Weapon::Fists,
        Weapon::Dagger,
        Weapon::Sword,
        Weapon::Axe,
        Weapon::Hammer,
    ];

    pub fn index(self) -> usize {
        Weapon::ALL
            .iter()
            .position(|weapon| *weapon == self)
            .unwrap_or(0)
    }

    pub fn name(self) -> &'static str {
        match self {
            Weapon::Fists => "Fists",
            Weapon::Dagger => "Dagger",
            Weapon::Sword => "Sword",
            Weapon::Axe => "Battle axe",
            Weapon::Hammer => "Warhammer",
        }
    }

    pub fn damage(self) -> u32 {
        match self {
            Weapon::Fists => 1,
            Weapon::Dagger => 2,
            Weapon::Sword => 4,
            Weapon::Axe => 6,
            Weapon::Hammer => 8,
        }
    }

    pub fn tile(self) -> Option<u32> {
        match self {
            Weapon::Fists => None,
            Weapon::Dagger => Some(theme::DAGGER_TILE),
            Weapon::Sword => Some(theme::SWORD_TILE),
            Weapon::Axe => Some(theme::AXE_TILE),
            Weapon::Hammer => Some(theme::HAMMER_TILE),
        }
    }

    pub fn found_at(depth: u32) -> Weapon {
        if depth >= 9 {
            Weapon::Hammer
        } else if depth >= 6 {
            Weapon::Axe
        } else if depth >= 2 {
            Weapon::Sword
        } else {
            Weapon::Dagger
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Potion {
    #[default]
    Heal,
    Strength,
    Speed,
    Grit,
}

impl Potion {
    pub const ALL: [Potion; 4] = [Potion::Heal, Potion::Strength, Potion::Speed, Potion::Grit];

    pub fn name(self) -> &'static str {
        match self {
            Potion::Heal => "healing draught",
            Potion::Strength => "strength tonic",
            Potion::Speed => "quickfoot brew",
            Potion::Grit => "stoneskin salve",
        }
    }

    pub fn tile(self) -> u32 {
        match self {
            Potion::Heal => theme::POTION_HEAL,
            Potion::Strength => theme::POTION_STRENGTH,
            Potion::Speed => theme::POTION_SPEED,
            Potion::Grit => theme::POTION_GRIT,
        }
    }
}

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct SavedPlot {
    pub column: u32,
    pub row: u32,
    pub watered_day: Option<u32>,
    pub crop: Option<usize>,
    pub progress: f32,
}

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Save {
    pub day: u32,
    pub hour: f32,
    pub coins: u32,
    pub counts: Vec<u32>,
    pub tools: Vec<bool>,
    pub levels: Vec<u32>,
    pub slot: usize,
    pub errands: Vec<bool>,
    pub stamina: f32,
    pub hp: f32,
    pub max_hp: u32,
    pub xp: u32,
    pub level: u32,
    pub weapon: usize,
    pub deaths: u32,
    pub deepest: u32,
    pub plots: Vec<SavedPlot>,
}

impl nightshade_save::Save for Save {
    const FOLDER: &'static str = crate::theme::SAVE_FOLDER;
    const NAME: &'static str = crate::theme::SAVE_NAME;
    const VERSION: u32 = crate::theme::SAVE_VERSION;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Walk,
    WalkAlong,
    Act,
    Advance,
    Fight,
    Pocket,
    Pause,
    Quit,
    TurnLeft,
    TurnRight,
    NextTool,
    PreviousTool,
    Slot1,
    Slot2,
    Slot3,
    Slot4,
    Slot5,
    Slot6,
    Slot7,
    Slot8,
    Slot9,
    Slot10,
}

impl Ask {
    pub const SLOTS: [Ask; 10] = [
        Ask::Slot1,
        Ask::Slot2,
        Ask::Slot3,
        Ask::Slot4,
        Ask::Slot5,
        Ask::Slot6,
        Ask::Slot7,
        Ask::Slot8,
        Ask::Slot9,
        Ask::Slot10,
    ];
}

impl nightshade_input::Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::Walk,
            Self::WalkAlong,
            Self::Act,
            Self::Advance,
            Self::Fight,
            Self::Pocket,
            Self::Pause,
            Self::Quit,
            Self::TurnLeft,
            Self::TurnRight,
            Self::NextTool,
            Self::PreviousTool,
            Self::Slot1,
            Self::Slot2,
            Self::Slot3,
            Self::Slot4,
            Self::Slot5,
            Self::Slot6,
            Self::Slot7,
            Self::Slot8,
            Self::Slot9,
            Self::Slot10,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Walk => "walk",
            Self::WalkAlong => "walk along",
            Self::Act => "act",
            Self::Advance => "advance",
            Self::Fight => "fight",
            Self::Pocket => "pocket",
            Self::Pause => "pause",
            Self::Quit => "quit",
            Self::TurnLeft => "turn left",
            Self::TurnRight => "turn right",
            Self::NextTool => "next tool",
            Self::PreviousTool => "previous tool",
            Self::Slot1 => "slot 1",
            Self::Slot2 => "slot 2",
            Self::Slot3 => "slot 3",
            Self::Slot4 => "slot 4",
            Self::Slot5 => "slot 5",
            Self::Slot6 => "slot 6",
            Self::Slot7 => "slot 7",
            Self::Slot8 => "slot 8",
            Self::Slot9 => "slot 9",
            Self::Slot10 => "slot 10",
        }
    }

    fn default_bindings(&self) -> &'static [nightshade_input::Binding] {
        use nightshade_gamepad::contract::GamepadButton;
        use nightshade_input::{Binding, Pointer, Press, Stick};
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
                Binding::Pair {
                    positive: Press::Pad(GamepadButton::DPadRight),
                    negative: Press::Pad(GamepadButton::DPadLeft),
                },
            ],
            Self::WalkAlong => &[Binding::Pair {
                positive: Press::Pad(GamepadButton::DPadUp),
                negative: Press::Pad(GamepadButton::DPadDown),
            }],
            Self::Act => &[
                Binding::Key(KeyCode::Space),
                Binding::Pad(GamepadButton::South),
            ],
            Self::Advance => &[
                Binding::Key(KeyCode::Space),
                Binding::Key(KeyCode::Enter),
                Binding::Pad(GamepadButton::South),
            ],
            Self::Fight => &[
                Binding::Key(KeyCode::KeyF),
                Binding::Pointer(Pointer::Left),
                Binding::Pad(GamepadButton::West),
            ],
            Self::Pocket => &[
                Binding::Key(KeyCode::KeyI),
                Binding::Key(KeyCode::Tab),
                Binding::Pad(GamepadButton::North),
            ],
            Self::Pause => &[
                Binding::Key(KeyCode::Escape),
                Binding::Pad(GamepadButton::Start),
                Binding::Pad(GamepadButton::East),
            ],
            Self::Quit => &[Binding::Key(KeyCode::F4)],
            Self::TurnLeft => &[
                Binding::Key(KeyCode::KeyQ),
                Binding::Key(KeyCode::Comma),
                Binding::Key(KeyCode::PageUp),
                Binding::Pad(GamepadButton::LeftBumper),
            ],
            Self::TurnRight => &[
                Binding::Key(KeyCode::KeyE),
                Binding::Key(KeyCode::Period),
                Binding::Key(KeyCode::PageDown),
                Binding::Pad(GamepadButton::RightBumper),
            ],
            Self::NextTool => &[Binding::Pad(GamepadButton::RightTrigger)],
            Self::PreviousTool => &[Binding::Pad(GamepadButton::LeftTrigger)],
            Self::Slot1 => &[Binding::Key(KeyCode::Digit1)],
            Self::Slot2 => &[Binding::Key(KeyCode::Digit2)],
            Self::Slot3 => &[Binding::Key(KeyCode::Digit3)],
            Self::Slot4 => &[Binding::Key(KeyCode::Digit4)],
            Self::Slot5 => &[Binding::Key(KeyCode::Digit5)],
            Self::Slot6 => &[Binding::Key(KeyCode::Digit6)],
            Self::Slot7 => &[Binding::Key(KeyCode::Digit7)],
            Self::Slot8 => &[Binding::Key(KeyCode::Digit8)],
            Self::Slot9 => &[Binding::Key(KeyCode::Digit9)],
            Self::Slot10 => &[Binding::Key(KeyCode::Digit0)],
        }
    }
}
