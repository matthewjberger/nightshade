use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;
use std::collections::{HashMap, HashSet};

use super::super::types::{ModelMatrix, ObjectData, grown_buffer_size};
use super::super::world_state::WorldRenderState;
use super::MeshPass;
use crate::systems::geometry::mesh::types::MeshData;

struct ClassifiedEntity {
    entity: nightshade::ecs::Entity,
    mesh_id: u32,
    material_id: u32,
    is_transparent: bool,
    is_mask: bool,
    transform: nalgebra_glm::Mat4,
    custom_data: [f32; 4],
    pipeline_class: u32,
}

fn combo_add_one(
    combos: &mut HashMap<(u32, u32, u32), u32>,
    pipeline_class: u32,
    mesh_id: u32,
    material_id: u32,
) {
    *combos
        .entry((pipeline_class, mesh_id, material_id))
        .or_default() += 1;
}

fn combo_remove_one(
    combos: &mut HashMap<(u32, u32, u32), u32>,
    pipeline_class: u32,
    mesh_id: u32,
    material_id: u32,
) {
    let key = (pipeline_class, mesh_id, material_id);
    if let Some(count) = combos.get_mut(&key) {
        *count -= 1;
        if *count == 0 {
            combos.remove(&key);
        }
    }
}

pub(in super::super) fn debug_view_exempt(
    scene_world: &nightshade::ecs::World,
    entity: nightshade::ecs::Entity,
) -> bool {
    scene_world
        .get::<nightshade_render::components::mesh::DebugViewExempt>(entity)
        .is_some()
}

pub(in super::super) fn world_dynamic_object(
    scene_world: &nightshade::ecs::World,
    entity: nightshade::ecs::Entity,
) -> Option<nightshade_render::data::config::DynamicRenderState> {
    use nightshade_render::components::render_world::{
        CullingMask, Layer, MorphWeights, Transform, Visible,
    };
    let transform = scene_world.get::<Transform>(entity)?.0;
    let render_layer = scene_world
        .get::<Layer>(entity)
        .map(|layer| layer.0)
        .unwrap_or(nightshade_render::data::render_layer::RenderLayer::WORLD);
    Some(nightshade_render::data::config::DynamicRenderState {
        visible: scene_world
            .get::<Visible>(entity)
            .map(|visible| u32::from(visible.0))
            .unwrap_or(1),
        morph_weights: scene_world
            .get::<MorphWeights>(entity)
            .map(|weights| weights.0)
            .unwrap_or([0.0; 8]),
        transform,
        culling_mask: scene_world
            .get::<CullingMask>(entity)
            .map(|mask| mask.0)
            .unwrap_or(u32::MAX),
        is_overlay: u32::from(
            render_layer == nightshade_render::data::render_layer::RenderLayer::OVERLAY,
        ),
        render_layer,
    })
}

impl MeshPass {
    pub(in super::super) fn sync_dynamic_object_state(
        &mut self,
        configs: &nightshade_wgpu::backend::RenderInputs,
        queue: &wgpu::Queue,
    ) {
        let dirty = self
            .frame
            .dirty
            .as_mut()
            .map(|frame_dirty| std::mem::take(&mut frame_dirty.object_state_dirty))
            .unwrap_or_default();
        if dirty.is_empty() {
            return;
        }
        let Some(world_state) = self.state.as_mut() else {
            return;
        };
        let Some(gpu) = world_state.gpu_buffers.as_ref() else {
            return;
        };

        let object_size = std::mem::size_of::<ObjectData>() as u64;

        for entity in dirty {
            let Some(&slot) = world_state.gpu_registry.entity_to_slot.get(&entity) else {
                continue;
            };
            let slot = slot as usize;
            if (slot as u32) >= world_state.regular_object_count {
                continue;
            }
            if slot >= world_state.cached_objects.len() {
                continue;
            }

            let mut changed = false;

            let dynamic = world_dynamic_object(&configs.scene_world, entity).unwrap_or(
                nightshade_render::data::config::DynamicRenderState {
                    visible: 1,
                    morph_weights: [0.0; 8],
                    transform: nalgebra_glm::Mat4::identity(),
                    culling_mask: u32::MAX,
                    is_overlay: 0,
                    render_layer: nightshade_render::data::render_layer::RenderLayer::WORLD,
                },
            );

            if world_state.cached_objects[slot].visible != dynamic.visible {
                world_state.cached_objects[slot].visible = dynamic.visible;
                changed = true;
            }

            if world_state.cached_objects[slot].morph_weights != dynamic.morph_weights {
                world_state.cached_objects[slot].morph_weights = dynamic.morph_weights;
                changed = true;
            }

            if changed {
                queue.write_buffer(
                    &gpu.object_buffer,
                    slot as u64 * object_size,
                    bytemuck::cast_slice(&[world_state.cached_objects[slot]]),
                );
            }
        }
    }

