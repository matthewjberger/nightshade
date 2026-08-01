use nightshade::ecs::Storage;
use nightshade::ecs::system_param::ResMut;
use nightshade_render::commands::mesh_state::mesh_state_mark_material_dirty;

struct SceneSync<'a> {
    sync: &'a mut nightshade_render::resources::render_sync::RenderSceneSync,
    renderer_state: &'a mut nightshade_render::data::config::RendererState,
    mesh_state: &'a mut nightshade_render::resources::mesh_state::MeshRenderState,
    outline: &'a mut nightshade_render::resources::outline::Outline,
    matrices: &'a mut nightshade_render::components::mesh::InstancedMeshMatrices,
    hierarchy: &'a mut nightshade::ecs::HierarchyIndex,
    storage: &'a mut Storage,
}
use nightshade_render::components::render_world;
use std::collections::HashSet;

fn write_object_render(scene: &mut SceneSync<'_>, entity: nightshade::ecs::Entity) {
    let state = dynamic_object_state(scene.storage, entity);
    scene
        .storage
        .set(entity, render_world::Transform(state.transform));
    scene
        .storage
        .set(entity, render_world::Visible(state.visible != 0));
    scene
        .storage
        .set(entity, render_world::MorphWeights(state.morph_weights));
    scene
        .storage
        .set(entity, render_world::CullingMask(state.culling_mask));
    scene
        .storage
        .set(entity, render_world::Layer(state.render_layer));
    set_shadow_tag(scene, entity);
}

fn set_shadow_tag(scene: &mut SceneSync<'_>, entity: nightshade::ecs::Entity) {
    let casts = scene
        .storage
        .entity_has_components::<(nightshade_render::components::culling::CastsShadow,)>(entity)
        && !scene
            .storage
            .entity_has_components::<(nightshade_render::components::skin::Skin,)>(entity);
    let render = &mut scene.storage;
    if casts {
        if !render.has_tag_type::<render_world::ShadowCaster>(entity) {
            render.add_tag_type::<render_world::ShadowCaster>(entity);
        }
    } else {
        render.remove_tag_type::<render_world::ShadowCaster>(entity);
    }
}

fn clear_render_component<B: nightshade::ecs::Bundle>(scene: &mut SceneSync<'_>) {
    let render = &mut scene.storage;
    let entities: Vec<nightshade::ecs::Entity> = render.query_entities::<B>().collect();
    for entity in entities {
        render.remove_components::<B>(entity);
    }
}

fn clear_render_row(scene: &mut SceneSync<'_>, entity: nightshade::ecs::Entity) {
    let lit = scene
        .storage
        .entity_has_components::<(nightshade_render::contract::Light,)>(entity);
    let render = &mut scene.storage;
    render.remove_components::<RenderRow>(entity);
    if !lit {
        render.remove_components::<SharedRow>(entity);
    }
    render.remove_tag_type::<render_world::ShadowCaster>(entity);
}

#[derive(Default)]
struct SceneDirt {
    full: bool,
    object_refresh: HashSet<nightshade::ecs::Entity>,
    instanced_refresh: HashSet<nightshade::ecs::Entity>,
    mesh_name_refresh: HashSet<nightshade::ecs::Entity>,
    bounds_refresh: HashSet<nightshade::ecs::Entity>,
    object_state_dirty: Vec<nightshade::ecs::Entity>,
    skinned_objects: bool,
    entity_lists: bool,
}

type RenderRow = (
    render_world::MeshName,
    render_world::MaterialId,
    render_world::Visible,
    render_world::MorphWeights,
    render_world::CullingMask,
    render_world::Layer,
    nightshade_render::data::config::RenderBounds,
    nightshade_render::data::config::InstancedObjectData,
);

type SharedRow = (
    render_world::Transform,
    nightshade_render::components::light::RenderLightData,
);

