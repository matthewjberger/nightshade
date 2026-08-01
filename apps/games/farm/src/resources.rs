use crate::data::{CropKind, Errand, HouseKind, Item, ShopKind, Tool, Weapon, WeatherKind, Zone};
use fastrand::Rng;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::Entity;
use nightshade_cli::clap;
use nightshade_sprite::prelude::{SpriteRegion, TileTrim};

#[derive(clap::Args, Default)]
pub struct Hd2dArguments {
    #[arg(long, value_name = "HOUR", help = "Start the day at this hour")]
    pub hour: Option<f32>,
}

#[derive(Default)]
pub struct Diorama {
    pub camera: Option<Entity>,
    pub farmer: Option<Entity>,
    pub sun: Option<Entity>,
    pub start_hour: f32,
    pub day: u32,
    pub last_hour: f32,
    pub feet: Vec3,
    pub skipped_hours: f32,
    pub window_glow: f32,
    pub sea_color: Vec3,
    pub zone: Option<Zone>,
}

#[derive(Default)]
pub struct Home {
    pub active: bool,
    pub house: Option<HouseKind>,
    pub pending: Option<HouseKind>,
    pub leaving: bool,
    pub columns: u32,
    pub rows: u32,
    pub blocked: Vec<bool>,
    pub exit: Vec2,
    pub bed: Option<Vec2>,
    pub counter: Option<Vec2>,
}

#[derive(Default)]
pub struct Rig {
    pub yaw_index: i32,
    pub yaw: f32,
    pub focus: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Title,
    Playing,
    Paused,
    Inventory,
    Shop,
    Talking,
}

pub struct Inventory {
    pub coins: u32,
    pub counts: [u32; Item::ALL.len()],
    pub tools: [bool; Tool::ALL.len()],
    pub levels: [u32; Tool::ALL.len()],
    pub slot: usize,
}

