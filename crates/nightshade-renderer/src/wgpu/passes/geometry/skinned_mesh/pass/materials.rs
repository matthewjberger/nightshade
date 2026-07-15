use super::super::super::material_gpu;
use super::SkinnedMeshPass;

impl SkinnedMeshPass {
    pub fn apply_material_textures(
        &mut self,
        device: &wgpu::Device,
        arrays: &crate::wgpu::material_texture_arrays::MaterialTextureArrays,
    ) {
        if self.material_bindless_max.is_some() && arrays.is_bindless() {
            let views = arrays.bindless_view_refs();
            self.material_bind_group = Some(material_gpu::bindless_material_bind_group(
                device,
                &self.texture_bind_group_layout,
                &views,
                arrays.sampler(),
                "Skinned Mesh Material Bindless Bind Group",
            ));
            return;
        }

        self.material_array_srgb_view = Some(arrays.srgb_view().clone());
        self.material_array_linear_view = Some(arrays.linear_view().clone());
        self.material_array_sampler = Some(arrays.sampler().clone());
        self.material_bind_group = material_gpu::rebuild_array_material_bind_group(
            device,
            &self.texture_bind_group_layout,
            self.material_array_srgb_view.as_ref(),
            self.material_array_linear_view.as_ref(),
            self.material_array_sampler.as_ref(),
            "Skinned Mesh Material Texture Bind Group",
        );
    }

    pub(in super::super) fn rebuild_material_bind_group(&mut self, device: &wgpu::Device) {
        self.material_bind_group = material_gpu::rebuild_array_material_bind_group(
            device,
            &self.texture_bind_group_layout,
            self.material_array_srgb_view.as_ref(),
            self.material_array_linear_view.as_ref(),
            self.material_array_sampler.as_ref(),
            "Skinned Mesh Material Texture Bind Group",
        );
    }

    pub(in super::super) fn rebuild_instance_bind_group(&mut self, device: &wgpu::Device) {
        let world_state = self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap();
        if let Some(gpu) = world_state.gpu_buffers.as_mut() {
            gpu.instance_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Skinned Mesh Instance Bind Group (Per-World, Rebuilt)"),
                layout: &self.instance_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: gpu.materials_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: gpu.object_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: gpu.lights_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: self.joint_matrices_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: gpu.custom_data_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: self.morph_displacement_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: gpu.visible_indices_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: gpu.light_grid_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: gpu.light_indices_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 9,
                        resource: gpu.cluster_uniforms_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 10,
                        resource: gpu.area_lights_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 11,
                        resource: gpu.area_uniforms_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 12,
                        resource: self.prev_joint_matrices_buffer.as_entire_binding(),
                    },
                ],
            });
        }
    }

    pub(in super::super) fn rebuild_cull_bind_group(&mut self, device: &wgpu::Device) {
        let world_state = self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap();
        if let Some(gpu) = world_state.gpu_buffers.as_mut() {
            gpu.cull_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Skinned Mesh Cull Bind Group (Per-World, Rebuilt)"),
                layout: &self.cull_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: gpu.cull_objects_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: gpu.indirect_commands_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: gpu.visible_indices_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: gpu.cull_uniforms_buffer.as_entire_binding(),
                    },
                ],
            });
        }
    }

    pub(in super::super) fn rebuild_bounds_compute_bind_group(&mut self, device: &wgpu::Device) {
        let bind_group = {
            let Some(gpu) = self.world_states[self.current_world_id as usize]
                .as_ref()
                .and_then(|state| state.gpu_buffers.as_ref())
            else {
                return;
            };
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Skinned Bounds Compute Bind Group"),
                layout: &self.bounds_compute_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.bone_transforms_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: gpu.bounds_inputs_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: gpu.cull_objects_buffer.as_entire_binding(),
                    },
                ],
            })
        };
        self.bounds_compute_bind_group = Some(bind_group);
    }

    pub(in super::super) fn rebuild_skinning_compute_bind_group(&mut self, device: &wgpu::Device) {
        self.skinning_compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Skinning Compute Bind Group"),
            layout: &self._skinning_compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.bone_transforms_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.inverse_bind_matrices_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.skin_data_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.joint_matrices_buffer.as_entire_binding(),
                },
            ],
        });
    }
}
