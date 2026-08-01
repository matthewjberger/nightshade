use crate::data::{Chisel, Stage};
use crate::queries::hands_ladder;
use crate::theme;
use nightshade::ecs::Entity;

pub struct Gallery {
    pub stage: Vec<Stage>,
    pub chisel: Chisel,
    pub ladder: Vec<usize>,
    pub hands: usize,
    pub paused: bool,
    pub era: u32,
    pub thrown: u32,
    pub on_the_frame: bool,
}

impl Default for Gallery {
    fn default() -> Self {
        let ladder = hands_ladder();
        let step = ladder.len().saturating_sub(2);
        let hands = ladder.get(step).copied().unwrap_or(1);
        Self {
            stage: vec![Stage::Empty; theme::STANDS],
            chisel: Chisel::default(),
            ladder,
            hands,
            paused: false,
            era: 0,
            thrown: 0,
            on_the_frame: false,
        }
    }
}

#[derive(Default)]
pub struct Panel {
    pub root: Entity,
    pub rate: Entity,
    pub hands_line: Entity,
    pub stands_line: Entity,
    pub thrown_line: Entity,
    pub frame_lamps: Vec<Entity>,
    pub slot_lamps: Vec<Entity>,
    pub mode: Entity,
    pub crew: Entity,
    pub halt: Entity,
    pub again: Entity,
    pub stripe: usize,
    pub smooth: f32,
    pub last_rate: u32,
    pub last_hands: usize,
    pub last_busy: usize,
    pub last_standing: usize,
    pub last_thrown: u32,
    pub last_paused: bool,
    pub last_stage: Vec<u8>,
}
