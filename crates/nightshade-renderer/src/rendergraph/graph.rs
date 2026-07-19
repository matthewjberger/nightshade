//! Core render graph implementation.
//!
//! The [`RenderGraph`] orchestrates render pass execution by:
//!
//! 1. Building a dependency graph from pass read/write declarations
//! 2. Topologically sorting passes for correct execution order
//! 3. Computing resource lifetimes for memory aliasing
//! 4. Determining optimal load/store operations
//! 5. Culling passes that don't contribute to final output

use super::error::{RenderGraphError, Result};
use super::pass::{
    BufferBuilder, ColorTextureBuilder, DepthTextureBuilder, ExecutePhase, GraphNode, PassBuilder,
    PassExecutionContext, PassNode, ResourcePool, ResourceTemplate, SlotValue, SubGraphInputSlot,
};
use super::resources::{
    RenderGraphBufferDescriptor, RenderGraphResources, RenderGraphTextureDescriptor, ResourceId,
    ResourceType,
};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::{BinaryHeap, HashMap, HashSet};
use wgpu::{
    Buffer, BufferUsages, CommandBuffer, Device, StoreOp, Texture, TextureFormat, TextureUsages,
    TextureView,
};

/// Directed acyclic graph of render passes with automatic dependency ordering,
/// transient resource aliasing, and store-op computation. `C` is the per-frame
/// config type threaded to each pass at execution.
pub struct RenderGraph<C = ()> {
    pub(super) graph: DiGraph<GraphNode<C>, ResourceId>,
    pub(super) pass_nodes: HashMap<String, NodeIndex>,
    pub(super) pass_resource_mappings: HashMap<String, HashMap<String, ResourceId>>,
    pub(super) sub_graphs: HashMap<String, RenderGraph<C>>,
    pub(super) sub_graph_inputs: HashMap<String, Vec<SubGraphInputSlot>>,
    pub(super) resources: RenderGraphResources,
    pub(super) execution_order: Vec<NodeIndex>,
    pub(super) store_ops: HashMap<(NodeIndex, ResourceId), StoreOp>,
    pub(super) explicit_dependencies: Vec<(NodeIndex, NodeIndex)>,
    pub(super) clear_ops: std::collections::HashSet<(petgraph::graph::NodeIndex, ResourceId)>,
    pub(super) aliasing_info: Option<ResourceAliasingInfo>,
    pub(super) needs_recompile: bool,
    pub(super) needs_resource_reallocation: bool,
    pub(super) culled_passes: std::collections::HashSet<NodeIndex>,
    pub(super) resource_versions: HashMap<ResourceId, u64>,
    pub(super) texture_cache: Vec<TextureCacheEntry>,
    pub(super) texture_cache_clock: u64,
}

/// A transient texture retained after a resize so a later frame of the same
/// size can reuse it instead of reallocating.
pub struct TextureCacheEntry {
    pub(super) descriptor: RenderGraphTextureDescriptor,
    pub(super) texture: Texture,
    pub(super) inserted_at: u64,
}

/// Number of `texture_cache_clock` ticks an unused cache entry survives
/// before it gets evicted on the next push. The clock advances once per
/// transient-pool reset, so an entry that has not been popped after
/// roughly this many resize cycles is presumed stale and dropped to
/// keep the cache bounded.
pub(super) const TEXTURE_CACHE_MAX_AGE: u64 = 120;

/// Creates an empty render graph.
pub fn render_graph_new<C: 'static>() -> RenderGraph<C> {
    RenderGraph {
        graph: DiGraph::new(),
        pass_nodes: HashMap::new(),
        pass_resource_mappings: HashMap::new(),
        sub_graphs: HashMap::new(),
        sub_graph_inputs: HashMap::new(),
        resources: RenderGraphResources::new(),
        execution_order: Vec::new(),
        store_ops: HashMap::new(),
        explicit_dependencies: Vec::new(),
        clear_ops: std::collections::HashSet::new(),
        aliasing_info: None,
        needs_recompile: true,
        needs_resource_reallocation: false,
        culled_passes: std::collections::HashSet::new(),
        resource_versions: HashMap::new(),
        texture_cache: Vec::new(),
        texture_cache_clock: 0,
    }
}

