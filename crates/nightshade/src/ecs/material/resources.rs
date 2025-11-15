use crate::ecs::material::components::Material;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Default)]
pub struct MaterialCache {
    materials: HashMap<String, Material>,
    reference_counts: HashMap<String, usize>,
}

impl MaterialCache {
    pub fn new() -> Self {
        Self {
            materials: HashMap::new(),
            reference_counts: HashMap::new(),
        }
    }

    pub fn insert_or_get_id(&mut self, material: &Material) -> String {
        let id = Self::hash_material(material);

        if !self.materials.contains_key(&id) {
            self.materials.insert(id.clone(), material.clone());
            self.reference_counts.insert(id.clone(), 0);
        }

        id
    }

    pub fn get(&self, id: &str) -> Option<&Material> {
        self.materials.get(id)
    }

    pub fn contains(&self, id: &str) -> bool {
        self.materials.contains_key(id)
    }

    pub fn add_reference(&mut self, id: &str) {
        if let Some(count) = self.reference_counts.get_mut(id) {
            *count += 1;
        }
    }

    pub fn remove_reference(&mut self, id: &str) {
        if let Some(count) = self.reference_counts.get_mut(id) {
            *count = count.saturating_sub(1);
        }
    }

    pub fn get_reference_count(&self, id: &str) -> usize {
        self.reference_counts.get(id).copied().unwrap_or(0)
    }

    pub fn remove_unused(&mut self) -> Vec<String> {
        let unused: Vec<String> = self
            .reference_counts
            .iter()
            .filter(|&(_, count)| *count == 0)
            .map(|(id, _)| id.clone())
            .collect();

        for id in &unused {
            self.materials.remove(id);
            self.reference_counts.remove(id);
        }

        unused
    }

    pub fn clear(&mut self) {
        self.materials.clear();
        self.reference_counts.clear();
    }

    pub fn len(&self) -> usize {
        self.materials.len()
    }

    pub fn is_empty(&self) -> bool {
        self.materials.is_empty()
    }

    fn hash_material(material: &Material) -> String {
        let mut hasher = DefaultHasher::new();

        for &val in &material.base_color {
            val.to_bits().hash(&mut hasher);
        }
        for &val in &material.emissive_factor {
            val.to_bits().hash(&mut hasher);
        }
        material.alpha_mode.hash(&mut hasher);
        material.alpha_cutoff.to_bits().hash(&mut hasher);
        material.base_texture.hash(&mut hasher);
        material.emissive_texture.hash(&mut hasher);
        material.roughness.to_bits().hash(&mut hasher);
        material.metallic.to_bits().hash(&mut hasher);
        material.unlit.hash(&mut hasher);

        format!("material_{:016x}", hasher.finish())
    }
}
