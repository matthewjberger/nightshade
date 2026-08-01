use crate::contract::{Channel, ChatLine};
use crate::data::{
    ABILITY_COUNT, CALLING_COUNT, CropKind, GIVER_COUNT, ItemStack, Land, PLACE_COUNT, QUEST_COUNT,
    StartOfDay, WeatherKind, Woods,
};
use crate::theme;
use nalgebra_glm::{IVec3, Vec2, Vec3, Vec4};
use nightshade::ecs::Entity;
use nightshade_animation::contract::AnimationClip;
use nightshade_load::LoadHandle;
use nightshade_prefab::{ModelSkin, Prefab};
use nightshade_random::prelude::*;
use nightshade_save::prelude::Autosave;
use nightshade_sprite::prelude::SpriteRegion;
pub use nightshade_voxels::prelude::{
    ChunkKey, ChunkStage, ColumnKey, Edits, Hit as BlockHit, VoxelSettings, VoxelViewer, VoxelWorld,
};
use noise::{Fbm, MultiFractal, Perlin};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

pub struct Digging;

pub struct ColumnOrder {
    pub seed: u32,
    pub plan: WorldPlan,
    pub villages: HashMap<(i32, i32), Option<Village>>,
    pub zones: Vec<ZonePlan>,
    pub spill: HashMap<ChunkKey, Vec<(IVec3, u8, bool)>>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ZonePlan {
    pub center: (i32, i32),
    pub radius: f32,
    pub level: i32,
    pub facing: f32,
}

pub struct ColumnReport {
    pub spill: HashMap<ChunkKey, Vec<(IVec3, u8, bool)>>,
    pub villages: HashMap<(i32, i32), Option<Village>>,
    pub signs: Vec<([i32; 3], String)>,
}

pub struct Noises {
    pub temperature: Fbm<Perlin>,
    pub moisture: Fbm<Perlin>,
    pub hills: Fbm<Perlin>,
    pub sea: Fbm<Perlin>,
    pub rolling: Fbm<Perlin>,
    pub detail: Perlin,
    pub worm: Perlin,
    pub hollow: Fbm<Perlin>,
    pub ravine: Fbm<Perlin>,
    pub twist: Perlin,
    pub stone: Fbm<Perlin>,
    pub crag: Perlin,
    pub shelf: Fbm<Perlin>,
    pub river: Fbm<Perlin>,
    pub chip: Fbm<Perlin>,
}

impl Noises {
    pub fn new(seed: u32) -> Self {
        Self {
            temperature: Fbm::<Perlin>::new(seed.wrapping_add(11))
                .set_octaves(3)
                .set_frequency(1.0 / 70.0),
            moisture: Fbm::<Perlin>::new(seed.wrapping_add(23))
                .set_octaves(3)
                .set_frequency(1.0 / 64.0),
            hills: Fbm::<Perlin>::new(seed.wrapping_add(37))
                .set_octaves(2)
                .set_frequency(1.0 / 80.0),
            sea: Fbm::<Perlin>::new(seed.wrapping_add(53))
                .set_octaves(2)
                .set_frequency(1.0 / 260.0),
            rolling: Fbm::<Perlin>::new(seed)
                .set_octaves(4)
                .set_frequency(1.0 / 36.0)
                .set_persistence(0.5),
            detail: Perlin::new(seed.wrapping_add(5)),
            worm: Perlin::new(seed.wrapping_add(41)),
            hollow: Fbm::<Perlin>::new(seed.wrapping_add(61))
                .set_octaves(3)
                .set_frequency(1.0 / 30.0),
            ravine: Fbm::<Perlin>::new(seed.wrapping_add(71))
                .set_octaves(2)
                .set_frequency(1.0 / 90.0),
            twist: Perlin::new(seed.wrapping_add(43)),
            stone: Fbm::<Perlin>::new(seed.wrapping_add(83))
                .set_octaves(2)
                .set_frequency(1.0 / 21.0)
                .set_persistence(0.55),
            crag: Perlin::new(seed.wrapping_add(97)),
            shelf: Fbm::<Perlin>::new(seed.wrapping_add(103))
                .set_octaves(2)
                .set_frequency(1.0 / 47.0),
            chip: Fbm::<Perlin>::new(seed.wrapping_add(113))
                .set_octaves(3)
                .set_frequency(1.0 / 8.0)
                .set_persistence(0.5),
            river: Fbm::<Perlin>::new(seed.wrapping_add(131))
                .set_octaves(2)
                .set_frequency(1.0 / 150.0),
        }
    }
}

impl Default for Noises {
    fn default() -> Self {
        Self::new(theme::SEED)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Footprint {
    pub building: usize,
    pub x: i32,
    pub z: i32,
    pub width: i32,
    pub depth: i32,
    pub base: i32,
    pub facing: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct Pad {
    pub x: i32,
    pub z: i32,
    pub width: i32,
    pub depth: i32,
    pub base: i32,
}

#[derive(Clone, Debug, Default)]
pub struct Village {
    pub center: (i32, i32),
    pub square: (i32, i32),
    pub shop_door: Option<(i32, i32)>,
    pub buildings: Vec<Footprint>,
    pub roads: Vec<(i32, i32)>,
    pub torches: Vec<(i32, i32)>,
    pub signs: Vec<([i32; 3], String)>,
    pub kind: VillageKind,
    pub name: String,
    pub pier: Option<(Vec2, Vec2)>,
    pub level: i32,
    pub streets: [i32; 4],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum VillageKind {
    #[default]
    Hamlet,
    Town,
    Farm,
    Fishing,
    Mining,
    Shrine,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct WorldPlan {
    pub land: Land,
    pub woods: Woods,
    pub caves: bool,
    pub villages: bool,
    pub dungeons: bool,
    pub weather: bool,
    pub start: StartOfDay,
}

impl Default for WorldPlan {
    fn default() -> Self {
        Self {
            land: Land::default(),
            woods: Woods::default(),
            caves: true,
            villages: true,
            dungeons: true,
            weather: true,
            start: StartOfDay::default(),
        }
    }
}

pub struct Draft {
    pub seed: u32,
    pub plan: WorldPlan,
    pub asked: bool,
    pub building: bool,
}

impl Default for Draft {
    fn default() -> Self {
        Self {
            seed: theme::SEED,
            plan: WorldPlan::default(),
            asked: false,
            building: false,
        }
    }
}

pub struct Generation {
    pub seed: u32,
    pub plan: WorldPlan,
    pub noises: Noises,
    pub villages: HashMap<(i32, i32), Option<Village>>,
    pub spill: HashMap<ChunkKey, Vec<(IVec3, u8, bool)>>,
    pub spawn: Option<(i32, i32)>,
    pub heights: Option<Heights>,
    pub signs: HashMap<[i32; 3], String>,
    pub zones: Vec<ZonePlan>,
    pub crypt: Option<Crypt>,
    pub mine: Option<Mine>,
}

#[derive(Clone, Copy, Debug)]
pub struct Crypt {
    pub corner: IVec3,
    pub span: i32,
    pub stair_bottom: Vec2,
    pub stair_top: Vec2,
    pub top: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct Mine {
    pub mouth: Vec2,
    pub end: Vec2,
    pub mouth_y: i32,
    pub end_y: i32,
}

pub struct Heights {
    pub column: ColumnKey,
    pub values: Vec<i32>,
    pub tops: Vec<i32>,
    pub lattice: Vec<f32>,
}

impl Default for Generation {
    fn default() -> Self {
        Self {
            seed: theme::SEED,
            plan: WorldPlan::default(),
            noises: Noises::new(theme::SEED),
            villages: HashMap::new(),
            spill: HashMap::new(),
            spawn: None,
            heights: None,
            signs: HashMap::new(),
            zones: Vec::new(),
            crypt: None,
            mine: None,
        }
    }
}

pub struct Player {
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub on_ground: bool,
    pub in_water: bool,
    pub camera: Option<Entity>,
    pub home: Vec3,
    pub eye: Vec3,
    pub forward: Vec3,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.5, theme::WORLD_Y as f32, 0.5),
            velocity: Vec3::zeros(),
            yaw: 0.0,
            on_ground: false,
            in_water: false,
            camera: None,
            home: Vec3::new(0.5, theme::WORLD_Y as f32, 0.5),
            eye: Vec3::zeros(),
            forward: Vec3::new(0.0, 0.0, -1.0),
        }
    }
}

pub struct Inventory {
    pub slots: [Option<ItemStack>; theme::INVENTORY_SLOTS],
    pub selected: usize,
    pub coins: u32,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            slots: crate::data::starting_inventory(),
            selected: 0,
            coins: theme::STARTING_COINS,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Aim {
    pub target: Option<BlockHit>,
    pub mining: Option<IVec3>,
    pub progress: f32,
    pub stage: Option<usize>,
    pub outline: Option<Entity>,
    pub outlined: Option<IVec3>,
    pub crack: Option<Entity>,
    pub crack_shown: bool,
    pub right_press: Option<Vec2>,
    pub over_blocks: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Screen {
    #[default]
    Title,
    NewWorld,
    Creation,
    Playing,
    Inventory,
    Crafting,
    Shop,
    Settings,
    Chest,
}

#[derive(Default)]
pub struct Chests {
    pub open: Option<[i32; 3]>,
    pub slots: HashMap<[i32; 3], Vec<Option<ItemStack>>>,
}

pub struct Settings {
    pub show_rate: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { show_rate: true }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum Prompt {
    #[default]
    None,
    Chest(IVec3),
    Door(IVec3, bool),
    Harvest(Entity),
    Craft,
    Sign(String),
    Trade(usize),
}

#[derive(Default)]
pub struct Toast {
    pub text: String,
    pub remaining: f32,
}

pub struct Fireworks {
    pub rng: Random,
}

impl Default for Fireworks {
    fn default() -> Self {
        Self {
            rng: Random::seeded(977),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Sheet {
    pub tiles: SpriteRegion,
    pub empty_rows_below: [u8; theme::SHEET_TILES],
}

impl Default for Sheet {
    fn default() -> Self {
        Self {
            tiles: SpriteRegion::FULL,
            empty_rows_below: [0; theme::SHEET_TILES],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SavedCrop {
    pub kind: CropKind,
    pub block: [i32; 3],
    pub progress: f32,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct QuestState {
    pub quest: usize,
    pub progress: u32,
    pub complete: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SavedHero {
    pub class: usize,
    pub name: String,
    pub level: u32,
    pub experience: u32,
    pub kills: u32,
    pub health: f32,
    pub mana: f32,
    pub town: (i32, i32),
    pub active: Vec<QuestState>,
    pub finished: Vec<bool>,
    pub discovered: Vec<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct Saved {
    pub seed: u32,
    pub position: [f32; 3],
    pub home: [f32; 3],
    pub yaw: f32,
    pub slots: Vec<Option<ItemStack>>,
    pub selected: usize,
    pub coins: u32,
    pub chests: HashMap<[i32; 3], Vec<Option<ItemStack>>>,
    pub edited: Edits,
    pub crops: Vec<SavedCrop>,
    pub plan: WorldPlan,
    pub town: Option<(i32, i32)>,
    pub hero: Option<SavedHero>,
}

impl nightshade_save::Save for Saved {
    const FOLDER: &'static str = theme::SAVE_FOLDER;
    const NAME: &'static str = theme::SAVE_NAME;
    const VERSION: u32 = theme::SAVE_VERSION;
    const CODEC: nightshade_save::SaveCodec = nightshade_save::SaveCodec::Bincode;
}

pub struct Save {
    pub found: Option<Saved>,
    pub hero: Option<SavedHero>,
    pub autosave: Autosave,
}

impl Default for Save {
    fn default() -> Self {
        Self {
            found: None,
            hero: None,
            autosave: Autosave::every(theme::SAVE_EVERY),
        }
    }
}

pub struct Weather {
    pub kind: WeatherKind,
    pub told: Option<WeatherKind>,
    pub until: f32,
    pub rain: f32,
    pub snow: f32,
    pub cloud: f32,
    pub flash: f32,
    pub until_lightning: f32,
    pub rng: Random,
    pub rain_emitter: Option<Entity>,
    pub snow_emitter: Option<Entity>,
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            kind: WeatherKind::Clear,
            told: Some(WeatherKind::Clear),
            until: theme::WEATHER_FIRST,
            rain: 0.0,
            snow: 0.0,
            cloud: 0.0,
            flash: 0.0,
            until_lightning: theme::LIGHTNING_GAP_MIN,
            rng: Random::seeded(0x5A17_C10D),
            rain_emitter: None,
            snow_emitter: None,
        }
    }
}

#[derive(Default)]
pub struct Effects {
    pub synced_edits: Option<u64>,
    pub center: Option<ColumnKey>,
}

#[derive(Default, Clone, Copy)]
pub struct SlotHandles {
    pub frame: Entity,
    pub icon: Entity,
    pub count: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct ShopRowHandles {
    pub index: usize,
    pub price: Entity,
    pub have: Entity,
    pub button: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct CraftRowHandles {
    pub index: usize,
    pub needs: Entity,
    pub button: Entity,
}

#[derive(Default, Clone)]
pub struct UiHandles {
    pub root: Entity,
    pub hud: Entity,
    pub slots: [SlotHandles; theme::HOTBAR_SLOTS],
    pub prompt_panel: Entity,
    pub prompt_label: Entity,
    pub toast_panel: Entity,
    pub toast_label: Entity,
    pub inventory_panel: Entity,
    pub inventory_coins: Entity,
    pub inventory_craft: Entity,
    pub inventory_close: Entity,
    pub pocket_slots: Vec<SlotHandles>,
    pub pocket_detail: Entity,
    pub holding: Option<usize>,
    pub crafting_panel: Entity,
    pub crafting_rows: Vec<CraftRowHandles>,
    pub crafting_coins: Entity,
    pub crafting_close: Entity,
    pub chest_panel: Entity,
    pub chest_slots: Vec<SlotHandles>,
    pub chest_pocket: Vec<SlotHandles>,
    pub chest_close: Entity,
    pub chest_title: Entity,
    pub shown_chest: Vec<Option<ItemStack>>,
    pub shown_chest_pocket: Vec<Option<ItemStack>>,
    pub shop_panel: Entity,
    pub shop_rows: Vec<ShopRowHandles>,
    pub shop_coins: Entity,
    pub shop_close: Entity,
    pub title_panel: Entity,
    pub title_continue: Entity,
    pub title_new: Entity,
    pub title_leave: Entity,
    pub new_world_panel: Entity,
    pub land_button: Entity,
    pub land_told: Entity,
    pub seed_button: Entity,
    pub seed_label: Entity,
    pub woods_button: Entity,
    pub caves_button: Entity,
    pub villages_button: Entity,
    pub dungeons_button: Entity,
    pub weather_button: Entity,
    pub start_button: Entity,
    pub new_world_make: Entity,
    pub new_world_back: Entity,
    pub shown_draft: Option<(u32, WorldPlan)>,
    pub shown_found: Option<bool>,
    pub settings_panel: Entity,
    pub settings_close: Entity,
    pub rate_toggle: Entity,
    pub settings_save: Entity,
    pub settings_title: Entity,
    pub settings_leave: Entity,
    pub rate_panel: Entity,
    pub rate_label: Entity,
    pub shown_rate: Option<bool>,
    pub shown_rate_text: String,
    pub shown_rate_color: Option<Vec4>,
    pub atlas_layer: Option<u32>,
    pub tools_layer: Option<u32>,
    pub shown: [Option<ItemStack>; theme::HOTBAR_SLOTS],
    pub shown_selected: Option<usize>,
    pub shown_screen: Option<Screen>,
    pub shown_prompt: Option<String>,
    pub shown_pocket: Vec<Option<ItemStack>>,
    pub shown_marks: Option<(Option<usize>, usize)>,
    pub shown_detail: String,
    pub shown_panel_coins: Option<(Entity, u32)>,
    pub shown_have: Vec<u32>,
    pub shown_craft: Vec<bool>,
}

pub const QUEST_ROWS: usize = 10;

#[derive(Default, Clone)]
pub struct HudHandles {
    pub root: Entity,
    pub hurt_overlay: Entity,
    pub player_name: Entity,
    pub player_health: Entity,
    pub player_health_fill: Entity,
    pub player_mana: Entity,
    pub player_mana_fill: Entity,
    pub target_panel: Entity,
    pub target_name: Entity,
    pub target_health: Entity,
    pub target_health_fill: Entity,
    pub zone_name: Entity,
    pub zone_clock: Entity,
    pub minimap: Entity,
    pub minimap_dots: Vec<Entity>,
    pub quest_rows: [Entity; QUEST_ROWS],
    pub experience_line: Entity,
    pub experience_fill: Entity,
    pub slot_frames: [Entity; ABILITY_COUNT],
    pub slot_icons: [Entity; ABILITY_COUNT],
    pub slot_labels: [Entity; ABILITY_COUNT],
    pub slot_status: [Entity; ABILITY_COUNT],
    pub slot_shades: [Entity; ABILITY_COUNT],
    pub chat_rows: [Entity; theme::CHAT_ROWS],
    pub chat_panel: Entity,
    pub purse: Entity,
    pub banner_title: Entity,
    pub banner_subtitle: Entity,
    pub combat_label: Entity,
    pub shown_class: Option<usize>,
    pub map_root: Entity,
    pub map_picture: Entity,
    pub map_player: Entity,
    pub map_marks: Vec<Entity>,
    pub map_icons: Vec<Entity>,
    pub map_labels: Vec<Entity>,
    pub map_note: Entity,
}

#[derive(Default, Clone, PartialEq)]
pub struct HudText {
    pub player_name: String,
    pub player_health: String,
    pub player_health_fraction: u32,
    pub player_mana: String,
    pub player_mana_fraction: u32,
    pub target_visible: bool,
    pub target_name: String,
    pub target_health: String,
    pub target_health_fraction: u32,
    pub zone_name: String,
    pub zone_clock: String,
    pub quest_rows: Vec<String>,
    pub experience: String,
    pub experience_fraction: u32,
    pub slot_status: Vec<(String, Vec4)>,
    pub slot_shade: Vec<u32>,
    pub chat: Vec<(Channel, String)>,
    pub purse: String,
    pub combat_visible: bool,
    pub minimap: Vec<(u8, i16, i16)>,
    pub hurt_alpha: f32,
    pub map_open: bool,
    pub map_player: (i16, i16),
    pub map_marks: Vec<(u8, i16, i16)>,
    pub map_note: String,
    pub map_world: Option<(u32, (i32, i32))>,
}

pub struct QuestLog {
    pub active: Vec<QuestState>,
    pub finished: Vec<bool>,
    pub near_giver: Vec<bool>,
}

impl Default for QuestLog {
    fn default() -> Self {
        Self {
            active: Vec::new(),
            finished: vec![false; QUEST_COUNT],
            near_giver: vec![false; GIVER_COUNT],
        }
    }
}

pub struct Scheduled {
    pub delay: f32,
    pub line: ChatLine,
}

#[derive(Clone)]
pub struct Speaker {
    pub name: String,
    pub voice: usize,
    pub class: usize,
}

pub struct ChatLog {
    pub lines: VecDeque<(Channel, String)>,
    pub scheduled: Vec<Scheduled>,
    pub roster: Vec<Speaker>,
    pub chatter_timer: f32,
    pub whisper_timer: f32,
    pub scroll: usize,
    pub gap: f32,
}

impl Default for ChatLog {
    fn default() -> Self {
        Self {
            lines: VecDeque::new(),
            scheduled: Vec::new(),
            roster: Vec::new(),
            chatter_timer: 3.0,
            whisper_timer: 30.0,
            scroll: 0,
            gap: 0.0,
        }
    }
}

pub struct Clock {
    pub last_place: Option<usize>,
    pub discovered: Vec<bool>,
    pub population_timer: f32,
    pub online: usize,
    pub shown_target: Option<Entity>,
    pub minimap_timer: f32,
    pub map_open: bool,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            last_place: None,
            discovered: vec![false; PLACE_COUNT],
            population_timer: 0.0,
            online: 0,
            shown_target: None,
            minimap_timer: 0.0,
            map_open: false,
        }
    }
}

#[derive(Default)]
pub struct Journal {
    pub quests: QuestLog,
    pub chat: ChatLog,
    pub clock: Clock,
}

pub struct Hero {
    pub class: usize,
    pub name: String,
    pub player: Option<Entity>,
    pub camera: Option<Entity>,
    pub ring: Option<Entity>,
    pub named: bool,
    pub random: Random,
}

impl Default for Hero {
    fn default() -> Self {
        Self {
            class: 0,
            name: String::new(),
            player: None,
            camera: None,
            ring: None,
            named: false,
            random: Random::seeded(0x7404_9e2d_51c3_0a17),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum HitSource {
    #[default]
    Player,
    Monster,
    Person,
    Heal,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Hit {
    pub attacker: Entity,
    pub target: Entity,
    pub damage: f32,
    pub crit: bool,
    pub chill: bool,
    pub source: HitSource,
}

#[derive(Default)]
pub struct Hits {
    pub pending: Vec<Hit>,
}

#[derive(Clone)]
pub struct KillEvent {
    pub prey: String,
    pub rank: u8,
    pub place: Option<usize>,
    pub by_player: bool,
    pub killer_name: String,
}

#[derive(Default)]
pub struct Slain {
    pub events: Vec<KillEvent>,
}

#[derive(Clone)]
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
    pub next: usize,
}

pub struct Cue {
    pub clip: &'static str,
    pub volume: f32,
}

#[derive(Default)]
pub struct Cues {
    pub pending: Vec<Cue>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BurstKind {
    Hit,
    Crit,
    Frost,
    Heal,
    Kill,
    EliteKill,
    Coins,
    LevelUp,
    QuestDone,
    Gather,
    Dust,
    Puff,
    Cast,
    FrostCast,
    Fire,
    FrostHit,
}

#[derive(Clone, Copy)]
pub struct Burst {
    pub kind: BurstKind,
    pub position: Vec3,
}

#[derive(Default)]
pub struct Bursts {
    pub pending: Vec<Burst>,
}

pub struct Banner {
    pub title: String,
    pub subtitle: String,
    pub color: Vec4,
}

#[derive(Default)]
pub struct Banners {
    pub pending: VecDeque<Banner>,
    pub remaining: f32,
    pub shown: bool,
    pub color: Vec4,
}

#[derive(Default)]
pub struct Feel {
    pub trauma: f32,
    pub hurt: f32,
    pub grey: f32,
    pub combat: f32,
    pub combat_shown: bool,
    pub dust_marker: f32,
    pub step_marker: f32,
    pub slot_flash: [f32; ABILITY_COUNT],
    pub low_pulse: f32,
    pub first_person: bool,
}

#[derive(Default)]
pub struct Tracks {
    pub zone: Option<Entity>,
    pub night: Option<Entity>,
    pub battle: Option<Entity>,
    pub battle_on: bool,
    pub night_on: bool,
}

#[derive(Default)]
pub struct Creation {
    pub class: usize,
    pub panel: Entity,
    pub class_buttons: [Entity; CALLING_COUNT],
    pub title: Entity,
    pub blurb: Entity,
    pub stats: [Entity; 4],
    pub abilities: [Entity; ABILITY_COUNT],
    pub name_input: Entity,
    pub roll: Entity,
    pub enter: Entity,
    pub figure: Option<Entity>,
    pub camera: Option<Entity>,
    pub shown_class: Option<usize>,
}

#[derive(Default)]
pub struct Hover {
    pub entity: Option<Entity>,
    pub cursor: (u32, u32),
    pub timer: f32,
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

#[derive(Default)]
pub struct Icons {
    pub tint: Vec4,
    pub layer: Option<u32>,
    pub cells: Vec<(Vec2, Vec2)>,
    pub frame_layer: Option<u32>,
    pub mark_layer: Option<u32>,
    pub marks: Vec<(Vec2, Vec2)>,
    pub map_layer: Option<u32>,
    pub map_extent: Vec2,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PlaceSpot {
    pub center: Vec2,
    pub radius: f32,
    pub floor: Option<f32>,
}

#[derive(Default)]
pub struct Places {
    pub town: (i32, i32),
    pub spots: Vec<PlaceSpot>,
    pub laid: bool,
    pub peopled: bool,
}

pub struct Fill {
    pub speakers: Vec<Speaker>,
    pub monster: String,
    pub place: String,
    pub item: String,
    pub price: String,
    pub quest: String,
    pub guild: String,
    pub player: String,
    pub level: String,
    pub killer: String,
}
