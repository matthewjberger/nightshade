use crate::data::MENU_STYLE;
use crate::resources::{
    Banners, Bursts, Creation, Cues, Feel, Flashes, Hits, Hover, Icons, Models, Panels, Screen,
    Slain, ThornmereResources, Tracks, Walls,
};
use crate::systems;
use nightshade::prelude::*;
use nightshade_camera_controller::systems::controllers::{
    third_person_mouse_input, third_person_update_transform,
};
use nightshade_platform::prelude::*;
use nightshade_state::prelude::{StateAppExt, StatePlugin, while_in};
use nightshade_ui::prelude::RetainedUiGamepadNav;
use nightshade_ui_menu::prelude::MenuPlugin;

pub struct ThornmerePlugin;

impl Plugin for ThornmerePlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Thornmere".to_string();
        app.insert_resource(ThornmereResources::default());
        app.insert_resource(Hits::default());
        app.insert_resource(Slain::default());
        app.insert_resource(Flashes::default());
        app.insert_resource(Cues::default());
        app.insert_resource(Bursts::default());
        app.insert_resource(Banners::default());
        app.insert_resource(Feel::default());
        app.insert_resource(Tracks::default());
        app.insert_resource(Models::default());
        app.insert_resource(Icons::default());
        app.insert_resource(Walls::default());
        app.insert_resource(Hover::default());
        app.insert_resource(Creation::default());
        app.insert_resource(Panels::default());
        app.world.res_mut::<RetainedUiGamepadNav>().enabled = false;
        app.add_plugin(StatePlugin::new(Screen::Creation));
        app.add_plugin(nightshade_input::InputPlugin::<crate::data::Ask>::new());
        app.add_plugin(MenuPlugin::<Screen>::new(MENU_STYLE));
        app.on_enter(
            Screen::World,
            (systems::menu::enter_world, systems::menu::open_hud),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Creation, systems::menu::choose),
        );
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Startup, systems::setup::build_interface);
        app.add_system(Stage::Startup, systems::sound::load);
        app.add_system(Stage::Update, systems::models::demand);
        app.add_system(Stage::Update, systems::models::collect);
        app.add_system(Stage::Update, systems::models::dress);
        app.add_system(Stage::Update, systems::daylight::turn);
        app.add_system(Stage::Update, third_person_mouse_input);
        app.add_system(Stage::Update, third_person_update_transform);
        app.add_system(Stage::Update, systems::effects::shake);
        app.add_system(Stage::Update, systems::monsters::roam);
        app.add_system(Stage::Update, systems::monsters::space);
        app.add_system(Stage::Update, systems::people::keep_the_loop);
        app.add_system(Stage::Update, systems::people::population);
        app.add_system(Stage::Update, systems::guards::patrol);
        app.add_system(Stage::Update, systems::sky::drift);
        app.add_system(Stage::Update, systems::sky::wheel);
        app.add_system(Stage::Update, systems::combat::swings);
        app.add_system(Stage::Update, systems::projectiles::fly);
        app.add_system(Stage::Update, systems::combat::land);
        app.add_system(Stage::Update, systems::combat::deaths);
        app.add_system(Stage::Update, systems::combat::mend);
        app.add_system(Stage::Update, systems::combat::effects);
        app.add_system(Stage::Update, systems::animation::strikes);
        app.add_system(Stage::Update, systems::animation::pose);
        app.add_system(Stage::Update, systems::animation::hooves);
        app.add_system(Stage::Update, systems::animation::flinches);
        app.add_system(Stage::Update, systems::animation::falls);
        app.add_system(Stage::Update, systems::animation::rises);
        app.add_system(Stage::Update, systems::spells::animate);
        app.add_system(Stage::Update, systems::spells::auras);
        app.add_system(Stage::Update, systems::chat::roster);
        app.add_system(Stage::Update, systems::chat::react);
        app.add_system(Stage::Update, systems::chat::conversations);
        app.add_system(Stage::Update, systems::chat::collect);
        app.add_system(Stage::Update, systems::plates::relabel);
        app.add_system(Stage::Update, systems::plates::marks);
        app.add_system(Stage::Update, systems::plates::follow);
        app.add_system(Stage::Update, systems::floating::show);
        app.add_system(Stage::Update, systems::floating::rise);
        app.add_system(Stage::Update, systems::effects::bursts);
        app.add_system(Stage::Update, systems::effects::settle);
        app.add_system(Stage::Update, systems::effects::lamps);
        app.add_system(Stage::Update, systems::sound::cues);
        app.add_system(Stage::Update, systems::sound::music);
        app.add_system(Stage::Update, systems::combat::forget);
        app.add_system(
            Stage::Update,
            while_in(
                Screen::World,
                (
                    systems::picking::hover,
                    systems::picking::click,
                    systems::player::run,
                    systems::player::leap,
                    systems::player::select_target,
                    systems::player::cast,
                ),
            ),
        );
        app.add_system(
            Stage::Update,
            while_in(
                Screen::World,
                (
                    systems::pickups::gather,
                    systems::quests::givers,
                    systems::quests::progress,
                    systems::quests::explore,
                    systems::player::grow,
                ),
            ),
        );
        app.add_system(
            Stage::Update,
            while_in(
                Screen::World,
                (
                    systems::panels::tend,
                    systems::panels::touch,
                    systems::panels::refresh,
                    systems::save::keep,
                ),
            ),
        );
        app.add_system(
            Stage::Update,
            while_in(
                Screen::World,
                (
                    systems::chat::deliver,
                    systems::chat::scroll,
                    systems::plates::highlight,
                    systems::hud::refresh,
                    systems::hud::minimap,
                    systems::hud::banners,
                    systems::worldmap::show,
                ),
            ),
        );
    }
}