/// Adds a pass, resolving its declared read and write slots against
/// `slot_mappings` to concrete resource ids, and returns the new node index.
/// Errors when a declared slot has no mapping.
pub fn render_graph_add_pass<C: 'static>(
    graph: &mut RenderGraph<C>,
    pass: Box<dyn PassNode<C>>,
    slot_mappings: &[(&str, ResourceId)],
) -> Result<NodeIndex> {
    let name = pass.name().to_string();
    let slot_names_reads = pass.reads();
    let slot_names_writes = pass.writes();
    let slot_names_reads_writes = pass.reads_writes();
    let slot_names_optional_reads = pass.optional_reads();

    let mappings: HashMap<String, ResourceId> = slot_mappings
        .iter()
        .map(|(slot, resource_id)| (slot.to_string(), *resource_id))
        .collect();

    let reads: Vec<ResourceId> = slot_names_reads
        .iter()
        .map(|slot| {
            mappings
                .get(*slot)
                .copied()
                .ok_or_else(|| RenderGraphError::SlotNotMapped {
                    pass: name.clone(),
                    slot: slot.to_string(),
                })
        })
        .collect::<Result<Vec<_>>>()?;

    let writes: Vec<ResourceId> = slot_names_writes
        .iter()
        .map(|slot| {
            mappings
                .get(*slot)
                .copied()
                .ok_or_else(|| RenderGraphError::SlotNotMapped {
                    pass: name.clone(),
                    slot: slot.to_string(),
                })
        })
        .collect::<Result<Vec<_>>>()?;

    let reads_writes: Vec<ResourceId> = slot_names_reads_writes
        .iter()
        .map(|slot| {
            mappings
                .get(*slot)
                .copied()
                .ok_or_else(|| RenderGraphError::SlotNotMapped {
                    pass: name.clone(),
                    slot: slot.to_string(),
                })
        })
        .collect::<Result<Vec<_>>>()?;

    let optional_reads: Vec<ResourceId> = slot_names_optional_reads
        .iter()
        .filter_map(|slot| mappings.get(*slot).copied())
        .collect();

    let graph_node = GraphNode {
        name: name.clone(),
        reads,
        writes,
        reads_writes,
        optional_reads,
        pass,
        enabled: true,
    };

    let index = graph.graph.add_node(graph_node);
    graph.pass_nodes.insert(name.clone(), index);
    graph.pass_resource_mappings.insert(name, mappings);
    graph.needs_recompile = true;
    Ok(index)
}

/// Registers a named sub-graph and its input slots for a parent pass to dispatch.
pub fn render_graph_add_sub_graph<C: 'static>(
    graph: &mut RenderGraph<C>,
    name: String,
    sub_graph: RenderGraph<C>,
    input_slots: Vec<SubGraphInputSlot>,
) {
    graph.sub_graphs.insert(name.clone(), sub_graph);
    graph.sub_graph_inputs.insert(name, input_slots);
}

/// Returns the named sub-graph, or `None` if unregistered.
pub fn render_graph_get_sub_graph<'a, C: 'static>(
    graph: &'a RenderGraph<C>,
    name: &str,
) -> Option<&'a RenderGraph<C>> {
    graph.sub_graphs.get(name)
}

/// Returns a mutable reference to the named sub-graph, or `None` if unregistered.
pub fn render_graph_get_sub_graph_mut<'a, C: 'static>(
    graph: &'a mut RenderGraph<C>,
    name: &str,
) -> Option<&'a mut RenderGraph<C>> {
    graph.sub_graphs.get_mut(name)
}

/// Returns a mutable reference to the named pass as a trait object, or `None`
/// if no pass has that name.
pub fn render_graph_get_pass_mut<'a, C: 'static>(
    graph: &'a mut RenderGraph<C>,
    name: &str,
) -> Option<&'a mut (dyn PassNode<C> + 'static)> {
    let node_index = graph.pass_nodes.get(name)?;
    graph
        .graph
        .node_weight_mut(*node_index)
        .map(move |node| node.pass.as_mut())
}

fn render_graph_set_node_enabled<C: 'static>(
    graph: &mut RenderGraph<C>,
    node_index: NodeIndex,
    enabled: bool,
) -> bool {
    if let Some(node) = graph.graph.node_weight_mut(node_index) {
        let changed = node.enabled != enabled;
        node.enabled = enabled;
        changed
    } else {
        false
    }
}

fn render_graph_refresh_store_ops<C: 'static>(graph: &mut RenderGraph<C>) {
    if graph.execution_order.is_empty() {
        return;
    }
    let execution_order = graph.execution_order.clone();
    graph.store_ops = render_graph_compute_store_ops(graph, &execution_order);
}

/// Enables or disables a single pass by name, refreshing store ops if the flag
/// changed. Errors when no pass has that name.
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
        render_graph_refresh_store_ops(graph);
    }

    Ok(())
}

