use super::super::super::material_gpu;
use super::MeshPass;

impl MeshPass {
    pub fn apply_material_textures(
        &mut self,
        device: &wgpu::Device,
        arrays: &crate::wgpu::material_texture_arrays::MaterialTextureArrays,
    ) {
        if self.material_bindless_max.is_some() && arrays.is_bindless() {
            let views = arrays.bindless_view_refs();
            let samplers = arrays.samplers();
            self.material_bind_group = Some(material_gpu::bindless_material_bind_group(
                device,
                &self.texture_bind_group_layout,
                &views,
                &samplers,
                "Mesh Material Bindless Bind Group",
            ));
            return;
        }

        self.material_array_srgb_view = Some(arrays.srgb_view().clone());
        self.material_array_linear_view = Some(arrays.linear_view().clone());
        self.material_array_samplers = Some(arrays.samplers());
        self.material_bind_group = material_gpu::rebuild_array_material_bind_group(
            device,
            &self.texture_bind_group_layout,
            self.material_array_srgb_view.as_ref(),
            self.material_array_linear_view.as_ref(),
            self.material_array_samplers.as_ref(),
            "Mesh Material Texture Bind Group",
        );
    }
}
