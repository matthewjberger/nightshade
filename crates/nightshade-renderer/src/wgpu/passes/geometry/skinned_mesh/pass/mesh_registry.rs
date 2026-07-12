use crate::generational_registry::registry_entry_by_name;
use crate::geometry::SkinnedVertex;
use crate::mesh_cache::mesh_cache_iter;

use super::super::types::*;
use super::SkinnedMeshPass;

impl SkinnedMeshPass {
    pub fn register_texture(
        &mut self,
        name: String,
        view: wgpu::TextureView,
        sampler: wgpu::Sampler,
    ) {
        self.registered_textures.insert(name, (view, sampler));
    }

    pub fn unregister_texture(&mut self, name: &str) {
        self.registered_textures.remove(name);
    }

    pub fn add_material_layer_mapping(
        &mut self,
        texture: crate::asset_id::TextureId,
        layer: crate::wgpu::material_texture_arrays::MaterialTextureLayer,
    ) {
        self.material_layer_map.insert(texture, layer);
    }

    pub fn sync_textures(&mut self, texture_cache: &crate::wgpu::texture_cache::TextureCache) {
        if texture_cache.registry.name_to_index.is_empty() {
            return;
        }
        self.registered_textures
            .retain(|name, _| texture_cache.registry.name_to_index.contains_key(name));
    }

