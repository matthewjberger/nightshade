use super::super::types::InstancedTransformComputeUniforms;
use super::MeshPass;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;

impl MeshPass {
    pub(in super::super) fn dispatch_instanced_transform_compute(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        configs: &nightshade_wgpu::backend::RenderInputs,
    ) {
        let ranges: Vec<_> = self
            .state
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

        if ranges.len() > self.instanced.uniform_slots {
            self.instanced.uniforms = build::uniform(
                device,
                "Instanced Transform Compute Uniforms Buffer (Resized)",
                slot_size * ranges.len() as u64,
            );
            self.instanced.uniform_slots = ranges.len();
            for world_state in self.state.iter_mut() {
                if let Some(gpu) = world_state.gpu_buffers.as_mut() {
                    gpu.instanced_compute_bind_group = None;
                }
            }
        }

        if self
            .state
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
                .scene_world
                .get::<nightshade_render::data::config::InstancedObjectData>(entity)
                .map(|instanced| instanced.parent_transform)
                .unwrap_or_else(nalgebra_glm::Mat4::identity);
            let uniforms = InstancedTransformComputeUniforms {
                parent_transform: parent_transform.into(),
                instance_count: count,
                output_offset: start,
                padding: [0; 2],
            };
            queue.write_buffer(
                &self.instanced.uniforms,
                slot_size * slot_index as u64,
                bytemuck::bytes_of(&uniforms),
            );
        }

        let bind_group = self
            .state
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
        compute_pass.set_pipeline(&self.instanced.pipeline);
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

        let current_size = self
            .state
            .as_ref()
            .unwrap()
            .gpu_buffers
            .as_ref()
            .unwrap()
            .instanced_local_matrix_buffer_size;

        if local_matrices.len() > current_size {
            let new_size = super::super::types::grown_buffer_size(local_matrices.len());

            let new_buffer = build::buffer(
                device,
                "Instanced Local Matrix Buffer (Resized)",
                (std::mem::size_of::<[[f32; 4]; 4]>() * new_size) as u64,
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            );
            {
                let world_state = self.state.as_mut().unwrap();
                let gpu = world_state.gpu_buffers.as_mut().unwrap();
                gpu.instanced_local_matrix_buffer = new_buffer;
                gpu.instanced_local_matrix_buffer_size = new_size;
            }
        }

        let buffer = &self
            .state
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
