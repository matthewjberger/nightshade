use nightshade_render::data::generational_registry::*;
use nightshade_render::data::geometry::{Mesh, Vertex};
use nightshade_render::queries::mesh_cache::mesh_cache_iter;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;

use super::super::types::{BUFFER_GROWTH_FACTOR, MeshBoundsData, MeshData, MorphDisplacement};

use super::MeshPass;

fn take_span(free: &mut Vec<(u32, u32)>, high: &mut u32, count: u32) -> u32 {
    if count == 0 {
        return *high;
    }
    if let Some(index) = free.iter().position(|(_, size)| *size >= count) {
        let (offset, size) = free.remove(index);
        if size > count {
            free.push((offset + count, size - count));
        }
        return offset;
    }
    let offset = *high;
    *high += count;
    offset
}

fn give_span(free: &mut Vec<(u32, u32)>, offset: u32, count: u32) {
    if count > 0 {
        free.push((offset, count));
    }
}

impl MeshPass {
    pub fn register_texture(
        &mut self,
        name: String,
        view: wgpu::TextureView,
        sampler: wgpu::Sampler,
    ) {
        self.materials
            .registered_textures
            .insert(name, (view, sampler));
    }

    pub fn mesh_id_for_name(&self, name: &str) -> Option<u32> {
        self.registry
            .ids
            .get(&nightshade_render::data::generational_registry::registry_name_hash(name))
            .copied()
    }

    pub fn take_new_mesh_names(&mut self, names: Vec<String>) {
        self.registry.new_names.extend(names);
    }
    pub fn register_texture_with_data(
        &mut self,
        name: String,
        view: wgpu::TextureView,
        sampler: wgpu::Sampler,
    ) {
        self.materials
            .registered_textures
            .insert(name, (view, sampler));
    }

    pub fn add_material_layer_mapping(
        &mut self,
        texture: nightshade_render::data::asset_id::TextureId,
        layer: nightshade_wgpu::backend::MaterialTextureLayer,
    ) {
        self.materials.layer_map.insert(texture, layer);
    }

    pub fn unregister_texture(&mut self, name: &str) {
        self.materials.registered_textures.remove(name);
    }

    pub fn sync_textures(
        &mut self,
        texture_cache: &nightshade_render::resources::texture_cache::TextureCache,
    ) {
        if texture_cache.registry.name_to_index.is_empty() {
            return;
        }
        self.materials
            .registered_textures
            .retain(|name, _| texture_cache.registry.name_to_index.contains_key(name));
    }

