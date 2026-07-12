use super::error::{RenderGraphError, Result};
use std::collections::HashMap;
use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, Device, Extent3d, StoreOp, Texture, TextureDescriptor,
    TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u32);

impl ResourceId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub const EXPLICIT_DEPENDENCY: ResourceId = ResourceId(u32::MAX);
}

#[derive(Debug, Clone)]
pub struct RenderGraphTextureDescriptor {
    pub format: TextureFormat,
    pub width: u32,
    pub height: u32,
    pub usage: TextureUsages,
    pub sample_count: u32,
    pub mip_level_count: u32,
    pub dimension: wgpu::TextureDimension,
    pub depth_or_array_layers: u32,
}

impl RenderGraphTextureDescriptor {
    pub fn to_wgpu_descriptor<'a>(&self, label: Option<&'a str>) -> TextureDescriptor<'a> {
        TextureDescriptor {
            label,
            size: Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: self.depth_or_array_layers,
            },
            mip_level_count: self.mip_level_count,
            sample_count: self.sample_count,
            dimension: self.dimension,
            format: self.format,
            usage: self.usage,
            view_formats: &[],
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderGraphBufferDescriptor {
    pub size: u64,
    pub usage: BufferUsages,
    pub mapped_at_creation: bool,
}

impl RenderGraphBufferDescriptor {
    pub fn to_wgpu_descriptor<'a>(&self, label: Option<&'a str>) -> BufferDescriptor<'a> {
        BufferDescriptor {
            label,
            size: self.size,
            usage: self.usage,
            mapped_at_creation: self.mapped_at_creation,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResourceType {
    ExternalColor {
        clear_color: Option<wgpu::Color>,
    },
    TransientColor {
        descriptor: RenderGraphTextureDescriptor,
        clear_color: Option<wgpu::Color>,
        force_store: bool,
    },
    ExternalDepth {
        clear_depth: Option<f32>,
    },
    TransientDepth {
        descriptor: RenderGraphTextureDescriptor,
        clear_depth: Option<f32>,
        force_store: bool,
    },
    ExternalBuffer,
    TransientBuffer {
        descriptor: RenderGraphBufferDescriptor,
    },
}

#[derive(Debug, Clone)]
pub struct ResourceDescriptor {
    pub name: String,
    pub resource_type: ResourceType,
    pub is_external: bool,
    pub fixed_size: bool,
}

pub enum ResourceHandle {
    ExternalTexture {
        texture: Option<Texture>,
        view: TextureView,
        width: u32,
        height: u32,
    },
    TransientTexture {
        texture: Texture,
        view: TextureView,
    },
    ExternalBuffer {
        buffer: Buffer,
    },
    TransientBuffer {
        buffer: Buffer,
    },
}

impl ResourceHandle {
    pub fn view(&self) -> &TextureView {
        match self {
            ResourceHandle::ExternalTexture { view, .. } => view,
            ResourceHandle::TransientTexture { view, .. } => view,
            _ => panic!("view() called on buffer resource"),
        }
    }

    pub fn texture(&self) -> Option<&Texture> {
        match self {
            ResourceHandle::ExternalTexture {
                texture: Some(texture),
                ..
            } => Some(texture),
            ResourceHandle::TransientTexture { texture, .. } => Some(texture),
            _ => None,
        }
    }
}

pub struct RenderGraphResources {
    pub(super) descriptors: HashMap<ResourceId, ResourceDescriptor>,
    pub(super) handles: HashMap<ResourceId, ResourceHandle>,
    pub(super) versions: HashMap<ResourceId, u64>,
    pub(super) next_id: u32,
}

impl RenderGraphResources {
    pub fn new() -> Self {
        Self {
            descriptors: HashMap::new(),
            handles: HashMap::new(),
            versions: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn get_version(&self, id: ResourceId) -> u64 {
        *self.versions.get(&id).unwrap_or(&0)
    }

    pub(super) fn increment_version(&mut self, id: ResourceId) {
        let version = self.versions.entry(id).or_insert(0);
        *version += 1;
    }

    pub fn register_external_resource(
        &mut self,
        name: String,
        resource_type: ResourceType,
    ) -> ResourceId {
        let id = ResourceId::new(self.next_id);
        self.next_id += 1;
        self.descriptors.insert(
            id,
            ResourceDescriptor {
                name,
                resource_type,
                is_external: true,
                fixed_size: false,
            },
        );
        id
    }

    pub fn set_external_texture(
        &mut self,
        id: ResourceId,
        texture: Option<Texture>,
        view: TextureView,
        width: u32,
        height: u32,
    ) {
        self.handles.insert(
            id,
            ResourceHandle::ExternalTexture {
                texture,
                view,
                width,
                height,
            },
        );
        self.increment_version(id);
    }

    pub fn set_external_buffer(&mut self, id: ResourceId, buffer: Buffer) {
        self.handles
            .insert(id, ResourceHandle::ExternalBuffer { buffer });
        self.increment_version(id);
    }

    pub fn register_transient_resource(
        &mut self,
        name: String,
        resource_type: ResourceType,
    ) -> ResourceId {
        self.register_transient_resource_opts(name, resource_type, false)
    }

    pub fn register_transient_resource_fixed(
        &mut self,
        name: String,
        resource_type: ResourceType,
    ) -> ResourceId {
        self.register_transient_resource_opts(name, resource_type, true)
    }

    pub(super) fn register_transient_resource_opts(
        &mut self,
        name: String,
        resource_type: ResourceType,
        fixed_size: bool,
    ) -> ResourceId {
        let id = ResourceId::new(self.next_id);
        self.next_id += 1;
        self.descriptors.insert(
            id,
            ResourceDescriptor {
                name,
                resource_type,
                is_external: false,
                fixed_size,
            },
        );
        id
    }

    pub fn get_handle(&self, id: ResourceId) -> Option<&ResourceHandle> {
        self.handles.get(&id)
    }

    pub fn get_descriptor(&self, id: ResourceId) -> Option<&ResourceDescriptor> {
        self.descriptors.get(&id)
    }

    pub fn get_color_attachment(
        &self,
        id: ResourceId,
        node_index: petgraph::graph::NodeIndex,
        clear_ops: &std::collections::HashSet<(petgraph::graph::NodeIndex, ResourceId)>,
        store_ops: &HashMap<(petgraph::graph::NodeIndex, ResourceId), StoreOp>,
    ) -> Result<(&TextureView, wgpu::LoadOp<wgpu::Color>, StoreOp)> {
        let handle = self
            .get_handle(id)
            .ok_or_else(|| RenderGraphError::ResourceNotBound {
                resource: format!("color_attachment_{:?}", id),
                id,
            })?;
        let descriptor =
            self.get_descriptor(id)
                .ok_or_else(|| RenderGraphError::DescriptorNotFound {
                    resource: format!("color_attachment_{:?}", id),
                    id,
                })?;

        let load_op = match &descriptor.resource_type {
            ResourceType::ExternalColor { clear_color, .. }
            | ResourceType::TransientColor { clear_color, .. } => {
                if clear_color.is_some() && clear_ops.contains(&(node_index, id)) {
                    wgpu::LoadOp::Clear(*clear_color.as_ref().unwrap())
                } else {
                    wgpu::LoadOp::Load
                }
            }
            _ => {
                return Err(RenderGraphError::TypeMismatch {
                    operation: "get_color_attachment".to_string(),
                    actual_type: match &descriptor.resource_type {
                        ResourceType::ExternalDepth { .. }
                        | ResourceType::TransientDepth { .. } => "depth".to_string(),
                        ResourceType::ExternalBuffer | ResourceType::TransientBuffer { .. } => {
                            "buffer".to_string()
                        }
                        _ => "unknown".to_string(),
                    },
                    resource: descriptor.name.clone(),
                });
            }
        };

        let store_op = store_ops
            .get(&(node_index, id))
            .copied()
            .unwrap_or(StoreOp::Store);
        Ok((handle.view(), load_op, store_op))
    }

    pub fn get_depth_attachment(
        &self,
        id: ResourceId,
        node_index: petgraph::graph::NodeIndex,
        clear_ops: &std::collections::HashSet<(petgraph::graph::NodeIndex, ResourceId)>,
        store_ops: &HashMap<(petgraph::graph::NodeIndex, ResourceId), StoreOp>,
    ) -> Result<(&TextureView, wgpu::LoadOp<f32>, StoreOp)> {
        let handle = self
            .get_handle(id)
            .ok_or_else(|| RenderGraphError::ResourceNotBound {
                resource: format!("depth_attachment_{:?}", id),
                id,
            })?;
        let descriptor =
            self.get_descriptor(id)
                .ok_or_else(|| RenderGraphError::DescriptorNotFound {
                    resource: format!("depth_attachment_{:?}", id),
                    id,
                })?;

        let load_op = match &descriptor.resource_type {
            ResourceType::ExternalDepth { clear_depth, .. }
            | ResourceType::TransientDepth { clear_depth, .. } => {
                if clear_depth.is_some() && clear_ops.contains(&(node_index, id)) {
                    wgpu::LoadOp::Clear(*clear_depth.as_ref().unwrap())
                } else {
                    wgpu::LoadOp::Load
                }
            }
            _ => {
                return Err(RenderGraphError::TypeMismatch {
                    operation: "get_depth_attachment".to_string(),
                    actual_type: match &descriptor.resource_type {
                        ResourceType::ExternalColor { .. }
                        | ResourceType::TransientColor { .. } => "color".to_string(),
                        ResourceType::ExternalBuffer | ResourceType::TransientBuffer { .. } => {
                            "buffer".to_string()
                        }
                        _ => "unknown".to_string(),
                    },
                    resource: descriptor.name.clone(),
                });
            }
        };

        let store_op = store_ops
            .get(&(node_index, id))
            .copied()
            .unwrap_or(StoreOp::Store);
        Ok((handle.view(), load_op, store_op))
    }

    pub fn get_texture_view(&self, id: ResourceId) -> Option<&TextureView> {
        self.get_handle(id).map(|handle| handle.view())
    }

    pub fn get_texture(&self, id: ResourceId) -> Option<&Texture> {
        self.get_handle(id).and_then(|handle| handle.texture())
    }

    pub fn update_transient_descriptor(
        &mut self,
        id: ResourceId,
        width: u32,
        height: u32,
    ) -> Result<()> {
        let descriptor =
            self.get_descriptor(id)
                .ok_or_else(|| RenderGraphError::ResourceNotFound {
                    resource: format!("resource_{:?}", id),
                    id,
                })?;

        if descriptor.is_external {
            return Err(RenderGraphError::CannotResizeExternal {
                resource: descriptor.name.clone(),
            });
        }

        let name = descriptor.name.clone();
        let fixed_size = descriptor.fixed_size;

        let updated_descriptor = match &descriptor.resource_type {
            ResourceType::TransientColor {
                descriptor: tex_desc,
                clear_color,
                force_store,
            } => ResourceType::TransientColor {
                descriptor: RenderGraphTextureDescriptor {
                    width,
                    height,
                    ..tex_desc.clone()
                },
                clear_color: *clear_color,
                force_store: *force_store,
            },
            ResourceType::TransientDepth {
                descriptor: tex_desc,
                clear_depth,
                force_store,
            } => ResourceType::TransientDepth {
                descriptor: RenderGraphTextureDescriptor {
                    width,
                    height,
                    ..tex_desc.clone()
                },
                clear_depth: *clear_depth,
                force_store: *force_store,
            },
            ResourceType::TransientBuffer { .. } => {
                return Err(RenderGraphError::CannotResizeBuffer {
                    resource: name.clone(),
                });
            }
            _ => {
                return Err(RenderGraphError::CannotResizeNonTransient {
                    resource: name.clone(),
                });
            }
        };

        self.descriptors.insert(
            id,
            ResourceDescriptor {
                name,
                resource_type: updated_descriptor,
                is_external: false,
                fixed_size,
            },
        );

        Ok(())
    }

    pub fn allocate_transient_resources_with_aliasing(
        &mut self,
        device: &Device,
        aliasing_info: &mut super::graph::ResourceAliasingInfo,
        texture_cache: &mut Vec<super::graph::TextureCacheEntry>,
    ) {
        let mut pools_created = 0usize;
        let mut pools_skipped = 0usize;
        let _pool_span = tracing::info_span!("alloc_pools").entered();
        for (pool_index, pool_slot) in aliasing_info.pools.iter_mut().enumerate() {
            if pool_slot.resource.is_some() {
                pools_skipped += 1;
                continue;
            }
            pools_created += 1;

            if let Some(descriptor_info) = &pool_slot.descriptor_info {
                let label = format!("pool_{}", pool_index);

                match descriptor_info {
                    super::graph::PoolDescriptorInfo::Texture(tex_desc) => {
                        let cached_index = texture_cache.iter().position(|entry| {
                            entry.descriptor.format == tex_desc.format
                                && entry.descriptor.width == tex_desc.width
                                && entry.descriptor.height == tex_desc.height
                                && entry.descriptor.sample_count == tex_desc.sample_count
                                && entry.descriptor.mip_level_count == tex_desc.mip_level_count
                                && entry.descriptor.usage.contains(tex_desc.usage)
                        });

                        let texture = if let Some(index) = cached_index {
                            let _span = tracing::info_span!("alloc_pool_cache_hit").entered();
                            texture_cache.swap_remove(index).texture
                        } else {
                            let _span = tracing::info_span!(
                                "alloc_pool_create_texture",
                                w = tex_desc.width,
                                h = tex_desc.height
                            )
                            .entered();
                            let texture_descriptor = tex_desc.to_wgpu_descriptor(Some(&label));
                            device.create_texture(&texture_descriptor)
                        };

                        pool_slot.resource =
                            Some(super::graph::PooledResource::Texture { texture });
                    }
                    super::graph::PoolDescriptorInfo::Buffer(buf_desc) => {
                        let buffer_descriptor = buf_desc.to_wgpu_descriptor(Some(&label));
                        let buffer = device.create_buffer(&buffer_descriptor);
                        pool_slot.resource = Some(super::graph::PooledResource::Buffer { buffer });
                    }
                }
            }
        }

        drop(_pool_span);
        let _ = (pools_created, pools_skipped);

        let mut allocated_resources = Vec::new();
        let mut handles_inserted = 0usize;
        let mut descriptors_skipped = 0usize;
        let _alloc_span = tracing::info_span!("alloc_handles").entered();

        for (resource_id, descriptor) in &self.descriptors {
            if descriptor.is_external || self.handles.contains_key(resource_id) {
                descriptors_skipped += 1;
                continue;
            }
            handles_inserted += 1;

            if let Some(&pool_index) = aliasing_info.aliases.get(resource_id)
                && let Some(pool_slot) = aliasing_info.pools.get(pool_index)
            {
                match &pool_slot.resource {
                    Some(super::graph::PooledResource::Texture { texture, .. }) => {
                        let view = texture.create_view(&TextureViewDescriptor::default());

                        self.handles.insert(
                            *resource_id,
                            ResourceHandle::TransientTexture {
                                texture: texture.clone(),
                                view,
                            },
                        );
                        allocated_resources.push(*resource_id);
                    }
                    Some(super::graph::PooledResource::Buffer { buffer, .. }) => {
                        self.handles.insert(
                            *resource_id,
                            ResourceHandle::TransientBuffer {
                                buffer: buffer.clone(),
                            },
                        );
                        allocated_resources.push(*resource_id);
                    }
                    None => {}
                }
            }
        }

        drop(_alloc_span);
        let _ = (handles_inserted, descriptors_skipped);

        let _bump_span =
            tracing::info_span!("alloc_bump_versions", count = allocated_resources.len()).entered();
        for resource_id in allocated_resources {
            self.increment_version(resource_id);
        }
    }
}

impl Default for RenderGraphResources {
    fn default() -> Self {
        Self::new()
    }
}
