use crate::geometry::{
    Vertex, create_cone_mesh, create_cube_mesh, create_cylinder_mesh, create_plane_mesh,
    create_sphere_mesh, create_subdivided_plane_mesh, create_torus_mesh,
};
use crate::mesh_cache::mesh_cache_iter;

use super::super::types::{
    BUFFER_GROWTH_FACTOR, BUFFER_SHRINK_THRESHOLD, BatchDesc, BatchKey,
    COMPACTION_MIN_RECLAIM_BYTES, DrawIndexedIndirect, MAX_INSTANCES, ModelMatrix,
    NUM_DRAW_CLASSES,
};
use super::super::world_state::BatchRange;
use super::MeshPass;

impl MeshPass {
    /// Prepares the GPU-driven batch build (no readback). Uploads the per
    /// material flag table and the per-mesh LOD geo table, sizes the dense /
    /// fixed-region indirect / descriptor / key / visible buffers, sets the
    /// per-class draw offsets to the fixed regions, and marks the table dirty so
    /// the execute pass runs clear/count/build. The GPU emits the batch tables
    /// and per-class counts directly; the CPU never reads them back.
    pub(in super::super) fn gpu_batch_setup(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let num_materials = self.state().cached_materials_data.len().max(1);
        let num_meshes = self.mesh_data.len().max(1);
        let object_count = self.state().object_count as usize;

        let lod_expanded_slots: usize = (0..num_meshes)
            .map(|mesh_index| {
                self.mesh_lod_mesh_ids
                    .get(mesh_index)
                    .map(|ids| ids.len().clamp(1, 4))
                    .unwrap_or(1)
            })
            .sum();
        let worst_case_per_class =
            (lod_expanded_slots * num_materials).min(object_count.max(1) * 4);
        let required_cap = worst_case_per_class.max(super::super::types::MAX_BATCHES_PER_CLASS);
        self.gpu_batch_cap = self.gpu_batch_cap.max(required_cap.next_power_of_two());
        let cap = self.gpu_batch_cap;
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
        }
        if num_materials > self.gpu().material_flags_buffer_size {
            let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Mesh Material Flags Buffer (Resized)"),
                size: (std::mem::size_of::<u32>() * num_materials) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.gpu_mut().material_flags_buffer = new_buffer;
            self.gpu_mut().material_flags_buffer_size = num_materials;
            self.gpu_mut().gpu_batch_bind_group = None;
        }
        queue.write_buffer(
            &self.gpu().material_flags_buffer,
            0,
            bytemuck::cast_slice(&flags),
        );

        let mut lod_geo = vec![
            super::super::types::MeshLodGeoData {
                lod_count: 1,
                geo: [0; 4]
            };
            num_meshes
        ];
        for (mesh_index, entry) in lod_geo.iter_mut().enumerate() {
            match self
                .mesh_lod_mesh_ids
                .get(mesh_index)
                .filter(|ids| !ids.is_empty())
            {
                Some(ids) => {
                    let mut geo = [0u32; 4];
                    for (level, &id) in ids.iter().take(4).enumerate() {
                        geo[level] = id;
                    }
                    entry.lod_count = ids.len().min(4) as u32;
                    entry.geo = geo;
                }
                None => {
                    entry.lod_count = 1;
                    entry.geo = [mesh_index as u32, 0, 0, 0];
                }
            }
        }
        if num_meshes > self.mesh_lod_geo_buffer_size {
            self.mesh_lod_geo_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Mesh LOD Geo Buffer (Resized)"),
                size: (std::mem::size_of::<super::super::types::MeshLodGeoData>() * num_meshes)
                    as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.mesh_lod_geo_buffer_size = num_meshes;
            self.gpu_mut().gpu_batch_bind_group = None;
        }
        queue.write_buffer(&self.mesh_lod_geo_buffer, 0, bytemuck::cast_slice(&lod_geo));

        let dense_needed = 18 * num_meshes * num_materials;
        if dense_needed > self.gpu().dense_capacity_size {
            let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Mesh Dense Capacity Buffer (Resized)"),
                size: (std::mem::size_of::<u32>() * dense_needed) as u64,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            self.gpu_mut().dense_capacity_buffer = new_buffer;
            self.gpu_mut().dense_capacity_size = dense_needed;
            self.gpu_mut().gpu_batch_bind_group = None;
        }

