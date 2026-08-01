use crate::components::ItemKind;
use crate::data::rules::{CASE_DIGITS, MAX_HEALTH, SATCHEL_SLOTS, STARTING_AMMO};
use nightshade::prelude::{Entity, Quat, Vec3};
use nightshade_cli::clap;

#[derive(clap::Args, Default)]
pub struct AshfallArguments {
    #[arg(long, help = "Start with the town already turned over")]
    pub otherworld: bool,
    #[arg(long, help = "Start with every key, the map, and a full battery")]
    pub carrying_everything: bool,
    #[arg(
        long,
        value_name = "COL,ROW",
        help = "Stand the player on this tile instead of the waking place"
    )]
    pub start_at: Option<String>,
}

#[derive(Clone, Copy, Debug)]
pub struct Impact {
    pub at: Vec3,
    pub force: f32,
}

#[derive(Default)]
pub struct Senses {
    pub impacts: Vec<Impact>,
    pub stains: Vec<Vec3>,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    #[default]
    Title,
    Playing,
    Paused,
    Reading,
    Talking,
    Inventory,
    Inspect,
    TownMap,
    Dead,
    Escaped,
}

impl Screen {
    pub fn is_in_town(self) -> bool {
        matches!(
            self,
            Screen::Playing
                | Screen::Paused
                | Screen::Reading
                | Screen::Talking
                | Screen::Inventory
                | Screen::Inspect
                | Screen::TownMap
        )
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Weapon {
    #[default]
    Pipe,
    Pistol,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum WorldPhase {
    #[default]
    Fog,
    Otherworld,
}

#[derive(Default, Clone, Copy)]
pub struct Slot {
    pub kind: Option<ItemKind>,
    pub count: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlotAction {
    Use,
    Examine,
    Combine,
    Drop,
}

impl SlotAction {
    pub const ALL: [SlotAction; 4] = [
        SlotAction::Use,
        SlotAction::Examine,
        SlotAction::Combine,
        SlotAction::Drop,
    ];

    pub fn name(self) -> &'static str {
        match self {
            SlotAction::Use => "Use",
            SlotAction::Examine => "Examine",
            SlotAction::Combine => "Combine",
            SlotAction::Drop => "Leave behind",
        }
    }
}

pub struct Satchel {
    pub slots: [Slot; SATCHEL_SLOTS],
    pub chosen: usize,
    pub action: usize,
    pub pairing: Option<usize>,
}

impl Default for Satchel {
    fn default() -> Self {
        Self {
            slots: [Slot::default(); SATCHEL_SLOTS],
            chosen: 0,
            action: 0,
            pairing: None,
        }
    }
}

#[derive(Default)]
pub struct InspectState {
    pub camera: Option<Entity>,
    pub item: Option<Entity>,
    pub kind: Option<ItemKind>,
    pub slot: usize,
    pub spin: Quat,
    pub found_catch: bool,
    pub showing: bool,
    pub wheels: [u32; CASE_DIGITS],
    pub wheel: usize,
}

#[derive(Default)]
pub struct ChartState {
    pub camera: Option<Entity>,
    pub showing: bool,
    pub floor: bool,
}

#[derive(Default)]
pub struct AshfallResources {
    pub screen: Screen,
    pub chart: ChartState,
    pub satchel: Satchel,
    pub inspect: InspectState,
    pub pending_screen: Option<Screen>,
    pub player: PlayerState,
    pub run: RunState,
    pub ui: UiHandles,
    pub hud: HudCache,
}

pub struct PlayerState {
    pub health: f32,
    pub ammo: u32,
    pub weapon: Weapon,
    pub attack_cooldown: f32,
    pub crouching: bool,
    pub moving: bool,
    pub upstairs: bool,
    pub at: Vec3,
    pub footfall: f32,
    pub holding: Option<Entity>,
    pub hold_distance: f32,
    pub hold_latched: bool,
    pub body: Option<Entity>,
    pub eyes: Option<Entity>,
    pub flashlight: Option<Entity>,
    pub hand: Quat,
    pub head: Quat,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            health: MAX_HEALTH,
            ammo: STARTING_AMMO,
            weapon: Weapon::Pipe,
            attack_cooldown: 0.0,
            crouching: false,
            moving: false,
            upstairs: false,
            at: Vec3::new(0.0, 0.0, 0.0),
            footfall: 0.0,
            holding: None,
            hold_distance: 1.8,
            hold_latched: false,
            body: None,
            eyes: None,
            flashlight: None,
            hand: Quat::identity(),
            head: Quat::identity(),
        }
    }
}

#[derive(Default)]
pub struct RunState {
    pub phase: WorldPhase,
    pub blocked: Vec<bool>,
    pub blocked_upper: Vec<bool>,
    pub explored: Vec<bool>,
    pub explored_upper: Vec<bool>,
    pub found_notes: Vec<bool>,
    pub exit_tile: (i32, i32),
    pub otherworld_spawns: Vec<(i32, i32, u8)>,
    pub message: String,
    pub message_until: f32,
    pub prompt: String,
    pub clock: f32,
    pub radio: f32,
    pub reading: Option<usize>,
    pub talking: Option<usize>,
    pub said: usize,
    pub rng: fastrand::Rng,
    pub kills: u32,
    pub map_dirty: bool,
    pub here: (i32, i32),
}

#[derive(Default, Clone)]
pub struct HudCache {
    pub condition: String,
    pub weapon: String,
    pub keys: String,
    pub message: String,
    pub prompt: String,
    pub radio: String,
    pub stance: String,
    pub battery: String,
}

#[derive(Clone, Copy, Default)]
pub struct UiHandles {
    pub hud: Option<Entity>,
    pub condition: Option<Entity>,
    pub weapon: Option<Entity>,
    pub keys: Option<Entity>,
    pub message: Option<Entity>,
    pub prompt: Option<Entity>,
    pub radio: Option<Entity>,
    pub stance: Option<Entity>,
    pub battery: Option<Entity>,
    pub title: Option<Entity>,
    pub play_button: Option<Entity>,
    pub quit_button: Option<Entity>,
    pub pause: Option<Entity>,
    pub resume_button: Option<Entity>,
    pub restart_button: Option<Entity>,
    pub pause_title_button: Option<Entity>,
    pub note: Option<Entity>,
    pub note_title: Option<Entity>,
    pub note_body: Option<Entity>,
    pub note_close_button: Option<Entity>,
    pub talk: Option<Entity>,
    pub talk_name: Option<Entity>,
    pub talk_line: Option<Entity>,
    pub talk_button: Option<Entity>,
    pub inventory: Option<Entity>,
    pub inventory_carried: Option<Entity>,
    pub inventory_slots: [Option<Entity>; SATCHEL_SLOTS],
    pub inventory_actions: Option<Entity>,
    pub inspect: Option<Entity>,
    pub inspect_name: Option<Entity>,
    pub inspect_body: Option<Entity>,
    pub inspect_hint: Option<Entity>,
    pub inventory_close_button: Option<Entity>,
    pub inventory_notes: Option<Entity>,
    pub town_map: Option<Entity>,
    pub town_map_title: Option<Entity>,
    pub town_map_room: Option<Entity>,
    pub dead: Option<Entity>,
    pub dead_again_button: Option<Entity>,
    pub dead_title_button: Option<Entity>,
    pub escaped: Option<Entity>,
    pub escaped_body: Option<Entity>,
    pub escaped_again_button: Option<Entity>,
    pub escaped_title_button: Option<Entity>,
}