type ObjectStructural = (
    nightshade_render::components::mesh::RenderMesh,
    nightshade_render::components::mesh::InstancedMesh,
    nightshade_render::components::skin::Skin,
    nightshade_scene::contract::primitives::Visibility,
    nightshade_render::components::morph::MorphWeights,
    nightshade_render::data::render_layer::RenderLayer,
    nightshade_render::components::culling::CullingMask,
    nightshade_render::components::culling::CastsShadow,
);

type Drawable = (
    nightshade_render::components::mesh::RenderMesh,
    nightshade_render::components::mesh::InstancedMesh,
);

fn collect_scene_dirt(scene: &mut SceneSync<'_>) -> SceneDirt {
    let mut dirt = SceneDirt::default();

    if !scene.sync.initialized {
        dirt.full = true;
    }

    let structural_cursor = scene.sync.structural_cursor;
    let changes: Vec<nightshade::ecs::StructuralChange> = scene
        .storage
        .structural_changes_since(structural_cursor)
        .to_vec();
    if let Some(first) = changes.first()
        && scene.sync.initialized
        && first.sequence != scene.sync.structural_cursor + 1
    {
        dirt.full = true;
    }
    if let Some(last) = changes.last() {
        scene.sync.structural_cursor = last.sequence;
        scene.storage.trim_structural_log(last.sequence);
    }

    for change in &changes {
        let entity = change.entity;
        let (
            touches_drawable,
            touches_structural,
            touches_skin,
            touches_culling,
            touches_shadow,
            touches_transform,
            touches_bounds,
            touches_material,
        ) = {
            let core = &scene.storage;
            (
                core.change_touches::<Drawable>(change),
                core.change_touches::<ObjectStructural>(change),
                core.change_touches::<(nightshade_render::components::skin::Skin,)>(change),
                core.change_touches::<(nightshade_render::components::culling::CullingMask,)>(
                    change,
                ),
                core.change_touches::<(nightshade_render::components::culling::CastsShadow,)>(
                    change,
                ),
                core.change_touches::<(nightshade_scene::contract::GlobalTransform,)>(change),
                core.change_touches::<(nightshade_render::data::bounding_volume::BoundingVolume,)>(
                    change,
                ),
                core.change_touches::<(nightshade_render::components::material::MaterialRef,)>(
                    change,
                ),
            )
        };
        if touches_drawable {
            dirt.object_refresh.insert(entity);
            dirt.mesh_name_refresh.insert(entity);
            dirt.instanced_refresh.insert(entity);
            dirt.entity_lists = true;
            scene.sync.materials_dirty = true;
        }
        if touches_structural {
            dirt.object_refresh.insert(entity);
            dirt.instanced_refresh.insert(entity);
            if touches_skin || touches_culling {
                dirt.entity_lists = true;
            }
            if touches_skin {
                dirt.skinned_objects = true;
            }
        }
        if touches_shadow {
            dirt.entity_lists = true;
        }
        if touches_transform {
            let core = &scene.storage;
            if core.entity_has_any::<Drawable>(entity) {
                dirt.object_refresh.insert(entity);
                dirt.instanced_refresh.insert(entity);
                dirt.entity_lists = true;
            }
            if core.entity_has_any::<(nightshade_render::components::skin::Skin,)>(entity) {
                dirt.entity_lists = true;
                dirt.skinned_objects = true;
            }
        }
        if touches_bounds {
            dirt.bounds_refresh.insert(entity);
        }
        if touches_material {
            scene.sync.materials_dirty = true;
            mesh_state_mark_material_dirty(scene.mesh_state, entity);
        }
    }

    let since = scene.sync.data_cursor;
    let visibility_changed: Vec<nightshade::ecs::Entity> = scene
        .storage
        .query_entities_changed_since::<(nightshade_scene::contract::primitives::Visibility,)>(
            since,
        )
        .collect();
    let morph_changed: Vec<nightshade::ecs::Entity> = scene
        .storage
        .query_entities_changed_since::<(nightshade_render::components::morph::MorphWeights,)>(
            since,
        )
        .collect();
    let layer_changed: Vec<nightshade::ecs::Entity> = scene
        .storage
        .query_entities_changed_since::<(nightshade_render::data::render_layer::RenderLayer,)>(
            since,
        )
        .collect();
    let culling_changed: Vec<nightshade::ecs::Entity> = scene
        .storage
        .query_entities_changed_since::<(nightshade_render::components::culling::CullingMask,)>(
            since,
        )
        .collect();
    let mesh_changed: Vec<nightshade::ecs::Entity> = scene
        .storage
        .query_entities_changed_since::<(nightshade_render::components::mesh::RenderMesh,)>(since)
        .collect();
    let instanced_changed: Vec<nightshade::ecs::Entity> = scene
        .storage
        .query_entities_changed_since::<(nightshade_render::components::mesh::InstancedMesh,)>(
            since,
        )
        .collect();
    let bounds_changed: Vec<nightshade::ecs::Entity> = scene
        .storage
        .query_entities_changed_since::<(nightshade_render::data::bounding_volume::BoundingVolume,)>(since)
        .collect();
    let material_ref_changed: Vec<nightshade::ecs::Entity> = scene
        .storage
        .query_entities_changed_since::<(nightshade_render::components::material::MaterialRef,)>(
            since,
        )
        .collect();
    if !material_ref_changed.is_empty() {
        scene.sync.materials_dirty = true;
        for entity in material_ref_changed {
            mesh_state_mark_material_dirty(scene.mesh_state, entity);
        }
    }
    scene.sync.data_cursor = scene.storage.current_tick();
    scene.storage.increment_tick();

    for entity in visibility_changed {
        dirt.object_refresh.insert(entity);
        dirt.instanced_refresh.insert(entity);
        dirt.object_state_dirty.push(entity);
        let core = &scene.storage;
        dirt.skinned_objects |=
            core.entity_has_any::<(nightshade_render::components::skin::Skin,)>(entity);
    }
    for entity in morph_changed {
        dirt.object_refresh.insert(entity);
        dirt.object_state_dirty.push(entity);
        dirt.skinned_objects |= scene
            .storage
            .entity_has_any::<(nightshade_render::components::skin::Skin,)>(entity);
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
        dirt.skinned_objects |= scene
            .storage
            .entity_has_any::<(nightshade_render::components::skin::Skin,)>(entity);
    }
    for entity in instanced_changed {
        dirt.instanced_refresh.insert(entity);
    }
    for entity in bounds_changed {
        dirt.bounds_refresh.insert(entity);
        dirt.skinned_objects |= scene
            .storage
            .entity_has_any::<(nightshade_render::components::skin::Skin,)>(entity);
    }

    for &entity in &scene.mesh_state.transform_dirty {
        let core = &scene.storage;
        if core.entity_has_any::<(nightshade_render::components::mesh::RenderMesh,)>(entity) {
            dirt.object_refresh.insert(entity);
        }
        if core.entity_has_any::<(nightshade_render::components::mesh::InstancedMesh,)>(entity) {
            dirt.instanced_refresh.insert(entity);
        }
        if core.entity_has_any::<(nightshade_render::components::skin::Skin,)>(entity) {
            let previous_flipped = scene
                .renderer_state
                .render_skinned_dynamic_state
                .get(&entity)
                .map(|state| {
                    nalgebra_glm::determinant(&nalgebra_glm::mat4_to_mat3(&state.transform)) < 0.0
                });
            let current_flipped = scene
                .storage
                .get::<nightshade_scene::contract::GlobalTransform>(entity)
                .map(|global| {
                    nalgebra_glm::determinant(&nalgebra_glm::mat4_to_mat3(&global.0)) < 0.0
                });
            dirt.skinned_objects |= previous_flipped != current_flipped;
        }
    }

    scene.sync.initialized = true;
    dirt
}

