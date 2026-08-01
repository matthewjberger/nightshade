use crate::resources::{Chat, Chatter, Noted, Roster};
use crate::systems;
use blocker_screen::data::screen::Screen;
use nightshade::prelude::*;
use nightshade_state::prelude::*;

pub struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![nightshade_random::Random => "the app plugin"]
    }

    fn build(&self, app: &mut App) {
        app.insert_resource(Chat::default());
        app.insert_resource(Chatter::default());
        app.insert_resource(Roster::default());
        app.insert_resource(Noted::default());
        app.add_system(
            Stage::Update,
            (
                systems::fresh::clear_the_chat,
                systems::narration::hear_the_narration,
            ),
        );
        app.add_system(
            Stage::Update,
            while_in(
                Screen::Playing,
                (
                    systems::chat::gather_roster,
                    systems::chat::greet_the_player,
                    systems::chat::banter::conversations,
                    systems::chat::world::announce_the_world,
                    systems::chat::deliver_the_lines,
                ),
            ),
        );
    }
}
