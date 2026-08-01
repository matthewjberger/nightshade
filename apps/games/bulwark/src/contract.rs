#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Cell {
    pub column: u32,
    pub row: u32,
}

impl Cell {
    pub const fn new(column: u32, row: u32) -> Self {
        Self { column, row }
    }
}

pub const SCENERY_KINDS: usize = 15;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scenery {
    PinePair,
    PineTall,
    PineTrio,
    OakBroad,
    OakPair,
    RocksLow,
    RocksTall,
    RocksPair,
    RocksDome,
    RocksTrio,
    RocksSlab,
    ScrubPair,
    ScrubLow,
    PineStand,
    OakLone,
}

impl Scenery {
    pub const ALL: [Scenery; SCENERY_KINDS] = [
        Scenery::PinePair,
        Scenery::PineTall,
        Scenery::PineTrio,
        Scenery::OakBroad,
        Scenery::OakPair,
        Scenery::RocksLow,
        Scenery::RocksTall,
        Scenery::RocksPair,
        Scenery::RocksDome,
        Scenery::RocksTrio,
        Scenery::RocksSlab,
        Scenery::ScrubPair,
        Scenery::ScrubLow,
        Scenery::PineStand,
        Scenery::OakLone,
    ];

    pub const fn slot(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tower {
    #[default]
    Bastion,
    Longbow,
    Hearth,
    Barbican,
    Beacon,
    Cairn,
}

pub const TOWER_KINDS: usize = 6;

pub const TOWER_LEVELS: usize = 3;

impl Tower {
    pub const ALL: [Tower; TOWER_KINDS] = [
        Tower::Bastion,
        Tower::Longbow,
        Tower::Hearth,
        Tower::Barbican,
        Tower::Beacon,
        Tower::Cairn,
    ];

    pub const fn slot(self) -> usize {
        self as usize
    }

    pub const fn name(self) -> &'static str {
        match self {
            Tower::Bastion => "Bastion",
            Tower::Longbow => "Longbow",
            Tower::Hearth => "Hearth",
            Tower::Barbican => "Barbican",
            Tower::Beacon => "Beacon",
            Tower::Cairn => "Cairn",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tier {
    #[default]
    Shard,
    Cluster,
    Monolith,
}

impl Tier {
    pub const fn lesser(self) -> Tier {
        match self {
            Tier::Monolith => Tier::Cluster,
            _ => Tier::Shard,
        }
    }

    pub const fn slot(self) -> usize {
        self as usize
    }

    pub const fn name(self) -> &'static str {
        match self {
            Tier::Shard => "shards",
            Tier::Cluster => "clusters",
            Tier::Monolith => "monoliths",
        }
    }
}
