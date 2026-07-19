//! Per-world instance bind group construction and world state cleanup for the
//! mesh pass.

use super::super::world_state::WorldGpuBuffers;
use super::MeshPass;

impl MeshPass {
    /// Rebuilds the current world's instance and cluster bounds bind groups from
    /// its GPU buffers.
    pub(in super::super) fn rebuild_instance_bind_group(&mut self, device: &wgpu::Device) {
        let world_state = self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap();
        let gpu = world_state.gpu_buffers.as_mut().unwrap();

        gpu.instance_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Mesh Instance Bind Group"),
            layout: &self.instance_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: gpu.transform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: gpu.materials_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: gpu.object_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: gpu.lights_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: gpu.visible_indices_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: gpu.custom_data_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: self.morph_displacement_buffer.as_entire_binding(),
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
                    resource: gpu.prev_transform_buffer.as_entire_binding(),
                },
            ],
        });

        gpu.cluster_bounds_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Cluster Bounds Bind Group (Per-World)"),
            layout: &self._cluster_bounds_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: gpu.cluster_uniforms_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.cluster_bounds_buffer.as_entire_binding(),
                },
            ],
        });
    }

    /// Rebuilds the instance bind groups for every world that has GPU buffers,
    /// restoring the active world afterward.
    pub(in super::super) fn invalidate_all_instance_bind_groups(&mut self, device: &wgpu::Device) {
        let world_ids = self.world_state_ids();
        let saved_world_id = self.current_world_id;
        for world_id in world_ids {
            if self
                .world_state_get(world_id)
                .map(|state| state.gpu_buffers.is_some())
                .unwrap_or(false)
            {
                self.current_world_id = world_id;
                self.rebuild_instance_bind_group(device);
            }
        }
        self.current_world_id = saved_world_id;
    }

    /// Rebuilds the current world's instanced transform compute bind group.
    pub(in super::super) fn rebuild_instanced_compute_bind_group(&mut self, device: &wgpu::Device) {
        let world_state = self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap();
        let gpu = world_state.gpu_buffers.as_mut().unwrap();
        gpu.instanced_compute_bind_group = Some(
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Instanced Transform Compute Bind Group"),
                layout: &self.instanced_compute_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: gpu.instanced_local_matrix_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: gpu.transform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.instanced_compute_uniforms_buffer,
                            offset: 0,
                            size: Some(
                                std::num::NonZeroU64::new(std::mem::size_of::<
                                    super::super::types::InstancedTransformComputeUniforms,
                                >()
                                    as u64)
                                .unwrap(),
                            ),
                        }),
                    },
                ],
            }),
        );
    }

    /// Ensures `world_id` has a state slot with GPU buffers allocated, creating
    /// them on first use.
    pub(in super::super) fn ensure_world_gpu_buffers(
        &mut self,
        device: &wgpu::Device,
        world_id: u64,
    ) {
        self.world_state_ensure(world_id);

        let state = self.world_states[world_id as usize].as_mut().unwrap();
        if state.gpu_buffers.is_none() {
            state.gpu_buffers = Some(WorldGpuBuffers::new(
                device,
                &self.instance_bind_group_layout,
                &self._cluster_bounds_bind_group_layout,
                &self.cluster_bounds_buffer,
                &self.morph_displacement_buffer,
            ));
        }
    }

    /// Removes the state for `world_id`, clearing it as the last prepared world.
    pub fn cleanup_world_state(&mut self, world_id: u64) {
        if self.last_prepared_world_id == Some(world_id) {
            self.last_prepared_world_id = None;
        }
        self.world_state_remove(world_id);
    }

    /// Removes world states untouched for more than `max_age_frames`, keeping
    /// the last prepared world.
    pub fn cleanup_stale_world_states(&mut self, max_age_frames: u64) {
        let current_frame = self.frame_counter;
        let kept_world_id = self.last_prepared_world_id.unwrap_or(u64::MAX);
        let stale_world_ids: Vec<u64> = self
            .world_states
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| {
                let state = slot.as_ref()?;
                let world_id = index as u64;
                if world_id != kept_world_id
                    && current_frame.saturating_sub(state.last_used_frame) > max_age_frames
                {
                    Some(world_id)
                } else {
                    None
                }
            })
            .collect();

        for world_id in stale_world_ids {
            self.world_state_remove(world_id);
        }
    }
}
