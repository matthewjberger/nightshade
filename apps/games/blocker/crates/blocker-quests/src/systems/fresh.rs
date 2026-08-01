use crate::resources::{Journal, Waypoint};
use blocker_screen::data::fresh::NewWorld;
use nightshade::ecs::system_param::EventReader;
use nightshade::prelude::*;

pub fn clear_the_journal(
    fresh: EventReader<NewWorld>,
    mut journal: ResMut<Journal>,
    mut waypoint: ResMut<Waypoint>,
) {
    if fresh.into_iter().next().is_none() {
        return;
    }
    *journal = Journal::default();
    *waypoint = Waypoint::default();
}
