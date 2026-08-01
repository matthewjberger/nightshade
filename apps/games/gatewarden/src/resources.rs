use crate::contract::{Outcome, Screen};
use crate::data::battle::Battle;
use crate::data::cards::CardId;
use crate::data::climb::{LANES, STEP_COLUMNS, Stop};
use crate::data::decks::{STARTING_HEALTH, starting_deck};
use crate::data::relics::Relic;
use crate::data::shop::{GATE_COUNT, Offer, RELIC_SLOTS, REROLL_COST, STARTING_GOLD, STRIKE_COST};
use crate::data::story::{Ending, Shard};
use crate::theme::{LOG_LINES, OFFER_COUNT};
use nightshade::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

pub const DEFAULT_SEED: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Climb,
    Story,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum SceneKind {
    #[default]
    Prologue,
    Arrival(u32),
    Victory {
        gate: u32,
        spared: bool,
    },
    Ending(Ending),
    Yard,
    YardEnd,
    Chapter(usize),
    ChapterWon(usize),
    StoryEnd,
}

#[derive(Default, Clone, Copy)]
pub struct Tutorial {
    pub active: bool,
    pub step: usize,
    pub refusals: usize,
    pub swallow_pick: bool,
}

#[derive(Default, Clone)]
pub struct Report {
    pub gate: u32,
    pub turns: u32,
    pub warden: String,
    pub deck: usize,
    pub gold: i64,
}

pub struct Run {
    pub gate: u32,
    pub gold: i64,
    pub health: i32,
    pub deck: Vec<CardId>,
    pub forged: Vec<CardId>,
    pub relics: Vec<Relic>,
    pub seed: u64,
    pub offers: [Option<Offer>; OFFER_COUNT],
    pub reroll_cost: i64,
    pub strike_cost: i64,
    pub outcome: Outcome,
    pub final_gate: u32,
    pub shop_generation: u32,
    pub scene: SceneKind,
    pub mercy: u32,
    pub shards: Vec<Shard>,
    pub borrowed_bolt: bool,
    pub warden_bonus: i32,
    pub ending: Ending,
    pub roster: [u8; GATE_COUNT as usize],
    pub column: usize,
    pub stops: [[Stop; LANES]; STEP_COLUMNS],
    pub roads: [Option<u8>; STEP_COLUMNS],
    pub trail: String,
    pub mode: Mode,
    pub chapter: usize,
    pub report: Report,
}

