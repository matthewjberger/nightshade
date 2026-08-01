use crate::data::settings::Settings;
use crate::resources::Crew;
use crate::systems;
use blocker_screen::data::screen::Screen;
use nightshade::prelude::*;
use nightshade_config::prelude::ConfigPlugin;
use nightshade_state::prelude::*;

pub struct CrewPlugin;

impl Plugin for CrewPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![nightshade_random::Random => "the app plugin"]
    }

    fn build(&self, app: &mut App) {
        app.add_plugin(blocker_figure::plugin::FigurePlugin);
        app.add_plugin(ConfigPlugin::<Settings>::new().persist("blocker", "crew.toml"));
        app.insert_resource(Crew::default());
        app.add_system(Stage::Update, systems::fresh::clear_the_crew);
        app.add_system(
            Stage::LateUpdate,
            while_in(Screen::Playing, systems::space::hold_the_crew_apart),
        );
        app.add_system(
            Stage::Update,
            while_in(
                Screen::Playing,
                (
                    systems::crew::populate,
                    systems::crew::wander,
                    systems::crew::stride,
                ),
            ),
        );
    }
}
