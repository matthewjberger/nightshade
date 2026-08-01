use super::super::world_state::WorldGpuBuffers;
use super::MeshPass;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;

impl MeshPass {
    pub(in super::super) fn rebuild_instance_bind_group(&mut self, device: &wgpu::Device) {
        let world_state = self.state.as_mut().unwrap();
        let gpu = world_state.gpu_buffers.as_mut().unwrap();

        gpu.instance_bind_group = build::binding(
            device,
            "Mesh Instance Bind Group",
            &self.bindings.instance_layout,
            vec![
                gpu.transform_buffer.as_entire_binding(),
                gpu.materials_buffer.as_entire_binding(),
                gpu.object_buffer.as_entire_binding(),
                gpu.lights_buffer.as_entire_binding(),
                gpu.visible_indices_buffer.as_entire_binding(),
                gpu.custom_data_buffer.as_entire_binding(),
                self.morph.buffer.as_entire_binding(),
                gpu.light_grid_buffer.as_entire_binding(),
                gpu.light_indices_buffer.as_entire_binding(),
                gpu.cluster_uniforms_buffer.as_entire_binding(),
                gpu.area_lights_buffer.as_entire_binding(),
                gpu.area_uniforms_buffer.as_entire_binding(),
                gpu.prev_transform_buffer.as_entire_binding(),
            ],
        );

        gpu.cluster_bounds_bind_group = build::binding(
            device,
            "Cluster Bounds Bind Group (Per-World)",
            &self.clusters.bounds_layout,
            vec![
                gpu.cluster_uniforms_buffer.as_entire_binding(),
                self.clusters.bounds_buffer.as_entire_binding(),
            ],
        );
    }

    pub(in super::super) fn invalidate_all_instance_bind_groups(&mut self, device: &wgpu::Device) {
        if self
            .state
            .as_ref()
            .is_some_and(|state| state.gpu_buffers.is_some())
        {
            self.rebuild_instance_bind_group(device);
        }
    }

    pub(in super::super) fn rebuild_instanced_compute_bind_group(&mut self, device: &wgpu::Device) {
        let world_state = self.state.as_mut().unwrap();
        let gpu = world_state.gpu_buffers.as_mut().unwrap();
        gpu.instanced_compute_bind_group = Some(build::binding(
            device,
            "Instanced Transform Compute Bind Group",
            &self.instanced.layout,
            vec![
                gpu.instanced_local_matrix_buffer.as_entire_binding(),
                gpu.transform_buffer.as_entire_binding(),
                wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &self.instanced.uniforms,
                    offset: 0,
                    size: Some(
                        std::num::NonZeroU64::new(std::mem::size_of::<
                            super::super::types::InstancedTransformComputeUniforms,
                        >() as u64)
                        .unwrap(),
                    ),
                }),
            ],
        ));
    }

    pub(in super::super) fn ensure_gpu_buffers(&mut self, device: &wgpu::Device) {
        if self.state.is_none() {
            self.state = Some(super::super::world_state::WorldRenderState::new());
        }

        let state = self.state.as_mut().unwrap();
        if state.gpu_buffers.is_none() {
            state.gpu_buffers = Some(WorldGpuBuffers::new(
                device,
                &self.bindings.instance_layout,
                &self.clusters.bounds_layout,
                &self.clusters.bounds_buffer,
                &self.morph.buffer,
            ));
        }
    }
}
