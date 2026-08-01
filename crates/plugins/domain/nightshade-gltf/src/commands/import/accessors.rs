use super::*;

pub(super) type VariantMappings = Vec<(Vec<u32>, u32)>;
pub(super) type MeshVariantMap = HashMap<usize, Vec<VariantMappings>>;

pub(super) fn build_prefab_material_variants(
    mappings: &[(Vec<u32>, u32)],
    materials: &[Material],
    variant_names: &[String],
) -> Vec<nightshade_prefab::contract::PrefabMaterialVariant> {
    let mut out = Vec::with_capacity(mappings.len());
    for (variant_indices, material_index) in mappings {
        if let Some(material) = materials.get(*material_index as usize) {
            let names: Vec<String> = variant_indices
                .iter()
                .filter_map(|index| variant_names.get(*index as usize).cloned())
                .collect();
            if names.is_empty() {
                continue;
            }
            out.push(nightshade_prefab::contract::PrefabMaterialVariant {
                variant_names: names,
                material: material.clone(),
            });
        }
    }
    out
}

pub(super) fn read_accessor_floats(
    doc: &Document,
    buffers: &[buffer::Data],
    accessor_index: usize,
    components_per_element: usize,
) -> Option<Vec<f32>> {
    let accessor = doc.accessors().nth(accessor_index)?;
    if accessor.data_type() != gltf::accessor::DataType::F32 {
        tracing::warn!(
            "EXT_mesh_gpu_instancing: quantized component types are not supported (accessor {})",
            accessor_index
        );
        return None;
    }
    let view = accessor.view()?;
    let buffer_data = buffers.get(view.buffer().index())?;
    let buffer_bytes = &buffer_data.0;
    let element_bytes = components_per_element * 4;
    let stride = view.stride().unwrap_or(element_bytes);
    let start = view.offset() + accessor.offset();
    let mut out = Vec::with_capacity(accessor.count() * components_per_element);
    for element_index in 0..accessor.count() {
        let element_offset = start + element_index * stride;
        if element_offset + element_bytes > buffer_bytes.len() {
            return None;
        }
        for component_index in 0..components_per_element {
            let byte_offset = element_offset + component_index * 4;
            let bytes = &buffer_bytes[byte_offset..byte_offset + 4];
            out.push(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]));
        }
    }
    Some(out)
}

pub(super) fn read_gpu_instancing_transforms(
    doc: &Document,
    buffers: &[buffer::Data],
    node: &Node,
) -> Vec<LocalTransform> {
    let extension = match node.extension_value("EXT_mesh_gpu_instancing") {
        Some(value) => value,
        None => return Vec::new(),
    };
    let attributes = match extension.get("attributes").and_then(|v| v.as_object()) {
        Some(map) => map,
        None => return Vec::new(),
    };

    let translation_accessor = attributes
        .get("TRANSLATION")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);
    let rotation_accessor = attributes
        .get("ROTATION")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);
    let scale_accessor = attributes
        .get("SCALE")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);

    let translations =
        translation_accessor.and_then(|index| read_accessor_floats(doc, buffers, index, 3));
    let rotations =
        rotation_accessor.and_then(|index| read_accessor_floats(doc, buffers, index, 4));
    let scales = scale_accessor.and_then(|index| read_accessor_floats(doc, buffers, index, 3));

    let instance_count = translations
        .as_ref()
        .map(|values| values.len() / 3)
        .or_else(|| rotations.as_ref().map(|values| values.len() / 4))
        .or_else(|| scales.as_ref().map(|values| values.len() / 3))
        .unwrap_or(0);

    if instance_count == 0 {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(instance_count);
    for instance_index in 0..instance_count {
        let translation = translations
            .as_ref()
            .map(|values| {
                Vec3::new(
                    values[instance_index * 3],
                    values[instance_index * 3 + 1],
                    values[instance_index * 3 + 2],
                )
            })
            .unwrap_or_else(Vec3::zeros);
        let rotation = rotations
            .as_ref()
            .map(|values| {
                Quat::new(
                    values[instance_index * 4 + 3],
                    values[instance_index * 4],
                    values[instance_index * 4 + 1],
                    values[instance_index * 4 + 2],
                )
            })
            .unwrap_or_else(Quat::identity);
        let scale = scales
            .as_ref()
            .map(|values| {
                Vec3::new(
                    values[instance_index * 3],
                    values[instance_index * 3 + 1],
                    values[instance_index * 3 + 2],
                )
            })
            .unwrap_or_else(|| Vec3::new(1.0, 1.0, 1.0));
        out.push(LocalTransform {
            translation,
            rotation,
            scale,
        });
    }
    out
}
