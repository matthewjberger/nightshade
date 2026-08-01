use std::collections::HashMap;

use nightshade::prelude::*;
use nightshade_animation::contract::AnimationPlayer;
use nightshade_camera::components::{Camera, Projection};
use nightshade_gltf::data::{
    ExportCamera, ExportImage, ExportLight, ExportLightKind, ExportMesh, ExportNode, ExportSkin,
    SceneExport,
};
use nightshade_render::components::light::{Light, LightType};
use nightshade_render::components::material::MaterialRef;
use nightshade_render::components::mesh::RenderMesh;
use nightshade_render::components::morph::MorphWeights;
use nightshade_render::components::skin::Skin;
use nightshade_render::data::generational_registry::registry_entry_by_name;
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_render::resources::mesh_cache::MeshCache;
use nightshade_render::resources::texture_sources::{TextureSourceData, TextureSources};
use nightshade_scene::contract::{LocalTransform, Name};

use crate::resources::ViewerResources;

pub fn gather(
    viewer: &ViewerResources,
    storage: &nightshade::ecs::Storage,
    hierarchy: &nightshade::ecs::HierarchyIndex,
    meshes: &MeshCache,
    materials: &MaterialRegistry,
    textures: &TextureSources,
    name: &str,
) -> Result<SceneExport, String> {
    if viewer.model.roots.is_empty() {
        return Err("There is no model open to write.".to_string());
    }

    let mut scene = SceneExport {
        name: name.to_string(),
        ..SceneExport::default()
    };
    let mut index_of: HashMap<Entity, usize> = HashMap::new();
    let mut mesh_of: HashMap<String, usize> = HashMap::new();
    let mut material_of: HashMap<String, usize> = HashMap::new();
    let mut image_of: HashMap<String, usize> = HashMap::new();
    let mut skinned: Vec<(usize, Skin)> = Vec::new();

    let mut order: Vec<Entity> = Vec::new();
    let mut stack: Vec<Entity> = viewer.model.roots.iter().rev().copied().collect();
    while let Some(entity) = stack.pop() {
        if index_of.contains_key(&entity) {
            continue;
        }
        index_of.insert(entity, order.len());
        order.push(entity);
        for child in hierarchy.children(entity).iter().rev() {
            stack.push(*child);
        }
    }

    for entity in &order {
        let entity = *entity;
        let mut node = ExportNode {
            name: storage
                .get::<Name>(entity)
                .map(|name| name.0.clone())
                .unwrap_or_default(),
            ..ExportNode::default()
        };
        if let Some(transform) = storage.get::<LocalTransform>(entity) {
            node.translation = transform.translation;
            node.rotation = transform.rotation;
            node.scale = transform.scale;
        }
        node.children = hierarchy
            .children(entity)
            .iter()
            .filter_map(|child| index_of.get(child).copied())
            .collect();

        if let Some(render_mesh) = storage.get::<RenderMesh>(entity) {
            let material = storage.get::<MaterialRef>(entity).and_then(|reference| {
                let name = nightshade_render::queries::material_registry::material_registry_name(
                    materials,
                    reference.id,
                )?
                .to_string();
                material_index(
                    &name,
                    materials,
                    &mut material_of,
                    &mut scene,
                    textures,
                    &mut image_of,
                )
            });
            let key = format!("{}|{}", render_mesh.name, material.unwrap_or(usize::MAX));
            let mesh = match mesh_of.get(&key) {
                Some(index) => Some(*index),
                None => {
                    registry_entry_by_name(&meshes.registry, &render_mesh.name).map(|geometry| {
                        scene.meshes.push(ExportMesh {
                            name: render_mesh.name.clone(),
                            mesh: geometry.clone(),
                            material,
                        });
                        let index = scene.meshes.len() - 1;
                        mesh_of.insert(key, index);
                        index
                    })
                }
            };
            node.mesh = mesh;
        }

        if let Some(weights) = storage.get::<MorphWeights>(entity) {
            node.morph_weights = weights.weights.clone();
        }

        if let Some(light) = storage.get::<Light>(entity) {
            node.light = Some(ExportLight {
                name: storage.get::<Name>(entity).map(|name| name.0.clone()),
                kind: match light.light_type {
                    LightType::Directional => ExportLightKind::Directional,
                    LightType::Spot => ExportLightKind::Spot {
                        inner_cone: light.inner_cone_angle,
                        outer_cone: light.outer_cone_angle,
                    },
                    LightType::Point | LightType::Area => ExportLightKind::Point,
                },
                color: [light.color.x, light.color.y, light.color.z],
                intensity: light.intensity,
                range: (light.range > 0.0).then_some(light.range),
            });
        }

        if let Some(camera) = storage.get::<Camera>(entity) {
            node.camera = Some(match &camera.projection {
                Projection::Perspective(perspective) => ExportCamera::Perspective {
                    field_of_view: perspective.y_fov_rad,
                    aspect_ratio: perspective.aspect_ratio,
                    near: perspective.z_near,
                    far: perspective.z_far,
                },
                Projection::Orthographic(orthographic) => ExportCamera::Orthographic {
                    horizontal_magnification: orthographic.x_mag.unwrap_or(orthographic.y_mag),
                    vertical_magnification: orthographic.y_mag,
                    near: orthographic.z_near,
                    far: orthographic.z_far,
                },
            });
        }

        let position = index_of[&entity];
        if let Some(skin) = storage.get::<Skin>(entity) {
            skinned.push((position, skin.clone()));
        }
        scene.nodes.push(node);
    }

    for (position, skin) in skinned {
        let joints: Vec<usize> = skin
            .joints
            .iter()
            .filter_map(|joint| index_of.get(joint).copied())
            .collect();
        if joints.len() != skin.joints.len() {
            return Err("A skin points at a joint that is outside the model.".to_string());
        }
        let matrices = skin.inverse_bind_matrices.as_ref().clone();
        let existing = scene
            .skins
            .iter()
            .position(|held| held.joints == joints && held.inverse_bind_matrices == matrices);
        let index = match existing {
            Some(index) => index,
            None => {
                scene.skins.push(ExportSkin {
                    name: skin.name.clone(),
                    joints,
                    inverse_bind_matrices: matrices,
                    skeleton: None,
                });
                scene.skins.len() - 1
            }
        };
        scene.nodes[position].skin = Some(index);
    }

    scene.roots = viewer
        .model
        .roots
        .iter()
        .filter_map(|root| index_of.get(root).copied())
        .collect();

    for entity in &order {
        if let Some(player) = storage.get::<AnimationPlayer>(*entity) {
            scene.animations.extend(player.clips.iter().cloned());
        }
    }

    Ok(scene)
}