    pub(in super::super) fn update_dirty_transforms(
        &mut self,
        configs: &nightshade_wgpu::backend::RenderInputs,
        queue: &wgpu::Queue,
    ) -> bool {
        if self.instanced_structural_change_pending(configs) {
            return false;
        }

        let dirty_entities = self
            .frame
            .dirty
            .as_mut()
            .map(|fd| std::mem::take(&mut fd.transform_dirty))
            .unwrap_or_default();
        let dirty_count = dirty_entities.len();
        let dirty_lookup: HashSet<nightshade::ecs::Entity> =
            dirty_entities.iter().copied().collect();

        let local_matrices_dirty = self
            .frame
            .dirty
            .as_mut()
            .map(|fd| std::mem::take(&mut fd.instanced_local_matrices_dirty))
            .unwrap_or_default();

        let world_state = self.state.as_mut().unwrap();

        if !local_matrices_dirty.is_empty() {
            for &entity in &local_matrices_dirty {
                if let Some(&(start, count)) = world_state.instanced_transform_ranges.get(&entity)
                    && let Some(instanced) =
                        configs
                            .scene_world
                            .get::<nightshade_render::data::config::InstancedObjectData>(entity)
                {
                    let local_matrices = &instanced.local_matrices;
                    let upload_count = local_matrices.len().min(count as usize);
                    if upload_count > 0 {
                        let raw: Vec<[[f32; 4]; 4]> = local_matrices[..upload_count]
                            .iter()
                            .map(|mat| (*mat).into())
                            .collect();
                        if let Some(gpu) = world_state.gpu_buffers.as_ref() {
                            let offset_bytes =
                                (start as u64) * std::mem::size_of::<[[f32; 4]; 4]>() as u64;
                            queue.write_buffer(
                                &gpu.instanced_local_matrix_buffer,
                                offset_bytes,
                                bytemuck::cast_slice(&raw),
                            );
                        }
                        world_state.instanced_compute_dirty = true;
                    }
                }
            }
        }

        let total = world_state.gpu_registry.len();

        if dirty_count > 0 && total > 0 {
            let threshold = (total as f32 * 0.25).ceil() as usize;

            if dirty_count > threshold {
                let regular_count = world_state.regular_object_count as usize;
                let slot_limit = regular_count.min(world_state.cached_transforms.len());
                let all_dirty = dirty_count >= regular_count;
                let dirty_set: Option<&HashSet<nightshade::ecs::Entity>> =
                    if all_dirty { None } else { Some(&dirty_lookup) };
                let mut any_mesh_dirty = false;
                for slot in 0..slot_limit {
                    if let Some(entity) = world_state
                        .gpu_registry
                        .slot_to_entity
                        .get(slot)
                        .copied()
                        .flatten()
                        && (all_dirty || dirty_set.is_some_and(|s| s.contains(&entity)))
                        && let Some(dynamic) = world_dynamic_object(&configs.scene_world, entity)
                    {
                        any_mesh_dirty = true;
                        world_state.cached_transforms[slot] = ModelMatrix {
                            model: dynamic.transform.into(),
                            normal_matrix: [[0.0; 4]; 3],
                        };
                    }
                }

                for (&entity, &(start, count)) in &world_state.instanced_transform_ranges {
                    if !dirty_lookup.contains(&entity) {
                        continue;
                    }
                    world_state.instanced_compute_dirty = true;
                    let Some(instanced) =
                        configs
                            .scene_world
                            .get::<nightshade_render::data::config::InstancedObjectData>(entity)
                    else {
                        continue;
                    };
                    let start_idx = start as usize;
                    let end_idx = start_idx + count as usize;
                    if end_idx <= world_state.cached_custom_data.len() {
                        for (offset, tint) in instanced.custom_tints.iter().enumerate() {
                            world_state.cached_custom_data[start_idx + offset] = *tint;
                        }
                    }
                }

                let gpu = world_state.gpu_buffers.as_ref().unwrap();
                let regular_count_for_upload = world_state.regular_object_count as usize;
                if any_mesh_dirty
                    && regular_count_for_upload > 0
                    && regular_count_for_upload <= world_state.cached_transforms.len()
                {
                    queue.write_buffer(
                        &gpu.transform_buffer,
                        0,
                        bytemuck::cast_slice(
                            &world_state.cached_transforms[..regular_count_for_upload],
                        ),
                    );
                }
                let used_count = world_state.object_count as usize;
                if used_count > 0 && used_count <= world_state.cached_custom_data.len() {
                    queue.write_buffer(
                        &gpu.custom_data_buffer,
                        0,
                        bytemuck::cast_slice(&world_state.cached_custom_data[..used_count]),
                    );
                }
            } else {
                let mut mesh_updates = std::mem::take(&mut self.frame.scratch_updates);
                mesh_updates.clear();
                let mut instanced_dirty: Vec<nightshade::ecs::Entity> = Vec::new();
                let world_state = self.state.as_mut().unwrap();
                for &entity in &dirty_entities {
                    if let Some(&slot) = world_state.gpu_registry.entity_to_slot.get(&entity)
                        && let Some(dynamic) = world_dynamic_object(&configs.scene_world, entity)
                    {
                        let model_matrix = ModelMatrix {
                            model: dynamic.transform.into(),
                            normal_matrix: [[0.0; 4]; 3],
                        };
                        if (slot as usize) < world_state.cached_transforms.len() {
                            world_state.cached_transforms[slot as usize] = model_matrix;
                        }
                        mesh_updates.push((slot, model_matrix));
                    } else if world_state.instanced_transform_ranges.contains_key(&entity) {
                        instanced_dirty.push(entity);
                    } else if configs
                        .scene_world
                        .has::<nightshade_render::data::config::InstancedObjectData>(entity)
                    {
                        self.frame.scratch_updates = mesh_updates;
                        return false;
                    }
                }

                let gpu = world_state.gpu_buffers.as_ref().unwrap();

                if !mesh_updates.is_empty() {
                    mesh_updates.sort_by_key(|(slot, _)| *slot);
                    let element_size = std::mem::size_of::<ModelMatrix>() as u64;
                    let mut range_start = mesh_updates[0].0;
                    let mut range_data: Vec<ModelMatrix> = vec![mesh_updates[0].1];

                    for &(slot, matrix) in mesh_updates.iter().skip(1) {
                        if slot == range_start + range_data.len() as u32 {
                            range_data.push(matrix);
                        } else {
                            queue.write_buffer(
                                &gpu.transform_buffer,
                                (range_start as u64) * element_size,
                                bytemuck::cast_slice(&range_data),
                            );
                            range_start = slot;
                            range_data.clear();
                            range_data.push(matrix);
                        }
                    }
                    queue.write_buffer(
                        &gpu.transform_buffer,
                        (range_start as u64) * element_size,
                        bytemuck::cast_slice(&range_data),
                    );
                }

                if !instanced_dirty.is_empty() {
                    world_state.instanced_compute_dirty = true;
                    for entity in instanced_dirty {
                        let &(start, count) =
                            world_state.instanced_transform_ranges.get(&entity).unwrap();
                        let Some(instanced) = configs
                            .scene_world
                            .get::<nightshade_render::data::config::InstancedObjectData>(
                            entity,
                        ) else {
                            continue;
                        };
                        let updated_custom: Vec<[f32; 4]> = instanced.custom_tints.clone();
                        let start_idx = start as usize;
                        let end_idx = start_idx + count as usize;
                        if end_idx <= world_state.cached_custom_data.len() {
                            world_state.cached_custom_data[start_idx..end_idx]
                                .copy_from_slice(&updated_custom);
                        }
                        let custom_element_size = std::mem::size_of::<[f32; 4]>() as u64;
                        queue.write_buffer(
                            &gpu.custom_data_buffer,
                            (start as u64) * custom_element_size,
                            bytemuck::cast_slice(&updated_custom),
                        );
                    }
                }
                self.frame.scratch_updates = mesh_updates;
            }
        } else if dirty_count > 0 {
            for &entity in &dirty_entities {
                if !world_state.instanced_transform_ranges.contains_key(&entity)
                    && configs
                        .scene_world
                        .has::<nightshade_render::data::config::InstancedObjectData>(entity)
                {
                    return false;
                }
            }

            world_state.instanced_compute_dirty = true;

            if let Some(gpu) = world_state.gpu_buffers.as_ref() {
                for &entity in &dirty_entities {
                    let Some(&(start, count)) = world_state.instanced_transform_ranges.get(&entity)
                    else {
                        continue;
                    };
                    let Some(instanced) =
                        configs
                            .scene_world
                            .get::<nightshade_render::data::config::InstancedObjectData>(entity)
                    else {
                        continue;
                    };
                    let updated_custom: Vec<[f32; 4]> = instanced.custom_tints.clone();
                    let start_idx = start as usize;
                    let end_idx = start_idx + count as usize;
                    if end_idx <= world_state.cached_custom_data.len() {
                        world_state.cached_custom_data[start_idx..end_idx]
                            .copy_from_slice(&updated_custom);
                    }
                    let custom_element_size = std::mem::size_of::<[f32; 4]>() as u64;
                    queue.write_buffer(
                        &gpu.custom_data_buffer,
                        (start as u64) * custom_element_size,
                        bytemuck::cast_slice(&updated_custom),
                    );
                }
            }
        }

        true
    }