impl Default for Inventory {
    fn default() -> Self {
        let mut counts = [0; Item::ALL.len()];
        counts[Item::CarrotSeed.index()] = 3;
        counts[Item::WheatSeed.index()] = 2;
        Self {
            coins: crate::theme::START_COINS,
            counts,
            tools: [true; Tool::ALL.len()],
            levels: [0; Tool::ALL.len()],
            slot: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AfterTalk {
    #[default]
    Nothing,
    OpenShop(ShopKind),
    Deliver(Errand),
    GiveDagger,
    Temper(Tool),
}

pub struct Vigor {
    pub stamina: f32,
}

impl Default for Vigor {
    fn default() -> Self {
        Self {
            stamina: crate::theme::MAX_STAMINA,
        }
    }
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
pub struct Quests {
    pub done: [bool; Errand::ALL.len()],
}

#[derive(Default)]
pub struct OpenShop {
    pub kind: ShopKind,
}

#[derive(Default, Clone, Debug)]
pub struct Plot {
    pub column: u32,
    pub row: u32,
    pub crop: Option<Entity>,
    pub cap: Option<Entity>,
    pub watered_day: Option<u32>,
}

#[derive(Default)]
pub struct Farmland {
    pub plots: Vec<Plot>,
    pub dried_day: u32,
}

#[derive(Default, Clone, Debug, PartialEq)]
pub enum Prompt {
    #[default]
    None,
    Talk(Entity, String),
    Sleep,
    EnterShop(ShopKind),
    Till(u32, u32),
    Water(usize),
    Chop(Entity),
    Clear(Entity),
    Dig(Entity),
    Milk(Entity),
    Cast,
    Reel,
    Pull,
    Pick(Entity),
    EnterHollow,
    EnterHouse(HouseKind),
    LeaveHouse,
    ClimbUp,
    ClimbDown,
    Open(Entity),
    Plant(usize, CropKind),
    Harvest(usize),
}

#[derive(Default)]
pub struct Toast {
    pub text: String,
    pub remaining: f32,
}

pub struct Shell {
    pub entity: Entity,
    pub position: Vec3,
    pub velocity: Vec3,
    pub fuse: f32,
    pub color: Vec3,
    pub willow: bool,
}

pub struct Fireworks {
    pub shells: Vec<Shell>,
    pub until_next: f32,
    pub rng: Rng,
}

impl Default for Fireworks {
    fn default() -> Self {
        Self {
            shells: Vec::new(),
            until_next: 0.0,
            rng: Rng::with_seed(977_u64),
        }
    }
}

pub struct Weather {
    pub planned_day: Option<u32>,
    pub kind: WeatherKind,
    pub window: Option<(f32, f32)>,
    pub active: bool,
    pub raining: bool,
    pub rain: f32,
    pub cloud: f32,
    pub fog: f32,
    pub wind: f32,
    pub gust: f32,
    pub rain_emitter: Option<Entity>,
    pub leaf_emitter: Option<Entity>,
    pub until_lightning: f32,
    pub flash: f32,
    pub announced: Option<(WeatherKind, bool)>,
    pub rng: Rng,
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            planned_day: None,
            kind: WeatherKind::Clear,
            window: None,
            active: false,
            raining: false,
            rain: 0.0,
            cloud: 0.0,
            fog: 0.0,
            wind: 0.0,
            gust: 0.0,
            rain_emitter: None,
            leaf_emitter: None,
            until_lightning: 0.0,
            flash: 0.0,
            announced: None,
            rng: Rng::with_seed(4241_u64),
        }
    }
}

#[derive(Default)]
pub struct Quarry {
    pub spots: Vec<Vec2>,
    pub restocked_day: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum FishingState {
    #[default]
    Idle,
    Waiting(f32),
    Bite(f32),
}

pub struct Fishing {
    pub state: FishingState,
    pub bobber: Option<Entity>,
    pub spot: Vec2,
    pub rng: Rng,
}

impl Default for Fishing {
    fn default() -> Self {
        Self {
            state: FishingState::Idle,
            bobber: None,
            spot: Vec2::new(0.0, 0.0),
            rng: Rng::with_seed(313_u64),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Door {
    pub kind: HouseKind,
    pub spot: Vec2,
}

#[derive(Default)]
pub struct Doors {
    pub doors: Vec<Door>,
}

#[derive(Default, Clone)]
pub struct UiHandles {
    pub hud: Option<Entity>,
    pub hotbar: Option<Entity>,
    pub clock_label: Option<Entity>,
    pub coins_label: Option<Entity>,
    pub hand_label: Option<Entity>,
    pub vigor_label: Option<Entity>,
    pub hero_label: Option<Entity>,
    pub prompt_panel: Option<Entity>,
    pub prompt_label: Option<Entity>,
    pub toast_panel: Option<Entity>,
    pub toast_label: Option<Entity>,
    pub dialogue_panel: Option<Entity>,
    pub speaker_label: Option<Entity>,
    pub body_label: Option<Entity>,
    pub pause_panel: Option<Entity>,
    pub pause_status: Option<Entity>,
    pub resume_button: Option<Entity>,
    pub menu_button: Option<Entity>,
    pub title_panel: Option<Entity>,
    pub title_play: Option<Entity>,
    pub title_quit: Option<Entity>,
    pub inventory_panel: Option<Entity>,
    pub inventory_coins: Option<Entity>,
    pub pocket_slots: Vec<PocketSlot>,
    pub meal_buttons: Vec<MealButton>,
    pub scrim: Option<Entity>,
    pub errand_rows: Vec<Entity>,
    pub shop_panels: Vec<ShopPanel>,
    pub hotbar_slots: Vec<HotbarSlot>,
    pub shown: Option<Screen>,
    pub shown_shop: Option<ShopKind>,
}

#[derive(Default, Clone, Copy)]
pub struct PocketSlot {
    pub cell: Option<Entity>,
    pub icon: Option<Entity>,
    pub count: Option<Entity>,
}

#[derive(Clone, Copy)]
pub struct MealButton {
    pub item: Item,
    pub button: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct ShopRow {
    pub ware: usize,
    pub frame: Option<Entity>,
    pub name: Option<Entity>,
    pub note: Option<Entity>,
    pub price: Option<Entity>,
    pub button: Option<Entity>,
}

#[derive(Default, Clone)]
pub struct ShopPanel {
    pub kind: Option<ShopKind>,
    pub panel: Option<Entity>,
    pub coins: Option<Entity>,
    pub rows: Vec<ShopRow>,
    pub close_button: Option<Entity>,
}

#[derive(Default, Clone, Copy)]
pub struct HotbarSlot {
    pub frame: Option<Entity>,
    pub icon: Option<Entity>,
    pub label: Option<Entity>,
}

#[derive(Default)]
pub struct Terrain {
    pub columns: u32,
    pub rows: u32,
    pub origin: Vec2,
    pub heights: Vec<u32>,
    pub walkable: Vec<bool>,
    pub occupied: Vec<bool>,
    pub bridge: Vec<bool>,
}

#[derive(Clone, Copy)]
pub struct Sheet {
    pub tiles: SpriteRegion,
    pub shadow: SpriteRegion,
    pub trim: [TileTrim; crate::theme::SHEET_TILES],
    pub dungeon: SpriteRegion,
    pub dungeon_trim: [TileTrim; crate::theme::SHEET_TILES],
}

impl Default for Sheet {
    fn default() -> Self {
        Self {
            tiles: SpriteRegion::FULL,
            shadow: SpriteRegion::FULL,
            trim: [TileTrim::default(); crate::theme::SHEET_TILES],
            dungeon: SpriteRegion::FULL,
            dungeon_trim: [TileTrim::default(); crate::theme::SHEET_TILES],
        }
    }
}

#[derive(Default)]
pub struct Dungeon {
    pub active: bool,
    pub deepest: u32,
    pub depth: u32,
    pub pending: Option<u32>,
    pub leaving: bool,
    pub columns: u32,
    pub rows: u32,
    pub origin: Vec2,
    pub floor: Vec<bool>,
    pub entrance: Vec2,
    pub exit: Vec2,
}

pub struct Hero {
    pub hp: f32,
    pub max_hp: u32,
    pub xp: u32,
    pub level: u32,
    pub weapon: Weapon,
    pub cooldown: f32,
    pub invulnerable: f32,
    pub strength: f32,
    pub speed: f32,
    pub grit: f32,
    pub deaths: u32,
}

impl Default for Hero {
    fn default() -> Self {
        Self {
            hp: crate::theme::HERO_MAX_HP as f32,
            max_hp: crate::theme::HERO_MAX_HP,
            xp: 0,
            level: 1,
            weapon: Weapon::Fists,
            cooldown: 0.0,
            invulnerable: 0.0,
            strength: 0.0,
            speed: 0.0,
            grit: 0.0,
            deaths: 0,
        }
    }
}

nightshade::ecs::resource_bundle! {
    pub struct Homestead<'a> {
        write {
            terrain: Terrain,
            farmland: Farmland,
            inventory: Inventory,
            toast: Toast,
            fishing: Fishing,
            vigor: Vigor,
        }
        read {}
    }
}

nightshade::ecs::resource_bundle! {
    pub struct Household<'a> {
        write {
            dialogue: Dialogue,
            diorama: Diorama,
            home: Home,
            open_shop: OpenShop,
            renderer_state: nightshade_render::data::config::RendererState,
            next_screen: nightshade_state::resources::NextState<Screen>,
        }
        read {}
    }
}

nightshade::ecs::resource_bundle! {
    pub struct Readout<'a> {
        write {
            handles: UiHandles,
            inventory: Inventory,
            quests: Quests,
            toast: Toast,
            dirty: nightshade_ui::resources::RetainedUiDirty,
            renderer_state: nightshade_render::data::config::RendererState,
        }
        read {
            time: nightshade_platform::contract::time::Time,
            diorama: Diorama,
            weather: Weather,
            open_shop: OpenShop,
            prompt: Prompt,
            dialogue: Dialogue,
            hero: Hero,
            vigor: Vigor,
            dungeon: Dungeon,
        }
    }
}
