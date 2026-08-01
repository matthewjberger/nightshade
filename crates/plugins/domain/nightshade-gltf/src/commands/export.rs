mod animation;
mod buffer;
mod materials;
mod primitives;

use std::collections::HashMap;

use serde_json::{Map, Value, json};

use crate::data::{ExportCamera, ExportLightKind, SceneExport};

use animation::animation_json;
use buffer::BufferBuilder;
use materials::{TextureTable, material_json};
use primitives::mesh_json;

const MAGIC: u32 = 0x4654_6C67;
const JSON_CHUNK: u32 = 0x4E4F_534A;
const BINARY_CHUNK: u32 = 0x004E_4942;

pub fn export_glb(scene: &SceneExport) -> Result<Vec<u8>, String> {
    if scene.nodes.is_empty() {
        return Err("There is nothing in the scene to write.".to_string());
    }

    let mut builder = BufferBuilder::default();
    let mut table = TextureTable::new(scene.image_index());

    let materials: Vec<Value> = scene
        .materials
        .iter()
        .enumerate()
        .map(|(index, material)| material_json(material, &format!("material_{index}"), &mut table))
        .collect();

    let meshes: Vec<Value> = scene
        .meshes
        .iter()
        .map(|entry| mesh_json(&entry.mesh, &entry.name, entry.material, &mut builder))
        .collect();

    let skins: Vec<Value> = scene
        .skins
        .iter()
        .map(|skin| {
            let matrices: Vec<[f32; 16]> = skin
                .inverse_bind_matrices
                .iter()
                .map(|matrix| {
                    let mut values = [0.0f32; 16];
                    values.copy_from_slice(matrix.as_slice());
                    values
                })
                .collect();
            let mut block = json!({
                "joints": skin.joints,
                "inverseBindMatrices": builder.matrices(&matrices),
            });
            if let Some(name) = &skin.name {
                block["name"] = json!(name);
            }
            if let Some(skeleton) = skin.skeleton {
                block["skeleton"] = json!(skeleton);
            }
            block
        })
        .collect();

    let images: Vec<Value> = scene
        .images
        .iter()
        .map(|image| {
            json!({
                "name": image.name,
                "mimeType": "image/png",
                "bufferView": builder.image(&image.png),
            })
        })
        .collect();

    let mut cameras: Vec<Value> = Vec::new();
    let mut lights: Vec<Value> = Vec::new();
    let mut nodes: Vec<Value> = Vec::new();

    for node in &scene.nodes {
        let mut block = Map::new();
        if !node.name.is_empty() {
            block.insert("name".to_string(), json!(node.name));
        }
        if node.translation != nalgebra_glm::Vec3::new(0.0, 0.0, 0.0) {
            block.insert(
                "translation".to_string(),
                json!([node.translation.x, node.translation.y, node.translation.z]),
            );
        }
        let rotation = node.rotation;
        if rotation.i != 0.0 || rotation.j != 0.0 || rotation.k != 0.0 || rotation.w != 1.0 {
            block.insert(
                "rotation".to_string(),
                json!([rotation.i, rotation.j, rotation.k, rotation.w]),
            );
        }
        if node.scale != nalgebra_glm::Vec3::new(1.0, 1.0, 1.0) {
            block.insert(
                "scale".to_string(),
                json!([node.scale.x, node.scale.y, node.scale.z]),
            );
        }
        if !node.children.is_empty() {
            block.insert("children".to_string(), json!(node.children));
        }
        if let Some(mesh) = node.mesh {
            block.insert("mesh".to_string(), json!(mesh));
        }
        if let Some(skin) = node.skin {
            block.insert("skin".to_string(), json!(skin));
        }
        if !node.morph_weights.is_empty() {
            block.insert("weights".to_string(), json!(node.morph_weights));
        }
        if let Some(camera) = &node.camera {
            cameras.push(camera_json(camera));
            block.insert("camera".to_string(), json!(cameras.len() - 1));
        }
        if let Some(light) = &node.light {
            let mut entry = json!({
                "type": match light.kind {
                    ExportLightKind::Directional => "directional",
                    ExportLightKind::Point => "point",
                    ExportLightKind::Spot { .. } => "spot",
                },
                "color": light.color,
                "intensity": light.intensity,
            });
            if let Some(name) = &light.name {
                entry["name"] = json!(name);
            }
            if let Some(range) = light.range {
                entry["range"] = json!(range);
            }
            if let ExportLightKind::Spot {
                inner_cone,
                outer_cone,
            } = light.kind
            {
                entry["spot"] = json!({
                    "innerConeAngle": inner_cone,
                    "outerConeAngle": outer_cone,
                });
            }
            lights.push(entry);
            block.insert(
                "extensions".to_string(),
                json!({ "KHR_lights_punctual": { "light": lights.len() - 1 } }),
            );
        }
        nodes.push(Value::Object(block));
    }

    let named: HashMap<String, usize> = scene
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| !node.name.is_empty())
        .map(|(index, node)| (node.name.clone(), index))
        .collect();
    let fallback: Vec<usize> = (0..scene.nodes.len()).collect();

    let animations: Vec<Value> = scene
        .animations
        .iter()
        .filter_map(|clip| animation_json(clip, &named, &fallback, &mut builder))
        .collect();

    if !lights.is_empty() {
        table.used.push("KHR_lights_punctual".to_string());
    }

    let mut document = Map::new();
    document.insert(
        "asset".to_string(),
        json!({ "version": "2.0", "generator": "nightshade" }),
    );
    document.insert("scene".to_string(), json!(0));
    document.insert(
        "scenes".to_string(),
        json!([{ "name": scene.name, "nodes": scene.roots }]),
    );
    document.insert("nodes".to_string(), json!(nodes));
    if !meshes.is_empty() {
        document.insert("meshes".to_string(), json!(meshes));
    }
    if !materials.is_empty() {
        document.insert("materials".to_string(), json!(materials));
    }
    if !table.textures.is_empty() {
        document.insert("textures".to_string(), json!(table.textures));
    }
    if !images.is_empty() {
        document.insert("images".to_string(), json!(images));
    }
    if !skins.is_empty() {
        document.insert("skins".to_string(), json!(skins));
    }
    if !animations.is_empty() {
        document.insert("animations".to_string(), json!(animations));
    }
    if !cameras.is_empty() {
        document.insert("cameras".to_string(), json!(cameras));
    }
    if !lights.is_empty() {
        document.insert(
            "extensions".to_string(),
            json!({ "KHR_lights_punctual": { "lights": lights } }),
        );
    }
    if !table.used.is_empty() {
        document.insert("extensionsUsed".to_string(), json!(table.used));
    }
    if !builder.views.is_empty() {
        document.insert("bufferViews".to_string(), json!(builder.views));
        document.insert("accessors".to_string(), json!(builder.accessors));
        document.insert(
            "buffers".to_string(),
            json!([{ "byteLength": builder.bytes.len() }]),
        );
    }

    Ok(container(
        &serde_json::to_vec(&Value::Object(document)).map_err(|error| error.to_string())?,
        &builder.bytes,
    ))
}

