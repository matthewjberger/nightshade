use super::*;

pub struct Storage {
    pub(crate) registry: Components,
    pub allocator: EntityAllocator,
    pub(crate) entity_locations: EntityLocations,
    pub tables: Vec<ComponentArrays>,
    pub table_lookup: MaskMap<usize>,
    pub(crate) table_shards: MaskMap<Vec<usize>>,
    pub(crate) table_edges: Vec<ArchetypeEdges>,
    pub query_cache: MaskMap<Vec<usize>>,
    pub(crate) query_plans: MaskMap<Vec<PlannedTable>>,
    migration_plans: Vec<MigrationPlan>,
    migration_plan_lookup: TablePairMap<u32>,
    last_migration_plan: Option<(u64, u32)>,
    pub current_tick: u32,
    pub last_tick: u32,
    pub structural_log: Vec<StructuralChange>,
    pub structural_sequence: u64,
    pub structural_logging: bool,
    change_detection: bool,
    pub tags: Vec<SparseTagSet>,
    command_buffer: Vec<Command>,
    deferred_commands: std::sync::Mutex<Vec<DeferredCommands>>,
}

pub(crate) struct DeferredCommands {
    pub(crate) commands: Vec<Command>,
    pub(crate) reserved: Vec<Entity>,
}

pub struct World {
    pub storage: Storage,
    pub events: EventBus,
    pub resources: ResourceMap,
}

impl std::ops::Deref for World {
    type Target = Storage;
    fn deref(&self) -> &Storage {
        &self.storage
    }
}

impl std::ops::DerefMut for World {
    fn deref_mut(&mut self) -> &mut Storage {
        &mut self.storage
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn get_location(locations: &EntityLocations, entity: Entity) -> Option<(usize, usize)> {
    let location = locations.get(entity.id)?;
    if !location.allocated || location.generation != entity.generation {
        return None;
    }
    Some((location.table_index as usize, location.array_index as usize))
}

pub(crate) fn insert_location(
    locations: &mut EntityLocations,
    entity: Entity,
    location: (usize, usize),
) {
    locations.insert(
        entity.id,
        EntityLocation {
            generation: entity.generation,
            table_index: location.0 as u32,
            array_index: location.1 as u32,
            allocated: true,
        },
    );
}

impl Storage {
    pub fn tag_key<T: 'static>(&mut self) -> TagKey {
        let key = self.registry.register_tag_type::<T>();
        while self.tags.len() < self.registry.tag_count as usize {
            self.tags.push(SparseTagSet::default());
        }
        key
    }

    pub(crate) fn lookup_tag_key<T: 'static>(&self) -> Option<TagKey> {
        self.registry.lookup_tag_type::<T>()
    }

    pub(crate) fn remaining_bits(&self) -> u32 {
        self.registry.remaining_bits()
    }

    fn check_key(&self, registry_id: u32) {
        debug_assert_eq!(
            registry_id, self.registry.registry_id,
            "key was minted by a different registry than this world's"
        );
    }

    pub fn change_detection(&self) -> bool {
        self.change_detection
    }

    pub fn set_change_detection(&mut self, enabled: bool) {
        if self.change_detection == enabled {
            return;
        }
        self.change_detection = enabled;

        let tick = self.current_tick;
        for table in &mut self.tables {
            let rows = table.entity_indices.len();
            for column in &mut table.columns {
                if enabled {
                    column.changed.resize(rows, tick);
                    column.added.resize(rows, tick);
                    column.peak_changed = tick;
                    column.peak_added = tick;
                } else {
                    column.changed = Vec::new();
                    column.added = Vec::new();
                }
            }
        }
    }

    pub(crate) fn record_structural(
        &mut self,
        entity: Entity,
        kind: StructuralChangeKind,
        mask: Mask,
    ) {
        if !self.structural_logging {
            return;
        }
        if self.structural_log.len() >= STRUCTURAL_LOG_CAPACITY {
            self.structural_log.clear();
        }
        self.structural_sequence += 1;
        self.structural_log.push(StructuralChange {
            sequence: self.structural_sequence,
            entity,
            kind,
            mask,
        });
    }

    pub(crate) fn get_or_create_table(&mut self, mask: Mask) -> usize {
        debug_assert_eq!(
            mask & !self.registry.all_components_mask(),
            Mask::EMPTY,
            "archetype masks must not contain tag bits or unregistered component bits"
        );
        match self.table_lookup.get(&mask) {
            Some(&index) => index,
            None => self.create_table(mask),
        }
    }

    pub(crate) fn insert_table_for(&mut self, mask: Mask) -> usize {
        let table_index = self.get_or_create_table(mask);
        self.shard_with_room(table_index)
    }

    #[inline]
    pub(crate) fn shard_with_room(&mut self, table_index: usize) -> usize {
        let table = &self.tables[table_index];
        if table.entity_indices.len() < table.max_rows {
            return table_index;
        }
        let mask = table.mask;
        if let Some(shards) = self.table_shards.get(&mask) {
            for &shard in shards {
                let candidate = &self.tables[shard];
                if candidate.entity_indices.len() < candidate.max_rows {
                    return shard;
                }
            }
        }
        self.create_table(mask)
    }

