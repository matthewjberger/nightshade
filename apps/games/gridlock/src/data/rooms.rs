#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Room {
    #[default]
    Shop,
    Office,
    Warehouse,
    Home,
    Lobby,
}

pub const ROOM_NAMES: [&str; 5] = ["a shop", "an office", "a warehouse", "a home", "a lobby"];

pub const SHOP_ROOMS: [Room; 3] = [Room::Shop, Room::Shop, Room::Office];
pub const WORKS_ROOMS: [Room; 1] = [Room::Warehouse];
pub const HOME_ROOMS: [Room; 1] = [Room::Home];

pub const FLOOR: &str = "block-floor";
pub const WALL: &str = "block-wall";
pub const WALL_WINDOW: &str = "block-wall-window-square";
pub const WALL_DOORWAY: &str = "block-wall-doorway-square";
pub const DOOR_LEAF: &str = "block-door-rotate-square-a";
pub const COLUMN: &str = "block-column";
pub const WINDOW_CHANCE: f32 = 0.4;

pub const SHOP_COUNTER: [&str; 2] = ["furn-kitchenBar", "furn-kitchenBarEnd"];
pub const SHOP_SHELVES: [&str; 3] = [
    "furn-bookcaseOpen",
    "furn-bookcaseClosed",
    "furn-bookcaseClosedWide",
];
pub const SHOP_LOOSE: [&str; 5] = [
    "furn-cardboardBoxClosed",
    "furn-cardboardBoxOpen",
    "furn-pottedPlant",
    "furn-trashcan",
    "furn-stoolBar",
];

pub const OFFICE_WALLS: [&str; 3] = [
    "furn-bookcaseClosed",
    "furn-bookcaseOpen",
    "furn-coatRackStanding",
];
pub const OFFICE_LOOSE: [&str; 6] = [
    "furn-desk",
    "furn-deskCorner",
    "furn-chairDesk",
    "furn-pottedPlant",
    "furn-lampSquareFloor",
    "furn-plantSmall1",
];

pub const WAREHOUSE_WALLS: [&str; 4] = [
    "urban-pallet-small",
    "furn-cardboardBoxClosed",
    "urban-detail-bricks-type-a",
    "urban-detail-block",
];
pub const WAREHOUSE_LOOSE: [&str; 6] = [
    "urban-planks",
    "urban-pallet-small",
    "furn-cardboardBoxOpen",
    "urban-detail-beam",
    "urban-detail-bricks-type-b",
    "furn-deskCorner",
];

pub const HOME_KITCHEN: [&str; 4] = [
    "furn-kitchenCabinet",
    "furn-kitchenStove",
    "furn-kitchenSink",
    "furn-kitchenCabinetDrawer",
];
pub const HOME_BEDROOM: [&str; 3] = ["furn-bedDouble", "furn-cabinetBed", "furn-bedSingle"];
pub const HOME_LOOSE: [&str; 7] = [
    "furn-loungeSofa",
    "furn-loungeChair",
    "furn-tableRound",
    "furn-chairRounded",
    "furn-sideTable",
    "furn-pottedPlant",
    "furn-lampRoundFloor",
];
pub const HOME_RUGS: [&str; 2] = ["furn-rugRectangle", "furn-rugRound"];

pub const LOBBY_WALLS: [&str; 3] = ["furn-benchCushion", "furn-pottedPlant", "furn-loungeChair"];
pub const LOBBY_LOOSE: [&str; 4] = [
    "furn-desk",
    "furn-chairDesk",
    "furn-pottedPlant",
    "furn-plantSmall2",
];
