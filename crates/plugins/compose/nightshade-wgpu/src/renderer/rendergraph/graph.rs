use super::error::{RenderGraphError, Result};
use super::pass::{
    ColorTextureBuilder, DepthTextureBuilder, ExecutePhase, MaterialUpdate, NodeView, PassDecl,
    PassExecutionContext, PassNode, PassPosition, PassStage, ResourcePublisher, ResourceReader,
};
use super::resources::{
    AttachmentOp, RenderGraphBufferDescriptor, RenderGraphResources, RenderGraphTextureDescriptor,
    ResourceId, ResourceType,
};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::{BinaryHeap, HashMap, HashSet};
use wgpu::{
    Buffer, CommandBuffer, Device, StoreOp, Texture, TextureFormat, TextureUsages, TextureView,
};

pub struct RenderGraph<C = ()> {
    pub(super) graph: DiGraph<(), ResourceId>,
    pub(super) decls: Vec<PassDecl>,
    pub(super) passes: Vec<Box<dyn PassNode<C>>>,
    pub(super) slots: Vec<ResourceId>,
    pub(super) ops: Vec<AttachmentOp>,
    pub(super) slot_starts: Vec<u32>,
    pub(super) enabled: Vec<bool>,
    pub(super) active: Vec<bool>,
    pub(super) arrival: Vec<u64>,
    pub(super) sort_keys: Vec<u64>,
    pub(super) pass_nodes: HashMap<String, NodeIndex>,
    pub(super) resources: RenderGraphResources,
    pub(super) execution_order: Vec<NodeIndex>,
    pub(super) aliasing_info: Option<ResourceAliasingInfo>,
    pub(super) needs_recompile: bool,
    pub(super) needs_resource_reallocation: bool,
    pub(super) culled: Vec<bool>,
    pub(super) seen_versions: Vec<u64>,
    pub(super) texture_cache: Vec<TextureCacheEntry>,
    pub(super) next_sort_key: u64,
}

const SORT_KEY_STEP: u64 = 1 << 20;

pub struct TextureCacheEntry {
    pub(super) descriptor: RenderGraphTextureDescriptor,
    pub(super) texture: Texture,
    pub(super) bytes: u64,
}

pub(super) const TEXTURE_CACHE_BUDGET_BYTES: u64 = 96 * 1024 * 1024;

fn texture_cache_entry_bytes(descriptor: &RenderGraphTextureDescriptor) -> u64 {
    let block = descriptor.format.block_copy_size(None).unwrap_or(4).max(1) as u64;
    let layers = descriptor.depth_or_array_layers.max(1) as u64;
    let samples = descriptor.sample_count.max(1) as u64;
    let mut total = 0u64;
    for level in 0..descriptor.mip_level_count.max(1) {
        let width = (descriptor.width >> level).max(1) as u64;
        let height = (descriptor.height >> level).max(1) as u64;
        total += width * height * block;
    }
    total * layers * samples
}

pub fn render_graph_new<C: 'static>() -> RenderGraph<C> {
    RenderGraph {
        graph: DiGraph::new(),
        decls: Vec::new(),
        passes: Vec::new(),
        slots: Vec::new(),
        ops: Vec::new(),
        slot_starts: Vec::new(),
        enabled: Vec::new(),
        active: Vec::new(),
        arrival: Vec::new(),
        sort_keys: Vec::new(),
        pass_nodes: HashMap::new(),
        resources: RenderGraphResources::new(),
        execution_order: Vec::new(),
        aliasing_info: None,
        needs_recompile: true,
        needs_resource_reallocation: false,
        culled: Vec::new(),
        seen_versions: Vec::new(),
        texture_cache: Vec::new(),
        next_sort_key: 0,
    }
}

fn view<C: 'static>(graph: &RenderGraph<C>, node_index: NodeIndex) -> NodeView<'_> {
    let row = node_index.index();
    let decl = &graph.decls[row];
    let start = graph.slot_starts[row] as usize;
    let end = start + decl.slot_count();
    NodeView {
        decl,
        slots: &graph.slots[start..end],
        enabled: graph.enabled[row],
    }
}

fn slot_row<C: 'static>(graph: &RenderGraph<C>, node_index: NodeIndex, position: usize) -> usize {
    graph.slot_starts[node_index.index()] as usize + position
}

pub fn render_graph_add_pass<C: 'static>(
    graph: &mut RenderGraph<C>,
    pass: Box<dyn PassNode<C>>,
    slot_mappings: &[(&str, ResourceId)],
) -> Result<NodeIndex> {
    let sort_key = graph.next_sort_key;
    graph.next_sort_key += SORT_KEY_STEP;
    let index = add_pass_with_sort_key(graph, pass, slot_mappings, sort_key)?;
    render_graph_restack(graph);
    Ok(index)
}

pub fn render_graph_material_update<C: 'static>(
    graph: &mut RenderGraph<C>,
    device: &Device,
    update: &MaterialUpdate<'_>,
) {
    for pass in graph.passes.iter_mut() {
        pass.material_update(device, update);
    }
}

pub fn render_graph_resize_passes<C: 'static>(
    graph: &mut RenderGraph<C>,
    device: &Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
) {
    for pass in graph.passes.iter_mut() {
        pass.resize(device, queue, width, height);
    }
}

fn render_graph_restack<C: 'static>(graph: &mut RenderGraph<C>) {
    const POSITION_BAND: u64 = 1 << 40;
    for row in 0..graph.decls.len() {
        let rank = graph.decls[row].position as u64;
        graph.sort_keys[row] = rank * POSITION_BAND + graph.arrival[row];
    }
    graph.needs_recompile = true;
}

pub fn render_graph_add_pass_at<C: 'static>(
    graph: &mut RenderGraph<C>,
    pass: Box<dyn PassNode<C>>,
    slot_mappings: &[(&str, ResourceId)],
    position: PassPosition,
) -> Result<NodeIndex> {
    let sort_key = graph.next_sort_key;
    graph.next_sort_key += SORT_KEY_STEP;
    let index = add_pass_with_sort_key(graph, pass, slot_mappings, sort_key)?;
    graph.decls[index.index()].position = position;
    render_graph_restack(graph);
    Ok(index)
}

