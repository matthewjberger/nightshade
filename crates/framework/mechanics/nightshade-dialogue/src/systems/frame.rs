use crate::resources::{Conversation, DialogueSettings};
use nightshade::prelude::*;
use nightshade_platform::contract::time::Time;

pub fn reveal_the_line(
    time: Res<Time>,
    settings: Res<DialogueSettings>,
    mut conversation: ResMut<Conversation>,
) {
    let letters = conversation.line.chars().count() as f32;
    if conversation.revealed >= letters {
        return;
    }
    conversation.revealed =
        (conversation.revealed + settings.letters_a_second * time.delta_time).min(letters);
}
