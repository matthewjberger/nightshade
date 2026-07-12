use std::collections::HashMap;

use super::super::super::projection::*;
use super::super::pass::MeshPass;
use super::super::types::{
    BUFFER_GROWTH_FACTOR, BUFFER_SHRINK_THRESHOLD, COMPACTION_UTILIZATION_DROP_THRESHOLD,
    CullingUniforms, MAX_INSTANCES, MAX_LIGHTS, MaterialData, MeshUniforms, ModelMatrix,
    ObjectData,
};
use crate::wgpu::passes::geometry::material_gpu::{
    convert_material_to_gpu_data, default_material_data,
};
struct MaterialCollection {
    materials_data: Vec<MaterialData>,
    material_map: HashMap<freecs::Entity, u32>,
    name_to_material_id: HashMap<String, u32>,
    transparent_material_ids: std::collections::HashSet<u32>,
    double_sided_material_ids: std::collections::HashSet<u32>,
    mask_material_ids: std::collections::HashSet<u32>,
}

struct BatchAccumulator {
    transforms: Vec<ModelMatrix>,
    custom_data: Vec<[f32; 4]>,
    objects: Vec<ObjectData>,
    entities: Vec<freecs::Entity>,
}

struct EntityClass {
    pipeline_class: u32,
    is_overlay: u32,
    is_transparent: bool,
    is_mask: bool,
    is_double_sided: bool,
    skip_base: u32,
}

fn classify_entity(
    configs: &crate::wgpu::render_configs::RenderInputs,
    entity: freecs::Entity,
) -> EntityClass {
    let render_materials = &configs.scene.render_materials;
    let material_id = render_materials
        .entity_to_id
        .get(&entity)
        .copied()
        .unwrap_or(0);
    let is_transparent = render_materials.transparent_ids.contains(&material_id);
    let is_mask = render_materials.mask_ids.contains(&material_id);
    let material_double_sided = render_materials.double_sided_ids.contains(&material_id);

    let has_negative_scale = configs
        .scene
        .render_dynamic_objects
        .get(&entity)
        .map(|dynamic| {
            nalgebra_glm::determinant(&nalgebra_glm::mat4_to_mat3(&dynamic.transform)) < 0.0
        })
        .unwrap_or(false);
    let is_double_sided = material_double_sided || has_negative_scale;

    let is_overlay = configs
        .scene
        .render_dynamic_objects
        .get(&entity)
        .map(|dynamic| dynamic.is_overlay != 0)
        .unwrap_or(false);

    let pipeline_class = if is_overlay {
        if is_transparent {
            5
        } else if is_double_sided {
            4
        } else {
            3
        }
    } else if is_transparent {
        2
    } else if is_double_sided {
        1
    } else {
        0
    };

    EntityClass {
        pipeline_class,
        is_overlay: u32::from(is_overlay),
        is_transparent,
        is_mask,
        is_double_sided,
        skip_base: u32::from(pipeline_class == 2),
    }
}

impl MeshPass {
    fn collect_materials(
        &self,
        configs: &crate::wgpu::render_configs::RenderInputs,
    ) -> MaterialCollection {
        let render_materials = &configs.scene.render_materials;
        let mut materials_data = vec![default_material_data([0.7, 0.7, 0.7, 1.0])];
        for entry in &render_materials.entries {
            materials_data.push(convert_material_to_gpu_data(
                &entry.material,
                &entry.texture_ids,
                &self.material_layer_map,
            ));
        }

        MaterialCollection {
            materials_data,
            material_map: render_materials.entity_to_id.clone(),
            name_to_material_id: render_materials.name_to_id.clone(),
            transparent_material_ids: render_materials.transparent_ids.clone(),
            double_sided_material_ids: render_materials.double_sided_ids.clone(),
            mask_material_ids: render_materials.mask_ids.clone(),
        }
    }

