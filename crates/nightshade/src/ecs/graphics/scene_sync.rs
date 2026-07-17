//! Persistent scene sync: keeps `renderer_state`'s scene collections up to
//! date from deltas instead of rebuilding them every frame. Structural
//! changes come from the world's structural log, data changes from the ECS
//! change ticks, and transform changes from the transform system's dirty
//! marks. Each domain collection persists across frames and is touched only
//! when something in its domain actually changed; a gap in the structural
//! log (the log overflowed) falls back to one full resync.

use crate::ecs::world::CORE;
use crate::ecs::world::World;
use crate::render_driver::{render_entity, scene_entity};
use std::collections::{HashMap, HashSet};

/// Cursors and gates for the persistent scene sync.
#[derive(Default)]
pub struct RenderSceneSync {
    pub initialized: bool,
    pub structural_cursor: u64,
    pub data_cursor: u32,
    pub materials_dirty: bool,
    pub materials_initialized: bool,
    pub materials_version_seen: u64,
    pub materials_names_version_seen: u64,
    pub animation_initialized: bool,
    pub animation_cursor: u32,
    pub animation_skinned_generation_seen: u64,
    pub animation_player_clips: HashMap<crate::ecs::world::Entity, (i64, i64)>,
    pub animation_player_clips_scratch: HashMap<crate::ecs::world::Entity, (i64, i64)>,
}

#[derive(Default)]
struct SceneDirt {
    full: bool,
    object_refresh: HashSet<crate::ecs::world::Entity>,
    instanced_refresh: HashSet<crate::ecs::world::Entity>,
    mesh_name_refresh: HashSet<crate::ecs::world::Entity>,
    bounds_refresh: HashSet<crate::ecs::world::Entity>,
    object_state_dirty: Vec<crate::ecs::world::Entity>,
    skinned_objects: bool,
    entity_lists: bool,
    lights: bool,
    decals: bool,
    emitters: bool,
    water: bool,
    cloth: bool,
    texts: bool,
}

const OBJECT_STRUCTURAL_MASK: u64 = crate::ecs::world::RENDER_MESH
    | crate::ecs::world::INSTANCED_MESH
    | crate::ecs::world::SKIN
    | crate::ecs::world::VISIBILITY
    | crate::ecs::world::MORPH_WEIGHTS
    | crate::ecs::world::RENDER_LAYER
    | crate::ecs::world::CULLING_MASK;

