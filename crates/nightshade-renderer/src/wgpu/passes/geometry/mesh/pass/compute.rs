use super::super::types::InstancedTransformComputeUniforms;
use super::MeshPass;

impl MeshPass {
    pub(in super::super) fn dispatch_instanced_transform_compute(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        configs: &crate::wgpu::render_configs::RenderInputs,
    ) {
        let ranges: Vec<_> = self.world_states[self.current_world_id as usize]
            .as_ref()
            .unwrap()
            .instanced_transform_ranges
            .iter()
            .map(|(&entity, &(start, count))| (entity, start, count))
            .collect();

        if ranges.is_empty() {
            return;
        }

        let struct_size = std::mem::size_of::<InstancedTransformComputeUniforms>() as u64;
        let alignment = device.limits().min_uniform_buffer_offset_alignment as u64;
        let slot_size = struct_size.div_ceil(alignment) * alignment;

        if ranges.len() > self.instanced_compute_uniforms_slots {
            self.instanced_compute_uniforms_buffer =
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Instanced Transform Compute Uniforms Buffer (Resized)"),
                    size: slot_size * ranges.len() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            self.instanced_compute_uniforms_slots = ranges.len();
            for world_state in self.world_states.iter_mut().flatten() {
                if let Some(gpu) = world_state.gpu_buffers.as_mut() {
                    gpu.instanced_compute_bind_group = None;
                }
            }
        }

        if self.world_states[self.current_world_id as usize]
            .as_ref()
            .unwrap()
            .gpu_buffers
            .as_ref()
            .unwrap()
            .instanced_compute_bind_group
            .is_none()
        {
            self.rebuild_instanced_compute_bind_group(device);
        }

        for (slot_index, &(entity, start, count)) in ranges.iter().enumerate() {
            let parent_transform = configs
                .scene
                .render_instanced_objects
                .get(&entity)
                .map(|instanced| instanced.parent_transform)
                .unwrap_or_else(nalgebra_glm::Mat4::identity);
            let uniforms = InstancedTransformComputeUniforms {
                parent_transform: parent_transform.into(),
                instance_count: count,
                output_offset: start,
                _pad: [0; 2],
            };
            queue.write_buffer(
                &self.instanced_compute_uniforms_buffer,
                slot_size * slot_index as u64,
                bytemuck::bytes_of(&uniforms),
            );
        }

        let bind_group = self.world_states[self.current_world_id as usize]
            .as_ref()
            .unwrap()
            .gpu_buffers
            .as_ref()
            .unwrap()
            .instanced_compute_bind_group
            .as_ref()
            .unwrap();

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Instanced Transform Compute Pass"),
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(&self.instanced_compute_pipeline);
        for (slot_index, &(_, _, count)) in ranges.iter().enumerate() {
            let dynamic_offset = (slot_size * slot_index as u64) as u32;
            compute_pass.set_bind_group(0, bind_group, &[dynamic_offset]);
            compute_pass.dispatch_workgroups(count.div_ceil(256), 1, 1);
        }
    }

    pub(in super::super) fn upload_instanced_local_matrices(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        local_matrices: &[[[f32; 4]; 4]],
    ) {
        if local_matrices.is_empty() {
            return;
        }

        let current_size = self.world_states[self.current_world_id as usize]
            .as_ref()
            .unwrap()
            .gpu_buffers
            .as_ref()
            .unwrap()
            .instanced_local_matrix_buffer_size;

        if local_matrices.len() > current_size {
            let new_size = std::cmp::min(
                (local_matrices.len() as f32 * super::super::types::BUFFER_GROWTH_FACTOR).ceil()
                    as usize,
                super::super::types::MAX_INSTANCES,
            );

            let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instanced Local Matrix Buffer (Resized)"),
                size: (std::mem::size_of::<[[f32; 4]; 4]>() * new_size) as u64,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            {
                let world_state = self.world_states[self.current_world_id as usize]
                    .as_mut()
                    .unwrap();
                let gpu = world_state.gpu_buffers.as_mut().unwrap();
                gpu.instanced_local_matrix_buffer = new_buffer;
                gpu.instanced_local_matrix_buffer_size = new_size;
            }
        }

        let buffer = &self.world_states[self.current_world_id as usize]
            .as_ref()
            .unwrap()
            .gpu_buffers
            .as_ref()
            .unwrap()
            .instanced_local_matrix_buffer;
        queue.write_buffer(buffer, 0, bytemuck::cast_slice(local_matrices));

        self.rebuild_instanced_compute_bind_group(device);
    }
}