impl Default for Run {
    fn default() -> Self {
        Self {
            gate: 1,
            gold: STARTING_GOLD,
            health: STARTING_HEALTH,
            deck: starting_deck(),
            forged: Vec::new(),
            relics: Vec::with_capacity(RELIC_SLOTS),
            seed: DEFAULT_SEED,
            offers: [None; OFFER_COUNT],
            reroll_cost: REROLL_COST,
            strike_cost: STRIKE_COST,
            outcome: Outcome::Lost,
            final_gate: 1,
            shop_generation: 1,
            scene: SceneKind::Prologue,
            mercy: 0,
            shards: Vec::new(),
            borrowed_bolt: false,
            warden_bonus: 0,
            ending: Ending::Summit,
            roster: [0; GATE_COUNT as usize],
            column: 0,
            stops: [[Stop::Bazaar; LANES]; STEP_COLUMNS],
            roads: [None; STEP_COLUMNS],
            trail: String::new(),
            mode: Mode::Climb,
            chapter: 0,
            report: Report::default(),
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    #[default]
    None,
    Attacker(u32),
    Spell(u32),
    Power,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Zone {
    #[default]
    Hand,
    Board,
    Enemy,
}

#[derive(Default, Clone, Copy)]
pub struct Cursor {
    pub zone: Zone,
    pub index: usize,
    pub active: bool,
    pub stick_latched: i32,
}

#[derive(Clone, Copy)]
pub struct Lunge {
    pub id: u32,
    pub from: Vec2,
    pub to: Vec2,
    pub progress: f32,
}

#[derive(Default, Clone)]
pub struct LogLine {
    pub text: String,
    pub turn: bool,
}

#[derive(Default)]
pub struct Theater {
    pub delay: f32,
    pub think: f32,
    pub lunge: Option<Lunge>,
    pub banner: String,
    pub banner_timer: f32,
    pub shown_player_health: i32,
    pub shown_enemy_health: i32,
    pub revealed: HashSet<u32>,
    pub ending: bool,
    pub log: VecDeque<LogLine>,
}

#[derive(Default)]
pub struct Table {
    pub generation: u32,
    pub last_positions: HashMap<u32, Vec2>,
    pub retired: Vec<Entity>,
    pub hovered_card: Option<CardId>,
    pub pending_floaters: Vec<(Option<u32>, Vec2, String, Vec4)>,
    pub held_slots: HashMap<u32, Vec2>,
    pub shop_generation: u32,
}

#[derive(Default, Clone, Copy)]
pub struct Frames {
    pub panel: Option<u32>,
    pub button: Option<u32>,
    pub card: Option<u32>,
}

#[derive(Default, Clone, Copy)]
pub struct UiHandles {
    pub frames: Frames,
    pub layout_root: Option<Entity>,
    pub table_surface: Option<Entity>,
    pub hand_surface: Option<Entity>,
    pub divider: Option<Entity>,

    pub title_panel: Option<Entity>,
    pub new_run_button: Option<Entity>,
    pub learn_button: Option<Entity>,
    pub quit_button: Option<Entity>,

    pub lesson_panel: Option<Entity>,
    pub lesson_text: Option<Entity>,
    pub lesson_next_button: Option<Entity>,
    pub lesson_hint: Option<Entity>,

    pub strip: Option<Entity>,
    pub gate_text: Option<Entity>,
    pub gold_text: Option<Entity>,
    pub shard_text: Option<Entity>,

    pub scene_panel: Option<Entity>,
    pub scene_portrait: Option<Entity>,
    pub scene_sigil: Option<Entity>,
    pub scene_speaker: Option<Entity>,
    pub scene_lines: [Option<Entity>; 4],
    pub scene_prompt: Option<Entity>,
    pub scene_first_text: Option<Entity>,
    pub scene_second_text: Option<Entity>,
    pub scene_choices: Option<Entity>,
    pub scene_first_button: Option<Entity>,
    pub scene_first_label: Option<Entity>,
    pub scene_second_button: Option<Entity>,
    pub scene_second_label: Option<Entity>,
    pub scene_continue_button: Option<Entity>,

    pub map_panel: Option<Entity>,
    pub map_line: Option<Entity>,
    pub map_note: Option<Entity>,
    pub map_gate_nodes: [Option<Entity>; GATE_COUNT as usize],
    pub map_gate_frames: [Option<Entity>; GATE_COUNT as usize],
    pub map_gate_sigils: [Option<Entity>; GATE_COUNT as usize],
    pub map_stop_nodes: [[Option<Entity>; LANES]; STEP_COLUMNS],
    pub map_stop_frames: [[Option<Entity>; LANES]; STEP_COLUMNS],
    pub map_stop_sigils: [[Option<Entity>; LANES]; STEP_COLUMNS],
    pub map_stop_names: [[Option<Entity>; LANES]; STEP_COLUMNS],
    pub map_climb_button: Option<Entity>,
    pub map_climb_label: Option<Entity>,

    pub relic_row: Option<Entity>,
    pub relic_slots: [Option<Entity>; RELIC_SLOTS],
    pub relic_names: [Option<Entity>; RELIC_SLOTS],
    pub relic_texts: [Option<Entity>; RELIC_SLOTS],

    pub arena: Option<Entity>,
    pub enemy_hero: Option<Entity>,
    pub enemy_portrait: Option<Entity>,
    pub enemy_sigil: Option<Entity>,
    pub enemy_name: Option<Entity>,
    pub enemy_title: Option<Entity>,
    pub enemy_shard: Option<Entity>,
    pub enemy_health: Option<Entity>,
    pub enemy_health_fill: Option<Entity>,
    pub enemy_line: Option<Entity>,
    pub enemy_intent: Option<Entity>,
    pub gate_line: Option<Entity>,
    pub enemy_hand_backs: [Option<Entity>; 8],
    pub player_hero: Option<Entity>,
    pub player_name: Option<Entity>,
    pub player_health: Option<Entity>,
    pub player_health_fill: Option<Entity>,
    pub player_mana_fill: Option<Entity>,
    pub player_mana: Option<Entity>,
    pub player_line: Option<Entity>,
    pub preview_panel: Option<Entity>,
    pub preview_name: Option<Entity>,
    pub preview_meta: Option<Entity>,
    pub preview_stats: Option<Entity>,
    pub preview_text: Option<Entity>,
    pub preview_note: Option<Entity>,
    pub arrow_canvas: Option<Entity>,
    pub log_panel: Option<Entity>,
    pub log_lines: [Option<Entity>; LOG_LINES],
    pub controls: Option<Entity>,
    pub end_turn_button: Option<Entity>,
    pub end_turn_label: Option<Entity>,
    pub power_button: Option<Entity>,
    pub power_label: Option<Entity>,
    pub turn_text: Option<Entity>,
    pub power_text: Option<Entity>,
    pub hint_text: Option<Entity>,
    pub enemy_hand_row: Option<Entity>,

    pub banner: Option<Entity>,
    pub banner_text: Option<Entity>,

    pub shop_panel: Option<Entity>,
    pub shop_gold_text: Option<Entity>,
    pub offer_names: [Option<Entity>; OFFER_COUNT],
    pub offer_effects: [Option<Entity>; OFFER_COUNT],
    pub offer_areas: [Option<Entity>; OFFER_COUNT],
    pub offer_buttons: [Option<Entity>; OFFER_COUNT],
    pub offer_labels: [Option<Entity>; OFFER_COUNT],
    pub reroll_button: Option<Entity>,
    pub reroll_label: Option<Entity>,
    pub strike_button: Option<Entity>,
    pub strike_label: Option<Entity>,
    pub next_button: Option<Entity>,

    pub pause_scrim: Option<Entity>,
    pub pause_resume_button: Option<Entity>,
    pub pause_menu_button: Option<Entity>,
    pub pause_quit_button: Option<Entity>,

    pub over_panel: Option<Entity>,
    pub over_title: Option<Entity>,
    pub over_text: Option<Entity>,
    pub over_stats: Option<Entity>,
    pub over_deck: Option<Entity>,
    pub over_boon: Option<Entity>,
    pub story_button: Option<Entity>,
    pub over_new_run_button: Option<Entity>,
    pub over_new_run_label: Option<Entity>,
    pub over_title_button: Option<Entity>,
}

#[derive(Default)]
pub struct GatewardenResources {
    pub best_gate: u32,
    pub screen: Screen,
    pub run: Run,
    pub battle: Option<Battle>,
    pub selection: Selection,
    pub cursor: Cursor,
    pub theater: Theater,
    pub table: Table,
    pub ui: UiHandles,
    pub camera: Option<Entity>,
    pub tutorial: Tutorial,
    pub paused: bool,
}
