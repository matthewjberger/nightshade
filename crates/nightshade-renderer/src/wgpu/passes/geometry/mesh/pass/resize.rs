use super::super::pass::{OIT_RESOURCE_CACHE_CAPACITY, OitResourceBundle};
use super::MeshPass;

impl MeshPass {
    /// Resizes the order-independent-transparency render targets to `width` by
    /// `height`, reusing a cached bundle for that size when one exists.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self.oit_texture_size == (width, height) || width == 0 || height == 0 {
            return;
        }

        self.resize_full_rebuild_pending = true;
        self.scene_bind_group_dirty = true;

        if let Some(position) = self
            .oit_resource_cache
            .iter()
            .position(|(size, _)| *size == (width, height))
        {
            let (cached_size, cached_bundle) = self.oit_resource_cache.remove(position);
            let previous_bundle = self.swap_in_oit_bundle(cached_bundle);
            self.push_oit_cache_entry(self.oit_texture_size, previous_bundle);
            self.oit_texture_size = cached_size;

            if let Some(world_state) = self.world_state_get_mut(self.current_world_id)
                && let Some(gpu) = world_state.gpu_buffers.as_mut()
            {
                gpu.culling_bind_group = None;
            }
            return;
        }

        let new_bundle = MeshPass::create_oit_bundle(
            device,
            &self.oit_composite_bind_group_layout,
            &self.oit_sampler,
            width,
            height,
        );
        let previous_size = self.oit_texture_size;
        let previous_bundle = self.swap_in_oit_bundle(new_bundle);
        self.push_oit_cache_entry(previous_size, previous_bundle);
        self.oit_texture_size = (width, height);

        if let Some(world_state) = self.world_state_get_mut(self.current_world_id)
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
        let accum_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("OIT Accumulation Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let reveal_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("OIT Reveal Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let accum_view = accum_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let reveal_view = reveal_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let overlay_depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Overlay Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let overlay_depth_view =
            overlay_depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let composite_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("OIT Composite Bind Group"),
            layout: composite_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&accum_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&reveal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });

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
            std::mem::replace(&mut self.oit_accum_texture, bundle.accum_texture);
        let previous_accum_view = std::mem::replace(&mut self.oit_accum_view, bundle.accum_view);
        let previous_reveal_texture =
            std::mem::replace(&mut self.oit_reveal_texture, bundle.reveal_texture);
        let previous_reveal_view = std::mem::replace(&mut self.oit_reveal_view, bundle.reveal_view);
        let previous_overlay_depth_texture = std::mem::replace(
            &mut self.overlay_depth_texture,
            bundle.overlay_depth_texture,
        );
        let previous_overlay_depth_view =
            std::mem::replace(&mut self.overlay_depth_view, bundle.overlay_depth_view);
        let previous_composite_bind_group = std::mem::replace(
            &mut self.oit_composite_bind_group,
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
        self.oit_resource_cache.push((size, bundle));
        if self.oit_resource_cache.len() > OIT_RESOURCE_CACHE_CAPACITY {
            self.oit_resource_cache.remove(0);
        }
    }

    /// Sets the image-based lighting views for the current world and marks the
    /// scene bind group for rebuild.
    pub fn update_ibl_textures(
        &mut self,
        brdf_lut_view: wgpu::TextureView,
        irradiance_view: wgpu::TextureView,
        prefiltered_view: wgpu::TextureView,
    ) {
        self.brdf_lut_view = brdf_lut_view.clone();
        self.irradiance_view = irradiance_view.clone();
        self.prefiltered_view = prefiltered_view.clone();
        self.scene_bind_group_dirty = true;

        if let Some(world_state) = self.world_state_get_mut(self.current_world_id) {
            world_state.ibl_brdf_lut_view = Some(brdf_lut_view);
            world_state.ibl_irradiance_view = Some(irradiance_view);
            world_state.ibl_prefiltered_view = Some(prefiltered_view);
            world_state.ibl_dirty = true;
        }
    }

    /// Sets two image-based lighting probes and a blend factor for the current
    /// world so the shader can cross-fade between them.
    pub fn update_ibl_textures_blended(
        &mut self,
        brdf_lut_view: wgpu::TextureView,
        irradiance_a: wgpu::TextureView,
        prefiltered_a: wgpu::TextureView,
        irradiance_b: wgpu::TextureView,
        prefiltered_b: wgpu::TextureView,
        blend_factor: f32,
    ) {
        self.brdf_lut_view = brdf_lut_view.clone();
        self.irradiance_view = irradiance_a.clone();
        self.prefiltered_view = prefiltered_a.clone();
        self.irradiance_b_view = irradiance_b.clone();
        self.prefiltered_b_view = prefiltered_b.clone();
        self.scene_bind_group_dirty = true;

        if let Some(world_state) = self.world_state_get_mut(self.current_world_id) {
            world_state.ibl_brdf_lut_view = Some(brdf_lut_view);
            world_state.ibl_irradiance_view = Some(irradiance_a);
            world_state.ibl_prefiltered_view = Some(prefiltered_a);
            world_state.ibl_irradiance_b_view = Some(irradiance_b);
            world_state.ibl_prefiltered_b_view = Some(prefiltered_b);
            world_state.ibl_blend_factor = blend_factor;
            world_state.ibl_dirty = true;
        }
    }

    /// Sets the image-based lighting views for a specific world, marking the
    /// scene bind group dirty only when that world is current.
    pub fn update_ibl_textures_for_world(
        &mut self,
        world_id: u64,
        brdf_lut_view: wgpu::TextureView,
        irradiance_view: wgpu::TextureView,
        prefiltered_view: wgpu::TextureView,
    ) {
        if world_id == self.current_world_id {
            self.scene_bind_group_dirty = true;
        }
        if let Some(world_state) = self.world_state_get_mut(world_id) {
            world_state.ibl_brdf_lut_view = Some(brdf_lut_view);
            world_state.ibl_irradiance_view = Some(irradiance_view);
            world_state.ibl_prefiltered_view = Some(prefiltered_view);
            world_state.ibl_dirty = true;
        }
    }

    /// Sets the point-light shadow cubemap array view and marks the scene bind
    /// group for rebuild.
    pub fn update_point_shadow_cubemap(&mut self, view: wgpu::TextureView) {
        self.point_shadow_cubemap_view = view;
        self.scene_bind_group_dirty = true;
    }
}
