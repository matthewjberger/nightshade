use crate::commands::spawn_beast;
use blocker_screen::data::asks::BeastWanted;
use nightshade::ecs::system_param::EventReader;
use nightshade::prelude::*;

pub fn answer_the_asks(asks: EventReader<BeastWanted>, mut commands: Commands) {
    for ask in &asks {
        spawn_beast(&mut commands, ask.place, ask.heading, ask.kind, ask.lair);
    }
}
