use crate::Entity;

const SPARSE_TAG_ABSENT: u32 = u32::MAX;

#[derive(Default, Clone, Debug)]
pub struct SparseTagSet {
    pub dense: Vec<Entity>,
    pub sparse: Vec<u32>,
}

impl SparseTagSet {
    #[inline]
    pub fn insert(&mut self, entity: Entity) -> bool {
        let index = entity.id as usize;
        if index >= self.sparse.len() {
            self.sparse.resize(index + 1, SPARSE_TAG_ABSENT);
        }
        let slot = self.sparse[index];
        if slot != SPARSE_TAG_ABSENT {
            let existing = &mut self.dense[slot as usize];
            if *existing == entity {
                return false;
            }
            *existing = entity;
            return true;
        }
        self.sparse[index] = self.dense.len() as u32;
        self.dense.push(entity);
        true
    }

    #[inline]
    pub fn remove(&mut self, entity: Entity) -> bool {
        let index = entity.id as usize;
        let Some(&slot) = self.sparse.get(index) else {
            return false;
        };
        if slot == SPARSE_TAG_ABSENT || self.dense[slot as usize] != entity {
            return false;
        }
        self.dense.swap_remove(slot as usize);
        self.sparse[index] = SPARSE_TAG_ABSENT;
        if (slot as usize) < self.dense.len() {
            let moved = self.dense[slot as usize];
            self.sparse[moved.id as usize] = slot;
        }
        true
    }

    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        self.sparse
            .get(entity.id as usize)
            .is_some_and(|&slot| slot != SPARSE_TAG_ABSENT && self.dense[slot as usize] == entity)
    }

    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.dense.iter().copied()
    }

    pub fn len(&self) -> usize {
        self.dense.len()
    }

    pub fn is_empty(&self) -> bool {
        self.dense.is_empty()
    }

    pub fn clear(&mut self) {
        self.dense.clear();
        self.sparse.fill(SPARSE_TAG_ABSENT);
    }
}
