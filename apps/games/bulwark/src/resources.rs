use crate::contract::{Cell, SCENERY_KINDS, TOWER_KINDS, TOWER_LEVELS, Tower};
use crate::data::Piece;
use nightshade::prelude::Entity;
use nightshade_cli::clap;

#[derive(clap::ValueEnum, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opening {
    #[default]
    Title,
    Settings,
    Board,
}

#[derive(clap::Args, Default, Clone, Debug)]
pub struct Arguments {
    #[arg(long, value_enum, default_value_t = Opening::Title, help = "Which screen to open on")]
    pub open: Opening,
    #[arg(long, default_value_t = 0, help = "Which board to lay out")]
    pub board: usize,
    #[arg(
        long,
        default_value_t = 1,
        help = "Which wave the board opens on, with a purse to match"
    )]
    pub wave: usize,
    #[arg(
        long,
        default_value_t = 0,
        help = "How many towers stand beside the road at the start"
    )]
    pub wall: usize,
}

#[derive(Default, Clone, Copy)]
pub struct Art {
    pub grass: Piece,
    pub road: [Piece; 16],
    pub scenery: [Piece; SCENERY_KINDS],
    pub towers: [[Piece; TOWER_LEVELS]; TOWER_KINDS],
    pub shards: [Piece; 3],
    pub keep: Piece,
    pub marker: Piece,
    pub ring: Piece,
    pub bead: Piece,
    pub bar: Piece,
    pub shade: Piece,
    pub backdrop: Piece,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    #[default]
    Title,
    Options,
    Playing,
    Paused,
    Defeat,
    Victory,
}

#[derive(Default, Clone, Copy)]
pub struct Chips {
    pub coin: Entity,
    pub lives: Entity,
    pub wave: Entity,
    pub kills: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct Slot {
    pub panel: Entity,
    pub name: Entity,
    pub cost: Entity,
    pub blurb: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct Inspect {
    pub panel: Entity,
    pub name: Entity,
    pub stats: Entity,
    pub reach: Entity,
    pub upgrade: Entity,
    pub aim: Entity,
    pub sell: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct Hud {
    pub root: Entity,
    pub chips: Chips,
    pub slots: [Slot; TOWER_KINDS],
    pub inspect: Inspect,
    pub banner: Entity,
    pub pace: Entity,
    pub banner_text: Entity,
    pub banner_note: Entity,
    pub send: Entity,
    pub mend: Entity,
    pub hold: Entity,
}

#[derive(Default, Clone)]
pub struct Screens {
    pub dim: Entity,
    pub play: Entity,
    pub title_board: Entity,
    pub title_best: Entity,
    pub title_options: Entity,
    pub title_quit: Entity,
    pub options_back: Entity,
    pub resume: Entity,
    pub pause_options: Entity,
    pub pause_title: Entity,
    pub lost_again: Entity,
    pub lost_title: Entity,
    pub lost_line: Entity,
    pub won_again: Entity,
    pub won_title: Entity,
    pub won_line: Entity,
    pub came_from: Screen,
}

#[derive(Default)]
pub struct Bulwark {
    pub camera: Option<Entity>,
    pub keep: Option<Entity>,
    pub marker: Option<Entity>,
    pub ring: Option<Entity>,
    pub ghost: Option<Entity>,
    pub hovered: Option<Cell>,
    pub buildable: bool,
    pub taken: Vec<Option<Entity>>,
    pub chosen: Tower,
    pub board: usize,
    pub laid: Option<usize>,
    pub pace_step: usize,
    pub last_pace: usize,
    pub selected: Option<Entity>,
    pub coin: u32,
    pub best: u32,
    pub lives: u32,
    pub kills: u32,
    pub mends: u32,
    pub leaked: u32,
    pub shake: f32,
    pub art: Art,
    pub hud: Hud,
    pub screens: Screens,
    pub last_chips: [String; 4],
    pub last_banner: String,
    pub last_note: String,
    pub last_send: String,
    pub last_mend: String,
    pub last_hotbar: Option<(Tower, u32)>,
    pub last_inspect: bool,
    pub last_dim: bool,
    pub last_shown: Option<(Entity, u8, u8, u32, u32)>,
}
