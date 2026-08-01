use super::*;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChildOf(pub Entity);

pub struct HierarchyIndex {
    pub children: HashMap<Entity, Vec<Entity>>,
    pub parent_of: HashMap<Entity, Entity>,
    pub structural_cursor: u64,
    pub tick_cursor: u32,
}

impl Default for HierarchyIndex {
    fn default() -> Self {
        Self {
            children: HashMap::new(),
            parent_of: HashMap::new(),
            structural_cursor: 0,
            tick_cursor: u32::MAX,
        }
    }
}

fn insert_ordered(siblings: &mut Vec<Entity>, entity: Entity) {
    let position = siblings.partition_point(|sibling| sibling.id < entity.id);
    siblings.insert(position, entity);
}

impl HierarchyIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sync(&mut self, world: &mut Storage) {
        let child_mask = world
            .lookup_key::<ChildOf>()
            .map(|key| key.mask)
            .unwrap_or(Mask::EMPTY);

        if Self::can_track_incrementally(world) {
            self.sync_incrementally(world, child_mask);
        } else {
            self.rebuild_from_scan(world, child_mask);
        }
        world.increment_tick();
    }

    fn sync_incrementally(&mut self, world: &mut Storage, child_mask: Mask) {
        let unlinks: Vec<Entity> = world
            .structural_changes_since(self.structural_cursor)
            .iter()
            .filter(|change| match change.kind {
                StructuralChangeKind::Despawned => true,
                StructuralChangeKind::ComponentsRemoved => change.mask.intersects(child_mask),
                _ => false,
            })
            .map(|change| change.entity)
            .collect();
        for entity in unlinks {
            self.unlink(entity);
        }

        if !child_mask.is_empty() {
            let relinks: Vec<Entity> = world
                .query_entities_changed_since_raw(child_mask, self.tick_cursor)
                .collect();
            for entity in relinks {
                if let Some(child_of) = world.get::<ChildOf>(entity) {
                    self.relink(entity, child_of.0);
                }
            }
        }

        self.structural_cursor = world.structural_sequence();
        self.tick_cursor = world.current_tick();
    }

    fn can_track_incrementally(world: &Storage) -> bool {
        world.structural_logging && world.change_detection()
    }

    fn rebuild_from_scan(&mut self, world: &mut Storage, child_mask: Mask) {
        self.children.clear();
        self.parent_of.clear();
        if child_mask.is_empty() {
            return;
        }
        let holders: Vec<Entity> = world.query_entities_raw(child_mask).collect();
        for entity in holders {
            if let Some(child_of) = world.get::<ChildOf>(entity) {
                let parent = child_of.0;
                self.parent_of.insert(entity, parent);
                insert_ordered(self.children.entry(parent).or_default(), entity);
            }
        }
    }

    fn unlink(&mut self, entity: Entity) {
        if let Some(parent) = self.parent_of.remove(&entity)
            && let Some(siblings) = self.children.get_mut(&parent)
        {
            siblings.retain(|&child| child != entity);
            if siblings.is_empty() {
                self.children.remove(&parent);
            }
        }
    }

    fn relink(&mut self, entity: Entity, parent: Entity) {
        if self.parent_of.get(&entity) == Some(&parent) {
            return;
        }
        self.unlink(entity);
        self.parent_of.insert(entity, parent);
        insert_ordered(self.children.entry(parent).or_default(), entity);
    }

    pub fn children(&self, parent: Entity) -> &[Entity] {
        self.children.get(&parent).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn descendants(&self, root: Entity) -> Vec<Entity> {
        let mut pending = vec![root];
        let mut visited: Vec<Entity> = Vec::new();
        while let Some(parent) = pending.pop() {
            for &child in self.children(parent) {
                if child != root && !visited.contains(&child) {
                    visited.push(child);
                    pending.push(child);
                }
            }
        }
        visited
    }

    pub fn despawn_recursive(&mut self, world: &mut Storage, root: Entity) -> Vec<Entity> {
        let mut targets = self.descendants(root);
        targets.insert(0, root);
        for &entity in &targets {
            self.unlink(entity);
            self.children.remove(&entity);
        }
        world.despawn_entities(&targets)
    }
}
