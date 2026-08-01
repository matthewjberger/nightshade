use crate::components::Mark;
use crate::resources::Crew;
use blocker_figure::components::{Home, Plate};
use blocker_screen::data::fresh::NewWorld;
use nightshade::ecs::system_param::EventReader;
use nightshade::ecs::{Entity, HierarchyIndex};
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::despawn_entities_with_cache_cleanup;
use std::collections::HashSet;

pub fn clear_the_crew(
    fresh: EventReader<NewWorld>,
    mut crew: ResMut<Crew>,
    mut commands: Commands,
    folk: Query<(Option<&Plate>, Option<&Mark>), With<Home>>,
) {
    if fresh.into_iter().next().is_none() {
        return;
    }
    *crew = Crew::default();
    let mut gone: Vec<Entity> = Vec::new();
    folk.for_each(|entity, (plate, mark)| {
        gone.push(entity);
        gone.extend(plate.map(|plate| plate.0));
        gone.extend(mark.map(|mark| mark.0));
    });
    if gone.is_empty() {
        return;
    }
    commands.queue(move |storage, resources| {
        let hierarchy = resources.res::<HierarchyIndex>();
        let mut whole: HashSet<Entity> = gone.iter().copied().collect();
        for root in &gone {
            whole.extend(hierarchy.descendants(*root));
        }
        let whole: Vec<Entity> = whole.into_iter().collect();
        despawn_entities_with_cache_cleanup(storage, resources, &whole);
    });
}
