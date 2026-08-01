use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use crate::data::asset_id::{MaterialId, TextureId};
use crate::data::generational_registry::*;
use crate::data::material::{Material, MaterialTextureIds};
use crate::resources::material_registry::MaterialRegistry;

pub fn material_registry_insert(
    registry: &mut MaterialRegistry,
    name: String,
    material: Material,
) -> MaterialId {
    registry.version = registry.version.wrapping_add(1);
    if !registry.registry.name_to_index.contains_key(&name) {
        registry.names_version = registry.names_version.wrapping_add(1);
    }
    let new_texture_names = collect_material_texture_names(&material);
    if let Some(&existing_index) = registry.registry.name_to_index.get(&name)
        && let Some(existing_material) = registry.registry.entries[existing_index as usize].as_ref()
    {
        let previous_texture_names = collect_material_texture_names(existing_material);
        for previous_name in previous_texture_names {
            if let Some(materials) = registry.texture_name_to_materials.get_mut(&previous_name) {
                materials.remove(&existing_index);
                if materials.is_empty() {
                    registry.texture_name_to_materials.remove(&previous_name);
                }
            }
        }
    }
    let (index, generation) = registry_insert(&mut registry.registry, name, material);
    let slot = index as usize;
    if slot >= registry.texture_ids.len() {
        registry
            .texture_ids
            .resize_with(slot + 1, MaterialTextureIds::default);
    }
    for texture_name in new_texture_names {
        registry
            .texture_name_to_materials
            .entry(texture_name)
            .or_default()
            .insert(index);
    }
    registry.pending_resolve.push(index);
    registry.dirty_indices.push(index);
    MaterialId::new(index, generation)
}