    pub(in super::super) fn sync_skinned_meshes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh_cache: &crate::mesh_cache::MeshCache,
    ) {
        if mesh_cache.clear_generation != self.last_mesh_cache_clear_generation {
            self.last_mesh_cache_clear_generation = mesh_cache.clear_generation;
            self.skinned_meshes.clear();
            self.skinned_mesh_data.clear();
            self.current_skinned_vertex_offset = 0;
            self.current_skinned_index_offset = 0;
            self.dirty_skinned_mesh_names.clear();
        }

        for dirty_name in std::mem::take(&mut self.dirty_skinned_mesh_names) {
            let mesh_in_cache = registry_entry_by_name(&mesh_cache.registry, &dirty_name)
                .and_then(|mesh| mesh.skin_data.as_ref().map(|s| s.skinned_vertices.clone()));
            match (self.skinned_meshes.get(&dirty_name).copied(), mesh_in_cache) {
                (Some(mesh_id), Some(skinned_vertices)) => {
                    let mesh_data = &self.skinned_mesh_data[mesh_id as usize];
                    if skinned_vertices.len() as u32 == mesh_data.vertex_count {
                        let vertex_data = bytemuck::cast_slice(&skinned_vertices);
                        let vertex_offset_bytes = (mesh_data.vertex_offset as usize
                            * std::mem::size_of::<SkinnedVertex>())
                            as u64;
                        queue.write_buffer(
                            &self.skinned_vertex_buffer,
                            vertex_offset_bytes,
                            vertex_data,
                        );
                    }
                }
                (Some(mesh_id), None) => {
                    if let Some(slot) = self.skinned_mesh_data.get_mut(mesh_id as usize) {
                        slot.vertex_count = 0;
                        slot.index_count = 0;
                    }
                    self.skinned_meshes.remove(&dirty_name);
                }
                _ => {}
            }
        }

        for (name, mesh) in mesh_cache_iter(mesh_cache) {
            if mesh.skin_data.is_none() {
                continue;
            }

            if self.skinned_meshes.contains_key(name.as_str()) {
                continue;
            }

            let skin_data = mesh.skin_data.as_ref().unwrap();
            let skinned_vertices = &skin_data.skinned_vertices;

            let vertex_size =
                (skinned_vertices.len() * std::mem::size_of::<SkinnedVertex>()) as u64;
            let index_size = (mesh.indices.len() * std::mem::size_of::<u32>()) as u64;

            let required_vertex_size = (self.current_skinned_vertex_offset as u64
                * std::mem::size_of::<SkinnedVertex>() as u64)
                + vertex_size;
            let required_index_size = (self.current_skinned_index_offset as u64
                * std::mem::size_of::<u32>() as u64)
                + index_size;

            if required_vertex_size > self.skinned_vertex_buffer_size {
                let new_size = (required_vertex_size * 2).max(self.skinned_vertex_buffer_size * 2);
                let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Skinned Vertex Buffer (Resized)"),
                    size: new_size,
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                });

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Skinned Vertex Buffer Copy"),
                });
                encoder.copy_buffer_to_buffer(
                    &self.skinned_vertex_buffer,
                    0,
                    &new_buffer,
                    0,
                    self.skinned_vertex_buffer_size,
                );
                queue.submit(Some(encoder.finish()));

                self.skinned_vertex_buffer = new_buffer;
                self.skinned_vertex_buffer_size = new_size;
            }

            if required_index_size > self.skinned_index_buffer_size {
                let new_size = (required_index_size * 2).max(self.skinned_index_buffer_size * 2);
                let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Skinned Index Buffer (Resized)"),
                    size: new_size,
                    usage: wgpu::BufferUsages::INDEX
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                });

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Skinned Index Buffer Copy"),
                });
                encoder.copy_buffer_to_buffer(
                    &self.skinned_index_buffer,
                    0,
                    &new_buffer,
                    0,
                    self.skinned_index_buffer_size,
                );
                queue.submit(Some(encoder.finish()));

                self.skinned_index_buffer = new_buffer;
                self.skinned_index_buffer_size = new_size;
            }

            queue.write_buffer(
                &self.skinned_vertex_buffer,
                self.current_skinned_vertex_offset as u64
                    * std::mem::size_of::<SkinnedVertex>() as u64,
                bytemuck::cast_slice(skinned_vertices),
            );

            queue.write_buffer(
                &self.skinned_index_buffer,
                self.current_skinned_index_offset as u64 * std::mem::size_of::<u32>() as u64,
                bytemuck::cast_slice(&mesh.indices),
            );

            let (morph_displacement_offset, morph_target_count) = if let Some(morph_targets) =
                &mesh.morph_targets
            {
                let target_count = morph_targets.targets.len().min(8);
                let vertex_count = skinned_vertices.len();
                let total_displacements = target_count * vertex_count;

                let required_size =
                    self.current_morph_displacement_offset as usize + total_displacements;
                if required_size > self.morph_displacement_buffer_size {
                    let new_size = (required_size as f32 * 2.0).ceil() as usize;
                    let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Skinned Mesh Morph Displacement Buffer (Resized)"),
                        size: (std::mem::size_of::<MorphDisplacement>() * new_size) as u64,
                        usage: wgpu::BufferUsages::STORAGE
                            | wgpu::BufferUsages::COPY_DST
                            | wgpu::BufferUsages::COPY_SRC,
                        mapped_at_creation: false,
                    });

                    let mut encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Skinned Mesh Morph Displacement Buffer Copy"),
                        });
                    encoder.copy_buffer_to_buffer(
                        &self.morph_displacement_buffer,
                        0,
                        &new_buffer,
                        0,
                        (std::mem::size_of::<MorphDisplacement>()
                            * self.morph_displacement_buffer_size) as u64,
                    );
                    queue.submit(Some(encoder.finish()));

                    self.morph_displacement_buffer = new_buffer;
                    self.morph_displacement_buffer_size = new_size;
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
                            _pad0: 0.0,
                            normal,
                            _pad1: 0.0,
                            tangent,
                            _pad2: 0.0,
                        });
                    }
                }

                if !displacements.is_empty() {
                    queue.write_buffer(
                        &self.morph_displacement_buffer,
                        (self.current_morph_displacement_offset as usize
                            * std::mem::size_of::<MorphDisplacement>())
                            as u64,
                        bytemuck::cast_slice(&displacements),
                    );
                }

                let offset = self.current_morph_displacement_offset;
                self.current_morph_displacement_offset += total_displacements as u32;
                (offset, target_count as u32)
            } else {
                (0, 0)
            };

            let mesh_id = self.skinned_mesh_data.len() as u32;
            self.skinned_mesh_data.push(SkinnedMeshData {
                vertex_offset: self.current_skinned_vertex_offset,
                vertex_count: skinned_vertices.len() as u32,
                index_offset: self.current_skinned_index_offset,
                index_count: mesh.indices.len() as u32,
                _skin_index: skin_data.skin_index.unwrap_or(0) as u32,
                morph_displacement_offset,
                morph_target_count,
            });

            self.skinned_meshes.insert(name.clone(), mesh_id);

            self.current_skinned_vertex_offset += skinned_vertices.len() as u32;
            self.current_skinned_index_offset += mesh.indices.len() as u32;
        }
    }
}