fn collect_scene_dirt(world: &mut World) -> SceneDirt {
    let mut dirt = SceneDirt::default();

    if !world
        .res::<crate::ecs::graphics::scene_sync::RenderSceneSync>()
        .initialized
    {
        dirt.full = true;
    }

    let structural_cursor = world
        .res::<crate::ecs::graphics::scene_sync::RenderSceneSync>()
        .structural_cursor;
    let changes: Vec<freecs::StructuralChange> = world.ecs.worlds[CORE]
        .structural_changes_since(structural_cursor)
        .to_vec();
    if let Some(first) = changes.first()
        && world
            .res::<crate::ecs::graphics::scene_sync::RenderSceneSync>()
            .initialized
        && first.sequence
            != world
                .res::<crate::ecs::graphics::scene_sync::RenderSceneSync>()
                .structural_cursor
                + 1
    {
        dirt.full = true;
    }
    if let Some(last) = changes.last() {
        world
            .res_mut::<crate::ecs::graphics::scene_sync::RenderSceneSync>()
            .structural_cursor = last.sequence;
        world.ecs.worlds[CORE].trim_structural_log(last.sequence);
    }

    for change in &changes {
        let mask = change.mask;
        let entity = change.entity;
        if mask & (crate::ecs::world::RENDER_MESH | crate::ecs::world::INSTANCED_MESH) != 0 {
            dirt.object_refresh.insert(entity);
            dirt.mesh_name_refresh.insert(entity);
            dirt.instanced_refresh.insert(entity);
            dirt.entity_lists = true;
            world
                .res_mut::<crate::ecs::graphics::scene_sync::RenderSceneSync>()
                .materials_dirty = true;
        }
        if mask & OBJECT_STRUCTURAL_MASK != 0 {
            dirt.object_refresh.insert(entity);
            dirt.instanced_refresh.insert(entity);
            if mask & (crate::ecs::world::SKIN | crate::ecs::world::CULLING_MASK) != 0 {
                dirt.entity_lists = true;
            }
            if mask & crate::ecs::world::SKIN != 0 {
                dirt.skinned_objects = true;
            }
        }
        if mask & crate::ecs::world::CASTS_SHADOW != 0 {
            dirt.entity_lists = true;
        }
        if mask & crate::ecs::world::GLOBAL_TRANSFORM != 0
            && let Some(entity_mask) = world.ecs.worlds[CORE].component_mask(entity)
        {
            if entity_mask & (crate::ecs::world::RENDER_MESH | crate::ecs::world::INSTANCED_MESH)
                != 0
            {
                dirt.object_refresh.insert(entity);
                dirt.instanced_refresh.insert(entity);
                dirt.entity_lists = true;
            }
            if entity_mask & crate::ecs::world::SKIN != 0 {
                dirt.entity_lists = true;
                dirt.skinned_objects = true;
            }
            dirt.lights |= entity_mask & crate::ecs::world::LIGHT != 0;
            dirt.decals |= entity_mask & crate::ecs::world::DECAL != 0;
            dirt.water |= entity_mask & crate::ecs::world::WATER != 0;
            dirt.cloth |= entity_mask & crate::ecs::world::CLOTH != 0;
            dirt.texts |= entity_mask & crate::ecs::world::TEXT != 0;
        }
        if mask & crate::ecs::world::BOUNDING_VOLUME != 0 {
            dirt.bounds_refresh.insert(entity);
        }
        if mask & crate::ecs::world::LIGHT != 0 {
            dirt.lights = true;
        }
        if mask & crate::ecs::world::DECAL != 0 {
            dirt.decals = true;
        }
        if mask & crate::ecs::world::PARTICLE_EMITTER != 0 {
            dirt.emitters = true;
        }
        if mask & crate::ecs::world::WATER != 0 {
            dirt.water = true;
        }
        if mask & crate::ecs::world::CLOTH != 0 {
            dirt.cloth = true;
        }
        if mask
            & (crate::ecs::world::TEXT
                | crate::ecs::world::TEXT_CHARACTER_COLORS
                | crate::ecs::world::VISIBILITY)
            != 0
        {
            dirt.texts = true;
        }
        if mask & crate::ecs::world::MATERIAL_REF != 0 {
            world
                .res_mut::<crate::ecs::graphics::scene_sync::RenderSceneSync>()
                .materials_dirty = true;
            world
                .res_mut::<crate::render::mesh_state::MeshRenderState>()
                .mark_material_dirty(render_entity(entity));
        }
    }

    let since = world
        .res::<crate::ecs::graphics::scene_sync::RenderSceneSync>()
        .data_cursor;
    let visibility_changed: Vec<crate::ecs::world::Entity> = world.ecs.worlds[CORE]
        .query_entities_changed_since(crate::ecs::world::VISIBILITY, since)
        .collect();
    let morph_changed: Vec<crate::ecs::world::Entity> = world.ecs.worlds[CORE]
        .query_entities_changed_since(crate::ecs::world::MORPH_WEIGHTS, since)
        .collect();
    let layer_changed: Vec<crate::ecs::world::Entity> = world.ecs.worlds[CORE]
        .query_entities_changed_since(crate::ecs::world::RENDER_LAYER, since)
        .collect();
    let culling_changed: Vec<crate::ecs::world::Entity> = world.ecs.worlds[CORE]
        .query_entities_changed_since(crate::ecs::world::CULLING_MASK, since)
        .collect();
    let mesh_changed: Vec<crate::ecs::world::Entity> = world.ecs.worlds[CORE]
        .query_entities_changed_since(crate::ecs::world::RENDER_MESH, since)
        .collect();
    let instanced_changed: Vec<crate::ecs::world::Entity> = world.ecs.worlds[CORE]
        .query_entities_changed_since(crate::ecs::world::INSTANCED_MESH, since)
        .collect();
    let bounds_changed: Vec<crate::ecs::world::Entity> = world.ecs.worlds[CORE]
        .query_entities_changed_since(crate::ecs::world::BOUNDING_VOLUME, since)
        .collect();
    dirt.lights |= world.ecs.worlds[CORE]
        .query_entities_changed_since(crate::ecs::world::LIGHT, since)
        .next()
        .is_some();
    dirt.decals |= world.ecs.worlds[CORE]
        .query_entities_changed_since(crate::ecs::world::DECAL, since)
        .next()
        .is_some();
    dirt.emitters |= world.ecs.worlds[CORE]
        .query_entities_changed_since(crate::ecs::world::PARTICLE_EMITTER, since)
        .next()
        .is_some();
    dirt.water |= world.ecs.worlds[CORE]
        .query_entities_changed_since(crate::ecs::world::WATER, since)
        .next()
        .is_some();
    dirt.cloth |= world.ecs.worlds[CORE]
        .query_entities_changed_since(crate::ecs::world::CLOTH, since)
        .next()
        .is_some();
    dirt.texts |= world.ecs.worlds[CORE]
        .query_entities_changed_since(crate::ecs::world::TEXT, since)
        .next()
        .is_some();
    dirt.texts |= world.ecs.worlds[CORE]
        .query_entities_changed_since(crate::ecs::world::TEXT_CHARACTER_COLORS, since)
        .next()
        .is_some();
    let material_ref_changed: Vec<crate::ecs::world::Entity> = world.ecs.worlds[CORE]
        .query_entities_changed_since(crate::ecs::world::MATERIAL_REF, since)
        .collect();
    if !material_ref_changed.is_empty() {
        world
            .res_mut::<crate::ecs::graphics::scene_sync::RenderSceneSync>()
            .materials_dirty = true;
        for entity in material_ref_changed {
            world
                .res_mut::<crate::render::mesh_state::MeshRenderState>()
                .mark_material_dirty(render_entity(entity));
        }
    }
    world
        .res_mut::<crate::ecs::graphics::scene_sync::RenderSceneSync>()
        .data_cursor = world.ecs.worlds[CORE].current_tick();
    world.ecs.worlds[CORE].increment_tick();

    for entity in visibility_changed {
        dirt.object_refresh.insert(entity);
        dirt.instanced_refresh.insert(entity);
        dirt.object_state_dirty.push(entity);
        let mask = world.ecs.worlds[CORE].component_mask(entity).unwrap_or(0);
        dirt.skinned_objects |= mask & crate::ecs::world::SKIN != 0;
        dirt.lights |= mask & crate::ecs::world::LIGHT != 0;
        dirt.decals |= mask & crate::ecs::world::DECAL != 0;
        dirt.water |= mask & crate::ecs::world::WATER != 0;
        dirt.texts |= mask & crate::ecs::world::TEXT != 0;
    }
    for entity in morph_changed {
        dirt.object_refresh.insert(entity);
        dirt.object_state_dirty.push(entity);
        dirt.skinned_objects |= world.ecs.worlds[CORE].component_mask(entity).unwrap_or(0)
            & crate::ecs::world::SKIN
            != 0;
    }
    for entity in layer_changed {
        dirt.object_refresh.insert(entity);
        dirt.instanced_refresh.insert(entity);
    }
    for entity in culling_changed {
        dirt.object_refresh.insert(entity);
    }
    for entity in mesh_changed {
        dirt.object_refresh.insert(entity);
        dirt.mesh_name_refresh.insert(entity);
        dirt.skinned_objects |= world.ecs.worlds[CORE].component_mask(entity).unwrap_or(0)
            & crate::ecs::world::SKIN
            != 0;
    }
    for entity in instanced_changed {
        dirt.instanced_refresh.insert(entity);
    }
    for entity in bounds_changed {
        dirt.bounds_refresh.insert(entity);
        dirt.skinned_objects |= world.ecs.worlds[CORE].component_mask(entity).unwrap_or(0)
            & crate::ecs::world::SKIN
            != 0;
    }

    for &entity in &world
        .res::<crate::render::mesh_state::MeshRenderState>()
        .inner
        .transform_dirty
    {
        let mask = world.ecs.worlds[CORE]
            .component_mask(scene_entity(entity))
            .unwrap_or(0);
        if mask & crate::ecs::world::RENDER_MESH != 0 {
            dirt.object_refresh.insert(scene_entity(entity));
        }
        if mask & crate::ecs::world::INSTANCED_MESH != 0 {
            dirt.instanced_refresh.insert(scene_entity(entity));
        }
        if mask & crate::ecs::world::SKIN != 0 {
            let previous_flipped = world
                .res::<crate::render::config::RendererState>()
                .render_dynamic_objects
                .get(&entity)
                .map(|state| {
                    nalgebra_glm::determinant(&nalgebra_glm::mat4_to_mat3(&state.transform)) < 0.0
                });
            let current_flipped = world
                .get::<crate::ecs::transform::components::GlobalTransform>(scene_entity(entity))
                .map(|global| {
                    nalgebra_glm::determinant(&nalgebra_glm::mat4_to_mat3(&global.0)) < 0.0
                });
            dirt.skinned_objects |= previous_flipped != current_flipped;
        }
        dirt.lights |= mask & crate::ecs::world::LIGHT != 0;
        dirt.decals |= mask & crate::ecs::world::DECAL != 0;
        dirt.water |= mask & crate::ecs::world::WATER != 0;
        dirt.cloth |= mask & crate::ecs::world::CLOTH != 0;
        dirt.texts |= mask & crate::ecs::world::TEXT != 0;
    }

    world
        .res_mut::<crate::ecs::graphics::scene_sync::RenderSceneSync>()
        .initialized = true;
    dirt
}