    fn build_world_objects(
        &self,
        configs: &crate::wgpu::render_configs::RenderInputs,
        mesh_entities: &[freecs::Entity],
        materials: &mut MaterialCollection,
        accum: &mut BatchAccumulator,
        combos: &mut HashMap<(u32, u32, u32), u32>,
    ) {
        for &entity in mesh_entities {
            let render_layer = configs
                .scene
                .render_dynamic_objects
                .get(&entity)
                .map(|dynamic| dynamic.render_layer)
                .unwrap_or(crate::render_layer::RenderLayer::WORLD);

            let should_render = match render_layer {
                crate::render_layer::RenderLayer::WORLD => {
                    configs.settings.render_layer_world_enabled
                }
                crate::render_layer::RenderLayer::OVERLAY => {
                    configs.settings.render_layer_overlay_enabled
                }
                _ => true,
            };

            if !should_render {
                continue;
            }

            let Some(mesh_name) = configs.scene.render_object_meshes.get(&entity) else {
                continue;
            };
            let Some(dynamic) = configs.scene.render_dynamic_objects.get(&entity) else {
                continue;
            };
            let transform = dynamic.transform;
            let Some(&mesh_id) = self.meshes.get(mesh_name) else {
                continue;
            };

            let material_id = *materials.material_map.get(&entity).unwrap_or(&0);
            let is_overlay = u32::from(render_layer == crate::render_layer::RenderLayer::OVERLAY);

            let mesh_data = &self.mesh_data[mesh_id as usize];
            let transform_index = accum.transforms.len() as u32;
            let flip_winding =
                u32::from(nalgebra_glm::determinant(&nalgebra_glm::mat4_to_mat3(&transform)) < 0.0);

            let (pipeline_class, skip_occlusion) = if self.gpu_batching_enabled {
                (0u32, 0u32)
            } else {
                let class = classify_entity(configs, entity);
                if materials.material_map.contains_key(&entity) {
                    if class.is_transparent {
                        materials.transparent_material_ids.insert(material_id);
                    }
                    if class.is_mask {
                        materials.mask_material_ids.insert(material_id);
                    }
                    if class.is_double_sided {
                        materials.double_sided_material_ids.insert(material_id);
                    }
                }
                let skip =
                    class.skip_base | u32::from(materials.mask_material_ids.contains(&material_id));
                (class.pipeline_class, skip)
            };

            accum.transforms.push(ModelMatrix {
                model: transform.into(),
                normal_matrix: [[0.0; 4]; 3],
            });
            accum.custom_data.push([1.0, 1.0, 1.0, 1.0]);

            let morph_weights = dynamic.morph_weights;

            let culling_mask = configs
                .scene
                .render_dynamic_objects
                .get(&entity)
                .map(|dynamic| dynamic.culling_mask)
                .unwrap_or_default();
            accum.objects.push(ObjectData {
                transform_index,
                mesh_id,
                material_id,
                batch_id: 0,
                morph_weights,
                morph_target_count: mesh_data.morph_target_count,
                morph_displacement_offset: mesh_data.morph_displacement_offset,
                mesh_vertex_offset: mesh_data.vertex_offset,
                mesh_vertex_count: mesh_data.vertex_count,
                entity_id: entity.id,
                is_overlay,
                skip_occlusion,
                flip_winding,
                culling_mask,
                visible: dynamic.visible,
                pipeline_class,
                _pad_culling: [0; 1],
            });
            accum.entities.push(entity);
            if !self.gpu_batching_enabled {
                *combos
                    .entry((pipeline_class, mesh_id, material_id))
                    .or_default() += 1;
            }
        }
    }

