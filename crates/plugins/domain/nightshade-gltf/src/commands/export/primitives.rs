use nightshade_render::data::geometry::Mesh;
use serde_json::{Map, Value, json};

use super::buffer::{ARRAY_BUFFER, BufferBuilder};

pub fn mesh_json(
    mesh: &Mesh,
    name: &str,
    material: Option<usize>,
    builder: &mut BufferBuilder,
) -> Value {
    let mut attributes = Map::new();

    let positions: Vec<f32> = mesh
        .vertices
        .iter()
        .flat_map(|vertex| vertex.position)
        .collect();
    attributes.insert(
        "POSITION".to_string(),
        json!(builder.floats(&positions, "VEC3", Some(ARRAY_BUFFER))),
    );

    let normals: Vec<f32> = mesh
        .vertices
        .iter()
        .flat_map(|vertex| vertex.normal)
        .collect();
    attributes.insert(
        "NORMAL".to_string(),
        json!(builder.floats(&normals, "VEC3", Some(ARRAY_BUFFER))),
    );

    let tangents: Vec<f32> = mesh
        .vertices
        .iter()
        .flat_map(|vertex| vertex.tangent)
        .collect();
    attributes.insert(
        "TANGENT".to_string(),
        json!(builder.floats(&tangents, "VEC4", Some(ARRAY_BUFFER))),
    );

    let coordinates: Vec<f32> = mesh
        .vertices
        .iter()
        .flat_map(|vertex| vertex.tex_coords)
        .collect();
    attributes.insert(
        "TEXCOORD_0".to_string(),
        json!(builder.floats(&coordinates, "VEC2", Some(ARRAY_BUFFER))),
    );

    if mesh
        .vertices
        .iter()
        .any(|vertex| vertex.tex_coords_1 != [0.0, 0.0])
    {
        let second: Vec<f32> = mesh
            .vertices
            .iter()
            .flat_map(|vertex| vertex.tex_coords_1)
            .collect();
        attributes.insert(
            "TEXCOORD_1".to_string(),
            json!(builder.floats(&second, "VEC2", Some(ARRAY_BUFFER))),
        );
    }

    if mesh
        .vertices
        .iter()
        .any(|vertex| vertex.color != [1.0, 1.0, 1.0, 1.0])
    {
        let colors: Vec<f32> = mesh
            .vertices
            .iter()
            .flat_map(|vertex| vertex.color)
            .collect();
        attributes.insert(
            "COLOR_0".to_string(),
            json!(builder.floats(&colors, "VEC4", Some(ARRAY_BUFFER))),
        );
    }

    if let Some(skin) = &mesh.skin_data {
        let joints: Vec<[u32; 4]> = skin
            .skinned_vertices
            .iter()
            .map(|vertex| vertex.joint_indices)
            .collect();
        attributes.insert("JOINTS_0".to_string(), json!(builder.joints(&joints)));
        let weights: Vec<f32> = skin
            .skinned_vertices
            .iter()
            .flat_map(|vertex| vertex.joint_weights)
            .collect();
        attributes.insert(
            "WEIGHTS_0".to_string(),
            json!(builder.floats(&weights, "VEC4", Some(ARRAY_BUFFER))),
        );
    }

    let mut primitive = json!({
        "attributes": Value::Object(attributes),
        "indices": builder.indices(&mesh.indices),
        "mode": 4,
    });
    if let Some(material) = material {
        primitive["material"] = json!(material);
    }

    if let Some(morph) = &mesh.morph_targets {
        let mut targets = Vec::new();
        for target in &morph.targets {
            let mut block = Map::new();
            let displacements: Vec<f32> = target
                .position_displacements
                .iter()
                .flatten()
                .copied()
                .collect();
            block.insert(
                "POSITION".to_string(),
                json!(builder.floats(&displacements, "VEC3", Some(ARRAY_BUFFER))),
            );
            if let Some(normals) = &target.normal_displacements {
                let values: Vec<f32> = normals.iter().flatten().copied().collect();
                block.insert(
                    "NORMAL".to_string(),
                    json!(builder.floats(&values, "VEC3", Some(ARRAY_BUFFER))),
                );
            }
            targets.push(Value::Object(block));
        }
        if !targets.is_empty() {
            primitive["targets"] = json!(targets);
        }
    }

    json!({
        "name": name,
        "primitives": [primitive],
    })
}
