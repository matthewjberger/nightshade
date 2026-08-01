#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WorldSide {
    #[default]
    Meadow,
    Quarry,
}

impl WorldSide {
    pub const BOTH: [WorldSide; 2] = [WorldSide::Meadow, WorldSide::Quarry];

    pub fn index(self) -> usize {
        match self {
            WorldSide::Meadow => 0,
            WorldSide::Quarry => 1,
        }
    }

    pub fn other(self) -> WorldSide {
        match self {
            WorldSide::Meadow => WorldSide::Quarry,
            WorldSide::Quarry => WorldSide::Meadow,
        }
    }
}
