use blocker_blocks::data::blocks::BlockKind;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Biome {
    Ocean,
    Beach,
    Plains,
    Meadow,
    Forest,
    BirchForest,
    Swamp,
    Jungle,
    Savanna,
    Desert,
    Badlands,
    Taiga,
    SnowyPlains,
    Tundra,
    Mountain,
    SnowyPeaks,
}

impl Biome {
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn info(self) -> &'static BiomeDef {
        &BIOMES[self.index()]
    }

    pub fn name(self) -> &'static str {
        BIOMES[self.index()].name
    }
}

pub struct BiomeDef {
    pub name: &'static str,
    pub surface: BlockKind,
    pub filler: BlockKind,
    pub tree_chance: u32,
    pub flower_chance: u32,
    pub brush_chance: u32,
    pub village_weight: u32,
}

const fn biome(
    name: &'static str,
    surface: BlockKind,
    filler: BlockKind,
    tree_chance: u32,
    flower_chance: u32,
    brush_chance: u32,
    village_weight: u32,
) -> BiomeDef {
    BiomeDef {
        name,
        surface,
        filler,
        tree_chance,
        flower_chance,
        brush_chance,
        village_weight,
    }
}

pub const BIOMES: [BiomeDef; 16] = [
    biome("Ocean", BlockKind::Sand, BlockKind::Sand, 0, 0, 0, 0),
    biome("Beach", BlockKind::Sand, BlockKind::Sand, 0, 0, 3, 4),
    biome("Plains", BlockKind::Grass, BlockKind::Dirt, 4, 14, 0, 48),
    biome("Meadow", BlockKind::Grass, BlockKind::Dirt, 9, 26, 0, 40),
    biome("Forest", BlockKind::Grass, BlockKind::Dirt, 58, 8, 0, 24),
    biome(
        "Birch Forest",
        BlockKind::Grass,
        BlockKind::Dirt,
        46,
        12,
        0,
        26,
    ),
    biome("Swamp", BlockKind::Grass, BlockKind::Dirt, 22, 4, 6, 10),
    biome("Jungle", BlockKind::Grass, BlockKind::Dirt, 72, 6, 8, 14),
    biome("Savanna", BlockKind::Grass, BlockKind::Dirt, 8, 6, 12, 30),
    biome("Desert", BlockKind::Sand, BlockKind::Sand, 0, 0, 26, 12),
    biome(
        "Badlands",
        BlockKind::RedSand,
        BlockKind::Terracotta,
        0,
        0,
        20,
        8,
    ),
    biome("Taiga", BlockKind::SnowGrass, BlockKind::Dirt, 48, 4, 0, 20),
    biome(
        "Snowy Plains",
        BlockKind::SnowGrass,
        BlockKind::Dirt,
        6,
        3,
        0,
        22,
    ),
    biome("Tundra", BlockKind::Snow, BlockKind::Dirt, 4, 1, 0, 12),
    biome("Mountain", BlockKind::Stone, BlockKind::Stone, 6, 1, 0, 8),
    biome("Snowy Peaks", BlockKind::Snow, BlockKind::Stone, 2, 0, 0, 4),
];

const _: () = assert!(
    Biome::SnowyPeaks as usize + 1 == BIOMES.len(),
    "the Biome variants and the BIOMES rows are not the same count"
);