    fn create_table(&mut self, mask: Mask) -> usize {
        let component_count = self.registry.components.len();
        for edges in &mut self.table_edges {
            if edges.add_edges.len() < component_count {
                edges.add_edges.resize(component_count, None);
                edges.remove_edges.resize(component_count, None);
            }
        }
        for table in &mut self.tables {
            if table.column_index.len() < component_count {
                table.column_index.resize(component_count, COLUMN_ABSENT);
            }
        }

        let table_index = self.tables.len();
        let columns = self
            .registry
            .components
            .iter()
            .enumerate()
            .filter(|(_, info)| mask.intersects(info.mask))
            .map(|(component_index, info)| {
                ColumnCell::new(ColumnSlot {
                    component_index: component_index as u32,
                    data: (info.new_column)(),
                    changed: Vec::new(),
                    peak_changed: self.current_tick,
                    added: Vec::new(),
                    peak_added: self.current_tick,
                })
            })
            .collect();
        let mut table = ComponentArrays {
            mask,
            entity_indices: Vec::new(),
            columns,
            column_index: Vec::new(),
            max_rows: self.registry.shard_rows_for(mask),
            arena: TableArena::default(),
            capacity: 0,
        };
        table.rebuild_column_index(component_count);
        self.tables.push(table);
        self.query_plans.clear();
        self.table_shards.entry(mask).or_default().push(table_index);

        archetype_register_table(
            ArchetypeRouting {
                table_lookup: &mut self.table_lookup,
                table_edges: &mut self.table_edges,
                query_cache: &mut self.query_cache,
            },
            component_count,
            mask,
            table_index,
            self.registry
                .components
                .iter()
                .enumerate()
                .map(|(component_index, info)| (info.mask, component_index)),
        );

        table_index
    }

    pub fn spawn_entities<B: Bundle>(&mut self, count: usize) -> Vec<Entity> {
        let mask = B::component_mask(self);
        self.spawn_entities_raw(mask, count)
    }

    pub(crate) fn spawn_entities_raw(&mut self, mask: Mask, count: usize) -> Vec<Entity> {
        let _ = self.get_or_create_table(mask);
        let current_tick = self.current_tick;

        let mut allocator = std::mem::take(&mut self.allocator);
        let mut entities = Vec::new();
        allocator.allocate_batch(count, &mut entities);
        self.allocator = allocator;

        if let Some(highest) = entities.iter().map(|entity| entity.id).max() {
            self.entity_locations.reserve_through(highest);
        }

        let track = self.change_detection;
        let mut placed = 0;
        while placed < entities.len() {
            let table_index = self.insert_table_for(mask);
            let start_index = self.tables[table_index].entity_indices.len();
            let room = self.tables[table_index].max_rows - start_index;
            let taken = room.min(entities.len() - placed);
            let batch = &entities[placed..placed + taken];
            {
                let table = &mut self.tables[table_index];
                table.reserve_rows(taken);
                table.entity_indices.reserve(taken);
                for column in &mut table.columns {
                    let info = &self.registry.components[column.component_index as usize];
                    (info.push_default)(&mut column.data, taken);
                    column.track_extend(track, taken, current_tick);
                }
                table.entity_indices.extend_from_slice(batch);
            }
            for (offset, &entity) in batch.iter().enumerate() {
                insert_location(
                    &mut self.entity_locations,
                    entity,
                    (table_index, start_index + offset),
                );
            }
            placed += taken;
        }

        if self.structural_logging {
            for &entity in entities.iter() {
                self.record_structural(entity, StructuralChangeKind::Spawned, mask);
            }
        }

        entities
    }

    fn remove_row(&mut self, table_index: usize, array_index: usize) {
        let track = self.change_detection;
        let table = &mut self.tables[table_index];
        let last_index = table.entity_indices.len() - 1;
        let swapped = (array_index < last_index).then(|| table.entity_indices[last_index]);

        for column in &mut table.columns {
            column.data.swap_remove(array_index);
            column.track_swap_remove(track, array_index);
        }
        table.entity_indices.swap_remove(array_index);

        if let Some(swapped_entity) = swapped
            && let Some(location) = self.entity_locations.get_mut(swapped_entity.id)
            && location.allocated
        {
            location.array_index = array_index as u32;
        }
    }

    fn retire_entity(&mut self, entity: Entity) {
        if let Some((table_index, array_index)) = get_location(&self.entity_locations, entity) {
            self.entity_locations.mark_deallocated(entity.id);
            let despawned_mask = self.tables[table_index].mask;
            self.record_structural(entity, StructuralChangeKind::Despawned, despawned_mask);
            self.remove_row(table_index, array_index);
        }

        let next_generation = entity.generation.wrapping_add(1);
        let should_retire = match self.entity_locations.get(entity.id) {
            None => true,
            Some(location) => {
                !location.allocated && tick_is_newer(next_generation, location.generation)
            }
        };
        if should_retire {
            self.entity_locations
                .ensure_slot(entity.id, next_generation);
        }
    }

