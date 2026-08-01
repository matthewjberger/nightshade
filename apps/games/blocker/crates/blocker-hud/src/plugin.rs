use crate::data::settings::Settings;
use crate::data::style::card_style;
use crate::resources::{
    ArtLayers, ChatEntry, ChatPanel, CompassPanel, HotbarPanel, Icons, PromptPanel, QuestPanel,
    RatePanel, ShellHandles, TargetPanel, ToastPanel, VitalsPanel,
};
use crate::systems;
use crate::theme;
use blocker_screen::data::screen::Screen;
use nightshade::prelude::*;
use nightshade_config::prelude::ConfigPlugin;
use nightshade_state::prelude::*;
use nightshade_toast::{ToastPlugin, ToastSettings};
use nightshade_ui_menu::prelude::{MenuPlugin, MenuStyle};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![nightshade_random::Random => "the app plugin"]
    }

    fn build(&self, app: &mut App) {
        app.add_plugin(ConfigPlugin::<Settings>::new().persist("blocker", "interface.toml"));
        app.add_plugin(MenuPlugin::<Screen>::new(MenuStyle::default()));
        app.add_plugin(ToastPlugin {
            settings: ToastSettings {
                showing: theme::TOAST_SECONDS,
                fading: 0.0,
            },
        });
        app.insert_resource(card_style());
        app.insert_resource(ShellHandles::default());
        app.insert_resource(ArtLayers::default());
        app.insert_resource(VitalsPanel::default());
        app.insert_resource(TargetPanel::default());
        app.insert_resource(HotbarPanel::default());
        app.insert_resource(PromptPanel::default());
        app.insert_resource(ToastPanel::default());
        app.insert_resource(CompassPanel::default());
        app.insert_resource(RatePanel::default());
        app.insert_resource(ChatPanel::default());
        app.insert_resource(QuestPanel::default());
        app.insert_resource(Icons::default());
        app.insert_resource(ChatEntry::default());
        app.add_system(
            Stage::Startup,
            (systems::build::build, systems::frame::calm_the_interface),
        );
        app.add_system(
            Stage::Update,
            while_in(
                Screen::Playing,
                (
                    systems::hud::chat::open_the_chat,
                    systems::hud::chat::send_the_line,
                    systems::hud::outline::edge_the_aimed,
                ),
            ),
        );
        app.add_system(
            Stage::Update,
            (
                systems::fresh::forget_the_chat,
                systems::notices::take_the_notices,
                systems::textures::repaint_the_icons,
                systems::hud::vitals::show_the_vitals,
                systems::hud::target::show_the_target,
                systems::frame::show_the_backdrop,
                systems::hud::chat::choose_chat_tab,
                systems::hud::chat::show_the_chat,
                systems::hud::quests::show_the_quests,
                systems::hud::compass::show_the_compass,
                systems::frame::refresh,
            ),
        );
    }
}
