use crate::data::Ask;
use crate::resources::{
    Aim, Art, Chests, Cursor, Dialogue, Digging, Draft, Effects, Fireworks, Framing, Generation,
    Hand, Horde, Inventory, Player, Prompt, Save, Screen, Settings, Sheet, Toast, UiHandles,
    Weather,
};
use crate::systems;
use crate::theme;
use nightshade::prelude::*;
use nightshade_input::InputPlugin;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_voxels::VoxelsPlugin as BlockWorldPlugin;

pub struct VoxelsPlugin;

impl Plugin for VoxelsPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        app.insert_resource(Generation::default());
        app.insert_resource(Player::default());
        app.insert_resource(Inventory::default());
        app.insert_resource(Aim::default());
        app.insert_resource(Hand::default());
        app.insert_resource(Prompt::default());
        app.insert_resource(Dialogue::default());
        app.insert_resource(Framing::default());
        app.insert_resource(Toast::default());
        app.insert_resource(Fireworks::default());
        app.insert_resource(Sheet::default());
        app.insert_resource(Effects::default());
        app.insert_resource(Settings::default());
        app.insert_resource(Chests::default());
        app.insert_resource(UiHandles::default());
        app.insert_resource(Horde::default());
        app.insert_resource(Save::default());
        app.insert_resource(Cursor::default());
        app.insert_resource(Weather::default());
        app.insert_resource(Art::default());
        app.insert_resource(Draft::default());
        app.add_plugin(StatePlugin::new(Screen::Title));
        app.add_plugin(InputPlugin::<Ask>::new());

        app.add_system(
            Stage::Startup,
            (
                systems::setup::initialize,
                systems::save::read_the_save,
                systems::generation::place_player,
                systems::save::take_the_save,
                systems::save::sow_the_saved_things,
                systems::life::populate,
                systems::interface::build,
            ),
        );
        app.add_system(
            Stage::Update,
            (
                systems::menus::handle_the_keys,
                systems::menus::keep_the_cursor,
            ),
        );
        app.add_system(
            Stage::Update,
            (
                while_in(
                    Screen::Playing,
                    (
                        systems::boats::sail_the_boats,
                        systems::player::control,
                        systems::watch::ride_the_subject,
                        systems::interact::aim,
                        systems::interact::paint_the_crack,
                        systems::interact::use_item,
                        systems::interact::find_the_prompt,
                        systems::interact::act_on_the_prompt,
                        systems::monsters::swing_at_the_monsters,
                    ),
                ),
                while_in(Screen::Talking, (systems::menus::hold_the_conversation,)),
                while_in(Screen::Shop, (systems::menus::shop_buttons,)),
                while_in(Screen::Crafting, (systems::menus::crafting_buttons,)),
                while_in(Screen::Inventory, (systems::menus::pocket_buttons,)),
                while_in(Screen::Chest, (systems::menus::chest_buttons,)),
                while_in(Screen::Settings, (systems::menus::settings_buttons,)),
                while_in(
                    Screen::Title,
                    (
                        systems::menus::title_buttons,
                        systems::world::look_over_the_world,
                    ),
                ),
                while_in(
                    Screen::NewWorld,
                    (
                        systems::menus::new_world_buttons,
                        systems::world::look_over_the_world,
                    ),
                ),
            ),
        );
        app.add_system(
            Stage::Update,
            (
                systems::world::start_the_world,
                systems::world::raise_the_new_world,
            ),
        );
        app.add_system(
            Stage::Update,
            (
                systems::sky::paint_the_sky,
                systems::sight::aim_the_viewer,
                systems::talk::stage_the_talk,
                systems::gravity::drop_the_fallen,
                systems::life::wander,
                systems::farming::grow,
                systems::effects::fly_the_shells,
                systems::effects::follow_the_blocks,
                systems::items::drift,
                systems::hand::hold,
            ),
        );
        app.add_system(Stage::Update, systems::vitals::keep_the_body);
        app.add_system(Stage::Update, systems::interface::show_the_vitals);
        app.add_system(Stage::Update, systems::monsters::raise_the_monsters);
        app.add_system(Stage::Update, systems::monsters::hunt_the_player);
        app.add_system(Stage::Update, systems::save::keep_the_world);
        app.add_system(Stage::Update, systems::weather::turn_the_weather);
        app.add_system(Stage::Update, systems::textures::repaint_the_blocks);
        app.add_plugin(BlockWorldPlugin::<Digging>::default());
        app.add_system(
            Stage::Update,
            (
                systems::interface::show_the_screen,
                systems::interface::show_the_draft,
                systems::interface::refresh,
                systems::interface::refresh_panels,
            ),
        );
    }
}
