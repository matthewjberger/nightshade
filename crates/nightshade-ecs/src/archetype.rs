use crate::Mask;

#[derive(Clone, Default)]
pub(crate) struct ArchetypeEdges {
    pub add_edges: Vec<Option<usize>>,
    pub remove_edges: Vec<Option<usize>>,
    pub multi_add_cache: crate::MaskMap<usize>,
    pub multi_remove_cache: crate::MaskMap<usize>,
}

impl ArchetypeEdges {
    pub fn new(component_count: usize) -> Self {
        Self {
            add_edges: vec![None; component_count],
            remove_edges: vec![None; component_count],
            multi_add_cache: crate::MaskMap::default(),
            multi_remove_cache: crate::MaskMap::default(),
        }
    }
}

pub(crate) struct ArchetypeRouting<'world, S = std::collections::hash_map::RandomState> {
    pub table_lookup: &'world mut crate::MaskMap<usize>,
    pub table_edges: &'world mut Vec<ArchetypeEdges>,
    pub query_cache: &'world mut std::collections::HashMap<Mask, Vec<usize>, S>,
}

pub(crate) fn archetype_register_table<S>(
    routing: ArchetypeRouting<'_, S>,
    component_count: usize,
    mask: Mask,
    table_index: usize,
    component_bits: impl Iterator<Item = (Mask, usize)>,
) where
    S: std::hash::BuildHasher,
{
    routing
        .table_edges
        .push(ArchetypeEdges::new(component_count));
    routing.table_lookup.insert(mask, table_index);

    for (query_mask, cached_tables) in routing.query_cache.iter_mut() {
        if mask.contains(*query_mask) {
            cached_tables.push(table_index);
        }
    }

    for (component_mask, component_index) in component_bits {
        if mask.intersects(component_mask) {
            let Some(&neighbor) = routing.table_lookup.get(&(mask & !component_mask)) else {
                continue;
            };
            routing.table_edges[table_index].remove_edges[component_index] = Some(neighbor);
            routing.table_edges[neighbor].add_edges[component_index] = Some(table_index);
        } else {
            let Some(&neighbor) = routing.table_lookup.get(&(mask | component_mask)) else {
                continue;
            };
            routing.table_edges[table_index].add_edges[component_index] = Some(neighbor);
            routing.table_edges[neighbor].remove_edges[component_index] = Some(table_index);
        }
    }
}

fn tables_containing<M>(table_masks: M, mask: Mask) -> Vec<usize>
where
    M: Iterator<Item = Mask>,
{
    table_masks
        .enumerate()
        .filter(|(_, table_mask)| table_mask.contains(mask))
        .map(|(table_index, _)| table_index)
        .collect()
}

pub(crate) fn archetype_cached_tables<M, S>(
    query_cache: &mut std::collections::HashMap<Mask, Vec<usize>, S>,
    table_masks: M,
    mask: Mask,
) -> &[usize]
where
    M: Iterator<Item = Mask>,
    S: std::hash::BuildHasher,
{
    query_cache
        .entry(mask)
        .or_insert_with(|| tables_containing(table_masks, mask))
}

pub(crate) fn archetype_read_cached_tables<'cache, M, S>(
    query_cache: &'cache std::collections::HashMap<Mask, Vec<usize>, S>,
    table_masks: M,
    mask: Mask,
) -> std::borrow::Cow<'cache, [usize]>
where
    M: Iterator<Item = Mask>,
    S: std::hash::BuildHasher,
{
    match query_cache.get(&mask) {
        Some(cached) => std::borrow::Cow::Borrowed(cached.as_slice()),
        None => std::borrow::Cow::Owned(tables_containing(table_masks, mask)),
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PlannedTable {
    pub table: u32,
    pub positions: [u16; 8],
}

fn plan_tables(tables: &[crate::ComponentArrays], mask: Mask) -> Vec<PlannedTable> {
    let mut planned = Vec::new();
    for (table_index, table) in tables.iter().enumerate() {
        if !table.mask.contains(mask) {
            continue;
        }
        let mut positions = [u16::MAX; 8];
        for (rank, slot) in mask.indices().enumerate() {
            if rank >= positions.len() {
                break;
            }
            positions[rank] = table.column_index[slot as usize];
        }
        planned.push(PlannedTable {
            table: table_index as u32,
            positions,
        });
    }
    planned
}

pub(crate) fn archetype_planned_tables<'cache, S>(
    query_plans: &'cache mut std::collections::HashMap<Mask, Vec<PlannedTable>, S>,
    tables: &[crate::ComponentArrays],
    mask: Mask,
) -> &'cache [PlannedTable]
where
    S: std::hash::BuildHasher,
{
    query_plans
        .entry(mask)
        .or_insert_with(|| plan_tables(tables, mask))
}

pub(crate) fn archetype_read_planned_tables<'cache, S>(
    query_plans: &'cache std::collections::HashMap<Mask, Vec<PlannedTable>, S>,
    tables: &[crate::ComponentArrays],
    mask: Mask,
) -> std::borrow::Cow<'cache, [PlannedTable]>
where
    S: std::hash::BuildHasher,
{
    match query_plans.get(&mask) {
        Some(planned) => std::borrow::Cow::Borrowed(planned.as_slice()),
        None => std::borrow::Cow::Owned(plan_tables(tables, mask)),
    }
}
