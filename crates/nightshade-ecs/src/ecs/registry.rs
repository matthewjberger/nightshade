use super::*;

#[derive(Clone, Copy)]
pub struct ComponentInfo {
    pub type_id: TypeId,
    pub type_name: &'static str,
    pub(crate) mask: Mask,
    pub(crate) item_size: usize,
    pub new_column: fn() -> ErasedColumn,
    pub push_default: fn(&mut ErasedColumn, usize),
}

pub struct ComponentKey<T> {
    pub component_index: u32,
    pub(crate) mask: Mask,
    pub registry_id: u32,
    marker: PhantomData<fn() -> T>,
}

impl<T> Clone for ComponentKey<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ComponentKey<T> {}

impl<T> std::fmt::Debug for ComponentKey<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "ComponentKey({})", self.component_index)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TagKey {
    pub tag_index: u32,
    pub(crate) mask: Mask,
    pub registry_id: u32,
}

pub const MASK_BITS_LOW_WATER: u32 = 32;

#[derive(Clone)]
pub(crate) struct Components {
    pub registry_id: u32,
    pub components: Vec<ComponentInfo>,
    pub components_by_type: TypeIdMap<u32>,
    pub tag_count: u32,
    pub tags_by_type: TypeIdMap<u32>,
    low_water_reported: bool,
}

impl Default for Components {
    fn default() -> Self {
        Self {
            registry_id: NEXT_REGISTRY_ID.fetch_add(1, Ordering::Relaxed),
            components: Vec::new(),
            components_by_type: TypeIdMap::default(),
            tag_count: 0,
            tags_by_type: TypeIdMap::default(),
            low_water_reported: false,
        }
    }
}

impl Components {
    fn claim_bit(&mut self, kind: &'static str, name: &'static str) {
        let used = self.components.len() as u32 + self.tag_count;
        assert!(
            used < MASK_BITS,
            "{kind} {name} has no mask bit left: {} component types and {} tag types already fill all {MASK_BITS} bits of this world. Raise MASK_WORDS, or keep fewer types in one world.",
            self.components.len(),
            self.tag_count
        );
        let remaining = MASK_BITS - used - 1;
        if remaining < MASK_BITS_LOW_WATER && !self.low_water_reported {
            self.low_water_reported = true;
            tracing::warn!(
                target: "nightshade::ecs",
                "{kind} {name} takes a mask bit and leaves {remaining} of {MASK_BITS}. Components and tags take from the same {MASK_BITS}, and the world panics when a type finds no bit."
            );
        }
    }

    pub(crate) fn register<T: Send + Sync + Default + 'static>(&mut self) -> ComponentKey<T> {
        let type_id = TypeId::of::<T>();
        if let Some(&component_index) = self.components_by_type.get(&type_id) {
            return self.key_for(component_index);
        }

        self.claim_bit("component", std::any::type_name::<T>());
        let component_index = self.components.len() as u32;
        self.components.push(ComponentInfo {
            type_id,
            type_name: std::any::type_name::<T>(),
            mask: Mask::bit(component_index),
            item_size: std::mem::size_of::<T>(),
            new_column: column_new::<T>,
            push_default: column_push_default::<T>,
        });
        self.components_by_type.insert(type_id, component_index);
        self.key_for(component_index)
    }

    pub fn register_tag_type<T: 'static>(&mut self) -> TagKey {
        if let Some(&tag_index) = self.tags_by_type.get(&TypeId::of::<T>()) {
            return self.tag_key_for(tag_index);
        }
        self.claim_bit("tag", std::any::type_name::<T>());
        let tag_index = self.tag_count;
        self.tag_count += 1;
        self.tags_by_type.insert(TypeId::of::<T>(), tag_index);
        self.tag_key_for(tag_index)
    }

    pub fn lookup_tag_type<T: 'static>(&self) -> Option<TagKey> {
        let &tag_index = self.tags_by_type.get(&TypeId::of::<T>())?;
        Some(self.tag_key_for(tag_index))
    }

    pub(crate) fn tag_key_for(&self, tag_index: u32) -> TagKey {
        TagKey {
            tag_index,
            mask: Mask::bit(MASK_BITS - 1 - tag_index),
            registry_id: self.registry_id,
        }
    }

    pub(crate) fn key_for<T>(&self, component_index: u32) -> ComponentKey<T> {
        ComponentKey {
            component_index,
            mask: Mask::bit(component_index),
            registry_id: self.registry_id,
            marker: PhantomData,
        }
    }

    pub fn remaining_bits(&self) -> u32 {
        MASK_BITS - self.components.len() as u32 - self.tag_count
    }

    pub fn all_components_mask(&self) -> Mask {
        Mask::lowest(self.components.len() as u32)
    }

    pub fn all_tags_mask(&self) -> Mask {
        Mask::highest(self.tag_count)
    }

    pub(crate) fn shard_rows_for(&self, mask: Mask) -> usize {
        let row_bytes: usize = self
            .components
            .iter()
            .filter(|info| mask.intersects(info.mask))
            .map(|info| info.item_size)
            .sum();
        if row_bytes == 0 {
            return usize::MAX;
        }
        (SHARD_BUDGET_BYTES / row_bytes).max(MIN_SHARD_ROWS)
    }
}

