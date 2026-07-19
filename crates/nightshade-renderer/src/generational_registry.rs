//! Name-keyed generational registry with reference counting.

use std::collections::HashMap;

/// Slot-based store of `T` values keyed by name, tracking generations and reference counts.
pub struct GenerationalRegistry<T> {
    /// Value slots, `None` when the slot is empty or reserved.
    pub entries: Vec<Option<T>>,
    /// Generation counter per slot, bumped when a freed slot is reused.
    pub generations: Vec<u32>,
    /// Live reference count per slot.
    pub reference_counts: Vec<usize>,
    /// Slots available for reuse.
    pub free_indices: Vec<u32>,
    /// Maps each registered name to its slot index.
    pub name_to_index: HashMap<String, u32>,
    /// Name held by each slot, `None` when unnamed.
    pub index_to_name: Vec<Option<String>>,
}

impl<T: Clone> Clone for GenerationalRegistry<T> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            generations: self.generations.clone(),
            reference_counts: self.reference_counts.clone(),
            free_indices: self.free_indices.clone(),
            name_to_index: self.name_to_index.clone(),
            index_to_name: self.index_to_name.clone(),
        }
    }
}

impl<T> Default for GenerationalRegistry<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            generations: Vec::new(),
            reference_counts: Vec::new(),
            free_indices: Vec::new(),
            name_to_index: HashMap::new(),
            index_to_name: Vec::new(),
        }
    }
}

/// Inserts `value` under `name`, reusing the existing slot when the name is already registered, and returns its index and generation.
pub fn registry_insert<T>(
    registry: &mut GenerationalRegistry<T>,
    name: String,
    value: T,
) -> (u32, u32) {
    if let Some(&existing_index) = registry.name_to_index.get(&name) {
        let generation = registry.generations[existing_index as usize];
        registry.entries[existing_index as usize] = Some(value);
        return (existing_index, generation);
    }

    let index = if let Some(free_index) = registry.free_indices.pop() {
        registry.generations[free_index as usize] += 1;
        registry.entries[free_index as usize] = Some(value);
        registry.reference_counts[free_index as usize] = 0;
        registry.index_to_name[free_index as usize] = Some(name.clone());
        free_index
    } else {
        let index = registry.entries.len() as u32;
        registry.entries.push(Some(value));
        registry.generations.push(0);
        registry.reference_counts.push(0);
        registry.index_to_name.push(Some(name.clone()));
        index
    };

    registry.name_to_index.insert(name, index);
    (index, registry.generations[index as usize])
}

/// Reserves a slot for `name` without a value, returning its index and generation.
pub fn registry_reserve_name<T>(
    registry: &mut GenerationalRegistry<T>,
    name: String,
) -> (u32, u32) {
    if let Some(&existing_index) = registry.name_to_index.get(&name) {
        let generation = registry.generations[existing_index as usize];
        return (existing_index, generation);
    }

    let index = if let Some(free_index) = registry.free_indices.pop() {
        registry.generations[free_index as usize] += 1;
        registry.entries[free_index as usize] = None;
        registry.reference_counts[free_index as usize] = 0;
        registry.index_to_name[free_index as usize] = Some(name.clone());
        free_index
    } else {
        let index = registry.entries.len() as u32;
        registry.entries.push(None);
        registry.generations.push(0);
        registry.reference_counts.push(0);
        registry.index_to_name.push(Some(name.clone()));
        index
    };

    registry.name_to_index.insert(name, index);
    (index, registry.generations[index as usize])
}

/// Returns `true` when the slot at `index` holds a value.
pub fn registry_is_filled<T>(registry: &GenerationalRegistry<T>, index: u32) -> bool {
    registry
        .entries
        .get(index as usize)
        .map(|slot| slot.is_some())
        .unwrap_or(false)
}

/// Returns the name registered for `index`, if any.
pub fn registry_name_for<T>(registry: &GenerationalRegistry<T>, index: u32) -> Option<&str> {
    registry
        .index_to_name
        .get(index as usize)
        .and_then(|slot| slot.as_deref())
}

