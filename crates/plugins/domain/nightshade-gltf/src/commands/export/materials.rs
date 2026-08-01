use std::collections::HashMap;

use nightshade_render::data::material::{AlphaMode, Material, TextureTransform};
use serde_json::{Map, Value, json};

pub struct TextureTable {
    pub images: HashMap<String, usize>,
    pub textures: Vec<Value>,
    pub used: Vec<String>,
}

impl TextureTable {
    pub fn new(images: HashMap<String, usize>) -> Self {
        Self {
            images,
            textures: Vec::new(),
            used: Vec::new(),
        }
    }

    fn texture(&mut self, name: &str) -> Option<usize> {
        let image = *self.images.get(name)?;
        if let Some(index) = self
            .textures
            .iter()
            .position(|held| held["source"] == json!(image))
        {
            return Some(index);
        }
        self.textures.push(json!({ "source": image }));
        Some(self.textures.len() - 1)
    }

    fn reference(&mut self, name: &Option<String>, transform: &TextureTransform) -> Option<Value> {
        let name = name.as_ref()?;
        let index = self.texture(name)?;
        let mut reference = json!({ "index": index });
        if transform.uv_set != 0 {
            reference["texCoord"] = json!(transform.uv_set);
        }
        if transform.offset != [0.0, 0.0]
            || transform.scale != [1.0, 1.0]
            || transform.rotation != 0.0
        {
            let mut mapping = json!({
                "offset": transform.offset,
                "scale": transform.scale,
                "rotation": transform.rotation,
            });
            if transform.uv_set != 0 {
                mapping["texCoord"] = json!(transform.uv_set);
            }
            reference["extensions"] = json!({ "KHR_texture_transform": mapping });
            self.record("KHR_texture_transform");
        }
        Some(reference)
    }

    fn record(&mut self, extension: &str) {
        if !self.used.iter().any(|held| held == extension) {
            self.used.push(extension.to_string());
        }
    }
}

fn near(value: f32, target: f32) -> bool {
    (value - target).abs() < 1.0e-6
}

