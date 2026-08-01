use super::SkinnedMeshPass;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;
use nightshade_wgpu_shading::backend::material_gpu;

impl SkinnedMeshPass {
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
                "Skinned Mesh Material Bindless Bind Group",
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
            "Skinned Mesh Material Texture Bind Group",
        );
    }

    pub(in super::super) fn rebuild_material_bind_group(&mut self, device: &wgpu::Device) {
        self.materials.bind_group = material_gpu::rebuild_array_material_bind_group(
            device,
            &self.bindings.texture_layout,
            self.materials.array_srgb_view.as_ref(),
            self.materials.array_linear_view.as_ref(),
            self.materials.array_samplers.as_ref(),
            "Skinned Mesh Material Texture Bind Group",
        );
    }

    pub(in super::super) fn rebuild_instance_bind_group(&mut self, device: &wgpu::Device) {
        let world_state = self.state.as_mut().unwrap();
        if let Some(gpu) = world_state.gpu_buffers.as_mut() {
            gpu.instance_bind_group = build::binding(
                device,
                "Skinned Mesh Instance Bind Group (Per-World, Rebuilt)",
                &self.bindings.instance_layout,
                vec![
                    gpu.materials_buffer.as_entire_binding(),
                    gpu.object_buffer.as_entire_binding(),
                    gpu.lights_buffer.as_entire_binding(),
                    self.skinning.joints.as_entire_binding(),
                    gpu.custom_data_buffer.as_entire_binding(),
                    self.morph.buffer.as_entire_binding(),
                    gpu.visible_indices_buffer.as_entire_binding(),
                    gpu.light_grid_buffer.as_entire_binding(),
                    gpu.light_indices_buffer.as_entire_binding(),
                    gpu.cluster_uniforms_buffer.as_entire_binding(),
                    gpu.area_lights_buffer.as_entire_binding(),
                    gpu.area_uniforms_buffer.as_entire_binding(),
                    self.skinning.previous_joints.as_entire_binding(),
                ],
            );
        }
    }

    pub(in super::super) fn rebuild_cull_bind_group(&mut self, device: &wgpu::Device) {
        let world_state = self.state.as_mut().unwrap();
        if let Some(gpu) = world_state.gpu_buffers.as_mut() {
            gpu.cull_bind_group = build::binding(
                device,
                "Skinned Mesh Cull Bind Group (Per-World, Rebuilt)",
                &self.compute.cull_layout,
                vec![
                    gpu.cull_objects_buffer.as_entire_binding(),
                    gpu.indirect_commands_buffer.as_entire_binding(),
                    gpu.visible_indices_buffer.as_entire_binding(),
                    gpu.cull_uniforms_buffer.as_entire_binding(),
                ],
            );
        }
    }

    pub(in super::super) fn rebuild_bounds_compute_bind_group(&mut self, device: &wgpu::Device) {
        let bind_group = {
            let Some(gpu) = self
                .state
                .as_ref()
                .and_then(|state| state.gpu_buffers.as_ref())
            else {
                return;
            };
            build::binding(
                device,
                "Skinned Bounds Compute Bind Group",
                &self.compute.bounds_layout,
                vec![
                    self.skinning.bone_transforms.as_entire_binding(),
                    gpu.bounds_inputs_buffer.as_entire_binding(),
                    gpu.cull_objects_buffer.as_entire_binding(),
                ],
            )
        };
        self.compute.bounds_group = Some(bind_group);
    }

    pub(in super::super) fn rebuild_skinning_compute_bind_group(&mut self, device: &wgpu::Device) {
        self.skinning.bind_group = build::binding(
            device,
            "Skinning Compute Bind Group",
            &self.skinning.layout,
            vec![
                self.skinning.bone_transforms.as_entire_binding(),
                self.skinning.inverse_binds.as_entire_binding(),
                self.skinning.skin_data.as_entire_binding(),
                self.skinning.joints.as_entire_binding(),
            ],
        );
    }
}
