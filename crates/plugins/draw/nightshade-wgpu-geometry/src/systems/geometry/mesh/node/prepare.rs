use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;
use std::collections::HashMap;

use super::super::pass::MeshPass;
use super::super::pass::debug_view_exempt;
use super::super::types::{
    BUFFER_GROWTH_FACTOR, BUFFER_SHRINK_THRESHOLD, COMPACTION_UTILIZATION_DROP_THRESHOLD,
    CullingUniforms, MAX_LIGHTS, MaterialData, ModelMatrix, ObjectData, SceneUniforms,
    grown_buffer_size,
};
use nightshade_wgpu_shading::backend::material_gpu::{
    convert_material_to_gpu_data, default_material_data,
};
use nightshade_wgpu_shading::backend::projection::*;
pub(in super::super) struct MaterialCollection {
    pub(in super::super) materials_data: Vec<MaterialData>,
    pub(in super::super) material_ids: std::collections::HashSet<u32>,
    pub(in super::super) transparent_material_ids: std::collections::HashSet<u32>,
    pub(in super::super) double_sided_material_ids: std::collections::HashSet<u32>,
    pub(in super::super) mask_material_ids: std::collections::HashSet<u32>,
    pub(in super::super) transmissive_material_ids: std::collections::HashSet<u32>,
}

struct BatchAccumulator {
    transforms: Vec<ModelMatrix>,
    custom_data: Vec<[f32; 4]>,
    objects: Vec<ObjectData>,
    entities: Vec<nightshade::ecs::Entity>,
}

struct EntityClass {
    pipeline_class: u32,
    is_overlay: u32,
    is_transparent: bool,
    is_mask: bool,
    is_double_sided: bool,
    skip_base: u32,
}

fn collect_entity_materials(
    configs: &nightshade_wgpu::backend::RenderInputs,
) -> HashMap<nightshade::ecs::Entity, u32> {
    configs
        .scene_world
        .query_ref::<&nightshade_render::components::render_world::MaterialId>()
        .iter()
        .map(|(entity, material)| (entity, material.0))
        .collect()
}

