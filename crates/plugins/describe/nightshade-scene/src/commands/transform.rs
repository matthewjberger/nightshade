use nightshade_ecs::Entity;

use crate::contract::{ChildOf, GlobalTransform};
use crate::queries::computed_global_transform;

pub fn update_parent(
    storage: &mut nightshade_ecs::Storage,
    child: nightshade_ecs::Entity,
    new_parent: Option<Entity>,
) {
    match new_parent {
        Some(parent) => storage.set(child, ChildOf(parent)),
        None => {
            storage.remove_components::<(nightshade_ecs::ChildOf,)>(child);
        }
    }
}

pub fn propagate_subtree(
    hierarchy: &mut nightshade_ecs::HierarchyIndex,
    storage: &mut nightshade_ecs::Storage,
    root: Entity,
) {
    hierarchy.sync(storage);
    let mut stack = vec![root];
    while let Some(current) = stack.pop() {
        let new_global_transform = computed_global_transform(storage, current);
        if let Some(global_transform) = storage.get_mut::<crate::contract::GlobalTransform>(current)
        {
            *global_transform = GlobalTransform(new_global_transform);
        }
        for child in hierarchy.children(current) {
            stack.push(*child);
        }
    }
}

pub fn setup_entity_transforms(
    commands: &mut nightshade_ecs::system_param::Commands<'_>,
    entity: Entity,
    local_transform: crate::contract::LocalTransform,
) {
    commands.set(entity, local_transform);
    commands.set(entity, crate::contract::GlobalTransform::default());
    commands.queue(move |storage, resources| {
        propagate_subtree(
            resources.res_mut::<nightshade_ecs::HierarchyIndex>(),
            storage,
            entity,
        )
    });
}