fn add_pass_with_sort_key<C: 'static>(
    graph: &mut RenderGraph<C>,
    pass: Box<dyn PassNode<C>>,
    slot_mappings: &[(&str, ResourceId)],
    sort_key: u64,
) -> Result<NodeIndex> {
    let decl = *pass.decl();
    let mut slots = Vec::with_capacity(decl.slot_count());

    for names in [decl.reads, decl.writes, decl.reads_writes] {
        for slot in names {
            let resource = slot_mappings
                .iter()
                .find(|(name, _)| name == slot)
                .map(|(_, resource)| *resource)
                .ok_or_else(|| RenderGraphError::SlotNotMapped {
                    pass: decl.name.to_string(),
                    slot: slot.to_string(),
                })?;
            slots.push(resource);
        }
    }
    for slot in decl.optional_reads {
        let resource = slot_mappings
            .iter()
            .find(|(name, _)| name == slot)
            .map(|(_, resource)| *resource)
            .unwrap_or(ResourceId::UNBOUND);
        slots.push(resource);
    }

    let index = graph.graph.add_node(());
    graph.slot_starts.push(graph.slots.len() as u32);
    graph
        .ops
        .extend(std::iter::repeat_n(AttachmentOp::KEPT, slots.len()));
    graph.slots.extend(slots);
    graph.decls.push(decl);
    graph.passes.push(pass);
    graph.enabled.push(true);
    graph.active.push(true);
    graph.arrival.push(graph.arrival.len() as u64);
    graph.sort_keys.push(sort_key);
    graph.culled.resize(graph.graph.node_count(), false);
    graph.pass_nodes.insert(decl.name.to_string(), index);
    graph.needs_recompile = true;
    Ok(index)
}

pub fn render_graph_get_pass_mut<'a, C: 'static>(
    graph: &'a mut RenderGraph<C>,
    name: &str,
) -> Option<&'a mut (dyn PassNode<C> + 'static)> {
    let node_index = *graph.pass_nodes.get(name)?;
    graph
        .passes
        .get_mut(node_index.index())
        .map(|pass| pass.as_mut())
}

fn render_graph_set_node_enabled<C: 'static>(
    graph: &mut RenderGraph<C>,
    node_index: NodeIndex,
    enabled: bool,
) -> bool {
    let Some(held) = graph.enabled.get_mut(node_index.index()) else {
        return false;
    };
    let changed = *held != enabled;
    *held = enabled;
    changed
}

fn render_graph_refresh_attachment_ops<C: 'static>(graph: &mut RenderGraph<C>) {
    if graph.execution_order.is_empty() {
        return;
    }
    render_graph_compute_attachment_ops(graph);
}

pub fn render_graph_set_pass_enabled<C: 'static>(
    graph: &mut RenderGraph<C>,
    name: &str,
    enabled: bool,
) -> Result<()> {
    let node_index = *graph
        .pass_nodes
        .get(name)
        .ok_or_else(|| RenderGraphError::PassNotFound {
            pass: name.to_string(),
        })?;

    if render_graph_set_node_enabled(graph, node_index, enabled) {
        render_graph_refresh_attachment_ops(graph);
    }

    Ok(())
}

pub fn render_graph_pass_names_in_stage<C: 'static>(
    graph: &RenderGraph<C>,
    stage: PassStage,
) -> Vec<String> {
    let mut named: Vec<(u64, String)> = graph
        .decls
        .iter()
        .zip(&graph.sort_keys)
        .filter(|(decl, _)| decl.stage == stage)
        .map(|(decl, sort_key)| (*sort_key, decl.name.to_string()))
        .collect();
    named.sort_by_key(|(sort_key, _)| *sort_key);
    named.into_iter().map(|(_, name)| name).collect()
}

pub fn render_graph_enable_only_passes<C: 'static>(
    graph: &mut RenderGraph<C>,
    passes: &[(&str, bool)],
) {
    let mut changed = false;
    let node_indices: Vec<NodeIndex> = graph.pass_nodes.values().copied().collect();
    for node_index in node_indices {
        changed |= render_graph_set_node_enabled(graph, node_index, false);
    }
    for &(name, enabled) in passes {
        if let Some(&node_index) = graph.pass_nodes.get(name) {
            changed |= render_graph_set_node_enabled(graph, node_index, enabled);
        }
    }
    if changed {
        render_graph_refresh_attachment_ops(graph);
    }
}

pub fn render_graph_save_pass_enabled_states<C: 'static>(
    graph: &RenderGraph<C>,
) -> HashMap<String, bool> {
    graph
        .pass_nodes
        .iter()
        .filter_map(|(name, node_index)| {
            graph
                .enabled
                .get(node_index.index())
                .map(|enabled| (name.clone(), *enabled))
        })
        .collect()
}

pub fn render_graph_restore_pass_enabled_states<C: 'static>(
    graph: &mut RenderGraph<C>,
    states: &HashMap<String, bool>,
) {
    let mut changed = false;
    for (name, enabled) in states {
        if let Some(&node_index) = graph.pass_nodes.get(name) {
            changed |= render_graph_set_node_enabled(graph, node_index, *enabled);
        }
    }
    if changed {
        render_graph_refresh_attachment_ops(graph);
    }
}

pub fn render_graph_bind_pass_slot<C: 'static>(
    graph: &mut RenderGraph<C>,
    pass: &str,
    slot: &str,
    resource: ResourceId,
) -> Result<()> {
    let node_index = *graph
        .pass_nodes
        .get(pass)
        .ok_or_else(|| RenderGraphError::PassNotFound {
            pass: pass.to_string(),
        })?;
    let position = graph.decls[node_index.index()]
        .position(slot)
        .ok_or_else(|| RenderGraphError::SlotNotMapped {
            pass: pass.to_string(),
            slot: slot.to_string(),
        })?;
    let row = slot_row(graph, node_index, position);
    graph.slots[row] = resource;
    graph.needs_recompile = true;
    Ok(())
}