/// Disables every pass, then applies the given enabled states, recomputing
/// store ops once at the end instead of per toggle. Used by the window UI
/// flows that render with only the UI passes active.
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
        render_graph_refresh_store_ops(graph);
    }
}

/// Adds an explicit ordering edge forcing `before` to run ahead of `after`. A
/// self-dependency is ignored. Errors when either pass name is unknown.
pub fn render_graph_add_dependency<C: 'static>(
    graph: &mut RenderGraph<C>,
    before: &str,
    after: &str,
) -> Result<()> {
    let before_index =
        *graph
            .pass_nodes
            .get(before)
            .ok_or_else(|| RenderGraphError::PassNotFound {
                pass: before.to_string(),
            })?;
    let after_index =
        *graph
            .pass_nodes
            .get(after)
            .ok_or_else(|| RenderGraphError::PassNotFound {
                pass: after.to_string(),
            })?;
    if before_index == after_index {
        return Ok(());
    }
    if !graph
        .explicit_dependencies
        .contains(&(before_index, after_index))
    {
        graph
            .explicit_dependencies
            .push((before_index, after_index));
    }
    if !graph.graph.contains_edge(before_index, after_index) {
        graph
            .graph
            .add_edge(before_index, after_index, ResourceId::EXPLICIT_DEPENDENCY);
    }
    graph.needs_recompile = true;
    Ok(())
}

/// Snapshots the enabled flag of every pass keyed by name.
pub fn render_graph_save_pass_enabled_states<C: 'static>(
    graph: &RenderGraph<C>,
) -> HashMap<String, bool> {
    graph
        .pass_nodes
        .iter()
        .filter_map(|(name, node_index)| {
            graph
                .graph
                .node_weight(*node_index)
                .map(|node| (name.clone(), node.enabled))
        })
        .collect()
}

/// Restores previously saved enabled flags, refreshing store ops if any changed.
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
        render_graph_refresh_store_ops(graph);
    }
}

/// Returns a builder for a color texture resource named `name`.
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

/// Returns a builder for a depth texture resource named `name`.
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

/// Returns a builder for a buffer resource named `name`.
pub fn render_graph_add_buffer<'a, C: 'static>(
    graph: &'a mut RenderGraph<C>,
    name: &str,
) -> BufferBuilder<'a, C> {
    BufferBuilder {
        graph,
        name: name.to_string(),
        descriptor: RenderGraphBufferDescriptor {
            size: 256,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        },
    }
}

/// Registers a transient color resource from a template and returns its id.
pub fn render_graph_transient_color_from_template<C: 'static>(
    graph: &mut RenderGraph<C>,
    name: &str,
    template: &ResourceTemplate,
) -> ResourceId {
    graph.resources.register_transient_resource(
        name.to_string(),
        ResourceType::TransientColor {
            descriptor: RenderGraphTextureDescriptor {
                format: template.format,
                width: template.width,
                height: template.height,
                usage: template.usage,
                sample_count: template.sample_count,
                mip_level_count: template.mip_level_count,
                dimension: template.dimension,
                depth_or_array_layers: template.depth_or_array_layers,
            },
            clear_color: None,
            force_store: false,
        },
    )
}

/// Registers a transient color resource from a template with a clear color and
/// returns its id.
pub fn render_graph_transient_color_from_template_with_clear<C: 'static>(
    graph: &mut RenderGraph<C>,
    name: &str,
    template: &ResourceTemplate,
    clear_color: wgpu::Color,
) -> ResourceId {
    graph.resources.register_transient_resource(
        name.to_string(),
        ResourceType::TransientColor {
            descriptor: RenderGraphTextureDescriptor {
                format: template.format,
                width: template.width,
                height: template.height,
                usage: template.usage,
                sample_count: template.sample_count,
                mip_level_count: template.mip_level_count,
                dimension: template.dimension,
                depth_or_array_layers: template.depth_or_array_layers,
            },
            clear_color: Some(clear_color),
            force_store: false,
        },
    )
}

/// Registers an external color resource, supplied by the caller each frame, and
/// returns its id.
pub fn render_graph_external_color<C: 'static>(
    graph: &mut RenderGraph<C>,
    name: &str,
) -> ResourceId {
    graph.resources.register_external_resource(
        name.to_string(),
        ResourceType::ExternalColor { clear_color: None },
    )
}

/// Returns a builder that adds `pass` and binds its slots fluently.
pub fn render_graph_pass<'a, C: 'static>(
    graph: &'a mut RenderGraph<C>,
    pass: Box<dyn PassNode<C>>,
) -> PassBuilder<'a, C> {
    PassBuilder {
        graph,
        pass: Some(pass),
        slots: Vec::new(),
    }
}

