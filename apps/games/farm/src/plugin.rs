use crate::data::Ask;
use crate::resources::{
    Dialogue, Diorama, Doors, Dungeon, Farmland, Fireworks, Fishing, Hd2dArguments, Hero, Home,
    Inventory, OpenShop, Prompt, Quarry, Quests, Rig, Screen, Sheet, Terrain, Toast, UiHandles,
    Vigor, Weather,
};
use crate::systems;
use crate::theme;
use nightshade::prelude::*;
use nightshade_input::InputPlugin;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;

pub struct Hd2dPlugin;

impl Plugin for Hd2dPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        let hour = app.world.res::<Hd2dArguments>().hour;
        app.insert_resource(Diorama {
            start_hour: hour.unwrap_or(theme::START_HOUR),
            ..Default::default()
        });
        app.insert_resource(Rig::default());
        app.insert_resource(Terrain::default());
        app.insert_resource(Farmland::default());
        app.insert_resource(Doors::default());
        app.insert_resource(Quarry::default());
        app.insert_resource(Dungeon::default());
        app.insert_resource(Home::default());
        app.insert_resource(Hero::default());
        app.insert_resource(Sheet::default());
        app.insert_resource(Inventory::default());
        app.insert_resource(Vigor::default());
        app.insert_resource(Dialogue::default());
        app.insert_resource(Quests::default());
        app.insert_resource(OpenShop::default());
        app.insert_resource(Prompt::default());
        app.insert_resource(Toast::default());
        app.insert_resource(Fireworks::default());
        app.insert_resource(Weather::default());
        app.insert_resource(Fishing::default());
        app.insert_resource(UiHandles::default());
        app.add_plugin(StatePlugin::new(Screen::Title));
        app.add_plugin(InputPlugin::<Ask>::new());

        app.add_system(
            Stage::Startup,
            (
                systems::setup::initialize,
                systems::interface::build,
                systems::save::load_the_farm,
            ),
        );
        app.add_system(
            Stage::Update,
            (
                systems::menus::handle_the_keys,
                systems::daylight::follow_the_hour,
                systems::daylight::light_the_sprites,
                systems::weather::plan_the_weather,
                systems::weather::rain_and_shine,
                systems::effects::sway_the_plants,
                systems::effects::regrow_the_stumps,
                systems::effects::restock_the_quarry,
                systems::effects::ripen_the_bushes,
                systems::dungeon::tend_the_hero,
                systems::dungeon::build_the_hollow,
                systems::interior::build_the_room,
            ),
        );
        app.add_system(
            Stage::Update,
            (
                systems::farming::grow_the_crops,
                systems::farming::tend_the_plots,
                systems::eggs::lay_the_eggs,
                systems::people::keep_the_schedule,
                systems::effects::tend_the_fireflies,
                systems::effects::light_the_sky,
                systems::effects::shake_the_trees,
                systems::effects::swing_the_tools,
            ),
        );
        app.add_system(
            Stage::Update,
            (
                while_in(
                    Screen::Playing,
                    (
                        systems::camera::turn_the_camera,
                        systems::player::walk_the_farmer,
                        systems::animals::wander,
                        systems::menus::pick_the_slot,
                        systems::eggs::gather_the_eggs,
                        systems::pickups::gather_the_pickups,
                        systems::fishing::wait_for_a_bite,
                        systems::zones::name_the_zone,
                        systems::interact::find_the_prompt,
                        systems::dungeon::find_the_hollow_prompt,
                        systems::interact::act_on_the_prompt,
                        systems::dungeon::act_in_the_hollow,
                        systems::dungeon::stalk_the_hero,
                        systems::dungeon::fight,
                        systems::dungeon::gather_the_loot,
                    ),
                ),
                while_in(Screen::Talking, (systems::dialogue::hold_the_conversation,)),
                while_in(Screen::Paused, (systems::menus::pause_buttons,)),
                while_in(Screen::Title, (systems::menus::title_buttons,)),
                while_in(Screen::Shop, (systems::menus::shop_buttons,)),
                while_in(Screen::Inventory, (systems::menus::eat_from_the_pocket,)),
            ),
        );
        app.add_system(
            Stage::Update,
            (
                systems::camera::follow_the_farmer,
                systems::camera::turn_the_uprights,
                systems::interface::show_the_screen,
                systems::interface::refresh,
            ),
        );
    }
}
