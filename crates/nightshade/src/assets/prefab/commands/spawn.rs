#[cfg(feature = "assets")]
use crate::assets::prefab::commands::gltf_import::GltfSkin;
use crate::ecs::animation::components::{AnimationClip, AnimationPlayer};
use crate::ecs::material::components::MaterialRef;
use crate::ecs::material::resources::material_registry_find_or_insert;
use crate::ecs::morph::components::MorphWeights;
#[cfg(feature = "assets")]
use crate::ecs::skin::components::{Joint, Skin};
use crate::ecs::transform::components::{ChildOf, GlobalTransform};
use crate::ecs::world::{
    ANIMATION_PLAYER, BOUNDING_VOLUME, CAMERA, CASTS_SHADOW, GLOBAL_TRANSFORM, INSTANCED_MESH,
    LIGHT, LOCAL_TRANSFORM, MATERIAL_REF, MATERIAL_VARIANTS, MORPH_WEIGHTS, NAME, PARENT,
    RENDER_MESH, VISIBILITY, Vec3, World,
};
#[cfg(feature = "assets")]
use crate::ecs::world::{JOINT, PREFAB_SOURCE, SKIN};
use crate::prelude::*;
use crate::render::bounding_volume::BoundingVolume;
use crate::render::generational_registry::registry_entry_by_name;
use crate::render::wgpu::texture_cache::{TextureOwner, texture_cache_acquire};
use nightshade_ecs::Entity;

fn install_render_or_instanced_mesh(world: &mut World, entity: Entity, node: &PrefabNode) {
    let Some(render_mesh) = &node.components.render_mesh else {
        return;
    };
    if !world
        .res::<crate::render::mesh_cache::MeshCache>()
        .registry
        .name_to_index
        .contains_key(&render_mesh.name)
    {
        tracing::error!(
            "Mesh '{}' not found in mesh_cache when spawning entity {:?}. Skipping mesh component.",
            render_mesh.name,
            entity
        );
        return;
    }
    if let Some(&index) = world
        .res::<crate::render::mesh_cache::MeshCache>()
        .registry
        .name_to_index
        .get(&render_mesh.name)
    {
        registry_add_reference(
            &mut world
                .res_mut::<crate::render::mesh_cache::MeshCache>()
                .registry,
            index,
        );
    }
    if node.components.instances.is_empty() {
        world.set(entity, render_mesh.clone());
        world
            .res_mut::<crate::render::mesh_state::MeshRenderState>()
            .mark_entity_added(entity);
    } else {
        let instance_transforms: Vec<crate::ecs::mesh::components::InstanceTransform> = node
            .components
            .instances
            .iter()
            .map(|local| {
                crate::ecs::mesh::components::InstanceTransform::new(
                    local.translation,
                    local.rotation,
                    local.scale,
                )
            })
            .collect();
        let instanced = crate::ecs::mesh::components::InstancedMesh::with_instances(
            &render_mesh.name,
            instance_transforms,
        );
        world.set(entity, instanced);
    }
    world.set(entity, crate::ecs::primitives::CastsShadow);
}

fn install_material_variants(
    world: &mut World,
    entity: Entity,
    node_name: &str,
    default_material_name: &str,
    variants: &[crate::assets::prefab::components::PrefabMaterialVariant],
) {
    if variants.is_empty() {
        return;
    }
    let variant_textures: Vec<String> = variants
        .iter()
        .flat_map(|variant| variant.material.texture_names().map(str::to_string))
        .collect();
    texture_cache_acquire(
        world.res_mut::<crate::render::wgpu::texture_cache::TextureCache>(),
        TextureOwner::EntityVariants(entity),
        variant_textures,
    );
    let mut mappings = Vec::with_capacity(variants.len());
    for (variant_index, variant) in variants.iter().enumerate() {
        let fallback_name = format!(
            "Prefab_{}_{}_variant_{}",
            node_name, entity.id, variant_index
        );
        let variant_material_name = material_registry_find_or_insert(
            world.res_mut::<crate::ecs::material::resources::MaterialRegistry>(),
            fallback_name,
            variant.material.clone(),
        );
        if let Some(&index) = world
            .res::<crate::ecs::material::resources::MaterialRegistry>()
            .registry
            .name_to_index
            .get(&variant_material_name)
        {
            registry_add_reference(
                &mut world
                    .res_mut::<crate::ecs::material::resources::MaterialRegistry>()
                    .registry,
                index,
            );
        }
        mappings.push(crate::ecs::material::components::MaterialVariantMapping {
            variant_names: variant.variant_names.clone(),
            material_name: variant_material_name,
        });
    }
    world.set(
        entity,
        crate::ecs::material::components::MaterialVariants {
            default_material_name: default_material_name.to_string(),
            mappings,
        },
    );
}
use std::collections::HashMap;
#[cfg(feature = "assets")]
use std::sync::Arc;