fn dynamic_object_state(
    storage: &nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
) -> nightshade_render::data::config::DynamicRenderState {
    let visible = u32::from(
        storage
            .get::<nightshade_scene::contract::primitives::Visibility>(entity)
            .is_none_or(|v| v.visible),
    );
    let mut morph_weights = [0.0f32; 8];
    if let Some(weights) = storage.get::<nightshade_render::components::morph::MorphWeights>(entity)
    {
        for (index, weight) in weights.weights.iter().take(8).enumerate() {
            morph_weights[index] = *weight;
        }
    }
    let transform = storage
        .get::<nightshade_scene::contract::GlobalTransform>(entity)
        .map(|global| global.0)
        .unwrap_or_else(nalgebra_glm::Mat4::identity);
    let culling_mask = storage
        .get::<nightshade_render::components::culling::CullingMask>(entity)
        .copied()
        .unwrap_or_default()
        .0;
    let render_layer_value = storage
        .get::<nightshade_render::data::render_layer::RenderLayer>(entity)
        .map(|layer| layer.0)
        .unwrap_or(nightshade_render::data::render_layer::RenderLayer::WORLD);
    let is_overlay = u32::from(
        render_layer_value == nightshade_render::data::render_layer::RenderLayer::OVERLAY,
    );
    nightshade_render::data::config::DynamicRenderState {
        visible,
        morph_weights,
        transform,
        culling_mask,
        is_overlay,
        render_layer: render_layer_value,
    }
}