fn dynamic_object_state(
    world: &World,
    entity: crate::ecs::world::Entity,
) -> crate::render::config::DynamicObjectState {
    let visible = u32::from(
        world
            .get::<crate::ecs::primitives::Visibility>(entity)
            .is_none_or(|v| v.visible),
    );
    let mut morph_weights = [0.0f32; 8];
    if let Some(weights) = world.get::<crate::ecs::morph::components::MorphWeights>(entity) {
        for (index, weight) in weights.weights.iter().take(8).enumerate() {
            morph_weights[index] = *weight;
        }
    }
    let transform = world
        .get::<crate::ecs::transform::components::GlobalTransform>(entity)
        .map(|global| global.0)
        .unwrap_or_else(nalgebra_glm::Mat4::identity);
    let culling_mask = world
        .get::<crate::ecs::primitives::CullingMask>(entity)
        .copied()
        .unwrap_or_default()
        .0;
    let render_layer_value = world
        .get::<crate::render::render_layer::RenderLayer>(entity)
        .map(|layer| layer.0)
        .unwrap_or(crate::render::render_layer::RenderLayer::WORLD);
    let is_overlay =
        u32::from(render_layer_value == crate::render::render_layer::RenderLayer::OVERLAY);
    crate::render::config::DynamicObjectState {
        visible,
        morph_weights,
        transform,
        culling_mask,
        is_overlay,
        render_layer: render_layer_value,
    }
}

