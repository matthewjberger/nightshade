use crate::generational_registry::*;
use crate::geometry::{Mesh, Vertex};
use crate::mesh_cache::mesh_cache_iter;

use super::super::types::{
    BUFFER_GROWTH_FACTOR, MeshBoundsData, MeshData, MeshLodInfo, MorphDisplacement,
};

use super::MeshPass;

impl MeshPass {
    pub fn register_texture(
        &mut self,
        name: String,
        view: wgpu::TextureView,
        sampler: wgpu::Sampler,
    ) {
        self.registered_textures.insert(name, (view, sampler));
    }

    pub fn mesh_id_for_name(&self, name: &str) -> Option<u32> {
        self.meshes.get(name).copied()
    }

    /// Returns a write target for compute passes that deform a registered
    /// mesh in place: a clone of the shared vertex buffer, the buffer's
    /// generation (bumped whenever the buffer is recreated, so callers know
    /// to rebuild bind groups), and the mesh's vertex offset in elements.
    pub fn vertex_write_target(&self, name: &str) -> Option<(wgpu::Buffer, u64, u32)> {
        let mesh_id = *self.meshes.get(name)?;
        let mesh_data = self.mesh_data.get(mesh_id as usize)?;
        if mesh_data.vertex_count == 0 {
            return None;
        }
        Some((
            self.vertex_buffer.clone(),
            self.vertex_buffer_generation,
            mesh_data.vertex_offset,
        ))
    }

    /// Replaces the culling bounds registered for a mesh. Used by passes
    /// that deform a mesh's vertices on the GPU and need the cull sphere to
    /// cover the full deformation range rather than the rest pose.
    pub fn override_mesh_bounds(
        &mut self,
        queue: &wgpu::Queue,
        name: &str,
        center: [f32; 3],
        radius: f32,
    ) {
        let Some(&mesh_id) = self.meshes.get(name) else {
            return;
        };
        let Some(bounds) = self.mesh_bounds_data.get_mut(mesh_id as usize) else {
            return;
        };
        if bounds.center == center && bounds.radius == radius {
            return;
        }
        bounds.center = center;
        bounds.radius = radius;
        queue.write_buffer(
            &self.mesh_bounds_buffer,
            (mesh_id as usize * std::mem::size_of::<MeshBoundsData>()) as u64,
            bytemuck::bytes_of(bounds),
        );
    }

    pub fn register_texture_with_data(
        &mut self,
        name: String,
        view: wgpu::TextureView,
        sampler: wgpu::Sampler,
    ) {
        self.registered_textures.insert(name, (view, sampler));
    }

    pub fn add_material_layer_mapping(
        &mut self,
        texture: crate::asset_id::TextureId,
        layer: crate::wgpu::material_texture_arrays::MaterialTextureLayer,
    ) {
        self.material_layer_map.insert(texture, layer);
    }

    pub fn unregister_texture(&mut self, name: &str) {
        self.registered_textures.remove(name);
    }

    pub fn sync_textures(&mut self, texture_cache: &crate::wgpu::texture_cache::TextureCache) {
        if texture_cache.registry.name_to_index.is_empty() {
            return;
        }
        self.registered_textures
            .retain(|name, _| texture_cache.registry.name_to_index.contains_key(name));
    }

