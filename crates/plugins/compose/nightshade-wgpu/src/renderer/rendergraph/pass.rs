use super::error::{RenderGraphError, Result};
use super::resources::{
    AttachmentOp, RenderGraphResources, RenderGraphTextureDescriptor, ResourceHandle, ResourceId,
    ResourceType,
};
use wgpu::{CommandEncoder, Device, TextureFormat, TextureUsages};

pub struct PassExecutionContext<'r, 'e, C = ()> {
    pub encoder: &'e mut CommandEncoder,
    pub resources: &'r RenderGraphResources,
    pub device: &'r Device,
    pub queue: &'r wgpu::Queue,
    pub(super) decl: &'r PassDecl,
    pub(super) slots: &'r [ResourceId],
    pub(super) ops: &'r [AttachmentOp],
    pub configs: &'r C,
    pub(super) pass_enabled: bool,
}

impl<'r, 'e, C> PassExecutionContext<'r, 'e, C> {
    pub fn is_pass_enabled(&self) -> bool {
        self.pass_enabled
    }

    fn bound(&self, slot: &str) -> Result<(usize, ResourceId)> {
        let missing = || RenderGraphError::SlotNotFound {
            slot: slot.to_string(),
            pass: self.decl.name.to_string(),
        };
        let position = self.decl.position(slot).ok_or_else(missing)?;
        let resource = self.slots.get(position).copied().ok_or_else(missing)?;
        if resource == ResourceId::UNBOUND {
            return Err(missing());
        }
        Ok((position, resource))
    }

    pub fn get_slot(&self, slot: &str) -> Result<ResourceId> {
        self.bound(slot).map(|(_, resource)| resource)
    }

    pub fn get_texture_view(&self, slot: &str) -> Result<&'r wgpu::TextureView> {
        let resource_id = self.get_slot(slot)?;
        self.resources.get_texture_view(resource_id).ok_or_else(|| {
            RenderGraphError::ResourceNotBound {
                resource: slot.to_string(),
                id: resource_id,
            }
        })
    }

    pub fn get_texture(&self, slot: &str) -> Result<&'r wgpu::Texture> {
        let resource_id = self.get_slot(slot)?;
        self.resources
            .get_texture(resource_id)
            .ok_or_else(|| RenderGraphError::ResourceNotBound {
                resource: slot.to_string(),
                id: resource_id,
            })
    }

    pub fn get_color_attachment(
        &self,
        slot: &str,
    ) -> Result<(
        &'r wgpu::TextureView,
        wgpu::LoadOp<wgpu::Color>,
        wgpu::StoreOp,
    )> {
        let (position, resource_id) = self.bound(slot)?;
        let op = self.ops.get(position).copied().unwrap_or_default();
        self.resources.get_color_attachment(resource_id, op)
    }

    pub fn get_depth_attachment(
        &self,
        slot: &str,
    ) -> Result<(&'r wgpu::TextureView, wgpu::LoadOp<f32>, wgpu::StoreOp)> {
        let (position, resource_id) = self.bound(slot)?;
        let op = self.ops.get(position).copied().unwrap_or_default();
        self.resources.get_depth_attachment(resource_id, op)
    }

    pub fn get_buffer(&self, slot: &str) -> Result<&'r wgpu::Buffer> {
        let resource_id = self.get_slot(slot)?;
        let handle = self.resources.get_handle(resource_id).ok_or_else(|| {
            RenderGraphError::ResourceNotBound {
                resource: slot.to_string(),
                id: resource_id,
            }
        })?;

        match handle {
            ResourceHandle::ExternalBuffer { buffer }
            | ResourceHandle::TransientBuffer { buffer } => Ok(buffer),
            _ => Err(RenderGraphError::TypeMismatch {
                operation: "get_buffer".to_string(),
                actual_type: "texture".to_string(),
                resource: slot.to_string(),
            }),
        }
    }

    pub fn get_texture_size(&self, slot: &str) -> Result<(u32, u32)> {
        let resource_id = self.get_slot(slot)?;
        let descriptor = self.resources.get_descriptor(resource_id).ok_or_else(|| {
            RenderGraphError::DescriptorNotFound {
                resource: slot.to_string(),
                id: resource_id,
            }
        })?;

        match &descriptor.resource_type {
            ResourceType::TransientColor {
                descriptor: texture_desc,
                ..
            }
            | ResourceType::TransientDepth {
                descriptor: texture_desc,
                ..
            } => Ok((texture_desc.width, texture_desc.height)),
            ResourceType::ExternalColor { .. } | ResourceType::ExternalDepth { .. } => {
                let handle = self.resources.get_handle(resource_id).ok_or_else(|| {
                    RenderGraphError::ResourceNotBound {
                        resource: slot.to_string(),
                        id: resource_id,
                    }
                })?;

                match handle {
                    ResourceHandle::ExternalTexture { width, height, .. } => Ok((*width, *height)),
                    _ => Err(RenderGraphError::TypeMismatch {
                        operation: "get_texture_size".to_string(),
                        actual_type: "transient_texture".to_string(),
                        resource: slot.to_string(),
                    }),
                }
            }
            _ => Err(RenderGraphError::TypeMismatch {
                operation: "get_texture_size".to_string(),
                actual_type: "buffer".to_string(),
                resource: slot.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PassDecl {
    pub name: &'static str,
    pub reads: &'static [&'static str],
    pub writes: &'static [&'static str],
    pub reads_writes: &'static [&'static str],
    pub optional_reads: &'static [&'static str],
    pub stage: PassStage,
    pub position: PassPosition,
    pub full_phase: bool,
    pub compose_only_phase: bool,
}