fn refill_instanced_object_data(
    world: &World,
    entity: crate::ecs::world::Entity,
    data: &mut crate::render::config::InstancedObjectData,
) -> bool {
    let Some(instanced_mesh) = world.get::<crate::ecs::mesh::components::InstancedMesh>(entity)
    else {
        return false;
    };
    data.world_models.clear();
    data.world_models.extend(
        instanced_mesh
            .cached_model_matrices()
            .iter()
            .map(|matrix| matrix.model),
    );
    data.world_normals.clear();
    data.world_normals.extend(
        instanced_mesh
            .cached_model_matrices()
            .iter()
            .map(|matrix| matrix.normal_matrix),
    );
    data.local_matrices.clear();
    data.local_matrices
        .extend_from_slice(instanced_mesh.cached_local_matrices());
    data.custom_tints.clear();
    data.custom_tints.extend(
        instanced_mesh
            .custom_data_slice()
            .iter()
            .map(|custom| custom.tint),
    );
    data.mesh_name.clear();
    data.mesh_name.push_str(&instanced_mesh.mesh_name);
    data.render_layer = world
        .get::<crate::render::render_layer::RenderLayer>(entity)
        .map(|layer| layer.0)
        .unwrap_or(crate::render::render_layer::RenderLayer::WORLD);
    data.visible = u32::from(
        world
            .get::<crate::ecs::primitives::Visibility>(entity)
            .is_none_or(|v| v.visible),
    );
    data.parent_transform = world
        .get::<crate::ecs::transform::components::GlobalTransform>(entity)
        .map(|global| global.0)
        .unwrap_or_else(nalgebra_glm::Mat4::identity);
    true
}

fn instanced_object_data(
    world: &World,
    entity: crate::ecs::world::Entity,
) -> Option<crate::render::config::InstancedObjectData> {
    world
        .get::<crate::ecs::mesh::components::InstancedMesh>(entity)
        .map(|instanced_mesh| {
            let world_models = instanced_mesh
                .cached_model_matrices()
                .iter()
                .map(|matrix| matrix.model)
                .collect();
            let world_normals = instanced_mesh
                .cached_model_matrices()
                .iter()
                .map(|matrix| matrix.normal_matrix)
                .collect();
            let local_matrices = instanced_mesh.cached_local_matrices().to_vec();
            let custom_tints = instanced_mesh
                .custom_data_slice()
                .iter()
                .map(|data| data.tint)
                .collect();
            let render_layer = world
                .get::<crate::render::render_layer::RenderLayer>(entity)
                .map(|layer| layer.0)
                .unwrap_or(crate::render::render_layer::RenderLayer::WORLD);
            let visible = u32::from(
                world
                    .get::<crate::ecs::primitives::Visibility>(entity)
                    .is_none_or(|v| v.visible),
            );
            let parent_transform = world
                .get::<crate::ecs::transform::components::GlobalTransform>(entity)
                .map(|global| global.0)
                .unwrap_or_else(nalgebra_glm::Mat4::identity);
            crate::render::config::InstancedObjectData {
                world_models,
                world_normals,
                local_matrices,
                custom_tints,
                mesh_name: instanced_mesh.mesh_name.clone(),
                render_layer,
                visible,
                parent_transform,
            }
        })
}

fn sync_objects_full(world: &mut World) {
    let entities: Vec<crate::ecs::world::Entity> = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::RENDER_MESH)
        .collect();
    let states: Vec<(
        crate::ecs::world::Entity,
        crate::render::config::DynamicObjectState,
    )> = entities
        .iter()
        .map(|&entity| (entity, dynamic_object_state(world, entity)))
        .collect();
    let mesh_names: Vec<(crate::ecs::world::Entity, String)> = entities
        .iter()
        .filter_map(|&entity| {
            world
                .get::<crate::ecs::mesh::components::RenderMesh>(entity)
                .map(|mesh| (entity, mesh.name.clone()))
        })
        .collect();
    let instanced: Vec<(
        crate::ecs::world::Entity,
        crate::render::config::InstancedObjectData,
    )> = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::INSTANCED_MESH)
        .collect::<Vec<_>>()
        .into_iter()
        .filter_map(|entity| instanced_object_data(world, entity).map(|data| (entity, data)))
        .collect();
    let bounds: Vec<(
        crate::ecs::world::Entity,
        crate::render::config::RenderBounds,
    )> = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::BOUNDING_VOLUME)
        .filter_map(|entity| {
            world
                .get::<crate::render::bounding_volume::BoundingVolume>(entity)
                .map(|volume| {
                    (
                        entity,
                        crate::render::config::RenderBounds {
                            center: volume.obb.center,
                            sphere_radius: volume.sphere_radius,
                        },
                    )
                })
        })
        .collect();

    let dynamic = &mut world
        .res_mut::<crate::render::config::RendererState>()
        .render_dynamic_objects;
    dynamic.clear();
    dynamic.extend(
        states
            .into_iter()
            .map(|(entity, state)| (render_entity(entity), state)),
    );
    let mesh_map = &mut world
        .res_mut::<crate::render::config::RendererState>()
        .render_object_meshes;
    mesh_map.clear();
    mesh_map.extend(
        mesh_names
            .into_iter()
            .map(|(entity, name)| (render_entity(entity), name)),
    );
    let instanced_map = &mut world
        .res_mut::<crate::render::config::RendererState>()
        .render_instanced_objects;
    instanced_map.clear();
    instanced_map.extend(
        instanced
            .into_iter()
            .map(|(entity, data)| (render_entity(entity), data)),
    );
    let render_bounds = &mut world
        .res_mut::<crate::render::config::RendererState>()
        .render_bounds;
    render_bounds.clear();
    render_bounds.extend(
        bounds
            .into_iter()
            .map(|(entity, entity_bounds)| (render_entity(entity), entity_bounds)),
    );
}

