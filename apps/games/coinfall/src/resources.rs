use crate::data;
use nightshade::prelude::Entity;
use nightshade_cli::clap;
use nightshade_sprite::prelude::SpriteRegion;
use nightshade_tiles::prelude::TileMap;

#[derive(clap::Args, Default, Clone)]
pub struct Arguments {
    #[arg(long, help = "Open in a level instead of the title screen")]
    pub play: bool,
    #[arg(long, help = "Open on the overworld instead of the title screen")]
    pub map: bool,
    #[arg(long, help = "Open in the level with this index")]
    pub level: Option<usize>,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Title,
    Overworld,
    Playing,
    Paused,
    Cleared,
}

#[derive(Default)]
pub struct CoinfallResources {
    pub run: RunState,
    pub hud: HudHandles,
    pub dots: Vec<Entity>,
    pub sheets: Sheets,
    pub solids: TileMap,
    pub camera: Option<Entity>,
    pub camera_x: f32,
    pub player: Option<Entity>,
}

pub struct RunState {
    pub screen: Screen,
    pub wanted_screen: Option<Screen>,
    pub level: usize,
    pub wanted_level: Option<usize>,
    pub selected: usize,
    pub unlocked: usize,
    pub bests: Vec<u32>,
    pub seconds: f32,
    pub coins: u32,
    pub score: u32,
    pub restart_wanted: bool,
    pub last_score: u32,
    pub last_selected: usize,
}

impl Default for RunState {
    fn default() -> Self {
        Self {
            screen: Screen::Title,
            wanted_screen: Some(Screen::Title),
            level: 0,
            wanted_level: Some(0),
            selected: 0,
            unlocked: 1,
            bests: vec![0; data::LEVELS.len()],
            seconds: data::LEVELS[0].seconds,
            coins: 0,
            score: 0,
            restart_wanted: false,
            last_score: u32::MAX,
            last_selected: usize::MAX,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct Sheets {
    pub tiles: SpriteRegion,
    pub characters: SpriteRegion,
    pub backgrounds: SpriteRegion,
    pub blank: SpriteRegion,
}

#[derive(Default, Clone, Copy)]
pub struct HudHandles {
    pub title: Entity,
    pub play_button: Entity,
    pub quit_button: Entity,
    pub overworld: Entity,
    pub overworld_name: Entity,
    pub overworld_best: Entity,
    pub start_button: Entity,
    pub overworld_title_button: Entity,
    pub pause: Entity,
    pub resume_button: Entity,
    pub restart_button: Entity,
    pub pause_map_button: Entity,
    pub cleared: Entity,
    pub cleared_score: Entity,
    pub again_button: Entity,
    pub cleared_map_button: Entity,
}
