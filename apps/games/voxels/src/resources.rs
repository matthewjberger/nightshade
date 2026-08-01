use crate::data::{
    AfterTalk, CropKind, Item, ItemStack, Land, Palette, StartOfDay, WeatherKind, Woods,
};
use crate::theme;
use fastrand::Rng;
use nalgebra_glm::{IVec3, Vec3, Vec4};
use nightshade::ecs::Entity;
use nightshade_save::prelude::Autosave;
use nightshade_sprite::prelude::SpriteRegion;
pub use nightshade_voxels::prelude::{
    ChunkKey, ChunkStage, ColumnKey, Edits, Hit, VoxelSettings, VoxelViewer, VoxelWorld,
};
use noise::{Fbm, MultiFractal, Perlin};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct Digging;

pub struct ColumnOrder {
    pub seed: u32,
    pub plan: WorldPlan,
    pub villages: HashMap<(i32, i32), Option<Village>>,
    pub spill: HashMap<ChunkKey, Vec<(IVec3, u8, bool)>>,
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
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct WorldPlan {
    pub land: Land,
    pub woods: Woods,
    pub caves: bool,
    pub villages: bool,
    pub dungeons: bool,
    pub monsters: bool,
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
            monsters: true,
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
        }
    }
}

#[derive(Default)]
pub struct Framing {
    pub subject: Option<Entity>,
    pub base: Vec3,
    pub phase: f32,
    pub blend: f32,
    pub talking: bool,
}

pub struct Player {
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
    pub flying: bool,
    pub watching: bool,
    pub watched: Option<Entity>,
    pub watch_speed: f32,
    pub in_water: bool,
    pub camera: Option<Entity>,
    pub boat: Option<Entity>,
    pub health: f32,
    pub food: f32,
    pub air: f32,
    pub fall_from: f32,
    pub hurt: f32,
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
            pitch: 0.0,
            on_ground: false,
            flying: false,
            watching: false,
            watched: None,
            watch_speed: theme::WATCH_SPEED,
            in_water: false,
            camera: None,
            boat: None,
            health: theme::FULL_HEALTH,
            food: theme::FULL_FOOD,
            air: theme::DROWN_SECONDS,
            fall_from: 0.0,
            hurt: 0.0,
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
    pub target: Option<Hit>,
    pub mining: Option<IVec3>,
    pub progress: f32,
    pub stage: Option<usize>,
    pub outline: Option<Entity>,
    pub outlined: Option<IVec3>,
    pub crack: Option<Entity>,
    pub crack_shown: bool,
}

#[derive(Default)]
pub struct Hand {
    pub entity: Option<Entity>,
    pub shown: Option<Item>,
    pub swing: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Screen {
    #[default]
    Title,
    NewWorld,
    Playing,
    Inventory,
    Crafting,
    Shop,
    Talking,
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
    pub palette: Palette,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_rate: true,
            palette: Palette::default(),
        }
    }
}

pub struct Art {
    pub painted: Palette,
    pub layers: [Option<u32>; 2],
}