fn refill_instanced_object_data(
    storage: &nightshade::ecs::Storage,
    matrices: &nightshade_render::components::mesh::InstancedMeshMatrices,
    entity: nightshade::ecs::Entity,
    data: &mut nightshade_render::data::config::InstancedObjectData,
) -> bool {
    let Some(instanced_mesh) =
        storage.get::<nightshade_render::components::mesh::InstancedMesh>(entity)
    else {
        return false;
    };
    let model_matrices = matrices.model_matrices(entity);
    data.world_models.clear();
    data.world_models
        .extend(model_matrices.iter().map(|matrix| matrix.model));
    data.world_normals.clear();
    data.world_normals
        .extend(model_matrices.iter().map(|matrix| matrix.normal_matrix));
    data.local_matrices.clear();
    data.local_matrices
        .extend_from_slice(matrices.local_matrices(entity));
    data.custom_tints.clear();
    data.custom_tints.extend(
        instanced_mesh
            .custom_data_slice()
            .iter()
            .map(|custom| custom.tint),
    );
    data.mesh_name.clear();
    data.mesh_name.push_str(&instanced_mesh.mesh_name);
    data.render_layer = storage
        .get::<nightshade_render::data::render_layer::RenderLayer>(entity)
        .map(|layer| layer.0)
        .unwrap_or(nightshade_render::data::render_layer::RenderLayer::WORLD);
    data.visible = u32::from(
        storage
            .get::<nightshade_scene::contract::primitives::Visibility>(entity)
            .is_none_or(|v| v.visible),
    );
    data.parent_transform = storage
        .get::<nightshade_scene::contract::GlobalTransform>(entity)
        .map(|global| global.0)
        .unwrap_or_else(nalgebra_glm::Mat4::identity);
    true
}

fn instanced_object_data(
    storage: &nightshade::ecs::Storage,
    matrices: &nightshade_render::components::mesh::InstancedMeshMatrices,
    entity: nightshade::ecs::Entity,
) -> Option<nightshade_render::data::config::InstancedObjectData> {
    let instanced_mesh =
        storage.get::<nightshade_render::components::mesh::InstancedMesh>(entity)?;
    let model_matrices = matrices.model_matrices(entity);
    let world_models = model_matrices.iter().map(|matrix| matrix.model).collect();
    let world_normals = model_matrices
        .iter()
        .map(|matrix| matrix.normal_matrix)
        .collect();
    let local_matrices = matrices.local_matrices(entity).to_vec();
    let custom_tints = instanced_mesh
        .custom_data_slice()
        .iter()
        .map(|data| data.tint)
        .collect();
    let render_layer = storage
        .get::<nightshade_render::data::render_layer::RenderLayer>(entity)
        .map(|layer| layer.0)
        .unwrap_or(nightshade_render::data::render_layer::RenderLayer::WORLD);
    let visible = u32::from(
        storage
            .get::<nightshade_scene::contract::primitives::Visibility>(entity)
            .is_none_or(|v| v.visible),
    );
    let parent_transform = storage
        .get::<nightshade_scene::contract::GlobalTransform>(entity)
        .map(|global| global.0)
        .unwrap_or_else(nalgebra_glm::Mat4::identity);
    Some(nightshade_render::data::config::InstancedObjectData {
        world_models,
        world_normals,
        local_matrices,
        custom_tints,
        mesh_name: instanced_mesh.mesh_name.clone(),
        render_layer,
        visible,
        parent_transform,
    })
}