        if total_slots > self.gpu().indirect_buffer_size {
            let gpu = self.gpu_mut();
            gpu.indirect_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Indirect Draw Buffer (GPU Batch)"),
                size: (std::mem::size_of::<DrawIndexedIndirect>() * total_slots) as u64,
                usage: wgpu::BufferUsages::INDIRECT
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            gpu.indirect_buffer_size = total_slots;
            gpu.indirect_reset_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Indirect Reset Buffer (GPU Batch)"),
                size: (std::mem::size_of::<DrawIndexedIndirect>() * total_slots) as u64,
                usage: wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            });
            gpu.batch_descs_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Batch Descriptor Buffer (GPU Batch)"),
                size: (std::mem::size_of::<BatchDesc>() * total_slots) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            gpu.culling_bind_group = None;
            gpu.cluster_assign_bind_group = None;
            gpu.compaction_bind_group = None;
            gpu.indirect_build_bind_group = None;
            gpu.batch_assign_bind_group = None;
            gpu.gpu_batch_bind_group = None;
        }
        if total_slots > self.gpu().batch_key_buffer_size {
            self.gpu_mut().batch_key_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Batch Key Buffer (GPU Batch)"),
                size: (std::mem::size_of::<BatchKey>() * total_slots) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.gpu_mut().batch_key_buffer_size = total_slots;
            self.gpu_mut().batch_assign_bind_group = None;
            self.gpu_mut().gpu_batch_bind_group = None;
        }

        let visible_size =
            (self.gpu().visible_indices_buffer.size() / std::mem::size_of::<u32>() as u64) as usize;
        if object_count > visible_size {
            let new_size = (object_count as f32 * BUFFER_GROWTH_FACTOR).ceil() as usize;
            self.gpu_mut().visible_indices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Visible Indices Buffer (GPU Batch)"),
                size: (std::mem::size_of::<u32>() * new_size) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.rebuild_instance_bind_group(device);
            self.gpu_mut().culling_bind_group = None;
        }

        let dummy: Vec<BatchRange> = vec![(0, 0, 0, 0); cap];
        {
            let state = self.state_mut();
            state.opaque_instances = dummy.clone();
            state.opaque_double_sided_instances = dummy.clone();
            state.transparent_instances = dummy.clone();
            state.overlay_opaque_instances = dummy.clone();
            state.overlay_opaque_double_sided_instances = dummy.clone();
            state.overlay_transparent_instances = dummy.clone();
            state.instanced_opaque_batches = dummy.clone();
            state.instanced_opaque_double_sided_batches = dummy.clone();
            state.instanced_transparent_batches = dummy;
            state.indirect_reset_count = total_slots;
            state.prepass_batch_counts = [0; super::super::types::NUM_DRAW_CLASSES];
        }
        self.gpu_batch_dirty = true;
    }

    pub(in super::super) fn build_lists_from_combos(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        if self.gpu_batching_enabled {
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
            let added = match self
                .mesh_lod_mesh_ids
                .get(mesh as usize)
                .filter(|ids| !ids.is_empty())
            {
                Some(lod_ids) => {
                    for &lod_mesh in lod_ids {
                        descs.push(BatchDesc {
                            mesh_geo_id: lod_mesh,
                            capacity: count,
                        });
                        lists[class as usize].push((lod_mesh, material, 0, 0));
                    }
                    lod_ids.len() as u32
                }
                None => {
                    descs.push(BatchDesc {
                        mesh_geo_id: mesh,
                        capacity: count,
                    });
                    lists[class as usize].push((mesh, material, 0, 0));
                    1
                }
            };
            if !mask_ids.contains(&material) {
                prepass_batch_counts[class as usize] += added;
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
            state.indirect_reset_count = total_batch_count;
            state.prepass_batch_counts = prepass_batch_counts;
        }

        if total_batch_count > self.gpu().indirect_buffer_size {
            let new_size = std::cmp::min(
                (total_batch_count as f32 * BUFFER_GROWTH_FACTOR).ceil() as usize,
                MAX_INSTANCES,
            );
            let gpu_mut = self.gpu_mut();
            gpu_mut.indirect_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Indirect Draw Buffer (Resized)"),
                size: (std::mem::size_of::<DrawIndexedIndirect>() * new_size) as u64,
                usage: wgpu::BufferUsages::INDIRECT
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            gpu_mut.indirect_buffer_size = new_size;
            gpu_mut.indirect_reset_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Indirect Reset Buffer (Resized)"),
                size: (std::mem::size_of::<DrawIndexedIndirect>() * new_size) as u64,
                usage: wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            });
            gpu_mut.batch_descs_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Batch Descriptor Buffer (Resized)"),
                size: (std::mem::size_of::<BatchDesc>() * new_size) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            gpu_mut.culling_bind_group = None;
            gpu_mut.cluster_assign_bind_group = None;
            gpu_mut.compaction_bind_group = None;
            gpu_mut.indirect_build_bind_group = None;
            gpu_mut.batch_assign_bind_group = None;
        }
        if keys.len() > self.gpu().batch_key_buffer_size {
            let new_size = (keys.len() as f32 * BUFFER_GROWTH_FACTOR).ceil() as usize;
            self.gpu_mut().batch_key_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Batch Key Buffer (Resized)"),
                size: (std::mem::size_of::<BatchKey>() * new_size) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.gpu_mut().batch_key_buffer_size = new_size;
            self.gpu_mut().batch_assign_bind_group = None;
        }

        let total_visible_slots: usize = descs.iter().map(|d| d.capacity as usize).sum();
        let visible_indices_size =
            (self.gpu().visible_indices_buffer.size() / std::mem::size_of::<u32>() as u64) as usize;
        if total_visible_slots > visible_indices_size {
            let new_size = (total_visible_slots as f32 * BUFFER_GROWTH_FACTOR).ceil() as usize;
            self.gpu_mut().visible_indices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Visible Indices Buffer (Resized)"),
                size: (std::mem::size_of::<u32>() * new_size) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
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
        let Some(fd) = self.frame_dirty.as_ref() else {
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

        let Some(world_state) = self.world_state_get(self.current_world_id) else {
            return false;
        };

        !world_state.cached_entities.is_empty() && world_state.gpu_buffers.is_some()
    }

    pub(in super::super) fn rebatch_cached_entities(
        &mut self,
        configs: &crate::wgpu::render_configs::RenderInputs,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let dirty_transforms = self
            .frame_dirty
            .as_mut()
            .map(|fd| std::mem::take(&mut fd.transform_dirty))
            .unwrap_or_default();

        let world_state = self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap();
        let gpu = world_state.gpu_buffers.as_ref().unwrap();

        for entity in &dirty_transforms {
            if let Some(&slot) = world_state.gpu_registry.entity_to_slot.get(entity)
                && let Some(dynamic) = configs.scene.render_dynamic_objects.get(entity)
            {
                let model_matrix = ModelMatrix {
                    model: dynamic.transform.into(),
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
        let actual_vertex_count: u64 = self.mesh_data.iter().map(|m| m.vertex_count as u64).sum();
        let actual_vertex_bytes = actual_vertex_count * std::mem::size_of::<Vertex>() as u64;
        if self.vertex_buffer_size > 0 {
            actual_vertex_bytes as f32 / self.vertex_buffer_size as f32
        } else {
            1.0
        }
    }

    pub(in super::super) fn compute_index_utilization(&self) -> f32 {
        let actual_index_count: u64 = self.mesh_data.iter().map(|m| m.index_count as u64).sum();
        let actual_index_bytes = actual_index_count * std::mem::size_of::<u32>() as u64;
        if self.index_buffer_size > 0 {
            actual_index_bytes as f32 / self.index_buffer_size as f32
        } else {
            1.0
        }
    }

    /// Clears the GPU geometry registry (vertex/index buffers offsets, mesh
    /// data, name and bounds tables) and re-registers the built-in primitives.
    /// Used both by buffer compaction and when the mesh cache is wholesale
    /// cleared so replaced models do not leave stale geometry behind. Callers
    /// must trigger a full rebuild afterwards because every mesh id changes.
    pub(in super::super) fn reset_mesh_registry(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        self.meshes.clear();
        self.mesh_data.clear();
        self.mesh_names.clear();
        self.mesh_geo_dirty = true;
        self.mesh_bounds_data.clear();
        self.mesh_lod_data.clear();
        for entry in &mut self.mesh_lod_mesh_ids {
            entry.clear();
        }
        self.current_vertex_offset = 0;
        self.current_index_offset = 0;
        self.current_morph_displacement_offset = 0;

        self.add_mesh(device, queue, "Cube", create_cube_mesh());
        self.add_mesh(device, queue, "Sphere", create_sphere_mesh(1.0, 16));
        self.add_mesh(device, queue, "Sphere_LOD1", create_sphere_mesh(1.0, 8));
        self.add_mesh(device, queue, "Sphere_LOD2", create_sphere_mesh(1.0, 4));
        self.add_mesh(device, queue, "Plane", create_plane_mesh(2.0));
        self.add_mesh(
            device,
            queue,
            "SubdividedPlane",
            create_subdivided_plane_mesh(2.0, 20),
        );
        self.add_mesh(
            device,
            queue,
            "Cylinder",
            create_cylinder_mesh(0.5, 1.0, 16),
        );
        self.add_mesh(device, queue, "Cone", create_cone_mesh(0.5, 1.0, 16));
        self.add_mesh(device, queue, "Torus", create_torus_mesh(1.0, 0.3, 16, 16));
    }

    pub(in super::super) fn check_and_compact_buffers(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh_cache: &crate::mesh_cache::MeshCache,
    ) -> bool {
        let actual_vertex_count: u64 = self.mesh_data.iter().map(|m| m.vertex_count as u64).sum();
        let actual_index_count: u64 = self.mesh_data.iter().map(|m| m.index_count as u64).sum();

        let actual_vertex_bytes = actual_vertex_count * std::mem::size_of::<Vertex>() as u64;
        let actual_index_bytes = actual_index_count * std::mem::size_of::<u32>() as u64;

        let vertex_utilization = if self.vertex_buffer_size > 0 {
            actual_vertex_bytes as f32 / self.vertex_buffer_size as f32
        } else {
            1.0
        };

        let index_utilization = if self.index_buffer_size > 0 {
            actual_index_bytes as f32 / self.index_buffer_size as f32
        } else {
            1.0
        };

        let utilization_below = vertex_utilization < BUFFER_SHRINK_THRESHOLD
            || index_utilization < BUFFER_SHRINK_THRESHOLD;
        let reclaimable_bytes = self.vertex_buffer_size.saturating_sub(actual_vertex_bytes)
            + self.index_buffer_size.saturating_sub(actual_index_bytes);
        let should_compact = utilization_below && reclaimable_bytes >= COMPACTION_MIN_RECLAIM_BYTES;

        let did_compact = should_compact && actual_vertex_bytes > 0 && actual_index_bytes > 0;
        if did_compact {
            self.reset_mesh_registry(device, queue);

            for (name, mesh) in mesh_cache_iter(mesh_cache) {
                if !self.meshes.contains_key(name) {
                    self.add_mesh(device, queue, name, mesh.clone());
                }
            }

            let final_vertex_bytes =
                self.current_vertex_offset as u64 * std::mem::size_of::<Vertex>() as u64;
            let final_index_bytes =
                self.current_index_offset as u64 * std::mem::size_of::<u32>() as u64;
            let new_vertex_size = (final_vertex_bytes as f32 * BUFFER_GROWTH_FACTOR).ceil() as u64;
            let new_index_size = (final_index_bytes as f32 * BUFFER_GROWTH_FACTOR).ceil() as u64;

            if new_vertex_size < self.vertex_buffer_size && final_vertex_bytes > 0 {
                let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Mesh Vertex Buffer (Compacted)"),
                    size: new_vertex_size,
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                });

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Mesh Vertex Buffer Compaction"),
                });

                encoder.copy_buffer_to_buffer(
                    &self.vertex_buffer,
                    0,
                    &new_buffer,
                    0,
                    final_vertex_bytes,
                );

                queue.submit(std::iter::once(encoder.finish()));
                self.vertex_buffer = new_buffer;
                self.vertex_buffer_generation += 1;
                self.vertex_buffer_size = new_vertex_size;
            }

            if new_index_size < self.index_buffer_size && final_index_bytes > 0 {
                let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Mesh Index Buffer (Compacted)"),
                    size: new_index_size,
                    usage: wgpu::BufferUsages::INDEX
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                });

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Mesh Index Buffer Compaction"),
                });

                encoder.copy_buffer_to_buffer(
                    &self.index_buffer,
                    0,
                    &new_buffer,
                    0,
                    final_index_bytes,
                );

                queue.submit(std::iter::once(encoder.finish()));
                self.index_buffer = new_buffer;
                self.index_buffer_size = new_index_size;
            }
        }
        did_compact
    }
}
