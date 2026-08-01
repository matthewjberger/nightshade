use super::MeshPass;
use nightshade_wgpu::wgpu;
use nightshade_wgpu_shading::backend::material_gpu;

impl MeshPass {
    pub fn apply_material_textures(
        &mut self,
        device: &wgpu::Device,
        arrays: &nightshade_wgpu::backend::MaterialTextureArrays,
    ) {
        if self.materials.bindless_max.is_some() && arrays.is_bindless() {
            let views = arrays.bindless_view_refs();
            let samplers = arrays.samplers();
            self.materials.bind_group = Some(material_gpu::bindless_material_bind_group(
                device,
                &self.bindings.texture_layout,
                &views,
                &samplers,
                "Mesh Material Bindless Bind Group",
            ));
            return;
        }

        self.materials.array_srgb_view = Some(arrays.srgb_view().clone());
        self.materials.array_linear_view = Some(arrays.linear_view().clone());
        self.materials.array_samplers = Some(arrays.samplers());
        self.materials.bind_group = material_gpu::rebuild_array_material_bind_group(
            device,
            &self.bindings.texture_layout,
            self.materials.array_srgb_view.as_ref(),
            self.materials.array_linear_view.as_ref(),
            self.materials.array_samplers.as_ref(),
            "Mesh Material Texture Bind Group",
        );
    }
}
