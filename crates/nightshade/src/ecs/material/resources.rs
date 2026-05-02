//! Material registry resource.

use crate::ecs::asset_id::{MaterialId, TextureId};
use crate::ecs::generational_registry::GenerationalRegistry;
use crate::ecs::material::components::{Material, MaterialTextureIds};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Global registry of named materials.
///
/// Materials are stored by name and assigned generational IDs for safe references.
/// Includes default color materials (Red, Green, Blue, etc.) that cannot be removed.
#[derive(Clone)]
pub struct MaterialRegistry {
    /// The underlying generational storage.
    pub registry: GenerationalRegistry<Material>,
    /// Resolved per-role [`TextureId`]s parallel to `registry.entries`.
    pub texture_ids: Vec<MaterialTextureIds>,
    /// Reverse index: for each texture name referenced by any material, the
    /// material indices that reference it. Used to re-resolve only the
    /// materials affected by a newly-uploaded texture.
    pub texture_name_to_materials: HashMap<String, Vec<u32>>,
    /// Indices of built-in materials that should not be garbage collected.
    pub protected_indices: Vec<u32>,
    /// Maps content hash to material name for deduplication.
    pub content_hash_to_name: HashMap<u64, String>,
}

impl Default for MaterialRegistry {
    fn default() -> Self {
        let mut instance = Self {
            registry: GenerationalRegistry::new(),
            texture_ids: Vec::new(),
            texture_name_to_materials: HashMap::new(),
            protected_indices: Vec::new(),
            content_hash_to_name: HashMap::new(),
        };

        let default_materials = [
            (
                "Default",
                Material {
                    base_color: [0.7, 0.7, 0.7, 1.0],
                    ..Default::default()
                },
            ),
            (
                "Red",
                Material {
                    base_color: [1.0, 0.3, 0.3, 1.0],
                    ..Default::default()
                },
            ),
            (
                "Green",
                Material {
                    base_color: [0.3, 1.0, 0.3, 1.0],
                    ..Default::default()
                },
            ),
            (
                "Blue",
                Material {
                    base_color: [0.3, 0.3, 1.0, 1.0],
                    ..Default::default()
                },
            ),
            (
                "Yellow",
                Material {
                    base_color: [1.0, 1.0, 0.3, 1.0],
                    ..Default::default()
                },
            ),
            (
                "Magenta",
                Material {
                    base_color: [1.0, 0.3, 1.0, 1.0],
                    ..Default::default()
                },
            ),
            (
                "Cyan",
                Material {
                    base_color: [0.3, 1.0, 1.0, 1.0],
                    ..Default::default()
                },
            ),
            (
                "White",
                Material {
                    base_color: [1.0, 1.0, 1.0, 1.0],
                    ..Default::default()
                },
            ),
            (
                "Black",
                Material {
                    base_color: [0.1, 0.1, 0.1, 1.0],
                    ..Default::default()
                },
            ),
        ];

        for (name, material) in default_materials {
            let (index, _generation) = instance.registry.insert(name.to_string(), material);
            instance.protected_indices.push(index);
        }

        instance
    }
}

/// Inserts a material into the registry, returning its ID.
///
/// The parallel `texture_ids` slot is grown to match and reset to defaults.
/// Texture name references are added to `texture_name_to_materials` so the
/// drain can resolve only the materials affected by a newly-uploaded texture.
pub fn material_registry_insert(
    registry: &mut MaterialRegistry,
    name: String,
    material: Material,
) -> MaterialId {
    let referenced_textures = collect_material_texture_names(&material);
    let (index, generation) = registry.registry.insert(name, material);
    let slot = index as usize;
    if slot >= registry.texture_ids.len() {
        registry
            .texture_ids
            .resize_with(slot + 1, MaterialTextureIds::default);
    } else {
        registry.texture_ids[slot] = MaterialTextureIds::default();
    }
    for texture_name in referenced_textures {
        registry
            .texture_name_to_materials
            .entry(texture_name)
            .or_default()
            .push(index);
    }
    MaterialId::new(index, generation)
}

fn collect_material_texture_names(material: &Material) -> Vec<String> {
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
    .filter_map(|opt| opt.clone())
    .collect()
}