impl PassDecl {
    pub const EMPTY: PassDecl = PassDecl {
        name: "",
        reads: &[],
        writes: &[],
        reads_writes: &[],
        optional_reads: &[],
        stage: PassStage::Scene,
        position: PassPosition::Last,
        full_phase: true,
        compose_only_phase: false,
    };

    pub const fn named(name: &'static str) -> PassDecl {
        PassDecl {
            name,
            ..PassDecl::EMPTY
        }
    }

    pub fn slot_count(&self) -> usize {
        self.reads.len() + self.writes.len() + self.reads_writes.len() + self.optional_reads.len()
    }

    pub fn reads_at(&self) -> std::ops::Range<usize> {
        0..self.reads.len()
    }

    pub fn writes_at(&self) -> std::ops::Range<usize> {
        let start = self.reads.len();
        start..start + self.writes.len()
    }

    pub fn reads_writes_at(&self) -> std::ops::Range<usize> {
        let start = self.reads.len() + self.writes.len();
        start..start + self.reads_writes.len()
    }

    pub fn optional_reads_at(&self) -> std::ops::Range<usize> {
        let start = self.reads.len() + self.writes.len() + self.reads_writes.len();
        start..start + self.optional_reads.len()
    }

    pub fn position(&self, slot: &str) -> Option<usize> {
        let mut base = 0;
        for names in [
            self.reads,
            self.writes,
            self.reads_writes,
            self.optional_reads,
        ] {
            if let Some(found) = names.iter().position(|name| *name == slot) {
                return Some(base + found);
            }
            base += names.len();
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExecutePhase {
    #[default]
    Full,
    ComposeOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum PassPosition {
    Background,
    Geometry,
    LitOverlay,
    SceneOverlay,
    SceneEnd,
    PostProcess,
    Present,
    Swapchain,
    #[default]
    Last,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PassStage {
    #[default]
    Scene,
    Swapchain,
}

pub enum MaterialUpdate<'u> {
    Register {
        name: &'u str,
        view: &'u wgpu::TextureView,
        sampler: &'u wgpu::Sampler,
        mapping: Option<&'u dyn std::any::Any>,
    },
    Unregister {
        name: &'u str,
    },
    Mapping {
        mapping: &'u dyn std::any::Any,
    },
    Rebind {
        arrays: &'u dyn std::any::Any,
    },
}

pub struct ResourcePublisher<'r> {
    pub(super) resources: &'r mut RenderGraphResources,
}

impl ResourcePublisher<'_> {
    pub fn texture(
        &mut self,
        name: &str,
        texture: Option<wgpu::Texture>,
        view: wgpu::TextureView,
        width: u32,
        height: u32,
    ) {
        let Some(id) = self.resources.identifier(name) else {
            return;
        };
        self.resources
            .set_external_texture(id, texture, view, width, height);
    }

    pub fn buffer(&mut self, name: &str, buffer: wgpu::Buffer) {
        let Some(id) = self.resources.identifier(name) else {
            return;
        };
        self.resources.set_external_buffer(id, buffer);
    }
}

pub struct ResourceReader<'r> {
    pub(super) resources: &'r RenderGraphResources,
}

impl ResourceReader<'_> {
    pub fn texture_view(&self, name: &str) -> Option<&wgpu::TextureView> {
        let id = self.resources.identifier(name)?;
        self.resources.get_texture_view(id)
    }

