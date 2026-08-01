use crate::resources::{Falling, Season, Sky};
use blocker_screen::data::fresh::NewWorld;
use nightshade::ecs::system_param::EventReader;
use nightshade::prelude::*;

pub fn clear_the_sky(
    fresh: EventReader<NewWorld>,
    mut sky: ResMut<Sky>,
    mut season: ResMut<Season>,
    mut falling: ResMut<Falling>,
    mut commands: Commands,
) {
    if fresh.into_iter().next().is_none() {
        return;
    }
    for emitter in falling
        .rain
        .iter()
        .chain(&falling.snow)
        .chain(&falling.splash)
    {
        commands.despawn(*emitter);
    }
    *falling = Falling::default();
    *sky = Sky::default();
    *season = Season::default();
}