fn resolve_one_material(
    registry: &mut MaterialRegistry,
    slot: usize,
    texture_cache: &crate::render::wgpu::texture_cache::TextureCache,
) {
    let lookup = |opt: &Option<String>| -> Option<TextureId> {
        opt.as_deref()
            .and_then(|name| texture_cache.registry.lookup_index(name))
            .map(|(index, generation)| TextureId::new(index, generation))
    };
    let Some(material) = registry
        .registry
        .entries
        .get(slot)
        .and_then(|entry| entry.as_ref())
    else {
        if let Some(stale) = registry.texture_ids.get_mut(slot) {
            *stale = MaterialTextureIds::default();
        }
        return;
    };
    let resolved = MaterialTextureIds {
        base: lookup(&material.base_texture),
        emissive: lookup(&material.emissive_texture),
        normal: lookup(&material.normal_texture),
        metallic_roughness: lookup(&material.metallic_roughness_texture),
        occlusion: lookup(&material.occlusion_texture),
        transmission: lookup(&material.transmission_texture),
        thickness: lookup(&material.thickness_texture),
        specular: lookup(&material.specular_texture),
        specular_color: lookup(&material.specular_color_texture),
        diffuse_transmission: lookup(&material.diffuse_transmission_texture),
        diffuse_transmission_color: lookup(&material.diffuse_transmission_color_texture),
        anisotropy: lookup(&material.anisotropy_texture),
        iridescence: lookup(&material.iridescence_texture),
        iridescence_thickness: lookup(&material.iridescence_thickness_texture),
        sheen_color: lookup(&material.sheen_color_texture),
        sheen_roughness: lookup(&material.sheen_roughness_texture),
        clearcoat: lookup(&material.clearcoat_texture),
        clearcoat_roughness: lookup(&material.clearcoat_roughness_texture),
        clearcoat_normal: lookup(&material.clearcoat_normal_texture),
    };
    if slot >= registry.texture_ids.len() {
        registry
            .texture_ids
            .resize_with(slot + 1, MaterialTextureIds::default);
    }
    registry.texture_ids[slot] = resolved;
}

/// Resolves only the materials that reference any of the supplied texture ids,
/// using the reverse index built at insert time.
pub fn material_registry_resolve_uploaded_textures(
    registry: &mut MaterialRegistry,
    texture_cache: &crate::render::wgpu::texture_cache::TextureCache,
    uploaded: &[TextureId],
) {
    let mut dirty: std::collections::HashSet<u32> = std::collections::HashSet::new();
    for id in uploaded {
        let Some(name) = texture_cache.registry.name_for(id.index) else {
            continue;
        };
        if let Some(materials) = registry.texture_name_to_materials.get(name) {
            dirty.extend(materials.iter().copied());
        }
    }
    for index in dirty {
        resolve_one_material(registry, index as usize, texture_cache);
    }
}

/// Resolves [`TextureId`]s for every material in the registry by looking up
/// the [`Material`]'s texture name strings against the supplied texture cache.
///
/// Idempotent and cheap when no texture names are present. Run after a
/// streaming batch lands so the hot-path material rebuild can index by id.
pub fn material_registry_resolve_textures(
    registry: &mut MaterialRegistry,
    texture_cache: &crate::render::wgpu::texture_cache::TextureCache,
) {
    let lookup = |opt: &Option<String>| -> Option<TextureId> {
        opt.as_deref()
            .and_then(|name| texture_cache.registry.lookup_index(name))
            .map(|(index, generation)| TextureId::new(index, generation))
    };
    if registry.texture_ids.len() < registry.registry.entries.len() {
        registry
            .texture_ids
            .resize_with(registry.registry.entries.len(), MaterialTextureIds::default);
    }
    for (slot, entry) in registry.registry.entries.iter().enumerate() {
        let Some(material) = entry else {
            registry.texture_ids[slot] = MaterialTextureIds::default();
            continue;
        };
        let resolved = MaterialTextureIds {
            base: lookup(&material.base_texture),
            emissive: lookup(&material.emissive_texture),
            normal: lookup(&material.normal_texture),
            metallic_roughness: lookup(&material.metallic_roughness_texture),
            occlusion: lookup(&material.occlusion_texture),
            transmission: lookup(&material.transmission_texture),
            thickness: lookup(&material.thickness_texture),
            specular: lookup(&material.specular_texture),
            specular_color: lookup(&material.specular_color_texture),
            diffuse_transmission: lookup(&material.diffuse_transmission_texture),
            diffuse_transmission_color: lookup(&material.diffuse_transmission_color_texture),
            anisotropy: lookup(&material.anisotropy_texture),
            iridescence: lookup(&material.iridescence_texture),
            iridescence_thickness: lookup(&material.iridescence_thickness_texture),
            sheen_color: lookup(&material.sheen_color_texture),
            sheen_roughness: lookup(&material.sheen_roughness_texture),
            clearcoat: lookup(&material.clearcoat_texture),
            clearcoat_roughness: lookup(&material.clearcoat_roughness_texture),
            clearcoat_normal: lookup(&material.clearcoat_normal_texture),
        };
        registry.texture_ids[slot] = resolved;
    }
}

/// Looks up a material ID by name.
pub fn material_registry_lookup_id(registry: &MaterialRegistry, name: &str) -> Option<MaterialId> {
    registry
        .registry
        .lookup_index(name)
        .map(|(index, generation)| MaterialId::new(index, generation))
}

