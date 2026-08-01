use crate::components::EnemyKind;
use crate::data::arsenal::{Ammo, GUN_COUNT, SHOTGUN};
use crate::data::levels::LEVEL_COUNT;
use nightshade::ecs::Entity;
use nightshade::prelude::{Vec2, Vec3};
use nightshade_animation::contract::AnimationClip;
use nightshade_cli::clap;
use nightshade_load::LoadHandle;
use nightshade_prefab::Prefab;

#[derive(clap::Args, Default, Clone)]
pub struct Arguments {
    #[arg(long, help = "Open in the level with this index")]
    pub level: Option<usize>,
}

pub const CLIP_IDLE: usize = 0;
pub const CLIP_WALK: usize = 1;
pub const CLIP_ATTACK: usize = 2;
pub const CLIP_DIE: usize = 3;

pub const MODEL_WHELP: usize = 0;
pub const MODEL_GUNNER: usize = 1;
pub const MODEL_LOBBER: usize = 2;
pub const MODEL_HULK: usize = 3;
pub const MODEL_COUNT: usize = 4;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    #[default]
    Title,
    Level,
    Paused,
    Tally,
    Ending,
}

#[derive(Default, Clone, Copy)]
pub struct HudHandles {
    pub root: Entity,
    pub health: Entity,
    pub armor: Entity,
    pub ammo: Entity,
    pub gun: Entity,
    pub slots: [Entity; GUN_COUNT],
    pub crosshair: [Entity; 4],
    pub keys: Entity,
    pub status: Entity,
    pub tally: Entity,
    pub flash: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct ScreenHandles {
    pub start: Entity,
    pub quit: Entity,
    pub resume: Entity,
    pub restart: Entity,
    pub abandon: Entity,
    pub carry_on: Entity,
    pub stand_down: Entity,
    pub tally_heading: Entity,
    pub tally_rows: [Entity; 3],
    pub ending_heading: Entity,
    pub ending_rows: [Entity; 3],
}

pub struct PlayerState {
    pub body: Option<Entity>,
    pub eyes: Option<Entity>,
    pub viewmodel: Option<Entity>,
    pub at: Option<Vec3>,
    pub gun: usize,
    pub owned: [bool; GUN_COUNT],
    pub health: f32,
    pub armor: f32,
    pub ammo: [u32; 4],
    pub red_key: bool,
    pub blue_key: bool,
    pub travelled: f32,
    pub bob_strength: f32,
    pub sway: Vec2,
    pub kick: f32,
    pub kick_applied: f32,
    pub punch: f32,
    pub flash: f32,
    pub cooldown: f32,
    pub hazard_clock: f32,
    pub hit_mark: f32,
    pub descent: f32,
    pub was_grounded: bool,
    pub dead: bool,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            body: None,
            eyes: None,
            viewmodel: None,
            at: None,
            gun: SHOTGUN,
            owned: [true, false, false, false],
            health: crate::data::START_HEALTH,
            armor: 0.0,
            ammo: [crate::data::START_SHELLS, 0, 0, 0],
            red_key: false,
            blue_key: false,
            travelled: 0.0,
            bob_strength: 0.0,
            sway: Vec2::zeros(),
            kick: 0.0,
            kick_applied: 0.0,
            punch: 0.0,
            flash: 0.0,
            cooldown: 0.0,
            hazard_clock: 0.0,
            hit_mark: 0.0,
            descent: 0.0,
            was_grounded: false,
            dead: false,
        }
    }
}

impl PlayerState {
    pub fn rounds(&self, ammo: Ammo) -> u32 {
        self.ammo[slot_of(ammo)]
    }

    pub fn spend(&mut self, ammo: Ammo) {
        let slot = slot_of(ammo);
        self.ammo[slot] = self.ammo[slot].saturating_sub(1);
    }
}

pub fn slot_of(ammo: Ammo) -> usize {
    match ammo {
        Ammo::Shells => 0,
        Ammo::Bullets => 1,
        Ammo::Rockets => 2,
        Ammo::Cells => 3,
    }
}

#[derive(Default)]
pub struct Assets {
    pub loads: [Option<LoadHandle>; MODEL_COUNT],
    pub prefabs: [Option<Prefab>; MODEL_COUNT],
    pub clips: [Vec<AnimationClip>; MODEL_COUNT],
}

impl Assets {
    pub fn ready(&self) -> bool {
        self.prefabs.iter().all(|held| held.is_some())
    }

    pub fn beast(&self, kind: EnemyKind) -> Option<(&Prefab, &[AnimationClip])> {
        let slot = crate::data::bestiary::beast(kind).model;
        self.prefabs[slot]
            .as_ref()
            .map(|prefab| (prefab, self.clips[slot].as_slice()))
    }
}

#[derive(Clone, Copy)]
pub struct Kit {
    pub health: f32,
    pub armor: f32,
    pub ammo: [u32; 4],
    pub owned: [bool; GUN_COUNT],
    pub gun: usize,
}

impl Default for Kit {
    fn default() -> Self {
        Self {
            health: crate::data::START_HEALTH,
            armor: 0.0,
            ammo: [crate::data::START_SHELLS, 0, 0, 0],
            owned: [true, false, false, false],
            gun: SHOTGUN,
        }
    }
}

#[derive(Default)]
pub struct Mission {
    pub level: usize,
    pub wanted: bool,
    pub retry: bool,
    pub kit: Kit,
    pub built: bool,
    pub populated: bool,
    pub armed: bool,
    pub kills: u32,
    pub standing: u32,
    pub secrets: u32,
    pub hidden: u32,
    pub clock: f32,
    pub done: bool,
    pub failed: bool,
    pub since_end: f32,
    pub banner: String,
    pub banner_left: f32,
    pub hint: String,
    pub thrown: Vec<u32>,
    pub ambushes: Vec<(Vec3, Vec3, u32)>,
    pub hazards: Vec<(Vec3, Vec3, f32)>,
}

#[derive(Default)]
pub struct Record {
    pub cleared: [bool; LEVEL_COUNT],
    pub kills: u32,
    pub secrets: u32,
    pub time: f32,
}

#[derive(Default)]
pub struct Readouts {
    pub health: u32,
    pub armor: u32,
    pub ammo: u32,
    pub gun: usize,
    pub gun_label: String,
    pub slots: String,
    pub keys: String,
    pub message: String,
    pub message_left: f32,
    pub flash: u32,
    pub mark: u32,
    pub tally: String,
    pub board: String,
}

#[derive(Default)]
pub struct BoomstickResources {
    pub player: PlayerState,
    pub hud: HudHandles,
    pub screens: ScreenHandles,
    pub assets: Assets,
    pub mission: Mission,
    pub record: Record,
    pub shown: Readouts,
}