fn camera_json(camera: &ExportCamera) -> Value {
    match camera {
        ExportCamera::Perspective {
            field_of_view,
            aspect_ratio,
            near,
            far,
        } => {
            let mut block = json!({ "yfov": field_of_view, "znear": near });
            if let Some(aspect_ratio) = aspect_ratio {
                block["aspectRatio"] = json!(aspect_ratio);
            }
            if let Some(far) = far {
                block["zfar"] = json!(far);
            }
            json!({ "type": "perspective", "perspective": block })
        }
        ExportCamera::Orthographic {
            horizontal_magnification,
            vertical_magnification,
            near,
            far,
        } => json!({
            "type": "orthographic",
            "orthographic": {
                "xmag": horizontal_magnification,
                "ymag": vertical_magnification,
                "znear": near,
                "zfar": far,
            },
        }),
    }
}

fn container(document: &[u8], binary: &[u8]) -> Vec<u8> {
    let mut json_chunk = document.to_vec();
    while !json_chunk.len().is_multiple_of(4) {
        json_chunk.push(b' ');
    }
    let mut binary_chunk = binary.to_vec();
    while !binary_chunk.len().is_multiple_of(4) {
        binary_chunk.push(0);
    }

    let mut length = 12 + 8 + json_chunk.len();
    if !binary_chunk.is_empty() {
        length += 8 + binary_chunk.len();
    }

    let mut output = Vec::with_capacity(length);
    output.extend_from_slice(&MAGIC.to_le_bytes());
    output.extend_from_slice(&2u32.to_le_bytes());
    output.extend_from_slice(&(length as u32).to_le_bytes());
    output.extend_from_slice(&(json_chunk.len() as u32).to_le_bytes());
    output.extend_from_slice(&JSON_CHUNK.to_le_bytes());
    output.extend_from_slice(&json_chunk);
    if !binary_chunk.is_empty() {
        output.extend_from_slice(&(binary_chunk.len() as u32).to_le_bytes());
        output.extend_from_slice(&BINARY_CHUNK.to_le_bytes());
        output.extend_from_slice(&binary_chunk);
    }
    output
}
