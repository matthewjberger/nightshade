use nightshade_graphics::prelude::MaterialRegistry;
use nightshade_render::data::material::{AlphaMode, Material};

use crate::data::{MaterialField, TextureSlot};
use crate::resources::MaterialView;

#[cfg(not(target_arch = "wasm32"))]
pub fn view_by_name(registry: &MaterialRegistry, name: &str) -> Option<MaterialView> {
    find_material(registry, name).map(view_of)
}

pub fn find_material<'a>(registry: &'a MaterialRegistry, name: &str) -> Option<&'a Material> {
    nightshade_render::queries::material_registry::material_registry_iter(registry)
        .find(|(known, _)| known.as_str() == name)
        .map(|(_, material)| material)
}

pub fn view_of(material: &Material) -> MaterialView {
    MaterialView {
        base_color: material.base_color,
        metallic: material.metallic,
        roughness: material.roughness,
        alpha_mode: match material.alpha_mode {
            AlphaMode::Opaque => 0,
            AlphaMode::Mask => 1,
            AlphaMode::Blend => 2,
            AlphaMode::Dither => 3,
        },
        alpha_cutoff: material.alpha_cutoff,
        unlit: material.unlit,
        double_sided: material.double_sided,
        emissive: material.emissive_factor,
        emissive_strength: material.emissive_strength,
        normal_scale: material.normal_scale,
        occlusion_strength: material.occlusion_strength,
        ior: material.ior,
        specular: material.specular_factor,
        transmission: material.transmission_factor,
        thickness: material.thickness,
        dispersion: material.dispersion,
        clearcoat: material.clearcoat_factor,
        clearcoat_roughness: material.clearcoat_roughness_factor,
        sheen_roughness: material.sheen_roughness_factor,
        iridescence: material.iridescence_factor,
        iridescence_ior: material.iridescence_ior,
        textures: textures_of(material),
    }
}

fn textures_of(material: &Material) -> Vec<(TextureSlot, String)> {
    TextureSlot::ALL
        .into_iter()
        .filter_map(|slot| {
            texture_slot(material, slot)
                .clone()
                .map(|name| (slot, name))
        })
        .collect()
}

pub fn texture_slot(material: &Material, slot: TextureSlot) -> &Option<String> {
    match slot {
        TextureSlot::BaseColor => &material.base_texture,
        TextureSlot::MetallicRoughness => &material.metallic_roughness_texture,
        TextureSlot::Normal => &material.normal_texture,
        TextureSlot::Occlusion => &material.occlusion_texture,
        TextureSlot::Emissive => &material.emissive_texture,
        TextureSlot::Transmission => &material.transmission_texture,
        TextureSlot::Thickness => &material.thickness_texture,
        TextureSlot::Specular => &material.specular_texture,
    }
}

pub fn fields_of(view: &MaterialView) -> Vec<MaterialField> {
    let mut fields: Vec<MaterialField> = TextureSlot::ALL
        .into_iter()
        .map(|slot| {
            let name = view
                .textures
                .iter()
                .find(|(held, _)| *held == slot)
                .map(|(_, name)| name.clone());
            MaterialField::Texture(slot, name)
        })
        .collect();
    fields.extend([
        MaterialField::BaseColor(view.base_color),
        MaterialField::Metallic(view.metallic),
        MaterialField::Roughness(view.roughness),
        MaterialField::AlphaMode(view.alpha_mode),
        MaterialField::AlphaCutoff(view.alpha_cutoff),
        MaterialField::Unlit(view.unlit),
        MaterialField::DoubleSided(view.double_sided),
        MaterialField::Emissive(view.emissive),
        MaterialField::EmissiveStrength(view.emissive_strength),
        MaterialField::NormalScale(view.normal_scale),
        MaterialField::OcclusionStrength(view.occlusion_strength),
        MaterialField::Ior(view.ior),
        MaterialField::Specular(view.specular),
        MaterialField::Transmission(view.transmission),
        MaterialField::Thickness(view.thickness),
        MaterialField::Dispersion(view.dispersion),
        MaterialField::Clearcoat(view.clearcoat),
        MaterialField::ClearcoatRoughness(view.clearcoat_roughness),
        MaterialField::SheenRoughness(view.sheen_roughness),
        MaterialField::Iridescence(view.iridescence),
        MaterialField::IridescenceIor(view.iridescence_ior),
    ]);
    fields
}