    pub fn buffer(&self, name: &str) -> Option<&wgpu::Buffer> {
        let id = self.resources.identifier(name)?;
        match self.resources.get_handle(id)? {
            ResourceHandle::ExternalBuffer { buffer }
            | ResourceHandle::TransientBuffer { buffer } => Some(buffer),
            _ => None,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub trait PassNode<C = ()>: Send + Sync + std::any::Any {
    fn decl(&self) -> &PassDecl;
    fn prepare(&mut self, _device: &Device, _queue: &wgpu::Queue, _configs: &C) {}
    fn invalidate_bind_groups(&mut self) {}
    fn enabled(&self, _configs: &C) -> bool {
        true
    }
    fn publish(&mut self, _publisher: &mut ResourcePublisher<'_>) {}
    fn consume(&mut self, _reader: &ResourceReader<'_>) {}
    fn resize(&mut self, _device: &Device, _queue: &wgpu::Queue, _width: u32, _height: u32) {}
    fn material_update(&mut self, _device: &Device, _update: &MaterialUpdate<'_>) {}
    fn execute<'r, 'e>(&mut self, context: PassExecutionContext<'r, 'e, C>) -> Result<()>;
}

#[cfg(target_arch = "wasm32")]
pub trait PassNode<C = ()>: std::any::Any {
    fn decl(&self) -> &PassDecl;
    fn prepare(&mut self, _device: &Device, _queue: &wgpu::Queue, _configs: &C) {}
    fn invalidate_bind_groups(&mut self) {}
    fn enabled(&self, _configs: &C) -> bool {
        true
    }
    fn publish(&mut self, _publisher: &mut ResourcePublisher<'_>) {}
    fn consume(&mut self, _reader: &ResourceReader<'_>) {}
    fn resize(&mut self, _device: &Device, _queue: &wgpu::Queue, _width: u32, _height: u32) {}
    fn material_update(&mut self, _device: &Device, _update: &MaterialUpdate<'_>) {}
    fn execute<'r, 'e>(&mut self, context: PassExecutionContext<'r, 'e, C>) -> Result<()>;
}

pub struct NodeView<'a> {
    pub decl: &'a PassDecl,
    pub slots: &'a [ResourceId],
    pub enabled: bool,
}

impl NodeView<'_> {
    pub fn reads(&self) -> &[ResourceId] {
        &self.slots[self.decl.reads_at()]
    }

    pub fn writes(&self) -> &[ResourceId] {
        &self.slots[self.decl.writes_at()]
    }

    pub fn reads_writes(&self) -> &[ResourceId] {
        &self.slots[self.decl.reads_writes_at()]
    }

    pub fn optional_reads(&self) -> impl Iterator<Item = ResourceId> + '_ {
        self.slots[self.decl.optional_reads_at()]
            .iter()
            .copied()
            .filter(|resource| *resource != ResourceId::UNBOUND)
    }
}

pub struct ColorTextureBuilder<'a, C = ()> {
    pub(super) graph: &'a mut super::graph::RenderGraph<C>,
    pub(super) name: String,
    pub(super) descriptor: RenderGraphTextureDescriptor,
    pub(super) clear_color: Option<wgpu::Color>,
    pub(super) force_store: bool,
    pub(super) fixed_size: bool,
}

impl<'a, C> ColorTextureBuilder<'a, C> {
    pub fn format(mut self, format: TextureFormat) -> Self {
        self.descriptor.format = format;
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.descriptor.width = width;
        self.descriptor.height = height;
        self
    }

    pub fn usage(mut self, usage: TextureUsages) -> Self {
        self.descriptor.usage = usage;
        self
    }

    pub fn clear_color(mut self, color: wgpu::Color) -> Self {
        self.clear_color = Some(color);
        self
    }

    pub fn force_store(mut self) -> Self {
        self.force_store = true;
        self
    }

    pub fn external(self) -> ResourceId {
        self.graph.resources.register_external_resource(
            self.name,
            ResourceType::ExternalColor {
                clear_color: self.clear_color,
            },
        )
    }

    pub fn transient(self) -> ResourceId {
        self.graph.resources.register_transient_resource_opts(
            self.name,
            ResourceType::TransientColor {
                descriptor: self.descriptor,
                clear_color: self.clear_color,
                force_store: self.force_store,
            },
            self.fixed_size,
        )
    }
}

pub struct DepthTextureBuilder<'a, C = ()> {
    pub(super) graph: &'a mut super::graph::RenderGraph<C>,
    pub(super) name: String,
    pub(super) descriptor: RenderGraphTextureDescriptor,
    pub(super) clear_depth: Option<f32>,
    pub(super) force_store: bool,
    pub(super) fixed_size: bool,
}

impl<'a, C> DepthTextureBuilder<'a, C> {
    pub fn format(mut self, format: TextureFormat) -> Self {
        self.descriptor.format = format;
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.descriptor.width = width;
        self.descriptor.height = height;
        self
    }

    pub fn usage(mut self, usage: TextureUsages) -> Self {
        self.descriptor.usage = usage;
        self
    }

    pub fn clear_depth(mut self, depth: f32) -> Self {
        self.clear_depth = Some(depth);
        self
    }

    pub fn force_store(mut self) -> Self {
        self.force_store = true;
        self
    }

    pub fn fixed_size(mut self) -> Self {
        self.fixed_size = true;
        self
    }

    pub fn external(self) -> ResourceId {
        self.graph.resources.register_external_resource(
            self.name,
            ResourceType::ExternalDepth {
                clear_depth: self.clear_depth,
            },
        )
    }

    pub fn transient(self) -> ResourceId {
        self.graph.resources.register_transient_resource_opts(
            self.name,
            ResourceType::TransientDepth {
                descriptor: self.descriptor,
                clear_depth: self.clear_depth,
                force_store: self.force_store,
            },
            self.fixed_size,
        )
    }
}