/// Returns a resource pool that hands out transient resources sharing `template`.
pub fn render_graph_resource_pool<'a, C: 'static>(
    graph: &'a mut RenderGraph<C>,
    template: &ResourceTemplate,
) -> ResourcePool<'a, C> {
    ResourcePool {
        graph,
        template: template.clone(),
    }
}

/// Supplies the backing texture and view for an external resource this frame.
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

/// Supplies the backing buffer for an external resource this frame.
pub fn render_graph_set_external_buffer<C: 'static>(
    graph: &mut RenderGraph<C>,
    id: ResourceId,
    buffer: Buffer,
) {
    graph.resources.set_external_buffer(id, buffer);
}

/// Returns the aliased pool texture backing `id`, or `None` when unallocated.
pub fn render_graph_get_texture<C: 'static>(
    graph: &RenderGraph<C>,
    id: ResourceId,
) -> Option<&Texture> {
    let aliasing_info = graph.aliasing_info.as_ref()?;
    let pool_index = aliasing_info.aliases.get(&id)?;
    let pool_slot = aliasing_info.pools.get(*pool_index)?;
    match &pool_slot.resource {
        Some(PooledResource::Texture { texture }) => Some(texture),
        _ => None,
    }
}

/// Returns the texture view for `id`, or `None` when unallocated.
pub fn render_graph_get_texture_view<C: 'static>(
    graph: &RenderGraph<C>,
    id: ResourceId,
) -> Option<&TextureView> {
    graph.resources.get_texture_view(id)
}

fn render_graph_build_dependency_edges<C: 'static>(graph: &mut RenderGraph<C>) {
    let mut resource_writers: HashMap<ResourceId, Vec<NodeIndex>> = HashMap::new();

    let node_indices: Vec<NodeIndex> = graph.graph.node_indices().collect();
    let mut edges_to_add: Vec<(NodeIndex, NodeIndex, ResourceId)> = Vec::new();

    for &node_index in &node_indices {
        let node = &graph.graph[node_index];
        let reads = node.reads.clone();
        let writes = node.writes.clone();
        let reads_writes = node.reads_writes.clone();
        let optional_reads = node.optional_reads.clone();

        // A consumer must run after every producer of the resource it reads, not
        // only the most recent one. A resource with several producers (such as a
        // gated temporal resolve and its passthrough sharing one output) would
        // otherwise leave the earlier producers without an outgoing edge, letting
        // the topological sort schedule them after their own consumer.
        for &read_resource in reads.iter().chain(&optional_reads).chain(&reads_writes) {
            if let Some(writers) = resource_writers.get(&read_resource) {
                for &writer_index in writers {
                    edges_to_add.push((writer_index, node_index, read_resource));
                }
            }
        }

        // Order producers of the same resource against each other so their
        // relative order is stable no matter which producers are enabled.
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

    let explicit_dependencies = graph.explicit_dependencies.clone();
    for (before_index, after_index) in explicit_dependencies {
        if !graph.graph.contains_edge(before_index, after_index) {
            graph
                .graph
                .add_edge(before_index, after_index, ResourceId::EXPLICIT_DEPENDENCY);
        }
    }
}