    pub fn despawn_entities(&mut self, entities: &[Entity]) -> Vec<Entity> {
        let mut allocator = std::mem::take(&mut self.allocator);
        let mut despawned = Vec::with_capacity(entities.len());
        for &entity in entities {
            if allocator.deallocate(entity) {
                self.retire_entity(entity);
                for tag_set in &mut self.tags {
                    tag_set.remove(entity);
                }
                despawned.push(entity);
            }
        }
        self.allocator = allocator;
        despawned
    }

    fn children(&self, parent: Entity) -> Vec<Entity> {
        self.query_ref::<&ChildOf>()
            .iter()
            .filter(|(_entity, child_of)| child_of.0 == parent)
            .map(|(entity, _child_of)| entity)
            .collect()
    }

    pub fn despawn_recursive(&mut self, root: Entity) -> Vec<Entity> {
        let mut pending = vec![root];
        let mut to_despawn: Vec<Entity> = Vec::new();
        while let Some(parent) = pending.pop() {
            if to_despawn.contains(&parent) {
                continue;
            }
            to_despawn.push(parent);
            pending.extend(self.children(parent));
        }
        self.despawn_entities(&to_despawn)
    }

    fn resolve_migration_plan(&mut self, from_table: usize, to_table: usize) -> usize {
        let key = ((from_table as u64) << 32) | to_table as u64;
        if let Some((cached_key, plan_index)) = self.last_migration_plan
            && cached_key == key
        {
            return plan_index as usize;
        }
        if let Some(&plan_index) = self.migration_plan_lookup.get(&key) {
            self.last_migration_plan = Some((key, plan_index));
            return plan_index as usize;
        }

        let plan = self.build_migration_plan(from_table, to_table);
        let plan_index = self.migration_plans.len() as u32;
        self.migration_plans.push(plan);
        self.migration_plan_lookup.insert(key, plan_index);
        self.last_migration_plan = Some((key, plan_index));
        plan_index as usize
    }

    fn build_migration_plan(&self, from_table: usize, to_table: usize) -> MigrationPlan {
        let source = &self.tables[from_table];
        let destination = &self.tables[to_table];
        let mut plan = MigrationPlan::default();

        for slot in (destination.mask & !source.mask).indices() {
            let position = destination.column_at(slot);
            plan.gained.push((
                position as u32,
                destination.columns[position].component_index,
            ));
        }
        for slot in (source.mask & destination.mask).indices() {
            plan.shared.push((
                source.column_at(slot) as u32,
                destination.column_at(slot) as u32,
            ));
        }
        for slot in (source.mask & !destination.mask).indices() {
            plan.removed.push(source.column_at(slot) as u32);
        }

        plan
    }

    fn move_entity(
        &mut self,
        entity: Entity,
        from_table: usize,
        from_index: usize,
        to_table: usize,
        skip_gained: u32,
    ) -> (usize, usize) {
        let tick = self.current_tick;
        let track = self.change_detection;
        self.tables[to_table].reserve_rows(1);
        let plan_index = self.resolve_migration_plan(from_table, to_table);
        let swapped_entity;
        {
            let plan = &self.migration_plans[plan_index];
            let [source, destination] = self
                .tables
                .get_disjoint_mut([from_table, to_table])
                .expect("migration source and destination must differ");

            for &(destination_position, component_index) in &plan.gained {
                if component_index == skip_gained {
                    continue;
                }
                let destination_column = &mut destination.columns[destination_position as usize];
                let info = &self.registry.components[component_index as usize];
                (info.push_default)(&mut destination_column.data, 1);
                destination_column.track_push(track, tick, tick);
            }

            for &(source_position, destination_position) in &plan.shared {
                let source_position = source_position as usize;
                let destination_position = destination_position as usize;
                let carried_added = if track {
                    source.columns[source_position].carried_added(from_index)
                } else {
                    0
                };
                source.columns[source_position].data.swap_remove_into(
                    from_index,
                    &mut destination.columns[destination_position].data,
                );
                source.columns[source_position].track_swap_remove(track, from_index);
                destination.columns[destination_position].track_push(track, tick, carried_added);
            }

            for &source_position in &plan.removed {
                let source_position = source_position as usize;
                source.columns[source_position].data.swap_remove(from_index);
                source.columns[source_position].track_swap_remove(track, from_index);
            }

            destination.entity_indices.push(entity);

            let last_index = source.entity_indices.len() - 1;
            swapped_entity = (from_index < last_index).then(|| source.entity_indices[last_index]);
            source.entity_indices.swap_remove(from_index);
        }

        let new_index = self.tables[to_table].entity_indices.len() - 1;
        insert_location(&mut self.entity_locations, entity, (to_table, new_index));
        if let Some(swapped) = swapped_entity
            && let Some(location) = self.entity_locations.get_mut(swapped.id)
            && location.allocated
        {
            location.array_index = from_index as u32;
        }
        (to_table, new_index)
    }