fn sync_objects_full(scene: &mut SceneSync<'_>) {
    let entities: Vec<nightshade::ecs::Entity> = scene
        .storage
        .query_entities::<(nightshade_render::components::mesh::RenderMesh,)>()
        .filter(|&entity| {
            !scene
                .storage
                .entity_has_components::<(nightshade_render::components::skin::Skin,)>(entity)
        })
        .collect();
    let states: Vec<(
        nightshade::ecs::Entity,
        nightshade_render::data::config::DynamicRenderState,
    )> = entities
        .iter()
        .map(|&entity| (entity, dynamic_object_state(scene.storage, entity)))
        .collect();
    let mesh_names: Vec<(nightshade::ecs::Entity, String)> = entities
        .iter()
        .filter_map(|&entity| {
            scene
                .storage
                .get::<nightshade_render::components::mesh::RenderMesh>(entity)
                .map(|mesh| (entity, mesh.name.clone()))
        })
        .collect();
    let skinned_entities: Vec<nightshade::ecs::Entity> = scene
        .storage
        .query_entities::<(
            nightshade_render::components::mesh::RenderMesh,
            nightshade_render::components::skin::Skin,
        )>()
        .collect();
    let skinned_states: Vec<(
        nightshade::ecs::Entity,
        nightshade_render::data::config::DynamicRenderState,
        Option<String>,
    )> = skinned_entities
        .iter()
        .map(|&entity| {
            let name = scene
                .storage
                .get::<nightshade_render::components::mesh::RenderMesh>(entity)
                .map(|mesh| mesh.name.clone());
            (entity, dynamic_object_state(scene.storage, entity), name)
        })
        .collect();
    let instanced: Vec<(
        nightshade::ecs::Entity,
        nightshade_render::data::config::InstancedObjectData,
    )> = scene
        .storage
        .query_entities::<(nightshade_render::components::mesh::InstancedMesh,)>()
        .collect::<Vec<_>>()
        .into_iter()
        .filter_map(|entity| {
            instanced_object_data(scene.storage, scene.matrices, entity).map(|data| (entity, data))
        })
        .collect();
    let bounds: Vec<(
        nightshade::ecs::Entity,
        nightshade_render::data::config::RenderBounds,
    )> = scene
        .storage
        .query_entities::<(nightshade_render::data::bounding_volume::BoundingVolume,)>()
        .filter_map(|entity| {
            scene
                .storage
                .get::<nightshade_render::data::bounding_volume::BoundingVolume>(entity)
                .map(|volume| {
                    (
                        entity,
                        nightshade_render::data::config::RenderBounds {
                            center: volume.obb.center,
                            sphere_radius: volume.sphere_radius,
                        },
                    )
                })
        })
        .collect();

    for (entity, state) in states {
        scene
            .storage
            .set(entity, render_world::Transform(state.transform));
        scene
            .storage
            .set(entity, render_world::Visible(state.visible != 0));
        scene
            .storage
            .set(entity, render_world::MorphWeights(state.morph_weights));
        scene
            .storage
            .set(entity, render_world::CullingMask(state.culling_mask));
        scene
            .storage
            .set(entity, render_world::Layer(state.render_layer));
        set_shadow_tag(scene, entity);
    }
    for (entity, name) in mesh_names {
        let key = nightshade_render::data::generational_registry::registry_name_hash(&name);
        scene.storage.set(entity, render_world::MeshKey(key));
        scene.storage.set(entity, render_world::MeshName(name));
    }
    for (entity, data) in instanced {
        scene.storage.set(entity, data);
        set_shadow_tag(scene, entity);
    }
    {
        let state = &mut *scene.renderer_state;
        state.render_skinned_dynamic_state.clear();
        state.render_skinned_mesh_names.clear();
        for (entity, object_state, name) in skinned_states {
            state
                .render_skinned_dynamic_state
                .insert(entity, object_state);
            if let Some(name) = name {
                state.render_skinned_mesh_names.insert(entity, name);
            }
        }
    }
    clear_render_component::<(nightshade_render::data::config::RenderBounds,)>(scene);
    for (entity, bound) in bounds {
        scene.storage.set(entity, bound);
    }
}

