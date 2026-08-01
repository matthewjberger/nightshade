use nightshade_render::data::generational_registry::registry_entry_by_name;
use nightshade_render::data::geometry::SkinnedVertex;
use nightshade_render::queries::mesh_cache::mesh_cache_iter;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;

use super::super::types::*;
use super::SkinnedMeshPass;

impl SkinnedMeshPass {
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

    pub fn unregister_texture(&mut self, name: &str) {
        self.materials.registered_textures.remove(name);
    }

    pub fn add_material_layer_mapping(
        &mut self,
        texture: nightshade_render::data::asset_id::TextureId,
        layer: nightshade_wgpu::backend::MaterialTextureLayer,
    ) {
        self.materials.layer_map.insert(texture, layer);
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

    pub(in super::super) fn sync_skinned_meshes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh_cache: &nightshade_render::resources::mesh_cache::MeshCache,
    ) -> bool {
        let mut registry_changed = false;
        if mesh_cache.clear_generation != self.last_mesh_cache_clear_generation {
            registry_changed = true;
            self.last_mesh_cache_clear_generation = mesh_cache.clear_generation;
            self.buffers.ids.clear();
            self.buffers.data.clear();
            self.buffers.vertex_offset = 0;
            self.buffers.index_offset = 0;
            self.dirty_skinned_mesh_names.clear();
        }

        for dirty_name in std::mem::take(&mut self.dirty_skinned_mesh_names) {
            let mesh_in_cache =
                registry_entry_by_name(&mesh_cache.registry, &dirty_name).and_then(|mesh| {
                    mesh.skin_data
                        .as_ref()
                        .map(|skin| (skin.skinned_vertices.clone(), mesh.indices.clone()))
                });
            match (self.buffers.ids.get(&dirty_name).copied(), mesh_in_cache) {
                (Some(mesh_id), Some((skinned_vertices, indices))) => {
                    let mesh_data = &self.buffers.data[mesh_id as usize];
                    if skinned_vertices.len() as u32 == mesh_data.vertex_count
                        && indices.len() as u32 == mesh_data.index_count
                    {
                        let vertex_data = bytemuck::cast_slice(&skinned_vertices);
                        let vertex_offset_bytes = (mesh_data.vertex_offset as usize
                            * std::mem::size_of::<SkinnedVertex>())
                            as u64;
                        queue.write_buffer(&self.buffers.vertex, vertex_offset_bytes, vertex_data);
                        let index_offset_bytes =
                            (mesh_data.index_offset as usize * std::mem::size_of::<u32>()) as u64;
                        queue.write_buffer(
                            &self.buffers.index,
                            index_offset_bytes,
                            bytemuck::cast_slice(&indices),
                        );
                    } else {
                        registry_changed = true;
                        if let Some(slot) = self.buffers.data.get_mut(mesh_id as usize) {
                            slot.vertex_count = 0;
                            slot.index_count = 0;
                        }
                        self.buffers.ids.remove(&dirty_name);
                    }
                }
                (Some(mesh_id), None) => {
                    registry_changed = true;
                    if let Some(slot) = self.buffers.data.get_mut(mesh_id as usize) {
                        slot.vertex_count = 0;
                        slot.index_count = 0;
                    }
                    self.buffers.ids.remove(&dirty_name);
                }
                _ => {}
            }
        }

        for (name, mesh) in mesh_cache_iter(mesh_cache) {
            if mesh.skin_data.is_none() {
                continue;
            }

            if self.buffers.ids.contains_key(name.as_str()) {
                continue;
            }

            let skin_data = mesh.skin_data.as_ref().unwrap();
            let skinned_vertices = &skin_data.skinned_vertices;

            let vertex_size =
                (skinned_vertices.len() * std::mem::size_of::<SkinnedVertex>()) as u64;
            let index_size = (mesh.indices.len() * std::mem::size_of::<u32>()) as u64;

            let required_vertex_size = (self.buffers.vertex_offset as u64
                * std::mem::size_of::<SkinnedVertex>() as u64)
                + vertex_size;
            let required_index_size =
                (self.buffers.index_offset as u64 * std::mem::size_of::<u32>() as u64) + index_size;

            if required_vertex_size > self.buffers.vertex_size {
                let new_size = (required_vertex_size * 2).max(self.buffers.vertex_size * 2);
                let new_buffer = build::buffer(
                    device,
                    "Skinned Vertex Buffer (Resized)",
                    new_size,
                    wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                );

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Skinned Vertex Buffer Copy"),
                });
                encoder.copy_buffer_to_buffer(
                    &self.buffers.vertex,
                    0,
                    &new_buffer,
                    0,
                    self.buffers.vertex_size,
                );
                queue.submit(Some(encoder.finish()));

                self.buffers.vertex = new_buffer;
                self.buffers.vertex_size = new_size;
            }