fn render_graph_compute_resource_lifetimes<C: 'static>(
    graph: &RenderGraph<C>,
    execution_order: &[NodeIndex],
) -> Vec<ResourceLifetime> {
    let mut lifetimes: HashMap<ResourceId, ResourceLifetime> = HashMap::new();

    for (pass_index, &node_index) in execution_order.iter().enumerate() {
        let node = &graph.graph[node_index];

        for &resource_id in &node.writes {
            lifetimes.entry(resource_id).or_insert(ResourceLifetime {
                resource_id,
                first_use: pass_index,
                last_use: pass_index,
            });
        }

        for &resource_id in &node.reads {
            let lifetime = lifetimes.entry(resource_id).or_insert(ResourceLifetime {
                resource_id,
                first_use: pass_index,
                last_use: pass_index,
            });
            lifetime.last_use = pass_index;
        }

        for &resource_id in &node.reads_writes {
            let lifetime = lifetimes.entry(resource_id).or_insert(ResourceLifetime {
                resource_id,
                first_use: pass_index,
                last_use: pass_index,
            });
            lifetime.last_use = pass_index;
        }

        for &resource_id in &node.optional_reads {
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

/// Computes per pass and resource store ops from the reads of enabled passes.
/// Reads from culled passes still count on purpose: the pick keepalive pass is
/// dead-pass culled, yet its declared reads are what keep `entity_id` and
/// `depth` stored for the pick compute that samples them outside the graph.
/// Only the enabled flag may exclude a pass from this analysis.
/// External resources are always stored because their consumers are invisible
/// to this analysis: surface presentation, cached viewport preloads in a later
/// graph execution, and shadow atlases reused across frames. Backends like
/// Metal honor `StoreOp::Discard` strictly, so discarding an external write
/// drops the contents instead of merely hinting.
fn render_graph_compute_store_ops<C: 'static>(
    graph: &RenderGraph<C>,
    execution_order: &[NodeIndex],
) -> HashMap<(NodeIndex, ResourceId), StoreOp> {
    let mut last_read: HashMap<ResourceId, usize> = HashMap::new();

    for (index, &node_index) in execution_order.iter().enumerate().rev() {
        let node = &graph.graph[node_index];
        if !node.enabled {
            continue;
        }

        for &resource_id in node
            .reads
            .iter()
            .chain(&node.reads_writes)
            .chain(&node.optional_reads)
        {
            last_read.entry(resource_id).or_insert(index);
        }
    }

    let mut store_ops = HashMap::new();

    for (index, &node_index) in execution_order.iter().enumerate() {
        let node = &graph.graph[node_index];
        if !node.enabled {
            continue;
        }

        for &resource_id in node.writes.iter().chain(&node.reads_writes) {
            let descriptor = graph.resources.get_descriptor(resource_id).unwrap();

            let force_store = match &descriptor.resource_type {
                ResourceType::ExternalColor { .. } | ResourceType::ExternalDepth { .. } => true,
                ResourceType::TransientColor { force_store, .. }
                | ResourceType::TransientDepth { force_store, .. } => *force_store,
                _ => false,
            };
            let has_later_read = last_read
                .get(&resource_id)
                .is_some_and(|&last| last > index);

            let store_op = match &descriptor.resource_type {
                ResourceType::ExternalBuffer | ResourceType::TransientBuffer { .. } => {
                    StoreOp::Store
                }
                _ => {
                    if force_store || has_later_read {
                        StoreOp::Store
                    } else {
                        StoreOp::Discard
                    }
                }
            };

            store_ops.insert((node_index, resource_id), store_op);
        }
    }

    store_ops
}

fn render_graph_compute_clear_ops<C: 'static>(
    graph: &RenderGraph<C>,
    execution_order: &[NodeIndex],
) -> std::collections::HashSet<(NodeIndex, ResourceId)> {
    let mut clear_ops = std::collections::HashSet::new();
    let mut first_write: HashMap<ResourceId, NodeIndex> = HashMap::new();

    for &node_index in execution_order {
        let node = &graph.graph[node_index];

        for &resource_id in &node.writes {
            first_write.entry(resource_id).or_insert(node_index);
        }
        for &resource_id in &node.reads_writes {
            first_write.entry(resource_id).or_insert(node_index);
        }
    }

    for (&resource_id, &node_index) in &first_write {
        if let Some(descriptor) = graph.resources.get_descriptor(resource_id) {
            let has_clear = match &descriptor.resource_type {
                ResourceType::ExternalColor { clear_color, .. }
                | ResourceType::TransientColor { clear_color, .. } => clear_color.is_some(),
                ResourceType::ExternalDepth { clear_depth, .. }
                | ResourceType::TransientDepth { clear_depth, .. } => clear_depth.is_some(),
                _ => false,
            };

            if has_clear {
                clear_ops.insert((node_index, resource_id));
            }
        }
    }

    clear_ops
}

fn render_graph_compute_dead_passes<C: 'static>(
    graph: &RenderGraph<C>,
    execution_order: &[NodeIndex],
) -> HashSet<NodeIndex> {
    let mut required_resources: HashSet<ResourceId> = HashSet::new();
    let mut required_passes: HashSet<NodeIndex> = HashSet::new();

    for &resource_id in graph.resources.descriptors.keys() {
        let descriptor = graph.resources.get_descriptor(resource_id).unwrap();
        if descriptor.is_external {
            required_resources.insert(resource_id);
        }
    }

    for &node_index in execution_order.iter().rev() {
        let node = &graph.graph[node_index];

        let has_side_effects = node.writes.is_empty() && node.reads_writes.is_empty();

        let writes_required_resource = node.writes.iter().any(|r| required_resources.contains(r))
            || node
                .reads_writes
                .iter()
                .any(|r| required_resources.contains(r));

        if writes_required_resource || has_side_effects {
            required_passes.insert(node_index);
            required_resources.extend(&node.reads);
            required_resources.extend(&node.reads_writes);
            required_resources.extend(&node.optional_reads);
        }
    }

    let all_passes: HashSet<NodeIndex> = execution_order.iter().copied().collect();
    all_passes.difference(&required_passes).copied().collect()
}