use super::super::components::{Prefab, PrefabNode};

fn get_bounding_volume_from_mesh_name(world: &World, mesh_name: &str) -> Option<BoundingVolume> {
    let mesh = registry_entry_by_name(
        &world.res::<crate::render::mesh_cache::MeshCache>().registry,
        mesh_name,
    )?;
    mesh.bounding_volume
}

pub fn spawn_prefab(world: &mut World, prefab: &Prefab, offset: Vec3) -> Entity {
    let parent_entity = spawn_entities(world, NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM, 1)[0];

    world.set(
        parent_entity,
        crate::ecs::primitives::Name(prefab.name.clone()),
    );
    world.set(
        parent_entity,
        crate::ecs::transform::components::LocalTransform {
            translation: offset,
            rotation: nalgebra_glm::Quat::identity(),
            scale: nalgebra_glm::vec3(1.0, 1.0, 1.0),
        },
    );
    world.set(parent_entity, GlobalTransform::default());

    for root_node in &prefab.root_nodes {
        spawn_prefab_node(world, root_node, Vec3::zeros(), Some(parent_entity));
    }

    crate::ecs::transform::systems::propagate_subtree_global_transforms(world, parent_entity);
    parent_entity
}

/// Spawns the entity for a single prefab node and attaches its
/// components, without touching the node's children. The iterative
/// drivers below call this once per node so a deeply nested prefab
/// cannot overflow the stack.
fn spawn_prefab_node_single(
    world: &mut World,
    node: &PrefabNode,
    offset: Vec3,
    parent_entity: Option<Entity>,
) -> Entity {
    let calculated_bounding_volume = if node.components.bounding_volume.is_none() {
        if let Some(render_mesh) = &node.components.render_mesh {
            get_bounding_volume_from_mesh_name(world, &render_mesh.name)
        } else {
            None
        }
    } else {
        None
    };

    let mut component_mask = LOCAL_TRANSFORM | GLOBAL_TRANSFORM;

    if node.components.name.is_some() {
        component_mask |= NAME;
    }

    if node.components.render_mesh.is_some() {
        if node.components.instances.is_empty() {
            component_mask |= RENDER_MESH | CASTS_SHADOW;
        } else {
            component_mask |= INSTANCED_MESH | CASTS_SHADOW;
        }
    }

    if node.components.material.is_some() {
        component_mask |= MATERIAL_REF;
    }

    if !node.components.material_variants.is_empty() {
        component_mask |= MATERIAL_VARIANTS;
    }

    if node.components.camera.is_some() {
        component_mask |= CAMERA;
    }

    if node.components.light.is_some() {
        component_mask |= LIGHT;
    }

    if node.components.visibility.is_some() {
        component_mask |= VISIBILITY;
    }

    if node.components.bounding_volume.is_some() || calculated_bounding_volume.is_some() {
        component_mask |= BOUNDING_VOLUME;
    }

    if parent_entity.is_some() {
        component_mask |= PARENT;
    }

    let entity = spawn_entities(world, component_mask, 1)[0];

    let mut local_transform = node.local_transform;
    if parent_entity.is_none() {
        local_transform.translation += offset;
    }

    world.set(entity, local_transform);
    world.set(entity, GlobalTransform::default());

    if let Some(name) = &node.components.name {
        world.set(entity, name.clone());
    }

    install_render_or_instanced_mesh(world, entity, node);

    if let Some(material) = &node.components.material {
        let texture_names: Vec<String> = material.texture_names().map(str::to_string).collect();
        texture_cache_acquire(
            world.res_mut::<crate::render::wgpu::texture_cache::TextureCache>(),
            TextureOwner::EntityMaterial(entity),
            texture_names,
        );

        let node_name = node
            .components
            .name
            .as_ref()
            .map(|n| n.0.as_str())
            .unwrap_or("unnamed");
        let fallback_name = format!("Prefab_{}_{}", node_name, entity.id);
        let material_name = material_registry_find_or_insert(
            world.res_mut::<crate::ecs::material::resources::MaterialRegistry>(),
            fallback_name,
            material.clone(),
        );
        if let Some(&index) = world
            .res::<crate::ecs::material::resources::MaterialRegistry>()
            .registry
            .name_to_index
            .get(&material_name)
        {
            registry_add_reference(
                &mut world
                    .res_mut::<crate::ecs::material::resources::MaterialRegistry>()
                    .registry,
                index,
            );
        }
        world.set(entity, MaterialRef::new(material_name.clone()));
        install_material_variants(
            world,
            entity,
            node_name,
            &material_name,
            &node.components.material_variants,
        );
    }

    if let Some(camera) = &node.components.camera {
        world.set(entity, *camera);
    }

    if let Some(light) = &node.components.light {
        world.set(entity, light.clone());
    }

    if let Some(visibility) = &node.components.visibility {
        world.set(entity, visibility.clone());
    }

    if let Some(bounding_volume) = &node.components.bounding_volume {
        world.set(entity, *bounding_volume);
    } else if let Some(calculated_bv) = calculated_bounding_volume {
        world.set(entity, calculated_bv);
    }

    crate::assets::prefab::hooks::run_entity_spawn(
        world,
        entity,
        &node.components,
        local_transform,
    );

    if let Some(parent) = parent_entity {
        if world
            .get::<crate::ecs::transform::components::LocalTransform>(parent)
            .is_some()
        {
            world.set(entity, ChildOf(parent));
        } else {
            tracing::warn!(
                "Parent entity {:?} does not exist or lacks LocalTransform component when spawning entity {:?}",
                parent,
                entity
            );
        }
    }

    entity
}

