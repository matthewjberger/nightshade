use nightshade_graphics::prelude::MaterialRegistry;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::data::material::{AlphaMode, Material};

use crate::commands::history::push;
use crate::data::{MaterialField, TextureSlot};
use crate::queries::material::{fields_of, find_material, view_of};
use crate::resources::{Edit, MaterialView, ViewerResources};

pub fn set_material_field(stage: &mut crate::resources::Stage<'_>, field: MaterialField) {
    let viewer = &mut *stage.viewer;
    let before = viewer.selection.material_values.clone();
    let mut values = before.clone();
    let name = viewer.selection.material.clone();
    apply_to_view(&mut values, field.clone());
    if let Some(name) = name {
        write_field(stage.materials, &name, field);
        let viewer = &mut *stage.viewer;
        push(
            &mut viewer.history,
            Edit::Material {
                name,
                before,
                after: values.clone(),
            },
        );
    }
    stage.viewer.selection.material_values = values;
}

pub fn reset_material(stage: &mut crate::resources::Stage<'_>) {
    let (name, loaded, before) = {
        let viewer = &mut *stage.viewer;
        (
            viewer.selection.material.clone(),
            viewer.selection.material_loaded.clone(),
            viewer.selection.material_values.clone(),
        )
    };
    if let Some(name) = name {
        reset_to(stage.materials, &name, &loaded);
        let viewer = &mut *stage.viewer;
        push(
            &mut viewer.history,
            Edit::Material {
                name,
                before,
                after: loaded.clone(),
            },
        );
    }
    let viewer = &mut *stage.viewer;
    viewer.selection.material_values = loaded;
    viewer.view.detail_dirty = true;
}

pub fn read_selected(
    viewer: &mut ViewerResources,
    storage: &nightshade::ecs::Storage,
    registry: &MaterialRegistry,
) {
    let name = viewer
        .selection
        .selected
        .and_then(|entity| storage.get::<MaterialRef>(entity))
        .and_then(|reference| {
            nightshade_render::queries::material_registry::material_registry_name(
                registry,
                reference.id,
            )
        })
        .map(str::to_string);

    let values = name
        .as_ref()
        .and_then(|name| find_material(registry, name))
        .map(view_of);

    match values {
        Some(values) => {
            viewer.selection.material = name;
            viewer.selection.material_loaded = values.clone();
            viewer.selection.material_values = values;
        }
        None => {
            viewer.selection.material = None;
            viewer.selection.material_loaded = MaterialView::default();
            viewer.selection.material_values = MaterialView::default();
        }
    }
    viewer.view.detail_dirty = true;
}

fn texture_slot_mut(material: &mut Material, slot: TextureSlot) -> &mut Option<String> {
    match slot {
        TextureSlot::BaseColor => &mut material.base_texture,
        TextureSlot::MetallicRoughness => &mut material.metallic_roughness_texture,
        TextureSlot::Normal => &mut material.normal_texture,
        TextureSlot::Occlusion => &mut material.occlusion_texture,
        TextureSlot::Emissive => &mut material.emissive_texture,
        TextureSlot::Transmission => &mut material.transmission_texture,
        TextureSlot::Thickness => &mut material.thickness_texture,
        TextureSlot::Specular => &mut material.specular_texture,
    }
}

pub fn write_field(registry: &mut MaterialRegistry, name: &str, field: MaterialField) {
    nightshade_render::commands::material_registry::material_registry_mutate(
        registry,
        name,
        |material| apply_to_material(material, field),
    );
}

pub fn reset_to(registry: &mut MaterialRegistry, name: &str, loaded: &MaterialView) {
    nightshade_render::commands::material_registry::material_registry_mutate(
        registry,
        name,
        |material| {
            for field in fields_of(loaded) {
                apply_to_material(material, field);
            }
        },
    );
}

pub fn apply_to_view(view: &mut MaterialView, field: MaterialField) {
    match field {
        MaterialField::Texture(slot, name) => {
            view.textures.retain(|(held, _)| *held != slot);
            if let Some(name) = name {
                view.textures.push((slot, name));
                view.textures
                    .sort_by_key(|(slot, _)| TextureSlot::ALL.iter().position(|held| held == slot));
            }
        }
        MaterialField::BaseColor(value) => view.base_color = value,
        MaterialField::Metallic(value) => view.metallic = value,
        MaterialField::Roughness(value) => view.roughness = value,
        MaterialField::AlphaMode(value) => view.alpha_mode = value,
        MaterialField::AlphaCutoff(value) => view.alpha_cutoff = value,
        MaterialField::Unlit(value) => view.unlit = value,
        MaterialField::DoubleSided(value) => view.double_sided = value,
        MaterialField::Emissive(value) => view.emissive = value,
        MaterialField::EmissiveStrength(value) => view.emissive_strength = value,
        MaterialField::NormalScale(value) => view.normal_scale = value,
        MaterialField::OcclusionStrength(value) => view.occlusion_strength = value,
        MaterialField::Ior(value) => view.ior = value,
        MaterialField::Specular(value) => view.specular = value,
        MaterialField::Transmission(value) => view.transmission = value,
        MaterialField::Thickness(value) => view.thickness = value,
        MaterialField::Dispersion(value) => view.dispersion = value,
        MaterialField::Clearcoat(value) => view.clearcoat = value,
        MaterialField::ClearcoatRoughness(value) => view.clearcoat_roughness = value,
        MaterialField::SheenRoughness(value) => view.sheen_roughness = value,
        MaterialField::Iridescence(value) => view.iridescence = value,
        MaterialField::IridescenceIor(value) => view.iridescence_ior = value,
    }
}

fn apply_to_material(material: &mut Material, field: MaterialField) {
    match field {
        MaterialField::Texture(slot, name) => *texture_slot_mut(material, slot) = name,
        MaterialField::BaseColor(value) => material.base_color = value,
        MaterialField::Metallic(value) => material.metallic = value,
        MaterialField::Roughness(value) => material.roughness = value,
        MaterialField::AlphaMode(value) => {
            material.alpha_mode = match value {
                1 => AlphaMode::Mask,
                2 => AlphaMode::Blend,
                3 => AlphaMode::Dither,
                _ => AlphaMode::Opaque,
            }
        }
        MaterialField::AlphaCutoff(value) => material.alpha_cutoff = value,
        MaterialField::Unlit(value) => material.unlit = value,
        MaterialField::DoubleSided(value) => material.double_sided = value,
        MaterialField::Emissive(value) => material.emissive_factor = value,
        MaterialField::EmissiveStrength(value) => material.emissive_strength = value,
        MaterialField::NormalScale(value) => material.normal_scale = value,
        MaterialField::OcclusionStrength(value) => material.occlusion_strength = value,
        MaterialField::Ior(value) => material.ior = value,
        MaterialField::Specular(value) => material.specular_factor = value,
        MaterialField::Transmission(value) => material.transmission_factor = value,
        MaterialField::Thickness(value) => material.thickness = value,
        MaterialField::Dispersion(value) => material.dispersion = value,
        MaterialField::Clearcoat(value) => material.clearcoat_factor = value,
        MaterialField::ClearcoatRoughness(value) => material.clearcoat_roughness_factor = value,
        MaterialField::SheenRoughness(value) => material.sheen_roughness_factor = value,
        MaterialField::Iridescence(value) => material.iridescence_factor = value,
        MaterialField::IridescenceIor(value) => material.iridescence_ior = value,
    }
}
