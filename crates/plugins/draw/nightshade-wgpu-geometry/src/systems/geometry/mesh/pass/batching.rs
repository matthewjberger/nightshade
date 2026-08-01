use nightshade_render::data::geometry::{
    Vertex, create_cone_mesh, create_cube_mesh, create_cylinder_mesh, create_plane_mesh,
    create_quad_mesh, create_sphere_mesh, create_subdivided_plane_mesh, create_torus_mesh,
};
use nightshade_render::queries::mesh_cache::mesh_cache_iter;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;

use super::super::types::{
    BUFFER_GROWTH_FACTOR, BUFFER_SHRINK_THRESHOLD, BatchDesc, BatchKey, COMPACTION_COOLDOWN_FRAMES,
    COMPACTION_MIN_RECLAIM_BYTES, DrawIndexedIndirect, ModelMatrix, NUM_DRAW_CLASSES,
    grown_buffer_size,
};
use super::super::world_state::BatchRange;
use super::MeshPass;

fn hold_slots(list: &mut Vec<BatchRange>, cap: usize) {
    if list.len() == cap {
        return;
    }
    list.clear();
    list.resize(cap, (0, 0, 0, 0));
}

impl MeshPass {
    pub(in super::super) fn gpu_batch_setup(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let num_materials = self.state().cached_materials_data.len().max(1);
        let num_meshes = self.registry.data.len().max(1);
        let object_count = self.state().object_count as usize;

        let worst_case_per_class = (num_meshes * num_materials).min(object_count.max(1) * 4);
        let required_cap = worst_case_per_class.max(super::super::types::MAX_BATCHES_PER_CLASS);
        self.compute.batch_cap = self.compute.batch_cap.max(required_cap.next_power_of_two());
        let cap = self.compute.batch_cap;
        let total_slots = super::super::types::NUM_DRAW_CLASSES * cap;

        let mut flags = vec![0u32; num_materials];
        {
            let state = self.state();
            for &id in &state.cached_transparent_material_ids {
                if (id as usize) < num_materials {
                    flags[id as usize] |= 1;
                }
            }
            for &id in &state.cached_mask_material_ids {
                if (id as usize) < num_materials {
                    flags[id as usize] |= 2;
                }
            }
            for &id in &state.cached_double_sided_material_ids {
                if (id as usize) < num_materials {
                    flags[id as usize] |= 4;
                }
            }
            for &id in &state.cached_transmissive_material_ids {
                if (id as usize) < num_materials {
                    flags[id as usize] |= 8;
                }
            }
        }
        if num_materials > self.gpu().material_flags_buffer_size {
            let new_buffer = build::storage(
                device,
                "Mesh Material Flags Buffer (Resized)",
                (std::mem::size_of::<u32>() * num_materials) as u64,
            );
            self.gpu_mut().material_flags_buffer = new_buffer;
            self.gpu_mut().material_flags_buffer_size = num_materials;
            self.gpu_mut().gpu_batch_bind_group = None;
        }
        queue.write_buffer(
            &self.gpu().material_flags_buffer,
            0,
            bytemuck::cast_slice(&flags),
        );

        let lanes = 2 * super::super::types::NUM_DRAW_CLASSES;
        self.compute.batch_keys_per_lane = object_count.max(1);
        let dense_needed = lanes * num_meshes * num_materials
            + lanes
            + lanes * self.compute.batch_keys_per_lane * 2;
        if dense_needed > self.gpu().dense_capacity_size {
            let new_buffer = build::buffer(
                device,
                "Mesh Dense Capacity Buffer (Resized)",
                (std::mem::size_of::<u32>() * dense_needed) as u64,
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            );
            self.gpu_mut().dense_capacity_buffer = new_buffer;
            self.gpu_mut().dense_capacity_size = dense_needed;
            self.gpu_mut().gpu_batch_bind_group = None;
            self.gpu_mut().dense_capacity_clear_pending = true;
        }

        if total_slots > self.gpu().indirect_buffer_size {
            let gpu = self.gpu_mut();
            gpu.indirect_buffer = build::buffer(
                device,
                "Indirect Draw Buffer (GPU Batch)",
                (std::mem::size_of::<DrawIndexedIndirect>() * total_slots) as u64,
                wgpu::BufferUsages::INDIRECT
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
            );
            gpu.indirect_buffer_size = total_slots;
            gpu.indirect_reset_buffer = build::buffer(
                device,
                "Indirect Reset Buffer (GPU Batch)",
                (std::mem::size_of::<DrawIndexedIndirect>() * total_slots) as u64,
                wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::STORAGE,
            );
            gpu.batch_descs_buffer = build::storage(
                device,
                "Batch Descriptor Buffer (GPU Batch)",
                (std::mem::size_of::<BatchDesc>() * total_slots) as u64,
            );
            gpu.culling_bind_group = None;
            gpu.cluster_assign_bind_group = None;
            gpu.compaction_bind_group = None;
            gpu.indirect_build_bind_group = None;
            gpu.batch_assign_bind_group = None;
            gpu.gpu_batch_bind_group = None;
        }
        if total_slots > self.gpu().batch_key_buffer_size {
            self.gpu_mut().batch_key_buffer = build::storage(
                device,
                "Batch Key Buffer (GPU Batch)",
                (std::mem::size_of::<BatchKey>() * total_slots) as u64,
            );
            self.gpu_mut().batch_key_buffer_size = total_slots;
            self.gpu_mut().batch_assign_bind_group = None;
            self.gpu_mut().gpu_batch_bind_group = None;
        }

        let visible_size =
            (self.gpu().visible_indices_buffer.size() / std::mem::size_of::<u32>() as u64) as usize;
        if object_count > visible_size {
            let new_size = (object_count as f32 * BUFFER_GROWTH_FACTOR).ceil() as usize;
            self.gpu_mut().visible_indices_buffer = build::storage(
                device,
                "Visible Indices Buffer (GPU Batch)",
                (std::mem::size_of::<u32>() * new_size) as u64,
            );
            self.rebuild_instance_bind_group(device);
            self.gpu_mut().culling_bind_group = None;
        }

        {
            let state = self.state_mut();
            for list in [
                &mut state.opaque_instances,
                &mut state.opaque_double_sided_instances,
                &mut state.transparent_instances,
                &mut state.overlay_opaque_instances,
                &mut state.overlay_opaque_double_sided_instances,
                &mut state.overlay_transparent_instances,
                &mut state.instanced_opaque_batches,
                &mut state.instanced_opaque_double_sided_batches,
                &mut state.instanced_transparent_batches,
                &mut state.transmissive_instances,
            ] {
                hold_slots(list, cap);
            }
            state.indirect_reset_count = total_slots;
            state.prepass_batch_counts = [0; super::super::types::NUM_DRAW_CLASSES];
        }
        self.compute.batch_dirty = true;
    }