pub fn render_graph_add_color_texture<'a, C: 'static>(
    graph: &'a mut RenderGraph<C>,
    name: &str,
) -> ColorTextureBuilder<'a, C> {
    ColorTextureBuilder {
        graph,
        name: name.to_string(),
        descriptor: RenderGraphTextureDescriptor {
            format: TextureFormat::Rgba8UnormSrgb,
            width: 1,
            height: 1,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            sample_count: 1,
            mip_level_count: 1,
            dimension: wgpu::TextureDimension::D2,
            depth_or_array_layers: 1,
        },
        clear_color: None,
        force_store: false,
        fixed_size: false,
    }
}

pub fn render_graph_add_depth_texture<'a, C: 'static>(
    graph: &'a mut RenderGraph<C>,
    name: &str,
) -> DepthTextureBuilder<'a, C> {
    DepthTextureBuilder {
        graph,
        name: name.to_string(),
        descriptor: RenderGraphTextureDescriptor {
            format: TextureFormat::Depth32Float,
            width: 1,
            height: 1,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            sample_count: 1,
            mip_level_count: 1,
            dimension: wgpu::TextureDimension::D2,
            depth_or_array_layers: 1,
        },
        clear_depth: None,
        force_store: false,
        fixed_size: false,
    }
}

pub fn render_graph_set_external_texture<C: 'static>(
    graph: &mut RenderGraph<C>,
    id: ResourceId,
    texture: Option<Texture>,
    view: TextureView,
    width: u32,
    height: u32,
) {
    graph
        .resources
        .set_external_texture(id, texture, view, width, height);
}

pub fn render_graph_get_texture<C: 'static>(
    graph: &RenderGraph<C>,
    id: ResourceId,
) -> Option<&Texture> {
    let pooled = graph.aliasing_info.as_ref().and_then(|aliasing_info| {
        let pool_index = aliasing_info.aliases.get(&id)?;
        let pool_slot = aliasing_info.pools.get(*pool_index)?;
        match &pool_slot.resource {
            Some(PooledResource::Texture { texture }) => Some(texture),
            _ => None,
        }
    });
    pooled.or_else(|| graph.resources.get_texture(id))
}

pub fn render_graph_get_texture_view<C: 'static>(
    graph: &RenderGraph<C>,
    id: ResourceId,
) -> Option<&TextureView> {
    graph.resources.get_texture_view(id)
}

fn render_graph_build_dependency_edges<C: 'static>(graph: &mut RenderGraph<C>) {
    let mut resource_writers: HashMap<ResourceId, Vec<NodeIndex>> = HashMap::new();

    let mut node_indices: Vec<NodeIndex> = graph.graph.node_indices().collect();
    node_indices.sort_by_key(|&node_index| (graph.sort_keys[node_index.index()], node_index));
    let mut edges_to_add: Vec<(NodeIndex, NodeIndex, ResourceId)> = Vec::new();

    for &node_index in &node_indices {
        let node = view(graph, node_index);
        let reads = node.reads().to_vec();
        let writes = node.writes().to_vec();
        let reads_writes = node.reads_writes().to_vec();
        let optional_reads: Vec<ResourceId> = node.optional_reads().collect();

        for &read_resource in reads.iter().chain(&optional_reads).chain(&reads_writes) {
            if let Some(writers) = resource_writers.get(&read_resource) {
                for &writer_index in writers {
                    edges_to_add.push((writer_index, node_index, read_resource));
                }
            }
        }

        for &write_resource in writes.iter().chain(&reads_writes) {
            if let Some(writers) = resource_writers.get(&write_resource) {
                for &writer_index in writers {
                    edges_to_add.push((writer_index, node_index, write_resource));
                }
            }
        }

        for &write_resource in writes.iter().chain(&reads_writes) {
            resource_writers
                .entry(write_resource)
                .or_default()
                .push(node_index);
        }
    }

    for (from, to, resource) in edges_to_add {
        if from != to && !graph.graph.contains_edge(from, to) {
            graph.graph.add_edge(from, to, resource);
        }
    }
}

fn render_graph_compute_resource_lifetimes<C: 'static>(
    graph: &RenderGraph<C>,
    execution_order: &[NodeIndex],
) -> Vec<ResourceLifetime> {
    let mut lifetimes: HashMap<ResourceId, ResourceLifetime> = HashMap::new();

    for (pass_index, &node_index) in execution_order.iter().enumerate() {
        let node = view(graph, node_index);

        for &resource_id in node.writes() {
            lifetimes.entry(resource_id).or_insert(ResourceLifetime {
                resource_id,
                first_use: pass_index,
                last_use: pass_index,
            });
        }

        for &resource_id in node.reads() {
            let lifetime = lifetimes.entry(resource_id).or_insert(ResourceLifetime {
                resource_id,
                first_use: pass_index,
                last_use: pass_index,
            });
            lifetime.last_use = pass_index;
        }

        for &resource_id in node.reads_writes() {
            let lifetime = lifetimes.entry(resource_id).or_insert(ResourceLifetime {
                resource_id,
                first_use: pass_index,
                last_use: pass_index,
            });
            lifetime.last_use = pass_index;
        }

        for resource_id in node.optional_reads() {
            let lifetime = lifetimes.entry(resource_id).or_insert(ResourceLifetime {
                resource_id,
                first_use: pass_index,
                last_use: pass_index,
            });
            lifetime.last_use = pass_index;
        }
    }

    lifetimes
        .into_iter()
        .filter(|(id, _)| {
            if let Some(desc) = graph.resources.get_descriptor(*id) {
                !desc.is_external
            } else {
                false
            }
        })
        .map(|(_, lifetime)| lifetime)
        .collect()
}

fn can_alias_textures(
    desc1: &RenderGraphTextureDescriptor,
    desc2: &RenderGraphTextureDescriptor,
) -> bool {
    desc1.format == desc2.format
        && desc1.width == desc2.width
        && desc1.height == desc2.height
        && desc1.sample_count == desc2.sample_count
        && desc1.mip_level_count == desc2.mip_level_count
        && desc1.usage.contains(desc2.usage)
}

fn can_alias_buffers(
    desc1: &RenderGraphBufferDescriptor,
    desc2: &RenderGraphBufferDescriptor,
) -> bool {
    desc1.size >= desc2.size && desc1.usage == desc2.usage
}

