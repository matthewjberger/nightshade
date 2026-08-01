use crate::resources::{
    CatalogPanel, CharacterPanel, ChestPanel, CraftingPanel, DialoguePanel, JournalPanel,
    LoadingPanel, MapArt, MapPanel, NewWorldPanel, PocketPanel, Screens, SettingsPanel, SetupPanel,
    ShopPanel, TitlePanel,
};
use crate::systems;
use blocker_hud::data::settings::Settings;
use blocker_screen::data::screen::Screen;
use nightshade::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui_config::ConfigUiPlugin;

pub struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            blocker_hud::resources::ShellHandles => "the hud plugin",
            nightshade_ui_menu::resources::Menus<Screen> => "the hud plugin"
        ]
    }

    fn build(&self, app: &mut App) {
        app.add_plugin(ConfigUiPlugin::<Settings>::new());
        app.add_plugin(ConfigUiPlugin::<blocker_player::data::settings::Settings>::new());
        app.add_plugin(ConfigUiPlugin::<blocker_weather::data::settings::Settings>::new());
        app.add_plugin(ConfigUiPlugin::<blocker_crew::data::settings::Settings>::new());
        app.add_plugin(ConfigUiPlugin::<blocker_beasts::data::settings::Settings>::new());
        app.insert_resource(Screens::default());
        app.insert_resource(PocketPanel::default());
        app.insert_resource(CraftingPanel::default());
        app.insert_resource(ChestPanel::default());
        app.insert_resource(TitlePanel::default());
        app.insert_resource(LoadingPanel::default());
        app.insert_resource(NewWorldPanel::default());
        app.insert_resource(SettingsPanel::default());
        app.insert_resource(CatalogPanel::default());
        app.insert_resource(SetupPanel::default());
        app.insert_resource(MapPanel::default());
        app.insert_resource(CharacterPanel::default());
        app.insert_resource(JournalPanel::default());
        app.insert_resource(DialoguePanel::default());
        app.insert_resource(ShopPanel::default());
        app.insert_resource(MapArt::default());
        app.add_system(
            Stage::Startup,
            (
                systems::screens::title::build_title,
                systems::screens::new_world::build_new_world,
                systems::screens::loading::build_loading,
                systems::screens::setup::build_setup,
                systems::screens::settings::build_settings,
                systems::screens::pocket::build_pocket,
                systems::screens::crafting::build_crafting,
                systems::screens::chest::build_chest,
                systems::screens::map::build_map,
                systems::screens::character::build_character,
                systems::screens::journal::build_journal,
                systems::screens::dialogue::build_dialogue,
                systems::screens::shop::build_shop,
            ),
        );
        order_the_screens(app);
        app.add_system(
            Stage::Update,
            (
                while_in(
                    Screen::Loading,
                    (systems::screens::loading::watch_the_columns,),
                ),
                while_in(
                    Screen::Map,
                    (
                        systems::screens::map::paint_the_map,
                        systems::screens::map::show_the_map,
                        systems::screens::map::map_buttons,
                    ),
                ),
                while_in(
                    Screen::Character,
                    (
                        systems::screens::character::show_the_character,
                        systems::screens::character::character_buttons,
                    ),
                ),
                while_in(
                    Screen::Journal,
                    (
                        systems::screens::journal::show_the_journal,
                        systems::screens::journal::journal_buttons,
                    ),
                ),
                while_in(
                    Screen::Dialogue,
                    (
                        systems::screens::dialogue::show_the_dialogue,
                        systems::screens::dialogue::dialogue_buttons,
                    ),
                ),
                while_in(
                    Screen::Shop,
                    (
                        systems::screens::shop::show_the_shop,
                        systems::screens::shop::shop_buttons,
                    ),
                ),
            ),
        );
        app.add_system(
            Stage::Update,
            (
                while_in(Screen::Title, (systems::screens::title::title_buttons,)),
                while_in(
                    Screen::NewWorld,
                    (systems::screens::new_world::new_world_buttons,),
                ),
                while_in(Screen::Setup, (systems::screens::setup::setup_buttons,)),
                while_in(
                    Screen::Settings,
                    (
                        systems::screens::settings::show_the_toggles,
                        systems::screens::settings::settings_buttons,
                    ),
                ),
                while_in(
                    Screen::Crafting,
                    (systems::screens::crafting::crafting_buttons,),
                ),
                while_in(
                    Screen::Inventory,
                    (systems::screens::pocket::pocket_buttons,),
                ),
                while_in(Screen::Chest, (systems::screens::chest::chest_buttons,)),
            ),
        );
        app.add_system(
            Stage::Update,
            (
                systems::fresh::forget_the_loading,
                systems::screens::settings::keep_the_sight,
                systems::screens::new_world::show_the_draft,
                systems::screens::new_world::show_the_catalog,
                systems::frame::refresh_panels,
            ),
        );
    }
}

fn order_the_screens(app: &mut App) {
    let shell = std::any::type_name_of_val(&blocker_hud::systems::build::build);
    let calm = std::any::type_name_of_val(&blocker_hud::systems::frame::calm_the_interface);
    for screen in [
        std::any::type_name_of_val(&systems::screens::title::build_title),
        std::any::type_name_of_val(&systems::screens::new_world::build_new_world),
        std::any::type_name_of_val(&systems::screens::loading::build_loading),
        std::any::type_name_of_val(&systems::screens::setup::build_setup),
        std::any::type_name_of_val(&systems::screens::settings::build_settings),
        std::any::type_name_of_val(&systems::screens::pocket::build_pocket),
        std::any::type_name_of_val(&systems::screens::crafting::build_crafting),
        std::any::type_name_of_val(&systems::screens::chest::build_chest),
        std::any::type_name_of_val(&systems::screens::map::build_map),
        std::any::type_name_of_val(&systems::screens::character::build_character),
        std::any::type_name_of_val(&systems::screens::journal::build_journal),
        std::any::type_name_of_val(&systems::screens::dialogue::build_dialogue),
        std::any::type_name_of_val(&systems::screens::shop::build_shop),
    ] {
        app.startup.order(shell, screen);
        app.startup.order(screen, calm);
    }
}