    pub(in super::super) fn build_lists_from_combos(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        if self.compute.batching_enabled {
            self.gpu_batch_setup(device, queue);
            return;
        }
        let mask_ids = self.state().cached_mask_material_ids.clone();

        let mut sorted: Vec<(u32, u32, u32, u32)> = self
            .state()
            .combos
            .iter()
            .map(|(&(class, mesh, material), &count)| (class, mesh, material, count))
            .collect();
        sorted.sort_by_key(|&(class, mesh, material, _)| {
            (
                class,
                u32::from(mask_ids.contains(&material)),
                mesh,
                material,
            )
        });

        let mut lists: [Vec<BatchRange>; NUM_DRAW_CLASSES] = Default::default();
        let mut descs: Vec<BatchDesc> = Vec::new();
        let mut keys: Vec<BatchKey> = Vec::new();
        let mut prepass_batch_counts = [0u32; NUM_DRAW_CLASSES];
        for (class, mesh, material, count) in sorted {
            let base_slot = descs.len() as u32;
            descs.push(BatchDesc {
                mesh_geo_id: mesh,
                capacity: count,
            });
            lists[class as usize].push((mesh, material, 0, 0));
            if !mask_ids.contains(&material) {
                prepass_batch_counts[class as usize] += 1;
            }
            keys.push(BatchKey {
                pipeline_class: class,
                mesh_id: mesh,
                material_id: material,
                base_slot,
            });
        }
        keys.sort_by_key(|key| (key.pipeline_class, key.mesh_id, key.material_id));

        let total_batch_count = descs.len();
        {
            let state = self.state_mut();
            state.opaque_instances = std::mem::take(&mut lists[0]);
            state.opaque_double_sided_instances = std::mem::take(&mut lists[1]);
            state.transparent_instances = std::mem::take(&mut lists[2]);
            state.overlay_opaque_instances = std::mem::take(&mut lists[3]);
            state.overlay_opaque_double_sided_instances = std::mem::take(&mut lists[4]);
            state.overlay_transparent_instances = std::mem::take(&mut lists[5]);
            state.instanced_opaque_batches = std::mem::take(&mut lists[6]);
            state.instanced_opaque_double_sided_batches = std::mem::take(&mut lists[7]);
            state.instanced_transparent_batches = std::mem::take(&mut lists[8]);
            state.transmissive_instances = std::mem::take(&mut lists[9]);
            state.indirect_reset_count = total_batch_count;
            state.prepass_batch_counts = prepass_batch_counts;
        }

        if total_batch_count > self.gpu().indirect_buffer_size {
            let new_size = grown_buffer_size(total_batch_count);
            let gpu_mut = self.gpu_mut();
            gpu_mut.indirect_buffer = build::buffer(
                device,
                "Indirect Draw Buffer (Resized)",
                (std::mem::size_of::<DrawIndexedIndirect>() * new_size) as u64,
                wgpu::BufferUsages::INDIRECT
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
            );
            gpu_mut.indirect_buffer_size = new_size;
            gpu_mut.indirect_reset_buffer = build::buffer(
                device,
                "Indirect Reset Buffer (Resized)",
                (std::mem::size_of::<DrawIndexedIndirect>() * new_size) as u64,
                wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::STORAGE,
            );
            gpu_mut.batch_descs_buffer = build::storage(
                device,
                "Batch Descriptor Buffer (Resized)",
                (std::mem::size_of::<BatchDesc>() * new_size) as u64,
            );
            gpu_mut.culling_bind_group = None;
            gpu_mut.cluster_assign_bind_group = None;
            gpu_mut.compaction_bind_group = None;
            gpu_mut.indirect_build_bind_group = None;
            gpu_mut.batch_assign_bind_group = None;
        }
        if keys.len() > self.gpu().batch_key_buffer_size {
            let new_size = (keys.len() as f32 * BUFFER_GROWTH_FACTOR).ceil() as usize;
            self.gpu_mut().batch_key_buffer = build::storage(
                device,
                "Batch Key Buffer (Resized)",
                (std::mem::size_of::<BatchKey>() * new_size) as u64,
            );
            self.gpu_mut().batch_key_buffer_size = new_size;
            self.gpu_mut().batch_assign_bind_group = None;
        }

        let total_visible_slots: usize = descs.iter().map(|d| d.capacity as usize).sum();
        let visible_indices_size =
            (self.gpu().visible_indices_buffer.size() / std::mem::size_of::<u32>() as u64) as usize;
        if total_visible_slots > visible_indices_size {
            let new_size = (total_visible_slots as f32 * BUFFER_GROWTH_FACTOR).ceil() as usize;
            self.gpu_mut().visible_indices_buffer = build::storage(
                device,
                "Visible Indices Buffer (Resized)",
                (std::mem::size_of::<u32>() * new_size) as u64,
            );
            self.rebuild_instance_bind_group(device);
            self.gpu_mut().culling_bind_group = None;
        }

        if !descs.is_empty() {
            queue.write_buffer(
                &self.gpu().batch_descs_buffer,
                0,
                bytemuck::cast_slice(&descs),
            );
        }
        if !keys.is_empty() {
            queue.write_buffer(&self.gpu().batch_key_buffer, 0, bytemuck::cast_slice(&keys));
        }
        self.gpu_mut().batch_key_count = keys.len() as u32;
    }