pub struct ColumnSlot {
    pub component_index: u32,
    pub data: ErasedColumn,
    pub changed: Vec<u32>,
    pub peak_changed: u32,
    pub added: Vec<u32>,
    pub peak_added: u32,
}

impl ColumnSlot {
    #[inline]
    pub(crate) fn track_extend(&mut self, enabled: bool, count: usize, tick: u32) {
        if enabled {
            let filled_length = self.changed.len() + count;
            self.changed.resize(filled_length, tick);
            self.added.resize(filled_length, tick);
            self.peak_changed = tick;
            self.peak_added = tick;
        }
    }

    #[inline]
    pub(crate) fn track_push(&mut self, enabled: bool, tick: u32, added_value: u32) {
        if enabled {
            self.changed.push(tick);
            self.added.push(added_value);
            self.peak_changed = tick;
            self.peak_added = tick;
        }
    }

    #[inline]
    pub(crate) fn track_swap_remove(&mut self, enabled: bool, index: usize) {
        if enabled {
            self.changed.swap_remove(index);
            self.added.swap_remove(index);
        }
    }

    #[inline]
    pub(crate) fn carried_added(&self, index: usize) -> u32 {
        self.added.get(index).copied().unwrap_or(0)
    }
}

pub struct ColumnCell(std::cell::UnsafeCell<ColumnSlot>);

unsafe impl Sync for ColumnCell {}

impl ColumnCell {
    pub(crate) fn new(slot: ColumnSlot) -> Self {
        Self(std::cell::UnsafeCell::new(slot))
    }

    #[inline]
    pub(crate) fn write(&self) -> *mut ColumnSlot {
        self.0.get()
    }
}

impl std::ops::Deref for ColumnCell {
    type Target = ColumnSlot;

    #[inline]
    fn deref(&self) -> &ColumnSlot {
        unsafe { &*self.0.get() }
    }
}

impl std::ops::DerefMut for ColumnCell {
    #[inline]
    fn deref_mut(&mut self) -> &mut ColumnSlot {
        self.0.get_mut()
    }
}

pub const COLUMN_ABSENT: u16 = u16::MAX;

pub(crate) const SHARD_BUDGET_BYTES: usize = 1024 * 1024;

pub(crate) const MIN_SHARD_ROWS: usize = 64;

pub(crate) const COLUMN_STAGGER_BYTES: usize = 64;

pub struct ComponentArrays {
    pub(crate) mask: Mask,
    pub entity_indices: Vec<Entity>,
    pub(crate) columns: Vec<ColumnCell>,
    pub(crate) column_index: Vec<u16>,
    pub(crate) max_rows: usize,
    pub(crate) arena: TableArena,
    pub(crate) capacity: usize,
}

impl ComponentArrays {
    #[inline]
    pub(crate) fn reserve_rows(&mut self, additional: usize) {
        let required = self.entity_indices.len() + additional;
        if required <= self.capacity {
            return;
        }
        let mut new_capacity = self.capacity.max(4);
        while new_capacity < required {
            new_capacity *= 2;
        }
        self.reallocate(new_capacity);
    }