    pub(in super::super) fn can_do_incremental_update(&self) -> bool {
        let Some(fd) = self.frame.dirty.as_ref() else {
            return false;
        };
        fd.frame_initialized
            && !fd.full_rebuild_needed
            && !fd.batches_invalidated
            && !fd.instanced_meshes_changed
            && fd.entities_added.is_empty()
            && fd.entities_removed.is_empty()
            && fd.material_dirty.is_empty()
            && !fd.transform_dirty.is_empty()
    }

    pub(in super::super) fn can_do_incremental_entity_update(
        &self,
        configs: &nightshade_wgpu::backend::RenderInputs,
    ) -> bool {
        let Some(fd) = self.frame.dirty.as_ref() else {
            return false;
        };

        if !fd.frame_initialized
            || fd.full_rebuild_needed
            || fd.instanced_meshes_changed
            || !fd.material_dirty.is_empty()
        {
            return false;
        }

        let has_adds = !fd.entities_added.is_empty();
        let has_removes = !fd.entities_removed.is_empty();

        if !has_adds && !has_removes {
            return false;
        }

        let Some(world_state) = self.state.as_ref() else {
            return false;
        };

        if world_state.gpu_buffers.is_none() {
            return false;
        }

        if has_adds {
            if world_state.cached_material_ids.is_empty() {
                return false;
            }

            for entity in &fd.entities_added {
                if configs.scene.render_skinned_meshes.contains(entity) {
                    continue;
                }
                let has_mesh = configs
                    .scene_world
                    .get::<nightshade_render::components::render_world::MeshKey>(*entity)
                    .is_some_and(|key| self.registry.ids.contains_key(&key.0));
                if !has_mesh {
                    return false;
                }
                let material_id = configs
                    .scene_world
                    .get::<nightshade_render::components::render_world::MaterialId>(*entity)
                    .map(|material| material.0)
                    .unwrap_or(0);
                if !world_state.cached_material_ids.contains(&material_id) {
                    return false;
                }
            }
        }

        true
    }

    pub(in super::super) fn can_do_material_reclass_only(&self) -> bool {
        let Some(fd) = self.frame.dirty.as_ref() else {
            return false;
        };

        if !fd.frame_initialized
            || fd.full_rebuild_needed
            || !fd.entities_added.is_empty()
            || !fd.entities_removed.is_empty()
            || !fd.transform_dirty.is_empty()
            || fd.material_dirty.is_empty()
        {
            return false;
        }

        let Some(world_state) = self.state.as_ref() else {
            return false;
        };

        if world_state.gpu_buffers.is_none()
            || world_state.cached_entities.is_empty()
            || world_state.cached_material_ids.is_empty()
        {
            return false;
        }

        fd.material_dirty
            .iter()
            .all(|entity| world_state.gpu_registry.entity_to_slot.contains_key(entity))
    }

