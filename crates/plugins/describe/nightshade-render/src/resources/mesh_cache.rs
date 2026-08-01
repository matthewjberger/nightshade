use std::collections::HashSet;

use crate::data::generational_registry::*;
use crate::data::geometry::Mesh;

#[derive(Clone, Default)]
pub struct MeshCache {
    pub registry: GenerationalRegistry<Mesh>,
    pub dirty_meshes: HashSet<String>,
    pub dirty_skinned_meshes: HashSet<String>,
    pub dirty_clusters: HashSet<String>,
    pub clear_generation: u64,
}
