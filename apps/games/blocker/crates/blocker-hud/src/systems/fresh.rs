use crate::resources::ChatEntry;
use blocker_screen::data::fresh::NewWorld;
use nightshade::ecs::system_param::EventReader;
use nightshade::prelude::*;

pub fn forget_the_chat(fresh: EventReader<NewWorld>, mut entry: ResMut<ChatEntry>) {
    if fresh.into_iter().next().is_none() {
        return;
    }
    *entry = ChatEntry::default();
}
