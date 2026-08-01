use crate::components::Block;
use blocker_blocks::data::crops::CropKind;
use blocker_screen::data::fresh::NewWorld;
use nightshade::ecs::system_param::EventReader;
use nightshade::prelude::*;

pub fn clear_the_fields(
    fresh: EventReader<NewWorld>,
    mut commands: Commands,
    crops: Query<(&Block,), With<CropKind>>,
) {
    if fresh.into_iter().next().is_none() {
        return;
    }
    crops.for_each(|entity, _| commands.despawn(entity));
}
