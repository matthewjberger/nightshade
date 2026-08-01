use crate::resources::Save;
use blocker_screen::data::fresh::NewWorld;
use nightshade::ecs::system_param::EventReader;
use nightshade::prelude::*;

pub fn forget_the_save(fresh: EventReader<NewWorld>, mut save: ResMut<Save>) {
    if fresh.into_iter().next().is_none() {
        return;
    }
    save.found = None;
    save.autosave.ask();
}
