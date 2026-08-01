use crate::data;

pub struct RoundState {
    pub score: u32,
    pub lives: u32,
    pub bricks_standing: u32,
    pub serving: bool,
    pub rebuild_wanted: bool,
    pub seed: u32,
}

impl Default for RoundState {
    fn default() -> Self {
        Self {
            score: 0,
            lives: data::STARTING_LIVES,
            bricks_standing: 0,
            serving: true,
            rebuild_wanted: true,
            seed: 0x9e37_79b9,
        }
    }
}
