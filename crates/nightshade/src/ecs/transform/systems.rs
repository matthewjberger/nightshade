pub use crate::ecs::transform::commands::*;
pub use crate::ecs::transform::queries::*;

use crate::ecs::world::{
    GLOBAL_TRANSFORM, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY, PARENT, World,
    components::{GlobalTransform, Parent},
};

pub fn run_systems(world: &mut World) {
    rebuild_children_cache_if_needed(world);
    update_global_transforms_system(world);
}

fn rebuild_children_cache_if_needed(world: &mut World) {
    let needs_rebuild = if world.resources.children_cache.is_empty() {
        world.query_entities(PARENT).count() > 0
    } else {
        false
    };

    if needs_rebuild {
        world.validate_and_rebuild_children_cache();
    }
}

impl World {
    pub fn assign_local_transform(
        &mut self,
        entity: freecs::Entity,
        local_transform: crate::ecs::transform::components::LocalTransform,
    ) {
        self.set_local_transform(entity, local_transform);
        mark_local_transform_dirty(self, entity);
    }

    pub fn mark_local_transform_dirty(&mut self, entity: freecs::Entity) {
        mark_local_transform_dirty(self, entity);
    }

    pub fn update_parent(&mut self, child: freecs::Entity, new_parent: Option<Parent>) {
        let old_parent_entity = self.get_parent(child).and_then(|p| p.0);

        if let Some(old_parent_entity) = old_parent_entity
            && let Some(children) = self.resources.children_cache.get_mut(&old_parent_entity)
        {
            children.retain(|&e| e != child);
        }

        if let Some(new_parent) = new_parent {
            self.set_parent(child, new_parent);
            if let Some(parent_entity) = new_parent.0 {
                self.resources
                    .children_cache
                    .entry(parent_entity)
                    .or_default()
                    .push(child);
            }
        } else {
            self.remove_parent(child);
        }

        self.mark_local_transform_dirty(child);
    }

    pub fn validate_and_rebuild_children_cache(&mut self) {
        self.resources.children_cache.clear();

        let all_entities_with_parent: Vec<freecs::Entity> = self.query_entities(PARENT).collect();
        for child in all_entities_with_parent {
            if let Some(parent) = self.get_parent(child)
                && let Some(parent_entity) = parent.0
            {
                self.resources
                    .children_cache
                    .entry(parent_entity)
                    .or_default()
                    .push(child);
            }
        }
    }
}

pub fn update_global_transforms_system(world: &mut World) {
    let dirty_entities: Vec<freecs::Entity> = world
        .query_entities(LOCAL_TRANSFORM_DIRTY | LOCAL_TRANSFORM | GLOBAL_TRANSFORM)
        .collect();

    for entity in dirty_entities {
        let new_global_transform = global_transform_of(entity, world);
        if let Some(global_transform) = world.get_global_transform_mut(entity) {
            *global_transform = GlobalTransform(new_global_transform);
        }
        world.remove_local_transform_dirty(entity);
    }
}

pub fn global_transform_of(entity: freecs::Entity, world: &World) -> nalgebra_glm::Mat4 {
    let Some(local_transform) = world.get_local_transform(entity) else {
        return nalgebra_glm::Mat4::identity();
    };
    if let Some(parent) = world.get_parent(entity)
        && let Some(parent_entity) = parent.0
    {
        global_transform_of(parent_entity, world) * local_transform.as_matrix()
    } else {
        local_transform.as_matrix()
    }
}
