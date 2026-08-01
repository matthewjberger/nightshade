use crate::data::ModelSkin;
use crate::resources::PrefabSpawn;
use nalgebra_glm::Vec3;
use nightshade::ecs::Entity;
use nightshade_animation::contract::{AnimationClip, AnimationPlayer};
use nightshade_render::commands::material_registry::{
    material_registry_find_or_insert, material_registry_find_or_insert_hashed,
};
use nightshade_render::commands::mesh_state::mesh_state_mark_entity_added;
use nightshade_render::commands::texture_cache::texture_cache_acquire;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::components::morph::MorphWeights;
use nightshade_render::components::skin::{Joint, Skin};
use nightshade_render::data::bounding_volume::BoundingVolume;
use nightshade_render::data::generational_registry::{
    registry_add_reference, registry_entry_by_name,
};
use nightshade_render::queries::material_registry::material_registry_name;
use nightshade_render::resources::texture_cache::TextureOwner;
use nightshade_scene::contract::{ChildOf, GlobalTransform};

fn install_material_variants(
    spawn: &mut PrefabSpawn<'_>,
    entity: Entity,
    node_name: &str,
    default_material_name: &str,
    variants: &[crate::contract::PrefabMaterialVariant],
) {
    if variants.is_empty() {
        return;
    }
    let variant_textures: Vec<String> = variants
        .iter()
        .flat_map(|variant| variant.material.texture_names().map(str::to_string))
        .collect();
    texture_cache_acquire(
        spawn.parts.textures,
        TextureOwner::EntityVariants(entity),
        variant_textures,
    );
    let mut mappings = Vec::with_capacity(variants.len());
    for (variant_index, variant) in variants.iter().enumerate() {
        let fallback_name = format!(
            "Prefab_{}_{}_variant_{}",
            node_name, entity.id, variant_index
        );
        let variant_material_id = material_registry_find_or_insert(
            spawn.parts.materials,
            fallback_name,
            variant.material.clone(),
        );
        registry_add_reference(
            &mut spawn.parts.materials.registry,
            variant_material_id.index,
        );
        let variant_material_name =
            material_registry_name(spawn.parts.materials, variant_material_id)
                .unwrap_or_default()
                .to_string();
        mappings.push(
            nightshade_render::components::material::MaterialVariantMapping {
                variant_names: variant.variant_names.clone(),
                material_name: variant_material_name,
            },
        );
    }
    spawn.storage.set(
        entity,
        nightshade_render::components::material::MaterialVariants {
            default_material_name: default_material_name.to_string(),
            mappings,
        },
    );
}
use std::collections::HashMap;
use std::sync::Arc;

use crate::contract::{Prefab, PrefabNode};

fn get_bounding_volume_from_mesh_name(
    spawn: &PrefabSpawn<'_>,
    mesh_name: &str,
) -> Option<BoundingVolume> {
    let mesh = registry_entry_by_name(&spawn.parts.meshes.registry, mesh_name)?;
    mesh.bounding_volume
}

pub fn spawn_prefab(spawn: &mut PrefabSpawn<'_>, prefab: &Prefab, offset: Vec3) -> Entity {
    let parent_entity = spawn.storage.spawn_entities::<(
        nightshade_scene::contract::primitives::Name,
        nightshade_scene::contract::LocalTransform,
        nightshade_scene::contract::GlobalTransform,
    )>(1)[0];

    spawn.storage.set(
        parent_entity,
        nightshade_scene::contract::primitives::Name(prefab.name.clone()),
    );
    spawn.storage.set(
        parent_entity,
        nightshade_scene::contract::LocalTransform {
            translation: offset,
            rotation: nalgebra_glm::Quat::identity(),
            scale: nalgebra_glm::vec3(1.0, 1.0, 1.0),
        },
    );
    spawn.storage.set(parent_entity, GlobalTransform::default());

    for root_node in &prefab.root_nodes {
        spawn_prefab_node(spawn, root_node, Vec3::zeros(), Some(parent_entity));
    }

    nightshade_scene::commands::transform::propagate_subtree(
        spawn.parts.hierarchy,
        spawn.storage,
        parent_entity,
    );
    parent_entity
}

