use super::super::pass::{OIT_RESOURCE_CACHE_CAPACITY, OitResourceBundle};
use super::MeshPass;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;

impl MeshPass {
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self.oit.texture_size == (width, height) || width == 0 || height == 0 {
            return;
        }

        self.bindings.scene_group_dirty = true;

        if let Some(position) = self
            .oit
            .resource_cache
            .iter()
            .position(|(size, _)| *size == (width, height))
        {
            let (cached_size, cached_bundle) = self.oit.resource_cache.remove(position);
            let previous_bundle = self.swap_in_oit_bundle(cached_bundle);
            self.push_oit_cache_entry(self.oit.texture_size, previous_bundle);
            self.oit.texture_size = cached_size;

            if let Some(world_state) = self.state.as_mut()
                && let Some(gpu) = world_state.gpu_buffers.as_mut()
            {
                gpu.culling_bind_group = None;
            }
            return;
        }

        self.frame.resize_rebuild_pending = true;
        let new_bundle = MeshPass::create_oit_bundle(
            device,
            &self.oit.composite_bind_group_layout,
            &self.oit.sampler,
            width,
            height,
        );
        let previous_size = self.oit.texture_size;
        let previous_bundle = self.swap_in_oit_bundle(new_bundle);
        self.push_oit_cache_entry(previous_size, previous_bundle);
        self.oit.texture_size = (width, height);

        if let Some(world_state) = self.state.as_mut()
            && let Some(gpu) = world_state.gpu_buffers.as_mut()
        {
            gpu.culling_bind_group = None;
        }
    }

    fn create_oit_bundle(
        device: &wgpu::Device,
        composite_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        width: u32,
        height: u32,
    ) -> OitResourceBundle {
        let accum_texture = build::surface(
            device,
            "OIT Accumulation Texture",
            wgpu::TextureFormat::Rgba16Float,
            [width, height, 1],
            1,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );

        let reveal_texture = build::surface(
            device,
            "OIT Reveal Texture",
            wgpu::TextureFormat::R8Unorm,
            [width, height, 1],
            1,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );

        let accum_view = accum_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let reveal_view = reveal_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let overlay_depth_texture = build::surface(
            device,
            "Overlay Depth Texture",
            wgpu::TextureFormat::Depth32Float,
            [width, height, 1],
            1,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        );

        let overlay_depth_view =
            overlay_depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let composite_bind_group = build::binding(
            device,
            "OIT Composite Bind Group",
            composite_layout,
            vec![
                wgpu::BindingResource::TextureView(&accum_view),
                wgpu::BindingResource::Sampler(sampler),
                wgpu::BindingResource::TextureView(&reveal_view),
                wgpu::BindingResource::Sampler(sampler),
            ],
        );

        OitResourceBundle {
            accum_texture,
            accum_view,
            reveal_texture,
            reveal_view,
            overlay_depth_texture,
            overlay_depth_view,
            composite_bind_group,
        }
    }

    fn swap_in_oit_bundle(&mut self, bundle: OitResourceBundle) -> OitResourceBundle {
        let previous_accum_texture =
            std::mem::replace(&mut self.oit.accum_texture, bundle.accum_texture);
        let previous_accum_view = std::mem::replace(&mut self.oit.accum_view, bundle.accum_view);
        let previous_reveal_texture =
            std::mem::replace(&mut self.oit.reveal_texture, bundle.reveal_texture);
        let previous_reveal_view = std::mem::replace(&mut self.oit.reveal_view, bundle.reveal_view);
        let previous_overlay_depth_texture = std::mem::replace(
            &mut self.pipelines.overlay_depth_texture,
            bundle.overlay_depth_texture,
        );
        let previous_overlay_depth_view = std::mem::replace(
            &mut self.pipelines.overlay_depth_view,
            bundle.overlay_depth_view,
        );
        let previous_composite_bind_group = std::mem::replace(
            &mut self.oit.composite_bind_group,
            bundle.composite_bind_group,
        );

        OitResourceBundle {
            accum_texture: previous_accum_texture,
            accum_view: previous_accum_view,
            reveal_texture: previous_reveal_texture,
            reveal_view: previous_reveal_view,
            overlay_depth_texture: previous_overlay_depth_texture,
            overlay_depth_view: previous_overlay_depth_view,
            composite_bind_group: previous_composite_bind_group,
        }
    }

    fn push_oit_cache_entry(&mut self, size: (u32, u32), bundle: OitResourceBundle) {
        self.oit.resource_cache.push((size, bundle));
        if self.oit.resource_cache.len() > OIT_RESOURCE_CACHE_CAPACITY {
            self.oit.resource_cache.remove(0);
        }
    }
}
