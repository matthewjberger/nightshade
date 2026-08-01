use crate::resources::{
    Aim, Banners, Bursts, Chests, Creation, Cues, Digging, Draft, Effects, Feel, Fireworks,
    Flashes, Generation, Hero, Hits, Hover, HudHandles, HudText, Icons, Inventory, Journal, Models,
    Places, Player, Prompt, Save, Screen, Settings, Sheet, Slain, Toast, Tracks, UiHandles,
    Weather,
};
use crate::systems;
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera_controller::systems::controllers::{
    third_person_mouse_input, third_person_update_transform,
};
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::prelude::RetainedUiGamepadNav;
use nightshade_voxels::VoxelsPlugin as BlockWorldPlugin;

pub struct HollowmerePlugin;

impl Plugin for HollowmerePlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        app.world.res_mut::<RetainedUiGamepadNav>().enabled = false;
        app.insert_resource(Generation::default());
        app.insert_resource(Player::default());
        app.insert_resource(Inventory::default());
        app.insert_resource(Aim::default());
        app.insert_resource(Prompt::default());
        app.insert_resource(Toast::default());
        app.insert_resource(Fireworks::default());
        app.insert_resource(Sheet::default());
        app.insert_resource(Effects::default());
        app.insert_resource(Settings::default());
        app.insert_resource(Chests::default());
        app.insert_resource(UiHandles::default());
        app.insert_resource(Save::default());
        app.insert_resource(Weather::default());
        app.insert_resource(Draft::default());
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
        app.insert_resource(Hover::default());
        app.insert_resource(Creation::default());
        app.insert_resource(Hero::default());
        app.insert_resource(Journal::default());
        app.insert_resource(Places::default());
        app.insert_resource(HudHandles::default());
        app.insert_resource(HudText::default());
        app.add_plugin(StatePlugin::new(Screen::Title));
        app.add_plugin(nightshade_input::InputPlugin::<crate::data::Ask>::new());

        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Startup, systems::setup::stage);
        app.add_system(Stage::Startup, systems::save::read_the_save);
        app.add_system(Stage::Startup, systems::generation::place_player);
        app.add_system(Stage::Startup, systems::save::take_the_save);
        app.add_system(Stage::Startup, systems::save::sow_the_saved_things);
        app.add_system(Stage::Startup, systems::interface::build);
        app.add_system(Stage::Startup, systems::sound::load);
        app.on_enter(Screen::Creation, systems::creation::open);
        app.on_enter(Screen::Playing, systems::creation::enter_world);
        app.add_system(
            Stage::Update,
            (
                systems::menus::handle_the_keys,
                systems::world::start_the_world,
                systems::generation::raise_the_new_world,
                systems::world::people_the_world,
                systems::models::demand,
                systems::models::collect,
                systems::models::dress,
                systems::rooted::settle,
                systems::ground::land,
                systems::sky::drift,
                systems::sky::wheel,
            ),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Creation, systems::creation::choose),
        );
        app.add_system(
            Stage::Update,
            while_in_any(
                [Screen::Playing, Screen::Creation],
                (third_person_mouse_input, third_person_update_transform),
            ),
        );
        app.add_system(
            Stage::Update,
            (
                while_in(
                    Screen::Playing,
                    (
                        systems::picking::hover,
                        systems::picking::click,
                        systems::player::run,
                        systems::player::select_target,
                        systems::player::cast,
                        systems::player::grow,
                        systems::interact::aim,
                        systems::interact::mine,
                        systems::interact::paint_the_crack,
                        systems::interact::use_item,
                        systems::interact::choose_slot,
                        systems::interact::find_the_prompt,
                        systems::interact::act_on_the_prompt,
                        systems::menus::hotbar_buttons,
                    ),
                ),
                while_in(
                    Screen::Playing,
                    (
                        systems::pickups::gather,
                        systems::quests::givers,
                        systems::quests::progress,
                        systems::quests::tally,
                        systems::quests::explore,
                        systems::chat::deliver,
                        systems::chat::scroll,
                        systems::plates::highlight,
                        systems::hud::calling,
                        systems::hud::refresh,
                        systems::hud::minimap,
                        systems::hud::banners,
                        systems::worldmap::paint,
                        systems::worldmap::show,
                    ),
                ),
                while_in(
                    Screen::Playing,
                    (
                        systems::camera::keep_out_of_the_blocks,
                        systems::camera::first_person,
                    ),
                ),
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
                systems::sky::paint_the_sky,
                systems::camera::aim_the_viewer,
                systems::gravity::drop_the_fallen,
                systems::farming::grow,
                systems::effects::fly_the_shells,
                systems::effects::follow_the_blocks,
                systems::items::drift,
                systems::weather::turn_the_weather,
                systems::save::note_the_hero,
                systems::save::keep_the_world,
            ),
        );
        app.add_system(
            Stage::Update,
            (
                systems::feel::shake,
                systems::monsters::roam,
                systems::monsters::space,
                systems::people::keep_the_loop,
                systems::people::population,
                systems::guards::patrol,
                systems::combat::swings,
                systems::projectiles::fly,
                systems::combat::land,
                systems::combat::deaths,
                systems::combat::mend,
                systems::combat::effects,
                systems::animation::strikes,
                systems::animation::pose,
                systems::animation::hooves,
                systems::animation::flinches,
            ),
        );
        app.add_system(
            Stage::Update,
            (
                systems::animation::falls,
                systems::animation::rises,
                systems::spells::animate,
                systems::spells::auras,
                systems::chat::roster,
                systems::chat::react,
                systems::chat::conversations,
                systems::chat::collect,
                systems::plates::relabel,
                systems::plates::marks,
                systems::plates::follow,
                systems::floating::show,
                systems::floating::rise,
                systems::feel::bursts,
                systems::feel::settle,
                systems::sound::cues,
            ),
        );
        app.add_system(
            Stage::Update,
            (systems::sound::music, systems::combat::forget),
        );
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
