use crate::resources::{Counting, Den};
use blocker_screen::data::fresh::NewWorld;
use nightshade::ecs::system_param::EventReader;
use nightshade::prelude::*;

pub fn clear_the_den(
    fresh: EventReader<NewWorld>,
    mut den: ResMut<Den>,
    mut counting: ResMut<Counting>,
) {
    if fresh.into_iter().next().is_none() {
        return;
    }
    *den = Den::default();
    *counting = Counting::default();
}