    fn instanced_structural_change_pending(
        &self,
        configs: &nightshade_wgpu::backend::RenderInputs,
    ) -> bool {
        let Some(fd) = self.frame.dirty.as_ref() else {
            return false;
        };
        let Some(world_state) = self.state.as_ref() else {
            return false;
        };
        fd.transform_dirty.iter().any(|&entity| {
            if !configs
                .scene_world
                .has::<nightshade_render::data::config::InstancedObjectData>(entity)
            {
                return false;
            }
            let current = configs
                .scene_world
                .get::<nightshade_render::data::config::InstancedObjectData>(entity)
                .map(|instanced| instanced.world_models.len());
            let cached = world_state
                .instanced_transform_ranges
                .get(&entity)
                .map(|&(_, count)| count as usize);
            match (cached, current) {
                (Some(cached_count), Some(current_count)) => cached_count != current_count,
                (None, Some(_)) => true,
                _ => false,
            }
        })
    }

    pub(in super::super) fn can_do_instanced_tail_update(
        &self,
        configs: &nightshade_wgpu::backend::RenderInputs,
    ) -> bool {
        let Some(fd) = self.frame.dirty.as_ref() else {
            return false;
        };

        if !fd.frame_initialized
            || fd.full_rebuild_needed
            || !fd.material_dirty.is_empty()
            || !fd.entities_added.is_empty()
            || !fd.entities_removed.is_empty()
        {
            return false;
        }

        let Some(world_state) = self.state.as_ref() else {
            return false;
        };
        if world_state.gpu_buffers.is_none() {
            return false;
        }

        fd.instanced_meshes_changed || self.instanced_structural_change_pending(configs)
    }

    pub(in super::super) fn incremental_update_entities(
        &mut self,
        configs: &nightshade_wgpu::backend::RenderInputs,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let fd = self.frame.dirty.as_mut().unwrap();
        let entities_added = std::mem::take(&mut fd.entities_added);
        let entities_removed = std::mem::take(&mut fd.entities_removed);
        let dirty_transforms = std::mem::take(&mut fd.transform_dirty);

        let render_materials = &configs.scene.render_materials;

        let world_state = self.state.as_mut().unwrap();

        if !entities_removed.is_empty() {
            let gpu = world_state.gpu_buffers.as_ref().unwrap();
            let far_away_transform = ModelMatrix {
                model: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [1e10, 1e10, 1e10, 1.0],
                ],
                normal_matrix: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                ],
            };

