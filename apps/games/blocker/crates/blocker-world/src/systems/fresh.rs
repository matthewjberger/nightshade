use crate::resources::{Arrival, Digging, Effects, Generation};
use blocker_screen::data::fresh::NewWorld;
use nightshade::ecs::system_param::EventReader;
use nightshade::prelude::*;
use nightshade_voxels::prelude::reset_the_world;

pub fn clear_the_ground(
    fresh: EventReader<NewWorld>,
    mut generation: ResMut<Generation>,
    mut effects: ResMut<Effects>,
    mut arrival: ResMut<Arrival>,
    mut commands: Commands,
) {
    if fresh.into_iter().next().is_none() {
        return;
    }
    commands.queue(|storage, resources| {
        reset_the_world::<Digging>(storage, resources);
    });
    *generation = Generation::default();
    *effects = Effects::default();
    *arrival = Arrival::default();
}
