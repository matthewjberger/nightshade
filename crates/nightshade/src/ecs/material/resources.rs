//! Material registry resource.

use crate::ecs::asset_id::MaterialId;
use crate::ecs::generational_registry::GenerationalRegistry;
use crate::ecs::material::components::Material;
use std::collections::HashMap;

/// Global registry of named materials.
///
/// Materials are stored by name and assigned generational IDs for safe references.
/// Includes default color materials (Red, Green, Blue, etc.) that cannot be removed.
pub struct MaterialRegistry {
    /// The underlying generational storage.
    pub registry: GenerationalRegistry<Material>,
    /// Indices of built-in materials that should not be garbage collected.
    pub protected_indices: Vec<u32>,
}

impl Default for MaterialRegistry {
    fn default() -> Self {
        let mut instance = Self {
            registry: GenerationalRegistry::new(),
            protected_indices: Vec::new(),
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
pub fn material_registry_insert(
    registry: &mut MaterialRegistry,
    name: String,
    material: Material,
) -> MaterialId {
    let (index, generation) = registry.registry.insert(name, material);
    MaterialId { index, generation }
}

/// Looks up a material ID by name.
pub fn material_registry_lookup_id(registry: &MaterialRegistry, name: &str) -> Option<MaterialId> {
    registry
        .registry
        .lookup_index(name)
        .map(|(index, generation)| MaterialId { index, generation })
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

/// Returns an iterator over all material names.
pub fn material_registry_names(registry: &MaterialRegistry) -> impl Iterator<Item = &String> {
    registry.registry.name_to_index.keys()
}

/// Loads materials from a HashMap into the registry.
pub fn material_registry_load_from_map(
    registry: &mut MaterialRegistry,
    materials: HashMap<String, Material>,
) {
    for (name, material) in materials {
        material_registry_insert(registry, name, material);
    }
}

/// Exports all materials to a HashMap.
pub fn material_registry_to_map(registry: &MaterialRegistry) -> HashMap<String, Material> {
    material_registry_iter(registry)
        .map(|(name, material)| (name.clone(), material.clone()))
        .collect()
}