/// Removes materials with zero references (except protected materials).
pub fn material_registry_remove_unused(registry: &mut MaterialRegistry) -> Vec<String> {
    let mut removed = Vec::new();

    for index in 0..registry.registry.entries.len() {
        if registry.protected_indices.contains(&(index as u32)) {
            continue;
        }

        if registry.registry.reference_counts[index] == 0
            && registry.registry.entries[index].is_some()
        {
            if let Some(name) = registry.registry.index_to_name[index].take() {
                registry.registry.name_to_index.remove(&name);
                removed.push(name);
            }
            registry.registry.entries[index] = None;
            registry.registry.free_indices.push(index as u32);
            if let Some(slot) = registry.texture_ids.get_mut(index) {
                *slot = MaterialTextureIds::default();
            }
        }
    }

    removed
}

/// Iterates over all materials as (name, material) pairs.
pub fn material_registry_iter(
    registry: &MaterialRegistry,
) -> impl Iterator<Item = (&String, &Material)> {
    registry
        .registry
        .index_to_name
        .iter()
        .enumerate()
        .filter_map(|(index, name_opt)| {
            name_opt.as_ref().and_then(|name| {
                registry.registry.entries[index]
                    .as_ref()
                    .map(|material| (name, material))
            })
        })
}

fn hash_texture_transform(
    transform: &crate::ecs::material::components::TextureTransform,
    hasher: &mut std::collections::hash_map::DefaultHasher,
) {
    for component in &transform.offset {
        component.to_bits().hash(hasher);
    }
    transform.rotation.to_bits().hash(hasher);
    for component in &transform.scale {
        component.to_bits().hash(hasher);
    }
    transform.uv_set.hash(hasher);
}

fn compute_material_content_hash(material: &Material) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for component in &material.base_color {
        component.to_bits().hash(&mut hasher);
    }
    for component in &material.emissive_factor {
        component.to_bits().hash(&mut hasher);
    }
    material.alpha_mode.hash(&mut hasher);
    material.alpha_cutoff.to_bits().hash(&mut hasher);
    material.base_texture.hash(&mut hasher);
    hash_texture_transform(&material.base_texture_transform, &mut hasher);
    material.emissive_texture.hash(&mut hasher);
    hash_texture_transform(&material.emissive_texture_transform, &mut hasher);
    material.normal_texture.hash(&mut hasher);
    hash_texture_transform(&material.normal_texture_transform, &mut hasher);
    material.normal_scale.to_bits().hash(&mut hasher);
    material.normal_map_flip_y.hash(&mut hasher);
    material.normal_map_two_component.hash(&mut hasher);
    material.metallic_roughness_texture.hash(&mut hasher);
    hash_texture_transform(&material.metallic_roughness_texture_transform, &mut hasher);
    material.occlusion_texture.hash(&mut hasher);
    hash_texture_transform(&material.occlusion_texture_transform, &mut hasher);
    material.occlusion_strength.to_bits().hash(&mut hasher);
    material.roughness.to_bits().hash(&mut hasher);
    material.metallic.to_bits().hash(&mut hasher);
    material.unlit.hash(&mut hasher);
    material.double_sided.hash(&mut hasher);
    material.transmission_factor.to_bits().hash(&mut hasher);
    material.transmission_texture.hash(&mut hasher);
    hash_texture_transform(&material.transmission_texture_transform, &mut hasher);
    material.thickness.to_bits().hash(&mut hasher);
    material.thickness_texture.hash(&mut hasher);
    hash_texture_transform(&material.thickness_texture_transform, &mut hasher);
    for component in &material.attenuation_color {
        component.to_bits().hash(&mut hasher);
    }
    material.attenuation_distance.to_bits().hash(&mut hasher);
    material.ior.to_bits().hash(&mut hasher);
    material.specular_factor.to_bits().hash(&mut hasher);
    for component in &material.specular_color_factor {
        component.to_bits().hash(&mut hasher);
    }
    material.specular_texture.hash(&mut hasher);
    hash_texture_transform(&material.specular_texture_transform, &mut hasher);
    material.specular_color_texture.hash(&mut hasher);
    hash_texture_transform(&material.specular_color_texture_transform, &mut hasher);
    material.emissive_strength.to_bits().hash(&mut hasher);
    hasher.finish()
}

pub fn material_registry_find_or_insert(
    registry: &mut MaterialRegistry,
    fallback_name: String,
    material: Material,
) -> String {
    let content_hash = compute_material_content_hash(&material);
    if let Some(existing_name) = registry.content_hash_to_name.get(&content_hash) {
        return existing_name.clone();
    }
    registry
        .content_hash_to_name
        .insert(content_hash, fallback_name.clone());
    material_registry_insert(registry, fallback_name.clone(), material);
    fallback_name
}
