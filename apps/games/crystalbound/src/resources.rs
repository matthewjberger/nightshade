use crate::contract::{BattleCommand, Job, MonsterKind, Place, StationKind};
use crate::data;
use crate::theme;
use nalgebra_glm::Vec2;
use nightshade::ecs::Entity;
use nightshade_cli::clap;
use nightshade_sprite::prelude::SpriteRegion;

#[derive(clap::Args, Default)]
pub struct Arguments {
    #[arg(
        long,
        value_name = "PLACE",
        help = "Open in overworld, town, or dungeon"
    )]
    pub place: Option<String>,
    #[arg(long, help = "Open straight into a fight")]
    pub battle: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Title,
    Field,
    Battle,
    Shop,
    Jobs,
    Paused,
    Defeat,
}

#[derive(Default)]
pub struct ScreenRequest {
    pub wanted: Option<Screen>,
}

pub struct ShownScreen(pub Screen);

impl Default for ShownScreen {
    fn default() -> Self {
        Self(Screen::Title)
    }
}

#[derive(Default, Clone, Copy)]
pub struct Sheets {
    pub town: SpriteRegion,
    pub dungeon: SpriteRegion,
    pub water: SpriteRegion,
    pub gem: SpriteRegion,
    pub shade: SpriteRegion,
}

impl Sheets {
    pub fn town_tile(&self, index: u32) -> SpriteRegion {
        self.town
            .cell(theme::SHEET_COLUMNS, theme::SHEET_ROWS, index)
    }

    pub fn dungeon_tile(&self, index: u32) -> SpriteRegion {
        self.dungeon
            .cell(theme::SHEET_COLUMNS, theme::SHEET_ROWS, index)
    }
}

pub struct Rig {
    pub camera: Option<Entity>,
    pub walker: Option<Entity>,
    pub sheets: Sheets,
    pub rng: fastrand::Rng,
    pub focus: Vec2,
    pub zoom: f32,
    pub shake: f32,
    pub shake_phase: f32,
    pub snap: bool,
    pub bounds: Option<(Vec2, Vec2)>,
}

impl Default for Rig {
    fn default() -> Self {
        Self {
            camera: None,
            walker: None,
            sheets: Sheets::default(),
            rng: fastrand::Rng::with_seed(1),
            focus: Vec2::zeros(),
            zoom: 1.0,
            shake: 0.0,
            shake_phase: 0.0,
            snap: true,
            bounds: None,
        }
    }
}

pub struct Portal {
    pub tile: Vec2,
    pub to: Place,
}

#[derive(Default)]
pub struct Field {
    pub place: Place,
    pub columns: u32,
    pub rows: u32,
    pub blocked: Vec<bool>,
    pub portals: Vec<Portal>,
    pub armed: bool,
    pub pending: Option<(Place, Option<Vec2>)>,
    pub walked: f32,
    pub next_encounter: f32,
}

impl Field {
    pub fn origin(&self) -> Vec2 {
        Vec2::new(-(self.columns as f32) / 2.0, -(self.rows as f32) / 2.0)
    }

    pub fn is_blocked(&self, column: i32, row: i32) -> bool {
        if column < 0 || row < 0 || column >= self.columns as i32 || row >= self.rows as i32 {
            return true;
        }
        self.blocked[(row as u32 * self.columns + column as u32) as usize]
    }
}

#[derive(Clone, Debug)]
pub struct Hero {
    pub name: &'static str,
    pub job: Job,
    pub level: u32,
    pub experience: u32,
    pub health: i32,
    pub max_health: i32,
    pub magic: i32,
    pub max_magic: i32,
}

impl Hero {
    pub fn alive(&self) -> bool {
        self.health > 0
    }

    pub fn power(&self) -> i32 {
        let stats = data::job_stats(self.job);
        stats.power + self.level as i32
    }

    pub fn guard(&self) -> i32 {
        let stats = data::job_stats(self.job);
        stats.guard + self.level as i32 / 2
    }

    pub fn speed(&self) -> i32 {
        data::job_stats(self.job).speed + self.level as i32 / 3
    }
}