    pub(in super::super) fn can_do_rebatch_only(&self) -> bool {
        let Some(fd) = self.frame.dirty.as_ref() else {
            return false;
        };

        if !fd.frame_initialized
            || fd.full_rebuild_needed
            || fd.instanced_meshes_changed
            || !fd.entities_added.is_empty()
            || !fd.entities_removed.is_empty()
            || !fd.material_dirty.is_empty()
        {
            return false;
        }

        if !fd.batches_invalidated {
            return false;
        }

        let Some(world_state) = self.state.as_ref() else {
            return false;
        };

        !world_state.cached_entities.is_empty() && world_state.gpu_buffers.is_some()
    }

    pub(in super::super) fn rebatch_cached_entities(
        &mut self,
        configs: &nightshade_wgpu::backend::RenderInputs,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let dirty_transforms = self
            .frame
            .dirty
            .as_mut()
            .map(|fd| std::mem::take(&mut fd.transform_dirty))
            .unwrap_or_default();

        let world_state = self.state.as_mut().unwrap();
        let gpu = world_state.gpu_buffers.as_ref().unwrap();

        for entity in &dirty_transforms {
            if let Some(&slot) = world_state.gpu_registry.entity_to_slot.get(entity)
                && let Some(transform) =
                    configs
                        .scene_world
                        .get::<nightshade_render::components::render_world::Transform>(*entity)
            {
                let model_matrix = ModelMatrix {
                    model: transform.0.into(),
                    normal_matrix: [[0.0; 4]; 3],
                };
                if (slot as usize) < world_state.cached_transforms.len() {
                    world_state.cached_transforms[slot as usize] = model_matrix;
                }
                let offset = (slot as u64) * std::mem::size_of::<ModelMatrix>() as u64;
                queue.write_buffer(
                    &gpu.transform_buffer,
                    offset,
                    bytemuck::cast_slice(&[model_matrix]),
                );
            }
        }

        {
            let gpu = world_state.gpu_buffers.as_ref().unwrap();
            queue.write_buffer(
                &gpu.object_buffer,
                0,
                bytemuck::cast_slice(&world_state.cached_objects),
            );
        }

        self.build_lists_from_combos(device, queue);
    }

