use crate::data::districts::District;
use crate::data::rooms::Room;
use nightshade::prelude::{Entity, Vec2, Vec3, Vec4};
use nightshade_animation::contract::AnimationClip;
use nightshade_load::LoadHandle;
use nightshade_prefab::{ModelSkin, Prefab};
use nightshade_render::data::asset_id::MaterialId;
use std::collections::{HashMap, VecDeque};

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    #[default]
    Title,
    Playing,
    Paused,
    Settings,
}

#[derive(Default, Clone, Copy)]
pub struct HudHandles {
    pub root: Option<Entity>,
    pub rate: Option<Entity>,
    pub readout: Option<Entity>,
    pub notice: Option<Entity>,
    pub settings: Option<Entity>,
}

#[derive(Default, Clone, Copy)]
pub struct ScreenHandles {
    pub play_button: Option<Entity>,
    pub title_settings_button: Option<Entity>,
    pub title_quit_button: Option<Entity>,
    pub resume_button: Option<Entity>,
    pub pause_settings_button: Option<Entity>,
    pub pause_title_button: Option<Entity>,
    pub settings_back_button: Option<Entity>,
}

#[derive(Default)]
pub struct GridlockResources {
    pub seed: u64,
    pub hud: HudHandles,
    pub screens: ScreenHandles,
    pub last_readout: String,
    pub last_rate: String,
    pub rate_left: f32,
    pub last_notice: String,
    pub notice_left: f32,
    pub neon_materials: Vec<MaterialId>,
    pub pavement_material: Option<MaterialId>,
    pub applied_revision: Option<u64>,
    pub settings_from: Screen,
}

#[derive(Default)]
pub struct Player {
    pub body: Option<Entity>,
    pub car: Option<Entity>,
    pub walk_camera: Option<Entity>,
    pub chase_camera: Option<Entity>,
    pub fly_camera: Option<Entity>,
    pub rain: Option<Entity>,
    pub sprinting: bool,
    pub first_person: bool,
    pub flying: bool,
    pub fly_yaw: f32,
    pub fly_pitch: f32,
}

#[derive(Clone)]
pub struct ModelKit {
    pub prefab: Prefab,
    pub clips: Vec<AnimationClip>,
    pub skins: Vec<ModelSkin>,
}

#[derive(Default)]
pub struct Models {
    pub handles: Vec<Option<LoadHandle>>,
    pub kits: Vec<Option<ModelKit>>,
    pub failed: Vec<bool>,
    pub requested: Vec<bool>,
    pub settle: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
pub struct Placement {
    pub model: usize,
    pub position: Vec3,
    pub yaw: f32,
    pub scale: f32,
    pub prop: Option<usize>,
}

#[derive(Clone, Copy, Debug)]
pub struct Lot {
    pub minimum: Vec2,
    pub maximum: Vec2,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Hull {
    pub center: Vec2,
    pub heading: Vec2,
    pub half: Vec2,
}

#[derive(Clone, Copy, Debug)]
pub struct Neon {
    pub position: Vec3,
    pub size: Vec3,
    pub color: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Lamp {
    pub position: Vec3,
    pub warm: bool,
    pub prop: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Pavement {
    pub position: Vec3,
    pub size: Vec3,
}

#[derive(Clone, Copy, Debug)]
pub struct Door {
    pub position: Vec3,
    pub facing: Vec3,
    pub room: Room,
    pub size: Vec2,
}

#[derive(Clone, Debug)]
pub struct CellLayout {
    pub key: [i32; 2],
    pub district: District,
    pub placements: Vec<Placement>,
    pub lots: Vec<Lot>,
    pub props: Vec<Option<Lot>>,
    pub lamps: Vec<Lamp>,
    pub neons: Vec<Neon>,
    pub pavements: Vec<Pavement>,
    pub doors: Vec<Door>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Plan {
    pub neon_chance: f32,
    pub parking: f32,
    pub trees: usize,
    pub tower_chance: f32,
    pub props: f32,
    pub site_chance: f32,
}

pub struct CellJob {
    pub key: [i32; 2],
    pub seed: u64,
    pub plan: Plan,
}

#[derive(Default)]
pub struct City {
    pub cells: HashMap<[i32; 2], CellLayout>,
    pub cache: HashMap<[i32; 2], CellLayout>,
}

#[derive(Default)]
pub struct Streaming {
    pub center: Option<[i32; 2]>,
    pub facing: Option<i32>,
    pub forward: Vec3,
    pub view: i32,
    pub keep: i32,
    pub ring: i32,
    pub life: i32,
    pub lit: Option<([i32; 2], i32)>,
    pub pending: Vec<[i32; 2]>,
    pub leaving: Vec<[i32; 2]>,
    pub settling: VecDeque<(CellLayout, usize)>,
    pub dying: Vec<Entity>,
}

#[derive(Clone, Copy, Debug)]
pub struct Knock {
    pub key: [i32; 2],
    pub index: usize,
    pub direction: Vec3,
}

#[derive(Default)]
pub struct Knocks {
    pub pending: Vec<Knock>,
}

#[derive(Clone, Copy, Debug)]
pub struct Visit {
    pub outside: Vec3,
    pub yaw: f32,
    pub exit: Vec3,
    pub room: Room,
}

#[derive(Default)]
pub struct Indoors {
    pub visit: Option<Visit>,
    pub lots: Vec<Lot>,
    pub serial: u32,
    pub used: bool,
}

#[derive(Clone, Debug)]
pub struct Flash {
    pub position: Vec3,
    pub text: String,
    pub color: Vec4,
    pub pop: f32,
}

#[derive(Default)]
pub struct Flashes {
    pub pending: Vec<Flash>,
    pub pool: Vec<Entity>,
    pub live: Vec<Entity>,
    pub next: usize,
    pub streak: u32,
    pub streak_left: f32,
}