fn render_graph_compute_resource_aliasing<C: 'static>(
    graph: &RenderGraph<C>,
    mut lifetimes: Vec<ResourceLifetime>,
) -> ResourceAliasingInfo {
    lifetimes.sort_by_key(|lt| lt.first_use);

    let mut aliasing_info = ResourceAliasingInfo {
        aliases: HashMap::new(),
        pools: Vec::new(),
    };

    let mut available_pools: BinaryHeap<PoolHeapEntry> = BinaryHeap::new();

    for lifetime in lifetimes {
        let descriptor = graph
            .resources
            .get_descriptor(lifetime.resource_id)
            .unwrap();

        if descriptor.fixed_size {
            continue;
        }

        let force_store = match &descriptor.resource_type {
            ResourceType::TransientColor { force_store, .. }
            | ResourceType::TransientDepth { force_store, .. } => *force_store,
            _ => false,
        };

        let mut reused_candidates = Vec::new();
        while let Some(entry) = available_pools.peek() {
            if entry.lifetime_end < lifetime.first_use {
                reused_candidates.push(available_pools.pop().unwrap());
            } else {
                break;
            }
        }

        let mut assigned_slot = None;
        if force_store {
            for candidate in reused_candidates {
                available_pools.push(candidate);
            }
            let slot_index = aliasing_info.pools.len();
            let descriptor_info = match &descriptor.resource_type {
                ResourceType::TransientColor {
                    descriptor: tex_desc,
                    ..
                }
                | ResourceType::TransientDepth {
                    descriptor: tex_desc,
                    ..
                } => PoolDescriptorInfo::Texture(tex_desc.clone()),
                ResourceType::TransientBuffer {
                    descriptor: buf_desc,
                } => PoolDescriptorInfo::Buffer(buf_desc.clone()),
                _ => continue,
            };
            aliasing_info.pools.push(PoolSlot {
                resource: None,
                descriptor_info: Some(descriptor_info),
                lifetime_end: lifetime.last_use,
            });
            aliasing_info
                .aliases
                .insert(lifetime.resource_id, slot_index);
            continue;
        }

        for candidate in reused_candidates.iter_mut() {
            let can_reuse = match (&candidate.descriptor_info, &descriptor.resource_type) {
                (
                    PoolDescriptorInfo::Texture(pool_desc),
                    ResourceType::TransientColor {
                        descriptor: res_desc,
                        ..
                    }
                    | ResourceType::TransientDepth {
                        descriptor: res_desc,
                        ..
                    },
                ) => can_alias_textures(pool_desc, res_desc),
                (
                    PoolDescriptorInfo::Buffer(pool_desc),
                    ResourceType::TransientBuffer {
                        descriptor: res_desc,
                    },
                ) => can_alias_buffers(pool_desc, res_desc),
                _ => false,
            };

            if can_reuse {
                let needs_new_resource =
                    match (&mut candidate.descriptor_info, &descriptor.resource_type) {
                        (
                            PoolDescriptorInfo::Texture(pool_desc),
                            ResourceType::TransientColor {
                                descriptor: res_desc,
                                ..
                            }
                            | ResourceType::TransientDepth {
                                descriptor: res_desc,
                                ..
                            },
                        ) if !pool_desc.usage.contains(res_desc.usage) => {
                            pool_desc.usage |= res_desc.usage;
                            true
                        }
                        (
                            PoolDescriptorInfo::Buffer(pool_desc),
                            ResourceType::TransientBuffer {
                                descriptor: res_desc,
                            },
                        ) if res_desc.size > pool_desc.size => {
                            *pool_desc = res_desc.clone();
                            true
                        }
                        _ => false,
                    };

                let pool_slot = &mut aliasing_info.pools[candidate.pool_index];
                pool_slot.lifetime_end = lifetime.last_use;
                pool_slot.descriptor_info = Some(candidate.descriptor_info.clone());

                if needs_new_resource {
                    pool_slot.resource = None;
                }

                candidate.lifetime_end = lifetime.last_use;
                assigned_slot = Some(candidate.pool_index);
                break;
            }
        }

        for candidate in reused_candidates {
            available_pools.push(candidate);
        }

        if assigned_slot.is_none() {
            let slot_index = aliasing_info.pools.len();

            let descriptor_info = match &descriptor.resource_type {
                ResourceType::TransientColor {
                    descriptor: tex_desc,
                    ..
                }
                | ResourceType::TransientDepth {
                    descriptor: tex_desc,
                    ..
                } => PoolDescriptorInfo::Texture(tex_desc.clone()),
                ResourceType::TransientBuffer {
                    descriptor: buf_desc,
                } => PoolDescriptorInfo::Buffer(buf_desc.clone()),
                _ => continue,
            };

            aliasing_info.pools.push(PoolSlot {
                resource: None,
                descriptor_info: Some(descriptor_info.clone()),
                lifetime_end: lifetime.last_use,
            });

            available_pools.push(PoolHeapEntry {
                pool_index: slot_index,
                lifetime_end: lifetime.last_use,
                descriptor_info,
            });

            assigned_slot = Some(slot_index);
        }

        aliasing_info
            .aliases
            .insert(lifetime.resource_id, assigned_slot.unwrap());
    }

    aliasing_info
}

