use crate::contract::Prefab;
use nalgebra_glm::Mat4;
use nightshade_animation::contract::AnimationClip;
use nightshade_graphics::resources::loading::TextureProductionPlan;
use nightshade_render::components::mesh::Mesh;
use nightshade_render::data::material::Material;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ModelSkin {
    pub name: Option<String>,
    pub joints: Vec<usize>,
    pub inverse_bind_matrices: Vec<Mat4>,
}

pub struct ModelImport {
    pub prefabs: Vec<Prefab>,
    pub meshes: HashMap<String, Mesh>,
    pub materials: Vec<Material>,
    pub encoded_images: Vec<Vec<u8>>,
    pub texture_plan: Vec<TextureProductionPlan>,
    pub animations: Vec<AnimationClip>,
    pub skins: Vec<ModelSkin>,
    pub node_to_skin: HashMap<usize, usize>,
    pub node_to_morph_target_count: HashMap<usize, usize>,
    pub node_count: usize,
    pub suggested_exposure: f32,
}
