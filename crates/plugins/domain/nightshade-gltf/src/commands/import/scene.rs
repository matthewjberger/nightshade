use super::*;

pub(super) fn process_gltf_data(
    doc: Document,
    buffers: &[buffer::Data],
    encoded_images: EncodedImages,
    namespace: &str,
) -> Result<ModelImport, Box<dyn std::error::Error>> {
    let mut meshes = HashMap::new();
    let mut texture_specs: TextureSpecMap = HashMap::new();
    let mut mesh_index_to_primitives: HashMap<usize, Vec<(String, usize)>> = HashMap::new();

    let mut mesh_to_skin: HashMap<usize, usize> = HashMap::new();
    let mut node_to_morph_target_count: HashMap<usize, usize> = HashMap::new();
    for node in doc.nodes() {
        if let (Some(mesh_index), Some(skin_index)) = (
            node.mesh().map(|m| m.index()),
            node.skin().map(|s| s.index()),
        ) {
            mesh_to_skin.insert(mesh_index, skin_index);
        }
        if let Some(mesh) = node.mesh() {
            let morph_count = mesh
                .primitives()
                .next()
                .map(|p| p.morph_targets().count())
                .unwrap_or(0);
            if morph_count > 0 {
                node_to_morph_target_count.insert(node.index(), morph_count);
            }
        }
    }

    let image_count = encoded_images.len();
    let mut image_index_to_id: HashMap<usize, SourceImageId> = HashMap::new();
    let mut encoded_image_arena: Vec<Vec<u8>> = Vec::with_capacity(image_count);
    let mut image_hashes: Vec<u64> = Vec::with_capacity(image_count);
    for (index, opt_bytes) in encoded_images.into_iter().enumerate() {
        if let Some(bytes) = opt_bytes {
            let id = SourceImageId(encoded_image_arena.len() as u32);
            let mut hasher = DefaultHasher::new();
            bytes.hash(&mut hasher);
            image_hashes.push(hasher.finish());
            encoded_image_arena.push(bytes);
            image_index_to_id.insert(index, id);
        }
    }

    let (materials, gltf_material_index_to_deduplicated_index) = build_materials(
        &doc,
        namespace,
        &image_hashes,
        &image_index_to_id,
        &mut texture_specs,
    );

    let document_variant_names: Vec<String> = doc
        .as_json()
        .extensions
        .as_ref()
        .and_then(|exts| exts.others.get("KHR_materials_variants"))
        .and_then(|v| v.get("variants"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|entry| entry.get("name").and_then(|n| n.as_str()))
                .map(|name| name.to_string())
                .collect()
        })
        .unwrap_or_default();

    let mut mesh_index_to_primitive_variants: MeshVariantMap = HashMap::new();

    for mesh in doc.meshes() {
        let base_mesh_name = mesh
            .name()
            .unwrap_or(&format!("Mesh_{}", mesh.index()))
            .to_string();

        let mut primitive_names = Vec::new();
        let mut primitive_variants: Vec<Vec<(Vec<u32>, u32)>> = Vec::new();
        let default_morph_weights: Option<Vec<f32>> = mesh.weights().map(|w| w.to_vec());

        for (primitive_index, primitive) in mesh.primitives().enumerate() {
            if let Some(mut mesh_primitive) =
                load_gltf_primitive(&primitive, buffers, default_morph_weights.as_deref())
            {
                if let Some(&skin_index) = mesh_to_skin.get(&mesh.index())
                    && let Some(skin_data) = &mut mesh_primitive.skin_data
                {
                    skin_data.skin_index = Some(skin_index);
                }

                let base_primitive_name = if mesh.primitives().len() > 1 {
                    format!("{}_{}", base_mesh_name, primitive_index)
                } else {
                    base_mesh_name.clone()
                };
                let primitive_name = format!("{}::{}", namespace, base_primitive_name);

                let gltf_material_index = primitive.material().index().unwrap_or(0);
                let material_index = gltf_material_index_to_deduplicated_index
                    .get(&gltf_material_index)
                    .copied()
                    .unwrap_or(0);
                primitive_names.push((primitive_name.clone(), material_index));

                let mut mappings: Vec<(Vec<u32>, u32)> = Vec::new();
                if let Some(mappings_value) = primitive
                    .extension_value("KHR_materials_variants")
                    .and_then(|v| v.get("mappings"))
                    .and_then(|v| v.as_array())
                {
                    for entry in mappings_value {
                        let Some(material_idx) = entry.get("material").and_then(|v| v.as_u64())
                        else {
                            continue;
                        };
                        let Some(variant_arr) = entry.get("variants").and_then(|v| v.as_array())
                        else {
                            continue;
                        };
                        let variant_indices: Vec<u32> = variant_arr
                            .iter()
                            .filter_map(|i| i.as_u64().map(|n| n as u32))
                            .collect();
                        if variant_indices.is_empty() {
                            continue;
                        }
                        let dedup = gltf_material_index_to_deduplicated_index
                            .get(&(material_idx as usize))
                            .copied()
                            .unwrap_or(material_idx as usize);
                        mappings.push((variant_indices, dedup as u32));
                    }
                }
                primitive_variants.push(mappings);

                meshes.insert(primitive_name, mesh_primitive);
            }
        }

        if !primitive_names.is_empty() {
            mesh_index_to_primitives.insert(mesh.index(), primitive_names);
            mesh_index_to_primitive_variants.insert(mesh.index(), primitive_variants);
        }
    }

    let mut prefabs = Vec::new();

    let point_intensity_scale =
        point_light_intensity_scale_for_generator(doc.as_json().asset.generator.as_deref());

    let conversion_context = NodeConversionContext {
        doc: &doc,
        buffers,
        mesh_index_to_primitives: &mesh_index_to_primitives,
        mesh_index_to_primitive_variants: &mesh_index_to_primitive_variants,
        materials: &materials,
        document_variant_names: &document_variant_names,
        point_intensity_scale,
    };

    for scene in doc.scenes() {
        let scene_name = scene
            .name()
            .unwrap_or(&format!("Scene_{}", scene.index()))
            .to_string();

        let mut root_nodes = Vec::new();

        for node in scene.nodes() {
            convert_node_to_prefab_nodes(&conversion_context, &node, &mut root_nodes);
        }

        prefabs.push(Prefab {
            name: scene_name,
            root_nodes,
            material_variants: document_variant_names.clone(),
        });
    }

    let mut animations: Vec<AnimationClip> = doc
        .animations()
        .map(|animation| load_gltf_animation(&animation, buffers, &node_to_morph_target_count))
        .collect();
    for (index, clip) in animations.iter_mut().enumerate() {
        let mut hasher = DefaultHasher::new();
        namespace.hash(&mut hasher);
        index.hash(&mut hasher);
        clip.source = hasher.finish();
    }

    let skins: Vec<ModelSkin> = doc
        .skins()
        .map(|skin| load_gltf_skin(&skin, buffers))
        .collect();

    let mut node_to_skin: HashMap<usize, usize> = HashMap::new();
    for node in doc.nodes() {
        if let Some(skin) = node.skin() {
            node_to_skin.insert(node.index(), skin.index());
        }
    }

    let node_count = doc.nodes().count();

    let mut max_intensity: f32 = 1.0;
    if let Some(lights) = doc.lights() {
        for light in lights {
            max_intensity = max_intensity.max(light.intensity());
        }
    }
    for material in materials.iter() {
        max_intensity = max_intensity.max(material.emissive_strength);
    }
    let suggested_exposure = if max_intensity > 10.0 {
        (1.0 / max_intensity).clamp(1e-4, 1.0)
    } else {
        1.0
    };

    let texture_plan: Vec<TextureProductionPlan> = texture_specs
        .into_iter()
        .map(|(name, spec)| TextureProductionPlan {
            name,
            recipe: spec.recipe,
            usage: spec.usage,
            sampler: spec.sampler,
        })
        .collect();

    Ok(ModelImport {
        prefabs,
        meshes,
        materials,
        encoded_images: encoded_image_arena,
        texture_plan,
        animations,
        skins,
        node_to_skin,
        node_to_morph_target_count,
        node_count,
        suggested_exposure,
    })
}
