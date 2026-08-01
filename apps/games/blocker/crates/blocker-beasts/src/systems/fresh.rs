use crate::components::Beast;
use crate::resources::Quarry;
use blocker_figure::components::Plate;
use blocker_screen::data::fresh::NewWorld;
use nightshade::ecs::system_param::EventReader;
use nightshade::prelude::*;

pub fn clear_the_beasts(
    fresh: EventReader<NewWorld>,
    mut quarry: ResMut<Quarry>,
    mut commands: Commands,
    beasts: Query<(Option<&Plate>,), With<Beast>>,
) {
    if fresh.into_iter().next().is_none() {
        return;
    }
    *quarry = Quarry::default();
    beasts.for_each(|entity, (plate,)| {
        commands.despawn_recursive(entity);
        if let Some(plate) = plate {
            commands.despawn(plate.0);
        }
    });
}