fn spawn_prefab_node_single(
    spawn: &mut PrefabSpawn<'_>,
    node: &PrefabNode,
    offset: Vec3,
    parent_entity: Option<Entity>,
) -> Entity {
    let mut local_transform = node.local_transform;
    if parent_entity.is_none() {
        local_transform.translation += offset;
    }
    let node_name = node
        .components
        .name
        .as_ref()
        .map(|name| name.0.as_str())
        .unwrap_or("unnamed");

    let mesh = node.components.render_mesh.as_ref().and_then(|render_mesh| {
        let Some(&index) = spawn
            .parts
            .meshes
            .registry
            .name_to_index
            .get(&render_mesh.name)
        else {
            tracing::error!(
                "Mesh '{}' not found in mesh_cache when spawning node {}. Skipping mesh component.",
                render_mesh.name,
                node_name
            );
            return None;
        };
        registry_add_reference(&mut spawn.parts.meshes.registry, index);
        Some(render_mesh)
    });
    let instanced = mesh.is_some() && !node.components.instances.is_empty();
    let bounds = node.components.bounding_volume.or_else(|| {
        node.components
            .render_mesh
            .as_ref()
            .and_then(|render_mesh| get_bounding_volume_from_mesh_name(spawn, &render_mesh.name))
    });
    let material_id = node.components.material.as_ref().map(|material| {
        let material_id = material_registry_find_or_insert_hashed(
            spawn.parts.materials,
            &format!("Prefab_{node_name}"),
            material.clone(),
        );
        registry_add_reference(&mut spawn.parts.materials.registry, material_id.index);
        material_id
    });
    let parent = parent_entity.filter(|parent| {
        spawn
            .storage
            .get::<nightshade_scene::contract::LocalTransform>(*parent)
            .is_some()
    });
    if let Some(missing) = parent_entity
        && parent.is_none()
    {
        tracing::warn!(
            "Parent entity {:?} does not exist or lacks LocalTransform component when spawning node {}",
            missing,
            node_name
        );
    }

    let entity = match (
        &node.components.name,
        mesh,
        material_id,
        bounds,
        parent,
        &node.components.visibility,
    ) {
        (
            Some(name),
            Some(render_mesh),
            Some(material_id),
            Some(bounds),
            Some(parent),
            Some(visibility),
        ) if !instanced => spawn.storage.spawn((
            local_transform,
            GlobalTransform::default(),
            name.clone(),
            render_mesh.clone(),
            MaterialRef::new(material_id),
            bounds,
            visibility.clone(),
            nightshade_render::components::culling::CastsShadow,
            ChildOf(parent),
        )),
        (Some(name), None, None, None, Some(parent), Some(visibility)) => spawn.storage.spawn((
            local_transform,
            GlobalTransform::default(),
            name.clone(),
            visibility.clone(),
            nightshade_scene::contract::Frozen(false),
            ChildOf(parent),
        )),
        _ => {
            let entity = spawn
                .storage
                .spawn((local_transform, GlobalTransform::default()));
            if let Some(name) = &node.components.name {
                spawn.storage.set(entity, name.clone());
            }
            if let Some(render_mesh) = mesh {
                if instanced {
                    let instance_transforms: Vec<
                        nightshade_render::components::mesh::InstanceTransform,
                    > = node
                        .components
                        .instances
                        .iter()
                        .map(|local| {
                            nightshade_render::components::mesh::InstanceTransform::new(
                                local.translation,
                                local.rotation,
                                local.scale,
                            )
                        })
                        .collect();
                    let instanced_mesh =
                        nightshade_render::components::mesh::InstancedMesh::with_instances(
                            &render_mesh.name,
                            instance_transforms,
                        );
                    spawn.storage.set(entity, instanced_mesh);
                } else {
                    spawn.storage.set(entity, render_mesh.clone());
                }
                spawn
                    .storage
                    .set(entity, nightshade_render::components::culling::CastsShadow);
            }
            if let Some(material_id) = material_id {
                spawn.storage.set(entity, MaterialRef::new(material_id));
            }
            if let Some(visibility) = &node.components.visibility {
                spawn.storage.set(entity, visibility.clone());
            }
            if let Some(bounds) = bounds {
                spawn.storage.set(entity, bounds);
            }
            if let Some(parent) = parent {
                spawn.storage.set(entity, ChildOf(parent));
            }
            entity
        }
    };

    if mesh.is_some() && !instanced {
        mesh_state_mark_entity_added(spawn.parts.mesh_state, entity);
    }
    if let (Some(material), Some(material_id)) = (&node.components.material, material_id) {
        let texture_names: Vec<String> = material.texture_names().map(str::to_string).collect();
        texture_cache_acquire(
            spawn.parts.textures,
            TextureOwner::EntityMaterial(entity),
            texture_names,
        );
        let material_name = material_registry_name(spawn.parts.materials, material_id)
            .unwrap_or_default()
            .to_string();
        install_material_variants(
            spawn,
            entity,
            node_name,
            &material_name,
            &node.components.material_variants,
        );
    }
    if let Some(camera) = &node.components.camera {
        spawn.storage.set(entity, *camera);
    }
    if let Some(light) = &node.components.light {
        spawn.storage.set(entity, light.clone());
    }

    entity
}