impl Default for Art {
    fn default() -> Self {
        Self {
            painted: Palette::Classic,
            layers: [None, None],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum Prompt {
    #[default]
    None,
    Chest(IVec3),
    Board(Entity),
    Leave,
    Talk(Entity, &'static str),
    Door(IVec3, bool),
    Harvest(Entity),
    Craft,
    Sign(String),
}

#[derive(Default)]
pub struct Dialogue {
    pub speaker: String,
    pub lines: Vec<String>,
    pub line: usize,
    pub revealed: f32,
    pub after: AfterTalk,
}

#[derive(Default)]
pub struct Toast {
    pub text: String,
    pub remaining: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Shell {
    pub entity: Entity,
    pub position: Vec3,
    pub velocity: Vec3,
    pub fuse: f32,
    pub color: Vec3,
}

pub struct Fireworks {
    pub shells: Vec<Shell>,
    pub rng: Rng,
}

impl Default for Fireworks {
    fn default() -> Self {
        Self {
            shells: Vec::new(),
            rng: Rng::with_seed(977),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Sheet {
    pub tiles: SpriteRegion,
    pub empty_rows_below: [u8; theme::SHEET_TILES],
    pub dungeon_tiles: SpriteRegion,
    pub dungeon_rows: [u8; theme::SHEET_TILES],
}

impl Default for Sheet {
    fn default() -> Self {
        Self {
            tiles: SpriteRegion::FULL,
            empty_rows_below: [0; theme::SHEET_TILES],
            dungeon_tiles: SpriteRegion::FULL,
            dungeon_rows: [0; theme::SHEET_TILES],
        }
    }
}

pub struct Horde {
    pub wait: f32,
    pub rng: Rng,
}

#[derive(Serialize, Deserialize)]
pub struct SavedCrop {
    pub kind: CropKind,
    pub block: [i32; 3],
    pub progress: f32,
}

#[derive(Serialize, Deserialize)]
pub struct SavedBoat {
    pub place: [f32; 3],
    pub yaw: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Saved {
    pub seed: u32,
    pub position: [f32; 3],
    pub home: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub health: f32,
    pub food: f32,
    pub slots: Vec<Option<ItemStack>>,
    pub selected: usize,
    pub coins: u32,
    pub chests: HashMap<[i32; 3], Vec<Option<ItemStack>>>,
    pub edited: Edits,
    pub crops: Vec<SavedCrop>,
    pub boats: Vec<SavedBoat>,
    pub palette: Palette,
    pub plan: WorldPlan,
}

impl nightshade_save::Save for Saved {
    const FOLDER: &'static str = theme::SAVE_FOLDER;
    const NAME: &'static str = theme::SAVE_NAME;
    const VERSION: u32 = theme::SAVE_VERSION;
    const CODEC: nightshade_save::SaveCodec = nightshade_save::SaveCodec::Bincode;
}

pub struct Save {
    pub found: Option<Saved>,
    pub autosave: Autosave,
}

impl Default for Save {
    fn default() -> Self {
        Self {
            found: None,
            autosave: Autosave::every(theme::SAVE_EVERY),
        }
    }
}

pub struct Cursor {
    pub shown: bool,
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
    pub rng: Rng,
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
            rng: Rng::with_seed(0x5A17_C10D),
            rain_emitter: None,
            snow_emitter: None,
        }
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self { shown: true }
    }
}

impl Default for Horde {
    fn default() -> Self {
        Self {
            wait: theme::MONSTER_WAIT,
            rng: Rng::with_seed(0x5EED_B0A7),
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
    pub hearts: Vec<Entity>,
    pub bites: Vec<Entity>,
    pub shown_health: Option<u32>,
    pub shown_food: Option<u32>,
    pub hud: Entity,
    pub gear: Entity,
    pub slots: [SlotHandles; theme::HOTBAR_SLOTS],
    pub status: Entity,
    pub prompt_panel: Entity,
    pub prompt_label: Entity,
    pub toast_panel: Entity,
    pub toast_label: Entity,
    pub coins_label: Entity,
    pub inventory_panel: Entity,
    pub inventory_coins: Entity,
    pub inventory_craft: Entity,
    pub inventory_close: Entity,
    pub pocket_slots: Vec<SlotHandles>,
    pub pocket_detail: Entity,
    pub pocket_hint: Entity,
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
    pub dialogue_panel: Entity,
    pub speaker_label: Entity,
    pub body_label: Entity,
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
    pub monsters_button: Entity,
    pub weather_button: Entity,
    pub start_button: Entity,
    pub new_world_make: Entity,
    pub new_world_back: Entity,
    pub shown_draft: Option<(u32, WorldPlan)>,
    pub shown_found: Option<bool>,
    pub settings_panel: Entity,
    pub settings_close: Entity,
    pub rate_toggle: Entity,
    pub palette_toggle: Entity,
    pub watch_toggle: Entity,
    pub settings_save: Entity,
    pub settings_leave: Entity,
    pub settings_title: Entity,
    pub rate_panel: Entity,
    pub rate_label: Entity,
    pub shown_rate: Option<bool>,
    pub shown_watching: Option<bool>,
    pub shown_palette: Option<Palette>,
    pub shown_rate_text: String,
    pub shown_rate_color: Option<Vec4>,
    pub atlas_layer: Option<u32>,
    pub tools_layer: Option<u32>,
    pub shown: [Option<ItemStack>; theme::HOTBAR_SLOTS],
    pub shown_selected: Option<usize>,
    pub shown_status: String,
    pub shown_screen: Option<Screen>,
    pub shown_prompt: Option<String>,
    pub shown_coins: Option<u32>,
    pub shown_pocket: Vec<Option<ItemStack>>,
    pub shown_marks: Option<(Option<usize>, usize)>,
    pub shown_detail: String,
    pub shown_panel_coins: Option<(Entity, u32)>,
    pub shown_have: Vec<u32>,
    pub shown_craft: Vec<bool>,
}
