use crate::contract::{Channel, ChatLine};
use crate::data::items::{BAG_SLOTS, LOG_ROWS, SHEET_ROWS, WARE_ROWS, WEAR_SLOTS};
use crate::data::{ABILITY_COUNT, CALLING_COUNT, CHAT_ROWS, PLACES, QUESTS};
use nightshade::prelude::{Entity, Vec2, Vec3, Vec4};
use nightshade_animation::contract::AnimationClip;
use nightshade_load::LoadHandle;
use nightshade_prefab::{ModelSkin, Prefab};
use nightshade_random::prelude::*;
use nightshade_render::data::asset_id::MaterialId;
use std::collections::VecDeque;

#[derive(Default, Clone)]
pub struct Palette {
    pub grass: MaterialId,
    pub tuft: MaterialId,
    pub water: MaterialId,
    pub fire: MaterialId,
    pub frost: MaterialId,
    pub lantern: MaterialId,
    pub holy: MaterialId,
    pub life: MaterialId,
    pub ice: MaterialId,
    pub soot: MaterialId,
    pub violet: MaterialId,
    pub stone: MaterialId,
    pub cloud: MaterialId,
    pub feather: MaterialId,
    pub flowers: Vec<MaterialId>,
    pub mobs: Vec<MaterialId>,
    pub pickups: Vec<MaterialId>,
}

pub const QUEST_ROWS: usize = 10;
pub const MAP_MARK_COUNT: usize = 8;

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
    pub slot_names: [Entity; ABILITY_COUNT],
    pub slot_status: [Entity; ABILITY_COUNT],
    pub slot_shades: [Entity; ABILITY_COUNT],
    pub slot_wipes: [Entity; ABILITY_COUNT],
    pub chat_rows: [Entity; CHAT_ROWS],
    pub chat_panel: Entity,
    pub purse: Entity,
    pub banner_title: Entity,
    pub banner_subtitle: Entity,
    pub combat_label: Entity,
    pub map_root: Entity,
    pub map_player: Entity,
    pub map_marks: Vec<Entity>,
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
}

#[derive(Clone, Copy)]
pub struct QuestState {
    pub quest: usize,
    pub progress: u32,
    pub complete: bool,
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
            finished: vec![false; QUESTS.len()],
            near_giver: vec![false; crate::data::GIVERS.len()],
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
    pub killer: Entity,
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

pub struct Clock {
    pub save_timer: f32,
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
            save_timer: crate::data::save::SAVE_PERIOD,
            last_place: None,
            discovered: vec![false; PLACES.len()],
            population_timer: 0.0,
            online: 0,
            shown_target: None,
            minimap_timer: 0.0,
            map_open: false,
        }
    }
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
    pub lamps_lit: Option<bool>,
}

#[derive(Default)]
pub struct Tracks {
    pub zone: Option<Entity>,
    pub night: Option<Entity>,
    pub battle: Option<Entity>,
    pub battle_on: bool,
    pub night_on: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Screen {
    #[default]
    Creation,
    World,
}

#[derive(Default)]
pub struct Creation {
    pub class: usize,
    pub resume: Option<crate::data::save::Save>,
    pub resume_button: Entity,
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
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Stack {
    pub item: usize,
    pub count: u32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Bag {
    pub slots: Vec<Option<Stack>>,
}

impl Default for Bag {
    fn default() -> Self {
        Self {
            slots: vec![None; BAG_SLOTS],
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Gear {
    pub worn: [Option<usize>; WEAR_SLOTS],
    pub change: u32,
}

#[derive(Default, Clone)]
pub struct PanelHandles {
    pub sheet_root: Entity,
    pub sheet_rows: [Entity; SHEET_ROWS],
    pub wear_frames: [Entity; WEAR_SLOTS],
    pub wear_icons: [Entity; WEAR_SLOTS],
    pub wear_names: [Entity; WEAR_SLOTS],
    pub bag_root: Entity,
    pub bag_frames: [Entity; BAG_SLOTS],
    pub bag_icons: [Entity; BAG_SLOTS],
    pub bag_counts: [Entity; BAG_SLOTS],
    pub bag_note: Entity,
    pub shop_root: Entity,
    pub shop_title: Entity,
    pub shop_note: Entity,
    pub ware_frames: [Entity; WARE_ROWS],
    pub ware_icons: [Entity; WARE_ROWS],
    pub ware_names: [Entity; WARE_ROWS],
    pub log_root: Entity,
    pub log_rows: [Entity; LOG_ROWS],
}

#[derive(Default, Clone, PartialEq)]
pub struct PanelText {
    pub sheet: Vec<(String, Vec4)>,
    pub wear: Vec<(String, Vec4)>,
    pub bag: Vec<(String, String, Vec4)>,
    pub bag_note: String,
    pub shop_title: String,
    pub shop_note: String,
    pub wares: Vec<(String, String, Vec4)>,
    pub log: Vec<(String, Vec4)>,
}

#[derive(Default)]
pub struct Panels {
    pub handles: PanelHandles,
    pub sheet_open: bool,
    pub bag_open: bool,
    pub log_open: bool,
    pub shop: Option<usize>,
    pub shown_sheet: bool,
    pub shown_bag: bool,
    pub shown_log: bool,
    pub shown_shop: bool,
    pub shown: PanelText,
    pub note: String,
    pub built: bool,
}

#[derive(Default)]
pub struct Hover {
    pub entity: Option<Entity>,
    pub cursor: (u32, u32),
    pub timer: f32,
}

pub struct ThornmereResources {
    pub palette: Palette,
    pub class: usize,
    pub shown_level: u32,
    pub player: Option<Entity>,
    pub camera: Option<Entity>,
    pub ring: Option<Entity>,
    pub player_name: String,
    pub hud: HudHandles,
    pub hud_text: HudText,
    pub quests: QuestLog,
    pub purse: u32,
    pub bag: Bag,
    pub gear: Gear,
    pub chat: ChatLog,
    pub clock: Clock,
    pub random: Random,
}

impl Default for ThornmereResources {
    fn default() -> Self {
        Self {
            palette: Palette::default(),
            class: 0,
            shown_level: 0,
            player: None,
            camera: None,
            ring: None,
            player_name: String::new(),
            hud: HudHandles::default(),
            hud_text: HudText::default(),
            quests: QuestLog::default(),
            purse: 0,
            bag: Bag::default(),
            gear: Gear::default(),
            chat: ChatLog::default(),
            clock: Clock::default(),
            random: Random::seeded(0x7404_9e2d_51c3_0a17),
        }
    }
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
pub struct Walls {
    pub segments: Vec<(Vec2, Vec2)>,
}

#[derive(Default)]
pub struct Icons {
    pub tint: Vec4,
    pub frame_layers: Vec<Option<u32>>,
    pub layer: Option<u32>,
    pub cells: Vec<(Vec2, Vec2)>,
    pub map_layer: Option<u32>,
    pub map_extent: Vec2,
    pub mark_layer: Option<u32>,
    pub marks: Vec<(Vec2, Vec2)>,
    pub item_layer: Option<u32>,
    pub item_cells: Vec<(Vec2, Vec2)>,
    pub frame_layer: Option<u32>,
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
