use crate::resources::{Chat, Chatter, Noted, Roster};
use blocker_screen::data::fresh::NewWorld;
use nightshade::ecs::system_param::EventReader;
use nightshade::prelude::*;

pub fn clear_the_chat(
    fresh: EventReader<NewWorld>,
    mut chat: ResMut<Chat>,
    mut chatter: ResMut<Chatter>,
    mut roster: ResMut<Roster>,
    mut noted: ResMut<Noted>,
) {
    if fresh.into_iter().next().is_none() {
        return;
    }
    *chat = Chat::default();
    *chatter = Chatter::default();
    *roster = Roster::default();
    *noted = Noted::default();
}