/// Removes and returns the value at `index` when `generation` matches, freeing the slot.
pub fn registry_remove<T>(
    registry: &mut GenerationalRegistry<T>,
    index: u32,
    generation: u32,
) -> Option<T> {
    if !registry_is_valid(registry, index, generation) {
        return None;
    }

    let slot = index as usize;
    let value = registry.entries[slot].take();

    if let Some(name) = registry.index_to_name[slot].take() {
        registry.name_to_index.remove(&name);
    }

    registry.reference_counts[slot] = 0;
    registry.free_indices.push(index);

    value
}

/// Returns `true` when `index` and `generation` name a slot that currently holds a value.
pub fn registry_is_valid<T>(
    registry: &GenerationalRegistry<T>,
    index: u32,
    generation: u32,
) -> bool {
    let slot = index as usize;
    slot < registry.generations.len()
        && registry.generations[slot] == generation
        && registry.entries[slot].is_some()
}

/// Returns the index and generation registered for `name`, if any.
pub fn registry_lookup_index<T>(
    registry: &GenerationalRegistry<T>,
    name: &str,
) -> Option<(u32, u32)> {
    registry.name_to_index.get(name).map(|&index| {
        let generation = registry.generations[index as usize];
        (index, generation)
    })
}

/// Increments the reference count for `index`.
pub fn registry_add_reference<T>(registry: &mut GenerationalRegistry<T>, index: u32) {
    if let Some(count) = registry.reference_counts.get_mut(index as usize) {
        *count += 1;
    }
}

/// Decrements the reference count for `index`, saturating at zero.
pub fn registry_remove_reference<T>(registry: &mut GenerationalRegistry<T>, index: u32) {
    if let Some(count) = registry.reference_counts.get_mut(index as usize) {
        *count = count.saturating_sub(1);
    }
}

/// Returns the reference count for `index`, or zero when out of range.
pub fn registry_reference_count<T>(registry: &GenerationalRegistry<T>, index: u32) -> usize {
    registry
        .reference_counts
        .get(index as usize)
        .copied()
        .unwrap_or(0)
}

/// Clears every filled slot with a zero reference count and returns the removed names.
pub fn registry_remove_unused<T>(registry: &mut GenerationalRegistry<T>) -> Vec<String> {
    let mut removed = Vec::new();

    for index in 0..registry.entries.len() {
        if registry.reference_counts[index] == 0 && registry.entries[index].is_some() {
            if let Some(name) = registry.index_to_name[index].take() {
                registry.name_to_index.remove(&name);
                removed.push(name);
            }
            registry.entries[index] = None;
            registry.free_indices.push(index as u32);
        }
    }

    removed
}

/// Returns the number of filled slots.
pub fn registry_len<T>(registry: &GenerationalRegistry<T>) -> usize {
    registry
        .entries
        .iter()
        .filter(|entry| entry.is_some())
        .count()
}

/// Returns `true` when no slot holds a value.
pub fn registry_is_empty<T>(registry: &GenerationalRegistry<T>) -> bool {
    registry_len(registry) == 0
}

/// Empties the registry, dropping all slots, names, and free-list state.
pub fn registry_clear<T>(registry: &mut GenerationalRegistry<T>) {
    registry.entries.clear();
    registry.generations.clear();
    registry.reference_counts.clear();
    registry.free_indices.clear();
    registry.name_to_index.clear();
    registry.index_to_name.clear();
}

/// Returns the value at `index` when `index` and `generation` are valid.
pub fn registry_entry<T>(
    registry: &GenerationalRegistry<T>,
    index: u32,
    generation: u32,
) -> Option<&T> {
    if registry_is_valid(registry, index, generation) {
        registry.entries[index as usize].as_ref()
    } else {
        None
    }
}

/// Returns the value registered under `name`, if any.
pub fn registry_entry_by_name<'a, T>(
    registry: &'a GenerationalRegistry<T>,
    name: &str,
) -> Option<&'a T> {
    registry
        .name_to_index
        .get(name)
        .and_then(|&index| registry.entries[index as usize].as_ref())
}

/// Returns a mutable reference to the value registered under `name`, if any.
pub fn registry_entry_by_name_mut<'a, T>(
    registry: &'a mut GenerationalRegistry<T>,
    name: &str,
) -> Option<&'a mut T> {
    registry
        .name_to_index
        .get(name)
        .copied()
        .and_then(|index| registry.entries[index as usize].as_mut())
}