    #[inline]
    fn resolve_slot(&mut self, table_index: usize, component_index: u32, adding: bool) -> usize {
        let edges = &self.table_edges[table_index];
        let cached = if adding {
            &edges.add_edges
        } else {
            &edges.remove_edges
        };
        if let Some(Some(target)) = cached.get(component_index as usize).copied() {
            return self.shard_with_room(target);
        }
        let current = self.tables[table_index].mask;
        let bit = Mask::bit(component_index);
        let target = self.get_or_create_table(if adding {
            current | bit
        } else {
            current & !bit
        });
        let edges = &mut self.table_edges[table_index];
        let slot = component_index as usize;
        if adding {
            edges.add_edges[slot] = Some(target);
        } else {
            edges.remove_edges[slot] = Some(target);
        }
        self.shard_with_room(target)
    }

    fn resolve_target(&mut self, table_index: usize, mask: Mask, adding: bool) -> usize {
        if let Some(component_index) = mask.single_slot() {
            return self.resolve_slot(table_index, component_index, adding);
        }
        let edges = &self.table_edges[table_index];
        let cache = if adding {
            &edges.multi_add_cache
        } else {
            &edges.multi_remove_cache
        };
        if let Some(&target) = cache.get(&mask) {
            return self.shard_with_room(target);
        }
        let current = self.tables[table_index].mask;
        let target = self.get_or_create_table(if adding {
            current | mask
        } else {
            current & !mask
        });
        let edges = &mut self.table_edges[table_index];
        if adding {
            edges.multi_add_cache.insert(mask, target);
        } else {
            edges.multi_remove_cache.insert(mask, target);
        }
        self.shard_with_room(target)
    }

    pub fn add_components<B: Bundle>(&mut self, entity: Entity) -> bool {
        let mask = B::component_mask(self);
        self.add_components_raw(entity, mask)
    }

    pub(crate) fn add_components_raw(&mut self, entity: Entity, mask: Mask) -> bool {
        let Some((table_index, array_index)) = get_location(&self.entity_locations, entity) else {
            return false;
        };
        let current_mask = self.tables[table_index].mask;
        if current_mask.contains(mask) {
            return true;
        }

        let target = self.resolve_target(table_index, mask, true);
        self.move_entity(entity, table_index, array_index, target, u32::MAX);
        self.record_structural(
            entity,
            StructuralChangeKind::ComponentsAdded,
            mask & !current_mask,
        );
        true
    }

    pub(crate) fn insert_row(&mut self, entity: Entity, mask: Mask) -> bool {
        if let Some(location) = self.entity_locations.get(entity.id)
            && (location.allocated || location.generation != entity.generation)
        {
            return false;
        }

        let table_index = self.insert_table_for(mask);
        let current_tick = self.current_tick;
        let track = self.change_detection;
        let start_index = self.tables[table_index].entity_indices.len();
        {
            let table = &mut self.tables[table_index];
            table.reserve_rows(1);
            for column in &mut table.columns {
                let info = &self.registry.components[column.component_index as usize];
                (info.push_default)(&mut column.data, 1);
                column.track_push(track, current_tick, current_tick);
            }
            table.entity_indices.push(entity);
        }
        insert_location(
            &mut self.entity_locations,
            entity,
            (table_index, start_index),
        );
        self.record_structural(entity, StructuralChangeKind::Spawned, mask);
        true
    }

    pub fn remove_components<B: Bundle>(&mut self, entity: Entity) -> bool {
        let mask = B::lookup_mask(self);
        self.remove_components_raw(entity, mask)
    }

    pub(crate) fn remove_components_raw(&mut self, entity: Entity, mask: Mask) -> bool {
        let Some((table_index, array_index)) = get_location(&self.entity_locations, entity) else {
            return false;
        };
        let current_mask = self.tables[table_index].mask;
        if !current_mask.intersects(mask) {
            return true;
        }

        let target = self.resolve_target(table_index, mask, false);
        self.move_entity(entity, table_index, array_index, target, u32::MAX);
        self.record_structural(
            entity,
            StructuralChangeKind::ComponentsRemoved,
            current_mask & mask,
        );
        true
    }

