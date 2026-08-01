use blocker_blocks::data::blocks::Palette;
use blocker_blocks::data::items::ItemStack;
use blocker_hud::resources::ArtLayers;
use blocker_screen::data::screen::Screen;
use nightshade::ecs::Entity;
use nightshade_config::prelude::Config;
use nightshade_ui_config::prelude::ConfigPanel;
use nightshade_ui_menu::prelude::SlotHandles;

nightshade::ecs::resource_bundle! {
    pub struct Tuning<'a> {
        write {
            interface_rows: ConfigPanel<blocker_hud::data::settings::Settings>,
            player_rows: ConfigPanel<blocker_player::data::settings::Settings>,
            weather_rows: ConfigPanel<blocker_weather::data::settings::Settings>,
            crew_rows: ConfigPanel<blocker_crew::data::settings::Settings>,
            beast_rows: ConfigPanel<blocker_beasts::data::settings::Settings>,
        }
        read {
            interface: Config<blocker_hud::data::settings::Settings>,
            player: Config<blocker_player::data::settings::Settings>,
            weather: Config<blocker_weather::data::settings::Settings>,
            crew: Config<blocker_crew::data::settings::Settings>,
            beasts: Config<blocker_beasts::data::settings::Settings>,
        }
    }
}

pub type JournalKey = (Vec<(usize, String, bool)>, usize);

pub type ShopKey = (usize, u32, Option<(String, u32, u32)>);

pub type DialogueKey = (String, String, bool);

#[derive(Default, Clone, Copy)]
pub struct CraftRowHandles {
    pub index: usize,
    pub needs: Entity,
    pub button: Entity,
}

#[derive(Default, Clone)]
pub struct PocketPanel {
    pub panel: Entity,
    pub craft: Entity,
    pub close: Entity,
    pub slots: Vec<SlotHandles>,
    pub detail: Entity,
    pub hint: Entity,
    pub holding: Option<usize>,
    pub shown: Vec<Option<ItemStack>>,
    pub shown_marks: Option<(Option<usize>, usize)>,
    pub shown_detail: String,
}

#[derive(Default, Clone)]
pub struct CraftingPanel {
    pub panel: Entity,
    pub rows: Vec<CraftRowHandles>,
    pub close: Entity,
    pub shown: Vec<bool>,
}

#[derive(Default, Clone)]
pub struct ChestPanel {
    pub panel: Entity,
    pub slots: Vec<SlotHandles>,
    pub pocket: Vec<SlotHandles>,
    pub close: Entity,
    pub shown: Vec<Option<ItemStack>>,
    pub shown_pocket: Vec<Option<ItemStack>>,
}

#[derive(Default, Clone)]
pub struct TitlePanel {
    pub panel: Entity,
    pub resume: Entity,
    pub new: Entity,
    pub leave: Entity,
    pub settings: Entity,
}

#[derive(Default, Clone)]
pub struct LoadingPanel {
    pub panel: Entity,
    pub fill: Entity,
    pub count: Entity,
    pub place: Entity,
    pub tip: Entity,
    pub enter_row: Entity,
    pub enter: Entity,
    pub welcomed: bool,
    pub enter_shown: bool,
    pub tip_index: usize,
    pub tip_since: f32,
    pub shown_count: String,
    pub shown_place: String,
    pub shown_tip: String,
}

#[derive(Default, Clone)]
pub struct NewWorldPanel {
    pub panel: Entity,
    pub seed_button: Entity,
    pub seed_label: Entity,
    pub make: Entity,
    pub back: Entity,
    pub shown_seed: Option<u32>,
    pub shown_found: Option<bool>,
}

#[derive(Default, Clone)]
pub struct SettingsPanel {
    pub panel: Entity,
    pub close: Entity,
    pub watch_toggle: Entity,
    pub save: Entity,
    pub title: Entity,
    pub shown_watching: Option<bool>,
}

#[derive(Default, Clone)]
pub struct CatalogPanel {
    pub all: Entity,
    pub buttons: Vec<Entity>,
    pub labels: Vec<Entity>,
    pub shown: Option<u32>,
}

#[derive(Default, Clone)]
pub struct SetupPanel {
    pub panel: Entity,
    pub name_input: Entity,
    pub play: Entity,
}

#[derive(Default, Clone)]
pub struct MapPanel {
    pub panel: Entity,
    pub picture: Entity,
    pub here: Entity,
    pub nose: Entity,
    pub markers: Vec<Entity>,
    pub labels: Vec<Entity>,
    pub readout: Entity,
    pub close: Entity,
    pub layer: Option<u32>,
    pub zoom: usize,
    pub painted: Option<([i32; 2], usize)>,
    pub bar: Entity,
    pub bar_label: Entity,
    pub shown_zoom: Option<usize>,
}

#[derive(Default, Clone)]
pub struct CharacterPanel {
    pub panel: Entity,
    pub rows: Vec<Entity>,
    pub close: Entity,
    pub shown: Option<(Vec<String>, i32, i32)>,
    pub caption: Entity,
    pub rank: Entity,
    pub health_fill: Entity,
    pub health_text: Entity,
    pub food_fill: Entity,
    pub food_text: Entity,
    pub xp_fill: Entity,
    pub xp_text: Entity,
}

#[derive(Default, Clone)]
pub struct JournalPanel {
    pub panel: Entity,
    pub rows: Vec<(Entity, Entity)>,
    pub detail_title: Entity,
    pub detail_giver: Entity,
    pub detail_story: Entity,
    pub detail_next: Entity,
    pub detail_objective: Entity,
    pub detail_reward: Entity,
    pub picked: usize,
    pub close: Entity,
    pub shown: Option<JournalKey>,
}

#[derive(Default, Clone)]
pub struct DialoguePanel {
    pub panel: Entity,
    pub name: Entity,
    pub scroll: Entity,
    pub body: Entity,
    pub objective: Entity,
    pub rows: Vec<Entity>,
    pub choices: Vec<Entity>,
    pub shown: Option<DialogueKey>,
}

#[derive(Default, Clone)]
pub struct ShopPanel {
    pub panel: Entity,
    pub coin: Entity,
    pub rows: Vec<Entity>,
    pub sell: Entity,
    pub close: Entity,
    pub shown: Option<ShopKey>,
}

#[derive(Default)]
pub struct MapArt {
    pub palette: Option<Palette>,
    pub pixels: Vec<u8>,
    pub width: u32,
}

pub struct Screens {
    pub return_to: Screen,
}

impl Default for Screens {
    fn default() -> Self {
        Self {
            return_to: Screen::Title,
        }
    }
}

nightshade::ecs::resource_bundle! {
    pub struct ShelfPanels<'a> {
        write {
            chest: ChestPanel,
            crafting: CraftingPanel,
            pocket: PocketPanel,
        }
        read {
            layers: ArtLayers,
        }
    }
}