fn sync_objects_delta(world: &mut World, dirt: &SceneDirt) {
    for &entity in &dirt.object_refresh {
        if world.ecs.worlds[CORE].entity_has_components(entity, crate::ecs::world::RENDER_MESH) {
            let state = dynamic_object_state(world, entity);
            world
                .res_mut::<crate::render::config::RendererState>()
                .render_dynamic_objects
                .insert(render_entity(entity), state);
        } else {
            world
                .res_mut::<crate::render::config::RendererState>()
                .render_dynamic_objects
                .remove(&render_entity(entity));
        }
    }
    for &entity in &dirt.mesh_name_refresh {
        match world.get::<crate::ecs::mesh::components::RenderMesh>(entity) {
            Some(mesh) => {
                let name = mesh.name.clone();
                world
                    .res_mut::<crate::render::config::RendererState>()
                    .render_object_meshes
                    .insert(render_entity(entity), name);
            }
            None => {
                world
                    .res_mut::<crate::render::config::RendererState>()
                    .render_object_meshes
                    .remove(&render_entity(entity));
            }
        }
    }
    for &entity in &dirt.instanced_refresh {
        if let Some(existing) = world
            .res_mut::<crate::render::config::RendererState>()
            .render_instanced_objects
            .get_mut(&render_entity(entity))
        {
            let mut data = std::mem::take(existing);
            if refill_instanced_object_data(world, entity, &mut data) {
                world
                    .res_mut::<crate::render::config::RendererState>()
                    .render_instanced_objects
                    .insert(render_entity(entity), data);
            } else {
                world
                    .res_mut::<crate::render::config::RendererState>()
                    .render_instanced_objects
                    .remove(&render_entity(entity));
            }
        } else if let Some(data) = instanced_object_data(world, entity) {
            world
                .res_mut::<crate::render::config::RendererState>()
                .render_instanced_objects
                .insert(render_entity(entity), data);
        }
    }
    for &entity in &dirt.bounds_refresh {
        match world.get::<crate::render::bounding_volume::BoundingVolume>(entity) {
            Some(volume) => {
                let bounds = crate::render::config::RenderBounds {
                    center: volume.obb.center,
                    sphere_radius: volume.sphere_radius,
                };
                world
                    .res_mut::<crate::render::config::RendererState>()
                    .render_bounds
                    .insert(render_entity(entity), bounds);
            }
            None => {
                world
                    .res_mut::<crate::render::config::RendererState>()
                    .render_bounds
                    .remove(&render_entity(entity));
            }
        }
    }
    for &entity in &dirt.object_state_dirty {
        if world.ecs.worlds[CORE].entity_has_components(entity, crate::ecs::world::RENDER_MESH) {
            world
                .res_mut::<crate::render::mesh_state::MeshRenderState>()
                .mark_object_state_dirty(render_entity(entity));
        }
    }
}

fn sync_entity_lists(world: &mut World) {
    let regular_mesh_entities: Vec<crate::ecs::world::Entity> = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::RENDER_MESH)
        .filter(|&entity| {
            !world.ecs.worlds[CORE].entity_has_components(entity, crate::ecs::world::SKIN)
        })
        .collect();
    let shadow_meshes: Vec<crate::ecs::world::Entity> = world.ecs.worlds[CORE]
        .query_entities(
            crate::ecs::world::RENDER_MESH
                | crate::ecs::world::GLOBAL_TRANSFORM
                | crate::ecs::world::CASTS_SHADOW,
        )
        .collect();
    let shadow_instanced: Vec<crate::ecs::world::Entity> = world.ecs.worlds[CORE]
        .query_entities(
            crate::ecs::world::INSTANCED_MESH
                | crate::ecs::world::GLOBAL_TRANSFORM
                | crate::ecs::world::CASTS_SHADOW,
        )
        .collect();
    let shadow_skinned: Vec<crate::ecs::world::Entity> = world.ecs.worlds[CORE]
        .query_entities(
            crate::ecs::world::SKIN
                | crate::ecs::world::RENDER_MESH
                | crate::ecs::world::GLOBAL_TRANSFORM
                | crate::ecs::world::CASTS_SHADOW,
        )
        .collect();
    let skinned_meshes: Vec<crate::ecs::world::Entity> = world.ecs.worlds[CORE]
        .query_entities(
            crate::ecs::world::SKIN
                | crate::ecs::world::RENDER_MESH
                | crate::ecs::world::GLOBAL_TRANSFORM,
        )
        .collect();

    world
        .res_mut::<crate::render::config::RendererState>()
        .render_regular_mesh_entities = regular_mesh_entities
        .into_iter()
        .map(render_entity)
        .collect();
    let shadow_casters = &mut world
        .res_mut::<crate::render::config::RendererState>()
        .render_shadow_casters;
    shadow_casters.meshes = shadow_meshes.into_iter().map(render_entity).collect();
    shadow_casters.instanced = shadow_instanced.into_iter().map(render_entity).collect();
    shadow_casters.skinned = shadow_skinned.into_iter().map(render_entity).collect();
    world
        .res_mut::<crate::render::config::RendererState>()
        .render_skinned_meshes = skinned_meshes.into_iter().map(render_entity).collect();
}