fn classify_entity(
    configs: &nightshade_wgpu::backend::RenderInputs,
    entity: nightshade::ecs::Entity,
) -> EntityClass {
    let render_materials = &configs.scene.render_materials;
    let material_id = configs
        .scene_world
        .get::<nightshade_render::components::render_world::MaterialId>(entity)
        .map(|material| material.0)
        .unwrap_or(0);
    let is_transparent = render_materials.transparent_ids.contains(&material_id);
    let is_transmissive = render_materials.transmissive_ids.contains(&material_id);
    let is_mask = render_materials.mask_ids.contains(&material_id);
    let material_double_sided = render_materials.double_sided_ids.contains(&material_id);

    let has_negative_scale = configs
        .scene_world
        .get::<nightshade_render::components::render_world::Transform>(entity)
        .map(|transform| nalgebra_glm::determinant(&nalgebra_glm::mat4_to_mat3(&transform.0)) < 0.0)
        .unwrap_or(false);
    let is_double_sided = material_double_sided || has_negative_scale;

    let is_overlay = configs
        .scene_world
        .get::<nightshade_render::components::render_world::Layer>(entity)
        .map(|layer| layer.0 == nightshade_render::data::render_layer::RenderLayer::OVERLAY)
        .unwrap_or(false);

    let pipeline_class = if is_overlay {
        if is_transparent {
            5
        } else if is_double_sided {
            4
        } else {
            3
        }
    } else if is_transmissive {
        9
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
        configs: &nightshade_wgpu::backend::RenderInputs,
    ) -> MaterialCollection {
        let render_materials = &configs.scene.render_materials;
        let mut materials_data = vec![default_material_data([0.7, 0.7, 0.7, 1.0])];
        for entry in &render_materials.entries {
            materials_data.push(convert_material_to_gpu_data(
                &entry.material,
                &entry.texture_ids,
                &self.materials.layer_map,
            ));
        }

        MaterialCollection {
            materials_data,
            material_ids: render_materials.material_ids.clone(),
            transparent_material_ids: render_materials.transparent_ids.clone(),
            double_sided_material_ids: render_materials.double_sided_ids.clone(),
            mask_material_ids: render_materials.mask_ids.clone(),
            transmissive_material_ids: render_materials.transmissive_ids.clone(),
        }
    }

    fn build_world_objects(
        &self,
        configs: &nightshade_wgpu::backend::RenderInputs,
        materials: &mut MaterialCollection,
        accum: &mut BatchAccumulator,
        combos: &mut HashMap<(u32, u32, u32), u32>,
    ) {
        struct ObjectSource {
            entity: nightshade::ecs::Entity,
            transform: nalgebra_glm::Mat4,
            mesh_id: u32,
            material_id: u32,
            visible: u32,
            morph_weights: [f32; 8],
            culling_mask: u32,
            render_layer: u32,
        }

        let sources: Vec<ObjectSource> = configs
            .scene_world
            .query_ref::<(
                &nightshade_render::components::render_world::Transform,
                &nightshade_render::components::render_world::MeshKey,
                &nightshade_render::components::render_world::MaterialId,
                &nightshade_render::components::render_world::Visible,
                &nightshade_render::components::render_world::MorphWeights,
                &nightshade_render::components::render_world::CullingMask,
                &nightshade_render::components::render_world::Layer,
            )>()
            .iter()
            .filter_map(
                |(entity, (transform, mesh, material, visible, morph, culling, layer))| {
                    Some(ObjectSource {
                        entity,
                        transform: transform.0,
                        mesh_id: *self.registry.ids.get(&mesh.0)?,
                        material_id: material.0,
                        visible: u32::from(visible.0),
                        morph_weights: morph.0,
                        culling_mask: culling.0,
                        render_layer: layer.0,
                    })
                },
            )
            .collect();

        for source in sources {
            let render_layer = source.render_layer;
            let should_render = match render_layer {
                nightshade_render::data::render_layer::RenderLayer::WORLD => {
                    configs.settings.render_layer_world_enabled
                }
                nightshade_render::data::render_layer::RenderLayer::OVERLAY => {
                    configs.settings.render_layer_overlay_enabled
                }
                _ => true,
            };
            if !should_render {
                continue;
            }

            let mesh_id = source.mesh_id;
            let entity = source.entity;
            let transform = source.transform;
            let material_id = source.material_id;
            let is_overlay = u32::from(
                render_layer == nightshade_render::data::render_layer::RenderLayer::OVERLAY,
            );

            let mesh_data = &self.registry.data[mesh_id as usize];
            let transform_index = accum.transforms.len() as u32;
            let flip_winding =
                u32::from(nalgebra_glm::determinant(&nalgebra_glm::mat4_to_mat3(&transform)) < 0.0);

            let (pipeline_class, skip_occlusion) = if self.compute.batching_enabled {
                (0u32, 0u32)
            } else {
                let class = classify_entity(configs, entity);
                if configs
                    .scene_world
                    .get::<nightshade_render::components::render_world::MaterialId>(entity)
                    .is_some()
                {
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

            accum.objects.push(ObjectData {
                transform_index,
                mesh_id,
                material_id,
                batch_id: 0,
                morph_weights: source.morph_weights,
                morph_target_count: mesh_data.morph_target_count,
                morph_displacement_offset: mesh_data.morph_displacement_offset,
                mesh_vertex_offset: mesh_data.vertex_offset,
                mesh_vertex_count: mesh_data.vertex_count,
                entity_id: entity.id,
                is_overlay,
                skip_occlusion,
                flip_winding,
                culling_mask: source.culling_mask,
                visible: source.visible,
                pipeline_class,
                debug_view_exempt: u32::from(debug_view_exempt(&configs.scene_world, entity)),
            });
            accum.entities.push(entity);
            if !self.compute.batching_enabled {
                *combos
                    .entry((pipeline_class, mesh_id, material_id))
                    .or_default() += 1;
            }
        }
    }

    pub(in super::super) fn collect_and_upload_materials(
        &mut self,
        configs: &nightshade_wgpu::backend::RenderInputs,
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

                gpu.materials_buffer = build::buffer(
                    device,
                    "Mesh Materials Buffer (Per-World, Resized)",
                    (std::mem::size_of::<MaterialData>() * new_size) as u64,
                    wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                );
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
        {
            let state = self.state_mut();
            state.cached_materials_data = materials.materials_data.clone();
            state.cached_render_materials_generation = configs.scene.render_materials.generation;
            state.cached_material_ids = materials.material_ids.clone();
            state.cached_transparent_material_ids = materials.transparent_material_ids.clone();
            state.cached_double_sided_material_ids = materials.double_sided_material_ids.clone();
            state.cached_mask_material_ids = materials.mask_material_ids.clone();
            state.cached_transmissive_material_ids = materials.transmissive_material_ids.clone();
        }
        self.rebuild_instance_bind_group(device);
        materials
    }

    pub(in super::super) fn material_reclass_update(
        &mut self,
        configs: &nightshade_wgpu::backend::RenderInputs,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let dirty: Vec<nightshade::ecs::Entity> = self
            .frame
            .dirty
            .as_mut()
            .map(|fd| std::mem::take(&mut fd.material_dirty).into_iter().collect())
            .unwrap_or_default();

        let materials = self.collect_and_upload_materials(configs, device, queue);

        let mask_ids = materials.mask_material_ids.clone();

        let object_size = std::mem::size_of::<ObjectData>() as u64;
        for entity in dirty {
            let class = classify_entity(configs, entity);
            let new_material_id = configs
                .scene_world
                .get::<nightshade_render::components::render_world::MaterialId>(entity)
                .map(|material| material.0)
                .unwrap_or(0);
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

    fn build_instanced_objects(
        &mut self,
        configs: &nightshade_wgpu::backend::RenderInputs,
        transforms: &mut Vec<ModelMatrix>,
        custom_data: &mut Vec<[f32; 4]>,
        objects: &mut Vec<ObjectData>,
        combos: &mut HashMap<(u32, u32, u32), u32>,
    ) {
        let instanced_data: Vec<(
            nightshade::ecs::Entity,
            u32,
            nightshade_render::data::config::InstancedObjectData,
        )> = {
            let mut list: Vec<_> = configs
                .scene_world
                .query_ref::<(
                    &nightshade_render::data::config::InstancedObjectData,
                    &nightshade_render::components::render_world::MaterialId,
                )>()
                .iter()
                .map(|(entity, (data, material))| (entity, material.0, data.clone()))
                .collect::<Vec<_>>();
            list.sort_by_key(|(entity, _, _)| (entity.id, entity.generation));
            list
        };

        self.state
            .as_mut()
            .unwrap()
            .instanced_transform_ranges
            .clear();

        for (entity, material_id, data) in &instanced_data {
            let entity = *entity;
            let material_id = *material_id;
            if data.visible == 0 || data.world_models.is_empty() {
                continue;
            }

            let Some(&mesh_id) = self.registry.ids.get(
                &nightshade_render::data::generational_registry::registry_name_hash(
                    &data.mesh_name,
                ),
            ) else {
                continue;
            };

            let render_materials = &configs.scene.render_materials;
            let is_transparent = render_materials.transparent_ids.contains(&material_id);
            let is_mask = render_materials.mask_ids.contains(&material_id);
            let is_double_sided = render_materials.double_sided_ids.contains(&material_id);

            let start = objects.len() as u32;
            let base_transform_index = transforms.len() as u32;
            let instance_count = data.world_models.len();

            self.state
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

            let instanced_mesh_data = &self.registry.data[mesh_id as usize];
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
                debug_view_exempt: 0,
            }));

            let end = objects.len() as u32;
            if start < end {
                *combos
                    .entry((instanced_pipeline_class, mesh_id, material_id))
                    .or_default() += instance_count as u32;
            }
        }
    }

    pub(in super::super) fn instanced_tail_rebuild(
        &mut self,
        configs: &nightshade_wgpu::backend::RenderInputs,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        if let Some(fd) = self.frame.dirty.as_mut() {
            fd.instanced_meshes_changed = false;
        }

        let materials = self.collect_and_upload_materials(configs, device, queue);

        let regular = self.state().regular_object_count as usize;
        let regular_entities: Vec<nightshade::ecs::Entity> = {
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
            if let Some(transform) =
                configs
                    .scene_world
                    .get::<nightshade_render::components::render_world::Transform>(entity)
            {
                transforms[slot] = ModelMatrix {
                    model: transform.0.into(),
                    normal_matrix: [[0.0; 4]; 3],
                };
            }
        }

        let mut combos = std::mem::take(&mut self.state_mut().combos);
        combos.retain(|&(pipeline_class, _, _), _| pipeline_class < 6);

        self.build_instanced_objects(
            configs,
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
            state.cached_material_ids = materials.material_ids;
            state.cached_material_map = collect_entity_materials(configs);
            state.cached_transparent_material_ids = materials.transparent_material_ids;
            state.cached_double_sided_material_ids = materials.double_sided_material_ids;
            state.cached_transmissive_material_ids = materials.transmissive_material_ids.clone();
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
            let new_size = grown_buffer_size(all_transforms.len());

            if new_size > self.gpu().transform_buffer_size {
                let new_buffer = build::buffer(
                    device,
                    "Mesh Transform Buffer (Resized)",
                    (std::mem::size_of::<ModelMatrix>() * new_size) as u64,
                    wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                );

                self.gpu_mut().transform_buffer = new_buffer;
                self.gpu_mut().transform_buffer_size = new_size;

                self.gpu_mut().custom_data_buffer = build::buffer(
                    device,
                    "Mesh Custom Data Buffer (Resized)",
                    (std::mem::size_of::<[f32; 4]>() * new_size) as u64,
                    wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                );
                self.gpu_mut().custom_data_buffer_size = new_size;

                self.rebuild_instance_bind_group(device);
            }
        }

        if all_objects.len() > self.gpu().object_buffer_size {
            let new_size = grown_buffer_size(all_objects.len());

            if new_size > self.gpu().object_buffer_size {
                let new_buffer = build::buffer(
                    device,
                    "Mesh Object Buffer (Resized)",
                    (std::mem::size_of::<ObjectData>() * new_size) as u64,
                    wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                );

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
        configs: &nightshade_wgpu::backend::RenderInputs,
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
            let instanced = configs
                .scene_world
                .get::<nightshade_render::data::config::InstancedObjectData>(entity)
                .cloned();
            if let Some(instanced) = instanced.as_ref() {
                let local_matrices = &instanced.local_matrices;
                let start_idx = start as usize;
                for (offset, mat) in local_matrices.iter().take(count as usize).enumerate() {
                    let raw: [[f32; 4]; 4] = (*mat).into();
                    instanced_local_data[start_idx + offset] = raw;
                }
            }
        }

        self.upload_instanced_local_matrices(device, queue, &instanced_local_data);
        self.state.as_mut().unwrap().instanced_compute_dirty = true;
    }

    pub(super) fn build_culling_bind_groups(&mut self, device: &wgpu::Device) {
        let hiz_view = self.hiz.hiz_view_or_dummy();
        let mesh_bounds_buffer = &self.registry.bounds_buffer;
        let layout = &self.compute.cull_layout;
        let uniform_occlusion = &self.compute.cull_uniforms;
        let uniform_frustum = &self.compute.cull_uniforms_frustum;
        let world_state = self.state.as_mut().unwrap();
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
        configs: &nightshade_wgpu::backend::RenderInputs,
        queue: &wgpu::Queue,
    ) {
        if let Some(camera_matrices) = configs.scene.render_view.as_ref() {
            let view_proj = camera_matrices.projection * camera_matrices.view;
            let frustum_planes = camera_matrices.frustum_planes;

            let (screen_width, screen_height) = configs
                .scene
                .render_view
                .as_ref()
                .map(|view| view.screen_size)
                .or(configs.view.scene_target_size)
                .or(configs.view.viewport_size)
                .unwrap_or((1, 1));
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
                        && configs.scene.gpu_culling_enabled
                        && !camera_matrices.oblique_depth,
                ),
                frustum_enabled: u32::from(configs.scene.gpu_culling_enabled),
                pad_culling: [0; 3],
            };
            queue.write_buffer(
                &self.compute.cull_uniforms,
                0,
                bytemuck::cast_slice(&[culling_uniforms]),
            );

            let frustum_only_uniforms = CullingUniforms {
                occlusion_enabled: 0,
                ..culling_uniforms
            };
            queue.write_buffer(
                &self.compute.cull_uniforms_frustum,
                0,
                bytemuck::cast_slice(&[frustum_only_uniforms]),
            );
        }
    }

    pub(in super::super) fn prepare_pass_node(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &nightshade_wgpu::backend::RenderInputs,
    ) {
        self.frame.counter += 1;
        self.frame.dirty_consumed = true;
        self.compute.batching_enabled =
            configs.scene.gpu_batching_enabled && self.compute.supports_multi_draw_count;

        self.ensure_gpu_buffers(device);
        self.state.as_mut().unwrap().last_used_frame = self.frame.counter;

        if let Some(fd) = self.frame.dirty.as_ref()
            && fd.instanced_compute_dispatch_needed
        {
            self.state.as_mut().unwrap().instanced_compute_dirty = true;
        }

        self.sync_meshes_from_cache(device, queue, &configs.mesh_cache);

        let frame_initialized = self
            .frame
            .dirty
            .as_ref()
            .is_some_and(|fd| fd.frame_initialized);
        let has_any_changes = self.frame.dirty.as_ref().is_some_and(|fd| {
            !fd.transform_dirty.is_empty()
                || !fd.material_dirty.is_empty()
                || !fd.entities_added.is_empty()
                || !fd.entities_removed.is_empty()
                || fd.batches_invalidated
                || fd.full_rebuild_needed
                || fd.instanced_meshes_changed
        }) || self.frame.resize_rebuild_pending;

        if self.frame.resize_rebuild_pending {
            if let Some(fd) = self.frame.dirty.as_mut() {
                fd.full_rebuild_needed = true;
            }
            self.frame.resize_rebuild_pending = false;
        }

        if frame_initialized
            && self.state().cached_render_materials_generation
                != configs.scene.render_materials.generation
        {
            self.collect_and_upload_materials(configs, device, queue);
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

        self.sync_textures(&configs.texture_cache);

        if self.can_do_incremental_entity_update(configs) {
            self.incremental_update_entities(configs, device, queue);
            self.sync_dynamic_object_state(configs, queue);
            self.write_culling_uniforms(configs, queue);
            self.prepare_uniforms_and_lights(device, queue, configs);
            return;
        }

        if self.can_do_rebatch_only() {
            self.sync_dynamic_object_state(configs, queue);
            self.rebatch_cached_entities(configs, device, queue);
            self.write_culling_uniforms(configs, queue);
            self.prepare_uniforms_and_lights(device, queue, configs);
            return;
        }

        if self.can_do_material_reclass_only() {
            self.sync_dynamic_object_state(configs, queue);
            self.material_reclass_update(configs, device, queue);
            self.write_culling_uniforms(configs, queue);
            self.prepare_uniforms_and_lights(device, queue, configs);
            return;
        }

        if self.can_do_instanced_tail_update(configs) {
            self.sync_dynamic_object_state(configs, queue);
            self.instanced_tail_rebuild(configs, device, queue);
            self.write_culling_uniforms(configs, queue);
            self.prepare_uniforms_and_lights(device, queue, configs);
            return;
        }

        let current_vertex_utilization = self.compute_vertex_utilization();
        let current_index_utilization = self.compute_index_utilization();

        let vertex_drop = self.buffers.vertex_utilization - current_vertex_utilization;
        let index_drop = self.buffers.index_utilization - current_index_utilization;

        let significant_drop = vertex_drop > COMPACTION_UTILIZATION_DROP_THRESHOLD
            || index_drop > COMPACTION_UTILIZATION_DROP_THRESHOLD;
        let below_threshold = current_vertex_utilization < BUFFER_SHRINK_THRESHOLD
            || current_index_utilization < BUFFER_SHRINK_THRESHOLD;

        self.buffers.vertex_utilization = current_vertex_utilization;
        self.buffers.index_utilization = current_index_utilization;

        if significant_drop || below_threshold {
            self.check_and_compact_buffers(device, queue, &configs.mesh_cache);
        }

        let lighting = configs.scene.render_lighting.clone().unwrap_or_default();
        let mut lights_data = lighting.lights_data;
        let entity_to_lights_index = lighting.entity_to_lights_index;

        nightshade_wgpu_shading::backend::projection::resolve_cookie_layers(
            configs,
            &mut lights_data,
            &entity_to_lights_index,
            &self.materials.layer_map,
            &configs.texture_cache.registry,
        );

        let directional_light_direction = lighting.directional_light_direction;

        let cascade_view_projections = lighting.cascade_view_projections;
        let cascade_diameters = lighting.cascade_diameters;
        let cascade_split_distances = lighting.cascade_split_distances;
        let light_view_projection = lighting.light_view_projection;
        let shadow_bias = lighting.shadow_bias;
        let shadows_enabled = lighting.shadows_enabled;

        let cascade_atlas_offsets =
            nightshade_wgpu_shading::backend::projection::cascade_atlas_offsets(&cascade_diameters);

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

            let uniforms = SceneUniforms {
                view: camera_matrices.view.into(),
                projection: camera_matrices.projection.into(),
                camera_position: [
                    camera_matrices.camera_position.x,
                    camera_matrices.camera_position.y,
                    camera_matrices.camera_position.z,
                    1.0,
                ],
                num_lights: [lights_data.len() as u32, 0, 0, 0],
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
                cascade_count: nightshade_render::data::light_gpu::NUM_SHADOW_CASCADES as u32,
                directional_light_size: 1.0,
                cascade_view_projections,
                cascade_split_distances,
                cascade_atlas_offsets,
                cascade_atlas_scale: [0.5, 0.5, 0.0, 0.0],
                time,
                pad_pre_direction: [0; 2],
                pbr_debug_mode: configs.debug_draw.pbr_debug_mode.as_u32(),
                directional_light_direction,
                ibl_blend_factor: configs.settings.ibl_blend_factor,
                oit_z_scale,
                pad_pre_flat: [0.0; 2],
                flat_color: configs
                    .scene
                    .active_view
                    .flat_shading_color
                    .map(|c| [c.x, c.y, c.z, c.w])
                    .unwrap_or([0.0; 4]),
                prev_view_projection: configs.scene.prev_view_projection,
                cur_view_projection: configs.scene.view_projection,
                material_filtering: configs.debug_draw.material_filtering as u32,
                tail_to_block: [0.0; 47],
            };
            queue.write_buffer(
                &self.bindings.uniforms,
                0,
                bytemuck::cast_slice(&[uniforms]),
            );

            let overlay_uniforms = SceneUniforms {
                shadows_enabled: 0.0,
                ..uniforms
            };
            queue.write_buffer(
                &self.bindings.overlay_uniforms,
                0,
                bytemuck::cast_slice(&[overlay_uniforms]),
            );
        }

        let spotlight_result = collect_spotlight_shadows(configs);
        apply_spotlight_shadow_indices(
            &mut lights_data,
            &spotlight_result.entity_to_shadow_index,
            &entity_to_lights_index,
        );

        if !spotlight_result.shadow_data.is_empty() {
            queue.write_buffer(
                &self.bindings.spotlight_shadows,
                0,
                bytemuck::cast_slice(&spotlight_result.shadow_data),
            );
        }

        let point_shadow_result =
            collect_point_light_shadows(configs, &mut lights_data, &entity_to_lights_index);
        if !point_shadow_result.is_empty() {
            queue.write_buffer(
                &self.bindings.point_shadows,
                0,
                bytemuck::cast_slice(&point_shadow_result),
            );
        }

        if !lights_data.is_empty() {
            if lights_data.len() > self.gpu().lights_buffer_size {
                let new_size = (lights_data.len() as f32 * BUFFER_GROWTH_FACTOR).ceil() as usize;
                let new_size = new_size.min(MAX_LIGHTS);

                let new_buffer = build::buffer(
                    device,
                    "Mesh Lights Buffer (Resized)",
                    (std::mem::size_of::<LightData>() * new_size) as u64,
                    wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                );

                self.gpu_mut().lights_buffer = new_buffer;
                self.gpu_mut().lights_buffer_size = new_size;
                self.gpu_mut().cluster_assign_bind_group = None;
                self.rebuild_instance_bind_group(device);
            }

            queue.write_buffer(
                &self.gpu().lights_buffer,
                0,
                bytemuck::cast_slice(&lights_data),
            );
        }

        let mut materials = self.collect_and_upload_materials(configs, device, queue);

        let mut accum = BatchAccumulator {
            transforms: Vec::new(),
            custom_data: Vec::new(),
            objects: Vec::new(),
            entities: Vec::new(),
        };
        let mut combos: HashMap<(u32, u32, u32), u32> = HashMap::new();

        self.build_world_objects(configs, &mut materials, &mut accum, &mut combos);

        self.state.as_mut().unwrap().regular_object_count = accum.transforms.len() as u32;

        self.build_instanced_objects(
            configs,
            &mut accum.transforms,
            &mut accum.custom_data,
            &mut accum.objects,
            &mut combos,
        );

        let all_custom_data = accum.custom_data;

        let all_entities = accum.entities.clone();

        let prev_transforms = std::mem::take(&mut self.state.as_mut().unwrap().cached_transforms);
        let prev_objects = std::mem::take(&mut self.state.as_mut().unwrap().cached_objects);
        let prev_custom_data = std::mem::take(&mut self.state.as_mut().unwrap().cached_custom_data);

        self.state.as_mut().unwrap().object_count = accum.objects.len() as u32;
        self.state.as_mut().unwrap().cached_transforms = accum.transforms;
        self.state.as_mut().unwrap().cached_objects = accum.objects;
        self.state.as_mut().unwrap().cached_entities = all_entities.clone();

        self.state
            .as_mut()
            .unwrap()
            .entity_to_transform_index
            .clear();
        for (index, entity) in all_entities.iter().enumerate() {
            self.state
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

        self.state.as_mut().unwrap().cached_transparent_material_ids =
            materials.transparent_material_ids.clone();
        self.state
            .as_mut()
            .unwrap()
            .cached_double_sided_material_ids = materials.double_sided_material_ids.clone();
        self.state.as_mut().unwrap().cached_mask_material_ids = materials.mask_material_ids.clone();
        self.state
            .as_mut()
            .unwrap()
            .cached_transmissive_material_ids = materials.transmissive_material_ids.clone();
        self.state.as_mut().unwrap().combos = combos;
        self.build_lists_from_combos(device, queue);

        self.state.as_mut().unwrap().cached_material_ids = materials.material_ids;
        self.state.as_mut().unwrap().cached_material_map = collect_entity_materials(configs);
        self.state.as_mut().unwrap().cached_custom_data = all_custom_data;
        self.state.as_mut().unwrap().frames_since_full_rebuild = 0;

        self.populate_gpu_registry_from_instances();

        self.prepare_uniforms_and_lights(device, queue, configs);
    }
}