pub fn material_json(material: &Material, name: &str, table: &mut TextureTable) -> Value {
    let mut pbr = Map::new();
    pbr.insert("baseColorFactor".to_string(), json!(material.base_color));
    pbr.insert("metallicFactor".to_string(), json!(material.metallic));
    pbr.insert("roughnessFactor".to_string(), json!(material.roughness));
    if let Some(reference) =
        table.reference(&material.base_texture, &material.base_texture_transform)
    {
        pbr.insert("baseColorTexture".to_string(), reference);
    }
    if let Some(reference) = table.reference(
        &material.metallic_roughness_texture,
        &material.metallic_roughness_texture_transform,
    ) {
        pbr.insert("metallicRoughnessTexture".to_string(), reference);
    }

    let mut output = Map::new();
    output.insert("name".to_string(), json!(name));
    output.insert("pbrMetallicRoughness".to_string(), Value::Object(pbr));

    if material.emissive_factor != [0.0, 0.0, 0.0] {
        output.insert(
            "emissiveFactor".to_string(),
            json!(material.emissive_factor),
        );
    }
    if let Some(reference) = table.reference(
        &material.emissive_texture,
        &material.emissive_texture_transform,
    ) {
        output.insert("emissiveTexture".to_string(), reference);
    }
    if let Some(mut reference) =
        table.reference(&material.normal_texture, &material.normal_texture_transform)
    {
        if !near(material.normal_scale, 1.0) {
            reference["scale"] = json!(material.normal_scale);
        }
        output.insert("normalTexture".to_string(), reference);
    }
    if let Some(mut reference) = table.reference(
        &material.occlusion_texture,
        &material.occlusion_texture_transform,
    ) {
        if !near(material.occlusion_strength, 1.0) {
            reference["strength"] = json!(material.occlusion_strength);
        }
        output.insert("occlusionTexture".to_string(), reference);
    }

    match material.alpha_mode {
        AlphaMode::Opaque => {}
        AlphaMode::Mask | AlphaMode::Dither => {
            output.insert("alphaMode".to_string(), json!("MASK"));
            output.insert("alphaCutoff".to_string(), json!(material.alpha_cutoff));
        }
        AlphaMode::Blend => {
            output.insert("alphaMode".to_string(), json!("BLEND"));
        }
    }
    if material.double_sided {
        output.insert("doubleSided".to_string(), json!(true));
    }

    let mut extensions = Map::new();

    if material.unlit {
        extensions.insert("KHR_materials_unlit".to_string(), json!({}));
        table.record("KHR_materials_unlit");
    }
    if !near(material.emissive_strength, 1.0) {
        extensions.insert(
            "KHR_materials_emissive_strength".to_string(),
            json!({ "emissiveStrength": material.emissive_strength }),
        );
        table.record("KHR_materials_emissive_strength");
    }
    if !near(material.ior, 1.5) {
        extensions.insert(
            "KHR_materials_ior".to_string(),
            json!({ "ior": material.ior }),
        );
        table.record("KHR_materials_ior");
    }
    if !near(material.specular_factor, 1.0)
        || material.specular_color_factor != [1.0, 1.0, 1.0]
        || material.specular_texture.is_some()
        || material.specular_color_texture.is_some()
    {
        let mut block = json!({
            "specularFactor": material.specular_factor,
            "specularColorFactor": material.specular_color_factor,
        });
        if let Some(reference) = table.reference(
            &material.specular_texture,
            &material.specular_texture_transform,
        ) {
            block["specularTexture"] = reference;
        }
        if let Some(reference) = table.reference(
            &material.specular_color_texture,
            &material.specular_color_texture_transform,
        ) {
            block["specularColorTexture"] = reference;
        }
        extensions.insert("KHR_materials_specular".to_string(), block);
        table.record("KHR_materials_specular");
    }
    if material.transmission_factor > 0.0 || material.transmission_texture.is_some() {
        let mut block = json!({ "transmissionFactor": material.transmission_factor });
        if let Some(reference) = table.reference(
            &material.transmission_texture,
            &material.transmission_texture_transform,
        ) {
            block["transmissionTexture"] = reference;
        }
        extensions.insert("KHR_materials_transmission".to_string(), block);
        table.record("KHR_materials_transmission");
    }
    if material.thickness > 0.0 || material.thickness_texture.is_some() {
        let mut block = json!({
            "thicknessFactor": material.thickness,
            "attenuationColor": material.attenuation_color,
        });
        if material.attenuation_distance.is_finite() {
            block["attenuationDistance"] = json!(material.attenuation_distance);
        }
        if let Some(reference) = table.reference(
            &material.thickness_texture,
            &material.thickness_texture_transform,
        ) {
            block["thicknessTexture"] = reference;
        }
        extensions.insert("KHR_materials_volume".to_string(), block);
        table.record("KHR_materials_volume");
    }
    if material.dispersion > 0.0 {
        extensions.insert(
            "KHR_materials_dispersion".to_string(),
            json!({ "dispersion": material.dispersion }),
        );
        table.record("KHR_materials_dispersion");
    }
    if material.clearcoat_factor > 0.0 {
        let mut block = json!({
            "clearcoatFactor": material.clearcoat_factor,
            "clearcoatRoughnessFactor": material.clearcoat_roughness_factor,
        });
        if let Some(reference) = table.reference(
            &material.clearcoat_texture,
            &material.clearcoat_texture_transform,
        ) {
            block["clearcoatTexture"] = reference;
        }
        if let Some(reference) = table.reference(
            &material.clearcoat_roughness_texture,
            &material.clearcoat_roughness_texture_transform,
        ) {
            block["clearcoatRoughnessTexture"] = reference;
        }
        if let Some(mut reference) = table.reference(
            &material.clearcoat_normal_texture,
            &material.clearcoat_normal_texture_transform,
        ) {
            if !near(material.clearcoat_normal_scale, 1.0) {
                reference["scale"] = json!(material.clearcoat_normal_scale);
            }
            block["clearcoatNormalTexture"] = reference;
        }
        extensions.insert("KHR_materials_clearcoat".to_string(), block);
        table.record("KHR_materials_clearcoat");
    }
    if material.sheen_color_factor != [0.0, 0.0, 0.0] || material.sheen_roughness_factor > 0.0 {
        let mut block = json!({
            "sheenColorFactor": material.sheen_color_factor,
            "sheenRoughnessFactor": material.sheen_roughness_factor,
        });
        if let Some(reference) = table.reference(
            &material.sheen_color_texture,
            &material.sheen_color_texture_transform,
        ) {
            block["sheenColorTexture"] = reference;
        }
        if let Some(reference) = table.reference(
            &material.sheen_roughness_texture,
            &material.sheen_roughness_texture_transform,
        ) {
            block["sheenRoughnessTexture"] = reference;
        }
        extensions.insert("KHR_materials_sheen".to_string(), block);
        table.record("KHR_materials_sheen");
    }
    if material.iridescence_factor > 0.0 {
        let mut block = json!({
            "iridescenceFactor": material.iridescence_factor,
            "iridescenceIor": material.iridescence_ior,
            "iridescenceThicknessMinimum": material.iridescence_thickness_minimum,
            "iridescenceThicknessMaximum": material.iridescence_thickness_maximum,
        });
        if let Some(reference) = table.reference(
            &material.iridescence_texture,
            &material.iridescence_texture_transform,
        ) {
            block["iridescenceTexture"] = reference;
        }
        if let Some(reference) = table.reference(
            &material.iridescence_thickness_texture,
            &material.iridescence_thickness_texture_transform,
        ) {
            block["iridescenceThicknessTexture"] = reference;
        }
        extensions.insert("KHR_materials_iridescence".to_string(), block);
        table.record("KHR_materials_iridescence");
    }
    if material.anisotropy_strength > 0.0 {
        let mut block = json!({
            "anisotropyStrength": material.anisotropy_strength,
            "anisotropyRotation": material.anisotropy_rotation,
        });
        if let Some(reference) = table.reference(
            &material.anisotropy_texture,
            &material.anisotropy_texture_transform,
        ) {
            block["anisotropyTexture"] = reference;
        }
        extensions.insert("KHR_materials_anisotropy".to_string(), block);
        table.record("KHR_materials_anisotropy");
    }
    if material.diffuse_transmission_factor > 0.0 {
        let mut block = json!({
            "diffuseTransmissionFactor": material.diffuse_transmission_factor,
            "diffuseTransmissionColorFactor": material.diffuse_transmission_color_factor,
        });
        if let Some(reference) = table.reference(
            &material.diffuse_transmission_texture,
            &material.diffuse_transmission_texture_transform,
        ) {
            block["diffuseTransmissionTexture"] = reference;
        }
        if let Some(reference) = table.reference(
            &material.diffuse_transmission_color_texture,
            &material.diffuse_transmission_color_texture_transform,
        ) {
            block["diffuseTransmissionColorTexture"] = reference;
        }
        extensions.insert("KHR_materials_diffuse_transmission".to_string(), block);
        table.record("KHR_materials_diffuse_transmission");
    }

    if !extensions.is_empty() {
        output.insert("extensions".to_string(), Value::Object(extensions));
    }
    Value::Object(output)
}