fn material_index(
    name: &str,
    materials: &MaterialRegistry,
    material_of: &mut HashMap<String, usize>,
    scene: &mut SceneExport,
    textures: &TextureSources,
    image_of: &mut HashMap<String, usize>,
) -> Option<usize> {
    if let Some(index) = material_of.get(name) {
        return Some(*index);
    }
    let material = registry_entry_by_name(&materials.registry, name)?.clone();
    for texture in texture_names(&material) {
        collect_image(&texture, textures, image_of, scene);
    }
    scene.materials.push(material);
    let index = scene.materials.len() - 1;
    material_of.insert(name.to_string(), index);
    Some(index)
}

fn texture_names(material: &nightshade_render::data::material::Material) -> Vec<String> {
    [
        &material.base_texture,
        &material.emissive_texture,
        &material.normal_texture,
        &material.metallic_roughness_texture,
        &material.occlusion_texture,
        &material.transmission_texture,
        &material.thickness_texture,
        &material.specular_texture,
        &material.specular_color_texture,
        &material.diffuse_transmission_texture,
        &material.diffuse_transmission_color_texture,
        &material.anisotropy_texture,
        &material.iridescence_texture,
        &material.iridescence_thickness_texture,
        &material.sheen_color_texture,
        &material.sheen_roughness_texture,
        &material.clearcoat_texture,
        &material.clearcoat_roughness_texture,
        &material.clearcoat_normal_texture,
    ]
    .into_iter()
    .flatten()
    .cloned()
    .collect()
}

fn collect_image(
    name: &str,
    textures: &TextureSources,
    image_of: &mut HashMap<String, usize>,
    scene: &mut SceneExport,
) {
    if image_of.contains_key(name) {
        return;
    }
    let Some(source) = textures.get(name) else {
        return;
    };
    let png = match &source.data {
        TextureSourceData::Png(bytes) => bytes.clone(),
        TextureSourceData::Rgba {
            rgba,
            width,
            height,
        } => {
            let Some(buffer) = image::RgbaImage::from_raw(*width, *height, rgba.clone()) else {
                return;
            };
            let mut encoded = std::io::Cursor::new(Vec::new());
            if image::DynamicImage::ImageRgba8(buffer)
                .write_to(&mut encoded, image::ImageFormat::Png)
                .is_err()
            {
                return;
            }
            encoded.into_inner()
        }
    };
    scene.images.push(ExportImage {
        name: name.to_string(),
        png,
    });
    image_of.insert(name.to_string(), scene.images.len() - 1);
}
