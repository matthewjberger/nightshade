use super::error::{RenderGraphError, Result};
use std::collections::HashMap;
use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, Device, Extent3d, StoreOp, Texture, TextureDescriptor,
    TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttachmentOp {
    pub clear: bool,
    pub store: StoreOp,
}

impl AttachmentOp {
    pub const KEPT: AttachmentOp = AttachmentOp {
        clear: false,
        store: StoreOp::Store,
    };
}

impl Default for AttachmentOp {
    fn default() -> Self {
        AttachmentOp::KEPT
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u32);

impl ResourceId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub const UNBOUND: ResourceId = ResourceId(u32::MAX - 1);
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
    pub(super) descriptors: Vec<Option<ResourceDescriptor>>,
    pub(super) identifiers: HashMap<String, ResourceId>,
    pub(super) handles: Vec<Option<ResourceHandle>>,
    pub(super) versions: Vec<u64>,
}

impl RenderGraphResources {
    pub fn new() -> Self {
        Self {
            descriptors: Vec::new(),
            identifiers: HashMap::new(),
            handles: Vec::new(),
            versions: Vec::new(),
        }
    }

    fn make_room(&mut self) -> ResourceId {
        let id = ResourceId::new(self.descriptors.len() as u32);
        self.descriptors.push(None);
        self.handles.push(None);
        self.versions.push(0);
        id
    }

    fn row(&self, id: ResourceId) -> usize {
        id.0 as usize
    }