    #[inline]
    pub(crate) fn get_keyed<T: 'static>(&self, key: ComponentKey<T>, entity: Entity) -> Option<&T> {
        self.check_key(key.registry_id);
        let (table_index, array_index) = get_location(&self.entity_locations, entity)?;
        let table = &self.tables[table_index];
        let position = table.column_slot(key.component_index)?;
        Some(&column_vec::<T>(&table.columns[position].data)[array_index])
    }

    #[inline]
    pub(crate) fn get_mut_keyed<T: 'static>(
        &mut self,
        key: ComponentKey<T>,
        entity: Entity,
    ) -> Option<&mut T> {
        self.check_key(key.registry_id);
        let (table_index, array_index) = get_location(&self.entity_locations, entity)?;
        let current_tick = self.current_tick;
        let table = &mut self.tables[table_index];
        let position = table.column_slot(key.component_index)?;
        let column = &mut table.columns[position];
        if let Some(cell) = column.changed.get_mut(array_index) {
            *cell = current_tick;
        }
        column.peak_changed = current_tick;
        Some(&mut column_vec_mut::<T>(&mut column.data)[array_index])
    }

    pub(crate) fn set_keyed<T: 'static>(&mut self, key: ComponentKey<T>, entity: Entity, value: T) {
        self.check_key(key.registry_id);
        let current_tick = self.current_tick;
        let Some((table_index, array_index)) = get_location(&self.entity_locations, entity) else {
            return;
        };

        if let Some(position) = self.tables[table_index].column_slot(key.component_index) {
            let column = &mut self.tables[table_index].columns[position];
            column_vec_mut::<T>(&mut column.data)[array_index] = value;
            if let Some(cell) = column.changed.get_mut(array_index) {
                *cell = current_tick;
            }
            column.peak_changed = current_tick;
            return;
        }

        let track = self.change_detection;
        let target = self.resolve_slot(table_index, key.component_index, true);
        let (new_table, _) = self.move_entity(
            entity,
            table_index,
            array_index,
            target,
            key.component_index,
        );
        self.record_structural(entity, StructuralChangeKind::ComponentsAdded, key.mask);
        let position = self.tables[new_table]
            .column_slot(key.component_index)
            .expect("the migration target holds the added component");
        let column = &mut self.tables[new_table].columns[position];
        column.data.push::<T>(value);
        column.track_push(track, current_tick, current_tick);
        column.peak_changed = current_tick;
    }

    pub(crate) fn component_mask(&self, entity: Entity) -> Option<Mask> {
        get_location(&self.entity_locations, entity)
            .map(|(table_index, _)| self.tables[table_index].mask)
    }

    pub fn entity_has_any<B: Bundle>(&self, entity: Entity) -> bool {
        let mask = B::lookup_mask(self);
        !mask.is_empty()
            && self
                .component_mask(entity)
                .unwrap_or(Mask::EMPTY)
                .intersects(mask)
    }

    pub fn entity_has_components<B: Bundle>(&self, entity: Entity) -> bool {
        let Some(mask) = B::strict_mask(self) else {
            return false;
        };
        self.component_mask(entity)
            .unwrap_or(Mask::EMPTY)
            .contains(mask)
    }

    pub fn change_touches<B: Bundle>(&self, change: &StructuralChange) -> bool {
        let mask = B::lookup_mask(self);
        !mask.is_empty() && change.mask.intersects(mask)
    }

    pub fn contains_entity(&self, entity: Entity) -> bool {
        get_location(&self.entity_locations, entity).is_some()
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.allocator.is_alive(entity)
    }

    pub fn entity_count(&self) -> usize {
        self.tables
            .iter()
            .map(|table| table.entity_indices.len())
            .sum()
    }

    pub fn increment_tick(&mut self) {
        self.last_tick = self.current_tick;
        self.current_tick = self.current_tick.wrapping_add(1);
    }

    pub fn current_tick(&self) -> u32 {
        self.current_tick
    }

    pub fn last_tick(&self) -> u32 {
        self.last_tick
    }

    pub fn structural_sequence(&self) -> u64 {
        self.structural_sequence
    }

    pub fn structural_changes_since(&self, cursor: u64) -> &[StructuralChange] {
        let start = self
            .structural_log
            .partition_point(|change| change.sequence <= cursor);
        &self.structural_log[start..]
    }

    pub fn trim_structural_log(&mut self, up_to_sequence: u64) {
        let end = self
            .structural_log
            .partition_point(|change| change.sequence <= up_to_sequence);
        self.structural_log.drain(..end);
    }

    pub fn add_tag(&mut self, key: TagKey, entity: Entity) {
        self.check_key(key.registry_id);
        if self.contains_entity(entity) && self.tags[key.tag_index as usize].insert(entity) {
            self.record_structural(entity, StructuralChangeKind::TagsAdded, key.mask);
        }
    }

    pub fn remove_tag(&mut self, key: TagKey, entity: Entity) -> bool {
        self.check_key(key.registry_id);
        let removed = self.tags[key.tag_index as usize].remove(entity);
        if removed {
            self.record_structural(entity, StructuralChangeKind::TagsRemoved, key.mask);
        }
        removed
    }

    pub fn has_tag(&self, key: TagKey, entity: Entity) -> bool {
        self.check_key(key.registry_id);
        self.tags[key.tag_index as usize].contains(entity)
    }

    pub fn add_tag_type<T: 'static>(&mut self, entity: Entity) {
        let key = self.tag_key::<T>();
        self.add_tag(key, entity);
    }

    pub fn remove_tag_type<T: 'static>(&mut self, entity: Entity) -> bool {
        match self.lookup_tag_key::<T>() {
            Some(key) => self.remove_tag(key, entity),
            None => false,
        }
    }

    pub fn has_tag_type<T: 'static>(&self, entity: Entity) -> bool {
        match self.lookup_tag_key::<T>() {
            Some(key) => self.has_tag(key, entity),
            None => false,
        }
    }

    pub fn query_tag_type<T: 'static>(&self) -> impl Iterator<Item = Entity> + '_ {
        self.lookup_tag_key::<T>()
            .into_iter()
            .flat_map(|key| self.tags[key.tag_index as usize].iter())
    }

    fn reduce_tag_masks(&self, tag_include: Mask, tag_exclude: Mask) -> Option<(Mask, Mask)> {
        if tag_include.is_empty() && tag_exclude.is_empty() {
            return Some((tag_include, tag_exclude));
        }
        let mut reduced_exclude = tag_exclude;
        for (tag_index, tag_set) in self.tags.iter().enumerate() {
            let tag_mask = Mask::bit(MASK_BITS - 1 - tag_index as u32);
            if tag_set.is_empty() {
                if tag_include.intersects(tag_mask) {
                    return None;
                }
                reduced_exclude &= !tag_mask;
            }
        }
        Some((tag_include, reduced_exclude))
    }

    pub(crate) fn split_masks(
        &self,
        include: Mask,
        exclude: Mask,
    ) -> Option<(Mask, Mask, Mask, Mask)> {
        let all_tags = self.registry.all_tags_mask();
        let (tag_include, tag_exclude) =
            self.reduce_tag_masks(include & all_tags, exclude & all_tags)?;
        Some((
            include & !all_tags,
            exclude & !all_tags,
            tag_include,
            tag_exclude,
        ))
    }

    pub fn for_each<I: Bundle, E: Bundle>(
        &self,
        mut f: impl FnMut(Entity, &ComponentArrays, usize),
    ) {
        let Some(include) = I::strict_mask(self) else {
            return;
        };
        let Some((component_include, component_exclude, tag_include, tag_exclude)) =
            self.split_masks(include, E::lookup_mask(self))
        else {
            return;
        };

        for table in &self.tables {
            if !table.mask.contains(component_include) || table.mask.intersects(component_exclude) {
                continue;
            }
            for (index, &entity) in table.entity_indices.iter().enumerate() {
                if tags_match(&self.tags, entity, tag_include, tag_exclude) {
                    f(entity, table, index);
                }
            }
        }
    }

    pub fn for_each_mut<I: Bundle, E: Bundle>(
        &mut self,
        mut f: impl FnMut(Entity, &mut ComponentArrays, usize),
    ) {
        let Some(include) = I::strict_mask(self) else {
            return;
        };
        let Some((component_include, component_exclude, tag_include, tag_exclude)) =
            self.split_masks(include, E::lookup_mask(self))
        else {
            return;
        };

        let tags = &self.tags;
        let table_indices = archetype_cached_tables(
            &mut self.query_cache,
            self.tables.iter().map(|table| table.mask),
            component_include,
        );
        let tables = &mut self.tables;

        for &table_index in table_indices {
            let table = &mut tables[table_index];
            if table.mask.intersects(component_exclude) {
                continue;
            }
            for index in 0..table.entity_indices.len() {
                let entity = table.entity_indices[index];
                if tags_match(tags, entity, tag_include, tag_exclude) {
                    f(entity, table, index);
                }
            }
        }
    }

    pub fn query_entities<B: Bundle>(&self) -> impl Iterator<Item = Entity> + '_ {
        B::strict_mask(self)
            .into_iter()
            .flat_map(move |mask| self.query_entities_raw(mask))
    }

    pub(crate) fn query_entities_raw(&self, mask: Mask) -> impl Iterator<Item = Entity> + '_ {
        self.tables
            .iter()
            .filter(move |table| table.mask.contains(mask))
            .flat_map(|table| table.entity_indices.iter().copied())
    }

    pub fn query_entities_changed_since<B: Bundle>(
        &self,
        since_tick: u32,
    ) -> impl Iterator<Item = Entity> + '_ {
        B::strict_mask(self)
            .into_iter()
            .flat_map(move |mask| self.query_entities_changed_since_raw(mask, since_tick))
    }

    pub(crate) fn query_entities_changed_since_raw(
        &self,
        mask: Mask,
        since_tick: u32,
    ) -> impl Iterator<Item = Entity> + '_ {
        let watched = move |column: &ColumnSlot| mask.contains_bit(column.component_index);
        self.tables
            .iter()
            .filter(move |table| {
                table.mask.contains(mask)
                    && table.columns.iter().any(|column| {
                        watched(column) && tick_is_newer(column.peak_changed, since_tick)
                    })
            })
            .flat_map(move |table| {
                table
                    .entity_indices
                    .iter()
                    .enumerate()
                    .filter(move |(index, _)| {
                        table.columns.iter().any(|column| {
                            watched(column)
                                && column
                                    .changed
                                    .get(*index)
                                    .is_some_and(|&value| tick_is_newer(value, since_tick))
                        })
                    })
                    .map(|(_, &entity)| entity)
            })
    }

    pub fn queue_set<T: Send + Sync + Default + 'static>(&mut self, entity: Entity, value: T) {
        let key = self.component_key::<T>();
        self.queue(move |storage, _| storage.set_keyed(key, entity, value));
    }

    pub fn queue(
        &mut self,
        command: impl FnOnce(&mut Storage, &mut ResourceMap) + Send + Sync + 'static,
    ) {
        self.command_buffer
            .push(Command::Closure(Box::new(command)));
    }

    pub(crate) fn extend_commands(&mut self, commands: Vec<Command>) {
        self.command_buffer.extend(commands);
    }

    pub(crate) fn take_commands(&mut self) -> Vec<Command> {
        std::mem::take(&mut self.command_buffer)
    }

    pub fn command_count(&self) -> usize {
        self.command_buffer.len()
    }

    pub(crate) fn defer_commands(&self, commands: Vec<Command>, reserved: Vec<Entity>) {
        if commands.is_empty() && reserved.is_empty() {
            return;
        }
        self.deferred_commands
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(DeferredCommands { commands, reserved });
    }

    pub(crate) fn take_deferred_commands(&mut self) -> Vec<DeferredCommands> {
        std::mem::take(
            &mut *self
                .deferred_commands
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        )
    }

    pub fn component_key<T: Send + Sync + Default + 'static>(&mut self) -> ComponentKey<T> {
        self.registry.register::<T>()
    }

    pub(crate) fn lookup_key<T: Send + Sync + Default + 'static>(&self) -> Option<ComponentKey<T>> {
        let &component_index = self.registry.components_by_type.get(&TypeId::of::<T>())?;
        Some(self.registry.key_for::<T>(component_index))
    }

    pub fn get<T: Send + Sync + Default + 'static>(&self, entity: Entity) -> Option<&T> {
        self.get_keyed(self.lookup_key::<T>()?, entity)
    }

    pub fn get_mut<T: Send + Sync + Default + 'static>(
        &mut self,
        entity: Entity,
    ) -> Option<&mut T> {
        let key = self.component_key::<T>();
        self.get_mut_keyed(key, entity)
    }

    pub fn set<T: Send + Sync + Default + 'static>(&mut self, entity: Entity, value: T) {
        let key = self.component_key::<T>();
        if !self.contains_entity(entity) && self.is_alive(entity) {
            self.insert_row(entity, key.mask);
            self.set_keyed(key, entity, value);
            return;
        }
        self.set_keyed(key, entity, value);
    }

    pub fn remove<T: Send + Sync + Default + 'static>(&mut self, entity: Entity) -> bool {
        let key = self.component_key::<T>();
        self.remove_components_raw(entity, key.mask)
    }

    pub fn has<T: Send + Sync + Default + 'static>(&self, entity: Entity) -> bool {
        self.get::<T>(entity).is_some()
    }

    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> Entity {
        let mask = B::component_mask(self);
        let entity = self.spawn_entities_raw(mask, 1)[0];
        bundle.write(self, entity);
        entity
    }

    pub(crate) fn insert_bundle<B: Bundle>(&mut self, entity: Entity, bundle: B) {
        if self.is_alive(entity) {
            bundle.write(self, entity);
        }
    }

    pub fn query<Q: QueryTuple>(&mut self) -> QueryBuilder<'_, Q> {
        let element_masks = Q::element_masks(self);
        let include = Q::required_mask_from(&element_masks);
        self.warm_masks(include, Mask::EMPTY);
        QueryBuilder::over(&*self, element_masks, include)
    }

    pub(crate) fn query_shared<Q: QueryTuple>(&self) -> QueryBuilder<'_, Q> {
        match Q::lookup_element_masks(self) {
            Some((element_masks, include)) => QueryBuilder::over(self, element_masks, include),
            None => {
                let mut builder = QueryBuilder::over(self, [Mask::EMPTY; 8], Mask::EMPTY);
                builder.dead = true;
                builder
            }
        }
    }

    pub(crate) fn warm_masks(&mut self, include: Mask, exclude: Mask) {
        let Some((component_include, _, _, _)) = self.split_masks(include, exclude) else {
            return;
        };
        let Storage {
            query_cache,
            query_plans,
            tables,
            ..
        } = self;
        archetype_cached_tables(
            query_cache,
            tables.iter().map(|table| table.mask),
            component_include,
        );
        archetype_planned_tables(query_plans, tables, component_include);
    }

    pub fn query_ref<Q: ReadQueryTuple>(&self) -> QueryRef<'_, Q> {
        QueryRef {
            world: self,
            include: Mask::EMPTY,
            exclude: Mask::EMPTY,
            changed_mask: Mask::EMPTY,
            added_mask: Mask::EMPTY,
            dead: false,
            marker: PhantomData,
        }
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            storage: Storage {
                registry: Components::default(),
                allocator: EntityAllocator::default(),
                entity_locations: EntityLocations::default(),
                tables: Vec::new(),
                table_lookup: MaskMap::default(),
                table_shards: MaskMap::default(),
                table_edges: Vec::new(),
                query_cache: MaskMap::default(),
                query_plans: MaskMap::default(),
                migration_plans: Vec::new(),
                migration_plan_lookup: TablePairMap::default(),
                last_migration_plan: None,
                current_tick: 0,
                last_tick: 0,
                structural_log: Vec::new(),
                structural_sequence: 0,
                structural_logging: false,
                change_detection: false,
                tags: Vec::new(),
                command_buffer: Vec::new(),
                deferred_commands: std::sync::Mutex::new(Vec::new()),
            },
            events: EventBus::default(),
            resources: ResourceMap::default(),
        }
    }

    pub fn tracking() -> Self {
        let mut world = Self::new();
        world.set_change_detection(true);
        world.structural_logging = true;
        world
    }

    pub fn stats(&self) -> WorldStats {
        WorldStats {
            entity_count: self.entity_count(),
            table_count: self.tables.len(),
            empty_table_count: self
                .tables
                .iter()
                .filter(|table| table.entity_indices.is_empty())
                .count(),
            largest_table_rows: self
                .tables
                .iter()
                .map(|table| table.entity_indices.len())
                .max()
                .unwrap_or(0),
            table_rows: self
                .tables
                .iter()
                .map(|table| (table.mask, table.entity_indices.len()))
                .collect(),
            component_count: self.registry.components.len(),
            tag_count: self.registry.tag_count as usize,
            remaining_mask_bits: self.remaining_bits(),
            structural_log_entries: self.structural_log.len(),
            query_cache_entries: self.query_cache.len(),
            resource_count: self.resources.len(),
            event_channels: self.events.channel_count(),
            pending_commands: self.command_count(),
        }
    }

    pub fn step(&mut self) {
        self.events.update();
        self.increment_tick();
    }

    pub fn insert_resource<T: Send + Sync + 'static>(&mut self, value: T) {
        self.resources.insert(value);
    }

    pub fn insert_resources<B: ResourceBundle>(&mut self, bundle: B) {
        bundle.put(&mut self.resources);
    }

    pub fn resource_or_insert_default<T: Send + Sync + Default + 'static>(&mut self) -> &mut T {
        if self.resources.get::<T>().is_none() {
            self.resources.insert(T::default());
        }
        self.res_mut::<T>()
    }

    pub fn res<T: Send + Sync + 'static>(&self) -> &T {
        self.resources.res::<T>()
    }

    pub fn res_mut<T: Send + Sync + 'static>(&mut self) -> &mut T {
        self.resources.res_mut::<T>()
    }

    pub fn remove_resource<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.resources.remove::<T>()
    }

    pub fn resource_scope<R: Send + Sync + 'static, T>(
        &mut self,
        f: impl FnOnce(&mut World, &mut R) -> T,
    ) -> T {
        ResourceHostExt::resource_scope(self, f)
    }

    pub fn with_commands<R>(
        &mut self,
        f: impl FnOnce(&mut World, &mut crate::system_param::Commands) -> R,
    ) -> R {
        let mut buffer =
            crate::system_param::CommandBuffer::drawing_from(self.allocator.next_id.clone());
        let result = {
            let mut commands = crate::system_param::Commands::over(&mut buffer);
            f(self, &mut commands)
        };
        for entity in buffer.take_reserved() {
            self.allocator.revive(entity);
        }
        let queued = buffer.take_commands();
        self.storage.extend_commands(queued);
        self.apply_commands();
        result
    }

    pub fn apply_commands(&mut self) {
        let commands = self.storage.take_commands();
        apply_commands_to(commands, &mut self.storage, &mut self.resources);
    }
}

pub(crate) fn apply_commands_to(
    commands: Vec<Command>,
    storage: &mut Storage,
    resources: &mut ResourceMap,
) {
    for command in commands {
        match command {
            Command::DespawnEntity(entity) => {
                storage.despawn_entities(&[entity]);
            }
            Command::DespawnEntities(entities) => {
                storage.despawn_entities(&entities);
            }
            Command::Closure(command) => command(storage, resources),
        }
    }
}

pub fn with_commands_over<R>(
    storage: &mut Storage,
    resources: &mut ResourceMap,
    f: impl FnOnce(&mut Storage, &mut ResourceMap, &mut crate::system_param::Commands) -> R,
) -> R {
    let mut buffer =
        crate::system_param::CommandBuffer::drawing_from(storage.allocator.next_id.clone());
    let result = {
        let mut commands = crate::system_param::Commands::over(&mut buffer);
        f(storage, resources, &mut commands)
    };
    for entity in buffer.take_reserved() {
        storage.allocator.revive(entity);
    }
    let mut queued = storage.take_commands();
    queued.extend(buffer.take_commands());
    apply_commands_to(queued, storage, resources);
    result
}
