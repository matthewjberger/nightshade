use crate::data::shop::GATE_COUNT;

pub const LANES: usize = 3;
pub const STEP_COLUMNS: usize = GATE_COUNT as usize - 1;
pub const COLUMNS: usize = GATE_COUNT as usize + STEP_COLUMNS;
pub const TROVE_GOLD: i64 = 8;
pub const HEARTH_HEAL: i32 = 10;

const BAZAAR_SIGIL: char = '\u{e242}';
const TROVE_SIGIL: char = '\u{e1d6}';
const CAMP_SIGIL: char = '\u{e0d2}';
const ECHO_SIGIL: char = '\u{e0ba}';
const HEARTH_SIGIL: char = '\u{e0d3}';

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Stop {
    #[default]
    Bazaar,
    Trove,
    Camp,
    Echo,
    Hearth,
}

pub const ALL_STOPS: [Stop; 5] = [
    Stop::Bazaar,
    Stop::Trove,
    Stop::Camp,
    Stop::Echo,
    Stop::Hearth,
];

impl Stop {
    pub fn name(self) -> &'static str {
        match self {
            Stop::Bazaar => "Bazaar",
            Stop::Trove => "Trove",
            Stop::Camp => "Camp",
            Stop::Echo => "Echo",
            Stop::Hearth => "Hearth",
        }
    }

    pub fn text(self) -> &'static str {
        match self {
            Stop::Bazaar => "cards and trinkets for gold",
            Stop::Trove => "gold under the stones",
            Stop::Camp => "strike your weakest card",
            Stop::Echo => "an echo hands you a card",
            Stop::Hearth => "a fire mends what the gate took",
        }
    }

    pub fn sigil(self) -> char {
        match self {
            Stop::Bazaar => BAZAAR_SIGIL,
            Stop::Trove => TROVE_SIGIL,
            Stop::Camp => CAMP_SIGIL,
            Stop::Echo => ECHO_SIGIL,
            Stop::Hearth => HEARTH_SIGIL,
        }
    }
}

pub fn gate_at(column: usize) -> Option<u32> {
    if column < COLUMNS && column.is_multiple_of(2) {
        Some(column as u32 / 2 + 1)
    } else {
        None
    }
}

pub fn step_at(column: usize) -> Option<usize> {
    if column < COLUMNS && !column.is_multiple_of(2) {
        Some(column / 2)
    } else {
        None
    }
}