fn render_graph_compute_attachment_ops<C: 'static>(graph: &mut RenderGraph<C>) {
    let order = graph.execution_order.clone();

    let mut first_write: HashMap<ResourceId, NodeIndex> = HashMap::new();
    for &node_index in &order {
        let node = view(graph, node_index);
        for &resource_id in node.writes().iter().chain(node.reads_writes()) {
            first_write.entry(resource_id).or_insert(node_index);
        }
    }

    let mut last_read: HashMap<ResourceId, usize> = HashMap::new();
    for (index, &node_index) in order.iter().enumerate().rev() {
        let node = view(graph, node_index);
        if !node.enabled {
            continue;
        }
        for resource_id in node
            .reads()
            .iter()
            .copied()
            .chain(node.reads_writes().iter().copied())
            .chain(node.optional_reads())
        {
            last_read.entry(resource_id).or_insert(index);
        }
    }

    let mut rows: Vec<(NodeIndex, usize, AttachmentOp)> = Vec::new();
    for (index, &node_index) in order.iter().enumerate() {
        let node = view(graph, node_index);
        let enabled = node.enabled;
        let attachments: Vec<usize> = node
            .decl
            .writes_at()
            .chain(node.decl.reads_writes_at())
            .collect();

        for position in attachments {
            let resource_id = node.slots[position];
            let Some(descriptor) = graph.resources.get_descriptor(resource_id) else {
                continue;
            };

            let has_clear = match &descriptor.resource_type {
                ResourceType::ExternalColor { clear_color, .. }
                | ResourceType::TransientColor { clear_color, .. } => clear_color.is_some(),
                ResourceType::ExternalDepth { clear_depth, .. }
                | ResourceType::TransientDepth { clear_depth, .. } => clear_depth.is_some(),
                _ => false,
            };
            let clear = has_clear && first_write.get(&resource_id) == Some(&node_index);

            let force_store = match &descriptor.resource_type {
                ResourceType::ExternalColor { .. } | ResourceType::ExternalDepth { .. } => true,
                ResourceType::TransientColor { force_store, .. }
                | ResourceType::TransientDepth { force_store, .. } => *force_store,
                _ => false,
            };
            let has_later_read = last_read
                .get(&resource_id)
                .is_some_and(|&last| last > index);
            let store = match &descriptor.resource_type {
                ResourceType::ExternalBuffer | ResourceType::TransientBuffer { .. } => {
                    StoreOp::Store
                }
                _ if !enabled => StoreOp::Store,
                _ if force_store || has_later_read => StoreOp::Store,
                _ => StoreOp::Discard,
            };

            rows.push((node_index, position, AttachmentOp { clear, store }));
        }
    }

    for op in graph.ops.iter_mut() {
        *op = AttachmentOp::KEPT;
    }
    for (node_index, position, op) in rows {
        let row = slot_row(graph, node_index, position);
        graph.ops[row] = op;
    }
}

fn render_graph_mark_dead_passes<C: 'static>(graph: &mut RenderGraph<C>) {
    let execution_order = graph.execution_order.clone();
    let mut required_resources: HashSet<ResourceId> = HashSet::new();
    let mut required_passes: HashSet<NodeIndex> = HashSet::new();

    for (resource_id, descriptor) in graph.resources.entries() {
        if descriptor.is_external {
            required_resources.insert(resource_id);
        }
    }

    for &node_index in execution_order.iter().rev() {
        let node = view(graph, node_index);

        let has_side_effects = node.writes().is_empty() && node.reads_writes().is_empty();

        let writes_required_resource = node
            .writes()
            .iter()
            .any(|resource| required_resources.contains(resource))
            || node
                .reads_writes()
                .iter()
                .any(|resource| required_resources.contains(resource));

        if writes_required_resource || has_side_effects {
            required_passes.insert(node_index);
            required_resources.extend(node.reads());
            required_resources.extend(node.reads_writes());
            required_resources.extend(node.optional_reads());
        }
    }

    graph.culled.clear();
    graph.culled.resize(graph.graph.node_count(), false);
    for &node_index in &execution_order {
        graph.culled[node_index.index()] = !required_passes.contains(&node_index);
    }
}

pub fn render_graph_compile<C: 'static>(graph: &mut RenderGraph<C>) -> Result<()> {
    render_graph_build_dependency_edges(graph);

    graph.execution_order = petgraph::algo::toposort(&graph.graph, None).map_err(|cycle| {
        let pass = graph
            .graph
            .node_weight(cycle.node_id())
            .map(|_| graph.decls[cycle.node_id().index()].name.to_string())
            .unwrap_or_else(|| "<unknown>".to_string());
        RenderGraphError::CyclicDependency { pass }
    })?;

    render_graph_compute_attachment_ops(graph);

    let lifetimes = render_graph_compute_resource_lifetimes(graph, &graph.execution_order.clone());
    graph.aliasing_info = Some(render_graph_compute_resource_aliasing(graph, lifetimes));

    render_graph_mark_dead_passes(graph);

    graph.needs_recompile = false;
    Ok(())
}

fn render_graph_recompile_if_needed<C: 'static>(graph: &mut RenderGraph<C>) -> Result<()> {
    if graph.needs_resource_reallocation {
        let lifetimes =
            render_graph_compute_resource_lifetimes(graph, &graph.execution_order.clone());
        graph.aliasing_info = Some(render_graph_compute_resource_aliasing(graph, lifetimes));
        graph.needs_resource_reallocation = false;
        return Ok(());
    }

    if !graph.needs_recompile {
        return Ok(());
    }

    let edge_indices: Vec<_> = graph.graph.edge_indices().collect();
    for edge_index in edge_indices {
        graph.graph.remove_edge(edge_index);
    }

    render_graph_build_dependency_edges(graph);

    graph.execution_order = petgraph::algo::toposort(&graph.graph, None).map_err(|cycle| {
        let pass = graph
            .graph
            .node_weight(cycle.node_id())
            .map(|_| graph.decls[cycle.node_id().index()].name.to_string())
            .unwrap_or_else(|| "<unknown>".to_string());
        RenderGraphError::CyclicDependency { pass }
    })?;

    render_graph_compute_attachment_ops(graph);

    let lifetimes = render_graph_compute_resource_lifetimes(graph, &graph.execution_order.clone());
    graph.aliasing_info = Some(render_graph_compute_resource_aliasing(graph, lifetimes));

    render_graph_mark_dead_passes(graph);

    graph.needs_recompile = false;
    Ok(())
}

pub fn render_graph_execute<C: 'static + Sync>(
    graph: &mut RenderGraph<C>,
    device: &Device,
    queue: &wgpu::Queue,
    configs: &C,
) -> Result<Vec<CommandBuffer>> {
    render_graph_execute_with_phase(graph, device, queue, configs, ExecutePhase::Full)
}