            if required_index_size > self.buffers.index_size {
                let new_size = (required_index_size * 2).max(self.buffers.index_size * 2);
                let new_buffer = build::buffer(
                    device,
                    "Skinned Index Buffer (Resized)",
                    new_size,
                    wgpu::BufferUsages::INDEX
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                );

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Skinned Index Buffer Copy"),
                });
                encoder.copy_buffer_to_buffer(
                    &self.buffers.index,
                    0,
                    &new_buffer,
                    0,
                    self.buffers.index_size,
                );
                queue.submit(Some(encoder.finish()));

                self.buffers.index = new_buffer;
                self.buffers.index_size = new_size;
            }

            queue.write_buffer(
                &self.buffers.vertex,
                self.buffers.vertex_offset as u64 * std::mem::size_of::<SkinnedVertex>() as u64,
                bytemuck::cast_slice(skinned_vertices),
            );

            queue.write_buffer(
                &self.buffers.index,
                self.buffers.index_offset as u64 * std::mem::size_of::<u32>() as u64,
                bytemuck::cast_slice(&mesh.indices),
            );

            let (morph_displacement_offset, morph_target_count) = if let Some(morph_targets) =
                &mesh.morph_targets
            {
                let target_count = morph_targets.targets.len().min(8);
                let vertex_count = skinned_vertices.len();
                let total_displacements = target_count * vertex_count;

                let required_size = self.morph.offset as usize + total_displacements;
                if required_size > self.morph.buffer_size {
                    let new_size = (required_size as f32 * 2.0).ceil() as usize;
                    let new_buffer = build::buffer(
                        device,
                        "Skinned Mesh Morph Displacement Buffer (Resized)",
                        (std::mem::size_of::<MorphDisplacement>() * new_size) as u64,
                        wgpu::BufferUsages::STORAGE
                            | wgpu::BufferUsages::COPY_DST
                            | wgpu::BufferUsages::COPY_SRC,
                    );

                    let mut encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Skinned Mesh Morph Displacement Buffer Copy"),
                        });
                    encoder.copy_buffer_to_buffer(
                        &self.morph.buffer,
                        0,
                        &new_buffer,
                        0,
                        (std::mem::size_of::<MorphDisplacement>() * self.morph.buffer_size) as u64,
                    );
                    queue.submit(Some(encoder.finish()));

                    self.morph.buffer = new_buffer;
                    self.morph.buffer_size = new_size;
                    self.rebuild_instance_bind_group(device);
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

                if !displacements.is_empty() {
                    queue.write_buffer(
                        &self.morph.buffer,
                        (self.morph.offset as usize * std::mem::size_of::<MorphDisplacement>())
                            as u64,
                        bytemuck::cast_slice(&displacements),
                    );
                }

                let offset = self.morph.offset;
                self.morph.offset += total_displacements as u32;
                (offset, target_count as u32)
            } else {
                (0, 0)
            };

            let mesh_id = self.buffers.data.len() as u32;
            self.buffers.data.push(SkinnedMeshData {
                vertex_offset: self.buffers.vertex_offset,
                vertex_count: skinned_vertices.len() as u32,
                index_offset: self.buffers.index_offset,
                index_count: mesh.indices.len() as u32,
                morph_displacement_offset,
                morph_target_count,
            });

            self.buffers.ids.insert(name.clone(), mesh_id);
            registry_changed = true;

            self.buffers.vertex_offset += skinned_vertices.len() as u32;
            self.buffers.index_offset += mesh.indices.len() as u32;
        }
        registry_changed
    }
}