    pub fn ids(&self) -> impl Iterator<Item = ResourceId> + '_ {
        self.descriptors
            .iter()
            .enumerate()
            .filter(|(_, descriptor)| descriptor.is_some())
            .map(|(row, _)| ResourceId::new(row as u32))
    }

    pub fn entries(&self) -> impl Iterator<Item = (ResourceId, &ResourceDescriptor)> + '_ {
        self.descriptors
            .iter()
            .enumerate()
            .filter_map(|(row, descriptor)| {
                descriptor
                    .as_ref()
                    .map(|descriptor| (ResourceId::new(row as u32), descriptor))
            })
    }

    pub fn has_handle(&self, id: ResourceId) -> bool {
        self.handles
            .get(self.row(id))
            .is_some_and(|handle| handle.is_some())
    }

    pub(super) fn set_handle(&mut self, id: ResourceId, handle: ResourceHandle) {
        let row = self.row(id);
        self.handles[row] = Some(handle);
    }

    pub(super) fn keep_handles(&mut self, keep: impl Fn(ResourceId) -> bool) {
        for row in 0..self.handles.len() {
            if !keep(ResourceId::new(row as u32)) {
                self.handles[row] = None;
            }
        }
    }

    pub fn identifier(&self, name: &str) -> Option<ResourceId> {
        self.identifiers.get(name).copied()
    }

    pub fn get_version(&self, id: ResourceId) -> u64 {
        self.versions.get(self.row(id)).copied().unwrap_or(0)
    }

    pub(super) fn increment_version(&mut self, id: ResourceId) {
        let row = self.row(id);
        self.versions[row] += 1;
    }

    pub fn register_external_resource(
        &mut self,
        name: String,
        resource_type: ResourceType,
    ) -> ResourceId {
        let id = self.make_room();
        self.identifiers.insert(name.clone(), id);
        let row = self.row(id);
        self.descriptors[row] = Some(ResourceDescriptor {
            name,
            resource_type,
            is_external: true,
            fixed_size: false,
        });
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
        self.set_handle(
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
        self.set_handle(id, ResourceHandle::ExternalBuffer { buffer });
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
        let id = self.make_room();
        self.identifiers.insert(name.clone(), id);
        let row = self.row(id);
        self.descriptors[row] = Some(ResourceDescriptor {
            name,
            resource_type,
            is_external: false,
            fixed_size,
        });
        id
    }

    pub fn get_handle(&self, id: ResourceId) -> Option<&ResourceHandle> {
        self.handles.get(self.row(id))?.as_ref()
    }

    pub fn get_descriptor(&self, id: ResourceId) -> Option<&ResourceDescriptor> {
        self.descriptors.get(self.row(id))?.as_ref()
    }

    pub fn get_color_attachment(
        &self,
        id: ResourceId,
        op: AttachmentOp,
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
            | ResourceType::TransientColor { clear_color, .. } => match clear_color {
                Some(color) if op.clear => wgpu::LoadOp::Clear(*color),
                _ => wgpu::LoadOp::Load,
            },
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

        Ok((handle.view(), load_op, op.store))
    }

    pub fn get_depth_attachment(
        &self,
        id: ResourceId,
        op: AttachmentOp,
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
            | ResourceType::TransientDepth { clear_depth, .. } => match clear_depth {
                Some(depth) if op.clear => wgpu::LoadOp::Clear(*depth),
                _ => wgpu::LoadOp::Load,
            },
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

        Ok((handle.view(), load_op, op.store))
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

        let row = self.row(id);
        self.descriptors[row] = Some(ResourceDescriptor {
            name,
            resource_type: updated_descriptor,
            is_external: false,
            fixed_size,
        });

        Ok(())
    }

    pub fn allocate_transient_resources_with_aliasing(
        &mut self,
        device: &Device,
        aliasing_info: &mut super::graph::ResourceAliasingInfo,
        texture_cache: &mut Vec<super::graph::TextureCacheEntry>,
    ) {
        for (pool_index, pool_slot) in aliasing_info.pools.iter_mut().enumerate() {
            if pool_slot.resource.is_some() {
                continue;
            }

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
                            texture_cache.swap_remove(index).texture
                        } else {
                            let mut texture_descriptor = tex_desc.to_wgpu_descriptor(Some(&label));
                            texture_descriptor.size =
                                crate::renderer::texture_extent::texture_extent(
                                    device,
                                    texture_descriptor.size.width,
                                    texture_descriptor.size.height,
                                    texture_descriptor.size.depth_or_array_layers,
                                );
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

        let mut allocated_resources = Vec::new();

        let standing: Vec<(ResourceId, RenderGraphTextureDescriptor)> = self
            .entries()
            .filter(|(id, descriptor)| {
                !descriptor.is_external && descriptor.fixed_size && !self.has_handle(*id)
            })
            .filter_map(|(id, descriptor)| match &descriptor.resource_type {
                ResourceType::TransientColor {
                    descriptor: tex_desc,
                    ..
                }
                | ResourceType::TransientDepth {
                    descriptor: tex_desc,
                    ..
                } => Some((id, tex_desc.clone())),
                _ => None,
            })
            .collect();
        for (resource_id, tex_desc) in standing {
            let cached_index = texture_cache.iter().position(|entry| {
                entry.descriptor.format == tex_desc.format
                    && entry.descriptor.width == tex_desc.width
                    && entry.descriptor.height == tex_desc.height
                    && entry.descriptor.sample_count == tex_desc.sample_count
                    && entry.descriptor.mip_level_count == tex_desc.mip_level_count
                    && entry.descriptor.usage.contains(tex_desc.usage)
            });
            let texture = if let Some(index) = cached_index {
                texture_cache.swap_remove(index).texture
            } else {
                let label = format!("standing_{}", resource_id.0);
                let mut texture_descriptor = tex_desc.to_wgpu_descriptor(Some(&label));
                texture_descriptor.size = crate::renderer::texture_extent::texture_extent(
                    device,
                    texture_descriptor.size.width,
                    texture_descriptor.size.height,
                    texture_descriptor.size.depth_or_array_layers,
                );
                device.create_texture(&texture_descriptor)
            };
            let view = texture.create_view(&TextureViewDescriptor::default());
            self.set_handle(
                resource_id,
                ResourceHandle::TransientTexture { texture, view },
            );
            allocated_resources.push(resource_id);
        }

        let pooled: Vec<(ResourceId, usize)> = self
            .entries()
            .filter(|(id, descriptor)| !descriptor.is_external && !self.has_handle(*id))
            .filter_map(|(id, _)| aliasing_info.aliases.get(&id).map(|&pool| (id, pool)))
            .collect();

        for (resource_id, pool_index) in pooled {
            let Some(pool_slot) = aliasing_info.pools.get(pool_index) else {
                continue;
            };
            match &pool_slot.resource {
                Some(super::graph::PooledResource::Texture { texture, .. }) => {
                    let view = texture.create_view(&TextureViewDescriptor::default());
                    let texture = texture.clone();
                    self.set_handle(
                        resource_id,
                        ResourceHandle::TransientTexture { texture, view },
                    );
                    allocated_resources.push(resource_id);
                }
                Some(super::graph::PooledResource::Buffer { buffer, .. }) => {
                    let buffer = buffer.clone();
                    self.set_handle(resource_id, ResourceHandle::TransientBuffer { buffer });
                    allocated_resources.push(resource_id);
                }
                None => {}
            }
        }

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