pub fn spawn_prefab_node(
    spawn: &mut PrefabSpawn<'_>,
    node: &PrefabNode,
    offset: Vec3,
    parent_entity: Option<Entity>,
) -> Entity {
    let mut root_entity: Option<Entity> = None;
    let mut stack: Vec<(&PrefabNode, Option<Entity>)> = vec![(node, parent_entity)];
    while let Some((node, parent_entity)) = stack.pop() {
        let entity = spawn_prefab_node_single(spawn, node, offset, parent_entity);
        if root_entity.is_none() {
            root_entity = Some(entity);
        }
        for child_node in node.children.iter().rev() {
            stack.push((child_node, Some(entity)));
        }
    }
    root_entity.expect("spawn_prefab_node spawns at least the root node")
}

pub fn spawn_prefab_with_animations(
    spawn: &mut PrefabSpawn<'_>,
    prefab: &Prefab,
    animations: &[AnimationClip],
    offset: Vec3,
) -> Entity {
    let parent_entity = spawn.storage.spawn_entities::<(
        nightshade_scene::contract::primitives::Name,
        nightshade_scene::contract::LocalTransform,
        nightshade_scene::contract::GlobalTransform,
    )>(1)[0];

    spawn.storage.set(
        parent_entity,
        nightshade_scene::contract::primitives::Name(prefab.name.clone()),
    );
    spawn.storage.set(
        parent_entity,
        nightshade_scene::contract::LocalTransform {
            translation: offset,
            rotation: nalgebra_glm::Quat::identity(),
            scale: nalgebra_glm::vec3(1.0, 1.0, 1.0),
        },
    );
    spawn.storage.set(parent_entity, GlobalTransform::default());

    let mut node_index_to_entity: HashMap<usize, Entity> = HashMap::new();
    let mut bone_name_to_entity: HashMap<String, Entity> = HashMap::new();

    for root_node in &prefab.root_nodes {
        spawn_prefab_node_with_mapping(
            spawn,
            root_node,
            Vec3::zeros(),
            Some(parent_entity),
            &mut node_index_to_entity,
            &mut bone_name_to_entity,
        );
    }

    if !animations.is_empty() {
        let mut animation_player = AnimationPlayer {
            clips: animations.to_vec(),
            current_clip: if animations.is_empty() { None } else { Some(0) },
            time: 0.0,
            speed: 1.0,
            looping: true,
            playing: true,
            play_all: false,
            node_index_to_entity,
            bone_name_to_entity,
            blend_from_clip: None,
            blend_from_time: 0.0,
            blend_factor: 1.0,
            blend_duration: 0.0,
            layers: Vec::new(),
            ..Default::default()
        };

        if !animation_player.clips.is_empty() {
            animation_player.play(0);
        }

        spawn
            .storage
            .add_components::<(nightshade_animation::contract::AnimationPlayer,)>(parent_entity);
        spawn.storage.set(parent_entity, animation_player);
    }

    nightshade_scene::commands::transform::propagate_subtree(
        spawn.parts.hierarchy,
        spawn.storage,
        parent_entity,
    );
    parent_entity
}

