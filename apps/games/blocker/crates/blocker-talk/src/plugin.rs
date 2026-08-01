use crate::resources::{Deeds, Objective, Shopping};
use crate::systems;
use crate::theme;
use blocker_screen::data::screen::Screen;
use nightshade::prelude::*;
use nightshade_dialogue::{DialoguePlugin, DialogueSettings};
use nightshade_state::prelude::*;

pub struct TalkPlugin;

impl Plugin for TalkPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![nightshade_random::Random => "the app plugin"]
    }

    fn build(&self, app: &mut App) {
        app.add_plugin(DialoguePlugin {
            settings: DialogueSettings {
                letters_a_second: theme::LETTERS_A_SECOND,
            },
        });
        app.insert_resource(Deeds::default());
        app.insert_resource(Objective::default());
        app.insert_resource(Shopping::default());
        app.add_system(
            Stage::Update,
            while_in(
                Screen::Playing,
                (
                    systems::talk::talk_to_the_crew,
                    systems::marks::mark_the_givers,
                ),
            ),
        );
    }
}
