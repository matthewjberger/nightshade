use crate::commands::narrate;
use crate::resources::Chat;
use blocker_screen::data::notice::Narration;
use nightshade::ecs::system_param::EventReader;
use nightshade::prelude::*;

pub fn hear_the_narration(lines: EventReader<Narration>, mut chat: ResMut<Chat>) {
    for line in &lines {
        narrate(&mut chat, &line.0);
    }
}
