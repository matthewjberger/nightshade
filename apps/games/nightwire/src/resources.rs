use crate::data::{CARD_SLOTS, LINE_SLOTS, REPLY_SLOTS, SHIFT_START_MINUTES};
use nightshade::prelude::Entity;
use nightshade_cli::clap;
use std::collections::VecDeque;

#[derive(clap::Args, Default, Clone)]
pub struct Arguments {
    #[arg(long, help = "Open on the switchboard instead of the title screen")]
    pub shift: bool,
    #[arg(long, help = "Open with the CRT glass switched off")]
    pub no_glass: bool,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Title,
    Shift,
    Over,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum CallPhase {
    #[default]
    Speaking,
    Asking,
    Deciding,
    Closed,
}

#[derive(Default, Clone, Copy)]
pub struct UiHandles {
    pub title: Entity,
    pub begin_button: Entity,
    pub quit_button: Entity,
    pub shift: Entity,
    pub clock: Entity,
    pub caller: Entity,
    pub notebook_note: Entity,
    pub send_button: Entity,
    pub hold_button: Entity,
    pub advance_button: Entity,
    pub glass_button: Entity,
    pub over: Entity,
    pub again_button: Entity,
    pub over_title_button: Entity,
}

pub struct NightState {
    pub screen: Screen,
    pub call: usize,
    pub phase: CallPhase,
    pub pending_phase: Option<CallPhase>,
    pub minutes: u32,
    pub lines: Vec<String>,
    pub queue: VecDeque<String>,
    pub revealed: usize,
    pub reveal_timer: f32,
    pub caret_on: bool,
    pub caret_timer: f32,
    pub cards: [bool; CARD_SLOTS],
    pub spent: [bool; REPLY_SLOTS],
    pub raised: [bool; CARD_SLOTS],
    pub deputy_out: bool,
    pub sent_to_water: bool,
    pub knows_truck_empty: bool,
}

impl Default for NightState {
    fn default() -> Self {
        Self {
            screen: Screen::Title,
            call: 0,
            phase: CallPhase::Speaking,
            pending_phase: None,
            minutes: SHIFT_START_MINUTES,
            lines: Vec::new(),
            queue: VecDeque::new(),
            revealed: 0,
            reveal_timer: 0.0,
            caret_on: true,
            caret_timer: 0.0,
            cards: [false; CARD_SLOTS],
            spent: [false; REPLY_SLOTS],
            raised: [false; CARD_SLOTS],
            deputy_out: false,
            sent_to_water: false,
            knows_truck_empty: false,
        }
    }
}

pub struct BoardMirror {
    pub rows: Vec<String>,
}

impl Default for BoardMirror {
    fn default() -> Self {
        Self {
            rows: vec![String::new(); LINE_SLOTS],
        }
    }
}

#[derive(Default)]
pub struct NightwireResources {
    pub night: NightState,
    pub ui: UiHandles,
    pub mirror: BoardMirror,
    pub shown_screen: Option<Screen>,
    pub rows_dirty: bool,
    pub controls_dirty: bool,
    pub glass_on: bool,
    pub glass_amount: f32,
}
