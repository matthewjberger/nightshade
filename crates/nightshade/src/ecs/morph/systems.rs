use crate::ecs::generational_registry::registry_entry_by_name_mut;
use crate::ecs::prefab::resources::{mesh_cache_mark_dirty, mesh_cache_mark_dirty_skinned};
use crate::ecs::world::{MORPH_WEIGHTS, RENDER_MESH, World};
use freecs::Entity;

pub fn apply_morph_targets(world: &mut World) {
    let entities_with_morphs: Vec<Entity> =
        world.query_entities(MORPH_WEIGHTS | RENDER_MESH).collect();

    for entity in entities_with_morphs {
        let (needs_update, mesh_name) = {
            if let Some(morph_weights) = world.get_morph_weights(entity) {
                (
                    morph_weights.needs_update(),
                    morph_weights.mesh_name.clone(),
                )
            } else {
                continue;
            }
        };

        if !needs_update {
            continue;
        }

        let weights = {
            if let Some(morph_weights) = world.get_morph_weights(entity) {
                morph_weights.weights.clone()
            } else {
                continue;
            }
        };

        let result = apply_morph_to_mesh(
            &mut world.resources.mesh_cache.registry,
            &mesh_name,
            &weights,
        );

        if let Some(is_skinned) = result {
            if is_skinned {
                mesh_cache_mark_dirty_skinned(&world.resources.mesh_cache, mesh_name.clone());
            } else {
                mesh_cache_mark_dirty(&world.resources.mesh_cache, mesh_name.clone());
            }

            if let Some(morph_weights) = world.get_morph_weights_mut(entity) {
                morph_weights.applied_weights = weights;
            }
        }
    }
}

fn apply_morph_to_mesh(
    registry: &mut crate::ecs::generational_registry::GenerationalRegistry<
        crate::ecs::mesh::components::Mesh,
    >,
    mesh_name: &str,
    weights: &[f32],
) -> Option<bool> {
    let mesh = match registry_entry_by_name_mut(registry, mesh_name) {
        Some(m) => m,
        None => {
            return None;
        }
    };

    let is_skinned = mesh.skin_data.is_some();

    let morph_data = match &mesh.morph_targets {
        Some(data) => data,
        None => {
            return None;
        }
    };

    if morph_data.targets.is_empty() || morph_data.base_positions.is_empty() {
        return None;
    }

    let base_positions = &morph_data.base_positions;
    let base_normals = &morph_data.base_normals;
    let targets = &morph_data.targets;

    for (vertex_index, vertex) in mesh.vertices.iter_mut().enumerate() {
        let base_pos = match base_positions.get(vertex_index) {
            Some(p) => p,
            None => continue,
        };
        let base_normal = base_normals.get(vertex_index);

        let mut position_offset = [0.0f32; 3];
        let mut normal_offset = [0.0f32; 3];

        for (target_index, target) in targets.iter().enumerate() {
            let weight = weights.get(target_index).copied().unwrap_or(0.0);
            if weight.abs() < 0.0001 {
                continue;
            }

            if let Some(displacement) = target.position_displacements.get(vertex_index) {
                position_offset[0] += displacement[0] * weight;
                position_offset[1] += displacement[1] * weight;
                position_offset[2] += displacement[2] * weight;
            }

            if let Some(normal_displacements) = &target.normal_displacements
                && let Some(displacement) = normal_displacements.get(vertex_index)
            {
                normal_offset[0] += displacement[0] * weight;
                normal_offset[1] += displacement[1] * weight;
                normal_offset[2] += displacement[2] * weight;
            }
        }

        let final_position = [
            base_pos[0] + position_offset[0],
            base_pos[1] + position_offset[1],
            base_pos[2] + position_offset[2],
        ];

        vertex.position = final_position;

        let final_normal = if let Some(bn) = base_normal {
            let mut normal = [
                bn[0] + normal_offset[0],
                bn[1] + normal_offset[1],
                bn[2] + normal_offset[2],
            ];

            let len =
                (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
            if len > 0.0001 {
                normal[0] /= len;
                normal[1] /= len;
                normal[2] /= len;
            }
            vertex.normal = normal;
            Some(normal)
        } else {
            None
        };

        if let Some(skin_data) = &mut mesh.skin_data
            && let Some(skinned_vertex) = skin_data.skinned_vertices.get_mut(vertex_index)
        {
            skinned_vertex.position = final_position;
            if let Some(normal) = final_normal {
                skinned_vertex.normal = normal;
            }
        }
    }

    Some(is_skinned)
}
