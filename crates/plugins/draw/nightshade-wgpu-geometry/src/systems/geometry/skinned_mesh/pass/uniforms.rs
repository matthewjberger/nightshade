use super::super::pass::{SKINNED_OIT_CACHE_CAPACITY, SkinnedOitBundle};
use super::SkinnedMeshPass;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;

impl SkinnedMeshPass {
    pub fn resize_oit_textures(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self.oit.texture_size == (width, height) || width == 0 || height == 0 {
            return;
        }

        if let Some(position) = self
            .oit
            .resource_cache
            .iter()
            .position(|(size, _)| *size == (width, height))
        {
            let (cached_size, cached_bundle) = self.oit.resource_cache.remove(position);
            let previous_bundle = self.swap_in_skinned_oit_bundle(cached_bundle);
            self.push_skinned_oit_cache_entry(self.oit.texture_size, previous_bundle);
            self.oit.texture_size = cached_size;
            return;
        }

        let new_bundle = SkinnedMeshPass::create_skinned_oit_bundle(device, width, height);
        let previous_size = self.oit.texture_size;
        let previous_bundle = self.swap_in_skinned_oit_bundle(new_bundle);
        self.push_skinned_oit_cache_entry(previous_size, previous_bundle);
        self.oit.texture_size = (width, height);
    }

    fn create_skinned_oit_bundle(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> SkinnedOitBundle {
        let accum_texture = build::surface(
            device,
            "Skinned Mesh OIT Accum Texture",
            wgpu::TextureFormat::Rgba16Float,
            [width, height, 1],
            1,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let accum_view = accum_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let reveal_texture = build::surface(
            device,
            "Skinned Mesh OIT Reveal Texture",
            wgpu::TextureFormat::R16Float,
            [width, height, 1],
            1,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );
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
            std::mem::replace(&mut self.oit.accum_texture, bundle.accum_texture);
        let previous_accum_view = std::mem::replace(&mut self.oit.accum_view, bundle.accum_view);
        let previous_reveal_texture =
            std::mem::replace(&mut self.oit.reveal_texture, bundle.reveal_texture);
        let previous_reveal_view = std::mem::replace(&mut self.oit.reveal_view, bundle.reveal_view);

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
        self.oit.resource_cache.push((size, bundle));
        if self.oit.resource_cache.len() > SKINNED_OIT_CACHE_CAPACITY {
            self.oit.resource_cache.remove(0);
        }
    }
}