            for &entity in &entities_removed {
                if let Some(slot) = world_state.gpu_registry.entity_to_slot.remove(&entity) {
                    if (slot as usize) < world_state.gpu_registry.slot_to_entity.len() {
                        world_state.gpu_registry.slot_to_entity[slot as usize] = None;
                    }
                    world_state.entity_to_transform_index.remove(&entity);
                    world_state.cached_material_map.remove(&entity);

                    if (slot as usize) < world_state.cached_objects.len() {
                        let mesh_id = world_state.cached_objects[slot as usize].mesh_id;
                        let material_id = world_state.cached_objects[slot as usize].material_id;
                        let pipeline_class =
                            world_state.cached_objects[slot as usize].pipeline_class;
                        combo_remove_one(
                            &mut world_state.combos,
                            pipeline_class,
                            mesh_id,
                            material_id,
                        );
                        world_state
                            .free_slots_by_group
                            .entry((mesh_id, material_id))
                            .or_default()
                            .push(slot);
                    }

                    let transform_offset =
                        (slot as u64) * std::mem::size_of::<ModelMatrix>() as u64;
                    queue.write_buffer(
                        &gpu.transform_buffer,
                        transform_offset,
                        bytemuck::cast_slice(&[far_away_transform]),
                    );

                    if (slot as usize) < world_state.cached_transforms.len() {
                        world_state.cached_transforms[slot as usize] = far_away_transform;
                    }

                    if (slot as usize) < world_state.cached_objects.len() {
                        world_state.cached_objects[slot as usize].batch_id = u32::MAX;
                        let object_offset =
                            (slot as u64) * std::mem::size_of::<ObjectData>() as u64;
                        queue.write_buffer(
                            &gpu.object_buffer,
                            object_offset,
                            bytemuck::cast_slice(&[world_state.cached_objects[slot as usize]]),
                        );
                    }

                    if (slot as usize) < world_state.cached_custom_data.len() {
                        world_state.cached_custom_data[slot as usize] = [0.0, 0.0, 0.0, 0.0];
                        let custom_data_offset =
                            (slot as u64) * std::mem::size_of::<[f32; 4]>() as u64;
                        queue.write_buffer(
                            &gpu.custom_data_buffer,
                            custom_data_offset,
                            bytemuck::cast_slice(&[[0.0f32, 0.0, 0.0, 0.0]]),
                        );
                    }
                }
            }
        }

        let current_object_count = world_state.object_count as usize;

        let mut classified = Vec::with_capacity(entities_added.len());

        for &entity in &entities_added {
            if configs.scene.render_skinned_meshes.contains(&entity) {
                continue;
            }
            let Some(mesh_key) = configs
                .scene_world
                .get::<nightshade_render::components::render_world::MeshKey>(entity)
                .copied()
            else {
                continue;
            };
            let Some(&mesh_id) = self.registry.ids.get(&mesh_key.0) else {
                continue;
            };

            let material_id = configs
                .scene_world
                .get::<nightshade_render::components::render_world::MaterialId>(entity)
                .map(|material| material.0)
                .unwrap_or(0);

            let Some(dynamic) = world_dynamic_object(&configs.scene_world, entity) else {
                continue;
            };
            let transform = dynamic.transform;

            let material_is_transparent = render_materials.transparent_ids.contains(&material_id);
            let material_is_transmissive = render_materials.transmissive_ids.contains(&material_id);
            let material_is_mask = render_materials.mask_ids.contains(&material_id);
            let custom_alpha = 1.0_f32;
            let is_transparent = material_is_transparent || custom_alpha < 1.0;
            let is_double_sided = render_materials.double_sided_ids.contains(&material_id)
                || nalgebra_glm::determinant(&nalgebra_glm::mat4_to_mat3(&transform)) < 0.0;
            let is_overlay = dynamic.is_overlay != 0;
            let pipeline_class = if is_overlay {
                if is_transparent {
                    5
                } else if is_double_sided {
                    4
                } else {
                    3
                }
            } else if material_is_transmissive {
                9
            } else if is_transparent {
                2
            } else if is_double_sided {
                1
            } else {
                0
            };

            classified.push(ClassifiedEntity {
                entity,
                mesh_id,
                material_id,
                is_transparent,
                is_mask: material_is_mask,
                transform,
                custom_data: [1.0, 1.0, 1.0, 1.0],
                pipeline_class,
            });

            world_state.cached_material_map.insert(entity, material_id);
        }

        classified.sort_by(|a, b| {
            a.is_transparent
                .cmp(&b.is_transparent)
                .then(a.mesh_id.cmp(&b.mesh_id))
                .then(a.material_id.cmp(&b.material_id))
        });

        let mut reused_indices: Vec<(usize, u32)> = Vec::new();
        let mut remaining_indices: Vec<usize> = Vec::new();

        for (index, info) in classified.iter().enumerate() {
            let group_key = (info.mesh_id, info.material_id);
            if let Some(free_slot) = world_state
                .free_slots_by_group
                .get_mut(&group_key)
                .and_then(|slots| slots.pop())
            {
                reused_indices.push((index, free_slot));
            } else {
                remaining_indices.push(index);
            }
        }

        write_reused_slot_rows(
            &reused_indices,
            &classified,
            &self.registry.data,
            world_state,
            configs,
            queue,
        );

        let mut generic_free_slots: Vec<u32> = Vec::new();
        for slots in world_state.free_slots_by_group.values_mut() {
            generic_free_slots.append(slots);
        }
        world_state.free_slots_by_group.clear();

        let mut generic_reused_indices: Vec<(usize, u32)> = Vec::new();
        let mut truly_appended_indices: Vec<usize> = Vec::new();
        for &index in &remaining_indices {
            if let Some(slot) = generic_free_slots.pop() {
                generic_reused_indices.push((index, slot));
            } else {
                truly_appended_indices.push(index);
            }
        }

        for slot in generic_free_slots {
            if (slot as usize) < world_state.cached_objects.len() {
                let obj = &world_state.cached_objects[slot as usize];
                world_state
                    .free_slots_by_group
                    .entry((obj.mesh_id, obj.material_id))
                    .or_default()
                    .push(slot);
            }
        }

        write_reused_slot_rows(
            &generic_reused_indices,
            &classified,
            &self.registry.data,
            world_state,
            configs,
            queue,
        );

        if !dirty_transforms.is_empty() {
            let gpu = world_state.gpu_buffers.as_ref().unwrap();
            let dirty_count = dirty_transforms.len();
            let dirty_lookup: HashSet<nightshade::ecs::Entity> =
                dirty_transforms.iter().copied().collect();
            let total = world_state.gpu_registry.len();
            let threshold = (total as f32 * 0.25).ceil() as usize;

            if dirty_count > threshold {
                let regular_count = world_state.regular_object_count as usize;
                let slot_limit = regular_count.min(world_state.cached_transforms.len());
                let all_dirty = dirty_count >= regular_count;
                let dirty_set: Option<&HashSet<nightshade::ecs::Entity>> =
                    if all_dirty { None } else { Some(&dirty_lookup) };
                let mut any_mesh_dirty = false;
                for slot in 0..slot_limit {
                    if let Some(entity) = world_state
                        .gpu_registry
                        .slot_to_entity
                        .get(slot)
                        .copied()
                        .flatten()
                        && (all_dirty || dirty_set.is_some_and(|s| s.contains(&entity)))
                        && let Some(dynamic) = world_dynamic_object(&configs.scene_world, entity)
                    {
                        any_mesh_dirty = true;
                        world_state.cached_transforms[slot] = ModelMatrix {
                            model: dynamic.transform.into(),
                            normal_matrix: [[0.0; 4]; 3],
                        };
                    }
                }

                let used_count = world_state.object_count as usize;
                if any_mesh_dirty
                    && used_count > 0
                    && used_count <= world_state.cached_transforms.len()
                {
                    queue.write_buffer(
                        &gpu.transform_buffer,
                        0,
                        bytemuck::cast_slice(&world_state.cached_transforms[..used_count]),
                    );
                }
            } else {
                let mut mesh_updates: Vec<(u32, ModelMatrix)> = Vec::new();
                for entity in dirty_transforms {
                    if let Some(&slot) = world_state.gpu_registry.entity_to_slot.get(&entity)
                        && let Some(dynamic) = world_dynamic_object(&configs.scene_world, entity)
                    {
                        let model_matrix = ModelMatrix {
                            model: dynamic.transform.into(),
                            normal_matrix: [[0.0; 4]; 3],
                        };
                        if (slot as usize) < world_state.cached_transforms.len() {
                            world_state.cached_transforms[slot as usize] = model_matrix;
                        }
                        mesh_updates.push((slot, model_matrix));
                    }
                }

                if !mesh_updates.is_empty() {
                    mesh_updates.sort_by_key(|(slot, _)| *slot);
                    let element_size = std::mem::size_of::<ModelMatrix>() as u64;
                    let mut range_start = mesh_updates[0].0;
                    let mut range_data: Vec<ModelMatrix> = vec![mesh_updates[0].1];

                    for &(slot, matrix) in mesh_updates.iter().skip(1) {
                        if slot == range_start + range_data.len() as u32 {
                            range_data.push(matrix);
                        } else {
                            queue.write_buffer(
                                &gpu.transform_buffer,
                                (range_start as u64) * element_size,
                                bytemuck::cast_slice(&range_data),
                            );
                            range_start = slot;
                            range_data.clear();
                            range_data.push(matrix);
                        }
                    }
                    queue.write_buffer(
                        &gpu.transform_buffer,
                        (range_start as u64) * element_size,
                        bytemuck::cast_slice(&range_data),
                    );
                }
            }
        }

        if !truly_appended_indices.is_empty() {
            let new_total = current_object_count + truly_appended_indices.len();
            let needs_grow = new_total
                > world_state
                    .gpu_buffers
                    .as_ref()
                    .unwrap()
                    .transform_buffer_size
                || new_total > world_state.gpu_buffers.as_ref().unwrap().object_buffer_size
                || new_total
                    > world_state
                        .gpu_buffers
                        .as_ref()
                        .unwrap()
                        .custom_data_buffer_size;

            if needs_grow {
                let new_size = grown_buffer_size(new_total);

                if new_size
                    > world_state
                        .gpu_buffers
                        .as_ref()
                        .unwrap()
                        .transform_buffer_size
                {
                    let new_transform = build::buffer(
                        device,
                        "Mesh Transform Buffer (Incremental Resize)",
                        (std::mem::size_of::<ModelMatrix>() * new_size) as u64,
                        wgpu::BufferUsages::STORAGE
                            | wgpu::BufferUsages::COPY_DST
                            | wgpu::BufferUsages::COPY_SRC,
                    );
                    if !world_state.cached_transforms.is_empty() {
                        queue.write_buffer(
                            &new_transform,
                            0,
                            bytemuck::cast_slice(&world_state.cached_transforms),
                        );
                    }

                    let new_custom = build::buffer(
                        device,
                        "Mesh Custom Data Buffer (Incremental Resize)",
                        (std::mem::size_of::<[f32; 4]>() * new_size) as u64,
                        wgpu::BufferUsages::STORAGE
                            | wgpu::BufferUsages::COPY_DST
                            | wgpu::BufferUsages::COPY_SRC,
                    );
                    if !world_state.cached_custom_data.is_empty() {
                        queue.write_buffer(
                            &new_custom,
                            0,
                            bytemuck::cast_slice(&world_state.cached_custom_data),
                        );
                    }

                    let gpu_mut = world_state.gpu_buffers.as_mut().unwrap();
                    gpu_mut.transform_buffer = new_transform;
                    gpu_mut.transform_buffer_size = new_size;
                    gpu_mut.prev_transform_buffer = build::storage(
                        device,
                        "Mesh Previous Transform Buffer (Incremental Resize)",
                        (std::mem::size_of::<ModelMatrix>() * new_size) as u64,
                    );
                    gpu_mut.custom_data_buffer = new_custom;
                    gpu_mut.custom_data_buffer_size = new_size;
                }

                if new_size > world_state.gpu_buffers.as_ref().unwrap().object_buffer_size {
                    let new_object = build::buffer(
                        device,
                        "Mesh Object Buffer (Incremental Resize)",
                        (std::mem::size_of::<ObjectData>() * new_size) as u64,
                        wgpu::BufferUsages::STORAGE
                            | wgpu::BufferUsages::COPY_DST
                            | wgpu::BufferUsages::COPY_SRC,
                    );
                    if !world_state.cached_objects.is_empty() {
                        queue.write_buffer(
                            &new_object,
                            0,
                            bytemuck::cast_slice(&world_state.cached_objects),
                        );
                    }
                    let gpu_mut = world_state.gpu_buffers.as_mut().unwrap();
                    gpu_mut.object_buffer = new_object;
                    gpu_mut.object_buffer_size = new_size;
                    gpu_mut.batch_assign_bind_group = None;
                }

                let visible_size = (world_state
                    .gpu_buffers
                    .as_ref()
                    .unwrap()
                    .visible_indices_buffer
                    .size()
                    / std::mem::size_of::<u32>() as u64)
                    as usize;
                if new_size > visible_size {
                    let gpu_mut = world_state.gpu_buffers.as_mut().unwrap();
                    gpu_mut.visible_indices_buffer = build::storage(
                        device,
                        "Visible Indices Buffer (Incremental Resize)",
                        (std::mem::size_of::<u32>() * new_size) as u64,
                    );
                }

                {
                    let gpu_mut = world_state.gpu_buffers.as_mut().unwrap();
                    gpu_mut.instance_bind_group = build::binding(
                        device,
                        "Mesh Instance Bind Group",
                        &self.bindings.instance_layout,
                        vec![
                            gpu_mut.transform_buffer.as_entire_binding(),
                            gpu_mut.materials_buffer.as_entire_binding(),
                            gpu_mut.object_buffer.as_entire_binding(),
                            gpu_mut.lights_buffer.as_entire_binding(),
                            gpu_mut.visible_indices_buffer.as_entire_binding(),
                            gpu_mut.custom_data_buffer.as_entire_binding(),
                            self.morph.buffer.as_entire_binding(),
                            gpu_mut.light_grid_buffer.as_entire_binding(),
                            gpu_mut.light_indices_buffer.as_entire_binding(),
                            gpu_mut.cluster_uniforms_buffer.as_entire_binding(),
                            gpu_mut.area_lights_buffer.as_entire_binding(),
                            gpu_mut.area_uniforms_buffer.as_entire_binding(),
                            gpu_mut.prev_transform_buffer.as_entire_binding(),
                        ],
                    );
                    gpu_mut.culling_bind_group = None;
                    gpu_mut.cluster_assign_bind_group = None;
                    gpu_mut.instanced_compute_bind_group = None;
                }
            }
        }

        let gpu = world_state.gpu_buffers.as_ref().unwrap();

        let batches_invalidated = self
            .frame
            .dirty
            .as_ref()
            .is_some_and(|fd| fd.batches_invalidated);
        let needs_rebatch = batches_invalidated
            || !reused_indices.is_empty()
            || !generic_reused_indices.is_empty()
            || !truly_appended_indices.is_empty();

        if !truly_appended_indices.is_empty() {
            let insert_at = world_state.regular_object_count as usize;
            let instanced_slot_count = current_object_count.saturating_sub(insert_at);
            let appended_count = truly_appended_indices.len();
            let new_total = current_object_count + appended_count;

            if new_total > world_state.cached_transforms.len() {
                world_state
                    .cached_transforms
                    .resize(new_total, bytemuck::Zeroable::zeroed());
                world_state
                    .cached_objects
                    .resize(new_total, bytemuck::Zeroable::zeroed());
                world_state
                    .cached_custom_data
                    .resize(new_total, [1.0, 1.0, 1.0, 1.0]);
                world_state.cached_entities.resize(
                    new_total,
                    nightshade::ecs::Entity {
                        id: 0,
                        generation: 0,
                    },
                );
            }
            if world_state.gpu_registry.slot_to_entity.len() < new_total {
                world_state
                    .gpu_registry
                    .slot_to_entity
                    .resize(new_total, None);
            }

            if instanced_slot_count > 0 {
                world_state.cached_transforms.copy_within(
                    insert_at..insert_at + instanced_slot_count,
                    insert_at + appended_count,
                );
                world_state.cached_objects.copy_within(
                    insert_at..insert_at + instanced_slot_count,
                    insert_at + appended_count,
                );
                world_state.cached_custom_data.copy_within(
                    insert_at..insert_at + instanced_slot_count,
                    insert_at + appended_count,
                );

                for slot_index in (insert_at + appended_count)
                    ..(insert_at + appended_count + instanced_slot_count)
                {
                    world_state.cached_objects[slot_index].transform_index = slot_index as u32;
                }

                let shift = appended_count as u32;
                for batch in &mut world_state.instanced_opaque_batches {
                    batch.2 += shift;
                    batch.3 += shift;
                }
                for batch in &mut world_state.instanced_opaque_double_sided_batches {
                    batch.2 += shift;
                    batch.3 += shift;
                }
                for batch in &mut world_state.instanced_transparent_batches {
                    batch.2 += shift;
                    batch.3 += shift;
                }
            }

            for (local_index, &classified_index) in truly_appended_indices.iter().enumerate() {
                let info = &classified[classified_index];
                combo_add_one(
                    &mut world_state.combos,
                    info.pipeline_class,
                    info.mesh_id,
                    info.material_id,
                );
                let slot = (insert_at + local_index) as u32;
                let mesh_data = &self.registry.data[info.mesh_id as usize];

                let model_matrix = ModelMatrix {
                    model: info.transform.into(),
                    normal_matrix: [[0.0; 4]; 3],
                };
                let object_data = ObjectData {
                    transform_index: slot,
                    mesh_id: info.mesh_id,
                    material_id: info.material_id,
                    batch_id: 0,
                    morph_weights: [0.0f32; 8],
                    morph_target_count: mesh_data.morph_target_count,
                    morph_displacement_offset: mesh_data.morph_displacement_offset,
                    mesh_vertex_offset: mesh_data.vertex_offset,
                    mesh_vertex_count: mesh_data.vertex_count,
                    entity_id: info.entity.id,
                    is_overlay: 0,
                    skip_occlusion: u32::from(info.is_transparent || info.is_mask),
                    flip_winding: u32::from(
                        nalgebra_glm::determinant(&nalgebra_glm::mat4_to_mat3(&info.transform))
                            < 0.0,
                    ),
                    culling_mask: world_dynamic_object(&configs.scene_world, info.entity)
                        .map(|dynamic| dynamic.culling_mask)
                        .unwrap_or_default(),
                    visible: world_dynamic_object(&configs.scene_world, info.entity)
                        .map(|dynamic| dynamic.visible)
                        .unwrap_or(1),
                    pipeline_class: info.pipeline_class,
                    debug_view_exempt: u32::from(debug_view_exempt(
                        &configs.scene_world,
                        info.entity,
                    )),
                };

                world_state.cached_transforms[slot as usize] = model_matrix;
                world_state.cached_objects[slot as usize] = object_data;
                world_state.cached_custom_data[slot as usize] = info.custom_data;
                world_state.cached_entities[slot as usize] = info.entity;

                world_state
                    .gpu_registry
                    .entity_to_slot
                    .insert(info.entity, slot);
                world_state.gpu_registry.slot_to_entity[slot as usize] = Some(info.entity);
                world_state.gpu_registry.slot_count =
                    world_state.gpu_registry.slot_count.max(slot + 1);
                world_state
                    .entity_to_transform_index
                    .insert(info.entity, slot);
                world_state
                    .cached_material_map
                    .insert(info.entity, info.material_id);
            }

            {
                let append_end = insert_at + appended_count;
                let transform_offset =
                    (insert_at as u64) * std::mem::size_of::<ModelMatrix>() as u64;
                queue.write_buffer(
                    &gpu.transform_buffer,
                    transform_offset,
                    bytemuck::cast_slice(&world_state.cached_transforms[insert_at..append_end]),
                );
                let object_offset = (insert_at as u64) * std::mem::size_of::<ObjectData>() as u64;
                queue.write_buffer(
                    &gpu.object_buffer,
                    object_offset,
                    bytemuck::cast_slice(&world_state.cached_objects[insert_at..append_end]),
                );
                let custom_offset = (insert_at as u64) * std::mem::size_of::<[f32; 4]>() as u64;
                queue.write_buffer(
                    &gpu.custom_data_buffer,
                    custom_offset,
                    bytemuck::cast_slice(&world_state.cached_custom_data[insert_at..append_end]),
                );
            }

            if instanced_slot_count > 0 {
                let shifted_start = insert_at + appended_count;
                let shifted_end = shifted_start + instanced_slot_count;
                let transform_offset =
                    (shifted_start as u64) * std::mem::size_of::<ModelMatrix>() as u64;
                queue.write_buffer(
                    &gpu.transform_buffer,
                    transform_offset,
                    bytemuck::cast_slice(
                        &world_state.cached_transforms[shifted_start..shifted_end],
                    ),
                );
                let custom_offset = (shifted_start as u64) * std::mem::size_of::<[f32; 4]>() as u64;
                queue.write_buffer(
                    &gpu.custom_data_buffer,
                    custom_offset,
                    bytemuck::cast_slice(
                        &world_state.cached_custom_data[shifted_start..shifted_end],
                    ),
                );
            }

            world_state.object_count += appended_count as u32;
            world_state.regular_object_count += appended_count as u32;
        }

        if needs_rebatch {
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

        self.state_mut().frames_since_full_rebuild += 1;
    }
}