pub fn spawn_prefab_node(
    world: &mut World,
    node: &PrefabNode,
    offset: Vec3,
    parent_entity: Option<Entity>,
) -> Entity {
    let mut root_entity: Option<Entity> = None;
    let mut stack: Vec<(&PrefabNode, Option<Entity>)> = vec![(node, parent_entity)];
    while let Some((node, parent_entity)) = stack.pop() {
        let entity = spawn_prefab_node_single(world, node, offset, parent_entity);
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
    world: &mut World,
    prefab: &Prefab,
    animations: &[AnimationClip],
    offset: Vec3,
) -> Entity {
    let parent_entity = spawn_entities(world, NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM, 1)[0];

    world.set(
        parent_entity,
        crate::ecs::primitives::Name(prefab.name.clone()),
    );
    world.set(
        parent_entity,
        crate::ecs::transform::components::LocalTransform {
            translation: offset,
            rotation: nalgebra_glm::Quat::identity(),
            scale: nalgebra_glm::vec3(1.0, 1.0, 1.0),
        },
    );
    world.set(parent_entity, GlobalTransform::default());

    let mut node_index_to_entity: HashMap<usize, Entity> = HashMap::new();
    let mut bone_name_to_entity: HashMap<String, Entity> = HashMap::new();

    for root_node in &prefab.root_nodes {
        spawn_prefab_node_with_mapping(
            world,
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

        world.ecs.worlds[CORE].add_components(parent_entity, ANIMATION_PLAYER);
        world.set(parent_entity, animation_player);
    }

    crate::ecs::transform::systems::propagate_subtree_global_transforms(world, parent_entity);
    parent_entity
}

fn spawn_prefab_node_with_mapping(
    world: &mut World,
    node: &PrefabNode,
    offset: Vec3,
    parent_entity: Option<Entity>,
    node_index_to_entity: &mut HashMap<usize, Entity>,
    bone_name_to_entity: &mut HashMap<String, Entity>,
) -> Entity {
    let mut root_entity: Option<Entity> = None;
    let mut stack: Vec<(&PrefabNode, Option<Entity>)> = vec![(node, parent_entity)];
    while let Some((node, parent_entity)) = stack.pop() {
        let entity = spawn_prefab_node_single(world, node, offset, parent_entity);

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

#[cfg(feature = "assets")]
pub fn spawn_prefab_with_source(
    world: &mut World,
    prefab: &Prefab,
    animations: &[AnimationClip],
    skins: &[GltfSkin],
    offset: Vec3,
    source_path: Option<&str>,
) -> Entity {
    let parent_entity = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | PREFAB_SOURCE,
        1,
    )[0];

    world.set(
        parent_entity,
        crate::ecs::primitives::Name(prefab.name.clone()),
    );
    world.set(
        parent_entity,
        crate::ecs::transform::components::LocalTransform {
            translation: offset,
            rotation: nalgebra_glm::Quat::identity(),
            scale: nalgebra_glm::vec3(1.0, 1.0, 1.0),
        },
    );
    world.set(parent_entity, GlobalTransform::default());

    let prefab_source = if let Some(path) = source_path {
        super::super::components::prefab_source_with_path(&prefab.name, path)
    } else {
        super::super::components::prefab_source_new(&prefab.name)
    };
    world.set(parent_entity, prefab_source);

    let mut node_index_to_entity: HashMap<usize, Entity> = HashMap::new();
    let mut bone_name_to_entity: HashMap<String, Entity> = HashMap::new();
    let mut entity_to_skin_index: HashMap<Entity, usize> = HashMap::new();

    for root_node in &prefab.root_nodes {
        spawn_prefab_node_with_skin_mapping(
            world,
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
            let mesh_known_unskinned = world
                .get::<crate::ecs::mesh::components::RenderMesh>(entity)
                .map(|render_mesh| render_mesh.name.clone())
                .and_then(|name| {
                    registry_entry_by_name(
                        &world.res::<crate::render::mesh_cache::MeshCache>().registry,
                        &name,
                    )
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
                world.ecs.worlds[CORE].add_components(joint_entity, JOINT);
                world.set(
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

            world.ecs.worlds[CORE].add_components(entity, SKIN);
            world.set(entity, skin);
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

        world.ecs.worlds[CORE].add_components(parent_entity, ANIMATION_PLAYER);
        world.set(parent_entity, animation_player);
    }

    crate::ecs::transform::systems::propagate_subtree_global_transforms(world, parent_entity);
    parent_entity
}

#[cfg(feature = "assets")]
pub fn spawn_prefab_with_skins(
    world: &mut World,
    prefab: &Prefab,
    animations: &[AnimationClip],
    skins: &[GltfSkin],
    offset: Vec3,
) -> Entity {
    let parent_entity = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | PREFAB_SOURCE,
        1,
    )[0];

    world.set(
        parent_entity,
        crate::ecs::primitives::Name(prefab.name.clone()),
    );
    world.set(
        parent_entity,
        crate::ecs::transform::components::LocalTransform {
            translation: offset,
            rotation: nalgebra_glm::Quat::identity(),
            scale: nalgebra_glm::vec3(1.0, 1.0, 1.0),
        },
    );
    world.set(parent_entity, GlobalTransform::default());
    world.set(
        parent_entity,
        super::super::components::prefab_source_new(&prefab.name),
    );

    let mut node_index_to_entity: HashMap<usize, Entity> = HashMap::new();
    let mut bone_name_to_entity: HashMap<String, Entity> = HashMap::new();
    let mut entity_to_skin_index: HashMap<Entity, usize> = HashMap::new();

    for root_node in &prefab.root_nodes {
        spawn_prefab_node_with_skin_mapping(
            world,
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
            let mesh_known_unskinned = world
                .get::<crate::ecs::mesh::components::RenderMesh>(entity)
                .map(|render_mesh| render_mesh.name.clone())
                .and_then(|name| {
                    registry_entry_by_name(
                        &world.res::<crate::render::mesh_cache::MeshCache>().registry,
                        &name,
                    )
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
                world.ecs.worlds[CORE].add_components(joint_entity, JOINT);
                world.set(
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

            world.ecs.worlds[CORE].add_components(entity, SKIN);
            world.set(entity, skin);
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

        world.ecs.worlds[CORE].add_components(parent_entity, ANIMATION_PLAYER);
        world.set(parent_entity, animation_player);
    }

    crate::ecs::transform::systems::propagate_subtree_global_transforms(world, parent_entity);
    parent_entity
}

#[cfg(feature = "assets")]
fn spawn_prefab_node_with_skin_mapping(
    world: &mut World,
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
        let entity = spawn_prefab_node_single(world, node, offset, parent_entity);

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
            let morph_weights_to_add = registry_entry_by_name(
                &world.res::<crate::render::mesh_cache::MeshCache>().registry,
                &render_mesh.name,
            )
            .and_then(|mesh| mesh.morph_targets.as_ref())
            .map(|morph_data| {
                MorphWeights::new(morph_data.default_weights.clone(), render_mesh.name.clone())
            });

            if let Some(morph_weights) = morph_weights_to_add {
                world.ecs.worlds[CORE].add_components(entity, MORPH_WEIGHTS);
                world.set(entity, morph_weights);
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

/// Queues a glTF import's textures and meshes for upload.
///
/// Takes the import by reference and moves out only the fields it consumes,
/// leaving prefabs, animations, and skins in place: most callers spawn prefabs
/// and drive animations from the same import afterward, so consuming the whole
/// thing would force them to clone what they still need.
///
/// Lives here rather than beside
/// [`queue_texture_and_mesh_upload`](crate::assets::loading::queue_texture_and_mesh_upload)
/// because it names the import type, and `ecs::loading` sits below `ecs::prefab`
/// in the module graph. Reaching the other way would close a cycle, which
/// `just lint-dag` rejects.
#[cfg(feature = "assets")]
pub fn queue_gltf_load(world: &mut World, import: &mut crate::assets::prefab::GltfLoadResult) {
    crate::assets::loading::queue_texture_and_mesh_upload(
        world,
        std::mem::take(&mut import.encoded_images),
        std::mem::take(&mut import.texture_plan),
        std::mem::take(&mut import.meshes),
    );
}

/// Loads a glTF or GLB file from memory and puts it in the world at `position`,
/// returning the root entity of every prefab it contained.
///
/// This is the whole of what most callers want from a model file, and doing it
/// by hand is a three step ritual: import, queue the textures and meshes for
/// upload, then spawn each prefab. Getting the order wrong loads a model with no
/// textures, so it is worth having in one place.
///
/// Reach for [`import_gltf_from_bytes`](crate::assets::prefab::import_gltf_from_bytes)
/// and [`queue_gltf_load`](crate::assets::prefab::queue_gltf_load) directly when
/// the prefabs need inspecting or filtering before they are spawned, or when the
/// import's animations and skins are needed, which
/// [`spawn_prefab_with_skins`] takes.
#[cfg(feature = "assets")]
pub fn spawn_gltf_from_bytes(
    world: &mut World,
    bytes: &[u8],
    position: Vec3,
) -> Result<Vec<Entity>, Box<dyn std::error::Error>> {
    let mut import = crate::assets::prefab::import_gltf_from_bytes(bytes)?;
    queue_gltf_load(world, &mut import);
    Ok(import
        .prefabs
        .iter()
        .map(|prefab| spawn_prefab(world, prefab, position))
        .collect())
}