pub struct Party {
    pub heroes: Vec<Hero>,
    pub gold: u32,
    pub shards: u32,
    pub potions: u32,
    pub blade_level: u32,
    pub guard_level: u32,
    pub unlocked: Vec<Job>,
    pub chosen: usize,
}

impl Default for Party {
    fn default() -> Self {
        let heroes = data::HERO_NAMES
            .iter()
            .map(|name| {
                let stats = data::job_stats(Job::Freelancer);
                Hero {
                    name,
                    job: Job::Freelancer,
                    level: 1,
                    experience: 0,
                    health: stats.health,
                    max_health: stats.health,
                    magic: stats.magic,
                    max_magic: stats.magic,
                }
            })
            .collect();
        Self {
            heroes,
            gold: theme::START_GOLD,
            shards: 0,
            potions: theme::START_POTIONS,
            blade_level: 0,
            guard_level: 0,
            unlocked: vec![Job::Freelancer],
            chosen: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Intro,
    Choosing,
    Acting,
    Won,
    Lost,
    Fled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Actor {
    Hero(usize),
    Foe(usize),
}

pub struct Foe {
    pub kind: MonsterKind,
    pub health: i32,
    pub max_health: i32,
}

impl Foe {
    pub fn alive(&self) -> bool {
        self.health > 0
    }
}

pub struct Battle {
    pub foes: Vec<Foe>,
    pub order: Vec<Actor>,
    pub step: usize,
    pub phase: Phase,
    pub timer: f32,
    pub round: u32,
    pub log: String,
    pub gold: u32,
    pub experience: u32,
    pub staged: bool,
    pub command: Option<BattleCommand>,
    pub flash: Option<(bool, usize)>,
    pub strike: Option<(bool, usize)>,
}

impl Default for Battle {
    fn default() -> Self {
        Self {
            foes: Vec::new(),
            order: Vec::new(),
            step: 0,
            phase: Phase::Intro,
            timer: 0.0,
            round: 1,
            log: String::new(),
            gold: 0,
            experience: 0,
            staged: false,
            command: None,
            flash: None,
            strike: None,
        }
    }
}

impl Battle {
    pub fn current_hero(&self) -> Option<usize> {
        match self.order.get(self.step) {
            Some(Actor::Hero(index)) => Some(*index),
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct Prompt {
    pub text: String,
    pub station: Option<Entity>,
    pub kind: Option<StationKind>,
}

#[derive(Default)]
pub struct Toast {
    pub text: String,
    pub life: f32,
}

#[derive(Default)]
pub struct Readouts {
    pub cards: [String; 4],
    pub purse: String,
    pub banner: String,
    pub prompt: String,
    pub foes: [String; 3],
    pub log: String,
    pub shop_purse: String,
    pub jobs_note: String,
}

#[derive(Default)]
pub struct Hud {
    pub handles: UiHandles,
    pub shown: Option<Screen>,
    pub last: Readouts,
}

#[derive(Default, Clone, Copy)]
pub struct UiHandles {
    pub title: Option<Entity>,
    pub title_play: Option<Entity>,
    pub title_quit: Option<Entity>,
    pub party: Option<Entity>,
    pub party_cards: [Option<Entity>; 4],
    pub purse: Option<Entity>,
    pub banner: Option<Entity>,
    pub prompt: Option<Entity>,
    pub foes: Option<Entity>,
    pub foe_rows: [Option<Entity>; 3],
    pub log: Option<Entity>,
    pub commands: Option<Entity>,
    pub command_buttons: [Option<Entity>; 4],
    pub shop: Option<Entity>,
    pub shop_purse: Option<Entity>,
    pub shop_buttons: [Option<Entity>; 3],
    pub shop_leave: Option<Entity>,
    pub jobs: Option<Entity>,
    pub jobs_note: Option<Entity>,
    pub jobs_heroes: [Option<Entity>; 4],
    pub jobs_list: [Option<Entity>; 5],
    pub jobs_unlock: Option<Entity>,
    pub jobs_leave: Option<Entity>,
    pub paused: Option<Entity>,
    pub paused_resume: Option<Entity>,
    pub paused_title: Option<Entity>,
    pub defeat: Option<Entity>,
    pub defeat_title: Option<Entity>,
}