pub fn render_graph_execute_with_phase<C: 'static + Sync>(
    graph: &mut RenderGraph<C>,
    device: &Device,
    queue: &wgpu::Queue,
    configs: &C,
    phase: ExecutePhase,
) -> Result<Vec<CommandBuffer>> {
    {
        #[cfg(feature = "profiling")]
        let _span = tracing::info_span!("graph_recompile_if_needed").entered();
        render_graph_recompile_if_needed(graph)?;
    }

    if graph.aliasing_info.is_none() {
        #[cfg(feature = "profiling")]
        let _span = tracing::info_span!("graph_plan_aliasing").entered();
        let lifetimes =
            render_graph_compute_resource_lifetimes(graph, &graph.execution_order.clone());
        graph.aliasing_info = Some(render_graph_compute_resource_aliasing(graph, lifetimes));
    }

    {
        #[cfg(feature = "profiling")]
        let _span = tracing::info_span!("graph_allocate_transient").entered();
        if let Some(aliasing_info) = &mut graph.aliasing_info {
            let mut texture_cache = std::mem::take(&mut graph.texture_cache);
            graph.resources.allocate_transient_resources_with_aliasing(
                device,
                aliasing_info,
                &mut texture_cache,
            );
            graph.texture_cache = texture_cache;
        }
    }

    {
        #[cfg(feature = "profiling")]
        let _span = tracing::info_span!("graph_publish_resources").entered();
        let mut publisher = ResourcePublisher {
            resources: &mut graph.resources,
        };
        for pass in graph.passes.iter_mut() {
            pass.publish(&mut publisher);
        }
        let reader = ResourceReader {
            resources: &graph.resources,
        };
        for pass in graph.passes.iter_mut() {
            pass.consume(&reader);
        }
    }

    {
        #[cfg(feature = "profiling")]
        let _span = tracing::info_span!("graph_pass_enabled").entered();
        graph.active.resize(graph.passes.len(), true);
        for (row, pass) in graph.passes.iter().enumerate() {
            graph.active[row] = graph.enabled[row] && pass.enabled(configs);
        }
    }

    {
        #[cfg(feature = "profiling")]
        let _span = tracing::info_span!("graph_invalidate_bind_groups").entered();
        render_graph_invalidate_bind_groups_for_changed_resources(graph);
    }

    {
        for &node_index in &graph.execution_order {
            if graph.culled[node_index.index()] {
                continue;
            }

            let row = node_index.index();
            if !graph.active[row] {
                continue;
            }
            match phase {
                ExecutePhase::Full => {
                    if !graph.decls[row].full_phase {
                        continue;
                    }
                }
                ExecutePhase::ComposeOnly => {
                    if !graph.decls[row].compose_only_phase {
                        continue;
                    }
                }
            }
            #[cfg(feature = "profiling")]
            let _pass_span =
                tracing::info_span!("pass_prepare", name = %graph.decls[row].name).entered();
            graph.passes[row].prepare(device, queue, configs);
        }
    }

    #[cfg(all(feature = "parallel", not(target_family = "wasm")))]
    return render_graph_execute_parallel(graph, device, queue, configs, phase);

    #[cfg(not(all(feature = "parallel", not(target_family = "wasm"))))]
    render_graph_execute_serial(graph, device, queue, configs, phase)
}

fn render_graph_invalidate_bind_groups_for_changed_resources<C: 'static>(
    graph: &mut RenderGraph<C>,
) {
    let mut dirty_resources = HashSet::new();

    for resource_id in graph.resources.ids().collect::<Vec<_>>() {
        let row = resource_id.0 as usize;
        if graph.seen_versions.len() <= row {
            graph.seen_versions.resize(row + 1, 0);
        }
        let current_version = graph.resources.get_version(resource_id);

        if current_version != graph.seen_versions[row] {
            dirty_resources.insert(resource_id);
            graph.seen_versions[row] = current_version;
        }
    }

    if dirty_resources.is_empty() {
        return;
    }

    let mut passes_to_invalidate = HashSet::new();

    for &node_index in &graph.execution_order {
        let node = view(graph, node_index);

        for resource_id in node
            .reads()
            .iter()
            .copied()
            .chain(node.writes().iter().copied())
            .chain(node.reads_writes().iter().copied())
            .chain(node.optional_reads())
        {
            if dirty_resources.contains(&resource_id) {
                passes_to_invalidate.insert(node_index);
                break;
            }
        }
    }

    for &node_index in &passes_to_invalidate {
        graph.passes[node_index.index()].invalidate_bind_groups();
    }
}

pub(super) fn render_graph_recording_order<C: 'static>(
    graph: &RenderGraph<C>,
    phase: ExecutePhase,
) -> Vec<NodeIndex> {
    graph
        .execution_order
        .iter()
        .copied()
        .filter(|node_index| !graph.culled[node_index.index()])
        .filter(|node_index| match phase {
            ExecutePhase::Full => graph.decls[node_index.index()].full_phase,
            ExecutePhase::ComposeOnly => graph.decls[node_index.index()].compose_only_phase,
        })
        .collect()
}

#[cfg(all(feature = "parallel", not(target_family = "wasm")))]
fn render_graph_execute_parallel<C: 'static + Sync>(
    graph: &mut RenderGraph<C>,
    device: &Device,
    queue: &wgpu::Queue,
    configs: &C,
    phase: ExecutePhase,
) -> Result<Vec<CommandBuffer>> {
    use rayon::iter::{IntoParallelIterator, ParallelIterator};

    let order = render_graph_recording_order(graph, phase);

    let RenderGraph {
        decls,
        passes,
        slots,
        ops,
        slot_starts,
        active,
        resources,
        ..
    } = graph;

    let mut held: Vec<Option<&mut Box<dyn PassNode<C>>>> = passes.iter_mut().map(Some).collect();

    let mut jobs = Vec::with_capacity(order.len());
    for node_index in order {
        let row = node_index.index();
        let pass = held[row]
            .take()
            .expect("a pass appears twice in the execution order");
        let decl = &decls[row];
        let start = slot_starts[row] as usize;
        let end = start + decl.slot_count();
        jobs.push((
            pass,
            decl,
            &slots[start..end],
            &ops[start..end],
            active[row],
        ));
    }

    jobs.into_par_iter()
        .map(|(pass, decl, slots, ops, enabled)| {
            let name = decl.name;

            #[cfg(feature = "profiling")]
            let _pass_span = tracing::info_span!("pass_execute", name = %name).entered();

            let mut encoder = device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(name) });
            encoder.push_debug_group(name);
            pass.execute(PassExecutionContext {
                encoder: &mut encoder,
                resources,
                device,
                queue,
                decl,
                slots,
                ops,
                configs,
                pass_enabled: enabled,
            })?;
            encoder.pop_debug_group();

            Ok(encoder.finish())
        })
        .collect()
}

