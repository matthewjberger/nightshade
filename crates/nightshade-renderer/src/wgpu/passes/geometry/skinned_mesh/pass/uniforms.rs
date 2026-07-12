use super::super::pass::{SKINNED_OIT_CACHE_CAPACITY, SkinnedOitBundle};
use super::super::types::BlendedIblViews;
use super::SkinnedMeshPass;

impl SkinnedMeshPass {
    pub fn update_ibl_textures_for_world(
        &mut self,
        world_id: u64,
        brdf_lut_view: wgpu::TextureView,
        irradiance_view: wgpu::TextureView,
        prefiltered_view: wgpu::TextureView,
    ) {
        if let Some(Some(world_state)) = self.world_states.get_mut(world_id as usize) {
            world_state.ibl_brdf_lut_view = Some(brdf_lut_view);
            world_state.ibl_irradiance_view = Some(irradiance_view);
            world_state.ibl_prefiltered_view = Some(prefiltered_view);
        }
    }

    pub fn update_ibl_textures_blended_for_world(&mut self, world_id: u64, views: BlendedIblViews) {
        if let Some(Some(world_state)) = self.world_states.get_mut(world_id as usize) {
            world_state.ibl_brdf_lut_view = Some(views.brdf_lut);
            world_state.ibl_irradiance_view = Some(views.irradiance_a);
            world_state.ibl_prefiltered_view = Some(views.prefiltered_a);
            world_state.ibl_irradiance_b_view = Some(views.irradiance_b);
            world_state.ibl_prefiltered_b_view = Some(views.prefiltered_b);
            world_state.ibl_blend_factor = views.blend_factor;
        }
    }

    pub fn set_ibl_textures(
        &mut self,
        brdf_lut_view: wgpu::TextureView,
        irradiance_map_view: wgpu::TextureView,
        prefiltered_env_view: wgpu::TextureView,
    ) {
        self.brdf_lut_view = Some(brdf_lut_view);
        self.irradiance_map_view = Some(irradiance_map_view);
        self.prefiltered_env_view = Some(prefiltered_env_view);
    }

    pub fn update_point_shadow_cubemap(&mut self, view: wgpu::TextureView) {
        self.point_shadow_cubemap_view = view;
    }

    pub fn cleanup_stale_world_states(&mut self) {
        const STALE_THRESHOLD_FRAMES: u64 = 300;
        let current_frame = self.frame_counter;
        let current_world_id = self.current_world_id;
        for (index, slot) in self.world_states.iter_mut().enumerate() {
            let world_id = index as u64;
            let drop_slot = slot.as_ref().is_some_and(|state| {
                world_id != current_world_id
                    && current_frame.saturating_sub(state.last_used_frame) >= STALE_THRESHOLD_FRAMES
            });
            if drop_slot {
                *slot = None;
            }
        }
    }

    pub fn resize_oit_textures(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self.oit_texture_size == (width, height) || width == 0 || height == 0 {
            return;
        }

        if let Some(position) = self
            .oit_resource_cache
            .iter()
            .position(|(size, _)| *size == (width, height))
        {
            let (cached_size, cached_bundle) = self.oit_resource_cache.remove(position);
            let previous_bundle = self.swap_in_skinned_oit_bundle(cached_bundle);
            self.push_skinned_oit_cache_entry(self.oit_texture_size, previous_bundle);
            self.oit_texture_size = cached_size;
            return;
        }

        let new_bundle = SkinnedMeshPass::create_skinned_oit_bundle(device, width, height);
        let previous_size = self.oit_texture_size;
        let previous_bundle = self.swap_in_skinned_oit_bundle(new_bundle);
        self.push_skinned_oit_cache_entry(previous_size, previous_bundle);
        self.oit_texture_size = (width, height);
    }

    fn create_skinned_oit_bundle(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> SkinnedOitBundle {
        let accum_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Skinned Mesh OIT Accum Texture"),
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
        let accum_view = accum_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let reveal_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Skinned Mesh OIT Reveal Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let reveal_view = reveal_texture.create_view(&wgpu::TextureViewDescriptor::default());

        SkinnedOitBundle {
            accum_texture,
            accum_view,
            reveal_texture,
            reveal_view,
        }
    }

    fn swap_in_skinned_oit_bundle(&mut self, bundle: SkinnedOitBundle) -> SkinnedOitBundle {
        let previous_accum_texture =
            std::mem::replace(&mut self.oit_accum_texture, bundle.accum_texture);
        let previous_accum_view = std::mem::replace(&mut self.oit_accum_view, bundle.accum_view);
        let previous_reveal_texture =
            std::mem::replace(&mut self.oit_reveal_texture, bundle.reveal_texture);
        let previous_reveal_view = std::mem::replace(&mut self.oit_reveal_view, bundle.reveal_view);

        SkinnedOitBundle {
            accum_texture: previous_accum_texture,
            accum_view: previous_accum_view,
            reveal_texture: previous_reveal_texture,
            reveal_view: previous_reveal_view,
        }
    }

    fn push_skinned_oit_cache_entry(&mut self, size: (u32, u32), bundle: SkinnedOitBundle) {
        if size.0 == 0 || size.1 == 0 {
            return;
        }
        self.oit_resource_cache.push((size, bundle));
        if self.oit_resource_cache.len() > SKINNED_OIT_CACHE_CAPACITY {
            self.oit_resource_cache.remove(0);
        }
    }
}