fn sync_objects_delta(scene: &mut SceneSync<'_>, dirt: &SceneDirt) {
    for &entity in &dirt.object_refresh {
        let core = &scene.storage;
        let has_mesh = core
            .entity_has_components::<(nightshade_render::components::mesh::RenderMesh,)>(entity);
        let skinned =
            core.entity_has_components::<(nightshade_render::components::skin::Skin,)>(entity);
        let instanced = core
            .entity_has_components::<(nightshade_render::components::mesh::InstancedMesh,)>(entity);
        if has_mesh && !skinned {
            write_object_render(scene, entity);
        } else if has_mesh && skinned {
            let state = dynamic_object_state(scene.storage, entity);
            scene
                .renderer_state
                .render_skinned_dynamic_state
                .insert(entity, state);
        } else if !instanced {
            clear_render_row(scene, entity);
            scene
                .renderer_state
                .render_skinned_dynamic_state
                .remove(&entity);
        }
    }
    for &entity in &dirt.mesh_name_refresh {
        let skinned = scene
            .storage
            .entity_has_components::<(nightshade_render::components::skin::Skin,)>(entity);
        match scene
            .storage
            .get::<nightshade_render::components::mesh::RenderMesh>(entity)
        {
            Some(mesh) if !skinned => {
                let name = mesh.name.clone();
                let key = nightshade_render::data::generational_registry::registry_name_hash(&name);
                scene.storage.set(entity, render_world::MeshKey(key));
                scene.storage.set(entity, render_world::MeshName(name));
            }
            Some(mesh) => {
                let name = mesh.name.clone();
                scene
                    .renderer_state
                    .render_skinned_mesh_names
                    .insert(entity, name);
            }
            None => {
                scene.storage.remove::<render_world::MeshName>(entity);
                scene.storage.remove::<render_world::MeshKey>(entity);
                scene
                    .renderer_state
                    .render_skinned_mesh_names
                    .remove(&entity);
            }
        }
    }
    for &entity in &dirt.instanced_refresh {
        if let Some(mut data) = scene
            .storage
            .get_mut::<nightshade_render::data::config::InstancedObjectData>(entity)
            .map(std::mem::take)
        {
            if refill_instanced_object_data(scene.storage, scene.matrices, entity, &mut data) {
                scene.storage.set(entity, data);
                set_shadow_tag(scene, entity);
            } else {
                scene
                    .storage
                    .remove::<nightshade_render::data::config::InstancedObjectData>(entity);
            }
        } else if let Some(data) = instanced_object_data(scene.storage, scene.matrices, entity) {
            scene.storage.set(entity, data);
            set_shadow_tag(scene, entity);
        }
    }
    for &entity in &dirt.bounds_refresh {
        match scene
            .storage
            .get::<nightshade_render::data::bounding_volume::BoundingVolume>(entity)
        {
            Some(volume) => {
                let bounds = nightshade_render::data::config::RenderBounds {
                    center: volume.obb.center,
                    sphere_radius: volume.sphere_radius,
                };
                scene.storage.set(entity, bounds);
            }
            None => {
                scene
                    .storage
                    .remove::<nightshade_render::data::config::RenderBounds>(entity);
            }
        }
    }
    for &entity in &dirt.object_state_dirty {
        if scene
            .storage
            .entity_has_components::<(nightshade_render::components::mesh::RenderMesh,)>(entity)
        {
            scene.mesh_state.object_state_dirty.push(entity);
        }
    }
}