/// Builds dependency edges, topologically sorts the passes, and computes store
/// ops, clear ops, resource aliasing, and dead-pass culling. Errors on a cyclic
/// dependency.
pub fn render_graph_compile<C: 'static>(graph: &mut RenderGraph<C>) -> Result<()> {
    render_graph_build_dependency_edges(graph);

    graph.execution_order = petgraph::algo::toposort(&graph.graph, None).map_err(|cycle| {
        let pass = graph
            .graph
            .node_weight(cycle.node_id())
            .map(|node| node.name.clone())
            .unwrap_or_else(|| "<unknown>".to_string());
        RenderGraphError::CyclicDependency { pass }
    })?;

    graph.store_ops = render_graph_compute_store_ops(graph, &graph.execution_order.clone());
    graph.clear_ops = render_graph_compute_clear_ops(graph, &graph.execution_order.clone());

    let lifetimes = render_graph_compute_resource_lifetimes(graph, &graph.execution_order.clone());
    graph.aliasing_info = Some(render_graph_compute_resource_aliasing(graph, lifetimes));

    graph.culled_passes = render_graph_compute_dead_passes(graph, &graph.execution_order.clone());

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
            .map(|node| node.name.clone())
            .unwrap_or_else(|| "<unknown>".to_string());
        RenderGraphError::CyclicDependency { pass }
    })?;

    graph.store_ops = render_graph_compute_store_ops(graph, &graph.execution_order.clone());
    graph.clear_ops = render_graph_compute_clear_ops(graph, &graph.execution_order.clone());

    let lifetimes = render_graph_compute_resource_lifetimes(graph, &graph.execution_order.clone());
    graph.aliasing_info = Some(render_graph_compute_resource_aliasing(graph, lifetimes));

    graph.culled_passes = render_graph_compute_dead_passes(graph, &graph.execution_order.clone());

    graph.needs_recompile = false;
    Ok(())
}

/// Recompiles if needed, allocates transient resources, and runs every enabled
/// pass, returning the submitted command buffers. Runs the full phase.
pub fn render_graph_execute<C: 'static>(
    graph: &mut RenderGraph<C>,
    device: &Device,
    queue: &wgpu::Queue,
    configs: &C,
) -> Result<Vec<CommandBuffer>> {
    render_graph_execute_with_phase(graph, device, queue, configs, ExecutePhase::Full)
}

/// Like [`render_graph_execute`] but restricted to the passes that run in `phase`.
pub fn render_graph_execute_with_phase<C: 'static>(
    graph: &mut RenderGraph<C>,
    device: &Device,
    queue: &wgpu::Queue,
    configs: &C,
    phase: ExecutePhase,
) -> Result<Vec<CommandBuffer>> {
    {
        let _span = tracing::info_span!("graph_recompile_if_needed").entered();
        render_graph_recompile_if_needed(graph)?;
    }

    if graph.aliasing_info.is_none() {
        let _span = tracing::info_span!("graph_compute_aliasing").entered();
        let lifetimes =
            render_graph_compute_resource_lifetimes(graph, &graph.execution_order.clone());
        graph.aliasing_info = Some(render_graph_compute_resource_aliasing(graph, lifetimes));
    }

    {
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
        let _span = tracing::info_span!("graph_invalidate_bind_groups").entered();
        render_graph_invalidate_bind_groups_for_changed_resources(graph);
    }

    {
        let _span = tracing::info_span!("graph_prepare").entered();
        for &node_index in &graph.execution_order {
            if graph.culled_passes.contains(&node_index) {
                continue;
            }

            let node = &mut graph.graph[node_index];
            if !node.enabled {
                continue;
            }
            match phase {
                ExecutePhase::Full => {
                    if !node.pass.runs_in_full_phase() {
                        continue;
                    }
                }
                ExecutePhase::ComposeOnly => {
                    if !node.pass.runs_in_compose_only_phase() {
                        continue;
                    }
                }
            }
            let pass_name = node.name.clone();
            let _pass_span = tracing::info_span!("pass_prepare", name = %pass_name).entered();
            node.pass.prepare(device, queue, configs);
        }
    }

    let _span = tracing::info_span!("graph_execute").entered();
    render_graph_execute_serial(graph, device, queue, configs, phase)
}