fn sync_lights(world: &mut World) {
    let light_entities: Vec<crate::ecs::world::Entity> = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::LIGHT | crate::ecs::world::GLOBAL_TRANSFORM)
        .collect();
    let mut lights: Vec<crate::render::config::RenderLight> =
        Vec::with_capacity(light_entities.len());
    let mut light_index: std::collections::HashMap<crate::ecs::world::Entity, usize> =
        std::collections::HashMap::new();
    for entity in light_entities {
        if let (Some(light), Some(transform)) = (
            world.get::<crate::ecs::light::components::Light>(entity),
            world.get::<crate::ecs::transform::components::GlobalTransform>(entity),
        ) {
            light_index.insert(entity, lights.len());
            lights.push(crate::render::config::RenderLight {
                entity: render_entity(entity),
                light: crate::render::config::RenderLightData {
                    light_type: match light.light_type {
                        crate::ecs::light::components::LightType::Directional => {
                            crate::render::config::RenderLightType::Directional
                        }
                        crate::ecs::light::components::LightType::Point => {
                            crate::render::config::RenderLightType::Point
                        }
                        crate::ecs::light::components::LightType::Spot => {
                            crate::render::config::RenderLightType::Spot
                        }
                        crate::ecs::light::components::LightType::Area => {
                            crate::render::config::RenderLightType::Area
                        }
                    },
                    color: light.color,
                    intensity: light.intensity,
                    range: light.range,
                    inner_cone_angle: light.inner_cone_angle,
                    outer_cone_angle: light.outer_cone_angle,
                    cast_shadows: light.cast_shadows,
                    shadow_bias: light.shadow_bias,
                    shadow_resolution: light.shadow_resolution,
                    shadow_distance: light.shadow_distance,
                    cookie_texture: light.cookie_texture.clone(),
                    area_shape: match light.area_shape {
                        crate::ecs::light::components::AreaLightShape::Rectangle => {
                            crate::render::config::RenderAreaLightShape::Rectangle
                        }
                        crate::ecs::light::components::AreaLightShape::Disk => {
                            crate::render::config::RenderAreaLightShape::Disk
                        }
                        crate::ecs::light::components::AreaLightShape::Sphere => {
                            crate::render::config::RenderAreaLightShape::Sphere
                        }
                        crate::ecs::light::components::AreaLightShape::Tube => {
                            crate::render::config::RenderAreaLightShape::Tube
                        }
                    },
                    area_width: light.area_width,
                    area_height: light.area_height,
                    area_radius: light.area_radius,
                    area_two_sided: light.area_two_sided,
                    area_emissive_texture: light.area_emissive_texture.clone(),
                    shadow_normal_bias: light.shadow_normal_bias,
                    shadow_softness: light.shadow_softness,
                },
                transform: transform.0,
            });
        }
    }
    let render_lights = &mut world
        .res_mut::<crate::render::config::RendererState>()
        .render_lights;
    render_lights.lights = lights;
    render_lights.entity_to_index = light_index
        .into_iter()
        .map(|(entity, index)| (render_entity(entity), index))
        .collect();
}

fn sync_decals(world: &mut World) {
    let decals: Vec<crate::render::config::RenderDecal> = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::DECAL | crate::ecs::GLOBAL_TRANSFORM)
        .filter_map(|entity| {
            let decal = world.get::<crate::ecs::decal::components::Decal>(entity)?;
            let transform =
                world.get::<crate::ecs::transform::components::GlobalTransform>(entity)?;
            let visible = world
                .get::<crate::ecs::primitives::Visibility>(entity)
                .is_none_or(|visibility| visibility.visible);
            Some(crate::render::config::RenderDecal {
                decal: crate::render::config::RenderDecalData {
                    texture: decal.texture.clone(),
                    emissive_texture: decal.emissive_texture.clone(),
                    emissive_strength: decal.emissive_strength,
                    color: decal.color,
                    size: decal.size,
                    depth: decal.depth,
                    normal_threshold: decal.normal_threshold,
                    fade_start: decal.fade_start,
                    fade_end: decal.fade_end,
                },
                transform: transform.0,
                visible,
            })
        })
        .collect();
    world
        .res_mut::<crate::render::config::RendererState>()
        .render_decals = decals;
}

