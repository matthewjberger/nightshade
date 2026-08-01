#[derive(Default, Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Entity {
    pub id: u32,
    pub generation: u32,
}

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { id, generation } = self;
        write!(f, "Id: {id} - Generation: {generation}")
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EntitySlot {
    pub generation: u32,
    pub alive: bool,
}

#[derive(Clone, Default, Debug)]
pub struct IdCursor(std::sync::Arc<std::sync::atomic::AtomicU32>);

impl IdCursor {
    pub fn next(&self) -> u32 {
        self.take(1)
    }

    pub fn take(&self, count: u32) -> u32 {
        self.0
            .fetch_add(count, std::sync::atomic::Ordering::Relaxed)
    }

    pub fn peek(&self) -> u32 {
        self.0.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn raise_past(&self, id: u32) {
        self.0
            .fetch_max(id + 1, std::sync::atomic::Ordering::Relaxed);
    }
}

impl PartialEq for IdCursor {
    fn eq(&self, other: &Self) -> bool {
        self.peek() == other.peek()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for IdCursor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(self.peek())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for IdCursor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <u32 as serde::Deserialize>::deserialize(deserializer)?;
        let cursor = IdCursor::default();
        cursor.0.store(value, std::sync::atomic::Ordering::Relaxed);
        Ok(cursor)
    }
}

#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EntityAllocator {
    pub next_id: IdCursor,
    pub free_ids: Vec<(u32, u32)>,
    pub slots: Vec<EntitySlot>,
}

impl EntityAllocator {
    pub fn revive(&mut self, entity: Entity) {
        let index = entity.id as usize;
        if self.slots.len() <= index {
            self.slots.resize(
                index + 1,
                EntitySlot {
                    generation: 0,
                    alive: false,
                },
            );
        }
        self.slots[index] = EntitySlot {
            generation: entity.generation,
            alive: true,
        };
        self.free_ids.retain(|&(id, _)| id != entity.id);
        self.next_id.raise_past(entity.id);
    }

    #[inline]
    pub fn allocate(&mut self) -> Entity {
        let entity = if let Some((id, next_generation)) = self.free_ids.pop() {
            Entity {
                id,
                generation: next_generation,
            }
        } else {
            Entity {
                id: self.next_id.next(),
                generation: 0,
            }
        };
        let index = entity.id as usize;
        if index >= self.slots.len() {
            self.slots.resize(index + 1, EntitySlot::default());
        }
        self.slots[index] = EntitySlot {
            generation: entity.generation,
            alive: true,
        };
        entity
    }

    pub fn allocate_batch(&mut self, count: usize, entities: &mut Vec<Entity>) {
        entities.reserve(count);
        let recycled = count.min(self.free_ids.len());
        for _ in 0..recycled {
            let Some((id, generation)) = self.free_ids.pop() else {
                break;
            };
            let index = id as usize;
            if index >= self.slots.len() {
                self.slots.resize(index + 1, EntitySlot::default());
            }
            self.slots[index] = EntitySlot {
                generation,
                alive: true,
            };
            entities.push(Entity { id, generation });
        }

        let fresh = count - recycled;
        if fresh > 0 {
            let start = self.next_id.take(fresh as u32);
            let start_index = start as usize;
            let end_index = start_index + fresh;
            if end_index > self.slots.len() {
                self.slots.resize(end_index, EntitySlot::default());
            }
            self.slots[start_index..end_index].fill(EntitySlot {
                generation: 0,
                alive: true,
            });
            for id in start..start + fresh as u32 {
                entities.push(Entity { id, generation: 0 });
            }
        }
    }

    #[inline]
    pub fn deallocate(&mut self, entity: Entity) -> bool {
        if !self.is_alive(entity) {
            return false;
        }
        self.slots[entity.id as usize].alive = false;
        self.free_ids
            .push((entity.id, entity.generation.wrapping_add(1)));
        true
    }

    #[inline]
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.slots
            .get(entity.id as usize)
            .is_some_and(|slot| slot.alive && slot.generation == entity.generation)
    }
}

#[derive(Copy, Clone, Default)]
pub(crate) struct EntityLocation {
    pub generation: u32,
    pub table_index: u32,
    pub array_index: u32,
    pub allocated: bool,
}

#[derive(Default)]
pub(crate) struct EntityLocations {
    pub locations: Vec<EntityLocation>,
}

impl EntityLocations {
    pub fn get(&self, id: u32) -> Option<&EntityLocation> {
        self.locations.get(id as usize)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut EntityLocation> {
        self.locations.get_mut(id as usize)
    }

    pub fn ensure_slot(&mut self, id: u32, generation: u32) {
        let id_usize = id as usize;
        if id_usize >= self.locations.len() {
            self.locations.resize(
                (self.locations.len() * 2).max(64).max(id_usize + 1),
                EntityLocation::default(),
            );
        }
        self.locations[id_usize].generation = generation;
    }

    pub fn insert(&mut self, id: u32, location: EntityLocation) {
        let id_usize = id as usize;
        if id_usize >= self.locations.len() {
            self.locations.resize(
                (self.locations.len() * 2).max(64).max(id_usize + 1),
                EntityLocation::default(),
            );
        }
        self.locations[id_usize] = location;
    }

    pub fn reserve_through(&mut self, highest_id: u32) {
        let needed = highest_id as usize + 1;
        if needed > self.locations.len() {
            self.locations.resize(
                needed.max(self.locations.len() * 2).max(64),
                EntityLocation::default(),
            );
        }
    }

    pub fn mark_deallocated(&mut self, id: u32) {
        if let Some(loc) = self.locations.get_mut(id as usize) {
            loc.allocated = false;
        }
    }
}
