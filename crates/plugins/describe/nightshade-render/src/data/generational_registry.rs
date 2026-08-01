use std::collections::HashMap;

pub struct GenerationalRegistry<T> {
    pub entries: Vec<Option<T>>,
    pub generations: Vec<u32>,
    pub reference_counts: Vec<usize>,
    pub free_indices: Vec<u32>,
    pub name_to_index: HashMap<String, u32>,
    pub index_to_name: Vec<Option<String>>,
    pub name_hashes: Vec<u64>,
    pub observed: Vec<bool>,
    pub protected: Vec<bool>,
}

pub fn registry_name_hash(name: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    name.hash(&mut hasher);
    hasher.finish()
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
            name_hashes: self.name_hashes.clone(),
            observed: self.observed.clone(),
            protected: self.protected.clone(),
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
            name_hashes: Vec::new(),
            observed: Vec::new(),
            protected: Vec::new(),
        }
    }
}

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

    let hash = registry_name_hash(&name);
    let index = if let Some(free_index) = registry.free_indices.pop() {
        registry.generations[free_index as usize] += 1;
        registry.entries[free_index as usize] = Some(value);
        registry.reference_counts[free_index as usize] = 0;
        registry.index_to_name[free_index as usize] = Some(name.clone());
        registry.name_hashes[free_index as usize] = hash;
        registry.observed[free_index as usize] = false;
        registry.protected[free_index as usize] = false;
        free_index
    } else {
        let index = registry.entries.len() as u32;
        registry.entries.push(Some(value));
        registry.generations.push(0);
        registry.reference_counts.push(0);
        registry.index_to_name.push(Some(name.clone()));
        registry.name_hashes.push(hash);
        registry.observed.push(false);
        registry.protected.push(false);
        index
    };

    registry.name_to_index.insert(name, index);
    (index, registry.generations[index as usize])
}

pub fn registry_reserve_name<T>(
    registry: &mut GenerationalRegistry<T>,
    name: String,
) -> (u32, u32) {
    if let Some(&existing_index) = registry.name_to_index.get(&name) {
        let generation = registry.generations[existing_index as usize];
        return (existing_index, generation);
    }

    let hash = registry_name_hash(&name);
    let index = if let Some(free_index) = registry.free_indices.pop() {
        registry.generations[free_index as usize] += 1;
        registry.entries[free_index as usize] = None;
        registry.reference_counts[free_index as usize] = 0;
        registry.index_to_name[free_index as usize] = Some(name.clone());
        registry.name_hashes[free_index as usize] = hash;
        registry.observed[free_index as usize] = false;
        registry.protected[free_index as usize] = false;
        free_index
    } else {
        let index = registry.entries.len() as u32;
        registry.entries.push(None);
        registry.generations.push(0);
        registry.reference_counts.push(0);
        registry.index_to_name.push(Some(name.clone()));
        registry.name_hashes.push(hash);
        registry.observed.push(false);
        registry.protected.push(false);
        index
    };

    registry.name_to_index.insert(name, index);
    (index, registry.generations[index as usize])
}

pub fn registry_name_for<T>(registry: &GenerationalRegistry<T>, index: u32) -> Option<&str> {
    registry
        .index_to_name
        .get(index as usize)
        .and_then(|slot| slot.as_deref())
}

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

pub fn registry_lookup_index<T>(
    registry: &GenerationalRegistry<T>,
    name: &str,
) -> Option<(u32, u32)> {
    registry.name_to_index.get(name).map(|&index| {
        let generation = registry.generations[index as usize];
        (index, generation)
    })
}

pub fn registry_add_reference<T>(registry: &mut GenerationalRegistry<T>, index: u32) {
    if let Some(count) = registry.reference_counts.get_mut(index as usize) {
        *count += 1;
    }
}

pub fn registry_remove_reference<T>(registry: &mut GenerationalRegistry<T>, index: u32) {
    if let Some(count) = registry.reference_counts.get_mut(index as usize) {
        *count = count.saturating_sub(1);
    }
}

pub fn registry_set_protected<T>(
    registry: &mut GenerationalRegistry<T>,
    index: u32,
    protected: bool,
) {
    if let Some(slot) = registry.protected.get_mut(index as usize) {
        *slot = protected;
    }
}

pub fn registry_protect_name<T>(registry: &mut GenerationalRegistry<T>, name: &str) {
    if let Some(&index) = registry.name_to_index.get(name) {
        registry_set_protected(registry, index, true);
    }
}

pub fn registry_release_name<T>(registry: &mut GenerationalRegistry<T>, name: &str) {
    if let Some(&index) = registry.name_to_index.get(name) {
        registry_set_protected(registry, index, false);
    }
}

pub fn registry_is_protected<T>(registry: &GenerationalRegistry<T>, index: u32) -> bool {
    registry
        .protected
        .get(index as usize)
        .copied()
        .unwrap_or(false)
}

pub fn registry_observe_indices<T>(
    registry: &mut GenerationalRegistry<T>,
    live: &std::collections::HashSet<u32>,
) {
    for &index in live {
        if let Some(observed) = registry.observed.get_mut(index as usize) {
            *observed = true;
        }
    }
}

pub fn registry_observe<T>(
    registry: &mut GenerationalRegistry<T>,
    live: &std::collections::HashSet<u64>,
) {
    for index in 0..registry.entries.len() {
        if registry.index_to_name[index].is_some() && live.contains(&registry.name_hashes[index]) {
            registry.observed[index] = true;
        }
    }
}

pub fn registry_reclaim_range<T>(
    registry: &mut GenerationalRegistry<T>,
    live: &std::collections::HashSet<u64>,
    cursor: usize,
    budget: usize,
) -> (Vec<String>, usize) {
    let len = registry.entries.len();
    if len == 0 || budget == 0 {
        return (Vec::new(), 0);
    }
    let mut removed = Vec::new();
    let mut index = if cursor >= len { 0 } else { cursor };
    for _ in 0..budget.min(len) {
        let in_use =
            registry.index_to_name[index].is_some() && live.contains(&registry.name_hashes[index]);
        if !in_use
            && !registry.protected[index]
            && registry.observed[index]
            && registry.entries[index].is_some()
            && let Some(name) = registry.index_to_name[index].take()
        {
            registry.name_to_index.remove(&name);
            removed.push(name);
            registry.entries[index] = None;
            registry.reference_counts[index] = 0;
            registry.observed[index] = false;
            registry.free_indices.push(index as u32);
        }
        index += 1;
        if index >= len {
            index = 0;
        }
    }
    (removed, index)
}
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

pub fn registry_entry_by_name<'a, T>(
    registry: &'a GenerationalRegistry<T>,
    name: &str,
) -> Option<&'a T> {
    registry
        .name_to_index
        .get(name)
        .and_then(|&index| registry.entries[index as usize].as_ref())
}
