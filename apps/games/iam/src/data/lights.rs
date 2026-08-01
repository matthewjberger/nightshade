use crate::data::script::Fact;

pub const BYLINE: &str = "LIGHTS OUT  v1.2  Â·  b.finch (birdfeeder)  Â·  1998";

pub const OTHER_BYLINE: &str = "LIGHTS ON  v1.2  Â·  b.finch (birdfeeder)  Â·  1998";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Goal {
    #[default]
    Out,
    On,
}

pub fn instruction(goal: Goal) -> &'static str {
    match goal {
        Goal::Out => "turn them all off",
        Goal::On => "turn them all on",
    }
}

pub const DARK: &str = "0000000";

pub const BOARDS: &[&str] = &[
    "1011010", "1101101", "0110110", "1111111", "1010101", "0011100",
];

pub const REMARKS: &[&str] = &[
    "Solved. Nice one.",
    "Solved. You are getting the hang of it.",
    "Solved. Most people stop after two.",
    "Solved. Nobody has done four in a sitting before.",
    "Solved. I am running out of things to say here.",
    "Solved.",
];

pub const FINISHERS: usize = 8815;

pub const DOWNLOADED: Fact = "downloaded_lights_out";
pub const DECRYPTED: Fact = "decrypted_the_payload";
pub const TURNED_ON: Fact = "opened_the_case_files";
pub const PLAYED: Fact = "played_birdfeeders_game";
pub const COUNTED: Fact = "knows_the_program_counts_too";

pub fn next_board(done: usize) -> &'static str {
    BOARDS[done % BOARDS.len()]
}

pub fn remark(done: usize) -> &'static str {
    REMARKS[(done.saturating_sub(1)).min(REMARKS.len() - 1)]
}