    fn reallocate(&mut self, new_capacity: usize) {
        let mut align = 1usize;
        let mut offsets = Vec::with_capacity(self.columns.len());
        let mut total = 0usize;
        for column in &self.columns {
            let (item_size, item_align) = column.data.item_layout();
            align = align.max(item_align);
            if item_size == 0 {
                offsets.push(0);
                continue;
            }
            total = total.next_multiple_of(item_align);
            offsets.push(total);
            total += item_size * new_capacity + COLUMN_STAGGER_BYTES;
        }

        let arena = TableArena::allocate(total, align);
        let base = arena.base();
        for (column, offset) in self.columns.iter_mut().zip(offsets) {
            let destination = unsafe { base.add(offset) };
            let live = column.data.live_bytes();
            if live != 0 {
                unsafe {
                    std::ptr::copy_nonoverlapping(column.data.base_pointer(), destination, live)
                };
            }
            column.data.rebind(destination, new_capacity);
        }
        self.arena = arena;
        self.capacity = new_capacity;
    }

    #[inline]
    pub(crate) fn column_slot(&self, component_index: u32) -> Option<usize> {
        match self.column_index.get(component_index as usize) {
            Some(&position) if position != COLUMN_ABSENT => Some(position as usize),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn column_at(&self, component_index: u32) -> usize {
        self.column_slot(component_index)
            .expect("component is not present in this table")
    }

    pub(crate) fn rebuild_column_index(&mut self, component_count: usize) {
        self.column_index.clear();
        self.column_index.resize(component_count, COLUMN_ABSENT);
        for (position, column) in self.columns.iter().enumerate() {
            self.column_index[column.component_index as usize] = position as u16;
        }
    }

    #[inline]
    pub fn column<T: 'static>(&self, key: ComponentKey<T>) -> &[T] {
        column_vec::<T>(&self.columns[self.column_at(key.component_index)].data)
    }

    #[inline]
    pub fn column_mut<T: 'static>(&mut self, key: ComponentKey<T>) -> &mut [T] {
        let position = self.column_at(key.component_index);
        column_vec_mut::<T>(&mut self.columns[position].data)
    }

    pub fn mark_column_changed<T: 'static>(&mut self, key: ComponentKey<T>, tick: u32) {
        let Some(position) = self.column_slot(key.component_index) else {
            return;
        };
        let column = &mut self.columns[position];
        column.changed.fill(tick);
        column.peak_changed = tick;
    }
}

pub type DeferredChange = Box<dyn FnOnce(&mut crate::Storage, &mut ResourceMap) + Send + Sync>;

pub(crate) enum Command {
    DespawnEntity(Entity),
    DespawnEntities(Vec<Entity>),
    Closure(DeferredChange),
}

pub(crate) struct EventSlot {
    data: std::cell::UnsafeCell<BoxedAny>,
    update: fn(&mut (dyn Any + Send + Sync)),
}

unsafe impl Sync for EventSlot {}

impl EventSlot {
    pub(crate) fn holding<T: Send + Sync + 'static>() -> Self {
        Self {
            data: std::cell::UnsafeCell::new(Box::new(EventChannel::<T>::new())),
            update: event_update::<T>,
        }
    }

    #[inline]
    pub(crate) fn read<T: Send + Sync + 'static>(&self) -> &EventChannel<T> {
        unsafe { &*self.data.get() }
            .downcast_ref::<EventChannel<T>>()
            .expect("event channel type mismatch")
    }

    #[inline]
    pub(crate) fn lend<T: Send + Sync + 'static>(&self) -> *mut EventChannel<T> {
        unsafe { &mut *self.data.get() }
            .downcast_mut::<EventChannel<T>>()
            .expect("event channel type mismatch")
    }

    #[inline]
    pub(crate) fn advance(&self) {
        (self.update)(unsafe { &mut **self.data.get() });
    }
}

pub(crate) fn event_update<T: Send + Sync + 'static>(data: &mut (dyn Any + Send + Sync)) {
    data.downcast_mut::<EventChannel<T>>()
        .expect("event channel type mismatch")
        .update();
}