fn write_reused_slot_rows(
    reused: &[(usize, u32)],
    classified: &[ClassifiedEntity],
    mesh_data: &[MeshData],
    world_state: &mut WorldRenderState,
    configs: &nightshade_wgpu::backend::RenderInputs,
    queue: &wgpu::Queue,
) {
    let gpu = world_state.gpu_buffers.as_ref().unwrap();
    for &(classified_index, slot) in reused {
        let info = &classified[classified_index];
        combo_add_one(
            &mut world_state.combos,
            info.pipeline_class,
            info.mesh_id,
            info.material_id,
        );
        let mesh_data = &mesh_data[info.mesh_id as usize];

        let model_matrix = ModelMatrix {
            model: info.transform.into(),
            normal_matrix: [[0.0; 4]; 3],
        };
        let object_data = ObjectData {
            transform_index: slot,
            mesh_id: info.mesh_id,
            material_id: info.material_id,
            batch_id: 0,
            morph_weights: world_dynamic_object(&configs.scene_world, info.entity)
                .map(|dynamic| dynamic.morph_weights)
                .unwrap_or([0.0; 8]),
            morph_target_count: mesh_data.morph_target_count,
            morph_displacement_offset: mesh_data.morph_displacement_offset,
            mesh_vertex_offset: mesh_data.vertex_offset,
            mesh_vertex_count: mesh_data.vertex_count,
            entity_id: info.entity.id,
            is_overlay: world_dynamic_object(&configs.scene_world, info.entity)
                .map(|dynamic| dynamic.is_overlay)
                .unwrap_or(0),
            skip_occlusion: u32::from(info.is_transparent || info.is_mask),
            flip_winding: u32::from(
                nalgebra_glm::determinant(&nalgebra_glm::mat4_to_mat3(&info.transform)) < 0.0,
            ),
            culling_mask: world_dynamic_object(&configs.scene_world, info.entity)
                .map(|dynamic| dynamic.culling_mask)
                .unwrap_or(u32::MAX),
            visible: world_dynamic_object(&configs.scene_world, info.entity)
                .map(|dynamic| dynamic.visible)
                .unwrap_or(1),
            pipeline_class: info.pipeline_class,
            debug_view_exempt: u32::from(debug_view_exempt(&configs.scene_world, info.entity)),
        };

        let transform_offset = (slot as u64) * std::mem::size_of::<ModelMatrix>() as u64;
        queue.write_buffer(
            &gpu.transform_buffer,
            transform_offset,
            bytemuck::cast_slice(&[model_matrix]),
        );
        let object_offset = (slot as u64) * std::mem::size_of::<ObjectData>() as u64;
        queue.write_buffer(
            &gpu.object_buffer,
            object_offset,
            bytemuck::cast_slice(&[object_data]),
        );
        let custom_offset = (slot as u64) * std::mem::size_of::<[f32; 4]>() as u64;
        queue.write_buffer(
            &gpu.custom_data_buffer,
            custom_offset,
            bytemuck::cast_slice(&[info.custom_data]),
        );

        world_state.cached_transforms[slot as usize] = model_matrix;
        world_state.cached_objects[slot as usize] = object_data;
        world_state.cached_custom_data[slot as usize] = info.custom_data;
        world_state.cached_entities[slot as usize] = info.entity;

        world_state
            .gpu_registry
            .entity_to_slot
            .insert(info.entity, slot);
        world_state.gpu_registry.slot_to_entity[slot as usize] = Some(info.entity);
        world_state
            .entity_to_transform_index
            .insert(info.entity, slot);
        world_state
            .cached_material_map
            .insert(info.entity, info.material_id);
    }
}

impl MeshPass {
    pub(crate) fn take_frame_dirty_state(
        &mut self,
        configs: &nightshade_wgpu::backend::RenderInputs,
    ) {
        let Some(frame_state) = configs.mesh_frame_state.clone() else {
            return;
        };
        let mut dirty_ids = std::collections::HashSet::new();
        let mut fresh = Vec::new();
        for name in &configs.dirty_meshes {
            match self.mesh_id_for_name(name) {
                Some(id) => {
                    dirty_ids.insert(id);
                }
                None => fresh.push(name.clone()),
            }
        }
        self.take_new_mesh_names(fresh);
        match self.frame.dirty.take() {
            Some(mut unconsumed) if !self.frame.dirty_consumed => {
                nightshade_render::commands::mesh_state::mesh_state_merge_newer(
                    &mut unconsumed,
                    frame_state,
                );
                self.frame.dirty = Some(unconsumed);
                self.frame.dirty_mesh_ids.extend(dirty_ids);
            }
            _ => {
                self.frame.dirty = Some(frame_state);
                self.frame.dirty_mesh_ids.extend(dirty_ids);
            }
        }
        self.frame.dirty_consumed = false;
    }
}
