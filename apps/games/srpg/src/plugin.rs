use crate::data::{Ask, CHAPTERS};
use crate::queries;
use crate::resources::{
    Battle, Battlefield, CameraRig, Camp, Campaign, Cursor, Highlights, Pause, Screen, Sheet,
    SrpgArguments, Terrain, Tracks, UiHandles,
};
use crate::systems;
use crate::theme;
use fastrand::Rng;
use nightshade::prelude::*;
use nightshade_input::InputPlugin;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;

pub struct SrpgPlugin;

impl Plugin for SrpgPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        let (hour, chapter, auto, screen) = {
            let arguments = app.world.res::<SrpgArguments>();
            (
                arguments.hour,
                queries::opening_chapter(arguments),
                arguments.auto,
                queries::opening_screen(arguments),
            )
        };
        app.insert_resource(Battlefield {
            start_hour: hour.unwrap_or(theme::START_HOUR),
            fixed_hour: hour,
            auto,
            bars_shown: true,
            ..Default::default()
        });
        app.insert_resource(Terrain::default());
        app.insert_resource(Sheet::default());
        app.insert_resource(CameraRig::default());
        app.insert_resource(Cursor::default());
        app.insert_resource(Battle {
            rng: Rng::with_seed(4242),
            ..Default::default()
        });
        app.insert_resource(Camp::default());
        app.insert_resource(Campaign {
            chapter: chapter.min(CHAPTERS.len() - 1),
            march: screen == Screen::Battle,
            ..Default::default()
        });
        app.add_plugin(nightshade_audio::AudioPlugin);
        app.insert_resource(Tracks::default());
        app.insert_resource(Highlights::default());
        app.insert_resource(Pause::default());
        app.insert_resource(UiHandles::default());
        app.add_plugin(StatePlugin::new(screen));
        app.add_plugin(InputPlugin::<Ask>::new());

        app.add_system(
            Stage::Startup,
            (
                systems::setup::initialize,
                systems::campaign::raise_the_squad,
                systems::campaign::dress_the_title,
                systems::interface::build,
                systems::sound::open_the_tracks,
            ),
        );
        app.add_system(
            Stage::Update,
            (
                systems::menus::handle_the_keys,
                systems::camp::read_the_camp,
                systems::cursor::read_the_input,
                systems::camera::drive_the_rig,
                systems::daylight::follow_the_hour,
                systems::effects::tend_the_fireflies,
                systems::units::spawn_the_squads,
                systems::campaign::follow_the_shade,
                systems::viewport::fit_the_interface,
            ),
        );
        app.add_system(
            Stage::Update,
            (
                while_in(Screen::Title, (systems::menus::title_buttons,)),
                while_in(Screen::Paused, (systems::menus::pause_buttons,)),
                while_in(Screen::Over, (systems::menus::over_buttons,)),
                while_in_any(
                    [Screen::Prologue, Screen::Camp],
                    (systems::campaign::open_the_camp,),
                ),
                while_in(Screen::Prologue, (systems::campaign::read_the_prologue,)),
                while_in(
                    Screen::Camp,
                    (
                        systems::camp::walk_the_commander,
                        systems::camp::find_the_prompt,
                        systems::camp::act_on_the_prompt,
                        systems::camp::speak_in_camp,
                        systems::camp::camp_menu,
                        systems::camp::pick_the_partner,
                        systems::camp::pick_the_upgrade,
                        systems::camp::muster_menu,
                        systems::camp::stir_the_speaker,
                    ),
                ),
                while_in(
                    Screen::Camp,
                    (systems::camp::pass_the_day, systems::menus::camp_buttons),
                ),
                systems::campaign::keep_the_run,
                systems::campaign::march_out,
                while_in(
                    Screen::Battle,
                    (
                        systems::cursor::toggle_the_danger,
                        systems::cursor::toggle_the_bars,
                        systems::turns::tick_the_clocks,
                        systems::turns::open_the_turn,
                        systems::menus::command_menu,
                        systems::menus::ability_menu,
                        systems::movement::choose_the_move,
                    ),
                ),
                while_in(
                    Screen::Battle,
                    (
                        systems::movement::walk_the_path,
                        systems::combat::choose_the_target,
                        systems::combat::resolve_the_action,
                        systems::ai::think,
                        systems::turns::close_the_turn,
                        systems::campaign::finish_the_mission,
                    ),
                ),
            ),
        );
        app.add_system(
            Stage::Update,
            (
                systems::camera::turn_the_uprights,
                systems::units::face_the_camera,
                systems::units::show_the_statuses,
                systems::units::show_the_bars,
                systems::decor::paint_the_structures,
                systems::digits::float_the_digits,
                systems::cursor::sound_the_cursor,
                systems::cursor::show_the_cursor,
                systems::cursor::show_the_highlights,
                systems::interface::show_the_screen,
                systems::interface::refresh,
                systems::interface::refresh_the_camp,
                systems::interface::refresh_the_prologue,
                systems::interface::refresh_the_pause,
                systems::sound::follow_the_screen,
            ),
        );
    }
}