#[cfg(not(all(feature = "parallel", not(target_family = "wasm"))))]
fn render_graph_execute_serial<C: 'static + Sync>(
    graph: &mut RenderGraph<C>,
    device: &Device,
    queue: &wgpu::Queue,
    configs: &C,
    phase: ExecutePhase,
) -> Result<Vec<CommandBuffer>> {
    let order = render_graph_recording_order(graph, phase);
    let mut command_buffers = Vec::new();
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("RenderGraph Encoder"),
    });

    let RenderGraph {
        decls,
        passes,
        slots,
        ops,
        slot_starts,
        active,
        resources,
        ..
    } = graph;

    for node_index in order {
        let row = node_index.index();
        let decl = &decls[row];
        let start = slot_starts[row] as usize;
        let end = start + decl.slot_count();
        let name = decl.name;

        #[cfg(feature = "profiling")]
        let _pass_span = tracing::info_span!("pass_execute", name = %name).entered();
        encoder.push_debug_group(name);
        {
            let context = PassExecutionContext {
                encoder: &mut encoder,
                resources,
                device,
                queue,
                decl,
                slots: &slots[start..end],
                ops: &ops[start..end],
                configs,
                pass_enabled: active[row],
            };

            passes[row].execute(context)?;
        }
        encoder.pop_debug_group();
    }

    command_buffers.push(encoder.finish());

    Ok(command_buffers)
}

pub fn render_graph_resize_transient_resource<C: 'static>(
    graph: &mut RenderGraph<C>,
    _device: &Device,
    id: ResourceId,
    width: u32,
    height: u32,
) -> Result<()> {
    let already_matches = graph
        .resources
        .get_descriptor(id)
        .and_then(|descriptor| match &descriptor.resource_type {
            ResourceType::TransientColor {
                descriptor: tex_desc,
                ..
            }
            | ResourceType::TransientDepth {
                descriptor: tex_desc,
                ..
            } => Some(tex_desc.width == width && tex_desc.height == height),
            _ => None,
        })
        .unwrap_or(false);
    if already_matches {
        return Ok(());
    }

    graph
        .resources
        .update_transient_descriptor(id, width, height)?;

    drain_aliasing_pool_into_cache(graph);

    graph.aliasing_info = None;
    let external: std::collections::HashSet<ResourceId> = graph
        .resources
        .entries()
        .filter(|(_, descriptor)| descriptor.is_external)
        .map(|(id, _)| id)
        .collect();
    graph.resources.keep_handles(|id| external.contains(&id));
    graph.needs_resource_reallocation = true;
    Ok(())
}

fn texture_still_described<C: 'static>(graph: &RenderGraph<C>, entry: &TextureCacheEntry) -> bool {
    graph
        .resources
        .entries()
        .map(|(_, descriptor)| descriptor)
        .filter(|descriptor| !descriptor.is_external)
        .any(|descriptor| match &descriptor.resource_type {
            ResourceType::TransientColor {
                descriptor: wanted, ..
            }
            | ResourceType::TransientDepth {
                descriptor: wanted, ..
            } => {
                wanted.format == entry.descriptor.format
                    && wanted.width == entry.descriptor.width
                    && wanted.height == entry.descriptor.height
                    && wanted.sample_count == entry.descriptor.sample_count
                    && wanted.mip_level_count == entry.descriptor.mip_level_count
                    && entry.descriptor.usage.contains(wanted.usage)
            }
            _ => false,
        })
}

fn drain_aliasing_pool_into_cache<C: 'static>(graph: &mut RenderGraph<C>) {
    if let Some(aliasing_info) = &mut graph.aliasing_info {
        for pool_slot in &mut aliasing_info.pools {
            if let Some(PooledResource::Texture { texture }) = pool_slot.resource.take()
                && let Some(PoolDescriptorInfo::Texture(desc)) = &pool_slot.descriptor_info
            {
                let bytes = texture_cache_entry_bytes(desc);
                graph.texture_cache.push(TextureCacheEntry {
                    descriptor: desc.clone(),
                    texture,
                    bytes,
                });
            }
        }
    }

    let kept: Vec<bool> = graph
        .texture_cache
        .iter()
        .map(|entry| texture_still_described(graph, entry))
        .collect();
    let mut held: u64 = graph.texture_cache.iter().map(|entry| entry.bytes).sum();
    let mut dropped = vec![false; graph.texture_cache.len()];
    for (index, entry) in graph.texture_cache.iter().enumerate() {
        if held <= TEXTURE_CACHE_BUDGET_BYTES {
            break;
        }
        if kept[index] {
            continue;
        }
        held -= entry.bytes;
        dropped[index] = true;
    }
    if dropped.iter().any(|dropped| *dropped) {
        let mut index = 0;
        graph.texture_cache.retain(|_| {
            let keep = !dropped[index];
            index += 1;
            keep
        });
    }
}

pub fn render_graph_resize_all_transient_textures<C: 'static>(
    graph: &mut RenderGraph<C>,
    resizes: &[(ResourceId, u32, u32)],
) -> Result<()> {
    if resizes.is_empty() {
        return Ok(());
    }

    let mut actually_resized: Vec<(ResourceId, u32, u32)> = Vec::with_capacity(resizes.len());
    for &(id, width, height) in resizes {
        let needs_update = graph
            .resources
            .get_descriptor(id)
            .map(|descriptor| match &descriptor.resource_type {
                ResourceType::TransientColor {
                    descriptor: tex_desc,
                    ..
                }
                | ResourceType::TransientDepth {
                    descriptor: tex_desc,
                    ..
                } => tex_desc.width != width || tex_desc.height != height,
                _ => true,
            })
            .unwrap_or(false);
        if needs_update {
            graph
                .resources
                .update_transient_descriptor(id, width, height)?;
            actually_resized.push((id, width, height));
        }
    }

    if actually_resized.is_empty() {
        return Ok(());
    }

    let resized_ids: std::collections::HashSet<ResourceId> =
        actually_resized.iter().map(|&(id, _, _)| id).collect();

    drain_aliasing_pool_into_cache(graph);

    graph
        .resources
        .keep_handles(|id| !resized_ids.contains(&id));

    graph.aliasing_info = None;
    graph.needs_resource_reallocation = true;

    Ok(())
}