fn sync_emitters(world: &mut World) {
    let emitters: Vec<crate::render::config::RenderParticleEmitter> = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::PARTICLE_EMITTER)
        .filter_map(|entity| {
            world
                .get::<crate::render::particles::ParticleEmitter>(entity)
                .map(|emitter| crate::render::config::RenderParticleEmitter {
                    entity: render_entity(entity),
                    emitter: emitter.clone(),
                })
        })
        .collect();
    world
        .res_mut::<crate::render::config::RendererState>()
        .render_particle_emitters = emitters;
}

fn sync_water(world: &mut World) {
    let mut water_bodies: Vec<crate::render::config::RenderWater> = Vec::new();
    for (_, (water, global_transform)) in world
        .query_ref::<(
            &crate::ecs::water::components::Water,
            &crate::ecs::transform::components::GlobalTransform,
        )>()
        .iter()
    {
        {
            if water.enabled {
                water_bodies.push(crate::render::config::RenderWater {
                    water: crate::render::config::RenderWaterData {
                        half_extents: water.half_extents,
                        tessellation: water.tessellation,
                        wave_amplitude: water.wave_amplitude,
                        wave_steepness: water.wave_steepness,
                        wave_length: water.wave_length,
                        wave_speed: water.wave_speed,
                        wave_direction_radians: water.wave_direction_radians,
                        shallow_color: water.shallow_color,
                        deep_color: water.deep_color,
                        depth_fade_distance: water.depth_fade_distance,
                        edge_foam_distance: water.edge_foam_distance,
                        foam_amount: water.foam_amount,
                        foam_color: water.foam_color,
                        roughness: water.roughness,
                        fresnel_power: water.fresnel_power,
                        reflection_strength: water.reflection_strength,
                        refraction_strength: water.refraction_strength,
                        specular_strength: water.specular_strength,
                    },
                    transform: global_transform.0,
                });
            }
        }
    }
    world
        .res_mut::<crate::render::config::RendererState>()
        .render_water = water_bodies;
}

/// Rebuilds the renderer's meshlet placements and hands over any asset it has
/// not been given yet. Rebuilt in full every frame rather than from the dirt
/// set: an instance is a transform and an id, and the assets behind them are
/// shared handles, so there is nothing here worth tracking deltas for.
#[cfg(feature = "meshlet")]
fn sync_meshlets(world: &mut World) {
    let mut instances: Vec<crate::render::config::MeshletInstance> = Vec::new();
    for (_, (meshlet_mesh, global_transform)) in world
        .query_ref::<(
            &crate::ecs::meshlet::components::MeshletMesh,
            &crate::ecs::transform::components::GlobalTransform,
        )>()
        .iter()
    {
        instances.push(crate::render::config::MeshletInstance {
            transform: global_transform.0,
            asset_id: meshlet_mesh.asset_id,
            root_bvh_node_index: 0,
        });
    }

    let mut assets = std::mem::take(
        &mut world
            .res_mut::<crate::render::config::RendererState>()
            .meshlets
            .assets,
    );
    for instance in &instances {
        if assets.contains_key(&instance.asset_id) {
            continue;
        }
        let Some(asset) = world
            .res::<crate::ecs::meshlet::resources::MeshletAssets>()
            .get(instance.asset_id)
            .cloned()
        else {
            continue;
        };
        assets.insert(instance.asset_id, asset);
    }

    world
        .res_mut::<crate::render::config::RendererState>()
        .meshlets = crate::render::config::MeshletInstances { instances, assets };
}

fn sync_cloth(world: &mut World) {
    let mut cloth_bodies: Vec<crate::render::config::RenderCloth> = Vec::new();
    for (entity, (cloth, global_transform, render_mesh)) in world
        .query_ref::<(
            &crate::ecs::cloth::components::Cloth,
            &crate::ecs::transform::components::GlobalTransform,
            &crate::ecs::mesh::components::RenderMesh,
        )>()
        .iter()
    {
        {
            cloth_bodies.push(crate::render::config::RenderCloth {
                entity: render_entity(entity),
                cloth: crate::render::config::RenderClothData {
                    columns: cloth.columns,
                    rows: cloth.rows,
                    width: cloth.width,
                    height: cloth.height,
                    pinning: match cloth.pinning {
                        crate::ecs::cloth::components::ClothPinning::TopRow => {
                            crate::render::config::RenderClothPinning::TopRow
                        }
                        crate::ecs::cloth::components::ClothPinning::TopCorners => {
                            crate::render::config::RenderClothPinning::TopCorners
                        }
                        crate::ecs::cloth::components::ClothPinning::None => {
                            crate::render::config::RenderClothPinning::None
                        }
                    },
                    stiffness: cloth.stiffness,
                    damping: cloth.damping,
                    substeps: cloth.substeps,
                    solver_iterations: cloth.solver_iterations,
                    gravity: cloth.gravity,
                    wind_response: cloth.wind_response,
                    ground_height: cloth.ground_height,
                    texture_tiling: cloth.texture_tiling,
                    reset_epoch: cloth.reset_epoch,
                },
                transform: global_transform.0,
                mesh_name: render_mesh.name.clone(),
            });
        }
    }
    world
        .res_mut::<crate::render::config::RendererState>()
        .render_cloth = cloth_bodies;
}