fn spawn_prefab_node_with_mapping(
    spawn: &mut PrefabSpawn<'_>,
    node: &PrefabNode,
    offset: Vec3,
    parent_entity: Option<Entity>,
    node_index_to_entity: &mut HashMap<usize, Entity>,
    bone_name_to_entity: &mut HashMap<String, Entity>,
) -> Entity {
    let mut root_entity: Option<Entity> = None;
    let mut stack: Vec<(&PrefabNode, Option<Entity>)> = vec![(node, parent_entity)];
    while let Some((node, parent_entity)) = stack.pop() {
        let entity = spawn_prefab_node_single(spawn, node, offset, parent_entity);

        if let Some(node_index) = node.node_index {
            node_index_to_entity.insert(node_index, entity);
        }
        if let Some(name) = &node.components.name {
            bone_name_to_entity.insert(name.0.clone(), entity);
        }

        if root_entity.is_none() {
            root_entity = Some(entity);
        }
        for child_node in node.children.iter().rev() {
            stack.push((child_node, Some(entity)));
        }
    }
    root_entity.expect("spawn_prefab_node_with_mapping spawns at least the root node")
}
pub fn spawn_prefab_with_source(
    spawn: &mut PrefabSpawn<'_>,
    prefab: &Prefab,
    animations: &[AnimationClip],
    skins: &[ModelSkin],
    offset: Vec3,
    source_path: Option<&str>,
) -> Entity {
    let parent_entity = spawn.storage.spawn_entities::<(
        nightshade_scene::contract::primitives::Name,
        nightshade_scene::contract::LocalTransform,
        nightshade_scene::contract::GlobalTransform,
        crate::contract::PrefabSource,
    )>(1)[0];

    spawn.storage.set(
        parent_entity,
        nightshade_scene::contract::primitives::Name(prefab.name.clone()),
    );
    spawn.storage.set(
        parent_entity,
        nightshade_scene::contract::LocalTransform {
            translation: offset,
            rotation: nalgebra_glm::Quat::identity(),
            scale: nalgebra_glm::vec3(1.0, 1.0, 1.0),
        },
    );
    spawn.storage.set(parent_entity, GlobalTransform::default());

    let prefab_source = if let Some(path) = source_path {
        crate::contract::prefab_source_with_path(&prefab.name, path)
    } else {
        crate::contract::prefab_source_new(&prefab.name)
    };
    spawn.storage.set(parent_entity, prefab_source);

    let mut node_index_to_entity: HashMap<usize, Entity> = HashMap::new();
    let mut bone_name_to_entity: HashMap<String, Entity> = HashMap::new();
    let mut entity_to_skin_index: HashMap<Entity, usize> = HashMap::new();

    for root_node in &prefab.root_nodes {
        spawn_prefab_node_with_skin_mapping(
            spawn,
            root_node,
            Vec3::zeros(),
            Some(parent_entity),
            &mut node_index_to_entity,
            &mut bone_name_to_entity,
            &mut entity_to_skin_index,
        );
    }

    tracing::info!(
        "spawn_prefab_with_source: {} node mappings, {} bone names, {} entities need skins",
        node_index_to_entity.len(),
        bone_name_to_entity.len(),
        entity_to_skin_index.len()
    );

    for (&entity, &skin_index) in &entity_to_skin_index {
        if skin_index < skins.len() {
            let mesh_known_unskinned = spawn
                .storage
                .get::<nightshade_render::components::mesh::RenderMesh>(entity)
                .map(|render_mesh| render_mesh.name.clone())
                .and_then(|name| {
                    registry_entry_by_name(&spawn.parts.meshes.registry, &name)
                        .map(|mesh| mesh.skin_data.is_none())
                })
                .unwrap_or(false);
            if mesh_known_unskinned {
                continue;
            }

            let gltf_skin = &skins[skin_index];

            let joints: Vec<Entity> = gltf_skin
                .joints
                .iter()
                .filter_map(|&joint_node_index| {
                    node_index_to_entity.get(&joint_node_index).copied()
                })
                .collect();

            tracing::info!(
                "  Skin {}: expected {} joints, found {} (name: {:?})",
                skin_index,
                gltf_skin.joints.len(),
                joints.len(),
                gltf_skin.name
            );
            if joints.len() < gltf_skin.joints.len() {
                let missing: Vec<_> = gltf_skin
                    .joints
                    .iter()
                    .filter(|idx| !node_index_to_entity.contains_key(*idx))
                    .take(5)
                    .collect();
                tracing::warn!("  Missing joint node indices (first 5): {:?}", missing);
            }

            for (joint_index, &joint_entity) in joints.iter().enumerate() {
                spawn
                    .storage
                    .add_components::<(nightshade_render::components::skin::Joint,)>(joint_entity);
                spawn.storage.set(
                    joint_entity,
                    Joint {
                        index_in_skin: joint_index,
                        name: None,
                    },
                );
            }

            let skin = Skin {
                joints,
                inverse_bind_matrices: Arc::new(gltf_skin.inverse_bind_matrices.clone()),
                name: gltf_skin.name.clone(),
            };

            spawn
                .storage
                .add_components::<(nightshade_render::components::skin::Skin,)>(entity);
            spawn.storage.set(entity, skin);
        }
    }

    if !animations.is_empty() {
        tracing::info!("Setting up AnimationPlayer with {} clips", animations.len());
        for (clip_idx, clip) in animations.iter().enumerate() {
            tracing::info!(
                "  Clip {} '{}': {} channels, duration: {}s",
                clip_idx,
                clip.name,
                clip.channels.len(),
                clip.duration
            );
            for (ch_idx, channel) in clip.channels.iter().take(5).enumerate() {
                let found_by_name = channel
                    .target_bone_name
                    .as_ref()
                    .and_then(|name| bone_name_to_entity.get(name));
                let found_by_idx = node_index_to_entity.get(&channel.target_node);
                tracing::info!(
                    "    Ch{}: node={}, bone={:?}, found_name={}, found_idx={}",
                    ch_idx,
                    channel.target_node,
                    channel.target_bone_name,
                    found_by_name.is_some(),
                    found_by_idx.is_some()
                );
            }
            if clip.channels.len() > 5 {
                tracing::info!("    ... and {} more channels", clip.channels.len() - 5);
            }
        }
        let mut animation_player = AnimationPlayer {
            clips: animations.to_vec(),
            current_clip: if animations.is_empty() { None } else { Some(0) },
            time: 0.0,
            speed: 1.0,
            looping: true,
            playing: true,
            play_all: false,
            node_index_to_entity,
            bone_name_to_entity,
            blend_from_clip: None,
            blend_from_time: 0.0,
            blend_factor: 1.0,
            blend_duration: 0.0,
            layers: Vec::new(),
            ..Default::default()
        };

        if !animation_player.clips.is_empty() {
            animation_player.play(0);
        }

        spawn
            .storage
            .add_components::<(nightshade_animation::contract::AnimationPlayer,)>(parent_entity);
        spawn.storage.set(parent_entity, animation_player);
    }

    nightshade_scene::commands::transform::propagate_subtree(
        spawn.parts.hierarchy,
        spawn.storage,
        parent_entity,
    );
    parent_entity
}
pub fn spawn_prefab_with_skins(
    spawn: &mut PrefabSpawn<'_>,
    prefab: &Prefab,
    animations: &[AnimationClip],
    skins: &[ModelSkin],
    offset: Vec3,
) -> Entity {
    let parent_entity = spawn.storage.spawn_entities::<(
        nightshade_scene::contract::primitives::Name,
        nightshade_scene::contract::LocalTransform,
        nightshade_scene::contract::GlobalTransform,
        crate::contract::PrefabSource,
    )>(1)[0];

    spawn.storage.set(
        parent_entity,
        nightshade_scene::contract::primitives::Name(prefab.name.clone()),
    );
    spawn.storage.set(
        parent_entity,
        nightshade_scene::contract::LocalTransform {
            translation: offset,
            rotation: nalgebra_glm::Quat::identity(),
            scale: nalgebra_glm::vec3(1.0, 1.0, 1.0),
        },
    );
    spawn.storage.set(parent_entity, GlobalTransform::default());
    spawn.storage.set(
        parent_entity,
        crate::contract::prefab_source_new(&prefab.name),
    );

    let mut node_index_to_entity: HashMap<usize, Entity> = HashMap::new();
    let mut bone_name_to_entity: HashMap<String, Entity> = HashMap::new();
    let mut entity_to_skin_index: HashMap<Entity, usize> = HashMap::new();

    for root_node in &prefab.root_nodes {
        spawn_prefab_node_with_skin_mapping(
            spawn,
            root_node,
            Vec3::zeros(),
            Some(parent_entity),
            &mut node_index_to_entity,
            &mut bone_name_to_entity,
            &mut entity_to_skin_index,
        );
    }

    for (&entity, &skin_index) in &entity_to_skin_index {
        if skin_index < skins.len() {
            let mesh_known_unskinned = spawn
                .storage
                .get::<nightshade_render::components::mesh::RenderMesh>(entity)
                .map(|render_mesh| render_mesh.name.clone())
                .and_then(|name| {
                    registry_entry_by_name(&spawn.parts.meshes.registry, &name)
                        .map(|mesh| mesh.skin_data.is_none())
                })
                .unwrap_or(false);
            if mesh_known_unskinned {
                continue;
            }

            let gltf_skin = &skins[skin_index];

            let joints: Vec<Entity> = gltf_skin
                .joints
                .iter()
                .filter_map(|&joint_node_index| {
                    node_index_to_entity.get(&joint_node_index).copied()
                })
                .collect();

            for (joint_index, &joint_entity) in joints.iter().enumerate() {
                spawn
                    .storage
                    .add_components::<(nightshade_render::components::skin::Joint,)>(joint_entity);
                spawn.storage.set(
                    joint_entity,
                    Joint {
                        index_in_skin: joint_index,
                        name: None,
                    },
                );
            }

            let skin = Skin {
                joints,
                inverse_bind_matrices: Arc::new(gltf_skin.inverse_bind_matrices.clone()),
                name: gltf_skin.name.clone(),
            };

            spawn
                .storage
                .add_components::<(nightshade_render::components::skin::Skin,)>(entity);
            spawn.storage.set(entity, skin);
        }
    }

    if !animations.is_empty() {
        let mut animation_player = AnimationPlayer {
            clips: animations.to_vec(),
            current_clip: if animations.is_empty() { None } else { Some(0) },
            time: 0.0,
            speed: 1.0,
            looping: true,
            playing: true,
            play_all: false,
            node_index_to_entity,
            bone_name_to_entity,
            blend_from_clip: None,
            blend_from_time: 0.0,
            blend_factor: 1.0,
            blend_duration: 0.0,
            layers: Vec::new(),
            ..Default::default()
        };

        if !animation_player.clips.is_empty() {
            animation_player.play(0);
        }

        spawn
            .storage
            .add_components::<(nightshade_animation::contract::AnimationPlayer,)>(parent_entity);
        spawn.storage.set(parent_entity, animation_player);
    }

    nightshade_scene::commands::transform::propagate_subtree(
        spawn.parts.hierarchy,
        spawn.storage,
        parent_entity,
    );
    parent_entity
}
fn spawn_prefab_node_with_skin_mapping(
    spawn: &mut PrefabSpawn<'_>,
    node: &PrefabNode,
    offset: Vec3,
    parent_entity: Option<Entity>,
    node_index_to_entity: &mut HashMap<usize, Entity>,
    bone_name_to_entity: &mut HashMap<String, Entity>,
    entity_to_skin_index: &mut HashMap<Entity, usize>,
) -> Entity {
    let mut root_entity: Option<Entity> = None;
    let mut stack: Vec<(&PrefabNode, Option<Entity>)> = vec![(node, parent_entity)];
    while let Some((node, parent_entity)) = stack.pop() {
        let entity = spawn_prefab_node_single(spawn, node, offset, parent_entity);

        if let Some(node_index) = node.node_index {
            node_index_to_entity.insert(node_index, entity);
        }
        if let Some(name) = &node.components.name {
            bone_name_to_entity.insert(name.0.clone(), entity);
        }
        if let Some(skin_index) = node.components.skin_index {
            entity_to_skin_index.insert(entity, skin_index);
        }

        if let Some(render_mesh) = &node.components.render_mesh
            && node.components.instances.is_empty()
        {
            let morph_weights_to_add =
                registry_entry_by_name(&spawn.parts.meshes.registry, &render_mesh.name)
                    .and_then(|mesh| mesh.morph_targets.as_ref())
                    .map(|morph_data| {
                        MorphWeights::new(
                            morph_data.default_weights.clone(),
                            render_mesh.name.clone(),
                        )
                    });

            if let Some(morph_weights) = morph_weights_to_add {
                spawn
                    .storage
                    .add_components::<(nightshade_render::components::morph::MorphWeights,)>(
                        entity,
                    );
                spawn.storage.set(entity, morph_weights);
            }
        }

        if root_entity.is_none() {
            root_entity = Some(entity);
        }
        for child_node in node.children.iter().rev() {
            stack.push((child_node, Some(entity)));
        }
    }
    root_entity.expect("spawn_prefab_node_with_skin_mapping spawns at least the root node")
}
pub fn queue_model_load(spawn: &mut PrefabSpawn<'_>, import: &mut crate::data::ModelImport) {
    let mut load = nightshade_graphics::resources::loading::TextureLoad {
        sources: spawn.parts.sources,
        cache: spawn.parts.textures,
        loading: spawn.parts.loading,
    };
    let mesh_cache = &mut *spawn.parts.meshes;
    nightshade_graphics::commands::loading::queue_texture_and_mesh_upload(
        &mut load,
        mesh_cache,
        std::mem::take(&mut import.encoded_images),
        std::mem::take(&mut import.texture_plan),
        std::mem::take(&mut import.meshes),
    );
}