    fn collect_and_upload_materials(
        &mut self,
        configs: &crate::wgpu::render_configs::RenderInputs,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> MaterialCollection {
        let materials = self.collect_materials(configs);

        let prev_materials_data = std::mem::take(&mut self.state_mut().cached_materials_data);
        let (material_ranges, full_materials) = super::super::world_state::diff_into_ranges(
            &prev_materials_data,
            &materials.materials_data,
        );
        super::super::world_state::validate_upload_ranges(
            &prev_materials_data,
            &materials.materials_data,
            &material_ranges,
            full_materials,
            "cached_materials_data",
        );
        {
            let gpu = self.gpu_mut();
            let buffer_resized = materials.materials_data.len() > gpu.materials_buffer_size;
            if buffer_resized {
                let new_size =
                    (materials.materials_data.len() as f32 * BUFFER_GROWTH_FACTOR).ceil() as usize;

                gpu.materials_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Mesh Materials Buffer (Per-World, Resized)"),
                    size: (std::mem::size_of::<MaterialData>() * new_size) as u64,
                    usage: wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                });
                gpu.materials_buffer_size = new_size;
            }

            if buffer_resized || full_materials || material_ranges.is_empty() {
                queue.write_buffer(
                    &gpu.materials_buffer,
                    0,
                    bytemuck::cast_slice(&materials.materials_data),
                );
            } else {
                let stride = std::mem::size_of::<MaterialData>() as u64;
                for (start, end) in &material_ranges {
                    let s = *start as usize;
                    let e = *end as usize;
                    queue.write_buffer(
                        &gpu.materials_buffer,
                        (*start as u64) * stride,
                        bytemuck::cast_slice(&materials.materials_data[s..e]),
                    );
                }
            }
        }
        self.state_mut().cached_materials_data = materials.materials_data.clone();
        self.rebuild_instance_bind_group(device);
        materials
    }

    /// Delta path for a material reload/edit. The material registry data is
    /// re-uploaded, then each affected tracked object is reclassified in place:
    /// if its pipeline class, material, overlay, or occlusion flag changed, the
    /// combo set is moved and the single object record re-uploaded. The GPU
    /// batch tables are then rebuilt from the combo set. No per-scene rebuild.
    pub(in super::super) fn material_reclass_update(
        &mut self,
        configs: &crate::wgpu::render_configs::RenderInputs,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let dirty: Vec<freecs::Entity> = self
            .frame_dirty
            .as_mut()
            .map(|fd| std::mem::take(&mut fd.material_dirty).into_iter().collect())
            .unwrap_or_default();

        let materials = self.collect_and_upload_materials(configs, device, queue);

        let mask_ids = materials.mask_material_ids.clone();

        {
            let state = self.state_mut();
            state.cached_name_to_material_id = materials.name_to_material_id.clone();
            state.cached_material_map = materials.material_map.clone();
            state.cached_transparent_material_ids = materials.transparent_material_ids.clone();
            state.cached_double_sided_material_ids = materials.double_sided_material_ids.clone();
            state.cached_mask_material_ids = materials.mask_material_ids.clone();
        }

        let object_size = std::mem::size_of::<ObjectData>() as u64;
        for entity in dirty {
            let class = classify_entity(configs, entity);
            let new_material_id = materials.material_map.get(&entity).copied().unwrap_or(0);
            let new_skip = class.skip_base | u32::from(mask_ids.contains(&new_material_id));

            let state = self.state_mut();
            let Some(&slot) = state.gpu_registry.entity_to_slot.get(&entity) else {
                continue;
            };
            let slot_index = slot as usize;
            if slot_index >= state.cached_objects.len() {
                continue;
            }

            let old = state.cached_objects[slot_index];
            let changed = old.pipeline_class != class.pipeline_class
                || old.material_id != new_material_id
                || old.is_overlay != class.is_overlay
                || old.skip_occlusion != new_skip;
            if !changed {
                continue;
            }

            let old_key = (old.pipeline_class, old.mesh_id, old.material_id);
            if let Some(count) = state.combos.get_mut(&old_key) {
                *count -= 1;
                if *count == 0 {
                    state.combos.remove(&old_key);
                }
            }
            *state
                .combos
                .entry((class.pipeline_class, old.mesh_id, new_material_id))
                .or_default() += 1;

            {
                let object = &mut state.cached_objects[slot_index];
                object.pipeline_class = class.pipeline_class;
                object.material_id = new_material_id;
                object.is_overlay = class.is_overlay;
                object.skip_occlusion = new_skip;
                object.batch_id = 0;
            }
            let updated = state.cached_objects[slot_index];
            state.cached_material_map.insert(entity, new_material_id);

            let gpu = state.gpu_buffers.as_ref().unwrap();
            queue.write_buffer(
                &gpu.object_buffer,
                slot as u64 * object_size,
                bytemuck::cast_slice(&[updated]),
            );
        }

        self.build_lists_from_combos(device, queue);
    }

    /// Appends every visible instanced-mesh entity's expanded per-instance
    /// objects onto the supplied buffers and combo set, and records each
    /// entity's transform range. Shared by the full rebuild and the instanced
    /// tail rebuild. The instanced objects always form the tail (after the
    /// regular objects already in `objects`).
    fn build_instanced_objects(
        &mut self,
        configs: &crate::wgpu::render_configs::RenderInputs,
        material_map: &HashMap<freecs::Entity, u32>,
        transforms: &mut Vec<ModelMatrix>,
        custom_data: &mut Vec<[f32; 4]>,
        objects: &mut Vec<ObjectData>,
        combos: &mut HashMap<(u32, u32, u32), u32>,
    ) {
        let instanced_data: Vec<(freecs::Entity, crate::config::InstancedObjectData)> = {
            let mut list: Vec<_> = configs
                .scene
                .render_instanced_objects
                .iter()
                .map(|(entity, data)| (*entity, data.clone()))
                .collect();
            list.sort_by_key(|(entity, _)| (entity.id, entity.generation));
            list
        };

        self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap()
            .instanced_transform_ranges
            .clear();

        for (entity, data) in &instanced_data {
            let entity = *entity;
            if data.visible == 0 || data.world_models.is_empty() {
                continue;
            }

            let Some(&mesh_id) = self.meshes.get(&data.mesh_name) else {
                continue;
            };

            let material_id = *material_map.get(&entity).unwrap_or(&0);

            let render_materials = &configs.scene.render_materials;
            let is_transparent = render_materials.transparent_ids.contains(&material_id);
            let is_mask = render_materials.mask_ids.contains(&material_id);
            let is_double_sided = render_materials.double_sided_ids.contains(&material_id);

            let start = objects.len() as u32;
            let base_transform_index = transforms.len() as u32;
            let instance_count = data.world_models.len();

            self.world_states[self.current_world_id as usize]
                .as_mut()
                .unwrap()
                .instanced_transform_ranges
                .insert(entity, (base_transform_index, instance_count as u32));

            transforms.extend(data.world_models.iter().zip(&data.world_normals).map(
                |(model, normal_matrix)| ModelMatrix {
                    model: *model,
                    normal_matrix: *normal_matrix,
                },
            ));
            custom_data.extend(data.custom_tints.iter().copied());

            let instanced_mesh_data = &self.mesh_data[mesh_id as usize];
            let instanced_entity_id = entity.id;
            let instanced_culling_mask = u32::MAX;
            let instanced_visible = data.visible;
            let instanced_pipeline_class = if is_transparent {
                8
            } else if is_double_sided {
                7
            } else {
                6
            };
            objects.extend((0..instance_count as u32).map(|index| ObjectData {
                transform_index: base_transform_index + index,
                mesh_id,
                material_id,
                batch_id: 0,
                morph_weights: [0.0f32; 8],
                morph_target_count: instanced_mesh_data.morph_target_count,
                morph_displacement_offset: instanced_mesh_data.morph_displacement_offset,
                mesh_vertex_offset: instanced_mesh_data.vertex_offset,
                mesh_vertex_count: instanced_mesh_data.vertex_count,
                entity_id: instanced_entity_id,
                is_overlay: 0,
                skip_occlusion: u32::from(is_transparent || is_mask),
                flip_winding: 0,
                culling_mask: instanced_culling_mask,
                visible: instanced_visible,
                pipeline_class: instanced_pipeline_class,
                _pad_culling: [0; 1],
            }));

            let end = objects.len() as u32;
            if start < end {
                *combos
                    .entry((instanced_pipeline_class, mesh_id, material_id))
                    .or_default() += instance_count as u32;
            }
        }
    }

    /// Delta path for instanced-mesh structural changes (spawn/despawn, instance
    /// count change). The regular object region is left in place (only its
    /// transforms are refreshed from the world, to absorb any concurrent moves);
    /// the instanced tail and its combos are rebuilt from the current instanced
    /// entities, and the GPU batch tables rebuilt from the combo set. The regular
    /// scene is never re-classified.
    pub(in super::super) fn instanced_tail_rebuild(
        &mut self,
        configs: &crate::wgpu::render_configs::RenderInputs,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        if let Some(fd) = self.frame_dirty.as_mut() {
            fd.instanced_meshes_changed = false;
        }

        let materials = self.collect_and_upload_materials(configs, device, queue);

        let regular = self.state().regular_object_count as usize;
        let regular_entities: Vec<freecs::Entity> = {
            let state = self.state();
            state
                .cached_entities
                .iter()
                .take(regular)
                .copied()
                .collect()
        };

        let mut transforms = std::mem::take(&mut self.state_mut().cached_transforms);
        let mut objects = std::mem::take(&mut self.state_mut().cached_objects);
        let mut custom_data = std::mem::take(&mut self.state_mut().cached_custom_data);
        transforms.truncate(regular);
        objects.truncate(regular);
        custom_data.truncate(regular);

        for (slot, &entity) in regular_entities.iter().enumerate() {
            if slot >= transforms.len() {
                break;
            }
            if let Some(dynamic) = configs.scene.render_dynamic_objects.get(&entity) {
                transforms[slot] = ModelMatrix {
                    model: dynamic.transform.into(),
                    normal_matrix: [[0.0; 4]; 3],
                };
            }
        }

        let mut combos = std::mem::take(&mut self.state_mut().combos);
        combos.retain(|&(pipeline_class, _, _), _| pipeline_class < 6);

        self.build_instanced_objects(
            configs,
            &materials.material_map,
            &mut transforms,
            &mut custom_data,
            &mut objects,
            &mut combos,
        );

        let new_object_count = objects.len() as u32;
        self.resize_gpu_buffers_if_needed(device, &transforms, &objects);

        {
            let state = self.state_mut();
            state.cached_transforms = transforms;
            state.cached_objects = objects;
            state.cached_custom_data = custom_data;
            state.combos = combos;
            state.object_count = new_object_count;
            state.cached_name_to_material_id = materials.name_to_material_id;
            state.cached_material_map = materials.material_map;
            state.cached_transparent_material_ids = materials.transparent_material_ids;
            state.cached_double_sided_material_ids = materials.double_sided_material_ids;
            state.cached_mask_material_ids = materials.mask_material_ids;
        }

        {
            let state = self.state();
            let gpu = state.gpu_buffers.as_ref().unwrap();
            if !state.cached_transforms.is_empty() {
                queue.write_buffer(
                    &gpu.transform_buffer,
                    0,
                    bytemuck::cast_slice(&state.cached_transforms),
                );
            }
            if !state.cached_objects.is_empty() {
                queue.write_buffer(
                    &gpu.object_buffer,
                    0,
                    bytemuck::cast_slice(&state.cached_objects),
                );
            }
            if !state.cached_custom_data.is_empty() {
                queue.write_buffer(
                    &gpu.custom_data_buffer,
                    0,
                    bytemuck::cast_slice(&state.cached_custom_data),
                );
            }
        }

        self.upload_instanced_local_data(configs, device, queue);
        self.build_culling_bind_groups(device);
        self.build_lists_from_combos(device, queue);
    }

    fn resize_gpu_buffers_if_needed(
        &mut self,
        device: &wgpu::Device,
        all_transforms: &[ModelMatrix],
        all_objects: &[ObjectData],
    ) {
        if all_transforms.len() > self.gpu().transform_buffer_size {
            let new_size = std::cmp::min(
                (all_transforms.len() as f32 * BUFFER_GROWTH_FACTOR).ceil() as usize,
                MAX_INSTANCES,
            );

            if new_size > self.gpu().transform_buffer_size {
                let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Mesh Transform Buffer (Resized)"),
                    size: (std::mem::size_of::<ModelMatrix>() * new_size) as u64,
                    usage: wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                });

                self.gpu_mut().transform_buffer = new_buffer;
                self.gpu_mut().transform_buffer_size = new_size;

                self.gpu_mut().custom_data_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Mesh Custom Data Buffer (Resized)"),
                    size: (std::mem::size_of::<[f32; 4]>() * new_size) as u64,
                    usage: wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                });
                self.gpu_mut().custom_data_buffer_size = new_size;

                self.rebuild_instance_bind_group(device);
            }
        }

        if all_objects.len() > self.gpu().object_buffer_size {
            let new_size = std::cmp::min(
                (all_objects.len() as f32 * BUFFER_GROWTH_FACTOR).ceil() as usize,
                MAX_INSTANCES,
            );

            if new_size > self.gpu().object_buffer_size {
                let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Mesh Object Buffer (Resized)"),
                    size: (std::mem::size_of::<ObjectData>() * new_size) as u64,
                    usage: wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                });

                self.gpu_mut().object_buffer = new_buffer;
                self.gpu_mut().object_buffer_size = new_size;
                self.gpu_mut().batch_assign_bind_group = None;

                self.rebuild_instance_bind_group(device);
            }
        }
    }

    fn write_buffer_data(
        &self,
        queue: &wgpu::Queue,
        all_transforms: &[ModelMatrix],
        all_custom_data: &[[f32; 4]],
        all_objects: &[ObjectData],
        ranges: &super::super::world_state::WorldUploadRanges,
    ) {
        if all_transforms.len() <= self.gpu().transform_buffer_size {
            if ranges.full_transforms || ranges.transform_ranges.is_empty() {
                queue.write_buffer(
                    &self.gpu().transform_buffer,
                    0,
                    bytemuck::cast_slice(all_transforms),
                );
            } else {
                let stride = std::mem::size_of::<ModelMatrix>() as u64;
                for (start, end) in &ranges.transform_ranges {
                    let s = *start as usize;
                    let e = *end as usize;
                    queue.write_buffer(
                        &self.gpu().transform_buffer,
                        (*start as u64) * stride,
                        bytemuck::cast_slice(&all_transforms[s..e]),
                    );
                }
            }
        }

        if all_custom_data.len() <= self.gpu().custom_data_buffer_size {
            if ranges.full_custom_data || ranges.custom_data_ranges.is_empty() {
                queue.write_buffer(
                    &self.gpu().custom_data_buffer,
                    0,
                    bytemuck::cast_slice(all_custom_data),
                );
            } else {
                let stride = std::mem::size_of::<[f32; 4]>() as u64;
                for (start, end) in &ranges.custom_data_ranges {
                    let s = *start as usize;
                    let e = *end as usize;
                    queue.write_buffer(
                        &self.gpu().custom_data_buffer,
                        (*start as u64) * stride,
                        bytemuck::cast_slice(&all_custom_data[s..e]),
                    );
                }
            }
        }

        if all_objects.len() <= self.gpu().object_buffer_size {
            if ranges.full_objects || ranges.object_ranges.is_empty() {
                queue.write_buffer(
                    &self.gpu().object_buffer,
                    0,
                    bytemuck::cast_slice(all_objects),
                );
            } else {
                let stride = std::mem::size_of::<ObjectData>() as u64;
                for (start, end) in &ranges.object_ranges {
                    let s = *start as usize;
                    let e = *end as usize;
                    queue.write_buffer(
                        &self.gpu().object_buffer,
                        (*start as u64) * stride,
                        bytemuck::cast_slice(&all_objects[s..e]),
                    );
                }
            }
        }
    }

    fn upload_instanced_local_data(
        &mut self,
        configs: &crate::wgpu::render_configs::RenderInputs,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        if self.state().instanced_transform_ranges.is_empty() {
            return;
        }

        let mut instanced_local_data: Vec<[[f32; 4]; 4]> = Vec::new();
        let ranges: Vec<_> = self
            .state()
            .instanced_transform_ranges
            .iter()
            .map(|(&entity, &(start, count))| (entity, start, count))
            .collect();

        let mut max_end: usize = 0;
        for &(_, start, count) in &ranges {
            let end = start as usize + count as usize;
            if end > max_end {
                max_end = end;
            }
        }
        instanced_local_data.resize(max_end, [[0.0; 4]; 4]);

        for &(entity, start, count) in &ranges {
            if let Some(instanced) = configs.scene.render_instanced_objects.get(&entity) {
                let local_matrices = &instanced.local_matrices;
                let start_idx = start as usize;
                for (offset, mat) in local_matrices.iter().take(count as usize).enumerate() {
                    let raw: [[f32; 4]; 4] = (*mat).into();
                    instanced_local_data[start_idx + offset] = raw;
                }
            }
        }

        self.upload_instanced_local_matrices(device, queue, &instanced_local_data);
        self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap()
            .instanced_compute_dirty = true;
    }

    pub(super) fn build_culling_bind_groups(&mut self, device: &wgpu::Device) {
        let hiz_view = self.hiz.hiz_view_or_dummy();
        let mesh_bounds_buffer = &self.mesh_bounds_buffer;
        let mesh_lod_buffer = &self.mesh_lod_buffer;
        let layout = &self.culling_bind_group_layout;
        let uniform_occlusion = &self.culling_uniform_buffer;
        let uniform_frustum = &self.culling_uniform_buffer_frustum;
        let world_state = self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap();
        let gpu = world_state.gpu_buffers.as_mut().unwrap();

        let shared = [
            wgpu::BindGroupEntry {
                binding: 0,
                resource: gpu.transform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: gpu.object_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: mesh_bounds_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: gpu.indirect_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: gpu.visible_indices_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: mesh_lod_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 7,
                resource: wgpu::BindingResource::TextureView(hiz_view),
            },
        ];

        let mut occlusion_entries = shared.to_vec();
        occlusion_entries.push(wgpu::BindGroupEntry {
            binding: 2,
            resource: uniform_occlusion.as_entire_binding(),
        });
        let mut frustum_entries = shared.to_vec();
        frustum_entries.push(wgpu::BindGroupEntry {
            binding: 2,
            resource: uniform_frustum.as_entire_binding(),
        });

        gpu.culling_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Mesh Culling Bind Group"),
            layout,
            entries: &occlusion_entries,
        }));
        gpu.culling_bind_group_frustum =
            Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Mesh Culling Bind Group (Frustum Prepass)"),
                layout,
                entries: &frustum_entries,
            }));
    }

    fn write_culling_uniforms(
        &self,
        configs: &crate::wgpu::render_configs::RenderInputs,
        queue: &wgpu::Queue,
    ) {
        if let Some(camera_matrices) = configs.scene.render_view.as_ref() {
            let view_proj = camera_matrices.projection * camera_matrices.view;
            let frustum_planes = extract_frustum_planes(&view_proj);

            let (screen_width, screen_height) = configs.view.viewport_size.unwrap_or((1, 1));
            let projection_scale_y = camera_matrices.projection[(1, 1)];
            let min_screen_pixel_size = configs.scene.min_screen_pixel_size;

            let culling_uniforms = CullingUniforms {
                frustum_planes: frustum_planes.map(|v| [v.x, v.y, v.z, v.w]),
                view_projection: view_proj.into(),
                occluder_view_projection: view_proj.into(),
                screen_size: [screen_width as f32, screen_height as f32],
                object_count: self.state().object_count,
                min_screen_pixel_size,
                projection_scale_y,
                camera_culling_mask: configs.scene.active_view.culling_mask,
                hiz_mip_count: self.hiz.mip_count() as f32,
                occlusion_enabled: u32::from(
                    self.hiz.hiz_view().is_some()
                        && configs.scene.occlusion_culling_enabled
                        && configs.scene.gpu_culling_enabled,
                ),
                frustum_enabled: u32::from(configs.scene.gpu_culling_enabled),
                _pad_culling: [0; 3],
            };
            queue.write_buffer(
                &self.culling_uniform_buffer,
                0,
                bytemuck::cast_slice(&[culling_uniforms]),
            );

            let frustum_only_uniforms = CullingUniforms {
                occlusion_enabled: 0,
                ..culling_uniforms
            };
            queue.write_buffer(
                &self.culling_uniform_buffer_frustum,
                0,
                bytemuck::cast_slice(&[frustum_only_uniforms]),
            );
        }
    }

    pub(in super::super) fn prepare_pass_node(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &crate::wgpu::render_configs::RenderInputs,
    ) {
        let incoming_world_id = configs.world_id;
        if self.current_world_id != incoming_world_id {
            self.scene_bind_group_dirty = true;
        }
        self.current_world_id = incoming_world_id;
        self.frame_counter += 1;
        self.gpu_batching_enabled =
            configs.scene.gpu_batching_enabled && self.supports_multi_draw_count;

        if self.frame_counter.is_multiple_of(300) {
            self.cleanup_stale_world_states(600);
        }

        self.ensure_world_gpu_buffers(device, self.current_world_id);
        self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap()
            .last_used_frame = self.frame_counter;
        self.last_prepared_world_id = Some(self.current_world_id);

        if let Some(fd) = self.frame_dirty.as_ref()
            && fd.instanced_compute_dispatch_needed
        {
            self.world_states[self.current_world_id as usize]
                .as_mut()
                .unwrap()
                .instanced_compute_dirty = true;
        }

        let frame_initialized = self
            .frame_dirty
            .as_ref()
            .is_some_and(|fd| fd.frame_initialized);
        let has_any_changes = self.frame_dirty.as_ref().is_some_and(|fd| {
            !fd.transform_dirty.is_empty()
                || !fd.material_dirty.is_empty()
                || !fd.entities_added.is_empty()
                || !fd.entities_removed.is_empty()
                || fd.batches_invalidated
                || fd.full_rebuild_needed
                || fd.instanced_meshes_changed
        }) || self.resize_full_rebuild_pending;

        if self.resize_full_rebuild_pending {
            if let Some(fd) = self.frame_dirty.as_mut() {
                fd.full_rebuild_needed = true;
            }
            self.resize_full_rebuild_pending = false;
        }

        if frame_initialized && !has_any_changes {
            self.sync_dynamic_object_state(configs, queue);
            self.write_culling_uniforms(configs, queue);
            self.prepare_uniforms_and_lights(device, queue, configs);
            return;
        }

        if self.can_do_incremental_update() && self.update_dirty_transforms(configs, queue) {
            self.sync_dynamic_object_state(configs, queue);
            self.write_culling_uniforms(configs, queue);
            self.prepare_uniforms_and_lights(device, queue, configs);
            return;
        }

        self.sync_meshes_from_cache(device, queue, &configs.mesh_cache);
        self.sync_textures(&configs.texture_cache);

        if self.can_do_incremental_entity_update(configs) {
            self.incremental_update_entities(configs, device, queue);
            self.sync_dynamic_object_state(configs, queue);
            self.write_culling_uniforms(configs, queue);
            self.prepare_uniforms_and_lights(device, queue, configs);
            return;
        }

        if self.can_do_rebatch_only() {
            self.rebatch_cached_entities(configs, device, queue);
            self.write_culling_uniforms(configs, queue);
            self.prepare_uniforms_and_lights(device, queue, configs);
            return;
        }

        if self.can_do_material_reclass_only() {
            self.material_reclass_update(configs, device, queue);
            self.write_culling_uniforms(configs, queue);
            self.prepare_uniforms_and_lights(device, queue, configs);
            return;
        }

        if self.can_do_instanced_tail_update(configs) {
            self.instanced_tail_rebuild(configs, device, queue);
            self.write_culling_uniforms(configs, queue);
            self.prepare_uniforms_and_lights(device, queue, configs);
            return;
        }

        let current_vertex_utilization = self.compute_vertex_utilization();
        let current_index_utilization = self.compute_index_utilization();

        let vertex_drop = self.last_vertex_utilization - current_vertex_utilization;
        let index_drop = self.last_index_utilization - current_index_utilization;

        let significant_drop = vertex_drop > COMPACTION_UTILIZATION_DROP_THRESHOLD
            || index_drop > COMPACTION_UTILIZATION_DROP_THRESHOLD;
        let below_threshold = current_vertex_utilization < BUFFER_SHRINK_THRESHOLD
            || current_index_utilization < BUFFER_SHRINK_THRESHOLD;

        self.last_vertex_utilization = current_vertex_utilization;
        self.last_index_utilization = current_index_utilization;

        if significant_drop || below_threshold {
            self.check_and_compact_buffers(device, queue, &configs.mesh_cache);
        }

        let lighting = configs.scene.render_lighting.clone().unwrap_or_default();
        let mut lights_data = lighting.lights_data;
        let entity_to_lights_index = lighting.entity_to_lights_index;

        crate::wgpu::passes::geometry::projection::resolve_cookie_layers(
            configs,
            &mut lights_data,
            &entity_to_lights_index,
            &self.material_layer_map,
            &configs.texture_cache.registry,
        );

        let directional_light_direction = lighting.directional_light_direction;

        let cascade_view_projections = lighting.cascade_view_projections;
        let cascade_diameters = lighting.cascade_diameters;
        let cascade_split_distances = lighting.cascade_split_distances;
        let light_view_projection = lighting.light_view_projection;
        let shadow_bias = lighting.shadow_bias;
        let shadows_enabled = lighting.shadows_enabled;

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
                    (fog.color, 1, fog.start, fog.end)
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

            let uniforms = MeshUniforms {
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
                shadow_normal_bias: 1.8,
                snap_resolution,
                snap_enabled,
                affine_enabled,
                fog_color,
                fog_enabled,
                fog_start,
                fog_end,
                cascade_count: crate::wgpu::passes::NUM_SHADOW_CASCADES as u32,
                directional_light_size: 1.0,
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

            let overlay_uniforms = MeshUniforms {
                shadows_enabled: 0.0,
                ..uniforms
            };
            queue.write_buffer(
                &self.overlay_uniform_buffer,
                0,
                bytemuck::cast_slice(&[overlay_uniforms]),
            );
        }

        let camera_position = configs
            .scene
            .render_view
            .as_ref()
            .map(|view| view.camera_position)
            .unwrap_or_else(|| nalgebra_glm::vec3(0.0, 0.0, 0.0));

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

        let point_shadow_result = collect_point_light_shadows(
            configs,
            camera_position,
            &mut lights_data,
            &entity_to_lights_index,
        );
        if !point_shadow_result.is_empty() {
            queue.write_buffer(
                &self.point_shadow_buffer,
                0,
                bytemuck::cast_slice(&point_shadow_result),
            );
        }

        if !lights_data.is_empty() {
            if lights_data.len() > self.gpu().lights_buffer_size {
                let new_size = (lights_data.len() as f32 * BUFFER_GROWTH_FACTOR).ceil() as usize;
                let new_size = new_size.min(MAX_LIGHTS);

                let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Mesh Lights Buffer (Resized)"),
                    size: (std::mem::size_of::<LightData>() * new_size) as u64,
                    usage: wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                });

                self.gpu_mut().lights_buffer = new_buffer;
                self.gpu_mut().lights_buffer_size = new_size;
                self.rebuild_instance_bind_group(device);
            }

            queue.write_buffer(
                &self.gpu().lights_buffer,
                0,
                bytemuck::cast_slice(&lights_data),
            );
        }

        let mesh_entities: Vec<_> = configs.scene.render_regular_mesh_entities.clone();

        let mut materials = self.collect_and_upload_materials(configs, device, queue);

        self.resolve_lod_chains(&configs.scene.mesh_lod_chains, queue);

        let mut accum = BatchAccumulator {
            transforms: Vec::new(),
            custom_data: Vec::new(),
            objects: Vec::new(),
            entities: Vec::new(),
        };
        let mut combos: HashMap<(u32, u32, u32), u32> = HashMap::new();

        self.build_world_objects(
            configs,
            &mesh_entities,
            &mut materials,
            &mut accum,
            &mut combos,
        );

        self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap()
            .regular_object_count = accum.transforms.len() as u32;

        self.build_instanced_objects(
            configs,
            &materials.material_map,
            &mut accum.transforms,
            &mut accum.custom_data,
            &mut accum.objects,
            &mut combos,
        );

        let all_custom_data = accum.custom_data;

        let all_entities = accum.entities.clone();

        let prev_transforms = std::mem::take(
            &mut self.world_states[self.current_world_id as usize]
                .as_mut()
                .unwrap()
                .cached_transforms,
        );
        let prev_objects = std::mem::take(
            &mut self.world_states[self.current_world_id as usize]
                .as_mut()
                .unwrap()
                .cached_objects,
        );
        let prev_custom_data = std::mem::take(
            &mut self.world_states[self.current_world_id as usize]
                .as_mut()
                .unwrap()
                .cached_custom_data,
        );

        self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap()
            .object_count = accum.objects.len() as u32;
        self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap()
            .cached_transforms = accum.transforms;
        self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap()
            .cached_objects = accum.objects;
        self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap()
            .cached_entities = all_entities.clone();

        self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap()
            .entity_to_transform_index
            .clear();
        for (index, entity) in all_entities.iter().enumerate() {
            self.world_states[self.current_world_id as usize]
                .as_mut()
                .unwrap()
                .entity_to_transform_index
                .insert(*entity, index as u32);
        }

        let (transform_ranges, full_transforms) = super::super::world_state::diff_into_ranges(
            &prev_transforms,
            &self.state().cached_transforms,
        );
        let (object_ranges, full_objects) = super::super::world_state::diff_into_ranges(
            &prev_objects,
            &self.state().cached_objects,
        );
        let (custom_data_ranges, full_custom_data) =
            super::super::world_state::diff_into_ranges(&prev_custom_data, &all_custom_data);

        super::super::world_state::validate_upload_ranges(
            &prev_transforms,
            &self.state().cached_transforms,
            &transform_ranges,
            full_transforms,
            "cached_transforms",
        );
        super::super::world_state::validate_upload_ranges(
            &prev_objects,
            &self.state().cached_objects,
            &object_ranges,
            full_objects,
            "cached_objects",
        );
        super::super::world_state::validate_upload_ranges(
            &prev_custom_data,
            &all_custom_data,
            &custom_data_ranges,
            full_custom_data,
            "cached_custom_data",
        );

        let upload_ranges = super::super::world_state::WorldUploadRanges {
            transform_ranges,
            object_ranges,
            custom_data_ranges,
            full_transforms,
            full_objects,
            full_custom_data,
        };

        if !self.state().cached_objects.is_empty() {
            let transforms = self.state().cached_transforms.clone();
            let objects = self.state().cached_objects.clone();
            self.resize_gpu_buffers_if_needed(device, &transforms, &objects);
            self.write_buffer_data(
                queue,
                &transforms,
                &all_custom_data,
                &objects,
                &upload_ranges,
            );
            self.upload_instanced_local_data(configs, device, queue);
            self.build_culling_bind_groups(device);
        }

        self.write_culling_uniforms(configs, queue);

        self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap()
            .cached_transparent_material_ids = materials.transparent_material_ids.clone();
        self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap()
            .cached_double_sided_material_ids = materials.double_sided_material_ids.clone();
        self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap()
            .cached_mask_material_ids = materials.mask_material_ids.clone();
        self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap()
            .combos = combos;
        self.build_lists_from_combos(device, queue);

        self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap()
            .cached_name_to_material_id = materials.name_to_material_id;
        self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap()
            .cached_material_map = materials.material_map;
        self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap()
            .cached_custom_data = all_custom_data;
        self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap()
            .frames_since_full_rebuild = 0;

        self.populate_gpu_registry_from_instances();

        self.prepare_uniforms_and_lights(device, queue, configs);
    }
}