fn render_graph_invalidate_bind_groups_for_changed_resources<C: 'static>(
    graph: &mut RenderGraph<C>,
) {
    let mut dirty_resources = HashSet::new();

    for &resource_id in graph.resources.descriptors.keys() {
        let current_version = graph.resources.get_version(resource_id);
        let stored_version = graph
            .resource_versions
            .get(&resource_id)
            .copied()
            .unwrap_or(0);

        if current_version != stored_version {
            dirty_resources.insert(resource_id);
            graph.resource_versions.insert(resource_id, current_version);
        }
    }

    if dirty_resources.is_empty() {
        return;
    }

    let mut passes_to_invalidate = HashSet::new();

    for &node_index in &graph.execution_order {
        let node = &graph.graph[node_index];

        for &resource_id in node
            .reads
            .iter()
            .chain(&node.writes)
            .chain(&node.reads_writes)
            .chain(&node.optional_reads)
        {
            if dirty_resources.contains(&resource_id) {
                passes_to_invalidate.insert(node_index);
                break;
            }
        }
    }

    for &node_index in &passes_to_invalidate {
        let node = &mut graph.graph[node_index];
        node.pass.invalidate_bind_groups();
    }
}

/// Executes the sorted passes into a single command encoder so the whole
/// graph submits as one command buffer. Metal pays a real cost per command
/// buffer, and one buffer per pass multiplied that by the pass count every
/// frame. The encoder is split only when a pass dispatches a sub-graph,
/// which must submit between the surrounding commands. Each pass's commands
/// are wrapped in a debug group so GPU captures keep per-pass attribution.
fn render_graph_execute_serial<C: 'static>(
    graph: &mut RenderGraph<C>,
    device: &Device,
    queue: &wgpu::Queue,
    configs: &C,
    phase: ExecutePhase,
) -> Result<Vec<CommandBuffer>> {
    let mut command_buffers = Vec::new();
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("RenderGraph Encoder"),
    });

    let execution_order = graph.execution_order.clone();
    for node_index in execution_order {
        if graph.culled_passes.contains(&node_index) {
            continue;
        }

        let is_enabled = graph.graph[node_index].enabled;
        match phase {
            ExecutePhase::Full => {
                if !graph.graph[node_index].pass.runs_in_full_phase() {
                    continue;
                }
            }
            ExecutePhase::ComposeOnly => {
                if !graph.graph[node_index].pass.runs_in_compose_only_phase() {
                    continue;
                }
            }
        }
        let pass_name = graph.graph[node_index].name.clone();
        let _pass_span = tracing::info_span!("pass_execute", name = %pass_name).entered();
        let node = &mut graph.graph[node_index];

        let slot_mappings = graph
            .pass_resource_mappings
            .get(&node.name)
            .ok_or_else(|| RenderGraphError::ResourceNotFound {
                resource: format!("pass_{}_mappings", node.name),
                id: ResourceId(0),
            })?;

        encoder.push_debug_group(&pass_name);
        let sub_graph_commands = {
            let context = PassExecutionContext {
                encoder: &mut encoder,
                resources: &graph.resources,
                device,
                queue,
                slot_mappings,
                configs,
                sub_graph_commands: Vec::new(),
                node_index,
                clear_ops: &graph.clear_ops,
                store_ops: &graph.store_ops,
                pass_enabled: is_enabled,
            };

            node.pass.execute(context)?
        };
        encoder.pop_debug_group();

        for command in sub_graph_commands {
            command_buffers.push(encoder.finish());

            let sub_graph = graph
                .sub_graphs
                .get_mut(&command.sub_graph_name)
                .ok_or_else(|| RenderGraphError::SubGraphNotFound {
                    sub_graph: command.sub_graph_name.clone(),
                })?;

            let input_slots = graph
                .sub_graph_inputs
                .get(&command.sub_graph_name)
                .cloned()
                .unwrap_or_default();

            for (index, slot_value) in command.inputs.iter().enumerate() {
                if let Some(input_slot) = input_slots.get(index) {
                    match slot_value {
                        SlotValue::TextureView {
                            texture,
                            view,
                            width,
                            height,
                        } => {
                            if let Some(resource_id) = sub_graph
                                .resources
                                .descriptors
                                .iter()
                                .find(|(_, desc)| desc.name == input_slot.name && desc.is_external)
                                .map(|(id, _)| *id)
                            {
                                let descriptor = sub_graph
                                    .resources
                                    .get_descriptor(resource_id)
                                    .ok_or_else(|| RenderGraphError::DescriptorNotFound {
                                        resource: input_slot.name.clone(),
                                        id: resource_id,
                                    })?;
                                match &descriptor.resource_type {
                                    ResourceType::ExternalColor { .. }
                                    | ResourceType::ExternalDepth { .. } => {
                                        sub_graph.resources.set_external_texture(
                                            resource_id,
                                            texture.cloned(),
                                            (*view).clone(),
                                            *width,
                                            *height,
                                        );
                                    }
                                    _ => {
                                        return Err(RenderGraphError::SubGraphInputTypeMismatch {
                                            input: input_slot.name.clone(),
                                            expected: "buffer".to_string(),
                                            received: "texture".to_string(),
                                        });
                                    }
                                }
                            }
                        }
                        SlotValue::Buffer(buffer) => {
                            if let Some(resource_id) = sub_graph
                                .resources
                                .descriptors
                                .iter()
                                .find(|(_, desc)| desc.name == input_slot.name && desc.is_external)
                                .map(|(id, _)| *id)
                            {
                                let descriptor = sub_graph
                                    .resources
                                    .get_descriptor(resource_id)
                                    .ok_or_else(|| RenderGraphError::DescriptorNotFound {
                                        resource: input_slot.name.clone(),
                                        id: resource_id,
                                    })?;
                                match &descriptor.resource_type {
                                    ResourceType::ExternalBuffer => {
                                        sub_graph
                                            .resources
                                            .set_external_buffer(resource_id, (*buffer).clone());
                                    }
                                    _ => {
                                        return Err(RenderGraphError::SubGraphInputTypeMismatch {
                                            input: input_slot.name.clone(),
                                            expected: "texture".to_string(),
                                            received: "buffer".to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let sub_graph_buffers = render_graph_execute(sub_graph, device, queue, configs)?;
            command_buffers.extend(sub_graph_buffers);

            encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("RenderGraph Encoder"),
            });
        }
    }

    command_buffers.push(encoder.finish());

    Ok(command_buffers)
}

/// Returns a mutable reference to the graph's resource table.
pub fn render_graph_resources_mut<C: 'static>(
    graph: &mut RenderGraph<C>,
) -> &mut RenderGraphResources {
    &mut graph.resources
}

/// Resizes one transient resource to `width` by `height`, deferring
/// reallocation to the next execute. A no-op when the size already matches.
pub fn render_graph_resize_transient_resource<C: 'static>(
    graph: &mut RenderGraph<C>,
    _device: &Device,
    id: ResourceId,
    width: u32,
    height: u32,
) -> Result<()> {
    let already_matches = graph
        .resources
        .descriptors
        .get(&id)
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
    graph.resources.handles.retain(|id, _| {
        if let Some(descriptor) = graph.resources.descriptors.get(id) {
            descriptor.is_external
        } else {
            false
        }
    });
    graph.needs_resource_reallocation = true;
    Ok(())
}

fn drain_aliasing_pool_into_cache<C: 'static>(graph: &mut RenderGraph<C>) {
    graph.texture_cache_clock = graph.texture_cache_clock.wrapping_add(1);
    let now = graph.texture_cache_clock;

    if let Some(aliasing_info) = &mut graph.aliasing_info {
        for pool_slot in &mut aliasing_info.pools {
            if let Some(PooledResource::Texture { texture }) = pool_slot.resource.take()
                && let Some(PoolDescriptorInfo::Texture(desc)) = &pool_slot.descriptor_info
            {
                graph.texture_cache.push(TextureCacheEntry {
                    descriptor: desc.clone(),
                    texture,
                    inserted_at: now,
                });
            }
        }
    }

    graph
        .texture_cache
        .retain(|entry| now.saturating_sub(entry.inserted_at) <= TEXTURE_CACHE_MAX_AGE);
}

/// Resizes every non-fixed transient texture that tracked the old dimensions so
/// it keeps the same fraction of the new dimensions.
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
        .descriptors
        .iter()
        .filter_map(|(&id, descriptor)| {
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

/// Resizes the listed transient textures to their given sizes, deferring
/// reallocation to the next execute. Sizes that already match are skipped.
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
            .descriptors
            .get(&id)
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
        .handles
        .retain(|id, _| !resized_ids.contains(id));

    graph.aliasing_info = None;
    graph.needs_resource_reallocation = true;

    Ok(())
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

/// One slot in the aliased resource pool, holding a possibly shared texture or
/// buffer and the pass index at which its current lifetime ends.
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

/// The result of resource-lifetime aliasing: which pool slot backs each resource.
pub struct ResourceAliasingInfo {
    /// Maps each resource id to the index of its assigned pool slot.
    pub aliases: HashMap<ResourceId, usize>,
    /// The pool slots holding the shared backing resources.
    pub pools: Vec<PoolSlot>,
}
