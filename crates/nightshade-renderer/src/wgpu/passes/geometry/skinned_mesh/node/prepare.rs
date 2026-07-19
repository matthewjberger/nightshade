use crate::skinning::GpuSkinData;

use super::super::super::projection::*;
use super::super::pass::SkinnedMeshPass;
use super::super::types::*;
use super::super::world_state::SkinnedDrawGroup;
use crate::wgpu::passes::geometry::material_gpu::{
    convert_material_to_gpu_data, default_material_data,
};
use crate::wgpu::passes::grow_storage_buffer;
impl SkinnedMeshPass {
    pub(in super::super) fn prepare_pass_node(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &crate::wgpu::render_configs::RenderInputs,
    ) {
        self.set_current_world(configs.world_id);
        self.ensure_world_gpu_buffers(device);
        self.rebuild_bounds_compute_bind_group(device);
        self.frame_counter += 1;
        self.state_mut().last_used_frame = self.frame_counter;
        self.bounds_dispatch_count = 0;

        let mesh_registry_changed = self.sync_skinned_meshes(device, queue, &configs.mesh_cache);

        let mut buffers_resized = false;
        let max_buffer_size = device.limits().max_buffer_size as usize;
        let matrix_size = std::mem::size_of::<[[f32; 4]; 4]>();

        let skinning_generation = configs.scene.render_skinning.static_generation;
        let skinned_set_changed = skinning_generation != self.cached_skinning_generation;
        if skinned_set_changed {
            self.cached_skinning_generation = skinning_generation;
            self.skinning_cache = configs.scene.render_skinning.cache.clone();

            if !self.skinning_cache.inverse_bind_matrices.is_empty() {
                let required_size = self.skinning_cache.inverse_bind_matrices.len();
                if required_size > self.inverse_bind_matrices_buffer_size {
                    let new_size = (required_size as f32 * 2.0).ceil() as usize;
                    let buffer_bytes = matrix_size * new_size;
                    if buffer_bytes > max_buffer_size {
                        tracing::error!(
                            "Inverse bind matrices buffer would exceed GPU limit: {} > {} bytes",
                            buffer_bytes,
                            max_buffer_size
                        );
                    } else {
                        self.inverse_bind_matrices_buffer =
                            device.create_buffer(&wgpu::BufferDescriptor {
                                label: Some("Inverse Bind Matrices Buffer (Resized)"),
                                size: buffer_bytes as u64,
                                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                                mapped_at_creation: false,
                            });
                        self.inverse_bind_matrices_buffer_size = new_size;
                        buffers_resized = true;
                    }
                }
                if self.skinning_cache.inverse_bind_matrices.len()
                    <= self.inverse_bind_matrices_buffer_size
                {
                    queue.write_buffer(
                        &self.inverse_bind_matrices_buffer,
                        0,
                        bytemuck::cast_slice(&self.skinning_cache.inverse_bind_matrices),
                    );
                }
            }

            if !self.skinning_cache.skin_data.is_empty() {
                let required_size = self.skinning_cache.skin_data.len();
                if required_size > self.skin_data_buffer_size {
                    let new_size = (required_size as f32 * 2.0).ceil() as usize;
                    let buffer_bytes = std::mem::size_of::<GpuSkinData>() * new_size;
                    if buffer_bytes > max_buffer_size {
                        tracing::error!(
                            "Skin data buffer would exceed GPU limit: {} > {} bytes",
                            buffer_bytes,
                            max_buffer_size
                        );
                    } else {
                        self.skin_data_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                            label: Some("Skin Data Buffer (Resized)"),
                            size: buffer_bytes as u64,
                            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                            mapped_at_creation: false,
                        });
                        self.skin_data_buffer_size = new_size;
                        buffers_resized = true;
                    }
                }
                if self.skinning_cache.skin_data.len() <= self.skin_data_buffer_size {
                    queue.write_buffer(
                        &self.skin_data_buffer,
                        0,
                        bytemuck::cast_slice(&self.skinning_cache.skin_data),
                    );
                }
            }
        }

        let bone_transforms = &configs.scene.render_skinning.bone_transforms;
        let bones_changed = self.cached_palette_generation
            != configs.scene.render_skinning.bone_transforms_generation;
        if !bone_transforms.is_empty() {
            let required_size = bone_transforms.len();
            if required_size > self.bone_transforms_buffer_size {
                let new_size = (required_size as f32 * 2.0).ceil() as usize;
                let buffer_bytes = matrix_size * new_size;
                if buffer_bytes > max_buffer_size {
                    tracing::error!(
                        "Bone transforms buffer would exceed GPU limit: {} > {} bytes",
                        buffer_bytes,
                        max_buffer_size
                    );
                } else {
                    self.bone_transforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Bone Transforms Buffer (Resized)"),
                        size: buffer_bytes as u64,
                        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
                    self.bone_transforms_buffer_size = new_size;
                    self.bone_transforms_generation += 1;
                    buffers_resized = true;
                }
            }
            // Skip the upload when nothing moved, unless a buffer was just
            // reallocated or a skinned mesh was added or removed this frame, in
            // which case the bones must be refilled even if they did not move.
            if (bones_changed || buffers_resized || skinned_set_changed)
                && bone_transforms.len() <= self.bone_transforms_buffer_size
            {
                queue.write_buffer(
                    &self.bone_transforms_buffer,
                    0,
                    bytemuck::cast_slice(bone_transforms),
                );
            }
        }

        let required_joints = self.skinning_cache.total_joints as usize;
        if required_joints > self.joint_matrices_buffer_size {
            let new_size = (required_joints as f32 * 2.0).ceil() as usize;
            let buffer_bytes = matrix_size * new_size;
            if buffer_bytes > max_buffer_size {
                tracing::error!(
                    "Joint matrices buffer would exceed GPU limit: {} > {} bytes",
                    buffer_bytes,
                    max_buffer_size
                );
            } else {
                self.joint_matrices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Joint Matrices Buffer (Resized)"),
                    size: buffer_bytes as u64,
                    usage: wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                });
                self.prev_joint_matrices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Previous Joint Matrices Buffer (Resized)"),
                    size: buffer_bytes as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                self.joint_matrices_buffer_size = new_size;
                buffers_resized = true;
                self.rebuild_instance_bind_group(device);
            }
        }

        if buffers_resized {
            self.rebuild_skinning_compute_bind_group(device);
            self.rebuild_bounds_compute_bind_group(device);
        }

        self.animation.update(
            device,
            queue,
            configs,
            &self.skinning_cache,
            &self.bone_transforms_buffer,
            self.bone_transforms_generation,
        );

        // Re-run the skinning compute when the bones moved, a buffer was
        // reallocated, or the GPU animation pass wrote fresh bone world
        // matrices this frame; otherwise last frame's joint matrices are
        // still valid and the dispatch is skipped.
        let animation_active = self.animation.skeleton_count() > 0;
        self.total_joints_to_dispatch =
            if bones_changed || buffers_resized || animation_active || skinned_set_changed {
                self.skinning_cache.total_joints
            } else {
                0
            };
        self.cached_palette_generation = configs.scene.render_skinning.bone_transforms_generation;

        let statics_dirty = skinned_set_changed
            || buffers_resized
            || mesh_registry_changed
            || self.gpu_batching_enabled != configs.scene.gpu_batching_enabled
            || self.state().cached_materials_generation
                != configs.scene.render_materials.generation
            || self.state().cached_skinned_objects_generation
                != configs.scene.skinned_objects_generation;

        if !statics_dirty {
            if self.state().cached_cull_objects.is_empty() {
                return;
            }
            if !self.gpu_batching_enabled {
                let commands = std::mem::take(&mut self.state_mut().cached_indirect_commands);
                if !commands.is_empty() {
                    queue.write_buffer(
                        &self.gpu().indirect_commands_buffer,
                        0,
                        bytemuck::cast_slice(&commands),
                    );
                }
                self.state_mut().cached_indirect_commands = commands;
            }
            if bones_changed {
                self.bounds_dispatch_count = self.state().cached_bounds_inputs.len() as u32;
            }
        } else {
            self.state_mut().skinned_entities.clear();
            self.state_mut().opaque_draw_groups.clear();
            self.state_mut().transparent_draw_groups.clear();
            let mut objects = Vec::new();
            let mut materials = Vec::new();
            let mut custom_data = Vec::new();
            let mut entity_is_transparent: Vec<bool> = Vec::new();
            let mut object_bounds: Vec<[f32; 4]> = Vec::new();
            let mut bounds_inputs: Vec<(usize, usize, f32)> = Vec::new();

            for &entity in &configs.scene.render_skinned_meshes {
                if configs
                    .scene
                    .render_dynamic_objects
                    .get(&entity)
                    .is_some_and(|dynamic| dynamic.visible == 0)
                {
                    continue;
                }

                let mesh_name = match configs.scene.render_object_meshes.get(&entity) {
                    Some(name) => name,
                    None => continue,
                };

                let mesh_id = match self.skinned_meshes.get(mesh_name) {
                    Some(&id) => id,
                    None => continue,
                };

                let skin_index = self
                    .skinning_cache
                    .entity_skin_indices
                    .get(&entity)
                    .copied()
                    .unwrap_or(0);
                let joint_offset = self.skinning_cache.get_joint_offset(skin_index);

                let material_id = materials.len() as u32;
                let material_data = {
                    let render_materials = &configs.scene.render_materials;
                    let global_id = render_materials
                        .entity_to_id
                        .get(&entity)
                        .copied()
                        .unwrap_or(0);
                    match global_id
                        .checked_sub(1)
                        .and_then(|index| render_materials.entries.get(index as usize))
                    {
                        Some(entry) => convert_material_to_gpu_data(
                            &entry.material,
                            &entry.texture_ids,
                            &self.material_layer_map,
                        ),
                        None => default_material_data([1.0, 1.0, 1.0, 1.0]),
                    }
                };

                let is_transparent = material_data.alpha_mode == 2;
                entity_is_transparent.push(is_transparent);
                materials.push(material_data);
                custom_data.push([1.0f32, 1.0, 1.0, 1.0]);

                let mesh_data = &self.skinned_mesh_data[mesh_id as usize];
                let dynamic = configs.scene.render_dynamic_objects.get(&entity).copied();
                let morph_weights = dynamic.map(|d| d.morph_weights).unwrap_or([0.0; 8]);

                let flip_winding = dynamic
                    .map(|d| {
                        u32::from(
                            nalgebra_glm::determinant(&nalgebra_glm::mat4_to_mat3(&d.transform))
                                < 0.0,
                        )
                    })
                    .unwrap_or(0);

                objects.push(SkinnedObjectData {
                    transform_index: 0,
                    mesh_id,
                    material_id,
                    joint_offset,
                    morph_weights,
                    morph_target_count: mesh_data.morph_target_count,
                    morph_displacement_offset: mesh_data.morph_displacement_offset,
                    mesh_vertex_offset: mesh_data.vertex_offset,
                    mesh_vertex_count: mesh_data.vertex_count,
                    flip_winding,
                    entity_id: entity.id,
                    is_transparent: u32::from(is_transparent),
                    _padding: 0,
                });

                let mesh_rest_radius = configs
                    .scene_world
                    .get::<crate::config::RenderBounds>(entity)
                    .map(|bv| bv.sphere_radius)
                    .unwrap_or(2.0);
                let base_bone = self.skinning_cache.get_base_bone_index(skin_index) as usize;
                let joint_count = self
                    .skinning_cache
                    .skin_data
                    .get(skin_index as usize)
                    .map(|skin| skin.joint_count as usize)
                    .unwrap_or(0);
                object_bounds.push(crate::skinning::skinned_world_bounds(
                    &configs.scene.render_skinning.bone_transforms,
                    base_bone,
                    joint_count,
                    mesh_rest_radius,
                ));
                bounds_inputs.push((base_bone, joint_count, mesh_rest_radius));

                self.state_mut().skinned_entities.push(entity);
            }

            if objects.is_empty() {
                self.gpu_batching_enabled = configs.scene.gpu_batching_enabled;
                let materials_generation = configs.scene.render_materials.generation;
                let skinned_objects_generation = configs.scene.skinned_objects_generation;
                let state = self.state_mut();
                state.cached_cull_objects = Vec::new();
                state.cached_bounds_inputs = Vec::new();
                state.cached_indirect_commands = Vec::new();
                state.cached_materials_generation = materials_generation;
                state.cached_skinned_objects_generation = skinned_objects_generation;
                return;
            }

            self.gpu_batching_enabled = configs.scene.gpu_batching_enabled;
            let mesh_count = self.skinned_mesh_data.len().max(1);

            let (objects, custom_data, cull_objects, commands, bounds_inputs) = if self
                .gpu_batching_enabled
            {
                // GPU-driven path: no CPU sort or command build. Objects stay in ECS
                // order; the GPU classify/count/build emits the per-(class,mesh)
                // commands and stamps each cull object's command index. The draw
                // issues a fixed `mesh_count` commands per class (empty slots draw
                // nothing), so the per-class draw-group lengths are set to that.
                let has_transparent = entity_is_transparent.iter().any(|&value| value);
                let placeholder = SkinnedDrawGroup {
                    mesh_id: 0,
                    first_instance: 0,
                    instance_count: 0,
                };
                // The opaque region is [0, mesh_count) and the transparent region is
                // [mesh_count, 2 * mesh_count). The transparent draw offset is derived
                // from the opaque group length, so opaque is always `mesh_count`
                // (empty command slots draw nothing); transparent is set only when
                // there are transparent objects so the OIT path is skipped otherwise.
                self.state_mut().opaque_draw_groups = vec![placeholder; mesh_count];
                if has_transparent {
                    self.state_mut().transparent_draw_groups = vec![placeholder; mesh_count];
                }
                let cull_objects: Vec<SkinnedCullObject> = object_bounds
                    .iter()
                    .map(|&bounds| SkinnedCullObject {
                        bounds,
                        command_index: 0,
                        _pad: [0; 3],
                    })
                    .collect();
                let commands = vec![
                    SkinnedDrawIndexedIndirect {
                        index_count: 0,
                        instance_count: 0,
                        first_index: 0,
                        base_vertex: 0,
                        first_instance: 0,
                    };
                    2 * mesh_count
                ];
                (objects, custom_data, cull_objects, commands, bounds_inputs)
            } else {
                // CPU path: order instances by (opaque-before-transparent, mesh_id)
                // so every same-mesh run is contiguous and drawn with one instanced
                // draw call. `objects` and `custom_data` are indexed by the GPU
                // instance index, so they are permuted together; `materials` is
                // addressed by `object.material_id` (absolute) and stays put.
                let mut order: Vec<usize> = (0..objects.len()).collect();
                order.sort_by_key(|&index| (entity_is_transparent[index], objects[index].mesh_id));

                let objects: Vec<SkinnedObjectData> =
                    order.iter().map(|&index| objects[index]).collect();
                let custom_data: Vec<[f32; 4]> =
                    order.iter().map(|&index| custom_data[index]).collect();
                let entity_is_transparent: Vec<bool> = order
                    .iter()
                    .map(|&index| entity_is_transparent[index])
                    .collect();
                let object_bounds: Vec<[f32; 4]> =
                    order.iter().map(|&index| object_bounds[index]).collect();
                let bounds_inputs: Vec<(usize, usize, f32)> =
                    order.iter().map(|&index| bounds_inputs[index]).collect();
                {
                    let previous_entities: Vec<_> = self.state().skinned_entities.clone();
                    let reordered: Vec<_> = order
                        .iter()
                        .map(|&index| previous_entities[index])
                        .collect();
                    self.state_mut().skinned_entities = reordered;
                }

                let mut index = 0;
                while index < objects.len() {
                    let is_transparent = entity_is_transparent[index];
                    let mesh_id = objects[index].mesh_id;
                    let first_instance = index as u32;
                    while index < objects.len()
                        && entity_is_transparent[index] == is_transparent
                        && objects[index].mesh_id == mesh_id
                    {
                        index += 1;
                    }
                    let group = SkinnedDrawGroup {
                        mesh_id,
                        first_instance,
                        instance_count: index as u32 - first_instance,
                    };
                    if is_transparent {
                        self.state_mut().transparent_draw_groups.push(group);
                    } else {
                        self.state_mut().opaque_draw_groups.push(group);
                    }
                }

                let mut commands: Vec<SkinnedDrawIndexedIndirect> = Vec::new();
                let mut command_index_per_object = vec![0u32; objects.len()];
                {
                    let ordered_groups: Vec<(u32, u32, u32)> = self
                        .state()
                        .opaque_draw_groups
                        .iter()
                        .chain(self.state().transparent_draw_groups.iter())
                        .map(|group| (group.mesh_id, group.first_instance, group.instance_count))
                        .collect();
                    for (mesh_id, first_instance, instance_count) in ordered_groups {
                        let command_index = commands.len() as u32;
                        let mesh_data = &self.skinned_mesh_data[mesh_id as usize];
                        commands.push(SkinnedDrawIndexedIndirect {
                            index_count: mesh_data.index_count,
                            instance_count: 0,
                            first_index: mesh_data.index_offset,
                            base_vertex: mesh_data.vertex_offset as i32,
                            first_instance,
                        });
                        for object_index in first_instance..first_instance + instance_count {
                            command_index_per_object[object_index as usize] = command_index;
                        }
                    }
                }
                let cull_objects: Vec<SkinnedCullObject> = (0..objects.len())
                    .map(|index| SkinnedCullObject {
                        bounds: object_bounds[index],
                        command_index: command_index_per_object[index],
                        _pad: [0; 3],
                    })
                    .collect();
                (objects, custom_data, cull_objects, commands, bounds_inputs)
            };

            {
                let mut needs_bind_group_rebuild = false;
                {
                    let gpu = self.world_states[self.current_world_id as usize]
                        .as_mut()
                        .unwrap()
                        .gpu_buffers
                        .as_mut()
                        .unwrap();
                    if grow_storage_buffer::<SkinnedCullObject>(
                        device,
                        &mut gpu.cull_objects_buffer,
                        &mut gpu.cull_objects_buffer_size,
                        cull_objects.len(),
                        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                        "Skinned Mesh Cull Objects Buffer (Resized)",
                    ) {
                        needs_bind_group_rebuild = true;
                    }
                    if grow_storage_buffer::<SkinnedBoundsInput>(
                        device,
                        &mut gpu.bounds_inputs_buffer,
                        &mut gpu.bounds_inputs_buffer_size,
                        cull_objects.len(),
                        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                        "Skinned Mesh Bounds Inputs Buffer (Resized)",
                    ) {
                        needs_bind_group_rebuild = true;
                    }
                    if grow_storage_buffer::<SkinnedDrawIndexedIndirect>(
                        device,
                        &mut gpu.indirect_commands_buffer,
                        &mut gpu.indirect_commands_buffer_size,
                        commands.len(),
                        wgpu::BufferUsages::STORAGE
                            | wgpu::BufferUsages::INDIRECT
                            | wgpu::BufferUsages::COPY_DST,
                        "Skinned Mesh Indirect Commands Buffer (Resized)",
                    ) {
                        needs_bind_group_rebuild = true;
                    }
                    if grow_storage_buffer::<u32>(
                        device,
                        &mut gpu.visible_indices_buffer,
                        &mut gpu.visible_indices_buffer_size,
                        objects.len(),
                        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                        "Skinned Mesh Visible Indices Buffer (Resized)",
                    ) {
                        needs_bind_group_rebuild = true;
                    }
                }
                if needs_bind_group_rebuild {
                    self.rebuild_instance_bind_group(device);
                    self.rebuild_cull_bind_group(device);
                    self.rebuild_bounds_compute_bind_group(device);
                    self.gpu_mut().batch_bind_group = None;
                }
            }

            queue.write_buffer(
                &self.gpu().cull_objects_buffer,
                0,
                bytemuck::cast_slice(&cull_objects),
            );
            let bounds_inputs_packed: Vec<SkinnedBoundsInput> = bounds_inputs
                .iter()
                .map(
                    |&(base_bone, joint_count, rest_radius)| SkinnedBoundsInput {
                        base_bone: base_bone as u32,
                        joint_count: joint_count as u32,
                        rest_radius,
                        pad: 0,
                    },
                )
                .collect();
            queue.write_buffer(
                &self.gpu().bounds_inputs_buffer,
                0,
                bytemuck::cast_slice(&bounds_inputs_packed),
            );
            queue.write_buffer(
                &self.gpu().indirect_commands_buffer,
                0,
                bytemuck::cast_slice(&commands),
            );
            self.state_mut().cached_indirect_commands = commands;

            {
                {
                    let gpu = self.world_states[self.current_world_id as usize]
                        .as_mut()
                        .unwrap()
                        .gpu_buffers
                        .as_mut()
                        .unwrap();
                    if grow_storage_buffer::<SkinnedObjectData>(
                        device,
                        &mut gpu.object_buffer,
                        &mut gpu.object_buffer_size,
                        objects.len(),
                        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                        "Skinned Mesh Object Buffer (Per-World, Resized)",
                    ) {
                        gpu.batch_bind_group = None;
                    }
                }
                self.rebuild_instance_bind_group(device);
            }

            {
                let gpu = self.world_states[self.current_world_id as usize]
                    .as_mut()
                    .unwrap()
                    .gpu_buffers
                    .as_mut()
                    .unwrap();
                grow_storage_buffer::<MaterialData>(
                    device,
                    &mut gpu.materials_buffer,
                    &mut gpu.materials_buffer_size,
                    materials.len(),
                    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    "Skinned Mesh Materials Buffer (Per-World, Resized)",
                );
            }

            {
                {
                    let gpu = self.world_states[self.current_world_id as usize]
                        .as_mut()
                        .unwrap()
                        .gpu_buffers
                        .as_mut()
                        .unwrap();
                    grow_storage_buffer::<[f32; 4]>(
                        device,
                        &mut gpu.custom_data_buffer,
                        &mut gpu.custom_data_buffer_size,
                        custom_data.len(),
                        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                        "Skinned Mesh Custom Data Buffer (Per-World, Resized)",
                    );
                }
                self.rebuild_instance_bind_group(device);
            }

            queue.write_buffer(&self.gpu().object_buffer, 0, bytemuck::cast_slice(&objects));
            queue.write_buffer(
                &self.gpu().materials_buffer,
                0,
                bytemuck::cast_slice(&materials),
            );
            queue.write_buffer(
                &self.gpu().custom_data_buffer,
                0,
                bytemuck::cast_slice(&custom_data),
            );

            let materials_generation = configs.scene.render_materials.generation;
            let skinned_objects_generation = configs.scene.skinned_objects_generation;
            let state = self.state_mut();
            state.cached_cull_objects = cull_objects;
            state.cached_bounds_inputs = bounds_inputs;
            state.cached_materials_generation = materials_generation;
            state.cached_skinned_objects_generation = skinned_objects_generation;

            if self.gpu_batching_enabled {
                let object_count = objects.len() as u32;
                let mesh_geo: Vec<[u32; 4]> = self
                    .skinned_mesh_data
                    .iter()
                    .map(|mesh| [mesh.index_count, mesh.index_offset, mesh.vertex_offset, 0])
                    .collect();
                if mesh_geo.len() > self.batch_mesh_geo_buffer_size {
                    let new_size = mesh_geo.len().next_power_of_two();
                    self.batch_mesh_geo_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Skinned Mesh Batch Mesh Geo Buffer (Resized)"),
                        size: (std::mem::size_of::<[u32; 4]>() * new_size) as u64,
                        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
                    self.batch_mesh_geo_buffer_size = new_size;
                    self.gpu_mut().batch_bind_group = None;
                }
                if !mesh_geo.is_empty() {
                    queue.write_buffer(
                        &self.batch_mesh_geo_buffer,
                        0,
                        bytemuck::cast_slice(&mesh_geo),
                    );
                }

                let needed = 2 * mesh_count;
                if needed > self.gpu().dense_capacity_size {
                    let new_size = needed.next_power_of_two();
                    let gpu = self.gpu_mut();
                    gpu.dense_capacity_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Skinned Mesh Dense Capacity Buffer (Resized)"),
                        size: (std::mem::size_of::<u32>() * new_size) as u64,
                        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
                    gpu.dense_capacity_size = new_size;
                    gpu.command_map_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Skinned Mesh Command Map Buffer (Resized)"),
                        size: (std::mem::size_of::<u32>() * new_size) as u64,
                        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
                    gpu.command_map_size = new_size;
                    gpu.batch_bind_group = None;
                }

                let params = [object_count, mesh_count as u32, mesh_count as u32, 0u32];
                queue.write_buffer(&self.batch_params_buffer, 0, bytemuck::cast_slice(&params));
            }
        }

        self.sync_textures(&configs.texture_cache);

        let Some(lighting) = configs.scene.render_lighting.as_ref() else {
            return;
        };
        let mut lights_data = lighting.lights_data.clone();
        let entity_to_lights_index = lighting.entity_to_lights_index.clone();
        let stashed_num_directional_lights = lighting.num_directional_lights;
        let mut stashed_area_lights_data = lighting.area_lights_data.clone();
        let stashed_area_entity_to_index = lighting.area_entity_to_index.clone();
        let directional_light_direction = lighting.directional_light_direction;
        let stashed_cascade_view_projections = lighting.cascade_view_projections;
        let stashed_cascade_diameters = lighting.cascade_diameters;
        let stashed_cascade_split_distances = lighting.cascade_split_distances;
        let stashed_light_view_projection = lighting.light_view_projection;
        let stashed_shadow_bias = lighting.shadow_bias;
        let stashed_shadow_normal_bias = lighting.shadow_normal_bias;
        let stashed_shadows_enabled = lighting.shadows_enabled;

        crate::wgpu::passes::geometry::projection::resolve_cookie_layers(
            configs,
            &mut lights_data,
            &entity_to_lights_index,
            &self.material_layer_map,
            &configs.texture_cache.registry,
        );

        let spotlight_result = collect_spotlight_shadows(configs);
        apply_spotlight_shadow_indices(
            &mut lights_data,
            &spotlight_result.entity_to_shadow_index,
            &entity_to_lights_index,
        );

        if !spotlight_result.shadow_data.is_empty() {
            queue.write_buffer(
                &self.spotlight_shadow_buffer,
                0,
                bytemuck::cast_slice(&spotlight_result.shadow_data),
            );
        }

        {
            {
                let gpu = self.world_states[self.current_world_id as usize]
                    .as_mut()
                    .unwrap()
                    .gpu_buffers
                    .as_mut()
                    .unwrap();
                if grow_storage_buffer::<LightData>(
                    device,
                    &mut gpu.lights_buffer,
                    &mut gpu.lights_buffer_size,
                    lights_data.len(),
                    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    "Skinned Mesh Lights Buffer (Per-World, Resized)",
                ) {
                    gpu.cluster_assign_bind_group = None;
                }
            }
            self.rebuild_instance_bind_group(device);
        }

        if !lights_data.is_empty() {
            queue.write_buffer(
                &self.gpu().lights_buffer,
                0,
                bytemuck::cast_slice(&lights_data),
            );
        }

        apply_area_light_shadow_indices(
            &mut stashed_area_lights_data,
            &spotlight_result.entity_to_shadow_index,
            &stashed_area_entity_to_index,
        );
        resolve_area_emissive_layers(
            configs,
            &mut stashed_area_lights_data,
            &stashed_area_entity_to_index,
            &self.material_layer_map,
            &configs.texture_cache.registry,
        );
        if !stashed_area_lights_data.is_empty() {
            queue.write_buffer(
                &self.gpu().area_lights_buffer,
                0,
                bytemuck::cast_slice(&stashed_area_lights_data),
            );
        }
        {
            let area_uniforms = AreaUniforms {
                count: stashed_area_lights_data.len() as u32,
                _pad0: 0,
                _pad1: 0,
                _pad2: 0,
            };
            queue.write_buffer(
                &self.gpu().area_uniforms_buffer,
                0,
                bytemuck::cast_slice(&[area_uniforms]),
            );
        }

        let cascade_view_projections = stashed_cascade_view_projections;
        let cascade_diameters = stashed_cascade_diameters;
        let cascade_split_distances = stashed_cascade_split_distances;
        let light_view_projection = stashed_light_view_projection;
        let shadow_bias = stashed_shadow_bias;
        let shadow_normal_bias = stashed_shadow_normal_bias;
        let shadows_enabled = stashed_shadows_enabled;

        let cascade_texture_resolution = crate::wgpu::passes::CASCADE_SLOT_RESOLUTION;

        let cascade_atlas_offsets: [[f32; 4]; crate::wgpu::passes::NUM_SHADOW_CASCADES] = [
            [
                0.0,
                0.0,
                cascade_diameters[0] / cascade_texture_resolution,
                0.0,
            ],
            [
                0.5,
                0.0,
                cascade_diameters[1] / cascade_texture_resolution,
                0.0,
            ],
            [
                0.0,
                0.5,
                cascade_diameters[2] / cascade_texture_resolution,
                0.0,
            ],
            [
                0.5,
                0.5,
                cascade_diameters[3] / cascade_texture_resolution,
                0.0,
            ],
        ];

        if let Some(camera_matrices) = configs.scene.render_view.as_ref() {
            let global_unlit = if configs.scene.active_view.unlit_mode {
                1.0
            } else {
                0.0
            };

            let (snap_resolution, snap_enabled) =
                if let Some(ref vertex_snap) = configs.settings.vertex_snap {
                    (vertex_snap.resolution, 1)
                } else {
                    ([320.0, 240.0], 0)
                };

            let affine_enabled = if configs.settings.affine_texture_mapping {
                1
            } else {
                0
            };

            let (fog_color, fog_enabled, fog_start, fog_end) =
                if let Some(ref fog) = configs.scene.active_view.fog {
                    (fog.color, fog.mode.as_u32(), fog.start, fog.end)
                } else {
                    ([0.5, 0.5, 0.6], 0, 5.0, 30.0)
                };

            let time = configs.view.uptime_milliseconds as f32 / 1000.0;

            let camera_z_far = configs
                .scene
                .render_view
                .as_ref()
                .map(|view| view.z_far)
                .unwrap_or(1000.0);
            let oit_z_scale = (camera_z_far * 0.2).max(1.0);

            let uniforms = SkinnedMeshUniforms {
                view: camera_matrices.view.into(),
                projection: camera_matrices.projection.into(),
                camera_position: [
                    camera_matrices.camera_position.x,
                    camera_matrices.camera_position.y,
                    camera_matrices.camera_position.z,
                    1.0,
                ],
                num_lights: [lights_data.len() as u32, 0, 0, 0],
                ambient_light: configs.scene.active_view.ambient_light,
                light_view_projection,
                shadow_bias,
                shadows_enabled,
                global_unlit,
                shadow_normal_bias,
                snap_resolution,
                snap_enabled,
                affine_enabled,
                fog_color,
                fog_enabled,
                fog_start,
                fog_end,
                cascade_count: crate::wgpu::passes::NUM_SHADOW_CASCADES as u32,
                _padding2: 0.0,
                cascade_view_projections,
                cascade_split_distances,
                cascade_atlas_offsets,
                cascade_atlas_scale: [0.5, 0.5, 0.0, 0.0],
                time,
                pbr_debug_mode: configs.debug_draw.pbr_debug_mode.as_u32(),
                texture_debug_stripes: configs.debug_draw.texture_debug_stripes as u32,
                texture_debug_stripes_speed: configs.debug_draw.texture_debug_stripes_speed,
                directional_light_direction,
                ibl_blend_factor: configs.settings.ibl_blend_factor,
                oit_z_scale,
                _pad_pre_flat: [0.0; 2],
                flat_color: configs
                    .scene
                    .active_view
                    .flat_shading_color
                    .map(|c| [c.x, c.y, c.z, c.w])
                    .unwrap_or([0.0; 4]),
                prev_view_projection: configs.scene.prev_view_projection,
                cur_view_projection: configs.scene.view_projection,
                _padding3: [0.0; 44],
            };

            queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

            let view_projection = camera_matrices.projection * camera_matrices.view;
            let frustum_planes = extract_frustum_planes(&view_projection);
            let cull_uniforms = SkinnedCullUniforms {
                frustum_planes: frustum_planes.map(|plane| [plane.x, plane.y, plane.z, plane.w]),
                object_count: self.state().cached_cull_objects.len() as u32,
                _pad: [0; 3],
            };
            queue.write_buffer(
                &self.gpu().cull_uniforms_buffer,
                0,
                bytemuck::cast_slice(&[cull_uniforms]),
            );

            let (screen_width, screen_height) =
                if let Some((width, height)) = configs.view.viewport_size {
                    (width, height)
                } else {
                    (1920, 1080)
                };

            let (z_near, z_far) = configs
                .scene
                .render_view
                .as_ref()
                .map(|view| (view.z_near, view.z_far))
                .unwrap_or((0.1, 1000.0));

            let tile_size_x = (screen_width as f32) / (CLUSTER_GRID_X as f32);
            let tile_size_y = (screen_height as f32) / (CLUSTER_GRID_Y as f32);

            let inverse_projection: [[f32; 4]; 4] =
                nalgebra_glm::inverse(&camera_matrices.projection).into();

            let cluster_uniforms = ClusterUniforms {
                inverse_projection,
                screen_size: [screen_width as f32, screen_height as f32],
                z_near: cluster_slice_near(z_near),
                z_far,
                cluster_count: [CLUSTER_GRID_X, CLUSTER_GRID_Y, CLUSTER_GRID_Z, 0],
                tile_size: [tile_size_x, tile_size_y],
                num_lights: lights_data.len() as u32,
                num_directional_lights: stashed_num_directional_lights,
            };
            queue.write_buffer(
                &self.gpu().cluster_uniforms_buffer,
                0,
                bytemuck::cast_slice(&[cluster_uniforms]),
            );

            let view_matrix: [[f32; 4]; 4] = camera_matrices.view.into();
            queue.write_buffer(
                &self.view_matrix_buffer,
                0,
                bytemuck::cast_slice(&[view_matrix]),
            );

            let world_state = self.world_states[self.current_world_id as usize]
                .as_mut()
                .unwrap();
            let gpu = world_state.gpu_buffers.as_mut().unwrap();
            if gpu.cluster_assign_bind_group.is_none() {
                gpu.cluster_assign_bind_group =
                    Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Skinned Mesh Cluster Light Assign Bind Group (Per-World)"),
                        layout: &self.cluster_assign_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: gpu.cluster_uniforms_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: self.cluster_bounds_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: gpu.light_grid_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: gpu.light_indices_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 4,
                                resource: gpu.lights_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 5,
                                resource: self.view_matrix_buffer.as_entire_binding(),
                            },
                        ],
                    }));
            }
        }
    }
}