    pub fn sync_meshes_from_cache(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh_cache: &nightshade_render::resources::mesh_cache::MeshCache,
    ) {
        if mesh_cache.clear_generation != self.frame.last_cache_clear {
            self.frame.last_cache_clear = mesh_cache.clear_generation;
            self.reset_mesh_registry(device, queue);
            self.frame.dirty_mesh_ids.clear();
            if let Some(frame_dirty) = self.frame.dirty.as_mut() {
                frame_dirty.full_rebuild_needed = true;
            }
        }

        let mut evicted = false;
        let mut rewrites: Vec<(u32, String)> = Vec::new();
        for mesh_id in std::mem::take(&mut self.frame.dirty_mesh_ids) {
            let Some(name) = self.registry.names.get(mesh_id as usize) else {
                continue;
            };
            if let Some(mesh) = registry_entry_by_name(&mesh_cache.registry, name) {
                let mesh_data = &mut self.registry.data[mesh_id as usize];
                if mesh.vertices.len() as u32 != mesh_data.vertex_count
                    || mesh.indices.len() as u32 != mesh_data.index_count
                {
                    rewrites.push((mesh_id, name.clone()));
                    continue;
                }
                {
                    let vertex_data = bytemuck::cast_slice(&mesh.vertices);
                    let vertex_offset_bytes =
                        (mesh_data.vertex_offset as usize * std::mem::size_of::<Vertex>()) as u64;
                    queue.write_buffer(&self.buffers.vertex, vertex_offset_bytes, vertex_data);
                    let index_data = bytemuck::cast_slice(&mesh.indices);
                    let index_offset_bytes =
                        (mesh_data.index_offset as usize * std::mem::size_of::<u32>()) as u64;
                    queue.write_buffer(&self.buffers.index, index_offset_bytes, index_data);
                    self.registry.geo_dirty = true;

                    if let Some(morph_targets) = &mesh.morph_targets {
                        let vertex_count = mesh.vertices.len();
                        let target_count = morph_targets.targets.len().min(8);
                        let total_displacements = vertex_count * target_count;

                        if total_displacements > 0 {
                            if mesh_data.morph_target_count == 0 {
                                mesh_data.morph_displacement_offset = self.morph.offset;
                                mesh_data.morph_target_count = target_count as u32;
                                self.morph.offset += total_displacements as u32;
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
                                        padding_0: 0.0,
                                        normal,
                                        padding_1: 0.0,
                                        tangent,
                                        padding_2: 0.0,
                                    });
                                }
                            }

                            let displacement_data = bytemuck::cast_slice(&displacements);
                            let displacement_offset_bytes = mesh_data.morph_displacement_offset
                                as u64
                                * std::mem::size_of::<MorphDisplacement>() as u64;
                            let required_size =
                                displacement_offset_bytes + displacement_data.len() as u64;

                            if required_size > self.morph.buffer_size {
                                let new_size = (required_size as f64 * BUFFER_GROWTH_FACTOR as f64)
                                    .ceil() as u64;
                                let new_buffer = build::buffer(
                                    device,
                                    "Mesh Morph Displacement Buffer (Resized)",
                                    new_size,
                                    wgpu::BufferUsages::STORAGE
                                        | wgpu::BufferUsages::COPY_DST
                                        | wgpu::BufferUsages::COPY_SRC,
                                );

                                if displacement_offset_bytes > 0 {
                                    let mut encoder = device.create_command_encoder(
                                        &wgpu::CommandEncoderDescriptor {
                                            label: Some("Morph Displacement Buffer Reallocation"),
                                        },
                                    );
                                    encoder.copy_buffer_to_buffer(
                                        &self.morph.buffer,
                                        0,
                                        &new_buffer,
                                        0,
                                        displacement_offset_bytes,
                                    );
                                    queue.submit(std::iter::once(encoder.finish()));
                                }

                                self.morph.buffer = new_buffer;
                                self.morph.buffer_size = new_size;
                                self.invalidate_all_instance_bind_groups(device);
                            }

                            queue.write_buffer(
                                &self.morph.buffer,
                                displacement_offset_bytes,
                                displacement_data,
                            );
                        }
                    }

                    continue;
                }
            }
            let stale_name = self
                .registry
                .names
                .get(mesh_id as usize)
                .cloned()
                .unwrap_or_default();
            if !stale_name.is_empty()
                && self
                    .registry
                    .ids
                    .remove(
                        &nightshade_render::data::generational_registry::registry_name_hash(
                            &stale_name,
                        ),
                    )
                    .is_some()
            {
                if let Some(held) = self.registry.data.get(mesh_id as usize).copied() {
                    give_span(
                        &mut self.buffers.free_vertex,
                        held.vertex_offset,
                        held.vertex_count,
                    );
                    give_span(
                        &mut self.buffers.free_index,
                        held.index_offset,
                        held.index_count,
                    );
                }
                if let Some(slot) = self.registry.data.get_mut(mesh_id as usize) {
                    slot.vertex_count = 0;
                    slot.index_count = 0;
                    slot.morph_target_count = 0;
                }
                if let Some(name_slot) = self.registry.names.get_mut(mesh_id as usize) {
                    name_slot.clear();
                }
                self.registry.free_ids.push(mesh_id);
                self.registry.geo_dirty = true;
                evicted = true;
            }
        }

        if evicted
            && self.check_and_compact_buffers(device, queue, mesh_cache)
            && let Some(frame_dirty) = self.frame.dirty.as_mut()
        {
            frame_dirty.full_rebuild_needed = true;
        }

        for (mesh_id, name) in rewrites {
            let Some(mesh) = registry_entry_by_name(&mesh_cache.registry, &name) else {
                continue;
            };
            self.add_mesh(device, queue, &name, mesh, Some(mesh_id));
        }

        let mut meshes_to_add = Vec::new();
        if self.registry.rescan {
            self.registry.rescan = false;
            for (name, _) in mesh_cache_iter(mesh_cache) {
                if !self.registry.ids.contains_key(
                    &nightshade_render::data::generational_registry::registry_name_hash(name),
                ) {
                    meshes_to_add.push(name.clone());
                }
            }
        } else {
            for name in std::mem::take(&mut self.registry.new_names) {
                if self.registry.ids.contains_key(
                    &nightshade_render::data::generational_registry::registry_name_hash(&name),
                ) {
                    continue;
                }
                meshes_to_add.push(name);
            }
        }

        for name in meshes_to_add {
            let Some(mesh) = registry_entry_by_name(&mesh_cache.registry, &name) else {
                continue;
            };
            self.add_mesh(device, queue, &name, mesh, None);
        }
    }

    pub(in super::super) fn add_mesh(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        name: &str,
        mesh: &Mesh,
        slot: Option<u32>,
    ) {
        let max_buffer_size = device.limits().max_buffer_size;
        let mesh_id = match slot {
            Some(id) => id,
            None => match self.registry.free_ids.pop() {
                Some(id) => id,
                None => self.registry.data.len() as u32,
            },
        };
        let appending = slot.is_none() && mesh_id as usize == self.registry.data.len();
        let previous = slot.and_then(|id| self.registry.data.get(id as usize).copied());
        let vertex_count = mesh.vertices.len() as u32;
        let index_count = mesh.indices.len() as u32;
        let vertex_offset = match previous {
            Some(held) if held.vertex_count >= vertex_count => {
                give_span(
                    &mut self.buffers.free_vertex,
                    held.vertex_offset + vertex_count,
                    held.vertex_count - vertex_count,
                );
                held.vertex_offset
            }
            held => {
                if let Some(held) = held {
                    give_span(
                        &mut self.buffers.free_vertex,
                        held.vertex_offset,
                        held.vertex_count,
                    );
                }
                take_span(
                    &mut self.buffers.free_vertex,
                    &mut self.buffers.vertex_offset,
                    vertex_count,
                )
            }
        };
        let index_offset = match previous {
            Some(held) if held.index_count >= index_count => {
                give_span(
                    &mut self.buffers.free_index,
                    held.index_offset + index_count,
                    held.index_count - index_count,
                );
                held.index_offset
            }
            held => {
                if let Some(held) = held {
                    give_span(
                        &mut self.buffers.free_index,
                        held.index_offset,
                        held.index_count,
                    );
                }
                take_span(
                    &mut self.buffers.free_index,
                    &mut self.buffers.index_offset,
                    index_count,
                )
            }
        };

        let vertex_data = bytemuck::cast_slice(&mesh.vertices);
        let vertex_offset_bytes = (vertex_offset as usize * std::mem::size_of::<Vertex>()) as u64;
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

        if required_vertex_size > self.buffers.vertex_size {
            let new_size = (required_vertex_size as f32 * BUFFER_GROWTH_FACTOR).ceil() as u64;
            let new_size = new_size.min(max_buffer_size);

            let new_buffer = build::buffer(
                device,
                "Mesh Vertex Buffer (Resized)",
                new_size,
                wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            );

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Mesh Vertex Buffer Reallocation"),
            });

            let used_size = ((self.buffers.vertex_offset as usize * std::mem::size_of::<Vertex>())
                as u64)
                .min(self.buffers.vertex_size);
            if used_size > 0 {
                encoder.copy_buffer_to_buffer(&self.buffers.vertex, 0, &new_buffer, 0, used_size);
            }

            queue.submit(std::iter::once(encoder.finish()));
            self.buffers.vertex = new_buffer;
            self.buffers.vertex_size = new_size;
        }

        queue.write_buffer(&self.buffers.vertex, vertex_offset_bytes, vertex_data);

        let index_data = bytemuck::cast_slice(&mesh.indices);
        let index_offset_bytes = (index_offset as usize * std::mem::size_of::<u32>()) as u64;
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

        if required_index_size > self.buffers.index_size {
            let new_size = (required_index_size as f32 * BUFFER_GROWTH_FACTOR).ceil() as u64;
            let new_size = new_size.min(max_buffer_size);

            let new_buffer = build::buffer(
                device,
                "Mesh Index Buffer (Resized)",
                new_size,
                wgpu::BufferUsages::INDEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            );

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Mesh Index Buffer Reallocation"),
            });

            let used_size = ((self.buffers.index_offset as usize * std::mem::size_of::<u32>())
                as u64)
                .min(self.buffers.index_size);
            if used_size > 0 {
                encoder.copy_buffer_to_buffer(&self.buffers.index, 0, &new_buffer, 0, used_size);
            }

            queue.submit(std::iter::once(encoder.finish()));
            self.buffers.index = new_buffer;
            self.buffers.index_size = new_size;
        }

        queue.write_buffer(&self.buffers.index, index_offset_bytes, index_data);

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
            let half_x = (max_x - center_x).max(0.0);
            let half_y = (max_y - center_y).max(0.0);
            let half_z = (max_z - center_z).max(0.0);
            let radius = (half_x * half_x + half_y * half_y + half_z * half_z).sqrt();

            ([center_x, center_y, center_z], radius)
        };

        let (morph_displacement_offset, morph_target_count) =
            if let Some(morph_targets) = &mesh.morph_targets {
                let target_count = morph_targets.targets.len().min(8) as u32;
                let offset = self.morph.offset;
                (offset, target_count)
            } else {
                (0, 0)
            };

        let mesh_data = MeshData {
            vertex_offset,
            vertex_count,
            index_offset,
            index_count,
            morph_displacement_offset,
            morph_target_count,
        };

        match slot {
            Some(id) => {
                self.registry.data[id as usize] = mesh_data;
                if let Some(taken) = self.registry.names.get_mut(id as usize) {
                    taken.clear();
                    taken.push_str(name);
                }
                self.registry.ids.insert(
                    nightshade_render::data::generational_registry::registry_name_hash(name),
                    id,
                );
            }
            None if appending => {
                self.registry.ids.insert(
                    nightshade_render::data::generational_registry::registry_name_hash(name),
                    mesh_id,
                );
                self.registry.data.push(mesh_data);
                self.registry.names.push(name.to_string());
            }
            None => {
                self.registry.data[mesh_id as usize] = mesh_data;
                if let Some(taken) = self.registry.names.get_mut(mesh_id as usize) {
                    taken.clear();
                    taken.push_str(name);
                }
                self.registry.ids.insert(
                    nightshade_render::data::generational_registry::registry_name_hash(name),
                    mesh_id,
                );
            }
        }
        self.registry.geo_dirty = true;

        if mesh_id as usize >= self.registry.bounds_buffer_size {
            let new_size = ((mesh_id as usize + 1) as f32 * BUFFER_GROWTH_FACTOR).ceil() as usize;
            let new_buffer = build::buffer(
                device,
                "Mesh Bounds Buffer (Resized)",
                (std::mem::size_of::<MeshBoundsData>() * new_size) as u64,
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            );

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Mesh Bounds Buffer Reallocation"),
            });

            let used_size = (mesh_id as usize * std::mem::size_of::<MeshBoundsData>()) as u64;
            if used_size > 0 {
                encoder.copy_buffer_to_buffer(
                    &self.registry.bounds_buffer,
                    0,
                    &new_buffer,
                    0,
                    used_size,
                );
            }

            queue.submit(std::iter::once(encoder.finish()));
            self.registry.bounds_buffer = new_buffer;
            self.registry.bounds_buffer_size = new_size;
            if let Some(world_state) = self.state.as_mut()
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
            &self.registry.bounds_buffer,
            (mesh_id as usize * std::mem::size_of::<MeshBoundsData>()) as u64,
            bytemuck::bytes_of(&bounds_data),
        );
        self.registry.bounds_data.push(bounds_data);

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
                            padding_0: 0.0,
                            normal,
                            padding_1: 0.0,
                            tangent,
                            padding_2: 0.0,
                        });
                    }
                }

                let displacement_data = bytemuck::cast_slice(&displacements);
                let displacement_offset_bytes =
                    self.morph.offset as u64 * std::mem::size_of::<MorphDisplacement>() as u64;
                let required_size = displacement_offset_bytes + displacement_data.len() as u64;

                if required_size > self.morph.buffer_size {
                    let new_size =
                        (required_size as f64 * BUFFER_GROWTH_FACTOR as f64).ceil() as u64;
                    let new_buffer = build::buffer(
                        device,
                        "Mesh Morph Displacement Buffer (Resized)",
                        new_size,
                        wgpu::BufferUsages::STORAGE
                            | wgpu::BufferUsages::COPY_DST
                            | wgpu::BufferUsages::COPY_SRC,
                    );

                    if displacement_offset_bytes > 0 {
                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: Some("Morph Displacement Buffer Reallocation"),
                            });
                        encoder.copy_buffer_to_buffer(
                            &self.morph.buffer,
                            0,
                            &new_buffer,
                            0,
                            displacement_offset_bytes,
                        );
                        queue.submit(std::iter::once(encoder.finish()));
                    }

                    self.morph.buffer = new_buffer;
                    self.morph.buffer_size = new_size;
                    self.invalidate_all_instance_bind_groups(device);
                }

                queue.write_buffer(
                    &self.morph.buffer,
                    displacement_offset_bytes,
                    displacement_data,
                );

                self.morph.offset += total_displacements as u32;
            }
        }
    }

    pub(in super::super) fn populate_gpu_registry_from_instances(&mut self) {
        let state = self.state.as_mut().unwrap();
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
