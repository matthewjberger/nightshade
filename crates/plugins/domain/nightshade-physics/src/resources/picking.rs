use rapier3d::prelude::*;
use std::collections::HashMap;

pub struct PickingWorld {
    pub(crate) collider_set: ColliderSet,
    pub(crate) collider_to_entity: HashMap<ColliderHandle, nightshade::ecs::Entity>,
    pub(crate) entity_to_collider: HashMap<nightshade::ecs::Entity, ColliderHandle>,
}

impl Default for PickingWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl PickingWorld {
    pub fn new() -> Self {
        Self {
            collider_set: ColliderSet::new(),
            collider_to_entity: HashMap::new(),
            entity_to_collider: HashMap::new(),
        }
    }

    pub(crate) fn add_trimesh_collider(
        &mut self,
        entity: nightshade::ecs::Entity,
        vertices: Vec<Point<Real>>,
        indices: Vec<[u32; 3]>,
        position: Isometry<Real>,
    ) -> Option<ColliderHandle> {
        if let Some(existing_handle) = self.entity_to_collider.get(&entity) {
            self.collider_set.remove(
                *existing_handle,
                &mut IslandManager::new(),
                &mut RigidBodySet::new(),
                false,
            );
            self.collider_to_entity.remove(existing_handle);
            self.entity_to_collider.remove(&entity);
        }

        let shape = SharedShape::trimesh(vertices, indices);
        if shape.is_err() {
            return None;
        }
        let shape = shape.unwrap();

        let collider = ColliderBuilder::new(shape).position(position).build();

        let handle = self.collider_set.insert(collider);

        self.collider_to_entity.insert(handle, entity);
        self.entity_to_collider.insert(entity, handle);

        Some(handle)
    }

    pub fn remove_entity(&mut self, entity: nightshade::ecs::Entity) {
        if let Some(handle) = self.entity_to_collider.remove(&entity) {
            self.collider_set.remove(
                handle,
                &mut IslandManager::new(),
                &mut RigidBodySet::new(),
                false,
            );
            self.collider_to_entity.remove(&handle);
        }
    }

    pub(crate) fn update_collider_position(
        &mut self,
        entity: nightshade::ecs::Entity,
        position: Isometry<Real>,
    ) {
        if let Some(&handle) = self.entity_to_collider.get(&entity)
            && let Some(collider) = self.collider_set.get_mut(handle)
        {
            collider.set_position(position);
        }
    }

    pub(crate) fn get_entity(&self, handle: ColliderHandle) -> Option<nightshade::ecs::Entity> {
        self.collider_to_entity.get(&handle).copied()
    }
}