    pub(in super::super) fn draw_batches<'a>(
        pass: &mut wgpu::RenderPass<'a>,
        batch_count: usize,
        batch_offset: usize,
        class_index: usize,
        supports_count: bool,
        indirect_buffer: &'a wgpu::Buffer,
        count_buffer: &'a wgpu::Buffer,
    ) {
        if batch_count == 0 {
            return;
        }
        let indirect_offset = (batch_offset * std::mem::size_of::<DrawIndexedIndirect>()) as u64;
        if supports_count {
            pass.multi_draw_indexed_indirect_count(
                indirect_buffer,
                indirect_offset,
                count_buffer,
                (class_index * std::mem::size_of::<u32>()) as u64,
                batch_count as u32,
            );
        } else {
            pass.multi_draw_indexed_indirect(indirect_buffer, indirect_offset, batch_count as u32);
        }
    }

    pub(in super::super) fn compute_vertex_utilization(&self) -> f32 {
        let actual_vertex_count: u64 = self
            .registry
            .data
            .iter()
            .map(|m| m.vertex_count as u64)
            .sum();
        let actual_vertex_bytes = actual_vertex_count * std::mem::size_of::<Vertex>() as u64;
        if self.buffers.vertex_size > 0 {
            actual_vertex_bytes as f32 / self.buffers.vertex_size as f32
        } else {
            1.0
        }
    }

    pub(in super::super) fn compute_index_utilization(&self) -> f32 {
        let actual_index_count: u64 = self
            .registry
            .data
            .iter()
            .map(|m| m.index_count as u64)
            .sum();
        let actual_index_bytes = actual_index_count * std::mem::size_of::<u32>() as u64;
        if self.buffers.index_size > 0 {
            actual_index_bytes as f32 / self.buffers.index_size as f32
        } else {
            1.0
        }
    }

    pub(in super::super) fn reset_mesh_registry(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        self.registry.ids.clear();
        self.registry.data.clear();
        self.registry.names.clear();
        self.registry.geo_dirty = true;
        self.registry.rescan = true;
        self.registry.free_ids.clear();
        self.buffers.free_vertex.clear();
        self.buffers.free_index.clear();
        self.registry.bounds_data.clear();
        self.buffers.vertex_offset = 0;
        self.buffers.index_offset = 0;
        self.morph.offset = 0;

        self.add_mesh(device, queue, "Cube", &create_cube_mesh(), None);
        self.add_mesh(device, queue, "Sphere", &create_sphere_mesh(1.0, 16), None);
        self.add_mesh(device, queue, "Plane", &create_plane_mesh(2.0), None);
        self.add_mesh(
            device,
            queue,
            "SubdividedPlane",
            &create_subdivided_plane_mesh(2.0, 20),
            None,
        );
        self.add_mesh(
            device,
            queue,
            "Cylinder",
            &create_cylinder_mesh(0.5, 1.0, 16),
            None,
        );
        self.add_mesh(device, queue, "Cone", &create_cone_mesh(0.5, 1.0, 16), None);
        self.add_mesh(
            device,
            queue,
            "Torus",
            &create_torus_mesh(1.0, 0.3, 16, 16),
            None,
        );
        self.add_mesh(device, queue, "Quad", &create_quad_mesh(1.0), None);
    }

    pub(in super::super) fn check_and_compact_buffers(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh_cache: &nightshade_render::resources::mesh_cache::MeshCache,
    ) -> bool {
        let actual_vertex_count: u64 = self
            .registry
            .data
            .iter()
            .map(|m| m.vertex_count as u64)
            .sum();
        let actual_index_count: u64 = self
            .registry
            .data
            .iter()
            .map(|m| m.index_count as u64)
            .sum();

        let actual_vertex_bytes = actual_vertex_count * std::mem::size_of::<Vertex>() as u64;
        let actual_index_bytes = actual_index_count * std::mem::size_of::<u32>() as u64;

        let vertex_utilization = if self.buffers.vertex_size > 0 {
            actual_vertex_bytes as f32 / self.buffers.vertex_size as f32
        } else {
            1.0
        };

        let index_utilization = if self.buffers.index_size > 0 {
            actual_index_bytes as f32 / self.buffers.index_size as f32
        } else {
            1.0
        };

        let utilization_below = vertex_utilization < BUFFER_SHRINK_THRESHOLD
            || index_utilization < BUFFER_SHRINK_THRESHOLD;
        let reclaimable_bytes = self.buffers.vertex_size.saturating_sub(actual_vertex_bytes)
            + self.buffers.index_size.saturating_sub(actual_index_bytes);
        let should_compact = utilization_below && reclaimable_bytes >= COMPACTION_MIN_RECLAIM_BYTES;

        let cooled_down = self.frame.counter
            >= self.frame.last_compaction + COMPACTION_COOLDOWN_FRAMES
            || self.frame.last_compaction == 0;
        let did_compact =
            should_compact && cooled_down && actual_vertex_bytes > 0 && actual_index_bytes > 0;
        if did_compact {
            self.frame.last_compaction = self.frame.counter;
            self.reset_mesh_registry(device, queue);

            for (name, mesh) in mesh_cache_iter(mesh_cache) {
                if !self.registry.ids.contains_key(
                    &nightshade_render::data::generational_registry::registry_name_hash(name),
                ) {
                    self.add_mesh(device, queue, name, mesh, None);
                }
            }

            let final_vertex_bytes =
                self.buffers.vertex_offset as u64 * std::mem::size_of::<Vertex>() as u64;
            let final_index_bytes =
                self.buffers.index_offset as u64 * std::mem::size_of::<u32>() as u64;
            let new_vertex_size = (final_vertex_bytes as f32 * BUFFER_GROWTH_FACTOR).ceil() as u64;
            let new_index_size = (final_index_bytes as f32 * BUFFER_GROWTH_FACTOR).ceil() as u64;

            if new_vertex_size < self.buffers.vertex_size && final_vertex_bytes > 0 {
                let new_buffer = build::buffer(
                    device,
                    "Mesh Vertex Buffer (Compacted)",
                    new_vertex_size,
                    wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                );

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Mesh Vertex Buffer Compaction"),
                });

                encoder.copy_buffer_to_buffer(
                    &self.buffers.vertex,
                    0,
                    &new_buffer,
                    0,
                    final_vertex_bytes,
                );

                queue.submit(std::iter::once(encoder.finish()));
                self.buffers.vertex = new_buffer;
                self.buffers.vertex_size = new_vertex_size;
            }

            if new_index_size < self.buffers.index_size && final_index_bytes > 0 {
                let new_buffer = build::buffer(
                    device,
                    "Mesh Index Buffer (Compacted)",
                    new_index_size,
                    wgpu::BufferUsages::INDEX
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                );

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Mesh Index Buffer Compaction"),
                });

                encoder.copy_buffer_to_buffer(
                    &self.buffers.index,
                    0,
                    &new_buffer,
                    0,
                    final_index_bytes,
                );

                queue.submit(std::iter::once(encoder.finish()));
                self.buffers.index = new_buffer;
                self.buffers.index_size = new_index_size;
            }
        }
        did_compact
    }
}