fn collect_material_texture_names(material: &Material) -> Vec<String> {
    [
        &material.base_texture,
        &material.emissive_texture,
        &material.normal_texture,
        &material.detail_base_texture,
        &material.detail_normal_texture,
        &material.height_texture,
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
    texture_cache: &crate::resources::texture_cache::TextureCache,
) {
    let lookup = |opt: &Option<String>| -> Option<TextureId> {
        opt.as_deref().and_then(|name| {
            crate::queries::texture_cache::texture_cache_texture_id(texture_cache, name)
        })
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
        detail_base: lookup(&material.detail_base_texture),
        detail_normal: lookup(&material.detail_normal_texture),
        height: lookup(&material.height_texture),
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

pub fn material_registry_resolve_uploaded_textures(
    registry: &mut MaterialRegistry,
    texture_cache: &crate::resources::texture_cache::TextureCache,
    uploaded: &[TextureId],
) {
    registry.version = registry.version.wrapping_add(1);
    let mut dirty: HashSet<u32> = HashSet::new();
    for id in uploaded {
        let Some(name) =
            crate::queries::texture_cache::texture_cache_texture_name(texture_cache, id.index)
        else {
            continue;
        };
        if let Some(materials) = registry.texture_name_to_materials.get(name) {
            dirty.extend(materials.iter().copied());
        }
    }
    dirty.extend(registry.pending_resolve.drain(..));
    for index in dirty {
        resolve_one_material(registry, index as usize, texture_cache);
        registry.dirty_indices.push(index);
    }
}

fn remove_material_at(
    registry: &mut MaterialRegistry,
    index: usize,
    live: &std::collections::HashSet<u32>,
) -> Option<String> {
    let in_use = live.contains(&(index as u32));
    let protected = registry_is_protected(&registry.registry, index as u32);
    if protected
        || registry.registry.entries[index].is_none()
        || in_use
        || !registry.registry.observed[index]
    {
        return None;
    }
    registry.registry.observed[index] = false;

    let texture_names = registry.registry.entries[index]
        .as_ref()
        .map(collect_material_texture_names)
        .unwrap_or_default();
    let content_hash = registry.registry.entries[index]
        .as_ref()
        .map(compute_material_content_hash);
    for texture_name in texture_names {
        if let Some(materials) = registry.texture_name_to_materials.get_mut(&texture_name) {
            materials.remove(&(index as u32));
            if materials.is_empty() {
                registry.texture_name_to_materials.remove(&texture_name);
            }
        }
    }
    registry
        .pending_resolve
        .retain(|&pending_index| pending_index != index as u32);
    let name = registry.registry.index_to_name[index]
        .take()
        .inspect(|name| {
            registry.registry.name_to_index.remove(name);
            if let Some(hash) = content_hash
                && registry.content_hash_to_index.get(&hash) == Some(&(index as u32))
            {
                registry.content_hash_to_index.remove(&hash);
            }
        });
    registry.registry.entries[index] = None;
    registry.registry.free_indices.push(index as u32);
    if let Some(slot) = registry.texture_ids.get_mut(index) {
        *slot = MaterialTextureIds::default();
    }
    name
}

pub fn material_registry_reclaim_range(
    registry: &mut MaterialRegistry,
    live: &std::collections::HashSet<u32>,
    cursor: usize,
    budget: usize,
) -> usize {
    crate::data::generational_registry::registry_observe_indices(&mut registry.registry, live);
    let len = registry.registry.entries.len();
    if len == 0 || budget == 0 {
        return 0;
    }
    let mut index = if cursor >= len { 0 } else { cursor };
    for _ in 0..budget.min(len) {
        remove_material_at(registry, index, live);
        index += 1;
        if index >= len {
            index = 0;
        }
    }
    index
}

pub fn material_registry_mutate(
    registry: &mut MaterialRegistry,
    name: &str,
    mutate: impl FnOnce(&mut Material),
) -> bool {
    let Some(&index) = registry.registry.name_to_index.get(name) else {
        return false;
    };
    material_registry_mutate_index(registry, index, mutate)
}

pub fn material_registry_mutate_id(
    registry: &mut MaterialRegistry,
    id: MaterialId,
    mutate: impl FnOnce(&mut Material),
) -> bool {
    material_registry_mutate_index(registry, id.index, mutate)
}

fn material_registry_mutate_index(
    registry: &mut MaterialRegistry,
    index: u32,
    mutate: impl FnOnce(&mut Material),
) -> bool {
    let Some(material) = registry
        .registry
        .entries
        .get_mut(index as usize)
        .and_then(|entry| entry.as_mut())
    else {
        return false;
    };
    let previous_texture_names = collect_material_texture_names(material);
    mutate(material);
    let new_texture_names = collect_material_texture_names(material);
    if previous_texture_names != new_texture_names {
        for previous_name in previous_texture_names {
            if let Some(materials) = registry.texture_name_to_materials.get_mut(&previous_name) {
                materials.remove(&index);
                if materials.is_empty() {
                    registry.texture_name_to_materials.remove(&previous_name);
                }
            }
        }
        for texture_name in new_texture_names {
            registry
                .texture_name_to_materials
                .entry(texture_name)
                .or_default()
                .insert(index);
        }
        registry.pending_resolve.push(index);
    }
    registry.version = registry.version.wrapping_add(1);
    registry.dirty_indices.push(index);
    true
}

fn hash_texture_transform(
    transform: &crate::data::material::TextureTransform,
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
    material
        .blend_opaque_alpha_threshold
        .to_bits()
        .hash(&mut hasher);
    material.base_texture.hash(&mut hasher);
    hash_texture_transform(&material.base_texture_transform, &mut hasher);
    material.atlas_cells[0].to_bits().hash(&mut hasher);
    material.atlas_cells[1].to_bits().hash(&mut hasher);
    material.sky_light.hash(&mut hasher);
    material.vertex_occlusion.hash(&mut hasher);
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
    material
        .diffuse_transmission_factor
        .to_bits()
        .hash(&mut hasher);
    material.diffuse_transmission_texture.hash(&mut hasher);
    hash_texture_transform(
        &material.diffuse_transmission_texture_transform,
        &mut hasher,
    );
    for component in &material.diffuse_transmission_color_factor {
        component.to_bits().hash(&mut hasher);
    }
    material
        .diffuse_transmission_color_texture
        .hash(&mut hasher);
    hash_texture_transform(
        &material.diffuse_transmission_color_texture_transform,
        &mut hasher,
    );
    material.dispersion.to_bits().hash(&mut hasher);
    material.anisotropy_strength.to_bits().hash(&mut hasher);
    material.anisotropy_rotation.to_bits().hash(&mut hasher);
    material.anisotropy_texture.hash(&mut hasher);
    hash_texture_transform(&material.anisotropy_texture_transform, &mut hasher);
    material.iridescence_factor.to_bits().hash(&mut hasher);
    material.iridescence_texture.hash(&mut hasher);
    hash_texture_transform(&material.iridescence_texture_transform, &mut hasher);
    material.iridescence_ior.to_bits().hash(&mut hasher);
    material
        .iridescence_thickness_minimum
        .to_bits()
        .hash(&mut hasher);
    material
        .iridescence_thickness_maximum
        .to_bits()
        .hash(&mut hasher);
    material.iridescence_thickness_texture.hash(&mut hasher);
    hash_texture_transform(
        &material.iridescence_thickness_texture_transform,
        &mut hasher,
    );
    for component in &material.sheen_color_factor {
        component.to_bits().hash(&mut hasher);
    }
    material.sheen_color_texture.hash(&mut hasher);
    hash_texture_transform(&material.sheen_color_texture_transform, &mut hasher);
    material.sheen_roughness_factor.to_bits().hash(&mut hasher);
    material.sheen_roughness_texture.hash(&mut hasher);
    hash_texture_transform(&material.sheen_roughness_texture_transform, &mut hasher);
    material.clearcoat_factor.to_bits().hash(&mut hasher);
    material.clearcoat_texture.hash(&mut hasher);
    hash_texture_transform(&material.clearcoat_texture_transform, &mut hasher);
    material
        .clearcoat_roughness_factor
        .to_bits()
        .hash(&mut hasher);
    material.clearcoat_roughness_texture.hash(&mut hasher);
    hash_texture_transform(&material.clearcoat_roughness_texture_transform, &mut hasher);
    material.clearcoat_normal_texture.hash(&mut hasher);
    hash_texture_transform(&material.clearcoat_normal_texture_transform, &mut hasher);
    material.clearcoat_normal_scale.to_bits().hash(&mut hasher);
    hasher.finish()
}

pub fn material_registry_find_or_insert_hashed(
    registry: &mut MaterialRegistry,
    prefix: &str,
    material: Material,
) -> MaterialId {
    let name = format!(
        "{}_{:016x}",
        prefix,
        compute_material_content_hash(&material)
    );
    material_registry_find_or_insert(registry, name, material)
}

pub fn material_registry_find_or_insert(
    registry: &mut MaterialRegistry,
    fallback_name: String,
    material: Material,
) -> MaterialId {
    let content_hash = compute_material_content_hash(&material);
    if let Some(&existing_index) = registry.content_hash_to_index.get(&content_hash) {
        let still_valid = registry
            .registry
            .entries
            .get(existing_index as usize)
            .is_some_and(|entry| entry.is_some());
        if still_valid {
            return MaterialId::new(
                existing_index,
                registry.registry.generations[existing_index as usize],
            );
        }
        registry.content_hash_to_index.remove(&content_hash);
    }
    let id = material_registry_insert(registry, fallback_name, material);
    registry
        .content_hash_to_index
        .insert(content_hash, id.index);
    id
}

fn insert_named(
    registry: &mut MaterialRegistry,
    name: impl Into<String>,
    material: Material,
) -> MaterialId {
    let name = name.into();
    let held = registry.registry.name_to_index.contains_key(&name);
    let id = material_registry_insert(registry, name, material);
    if !held {
        registry_add_reference(&mut registry.registry, id.index);
    }
    id
}

pub fn insert_material(
    registry: &mut MaterialRegistry,
    name: impl Into<String>,
    material: Material,
) -> MaterialId {
    let id = insert_named(registry, name, material);
    registry_set_protected(&mut registry.registry, id.index, true);
    id
}

pub fn material_registry_release(registry: &mut MaterialRegistry, name: &str) {
    registry_release_name(&mut registry.registry, name);
}

pub fn set_material(
    registry: &mut MaterialRegistry,
    commands: &mut nightshade::ecs::system_param::Commands<'_>,
    entity: nightshade::ecs::Entity,
    name: impl Into<String>,
    material: Material,
) {
    let id = insert_named(registry, name, material);
    commands.set(entity, crate::components::material::MaterialRef::new(id));
}

pub fn queue_set_material(
    commands: &mut nightshade::ecs::system_param::Commands<'_>,
    entity: nightshade::ecs::Entity,
    name: impl Into<String>,
    material: Material,
) {
    let name = name.into();
    commands.queue(move |storage, resources| {
        let id = insert_named(resources.res_mut::<MaterialRegistry>(), name, material);
        storage.set(entity, crate::components::material::MaterialRef::new(id));
    });
}
