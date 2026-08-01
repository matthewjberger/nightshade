use super::schematics::{
    BOULDER_SCHEMATIC, CAMPFIRE_SCHEMATIC, COTTAGE_SCHEMATIC, DEADBUSH_SCHEMATIC, FARM_SCHEMATIC,
    FLOWER_RED_SCHEMATIC, FLOWER_YELLOW_SCHEMATIC, GRAVEYARD_SCHEMATIC, HENGE_SCHEMATIC,
    HOUSE_SCHEMATIC, KEEP_SCHEMATIC, LAMP_SCHEMATIC, LODGE_SCHEMATIC, MARKET_SCHEMATIC,
    OAK_SCHEMATIC, OAK_TALL_SCHEMATIC, OBELISK_SCHEMATIC, PINE_SCHEMATIC, REDROOF_SCHEMATIC,
    RUIN_SCHEMATIC, SHIP_SCHEMATIC, SHRINE_SCHEMATIC, STONEHOUSE_SCHEMATIC, TALLHOUSE_SCHEMATIC,
    TOWER_SCHEMATIC, TOWNHALL_SCHEMATIC, WATCHTOWER_SCHEMATIC, WELL_SCHEMATIC,
};
use blocker_blocks::data::blocks::BlockKind;

pub enum Density {
    Flat(u32),
    Tree,
    Flower,
    Brush,
    Settlement,
    Ocean,
}

pub struct Slot {
    pub dx: i32,
    pub dz: i32,
    pub child: usize,
    pub face: u8,
}

