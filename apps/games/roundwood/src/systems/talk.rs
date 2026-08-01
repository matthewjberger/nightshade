use crate::data;
use crate::data::Ask;
use crate::resources::{Chat, Screen};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_state::prelude::*;

pub fn hold_the_conversation(
    actions: Res<Actions<Ask>>,
    mut chat: ResMut<Chat>,
    mut state_next: ResMut<NextState<Screen>>,
) {
    if !actions.just_pressed(Ask::Advance) {
        return;
    }
    let done = {
        chat.line += 1;
        let lines = chat
            .villager
            .map_or(0, |index| data::NEIGHBORS[index].lines.len());
        chat.line >= lines
    };
    if !done {
        return;
    }
    chat.shown = false;
    chat.villager = None;
    chat.line = 0;
    state_next.pending = Some(Screen::Playing);
}
