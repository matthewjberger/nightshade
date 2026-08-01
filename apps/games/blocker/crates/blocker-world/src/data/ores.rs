use blocker_blocks::data::blocks::BlockKind;

pub struct Vein {
    pub kind: BlockKind,
    pub low: i32,
    pub high: i32,
    pub chance: u32,
    pub run: i32,
}

pub const VEINS: [Vein; 4] = [
    Vein {
        kind: BlockKind::CoalOre,
        low: 14,
        high: 72,
        chance: 42,
        run: 3,
    },
    Vein {
        kind: BlockKind::IronOre,
        low: 6,
        high: 56,
        chance: 26,
        run: 3,
    },
    Vein {
        kind: BlockKind::GoldOre,
        low: 3,
        high: 30,
        chance: 9,
        run: 2,
    },
    Vein {
        kind: BlockKind::DiamondOre,
        low: 1,
        high: 15,
        chance: 3,
        run: 2,
    },
];

pub const VEIN_ROLL: u32 = 10_000;

const fn bands_are_sane() -> bool {
    let mut index = 0;
    while index < VEINS.len() {
        if VEINS[index].low >= VEINS[index].high || VEINS[index].run < 1 {
            return false;
        }
        index += 1;
    }
    true
}

const _: () = assert!(bands_are_sane());
