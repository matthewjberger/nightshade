use crate::data;
use nightshade::prelude::Entity;
use nightshade_cli::clap;
use nightshade_load::LoadHandle;
use nightshade_prefab::Prefab;

#[derive(clap::Args, Default, Clone)]
pub struct Arguments {
    #[arg(long, help = "Money the lot starts with")]
    pub money: Option<u32>,
}

#[derive(Default)]
pub struct Models {
    pub handles: Vec<LoadHandle>,
    pub prefabs: Vec<Option<Prefab>>,
    pub ready: bool,
    pub dweller_handle: Option<LoadHandle>,
    pub dweller_spawned: bool,
    pub clip_idle: Option<usize>,
    pub clip_walk: Option<usize>,
    pub clip_sit: Option<usize>,
}

pub struct Grid {
    pub floored: Vec<bool>,
    pub cells: Vec<Option<usize>>,
    pub north: Vec<Option<usize>>,
    pub west: Vec<Option<usize>>,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            floored: vec![false; data::COLUMNS * data::ROWS],
            cells: vec![None; data::COLUMNS * data::ROWS],
            north: vec![None; data::COLUMNS * (data::ROWS + 1)],
            west: vec![None; (data::COLUMNS + 1) * data::ROWS],
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Aim {
    #[default]
    Nowhere,
    Cell(usize),
    North(usize),
    West(usize),
}

#[derive(Default)]
pub struct Interface {
    pub money_readout: Entity,
    pub wage_readout: Entity,
    pub status_readout: Entity,
    pub need_fills: Vec<Entity>,
    pub need_readouts: Vec<Entity>,
    pub palette_slots: Vec<Entity>,
    pub tabs: Vec<Entity>,
}

pub struct Lot {
    pub money: f32,
    pub wage: f32,
    pub pad: Entity,
    pub category: data::Category,
    pub selected: usize,
    pub yaw_steps: u8,
    pub aim: Aim,
    pub clear: bool,
    pub interface: Interface,
    pub last_money: i32,
    pub last_wage: i32,
    pub last_needs: [i32; 4],
    pub last_status: String,
    pub last_labels: Vec<String>,
    pub last_disabled: Vec<bool>,
    pub last_tabs: Vec<String>,
}

impl Default for Lot {
    fn default() -> Self {
        Self {
            money: data::STARTING_MONEY as f32,
            wage: 0.0,
            pad: Entity::default(),
            category: data::Category::Build,
            selected: 0,
            yaw_steps: 0,
            aim: Aim::Nowhere,
            clear: false,
            interface: Interface::default(),
            last_money: i32::MIN,
            last_wage: i32::MIN,
            last_needs: [i32::MIN; 4],
            last_status: String::new(),
            last_labels: Vec::new(),
            last_disabled: Vec::new(),
            last_tabs: Vec::new(),
        }
    }
}

#[derive(Default)]
pub struct HearthsideShell {
    pub asked: Vec<crate::contract::HearthsideCommand>,
    pub money: f32,
}