pub struct Street {
    pub from: [i32; 2],
    pub to: [i32; 2],
    pub wide: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WorkKind {
    Market,
    Farm,
    Well,
    Hall,
    Keep,
    Shrine,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlotRole {
    Home,
    Guide,
    Work(WorkKind),
    Light,
}

pub fn slot_role(child: usize) -> SlotRole {
    match child {
        HOUSE_PICK | COTTAGE_STAMP | HOUSE_STAMP | LODGE_STAMP | STONEHOUSE_STAMP
        | REDROOF_STAMP | TALLHOUSE_STAMP => SlotRole::Home,
        GUIDE_STAMP => SlotRole::Guide,
        MARKET_STAMP => SlotRole::Work(WorkKind::Market),
        FARM_STAMP => SlotRole::Work(WorkKind::Farm),
        WELL_STAMP => SlotRole::Work(WorkKind::Well),
        TOWNHALL_STAMP => SlotRole::Work(WorkKind::Hall),
        KEEP_STAMP => SlotRole::Work(WorkKind::Keep),
        SHRINE_STAMP => SlotRole::Work(WorkKind::Shrine),
        _ => SlotRole::Light,
    }
}

pub struct Layout {
    pub span: i32,
    pub slots: &'static [Slot],
    pub streets: &'static [Street],
    pub center: usize,
    pub path: BlockKind,
    pub wall: i32,
}

pub enum NodeKind {
    Base {
        children: &'static [usize],
    },
    Scatter {
        cell: i32,
        density: Density,
        child: usize,
        reach: i32,
        feature: usize,
    },
    Pick {
        options: &'static [(u32, usize)],
    },
    Grove {
        leafy: &'static [usize],
        conifer: &'static [usize],
    },
    Layout(Layout),
    Stamp {
        schematic: usize,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NodeName {
    Root,
    Settlements,
    Ruins,
    Trees,
    Flowers,
    Brush,
    Rocks,
    SettlementPick,
    Hamlet,
    Village,
    Town,
    HousePick,
    CottageStamp,
    HouseStamp,
    LodgeStamp,
    TownhallStamp,
    WellStamp,
    MarketStamp,
    FarmStamp,
    LampStamp,
    Grove,
    OakStamp,
    OakTallStamp,
    PineStamp,
    FlowerPick,
    FlowerRedStamp,
    FlowerYellowStamp,
    BrushStamp,
    BoulderStamp,
    RuinPick,
    TowerStamp,
    ObeliskStamp,
    RuinStamp,
    HengeStamp,
    Hamlets,
    Spots,
    SmallPick,
    Homestead,
    SpotPick,
    WatchtowerStamp,
    GraveyardStamp,
    CampStamp,
    City,
    Capital,
    ShipStamp,
    Ships,
    KeepStamp,
    ShrineStamp,
    GuideStamp,
    StonehouseStamp,
    RedroofStamp,
    TallhouseStamp,
}

pub struct Node {
    pub name: NodeName,
    pub kind: NodeKind,
}

pub const ROOT: usize = NodeName::Root as usize;
const SETTLEMENTS: usize = NodeName::Settlements as usize;
const RUINS: usize = NodeName::Ruins as usize;
const TREES: usize = NodeName::Trees as usize;
const FLOWERS: usize = NodeName::Flowers as usize;
const BRUSH: usize = NodeName::Brush as usize;
const ROCKS: usize = NodeName::Rocks as usize;
pub const SETTLEMENT_PICK: usize = NodeName::SettlementPick as usize;
const HAMLET: usize = NodeName::Hamlet as usize;
const VILLAGE: usize = NodeName::Village as usize;
const TOWN: usize = NodeName::Town as usize;
const HOUSE_PICK: usize = NodeName::HousePick as usize;
const COTTAGE_STAMP: usize = NodeName::CottageStamp as usize;
const HOUSE_STAMP: usize = NodeName::HouseStamp as usize;
const LODGE_STAMP: usize = NodeName::LodgeStamp as usize;
const TOWNHALL_STAMP: usize = NodeName::TownhallStamp as usize;
const WELL_STAMP: usize = NodeName::WellStamp as usize;
const MARKET_STAMP: usize = NodeName::MarketStamp as usize;
const FARM_STAMP: usize = NodeName::FarmStamp as usize;
const LAMP_STAMP: usize = NodeName::LampStamp as usize;
const GROVE: usize = NodeName::Grove as usize;
const OAK_STAMP: usize = NodeName::OakStamp as usize;
const OAK_TALL_STAMP: usize = NodeName::OakTallStamp as usize;
const PINE_STAMP: usize = NodeName::PineStamp as usize;
const FLOWER_PICK: usize = NodeName::FlowerPick as usize;
const FLOWER_RED_STAMP: usize = NodeName::FlowerRedStamp as usize;
const FLOWER_YELLOW_STAMP: usize = NodeName::FlowerYellowStamp as usize;
const BRUSH_STAMP: usize = NodeName::BrushStamp as usize;
const BOULDER_STAMP: usize = NodeName::BoulderStamp as usize;
const RUIN_PICK: usize = NodeName::RuinPick as usize;
const TOWER_STAMP: usize = NodeName::TowerStamp as usize;
const OBELISK_STAMP: usize = NodeName::ObeliskStamp as usize;
const RUIN_STAMP: usize = NodeName::RuinStamp as usize;
pub const HENGE_STAMP: usize = NodeName::HengeStamp as usize;
const HAMLETS: usize = NodeName::Hamlets as usize;
const SPOTS: usize = NodeName::Spots as usize;
pub const SMALL_PICK: usize = NodeName::SmallPick as usize;
const HOMESTEAD: usize = NodeName::Homestead as usize;
const SPOT_PICK: usize = NodeName::SpotPick as usize;
const WATCHTOWER_STAMP: usize = NodeName::WatchtowerStamp as usize;
const GRAVEYARD_STAMP: usize = NodeName::GraveyardStamp as usize;
const CAMP_STAMP: usize = NodeName::CampStamp as usize;
const CITY: usize = NodeName::City as usize;
pub const CAPITAL: usize = NodeName::Capital as usize;
const SHIP_STAMP: usize = NodeName::ShipStamp as usize;
const SHIPS: usize = NodeName::Ships as usize;
pub const CAPITAL_REACH: i32 = 80;
const KEEP_STAMP: usize = NodeName::KeepStamp as usize;
const SHRINE_STAMP: usize = NodeName::ShrineStamp as usize;
const GUIDE_STAMP: usize = NodeName::GuideStamp as usize;
const STONEHOUSE_STAMP: usize = NodeName::StonehouseStamp as usize;
const REDROOF_STAMP: usize = NodeName::RedroofStamp as usize;
const TALLHOUSE_STAMP: usize = NodeName::TallhouseStamp as usize;

const fn slot(dx: i32, dz: i32, child: usize, face: u8) -> Slot {
    Slot {
        dx,
        dz,
        child,
        face,
    }
}

const fn street(from_x: i32, from_z: i32, to_x: i32, to_z: i32, wide: i32) -> Street {
    Street {
        from: [from_x, from_z],
        to: [to_x, to_z],
        wide,
    }
}

const HAMLET_STREETS: [Street; 2] = [street(0, -13, 0, -2, 1), street(0, 2, 0, 13, 1)];

const HAMLET_SLOTS: [Slot; 6] = [
    slot(7, -6, HOUSE_PICK, 3),
    slot(-7, -6, HOUSE_PICK, 1),
    slot(7, 6, HOUSE_PICK, 3),
    slot(-7, 6, HOUSE_PICK, 1),
    slot(2, 0, LAMP_STAMP, 0),
    slot(-2, 0, LAMP_STAMP, 0),
];

const VILLAGE_STREETS: [Street; 4] = [
    street(0, -20, 0, -3, 3),
    street(0, 3, 0, 20, 3),
    street(-12, 0, -3, 0, 1),
    street(3, 0, 12, 0, 1),
];

const VILLAGE_SLOTS: [Slot; 10] = [
    slot(8, -8, HOUSE_PICK, 3),
    slot(-8, -8, HOUSE_PICK, 1),
    slot(8, 8, HOUSE_PICK, 3),
    slot(-8, 8, HOUSE_PICK, 1),
    slot(16, 0, MARKET_STAMP, 0),
    slot(-17, 0, FARM_STAMP, 0),
    slot(2, -14, LAMP_STAMP, 0),
    slot(-2, -14, LAMP_STAMP, 0),
    slot(2, 14, LAMP_STAMP, 0),
    slot(-2, 14, LAMP_STAMP, 0),
];

const TOWN_STREETS: [Street; 4] = [
    street(0, -14, 0, -3, 3),
    street(0, 3, 0, 30, 3),
    street(-30, 0, -3, 0, 3),
    street(3, 0, 30, 0, 3),
];

const TOWN_SLOTS: [Slot; 25] = [
    slot(0, -20, TOWNHALL_STAMP, 2),
    slot(9, -9, HOUSE_PICK, 3),
    slot(-9, -9, HOUSE_PICK, 1),
    slot(9, 9, HOUSE_PICK, 3),
    slot(-9, 9, HOUSE_PICK, 1),
    slot(21, -9, HOUSE_PICK, 2),
    slot(21, 9, HOUSE_PICK, 0),
    slot(-21, -9, HOUSE_PICK, 2),
    slot(-21, 9, HOUSE_PICK, 0),
    slot(9, 21, HOUSE_PICK, 3),
    slot(-9, 21, HOUSE_PICK, 1),
    slot(11, -21, HOUSE_PICK, 3),
    slot(-11, -21, HOUSE_PICK, 1),
    slot(21, 21, MARKET_STAMP, 0),
    slot(-21, -21, MARKET_STAMP, 0),
    slot(-21, 21, FARM_STAMP, 0),
    slot(21, -21, FARM_STAMP, 0),
    slot(2, -14, LAMP_STAMP, 0),
    slot(-2, -14, LAMP_STAMP, 0),
    slot(2, 14, LAMP_STAMP, 0),
    slot(-2, 14, LAMP_STAMP, 0),
    slot(14, 2, LAMP_STAMP, 0),
    slot(14, -2, LAMP_STAMP, 0),
    slot(-14, 2, LAMP_STAMP, 0),
    slot(-14, -2, LAMP_STAMP, 0),
];

const HOMESTEAD_STREETS: [Street; 1] = [street(0, -11, 0, -5, 1)];

const HOMESTEAD_SLOTS: [Slot; 3] = [
    slot(0, 9, FARM_STAMP, 0),
    slot(-8, -6, LAMP_STAMP, 0),
    slot(8, -6, LAMP_STAMP, 0),
];

const CITY_STREETS: [Street; 8] = [
    street(0, -22, 0, -3, 3),
    street(0, 3, 0, 36, 3),
    street(-36, 0, -3, 0, 3),
    street(3, 0, 36, 0, 3),
    street(-20, -20, 20, -20, 1),
    street(-20, 20, 20, 20, 1),
    street(-20, -20, -20, 20, 1),
    street(20, -20, 20, 20, 1),
];

const CITY_SLOTS: [Slot; 25] = [
    slot(0, -28, TOWNHALL_STAMP, 2),
    slot(5, -5, MARKET_STAMP, 0),
    slot(-5, 5, MARKET_STAMP, 0),
    slot(12, -12, HOUSE_PICK, 3),
    slot(-12, -12, HOUSE_PICK, 1),
    slot(12, 12, HOUSE_PICK, 3),
    slot(-12, 12, HOUSE_PICK, 1),
    slot(26, -12, HOUSE_PICK, 3),
    slot(26, 12, HOUSE_PICK, 3),
    slot(-26, -12, HOUSE_PICK, 1),
    slot(-26, 12, HOUSE_PICK, 1),
    slot(12, 26, HOUSE_PICK, 0),
    slot(-12, 26, HOUSE_PICK, 0),
    slot(12, -26, HOUSE_PICK, 2),
    slot(-12, -26, HOUSE_PICK, 2),
    slot(30, 30, FARM_STAMP, 0),
    slot(-30, -30, FARM_STAMP, 0),
    slot(2, -14, LAMP_STAMP, 0),
    slot(-2, -14, LAMP_STAMP, 0),
    slot(2, 14, LAMP_STAMP, 0),
    slot(-2, 14, LAMP_STAMP, 0),
    slot(14, 2, LAMP_STAMP, 0),
    slot(14, -2, LAMP_STAMP, 0),
    slot(-14, 2, LAMP_STAMP, 0),
    slot(-14, -2, LAMP_STAMP, 0),
];

const CAPITAL_STREETS: [Street; 8] = [
    street(0, -29, 0, -3, 3),
    street(0, 3, 0, 31, 3),
    street(-44, 0, -3, 0, 3),
    street(3, 0, 44, 0, 3),
    street(-28, -28, 28, -28, 1),
    street(-28, 28, 28, 28, 1),
    street(-28, -28, -28, 28, 1),
    street(28, -28, 28, 28, 1),
];

pub(super) const CAPITAL_SLOTS: [Slot; 56] = [
    slot(0, -34, KEEP_STAMP, 2),
    slot(0, 34, SHRINE_STAMP, 0),
    slot(-22, -8, TOWNHALL_STAMP, 2),
    slot(5, -5, MARKET_STAMP, 0),
    slot(-5, -5, MARKET_STAMP, 0),
    slot(5, 5, MARKET_STAMP, 0),
    slot(-5, 5, MARKET_STAMP, 0),
    slot(12, -12, HOUSE_PICK, 3),
    slot(-12, -12, HOUSE_PICK, 1),
    slot(12, 12, HOUSE_PICK, 3),
    slot(-12, 12, HOUSE_PICK, 1),
    slot(12, -22, HOUSE_PICK, 3),
    slot(22, -12, HOUSE_PICK, 2),
    slot(22, -22, HOUSE_PICK, 0),
    slot(-12, -22, HOUSE_PICK, 1),
    slot(-22, -22, HOUSE_PICK, 0),
    slot(12, 22, HOUSE_PICK, 3),
    slot(22, 12, HOUSE_PICK, 0),
    slot(22, 22, HOUSE_PICK, 2),
    slot(-12, 22, HOUSE_PICK, 1),
    slot(-22, 12, HOUSE_PICK, 0),
    slot(-22, 22, HOUSE_PICK, 2),
    slot(38, -20, HOUSE_PICK, 3),
    slot(38, -8, HOUSE_PICK, 3),
    slot(38, 8, HOUSE_PICK, 3),
    slot(38, 20, GUIDE_STAMP, 3),
    slot(-38, -20, HOUSE_PICK, 1),
    slot(-38, -8, HOUSE_PICK, 1),
    slot(-38, 8, HOUSE_PICK, 1),
    slot(-38, 20, HOUSE_PICK, 1),
    slot(-22, -38, HOUSE_PICK, 2),
    slot(-10, -38, HOUSE_PICK, 2),
    slot(10, -38, HOUSE_PICK, 2),
    slot(22, -38, HOUSE_PICK, 2),
    slot(-22, 38, HOUSE_PICK, 0),
    slot(-10, 38, HOUSE_PICK, 0),
    slot(10, 38, HOUSE_PICK, 0),
    slot(22, 38, HOUSE_PICK, 0),
    slot(-52, -22, FARM_STAMP, 0),
    slot(52, 22, FARM_STAMP, 0),
    slot(2, -14, LAMP_STAMP, 0),
    slot(-2, -14, LAMP_STAMP, 0),
    slot(2, 14, LAMP_STAMP, 0),
    slot(-2, 14, LAMP_STAMP, 0),
    slot(2, -26, LAMP_STAMP, 0),
    slot(-2, -26, LAMP_STAMP, 0),
    slot(2, 26, LAMP_STAMP, 0),
    slot(-2, 26, LAMP_STAMP, 0),
    slot(14, 2, LAMP_STAMP, 0),
    slot(14, -2, LAMP_STAMP, 0),
    slot(-14, 2, LAMP_STAMP, 0),
    slot(-14, -2, LAMP_STAMP, 0),
    slot(31, 2, LAMP_STAMP, 0),
    slot(31, -2, LAMP_STAMP, 0),
    slot(-31, 2, LAMP_STAMP, 0),
    slot(-31, -2, LAMP_STAMP, 0),
];

const fn stamp_node(name: NodeName, schematic: usize) -> Node {
    Node {
        name,
        kind: NodeKind::Stamp { schematic },
    }
}

const fn scatter_node(
    name: NodeName,
    cell: i32,
    density: Density,
    child: usize,
    reach: i32,
    feature: usize,
) -> Node {
    Node {
        name,
        kind: NodeKind::Scatter {
            cell,
            density,
            child,
            reach,
            feature,
        },
    }
}

const fn pick_node(name: NodeName, options: &'static [(u32, usize)]) -> Node {
    Node {
        name,
        kind: NodeKind::Pick { options },
    }
}

const fn layout_node(
    name: NodeName,
    span: i32,
    slots: &'static [Slot],
    streets: &'static [Street],
    center: usize,
) -> Node {
    Node {
        name,
        kind: NodeKind::Layout(Layout {
            span,
            slots,
            streets,
            center,
            path: BlockKind::Gravel,
            wall: 0,
        }),
    }
}

const fn walled_layout_node(
    name: NodeName,
    span: i32,
    slots: &'static [Slot],
    streets: &'static [Street],
    center: usize,
    wall: i32,
) -> Node {
    Node {
        name,
        kind: NodeKind::Layout(Layout {
            span,
            slots,
            streets,
            center,
            path: BlockKind::Cobblestone,
            wall,
        }),
    }
}

pub const NODES: [Node; 52] = [
    Node {
        name: NodeName::Root,
        kind: NodeKind::Base {
            children: &[
                SETTLEMENTS,
                HAMLETS,
                SPOTS,
                RUINS,
                TREES,
                FLOWERS,
                BRUSH,
                ROCKS,
                SHIPS,
            ],
        },
    },
    scatter_node(
        NodeName::Settlements,
        200,
        Density::Settlement,
        SETTLEMENT_PICK,
        45,
        0,
    ),
    scatter_node(NodeName::Ruins, 96, Density::Flat(6), RUIN_PICK, 8, 1),
    scatter_node(NodeName::Trees, 4, Density::Tree, GROVE, 3, 2),
    scatter_node(NodeName::Flowers, 3, Density::Flower, FLOWER_PICK, 0, 3),
    scatter_node(NodeName::Brush, 4, Density::Brush, BRUSH_STAMP, 0, 4),
    scatter_node(NodeName::Rocks, 30, Density::Flat(12), BOULDER_STAMP, 2, 5),
    pick_node(
        NodeName::SettlementPick,
        &[(3, VILLAGE), (2, TOWN), (1, CITY)],
    ),
    layout_node(
        NodeName::Hamlet,
        26,
        &HAMLET_SLOTS,
        &HAMLET_STREETS,
        WELL_STAMP,
    ),
    layout_node(
        NodeName::Village,
        40,
        &VILLAGE_SLOTS,
        &VILLAGE_STREETS,
        WELL_STAMP,
    ),
    layout_node(NodeName::Town, 60, &TOWN_SLOTS, &TOWN_STREETS, WELL_STAMP),
    pick_node(
        NodeName::HousePick,
        &[
            (3, COTTAGE_STAMP),
            (3, HOUSE_STAMP),
            (2, LODGE_STAMP),
            (3, STONEHOUSE_STAMP),
            (3, REDROOF_STAMP),
            (2, TALLHOUSE_STAMP),
        ],
    ),
    stamp_node(NodeName::CottageStamp, COTTAGE_SCHEMATIC),
    stamp_node(NodeName::HouseStamp, HOUSE_SCHEMATIC),
    stamp_node(NodeName::LodgeStamp, LODGE_SCHEMATIC),
    stamp_node(NodeName::TownhallStamp, TOWNHALL_SCHEMATIC),
    stamp_node(NodeName::WellStamp, WELL_SCHEMATIC),
    stamp_node(NodeName::MarketStamp, MARKET_SCHEMATIC),
    stamp_node(NodeName::FarmStamp, FARM_SCHEMATIC),
    stamp_node(NodeName::LampStamp, LAMP_SCHEMATIC),
    Node {
        name: NodeName::Grove,
        kind: NodeKind::Grove {
            leafy: &[OAK_STAMP, OAK_TALL_STAMP],
            conifer: &[PINE_STAMP],
        },
    },
    stamp_node(NodeName::OakStamp, OAK_SCHEMATIC),
    stamp_node(NodeName::OakTallStamp, OAK_TALL_SCHEMATIC),
    stamp_node(NodeName::PineStamp, PINE_SCHEMATIC),
    pick_node(
        NodeName::FlowerPick,
        &[(1, FLOWER_RED_STAMP), (1, FLOWER_YELLOW_STAMP)],
    ),
    stamp_node(NodeName::FlowerRedStamp, FLOWER_RED_SCHEMATIC),
    stamp_node(NodeName::FlowerYellowStamp, FLOWER_YELLOW_SCHEMATIC),
    stamp_node(NodeName::BrushStamp, DEADBUSH_SCHEMATIC),
    stamp_node(NodeName::BoulderStamp, BOULDER_SCHEMATIC),
    pick_node(
        NodeName::RuinPick,
        &[
            (3, TOWER_STAMP),
            (2, RUIN_STAMP),
            (2, OBELISK_STAMP),
            (1, HENGE_STAMP),
        ],
    ),
    stamp_node(NodeName::TowerStamp, TOWER_SCHEMATIC),
    stamp_node(NodeName::ObeliskStamp, OBELISK_SCHEMATIC),
    stamp_node(NodeName::RuinStamp, RUIN_SCHEMATIC),
    stamp_node(NodeName::HengeStamp, HENGE_SCHEMATIC),
    scatter_node(
        NodeName::Hamlets,
        88,
        Density::Settlement,
        SMALL_PICK,
        20,
        6,
    ),
    scatter_node(NodeName::Spots, 48, Density::Flat(11), SPOT_PICK, 8, 7),
    pick_node(NodeName::SmallPick, &[(3, HAMLET), (2, HOMESTEAD)]),
    layout_node(
        NodeName::Homestead,
        22,
        &HOMESTEAD_SLOTS,
        &HOMESTEAD_STREETS,
        COTTAGE_STAMP,
    ),
    pick_node(
        NodeName::SpotPick,
        &[
            (3, WATCHTOWER_STAMP),
            (3, GRAVEYARD_STAMP),
            (3, CAMP_STAMP),
            (2, RUIN_STAMP),
            (2, OBELISK_STAMP),
            (1, HENGE_STAMP),
        ],
    ),
    stamp_node(NodeName::WatchtowerStamp, WATCHTOWER_SCHEMATIC),
    stamp_node(NodeName::GraveyardStamp, GRAVEYARD_SCHEMATIC),
    stamp_node(NodeName::CampStamp, CAMPFIRE_SCHEMATIC),
    layout_node(NodeName::City, 76, &CITY_SLOTS, &CITY_STREETS, WELL_STAMP),
    walled_layout_node(
        NodeName::Capital,
        110,
        &CAPITAL_SLOTS,
        &CAPITAL_STREETS,
        WELL_STAMP,
        crate::data::world::CITY_WALL,
    ),
    stamp_node(NodeName::ShipStamp, SHIP_SCHEMATIC),
    scatter_node(NodeName::Ships, 360, Density::Ocean, SHIP_STAMP, 6, 8),
    stamp_node(NodeName::KeepStamp, KEEP_SCHEMATIC),
    stamp_node(NodeName::ShrineStamp, SHRINE_SCHEMATIC),
    stamp_node(NodeName::GuideStamp, LODGE_SCHEMATIC),
    stamp_node(NodeName::StonehouseStamp, STONEHOUSE_SCHEMATIC),
    stamp_node(NodeName::RedroofStamp, REDROOF_SCHEMATIC),
    stamp_node(NodeName::TallhouseStamp, TALLHOUSE_SCHEMATIC),
];

const fn nodes_line_up() -> bool {
    let mut index = 0;
    while index < NODES.len() {
        if NODES[index].name as usize != index {
            return false;
        }
        index += 1;
    }
    true
}

const _: () = assert!(nodes_line_up());