fn sync_entity_lists(scene: &mut SceneSync<'_>) {
    let shadow_skinned: Vec<nightshade::ecs::Entity> = scene
        .storage
        .query_entities::<(
            nightshade_render::components::skin::Skin,
            nightshade_render::components::mesh::RenderMesh,
            nightshade_scene::contract::GlobalTransform,
            nightshade_render::components::culling::CastsShadow,
        )>()
        .collect();
    let skinned_meshes: Vec<nightshade::ecs::Entity> = scene
        .storage
        .query_entities::<(
            nightshade_render::components::skin::Skin,
            nightshade_render::components::mesh::RenderMesh,
            nightshade_scene::contract::GlobalTransform,
        )>()
        .collect();

    scene.renderer_state.render_shadow_casters.skinned = shadow_skinned.into_iter().collect();
    scene.renderer_state.render_skinned_meshes = skinned_meshes.into_iter().collect();
}

fn sync_outline(scene: &mut SceneSync<'_>) {
    let outline_entities: Vec<nightshade::ecs::Entity> = {
        let mut seeds: Vec<nightshade::ecs::Entity> = Vec::new();
        if let Some(active) = scene.outline.active_entity {
            seeds.push(active);
        }
        for entity in &scene.outline.entities {
            if !seeds.contains(entity) {
                seeds.push(*entity);
            }
        }
        let mut entities: Vec<nightshade::ecs::Entity> = Vec::with_capacity(seeds.len() * 4);
        let hierarchy = &mut *scene.hierarchy;
        for seed in &seeds {
            entities.push(*seed);
            for descendant in nightshade_scene::queries::query_descendants(hierarchy, *seed) {
                entities.push(descendant);
            }
        }
        entities
    };
    let render = &mut scene.storage;
    let previous: Vec<nightshade::ecs::Entity> =
        render.query_tag_type::<render_world::Outlined>().collect();
    for entity in previous {
        render.remove_tag_type::<render_world::Outlined>(entity);
    }
    for entity in outline_entities {
        render.add_tag_type::<render_world::Outlined>(entity);
    }
    scene.renderer_state.outline_color = scene.outline.color;
}

pub fn render_sync_scene_system(
    mut sync: ResMut<nightshade_render::resources::render_sync::RenderSceneSync>,
    mut renderer_state: ResMut<nightshade_render::data::config::RendererState>,
    mut mesh_state: ResMut<nightshade_render::resources::mesh_state::MeshRenderState>,
    mut outline: ResMut<nightshade_render::resources::outline::Outline>,
    mut matrices: ResMut<nightshade_render::components::mesh::InstancedMeshMatrices>,
    mut hierarchy: ResMut<nightshade::ecs::HierarchyIndex>,
    storage: &mut Storage,
) {
    let scene = &mut SceneSync {
        sync: &mut sync,
        renderer_state: &mut renderer_state,
        mesh_state: &mut mesh_state,
        outline: &mut outline,
        matrices: &mut matrices,
        hierarchy: &mut hierarchy,
        storage,
    };
    let dirt = collect_scene_dirt(scene);

    if dirt.full {
        scene.sync.materials_dirty = true;
        scene.mesh_state.full_rebuild_needed = true;
        scene.renderer_state.skinned_generation =
            scene.renderer_state.skinned_generation.wrapping_add(1);
        sync_objects_full(scene);
        sync_entity_lists(scene);
        sync_outline(scene);
        return;
    }

    sync_objects_delta(scene, &dirt);
    if dirt.skinned_objects {
        scene.renderer_state.skinned_generation =
            scene.renderer_state.skinned_generation.wrapping_add(1);
    }
    if dirt.entity_lists {
        sync_entity_lists(scene);
    }
    sync_outline(scene);
}