    pub fn sync_meshes_from_cache(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh_cache: &crate::mesh_cache::MeshCache,
    ) {
        if mesh_cache.clear_generation != self.last_mesh_cache_clear_generation {
            self.last_mesh_cache_clear_generation = mesh_cache.clear_generation;
            self.reset_mesh_registry(device, queue);
            self.dirty_mesh_ids.clear();
            if let Some(frame_dirty) = self.frame_dirty.as_mut() {
                frame_dirty.full_rebuild_needed = true;
            }
        }

        let mut evicted = false;
        for mesh_id in std::mem::take(&mut self.dirty_mesh_ids) {
            let Some(name) = self.mesh_names.get(mesh_id as usize) else {
                continue;
            };
            if let Some(mesh) = registry_entry_by_name(&mesh_cache.registry, name) {
                let mesh_data = &mut self.mesh_data[mesh_id as usize];
                if mesh.vertices.len() as u32 == mesh_data.vertex_count {
                    let vertex_data = bytemuck::cast_slice(&mesh.vertices);
                    let vertex_offset_bytes =
                        (mesh_data.vertex_offset as usize * std::mem::size_of::<Vertex>()) as u64;
                    queue.write_buffer(&self.vertex_buffer, vertex_offset_bytes, vertex_data);

                    if let Some(morph_targets) = &mesh.morph_targets {
                        let vertex_count = mesh.vertices.len();
                        let target_count = morph_targets.targets.len().min(8);
                        let total_displacements = vertex_count * target_count;

                        if total_displacements > 0 {
                            if mesh_data.morph_target_count == 0 {
                                mesh_data.morph_displacement_offset =
                                    self.current_morph_displacement_offset;
                                mesh_data.morph_target_count = target_count as u32;
                                self.current_morph_displacement_offset +=
                                    total_displacements as u32;
                            }

                            let mut displacements = Vec::with_capacity(total_displacements);

                            for target_index in 0..target_count {
                                let target = &morph_targets.targets[target_index];
                                for vertex_index in 0..vertex_count {
                                    let position = target
                                        .position_displacements
                                        .get(vertex_index)
                                        .copied()
                                        .unwrap_or([0.0, 0.0, 0.0]);
                                    let normal = target
                                        .normal_displacements
                                        .as_ref()
                                        .and_then(|n| n.get(vertex_index))
                                        .copied()
                                        .unwrap_or([0.0, 0.0, 0.0]);
                                    let tangent = target
                                        .tangent_displacements
                                        .as_ref()
                                        .and_then(|t| t.get(vertex_index))
                                        .copied()
                                        .unwrap_or([0.0, 0.0, 0.0]);

                                    displacements.push(MorphDisplacement {
                                        position,
                                        _pad0: 0.0,
                                        normal,
                                        _pad1: 0.0,
                                        tangent,
                                        _pad2: 0.0,
                                    });
                                }
                            }

                            let displacement_data = bytemuck::cast_slice(&displacements);
                            let displacement_offset_bytes = mesh_data.morph_displacement_offset
                                as u64
                                * std::mem::size_of::<MorphDisplacement>() as u64;
                            let required_size =
                                displacement_offset_bytes + displacement_data.len() as u64;

                            if required_size > self.morph_displacement_buffer_size {
                                let new_size = (required_size as f64 * BUFFER_GROWTH_FACTOR as f64)
                                    .ceil() as u64;
                                let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                                    label: Some("Mesh Morph Displacement Buffer (Resized)"),
                                    size: new_size,
                                    usage: wgpu::BufferUsages::STORAGE
                                        | wgpu::BufferUsages::COPY_DST
                                        | wgpu::BufferUsages::COPY_SRC,
                                    mapped_at_creation: false,
                                });

                                if displacement_offset_bytes > 0 {
                                    let mut encoder = device.create_command_encoder(
                                        &wgpu::CommandEncoderDescriptor {
                                            label: Some("Morph Displacement Buffer Reallocation"),
                                        },
                                    );
                                    encoder.copy_buffer_to_buffer(
                                        &self.morph_displacement_buffer,
                                        0,
                                        &new_buffer,
                                        0,
                                        displacement_offset_bytes,
                                    );
                                    queue.submit(std::iter::once(encoder.finish()));
                                }

                                self.morph_displacement_buffer = new_buffer;
                                self.morph_displacement_buffer_size = new_size;
                                self.invalidate_all_instance_bind_groups(device);
                            }

                            queue.write_buffer(
                                &self.morph_displacement_buffer,
                                displacement_offset_bytes,
                                displacement_data,
                            );
                        }
                    }

                    continue;
                }
            }
            let stale_name = self
                .mesh_names
                .get(mesh_id as usize)
                .cloned()
                .unwrap_or_default();
            if !stale_name.is_empty() && self.meshes.remove(&stale_name).is_some() {
                if let Some(slot) = self.mesh_data.get_mut(mesh_id as usize) {
                    slot.vertex_count = 0;
                    slot.index_count = 0;
                    slot.morph_target_count = 0;
                }
                if let Some(name_slot) = self.mesh_names.get_mut(mesh_id as usize) {
                    name_slot.clear();
                }
                self.mesh_geo_dirty = true;
                evicted = true;
            }
        }

        if evicted
            && self.check_and_compact_buffers(device, queue, mesh_cache)
            && let Some(frame_dirty) = self.frame_dirty.as_mut()
        {
            frame_dirty.full_rebuild_needed = true;
        }

        let mut meshes_to_add = Vec::new();

        for (name, mesh) in mesh_cache_iter(mesh_cache) {
            if !self.meshes.contains_key(name) {
                meshes_to_add.push((name.clone(), mesh.clone()));
            }
        }

        for (name, mesh) in meshes_to_add {
            self.add_mesh(device, queue, &name, mesh);
        }
    }

    pub(in super::super) fn add_mesh(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        name: &str,
        mesh: Mesh,
    ) {
        let max_buffer_size = device.limits().max_buffer_size;
        let mesh_id = self.mesh_data.len() as u32;

        let vertex_data = bytemuck::cast_slice(&mesh.vertices);
        let vertex_offset_bytes =
            (self.current_vertex_offset as usize * std::mem::size_of::<Vertex>()) as u64;
        let required_vertex_size = vertex_offset_bytes + vertex_data.len() as u64;

        if required_vertex_size > max_buffer_size {
            tracing::warn!(
                "skipping mesh '{}' — vertex data ({} bytes) would exceed max buffer size ({} bytes)",
                name,
                required_vertex_size,
                max_buffer_size
            );
            return;
        }

        if required_vertex_size > self.vertex_buffer_size {
            let new_size = (required_vertex_size as f32 * BUFFER_GROWTH_FACTOR).ceil() as u64;
            let new_size = new_size.min(max_buffer_size);

            let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Mesh Vertex Buffer (Resized)"),
                size: new_size,
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Mesh Vertex Buffer Reallocation"),
            });

            let used_size = ((self.current_vertex_offset as usize * std::mem::size_of::<Vertex>())
                as u64)
                .min(self.vertex_buffer_size);
            if used_size > 0 {
                encoder.copy_buffer_to_buffer(&self.vertex_buffer, 0, &new_buffer, 0, used_size);
            }

            queue.submit(std::iter::once(encoder.finish()));
            self.vertex_buffer = new_buffer;
            self.vertex_buffer_size = new_size;
            self.vertex_buffer_generation += 1;
        }

        queue.write_buffer(&self.vertex_buffer, vertex_offset_bytes, vertex_data);

        let index_data = bytemuck::cast_slice(&mesh.indices);
        let index_offset_bytes =
            (self.current_index_offset as usize * std::mem::size_of::<u32>()) as u64;
        let required_index_size = index_offset_bytes + index_data.len() as u64;

        if required_index_size > max_buffer_size {
            tracing::warn!(
                "skipping mesh '{}' — index data ({} bytes) would exceed max buffer size ({} bytes)",
                name,
                required_index_size,
                max_buffer_size
            );
            return;
        }

        if required_index_size > self.index_buffer_size {
            let new_size = (required_index_size as f32 * BUFFER_GROWTH_FACTOR).ceil() as u64;
            let new_size = new_size.min(max_buffer_size);

            let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Mesh Index Buffer (Resized)"),
                size: new_size,
                usage: wgpu::BufferUsages::INDEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Mesh Index Buffer Reallocation"),
            });

            let used_size = ((self.current_index_offset as usize * std::mem::size_of::<u32>())
                as u64)
                .min(self.index_buffer_size);
            if used_size > 0 {
                encoder.copy_buffer_to_buffer(&self.index_buffer, 0, &new_buffer, 0, used_size);
            }

            queue.submit(std::iter::once(encoder.finish()));
            self.index_buffer = new_buffer;
            self.index_buffer_size = new_size;
        }

        queue.write_buffer(&self.index_buffer, index_offset_bytes, index_data);

        let (bounding_center, bounding_radius) = {
            let mut min_x = f32::MAX;
            let mut min_y = f32::MAX;
            let mut min_z = f32::MAX;
            let mut max_x = f32::MIN;
            let mut max_y = f32::MIN;
            let mut max_z = f32::MIN;

            for vertex in &mesh.vertices {
                min_x = min_x.min(vertex.position[0]);
                min_y = min_y.min(vertex.position[1]);
                min_z = min_z.min(vertex.position[2]);
                max_x = max_x.max(vertex.position[0]);
                max_y = max_y.max(vertex.position[1]);
                max_z = max_z.max(vertex.position[2]);
            }

            let center_x = (min_x + max_x) * 0.5;
            let center_y = (min_y + max_y) * 0.5;
            let center_z = (min_z + max_z) * 0.5;

            let mut max_dist_sq = 0.0f32;
            for vertex in &mesh.vertices {
                let dx = vertex.position[0] - center_x;
                let dy = vertex.position[1] - center_y;
                let dz = vertex.position[2] - center_z;
                let dist_sq = dx * dx + dy * dy + dz * dz;
                max_dist_sq = max_dist_sq.max(dist_sq);
            }

            ([center_x, center_y, center_z], max_dist_sq.sqrt())
        };

        let (morph_displacement_offset, morph_target_count) =
            if let Some(morph_targets) = &mesh.morph_targets {
                let target_count = morph_targets.targets.len().min(8) as u32;
                let offset = self.current_morph_displacement_offset;
                (offset, target_count)
            } else {
                (0, 0)
            };

        let mesh_data = MeshData {
            vertex_offset: self.current_vertex_offset,
            vertex_count: mesh.vertices.len() as u32,
            index_offset: self.current_index_offset,
            index_count: mesh.indices.len() as u32,
            morph_displacement_offset,
            morph_target_count,
        };

        self.meshes.insert(name.to_string(), mesh_id);
        self.mesh_data.push(mesh_data);
        self.mesh_names.push(name.to_string());
        self.mesh_geo_dirty = true;

        if mesh_id as usize >= self.mesh_bounds_buffer_size {
            let new_size = ((mesh_id as usize + 1) as f32 * BUFFER_GROWTH_FACTOR).ceil() as usize;
            let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Mesh Bounds Buffer (Resized)"),
                size: (std::mem::size_of::<MeshBoundsData>() * new_size) as u64,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Mesh Bounds Buffer Reallocation"),
            });

            let used_size = (mesh_id as usize * std::mem::size_of::<MeshBoundsData>()) as u64;
            if used_size > 0 {
                encoder.copy_buffer_to_buffer(
                    &self.mesh_bounds_buffer,
                    0,
                    &new_buffer,
                    0,
                    used_size,
                );
            }

            queue.submit(std::iter::once(encoder.finish()));
            self.mesh_bounds_buffer = new_buffer;
            self.mesh_bounds_buffer_size = new_size;
            if let Some(world_state) = self.world_state_get_mut(self.current_world_id)
                && let Some(gpu) = world_state.gpu_buffers.as_mut()
            {
                gpu.culling_bind_group = None;
            }
        }

        let bounds_data = MeshBoundsData {
            center: bounding_center,
            radius: bounding_radius,
        };
        queue.write_buffer(
            &self.mesh_bounds_buffer,
            (mesh_id as usize * std::mem::size_of::<MeshBoundsData>()) as u64,
            bytemuck::bytes_of(&bounds_data),
        );
        self.mesh_bounds_data.push(bounds_data);

        if mesh_id as usize >= self.mesh_lod_buffer_size {
            let new_size = ((mesh_id as usize + 1) as f32 * BUFFER_GROWTH_FACTOR).ceil() as usize;
            let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Mesh LOD Buffer (Resized)"),
                size: (std::mem::size_of::<MeshLodInfo>() * new_size) as u64,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Mesh LOD Buffer Reallocation"),
            });

            let used_size = (mesh_id as usize * std::mem::size_of::<MeshLodInfo>()) as u64;
            if used_size > 0 {
                encoder.copy_buffer_to_buffer(&self.mesh_lod_buffer, 0, &new_buffer, 0, used_size);
            }

            queue.submit(std::iter::once(encoder.finish()));
            self.mesh_lod_buffer = new_buffer;
            self.mesh_lod_buffer_size = new_size;
            if let Some(world_state) = self.world_state_get_mut(self.current_world_id)
                && let Some(gpu) = world_state.gpu_buffers.as_mut()
            {
                gpu.culling_bind_group = None;
            }
        }

        let lod_data = MeshLodInfo {
            lod_count: 1,
            thresholds: [0.0, 0.0, 0.0],
        };
        queue.write_buffer(
            &self.mesh_lod_buffer,
            (mesh_id as usize * std::mem::size_of::<MeshLodInfo>()) as u64,
            bytemuck::bytes_of(&lod_data),
        );
        self.mesh_lod_data.push(lod_data);

        if let Some(morph_targets) = &mesh.morph_targets {
            let vertex_count = mesh.vertices.len();
            let target_count = morph_targets.targets.len().min(8);
            let total_displacements = vertex_count * target_count;

            if total_displacements > 0 {
                let mut displacements = Vec::with_capacity(total_displacements);

                for target_index in 0..target_count {
                    let target = &morph_targets.targets[target_index];
                    for vertex_index in 0..vertex_count {
                        let position = target
                            .position_displacements
                            .get(vertex_index)
                            .copied()
                            .unwrap_or([0.0, 0.0, 0.0]);
                        let normal = target
                            .normal_displacements
                            .as_ref()
                            .and_then(|n| n.get(vertex_index))
                            .copied()
                            .unwrap_or([0.0, 0.0, 0.0]);
                        let tangent = target
                            .tangent_displacements
                            .as_ref()
                            .and_then(|t| t.get(vertex_index))
                            .copied()
                            .unwrap_or([0.0, 0.0, 0.0]);

                        displacements.push(MorphDisplacement {
                            position,
                            _pad0: 0.0,
                            normal,
                            _pad1: 0.0,
                            tangent,
                            _pad2: 0.0,
                        });
                    }
                }

                let displacement_data = bytemuck::cast_slice(&displacements);
                let displacement_offset_bytes = self.current_morph_displacement_offset as u64
                    * std::mem::size_of::<MorphDisplacement>() as u64;
                let required_size = displacement_offset_bytes + displacement_data.len() as u64;

                if required_size > self.morph_displacement_buffer_size {
                    let new_size =
                        (required_size as f64 * BUFFER_GROWTH_FACTOR as f64).ceil() as u64;
                    let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Mesh Morph Displacement Buffer (Resized)"),
                        size: new_size,
                        usage: wgpu::BufferUsages::STORAGE
                            | wgpu::BufferUsages::COPY_DST
                            | wgpu::BufferUsages::COPY_SRC,
                        mapped_at_creation: false,
                    });

                    if displacement_offset_bytes > 0 {
                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: Some("Morph Displacement Buffer Reallocation"),
                            });
                        encoder.copy_buffer_to_buffer(
                            &self.morph_displacement_buffer,
                            0,
                            &new_buffer,
                            0,
                            displacement_offset_bytes,
                        );
                        queue.submit(std::iter::once(encoder.finish()));
                    }

                    self.morph_displacement_buffer = new_buffer;
                    self.morph_displacement_buffer_size = new_size;
                    self.invalidate_all_instance_bind_groups(device);
                }

                queue.write_buffer(
                    &self.morph_displacement_buffer,
                    displacement_offset_bytes,
                    displacement_data,
                );

                self.current_morph_displacement_offset += total_displacements as u32;
            }
        }

        self.current_vertex_offset += mesh.vertices.len() as u32;
        self.current_index_offset += mesh.indices.len() as u32;
    }

    pub(in super::super) fn resolve_lod_chains(
        &mut self,
        chains: &[crate::config::MeshLodChain],
        queue: &wgpu::Queue,
    ) {
        for entry in &mut self.mesh_lod_mesh_ids {
            entry.clear();
        }

        for info in &mut self.mesh_lod_data {
            info.lod_count = 1;
            info.thresholds = [0.0, 0.0, 0.0];
        }

        for chain in chains {
            let Some(&base_id) = self.meshes.get(&chain.base_mesh) else {
                continue;
            };

            let mut lod_ids = Vec::new();
            let mut thresholds = [0.0f32; 3];

            for (index, level) in chain.levels.iter().enumerate() {
                if let Some(&mesh_id) = self.meshes.get(&level.mesh_name) {
                    lod_ids.push(mesh_id);
                    if index < 3 {
                        thresholds[index] = level.min_screen_pixels;
                    }
                }
            }

            lod_ids.truncate(4);
            if lod_ids.len() > 1 {
                let lod_count = lod_ids.len() as u32;
                if (base_id as usize) < self.mesh_lod_data.len() {
                    self.mesh_lod_data[base_id as usize] = MeshLodInfo {
                        lod_count,
                        thresholds,
                    };
                }
                let slot = base_id as usize;
                if self.mesh_lod_mesh_ids.len() <= slot {
                    self.mesh_lod_mesh_ids.resize(slot + 1, Vec::new());
                }
                self.mesh_lod_mesh_ids[slot] = lod_ids;
            }
        }

        if !self.mesh_lod_data.is_empty() {
            queue.write_buffer(
                &self.mesh_lod_buffer,
                0,
                bytemuck::cast_slice(&self.mesh_lod_data),
            );
        }
    }

    pub(in super::super) fn populate_gpu_registry_from_instances(&mut self) {
        let state = self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap();
        state.gpu_registry.clear();
        state.free_slots_by_group.clear();

        for (slot, entity) in state.cached_entities.iter().enumerate() {
            let slot = slot as u32;
            state.gpu_registry.entity_to_slot.insert(*entity, slot);
            if state.gpu_registry.slot_to_entity.len() <= slot as usize {
                state
                    .gpu_registry
                    .slot_to_entity
                    .resize(slot as usize + 1, None);
            }
            state.gpu_registry.slot_to_entity[slot as usize] = Some(*entity);
            state.gpu_registry.slot_count = state.gpu_registry.slot_count.max(slot + 1);
        }
    }
}
