use crate::data::{Ability, Feature};
use crate::theme;
use fastrand::Rng;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::Entity;
use nightshade_cli::clap;
use nightshade_sprite::prelude::{SpriteRegion, TileTrim};

#[derive(clap::Args, Default)]
pub struct SrpgArguments {
    #[arg(long, value_name = "HOUR", help = "Start the mission at this hour")]
    pub hour: Option<f32>,
    #[arg(
        long,
        value_name = "NUMBER",
        help = "Open the campaign at this chapter"
    )]
    pub chapter: Option<usize>,
    #[arg(long, help = "Skip the title and open the prologue")]
    pub prologue: bool,
    #[arg(long, help = "Skip the title and open the camp")]
    pub camp: bool,
    #[arg(long, help = "Skip the title and open the battle")]
    pub battle: bool,
    #[arg(long, help = "Let the planner play both squads")]
    pub auto: bool,
}

#[derive(Default)]
pub struct Battlefield {
    pub camera: Option<Entity>,
    pub sun: Option<Entity>,
    pub start_hour: f32,
    pub fixed_hour: Option<f32>,
    pub auto: bool,
    pub spawn_pending: bool,
    pub bars_shown: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Title,
    Prologue,
    Camp,
    Battle,
    Paused,
    Over,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Pause {
    pub from: Screen,
    pub index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CampPhase {
    #[default]
    Roam,
    Menu,
    Partner,
    Muster,
    Advance,
    Speaking,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Prompt {
    #[default]
    Nothing,
    Talk(Entity, String),
    Enter(crate::data::RoomKind),
}

#[derive(Default)]
pub struct Camp {
    pub feet: Vec3,
    pub direction: Vec2,
    pub confirm: bool,
    pub cancel: bool,
    pub step: i32,
    pub prompt: Prompt,
}

#[derive(Default)]
pub struct Campaign {
    pub chapter: usize,
    pub day: u32,
    pub actions: u32,
    pub supplies: i32,
    pub phase: CampPhase,
    pub selected: Option<Entity>,
    pub partner_of: Option<Entity>,
    pub activity: Option<crate::data::Activity>,
    pub menu_index: usize,
    pub page: usize,
    pub story: Vec<String>,
    pub report: Vec<String>,
    pub speech: Vec<(String, String)>,
    pub said: usize,
    pub briefed: bool,
    pub greeted: Option<Entity>,
    pub arrival: bool,
    pub march: bool,
    pub settled: bool,
    pub over: bool,
    pub keep: bool,
}

#[derive(Default)]
pub struct Terrain {
    pub columns: u32,
    pub rows: u32,
    pub roads: Vec<bool>,
    pub origin: Vec2,
    pub heights: Vec<u32>,
    pub features: Vec<Feature>,
}

#[derive(Clone, Copy)]
pub struct Sheet {
    pub tiles: SpriteRegion,
    pub town: SpriteRegion,
    pub shadow: SpriteRegion,
    pub cell: SpriteRegion,
    pub trim: [TileTrim; theme::SHEET_TILES],
    pub town_trim: [TileTrim; theme::TOWN_TILES],
}

impl Default for Sheet {
    fn default() -> Self {
        Self {
            tiles: SpriteRegion::FULL,
            town: SpriteRegion::FULL,
            shadow: SpriteRegion::FULL,
            cell: SpriteRegion::FULL,
            trim: [TileTrim::default(); theme::SHEET_TILES],
            town_trim: [TileTrim::default(); theme::TOWN_TILES],
        }
    }
}

pub struct CameraRig {
    pub yaw_index: i32,
    pub yaw: f32,
    pub pitch_index: usize,
    pub distance_index: usize,
    pub distance: f32,
    pub pitch: f32,
    pub focus: Vec3,
    pub target: Vec3,
    pub eye: Vec3,
    pub settled_yaw: f32,
    pub frame: Option<f32>,
}

impl Default for CameraRig {
    fn default() -> Self {
        Self {
            yaw_index: 0,
            yaw: 0.0,
            pitch_index: 0,
            distance_index: 1,
            distance: theme::CAMERA_DISTANCES[1],
            pitch: theme::CAMERA_PITCHES[0].to_radians(),
            focus: Vec3::zeros(),
            target: Vec3::zeros(),
            eye: Vec3::zeros(),
            settled_yaw: f32::NAN,
            frame: None,
        }
    }
}

#[derive(Default)]
pub struct Tracks {
    pub camp: Option<Entity>,
    pub battle: Option<Entity>,
    pub playing: Option<Screen>,
}

#[derive(Default)]
pub struct Cursor {
    pub column: u32,
    pub row: u32,
    pub held: Option<(i32, i32)>,
    pub step: (i32, i32),
    pub repeat: f32,
    pub confirm: bool,
    pub cancel: bool,
    pub pulse: f32,
    pub hovered: Option<(u32, u32)>,
    pub sounded: Option<(u32, u32)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Phase {
    #[default]
    Ticking,
    TurnStart,
    Menu,
    ChooseMove,
    Walking,
    ChooseAbility,
    ChooseTarget,
    Acting,
    TurnEnd,
    Thinking,
}

#[derive(Clone, Copy, Debug)]
pub struct Reach {
    pub column: u32,
    pub row: u32,
    pub steps: u32,
    pub from: Option<(u32, u32)>,
}

#[derive(Clone, Copy, Debug)]
pub struct Plan {
    pub destination: (u32, u32),
    pub action: Option<(Ability, (u32, u32))>,
    pub capture: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Hit {
    pub entity: Entity,
    pub amount: i32,
    pub landed: bool,
    pub heals: bool,
}

#[derive(Default)]
pub struct Battle {
    pub phase: Phase,
    pub active: Option<Entity>,
    pub moved: bool,
    pub acted: bool,
    pub menu_index: usize,
    pub ability_index: usize,
    pub reachable: Vec<Reach>,
    pub targets: Vec<(u32, u32)>,
    pub ability: Option<Ability>,
    pub target_tile: Option<(u32, u32)>,
    pub path: Vec<(u32, u32)>,
    pub walk_progress: f32,
    pub timer: f32,
    pub rng: Rng,
    pub log: Vec<String>,
    pub outcome: Option<crate::data::Team>,
    pub plan: Option<Plan>,
    pub hits: Vec<Hit>,
    pub hits_rolled: bool,
    pub hits_applied: bool,
    pub counters_applied: bool,
    pub turn: u32,
    pub start_column: u32,
    pub start_row: u32,
    pub start_facing: crate::data::Facing,
    pub danger_shown: bool,
}

#[derive(Default)]
pub struct Highlights {
    pub move_tiles: Vec<(u32, u32)>,
    pub target_tiles: Vec<(u32, u32)>,
    pub area_tiles: Vec<(u32, u32)>,
    pub path_tiles: Vec<(u32, u32)>,
    pub danger_tiles: Vec<(u32, u32)>,
    pub version: u32,
    pub shown_version: u32,
    pub markers: Vec<Entity>,
}

#[derive(Default, Clone)]
pub struct UiHandles {
    pub unit_card: Option<Entity>,
    pub unit_name: Option<Entity>,
    pub unit_class: Option<Entity>,
    pub unit_hp: Option<Entity>,
    pub unit_hp_fill: Option<Entity>,
    pub unit_mp: Option<Entity>,
    pub unit_mp_fill: Option<Entity>,
    pub unit_stats: Option<Entity>,
    pub unit_movement: Option<Entity>,
    pub unit_status: Option<Entity>,
    pub tile_card: Option<Entity>,
    pub tile_label: Option<Entity>,
    pub tile_site: Option<Entity>,
    pub tile_unit: Option<Entity>,
    pub tile_hp: Option<Entity>,
    pub order_panel: Option<Entity>,
    pub order_title: Option<Entity>,
    pub order_rows: Vec<Entity>,
    pub camp_panel: Option<Entity>,
    pub camp_chapter: Option<Entity>,
    pub camp_day: Option<Entity>,
    pub camp_supplies: Option<Entity>,
    pub camp_objective: Option<Entity>,
    pub camp_brief: Option<Entity>,
    pub camp_prompt: Option<Entity>,
    pub fighter_panel: Option<Entity>,
    pub fighter_name: Option<Entity>,
    pub fighter_class: Option<Entity>,
    pub fighter_level: Option<Entity>,
    pub fighter_level_fill: Option<Entity>,
    pub fighter_rapport: Option<Entity>,
    pub fighter_rapport_fill: Option<Entity>,
    pub fighter_stats: Option<Entity>,
    pub fighter_state: Option<Entity>,
    pub fighter_bond: Option<Entity>,
    pub layout_root: Option<Entity>,
    pub speech_panel: Option<Entity>,
    pub speech_name: Option<Entity>,
    pub speech_line: Option<Entity>,
    pub speech_hint: Option<Entity>,
    pub fighter_arc: Option<Entity>,
    pub fighter_arc_line: Option<Entity>,
    pub camp_menu_panel: Option<Entity>,
    pub camp_menu_title: Option<Entity>,
    pub camp_menu_buttons: Vec<Entity>,
    pub march_button: Option<Entity>,
    pub shown_camp_phase: Option<CampPhase>,
    pub shown_camp_rows: usize,
    pub shown_fighter: Option<bool>,
    pub menu_panel: Option<Entity>,
    pub menu_title: Option<Entity>,
    pub menu_buttons: Vec<Entity>,
    pub ability_panel: Option<Entity>,
    pub ability_title: Option<Entity>,
    pub ability_buttons: Vec<Entity>,
    pub log_panel: Option<Entity>,
    pub log_rows: Vec<Entity>,
    pub predict_panel: Option<Entity>,
    pub predict_label: Option<Entity>,
    pub predict_detail: Option<Entity>,
    pub title_panel: Option<Entity>,
    pub continue_button: Option<Entity>,
    pub start_button: Option<Entity>,
    pub title_quit_button: Option<Entity>,
    pub pause_panel: Option<Entity>,
    pub pause_buttons: [Option<Entity>; 3],
    pub over_panel: Option<Entity>,
    pub over_label: Option<Entity>,
    pub over_detail: Option<Entity>,
    pub over_rows: Vec<Entity>,
    pub prologue_panel: Option<Entity>,
    pub prologue_heading: Option<Entity>,
    pub prologue_rows: Vec<Entity>,
    pub prologue_next: Option<Entity>,
    pub prologue_skip: Option<Entity>,
    pub shown_page: Option<usize>,
    pub retry_button: Option<Entity>,
    pub over_quit_button: Option<Entity>,
    pub shown: Option<Screen>,
    pub shown_phase: Option<Phase>,
    pub predict_visible: bool,
}
