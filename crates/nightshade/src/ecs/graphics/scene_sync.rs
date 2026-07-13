//! Persistent scene sync: keeps `renderer_state`'s scene collections up to
//! date from deltas instead of rebuilding them every frame. Structural
//! changes come from the world's structural log, data changes from the ECS
//! change ticks, and transform changes from the transform system's dirty
//! marks. Each domain collection persists across frames and is touched only
//! when something in its domain actually changed; a gap in the structural
//! log (the log overflowed) falls back to one full resync.

use crate::ecs::world::World;
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

    if !world.resources.render_scene_sync.initialized {
        dirt.full = true;
    }

    let changes: Vec<freecs::StructuralChange> = world
        .core
        .structural_changes_since(world.resources.render_scene_sync.structural_cursor)
        .to_vec();
    if let Some(first) = changes.first()
        && world.resources.render_scene_sync.initialized
        && first.sequence != world.resources.render_scene_sync.structural_cursor + 1
    {
        dirt.full = true;
    }
    if let Some(last) = changes.last() {
        world.resources.render_scene_sync.structural_cursor = last.sequence;
        world.core.trim_structural_log(last.sequence);
    }

    for change in &changes {
        let mask = change.mask;
        let entity = change.entity;
        if mask & (crate::ecs::world::RENDER_MESH | crate::ecs::world::INSTANCED_MESH) != 0 {
            dirt.object_refresh.insert(entity);
            dirt.mesh_name_refresh.insert(entity);
            dirt.instanced_refresh.insert(entity);
            dirt.entity_lists = true;
            world.resources.render_scene_sync.materials_dirty = true;
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
            && let Some(entity_mask) = world.core.component_mask(entity)
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
            world.resources.render_scene_sync.materials_dirty = true;
            world
                .resources
                .mesh_render_state
                .mark_material_dirty(entity);
        }
    }

    let since = world.resources.render_scene_sync.data_cursor;
    let visibility_changed: Vec<crate::ecs::world::Entity> = world
        .core
        .query_entities_changed_since(crate::ecs::world::VISIBILITY, since)
        .collect();
    let morph_changed: Vec<crate::ecs::world::Entity> = world
        .core
        .query_entities_changed_since(crate::ecs::world::MORPH_WEIGHTS, since)
        .collect();
    let layer_changed: Vec<crate::ecs::world::Entity> = world
        .core
        .query_entities_changed_since(crate::ecs::world::RENDER_LAYER, since)
        .collect();
    let culling_changed: Vec<crate::ecs::world::Entity> = world
        .core
        .query_entities_changed_since(crate::ecs::world::CULLING_MASK, since)
        .collect();
    let mesh_changed: Vec<crate::ecs::world::Entity> = world
        .core
        .query_entities_changed_since(crate::ecs::world::RENDER_MESH, since)
        .collect();
    let instanced_changed: Vec<crate::ecs::world::Entity> = world
        .core
        .query_entities_changed_since(crate::ecs::world::INSTANCED_MESH, since)
        .collect();
    let bounds_changed: Vec<crate::ecs::world::Entity> = world
        .core
        .query_entities_changed_since(crate::ecs::world::BOUNDING_VOLUME, since)
        .collect();
    dirt.lights |= world
        .core
        .query_entities_changed_since(crate::ecs::world::LIGHT, since)
        .next()
        .is_some();
    dirt.decals |= world
        .core
        .query_entities_changed_since(crate::ecs::world::DECAL, since)
        .next()
        .is_some();
    dirt.emitters |= world
        .core
        .query_entities_changed_since(crate::ecs::world::PARTICLE_EMITTER, since)
        .next()
        .is_some();
    dirt.water |= world
        .core
        .query_entities_changed_since(crate::ecs::world::WATER, since)
        .next()
        .is_some();
    dirt.cloth |= world
        .core
        .query_entities_changed_since(crate::ecs::world::CLOTH, since)
        .next()
        .is_some();
    dirt.texts |= world
        .core
        .query_entities_changed_since(crate::ecs::world::TEXT, since)
        .next()
        .is_some();
    dirt.texts |= world
        .core
        .query_entities_changed_since(crate::ecs::world::TEXT_CHARACTER_COLORS, since)
        .next()
        .is_some();
    let material_ref_changed: Vec<crate::ecs::world::Entity> = world
        .core
        .query_entities_changed_since(crate::ecs::world::MATERIAL_REF, since)
        .collect();
    if !material_ref_changed.is_empty() {
        world.resources.render_scene_sync.materials_dirty = true;
        for entity in material_ref_changed {
            world
                .resources
                .mesh_render_state
                .mark_material_dirty(entity);
        }
    }
    world.resources.render_scene_sync.data_cursor = world.core.current_tick();
    world.core.increment_tick();

    for entity in visibility_changed {
        dirt.object_refresh.insert(entity);
        dirt.instanced_refresh.insert(entity);
        dirt.object_state_dirty.push(entity);
        let mask = world.core.component_mask(entity).unwrap_or(0);
        dirt.skinned_objects |= mask & crate::ecs::world::SKIN != 0;
        dirt.lights |= mask & crate::ecs::world::LIGHT != 0;
        dirt.decals |= mask & crate::ecs::world::DECAL != 0;
        dirt.water |= mask & crate::ecs::world::WATER != 0;
        dirt.texts |= mask & crate::ecs::world::TEXT != 0;
    }
    for entity in morph_changed {
        dirt.object_refresh.insert(entity);
        dirt.object_state_dirty.push(entity);
        dirt.skinned_objects |=
            world.core.component_mask(entity).unwrap_or(0) & crate::ecs::world::SKIN != 0;
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
        dirt.skinned_objects |=
            world.core.component_mask(entity).unwrap_or(0) & crate::ecs::world::SKIN != 0;
    }
    for entity in instanced_changed {
        dirt.instanced_refresh.insert(entity);
    }
    for entity in bounds_changed {
        dirt.bounds_refresh.insert(entity);
        dirt.skinned_objects |=
            world.core.component_mask(entity).unwrap_or(0) & crate::ecs::world::SKIN != 0;
    }

    for &entity in &world.resources.mesh_render_state.inner.transform_dirty {
        let mask = world.core.component_mask(entity).unwrap_or(0);
        if mask & crate::ecs::world::RENDER_MESH != 0 {
            dirt.object_refresh.insert(entity);
        }
        if mask & crate::ecs::world::INSTANCED_MESH != 0 {
            dirt.instanced_refresh.insert(entity);
        }
        if mask & crate::ecs::world::SKIN != 0 {
            let previous_flipped = world
                .resources
                .renderer_state
                .render_dynamic_objects
                .get(&entity)
                .map(|state| {
                    nalgebra_glm::determinant(&nalgebra_glm::mat4_to_mat3(&state.transform)) < 0.0
                });
            let current_flipped = world.core.get_global_transform(entity).map(|global| {
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

    world.resources.render_scene_sync.initialized = true;
    dirt
}

fn dynamic_object_state(
    world: &World,
    entity: crate::ecs::world::Entity,
) -> crate::render::config::DynamicObjectState {
    let visible = u32::from(world.core.get_visibility(entity).is_none_or(|v| v.visible));
    let mut morph_weights = [0.0f32; 8];
    if let Some(weights) = world.core.get_morph_weights(entity) {
        for (index, weight) in weights.weights.iter().take(8).enumerate() {
            morph_weights[index] = *weight;
        }
    }
    let transform = world
        .core
        .get_global_transform(entity)
        .map(|global| global.0)
        .unwrap_or_else(nalgebra_glm::Mat4::identity);
    let culling_mask = world
        .core
        .get_culling_mask(entity)
        .copied()
        .unwrap_or_default()
        .0;
    let render_layer_value = world
        .core
        .get_render_layer(entity)
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
    let Some(instanced_mesh) = world.core.get_instanced_mesh(entity) else {
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
        .core
        .get_render_layer(entity)
        .map(|layer| layer.0)
        .unwrap_or(crate::render::render_layer::RenderLayer::WORLD);
    data.visible = u32::from(world.core.get_visibility(entity).is_none_or(|v| v.visible));
    data.parent_transform = world
        .core
        .get_global_transform(entity)
        .map(|global| global.0)
        .unwrap_or_else(nalgebra_glm::Mat4::identity);
    true
}

fn instanced_object_data(
    world: &World,
    entity: crate::ecs::world::Entity,
) -> Option<crate::render::config::InstancedObjectData> {
    world.core.get_instanced_mesh(entity).map(|instanced_mesh| {
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
            .core
            .get_render_layer(entity)
            .map(|layer| layer.0)
            .unwrap_or(crate::render::render_layer::RenderLayer::WORLD);
        let visible = u32::from(world.core.get_visibility(entity).is_none_or(|v| v.visible));
        let parent_transform = world
            .core
            .get_global_transform(entity)
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
    let entities: Vec<crate::ecs::world::Entity> = world
        .core
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
                .core
                .get_render_mesh(entity)
                .map(|mesh| (entity, mesh.name.clone()))
        })
        .collect();
    let instanced: Vec<(
        crate::ecs::world::Entity,
        crate::render::config::InstancedObjectData,
    )> = world
        .core
        .query_entities(crate::ecs::world::INSTANCED_MESH)
        .collect::<Vec<_>>()
        .into_iter()
        .filter_map(|entity| instanced_object_data(world, entity).map(|data| (entity, data)))
        .collect();
    let bounds: Vec<(
        crate::ecs::world::Entity,
        crate::render::config::RenderBounds,
    )> = world
        .core
        .query_entities(crate::ecs::world::BOUNDING_VOLUME)
        .filter_map(|entity| {
            world.core.get_bounding_volume(entity).map(|volume| {
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

    let dynamic = &mut world.resources.renderer_state.render_dynamic_objects;
    dynamic.clear();
    dynamic.extend(states);
    let mesh_map = &mut world.resources.renderer_state.render_object_meshes;
    mesh_map.clear();
    mesh_map.extend(mesh_names);
    let instanced_map = &mut world.resources.renderer_state.render_instanced_objects;
    instanced_map.clear();
    instanced_map.extend(instanced);
    let render_bounds = &mut world.resources.renderer_state.render_bounds;
    render_bounds.clear();
    render_bounds.extend(bounds);
}

fn sync_objects_delta(world: &mut World, dirt: &SceneDirt) {
    for &entity in &dirt.object_refresh {
        if world
            .core
            .entity_has_components(entity, crate::ecs::world::RENDER_MESH)
        {
            let state = dynamic_object_state(world, entity);
            world
                .resources
                .renderer_state
                .render_dynamic_objects
                .insert(entity, state);
        } else {
            world
                .resources
                .renderer_state
                .render_dynamic_objects
                .remove(&entity);
        }
    }
    for &entity in &dirt.mesh_name_refresh {
        match world.core.get_render_mesh(entity) {
            Some(mesh) => {
                let name = mesh.name.clone();
                world
                    .resources
                    .renderer_state
                    .render_object_meshes
                    .insert(entity, name);
            }
            None => {
                world
                    .resources
                    .renderer_state
                    .render_object_meshes
                    .remove(&entity);
            }
        }
    }
    for &entity in &dirt.instanced_refresh {
        if let Some(existing) = world
            .resources
            .renderer_state
            .render_instanced_objects
            .get_mut(&entity)
        {
            let mut data = std::mem::take(existing);
            if refill_instanced_object_data(world, entity, &mut data) {
                world
                    .resources
                    .renderer_state
                    .render_instanced_objects
                    .insert(entity, data);
            } else {
                world
                    .resources
                    .renderer_state
                    .render_instanced_objects
                    .remove(&entity);
            }
        } else if let Some(data) = instanced_object_data(world, entity) {
            world
                .resources
                .renderer_state
                .render_instanced_objects
                .insert(entity, data);
        }
    }
    for &entity in &dirt.bounds_refresh {
        match world.core.get_bounding_volume(entity) {
            Some(volume) => {
                let bounds = crate::render::config::RenderBounds {
                    center: volume.obb.center,
                    sphere_radius: volume.sphere_radius,
                };
                world
                    .resources
                    .renderer_state
                    .render_bounds
                    .insert(entity, bounds);
            }
            None => {
                world.resources.renderer_state.render_bounds.remove(&entity);
            }
        }
    }
    for &entity in &dirt.object_state_dirty {
        if world
            .core
            .entity_has_components(entity, crate::ecs::world::RENDER_MESH)
        {
            world
                .resources
                .mesh_render_state
                .mark_object_state_dirty(entity);
        }
    }
}

fn sync_entity_lists(world: &mut World) {
    let regular_mesh_entities: Vec<crate::ecs::world::Entity> = world
        .core
        .query_entities(crate::ecs::world::RENDER_MESH)
        .filter(|&entity| {
            !world
                .core
                .entity_has_components(entity, crate::ecs::world::SKIN)
        })
        .collect();
    let shadow_meshes: Vec<crate::ecs::world::Entity> = world
        .core
        .query_entities(
            crate::ecs::world::RENDER_MESH
                | crate::ecs::world::GLOBAL_TRANSFORM
                | crate::ecs::world::CASTS_SHADOW,
        )
        .collect();
    let shadow_instanced: Vec<crate::ecs::world::Entity> = world
        .core
        .query_entities(
            crate::ecs::world::INSTANCED_MESH
                | crate::ecs::world::GLOBAL_TRANSFORM
                | crate::ecs::world::CASTS_SHADOW,
        )
        .collect();
    let shadow_skinned: Vec<crate::ecs::world::Entity> = world
        .core
        .query_entities(
            crate::ecs::world::SKIN
                | crate::ecs::world::RENDER_MESH
                | crate::ecs::world::GLOBAL_TRANSFORM
                | crate::ecs::world::CASTS_SHADOW,
        )
        .collect();
    let skinned_meshes: Vec<crate::ecs::world::Entity> = world
        .core
        .query_entities(
            crate::ecs::world::SKIN
                | crate::ecs::world::RENDER_MESH
                | crate::ecs::world::GLOBAL_TRANSFORM,
        )
        .collect();

    world.resources.renderer_state.render_regular_mesh_entities = regular_mesh_entities;
    let shadow_casters = &mut world.resources.renderer_state.render_shadow_casters;
    shadow_casters.meshes = shadow_meshes;
    shadow_casters.instanced = shadow_instanced;
    shadow_casters.skinned = shadow_skinned;
    world.resources.renderer_state.render_skinned_meshes = skinned_meshes;
}

fn sync_lights(world: &mut World) {
    let light_entities: Vec<crate::ecs::world::Entity> = world
        .core
        .query_entities(crate::ecs::world::LIGHT | crate::ecs::world::GLOBAL_TRANSFORM)
        .collect();
    let mut lights: Vec<crate::render::config::RenderLight> =
        Vec::with_capacity(light_entities.len());
    let mut light_index: std::collections::HashMap<crate::ecs::world::Entity, usize> =
        std::collections::HashMap::new();
    for entity in light_entities {
        if let (Some(light), Some(transform)) = (
            world.core.get_light(entity),
            world.core.get_global_transform(entity),
        ) {
            light_index.insert(entity, lights.len());
            lights.push(crate::render::config::RenderLight {
                entity,
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
    let render_lights = &mut world.resources.renderer_state.render_lights;
    render_lights.lights = lights;
    render_lights.entity_to_index = light_index;
}

fn sync_decals(world: &mut World) {
    let decals: Vec<crate::render::config::RenderDecal> = world
        .core
        .query_entities(crate::ecs::DECAL | crate::ecs::GLOBAL_TRANSFORM)
        .filter_map(|entity| {
            let decal = world.core.get_decal(entity)?;
            let transform = world.core.get_global_transform(entity)?;
            let visible = world
                .core
                .get_visibility(entity)
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
    world.resources.renderer_state.render_decals = decals;
}

fn sync_emitters(world: &mut World) {
    let emitters: Vec<crate::render::config::RenderParticleEmitter> = world
        .core
        .query_entities(crate::ecs::PARTICLE_EMITTER)
        .filter_map(|entity| {
            world.core.get_particle_emitter(entity).map(|emitter| {
                crate::render::config::RenderParticleEmitter {
                    entity,
                    emitter: emitter.clone(),
                }
            })
        })
        .collect();
    world.resources.renderer_state.render_particle_emitters = emitters;
}

fn sync_water(world: &mut World) {
    let mut water_bodies: Vec<crate::render::config::RenderWater> = Vec::new();
    world
        .core
        .query()
        .with(crate::ecs::world::WATER | crate::ecs::world::GLOBAL_TRANSFORM)
        .iter(|_, table, index| {
            let water = &table.water[index];
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
                    transform: table.global_transform[index].0,
                });
            }
        });
    world.resources.renderer_state.render_water = water_bodies;
}

fn sync_cloth(world: &mut World) {
    let mut cloth_bodies: Vec<crate::render::config::RenderCloth> = Vec::new();
    world
        .core
        .query()
        .with(
            crate::ecs::world::CLOTH
                | crate::ecs::world::GLOBAL_TRANSFORM
                | crate::ecs::world::RENDER_MESH,
        )
        .iter(|entity, table, index| {
            let cloth = &table.cloth[index];
            cloth_bodies.push(crate::render::config::RenderCloth {
                entity,
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
                transform: table.global_transform[index].0,
                mesh_name: table.render_mesh[index].name.clone(),
            });
        });
    world.resources.renderer_state.render_cloth = cloth_bodies;
}

fn sync_texts(world: &mut World) {
    let text_entities: Vec<crate::ecs::world::Entity> = world
        .core
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
                .core
                .get_visibility(entity)
                .map(|visibility| visibility.visible)
                .unwrap_or(false)
            {
                return None;
            }
            let text = world.core.get_text(entity)?;
            text.cached_mesh.as_ref()?;
            let transform = world.core.get_global_transform(entity)?.0;
            let char_count = world
                .resources
                .text
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
                .core
                .get_text_character_colors(entity)
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
    world.resources.renderer_state.render_texts = texts;
}

fn sync_selection_and_culling_camera(world: &mut World) {
    let selection_ids: Vec<u32> = if world
        .resources
        .renderer_state
        .active_view
        .selection_outline_enabled
    {
        let mut seeds: Vec<crate::ecs::world::Entity> = Vec::new();
        if let Some(primary) = world
            .resources
            .editor_selection
            .bounding_volume_selected_entity
        {
            seeds.push(primary);
        }
        for entity in &world.resources.editor_selection.selected_entities {
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
    world.resources.renderer_state.render_selection_outline_ids = selection_ids;

    let culling_camera_view_projection = world
        .resources
        .editor_selection
        .culling_camera_override
        .and_then(|camera| crate::ecs::camera::queries::query_camera_matrices(world, camera))
        .map(|matrices| matrices.projection * matrices.view);
    world
        .resources
        .renderer_state
        .culling_camera_view_projection = culling_camera_view_projection;
}

/// Keeps the scene collections in `renderer_state` current. On the first run
/// (and after a structural-log gap) every collection rebuilds; afterwards
/// only the domains something actually changed in are touched.
pub fn render_sync_system(world: &mut World) {
    let _span = tracing::info_span!("render_sync").entered();
    let dirt = collect_scene_dirt(world);

    if dirt.full {
        world.resources.render_scene_sync.materials_dirty = true;
        world.resources.mesh_render_state.request_full_rebuild();
        world.resources.renderer_state.skinned_objects_generation = world
            .resources
            .renderer_state
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
        sync_selection_and_culling_camera(world);
        return;
    }

    sync_objects_delta(world, &dirt);
    if dirt.skinned_objects {
        world.resources.renderer_state.skinned_objects_generation = world
            .resources
            .renderer_state
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
    sync_selection_and_culling_camera(world);
}
