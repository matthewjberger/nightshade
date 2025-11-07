pub use crate::ecs::transform::commands::*;
pub use crate::ecs::transform::queries::*;

use crate::ecs::world::{
    GLOBAL_TRANSFORM, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY, World,
    components::{GlobalTransform, Parent},
};

pub fn update_global_transforms_system(world: &mut World) {
    let entities_to_update: Vec<freecs::Entity> = world
        .query_entities(LOCAL_TRANSFORM_DIRTY | LOCAL_TRANSFORM | GLOBAL_TRANSFORM)
        .collect();

    for entity in entities_to_update {
        let global_transform = global_transform_of(world, entity);
        world.set_global_transform(entity, global_transform);
        world.remove_local_transform_dirty(entity);
    }
}

fn global_transform_of(world: &World, entity: freecs::Entity) -> GlobalTransform {
    let Some(local_transform) = world.get_local_transform(entity) else {
        return GlobalTransform::default();
    };

    let local_matrix = local_transform.as_matrix();

    match world.get_parent(entity) {
        Some(Parent(Some(parent_entity))) => {
            let parent_global = global_transform_of(world, *parent_entity);
            GlobalTransform(parent_global.0 * local_matrix)
        }
        _ => GlobalTransform(local_matrix),
    }
}