fn sync_texts(world: &mut World) {
    let text_entities: Vec<crate::ecs::world::Entity> = world.ecs.worlds[CORE]
        .query_entities(
            crate::ecs::world::TEXT
                | crate::ecs::world::VISIBILITY
                | crate::ecs::world::GLOBAL_TRANSFORM,
        )
        .collect();
    let texts: Vec<crate::render::config::RenderText> = text_entities
        .into_iter()
        .filter_map(|entity| {
            if !world
                .get::<crate::ecs::primitives::Visibility>(entity)
                .map(|visibility| visibility.visible)
                .unwrap_or(false)
            {
                return None;
            }
            let text = world.get::<crate::ecs::text::components::Text>(entity)?;
            text.cached_mesh.as_ref()?;
            let transform = world
                .get::<crate::ecs::transform::components::GlobalTransform>(entity)?
                .0;
            let char_count = world
                .res::<crate::ecs::text::resources::TextState>()
                .cache
                .get_text(text.text_index)
                .map(|content| {
                    content
                        .chars()
                        .filter(|character| *character != '\n')
                        .count()
                })
                .unwrap_or(0);
            let character_colors = world
                .get::<crate::ecs::text::components::TextCharacterColors>(entity)
                .map(|colors| colors.colors.clone());
            Some(crate::render::config::RenderText {
                mesh: text.cached_mesh.clone()?,
                color: text.properties.color,
                outline_color: text.properties.outline_color,
                outline_width: text.properties.outline_width,
                smoothing: text.properties.smoothing,
                billboard: text.billboard,
                transform,
                char_count,
                character_colors,
            })
        })
        .collect();
    world
        .res_mut::<crate::render::config::RendererState>()
        .render_texts = texts;
}

fn sync_selection_outline(world: &mut World) {
    let selection_ids: Vec<u32> = if world
        .res::<crate::render::config::RendererState>()
        .active_view
        .selection_outline_enabled
    {
        let mut seeds: Vec<crate::ecs::world::Entity> = Vec::new();
        if let Some(active) = world
            .res::<crate::ecs::graphics::selection::Selection>()
            .active_entity
        {
            seeds.push(active);
        }
        for entity in &world
            .res::<crate::ecs::graphics::selection::Selection>()
            .entities
        {
            if !seeds.contains(entity) {
                seeds.push(*entity);
            }
        }
        let mut ids: Vec<u32> = Vec::with_capacity(seeds.len() * 4);
        for seed in &seeds {
            ids.push(seed.id);
            for descendant in crate::ecs::transform::queries::query_descendants(world, *seed) {
                ids.push(descendant.id);
            }
        }
        ids
    } else {
        Vec::new()
    };
    world
        .res_mut::<crate::render::config::RendererState>()
        .render_selection_outline_ids = selection_ids;
    world
        .res_mut::<crate::render::config::RendererState>()
        .render_selection_outline_color = world
        .res_mut::<crate::ecs::graphics::selection::Selection>()
        .outline_color;
}

/// Keeps the scene collections in `renderer_state` current. On the first run
/// (and after a structural-log gap) every collection rebuilds; afterwards
/// only the domains something actually changed in are touched.
pub fn render_sync_system(world: &mut World) {
    let _span = tracing::info_span!("render_sync").entered();
    #[cfg(feature = "meshlet")]
    sync_meshlets(world);
    let dirt = collect_scene_dirt(world);

    if dirt.full {
        world
            .res_mut::<crate::ecs::graphics::scene_sync::RenderSceneSync>()
            .materials_dirty = true;
        world
            .res_mut::<crate::render::mesh_state::MeshRenderState>()
            .request_full_rebuild();
        world
            .res_mut::<crate::render::config::RendererState>()
            .skinned_objects_generation = world
            .res_mut::<crate::render::config::RendererState>()
            .skinned_objects_generation
            .wrapping_add(1);
        sync_objects_full(world);
        sync_entity_lists(world);
        sync_lights(world);
        sync_decals(world);
        sync_emitters(world);
        sync_water(world);
        sync_cloth(world);
        sync_texts(world);
        sync_selection_outline(world);
        return;
    }

    sync_objects_delta(world, &dirt);
    if dirt.skinned_objects {
        world
            .res_mut::<crate::render::config::RendererState>()
            .skinned_objects_generation = world
            .res_mut::<crate::render::config::RendererState>()
            .skinned_objects_generation
            .wrapping_add(1);
    }
    if dirt.entity_lists {
        sync_entity_lists(world);
    }
    if dirt.lights {
        sync_lights(world);
    }
    if dirt.decals {
        sync_decals(world);
    }
    if dirt.emitters {
        sync_emitters(world);
    }
    if dirt.water {
        sync_water(world);
    }
    if dirt.cloth {
        sync_cloth(world);
    }
    if dirt.texts {
        sync_texts(world);
    }
    sync_selection_outline(world);
}