pub fn render_graph_resize_all_transient_proportional<C: 'static>(
    graph: &mut RenderGraph<C>,
    old_width: u32,
    old_height: u32,
    new_width: u32,
    new_height: u32,
) -> Result<()> {
    if old_width == 0 || old_height == 0 {
        return Ok(());
    }

    let resizes: Vec<(ResourceId, u32, u32)> = graph
        .resources
        .entries()
        .filter_map(|(id, descriptor)| {
            if descriptor.is_external || descriptor.fixed_size {
                return None;
            }
            let (current_width, current_height) = match &descriptor.resource_type {
                ResourceType::TransientColor {
                    descriptor: tex_desc,
                    ..
                }
                | ResourceType::TransientDepth {
                    descriptor: tex_desc,
                    ..
                } => (tex_desc.width, tex_desc.height),
                _ => return None,
            };

            let ratio_x = current_width as f64 / old_width as f64;
            let ratio_y = current_height as f64 / old_height as f64;

            if (ratio_x - ratio_y).abs() > 0.01 {
                return None;
            }

            let target_width = (new_width as f64 * ratio_x).round() as u32;
            let target_height = (new_height as f64 * ratio_y).round() as u32;

            Some((id, target_width.max(1), target_height.max(1)))
        })
        .collect();

    render_graph_resize_all_transient_textures(graph, &resizes)
}

impl<C: 'static> Default for RenderGraph<C> {
    fn default() -> Self {
        render_graph_new()
    }
}

#[derive(Debug, Clone)]
struct ResourceLifetime {
    resource_id: ResourceId,
    first_use: usize,
    last_use: usize,
}

pub(super) enum PooledResource {
    Texture { texture: Texture },
    Buffer { buffer: Buffer },
}

#[derive(Clone)]
pub(super) enum PoolDescriptorInfo {
    Texture(RenderGraphTextureDescriptor),
    Buffer(RenderGraphBufferDescriptor),
}

pub struct PoolSlot {
    pub(super) resource: Option<PooledResource>,
    pub(super) descriptor_info: Option<PoolDescriptorInfo>,
    pub(super) lifetime_end: usize,
}

#[derive(Clone)]
struct PoolHeapEntry {
    pool_index: usize,
    lifetime_end: usize,
    descriptor_info: PoolDescriptorInfo,
}

impl PartialEq for PoolHeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.lifetime_end == other.lifetime_end
    }
}

impl Eq for PoolHeapEntry {}

impl PartialOrd for PoolHeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PoolHeapEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.lifetime_end.cmp(&self.lifetime_end)
    }
}

pub struct ResourceAliasingInfo {
    pub aliases: HashMap<ResourceId, usize>,
    pub pools: Vec<PoolSlot>,
}

#[cfg(test)]
mod tests {
    use super::super::pass::PassDecl;
    use super::*;

    struct OrderTestPass {
        decl: PassDecl,
    }

    impl OrderTestPass {
        fn boxed(name: &'static str, full: bool, compose: bool) -> Box<dyn PassNode<()>> {
            Box::new(Self {
                decl: PassDecl {
                    name,
                    full_phase: full,
                    compose_only_phase: compose,
                    ..PassDecl::EMPTY
                },
            })
        }
    }

    impl PassNode<()> for OrderTestPass {
        fn decl(&self) -> &PassDecl {
            &self.decl
        }

        fn execute<'r, 'e>(&mut self, _context: PassExecutionContext<'r, 'e, ()>) -> Result<()> {
            Ok(())
        }
    }

    fn graph_of(passes: &[(&'static str, bool, bool)]) -> RenderGraph<()> {
        let mut graph = render_graph_new::<()>();
        for (name, full, compose) in passes {
            render_graph_add_pass(&mut graph, OrderTestPass::boxed(name, *full, *compose), &[])
                .expect("the test passes bind no slots");
        }
        render_graph_recompile_if_needed(&mut graph).expect("the test graph has no cycle");
        graph
    }

    fn names(graph: &RenderGraph<()>, order: &[NodeIndex]) -> Vec<String> {
        order
            .iter()
            .map(|node_index| graph.decls[node_index.index()].name.to_string())
            .collect()
    }

    #[test]
    fn the_recording_order_follows_the_execution_order() {
        let graph = graph_of(&[("first", true, false), ("second", true, false)]);

        let order = render_graph_recording_order(&graph, ExecutePhase::Full);

        assert_eq!(order, graph.execution_order);
    }

    #[test]
    fn a_culled_pass_is_not_recorded() {
        let mut graph = graph_of(&[("first", true, false), ("second", true, false)]);
        let culled = graph.pass_nodes["first"];
        graph.culled[culled.index()] = true;

        let order = render_graph_recording_order(&graph, ExecutePhase::Full);

        assert_eq!(names(&graph, &order), vec!["second"]);
    }

    #[test]
    fn a_disabled_pass_is_still_recorded() {
        let mut graph = graph_of(&[("first", true, false), ("second", true, false)]);
        let disabled = graph.pass_nodes["first"];
        graph.enabled[disabled.index()] = false;

        let order = render_graph_recording_order(&graph, ExecutePhase::Full);

        assert!(order.contains(&disabled));
        assert_eq!(order.len(), 2);
    }

    #[test]
    fn the_full_phase_drops_a_pass_that_does_not_claim_it() {
        let graph = graph_of(&[("drawn", true, false), ("compose_only", false, true)]);

        let order = render_graph_recording_order(&graph, ExecutePhase::Full);

        assert_eq!(names(&graph, &order), vec!["drawn"]);
    }

    #[test]
    fn the_compose_phase_takes_only_the_passes_that_claim_it() {
        let graph = graph_of(&[("drawn", true, false), ("compose_only", false, true)]);

        let order = render_graph_recording_order(&graph, ExecutePhase::ComposeOnly);

        assert_eq!(names(&graph, &order), vec!["compose_only"]);
    }

    #[test]
    fn an_empty_graph_records_nothing() {
        let graph = graph_of(&[]);

        assert!(render_graph_recording_order(&graph, ExecutePhase::Full).is_empty());
    }
}
