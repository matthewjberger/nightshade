use std::collections::{HashMap, HashSet};

use crate::data::generational_registry::*;
use crate::data::material::{Material, MaterialTextureIds};

#[derive(Clone)]
pub struct MaterialRegistry {
    pub registry: GenerationalRegistry<Material>,
    pub texture_ids: Vec<MaterialTextureIds>,
    pub texture_name_to_materials: HashMap<String, HashSet<u32>>,
    pub pending_resolve: Vec<u32>,
    pub content_hash_to_index: HashMap<u64, u32>,
    pub version: u64,
    pub dirty_indices: Vec<u32>,
    pub names_version: u64,
}

impl Default for MaterialRegistry {
    fn default() -> Self {
        let mut instance = Self {
            registry: GenerationalRegistry::default(),
            texture_ids: Vec::new(),
            texture_name_to_materials: HashMap::new(),
            pending_resolve: Vec::new(),
            content_hash_to_index: HashMap::new(),
            version: 0,
            dirty_indices: Vec::new(),
            names_version: 0,
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
            let (index, _) = registry_insert(&mut instance.registry, name.to_string(), material);
            registry_set_protected(&mut instance.registry, index, true);
        }

        instance
    }
}
