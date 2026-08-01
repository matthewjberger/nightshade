use blocker_blocks::data::blocks::Palette;
use blocker_blocks::data::items::ItemStack;
use blocker_chat::data::chat::CHANNELS;
use blocker_screen::data::screen::Screen;
use nalgebra_glm::Vec4;
use nightshade::ecs::Entity;
use nightshade_toast::Toast;
use nightshade_ui_menu::prelude::{CardStyle, Menus, SlotHandles};

#[derive(Default, Clone)]
pub struct ShellHandles {
    pub root: Entity,
    pub backdrop: Entity,
    pub shown: Option<Screen>,
}

#[derive(Default, Clone)]
pub struct ArtLayers {
    pub atlas: Option<u32>,
    pub tools: Option<u32>,
    pub frame: Option<u32>,
}

#[derive(Default, Clone)]
pub struct VitalsPanel {
    pub name_label: Entity,
    pub shown_name: String,
    pub health_fill: Entity,
    pub health_text: Entity,
    pub food_fill: Entity,
    pub food_text: Entity,
    pub xp_fill: Entity,
    pub xp_text: Entity,
    pub shown_health: Option<u32>,
    pub shown_food: Option<u32>,
    pub shown_xp: Option<u32>,
}

#[derive(Default, Clone)]
pub struct TargetPanel {
    pub panel: Entity,
    pub name: Entity,
    pub health_fill: Entity,
    pub health_text: Entity,
    pub shown: Option<(Entity, u32, bool)>,
}

#[derive(Default, Clone)]
pub struct HotbarPanel {
    pub hud: Entity,
    pub gear: Entity,
    pub slots: [SlotHandles; blocker_blocks::data::items::HOTBAR_SLOTS],
    pub status: Entity,
    pub shown: [Option<ItemStack>; blocker_blocks::data::items::HOTBAR_SLOTS],
    pub shown_selected: Option<usize>,
    pub shown_status: String,
    pub shown_watching: Option<bool>,
}

#[derive(Default, Clone)]
pub struct PromptPanel {
    pub panel: Entity,
    pub label: Entity,
    pub shown: Option<String>,
}

#[derive(Default, Clone)]
pub struct ToastPanel {
    pub panel: Entity,
    pub label: Entity,
    pub shown: String,
    pub visible: bool,
}

#[derive(Default, Clone)]
pub struct CompassPanel {
    pub panel: Entity,
    pub points: Vec<Entity>,
    pub marks: Vec<Entity>,
    pub notch: Entity,
    pub readout_panel: Entity,
    pub readout: Entity,
    pub shown: Vec<bool>,
    pub shown_marks: Vec<usize>,
    pub shown_readout: String,
}

#[derive(Default, Clone)]
pub struct RatePanel {
    pub panel: Entity,
    pub label: Entity,
    pub shown: Option<bool>,
    pub shown_text: String,
    pub shown_color: Option<Vec4>,
}

#[derive(Default, Clone)]
pub struct ChatPanel {
    pub panel: Entity,
    pub tabs: Vec<Entity>,
    pub marks: Vec<Entity>,
    pub rows: Vec<Entity>,
    pub entry: Entity,
    pub shown_body: String,
    pub shown_key: Option<(u64, usize, usize)>,
    pub shown_tab: Option<usize>,
    pub shown_unread: Vec<bool>,
}

#[derive(Default)]
pub struct ChatEntry {
    pub scroll: usize,
    pub active_tab: usize,
    pub seen: [u64; CHANNELS],
}

#[derive(Default, Clone)]
pub struct QuestPanel {
    pub panel: Entity,
    pub title: Entity,
    pub rows: Vec<Entity>,
    pub shown: Vec<String>,
}

#[derive(Default)]
pub struct Icons {
    pub layers: [Option<u32>; 2],
    pub painted: Option<Palette>,
}

nightshade::ecs::resource_bundle! {
    pub struct HudPanels<'a> {
        write {
            hotbar: HotbarPanel,
            rate: RatePanel,
            prompt: PromptPanel,
            toast_panel: ToastPanel,
        }
        read {
            layers: ArtLayers,
            shell: ShellHandles,
            toast: Toast,
        }
    }
}

nightshade::ecs::resource_bundle! {
    pub struct HudBuild<'a> {
        write {
            shell: ShellHandles,
            menus: Menus<Screen>,
            art: ArtLayers,
            icons: Icons,
            vitals: VitalsPanel,
            target: TargetPanel,
            hotbar: HotbarPanel,
            rate: RatePanel,
            chat: ChatPanel,
            quests: QuestPanel,
            compass: CompassPanel,
            prompt: PromptPanel,
            toast: ToastPanel,
        }
        read {
            style: CardStyle,
        }
    }
}
