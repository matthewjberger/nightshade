use crate::ecs::mesh::components::RenderMesh;
use crate::ecs::skin::components::Skin;
use crate::ecs::world::CORE;
use crate::ecs::world::{RENDER_MESH, SKIN, World};
use freecs::Entity;
use nalgebra_glm::Mat4;
use std::collections::HashMap;

use crate::render::skinning::{GpuSkinData, SkinningCache};

/// Whether the live skinned entity set no longer matches the cache's recorded
/// set, requiring a static-data rebuild.
pub fn skinning_cache_needs_rebuild(cache: &SkinningCache, world: &World) -> bool {
    if !cache.static_data_uploaded {
        return true;
    }

    let mut count = 0usize;
    let expected = &cache.skinned_entity_ids;
    for entity in world.ecs.worlds[CORE].query_entities(SKIN | RENDER_MESH) {
        if count >= expected.len() || entity.id != expected[count] {
            return true;
        }
        count += 1;
    }
    count != expected.len()
}

/// Rebuilds the cache's static layout from the Skin components: deduplicated
/// skins, inverse bind matrices, and the dense bone-index assignment.
pub fn skinning_cache_rebuild_static_data(cache: &mut SkinningCache, world: &World) {
    cache.skinned_entity_ids.clear();
    cache.inverse_bind_matrices.clear();
    cache.skin_data.clear();
    cache.entity_skin_indices.clear();
    cache.joint_entities.clear();
    cache.total_joints = 0;

    let mut any = false;
    let mut bone_entity_to_index: HashMap<Entity, u32> = HashMap::new();
    let mut bone_index = 0u32;
    let joint_entities = &mut cache.joint_entities;
    let skinned_entity_ids = &mut cache.skinned_entity_ids;
    for (entity, skin) in world.query_ref::<&Skin>().with::<RenderMesh>().iter() {
        any = true;
        skinned_entity_ids.push(entity.id);
        for &joint_entity in &skin.joints {
            bone_entity_to_index.entry(joint_entity).or_insert_with(|| {
                let index = bone_index;
                joint_entities.push(joint_entity);
                bone_index += 1;
                index
            });
        }
    }

    if !any {
        cache.static_data_uploaded = true;
        return;
    }

    let mut skin_configs: HashMap<u64, u32> = HashMap::new();
    let mut unique_skin_index = 0u32;
    let mut current_ibm_offset = 0u32;
    let mut current_output_offset = 0u32;
    let skin_data = &mut cache.skin_data;
    let inverse_bind_matrices = &mut cache.inverse_bind_matrices;
    let entity_skin_indices = &mut cache.entity_skin_indices;
    for (entity, skin) in world.query_ref::<&Skin>().with::<RenderMesh>().iter() {
        {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            for joint in skin.joints.iter() {
                use std::hash::Hash;
                joint.id.hash(&mut hasher);
            }
            for matrix in skin.inverse_bind_matrices.iter() {
                for row in 0..4 {
                    for column in 0..4 {
                        use std::hash::Hash;
                        matrix[(row, column)].to_bits().hash(&mut hasher);
                    }
                }
            }
            use std::hash::Hasher;
            let skin_hash = hasher.finish();

            let skin_index = if let Some(&existing_index) = skin_configs.get(&skin_hash) {
                existing_index
            } else {
                let index = unique_skin_index;
                skin_configs.insert(skin_hash, index);

                let base_bone_index = if let Some(&first_joint) = skin.joints.first() {
                    bone_entity_to_index.get(&first_joint).copied().unwrap_or(0)
                } else {
                    0
                };

                let joint_count = skin.joints.len().min(256) as u32;

                let gpu_skin_data = GpuSkinData {
                    joint_count,
                    base_bone_index,
                    base_ibm_index: current_ibm_offset,
                    base_output_index: current_output_offset,
                };
                skin_data.push(gpu_skin_data);

                for (joint_index, &inverse_bind) in skin.inverse_bind_matrices.iter().enumerate() {
                    if joint_index < 256 {
                        inverse_bind_matrices.push(inverse_bind);
                    }
                }

                current_ibm_offset += joint_count;
                current_output_offset += joint_count;

                unique_skin_index += 1;
                index
            };

            entity_skin_indices.insert(entity, skin_index);
        }
    }

    cache.total_joints = current_output_offset;
    cache.static_data_uploaded = true;
}

/// Refreshes `bone_transforms` in place with the current world transform of
/// every cached joint entity, in the cache's dense bone order. Returns true
/// when any transform (or the palette length) changed, comparing while
/// writing so change detection costs no extra pass or allocation.
pub fn skinning_cache_collect_bone_transforms_into(
    cache: &SkinningCache,
    world: &World,
    bone_transforms: &mut Vec<Mat4>,
) -> bool {
    let mut changed = bone_transforms.len() != cache.joint_entities.len();
    bone_transforms.resize(cache.joint_entities.len(), Mat4::identity());

    for (index, &joint_entity) in cache.joint_entities.iter().enumerate() {
        let transform = world
            .get::<crate::ecs::transform::components::GlobalTransform>(joint_entity)
            .map(|joint_global_transform| joint_global_transform.0)
            .unwrap_or_else(Mat4::identity);
        if bone_transforms[index] != transform {
            bone_transforms[index] = transform;
            changed = true;
        }
    }

    changed
}

/// Maintains `renderer_state.render_skinning` so the skinned-mesh and shadow
/// passes read the bone palette without touching Skin components or bone
/// transforms. The static layout rebuilds only when the skinned set changes
/// (bumping `static_generation`); bone transforms refresh every frame.
pub fn render_sync_skinning_system(world: &mut World) {
    let mut cache = std::mem::take(&mut world.resources.renderer_state.render_skinning.cache);
    let mut generation = world
        .resources
        .renderer_state
        .render_skinning
        .static_generation;
    if skinning_cache_needs_rebuild(&cache, world) {
        skinning_cache_rebuild_static_data(&mut cache, world);
        generation = generation.wrapping_add(1);
    }
    let mut bone_transforms = std::mem::take(
        &mut world
            .resources
            .renderer_state
            .render_skinning
            .bone_transforms,
    );
    let bones_changed =
        skinning_cache_collect_bone_transforms_into(&cache, world, &mut bone_transforms);
    let skinning = &mut world.resources.renderer_state.render_skinning;
    skinning.cache = cache;
    skinning.bone_transforms = bone_transforms;
    skinning.static_generation = generation;
    if bones_changed {
        skinning.bone_transforms_generation = skinning.bone_transforms_generation.wrapping_add(1);
    }
}
