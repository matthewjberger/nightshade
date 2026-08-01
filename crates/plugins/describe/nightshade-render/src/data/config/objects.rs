#[derive(Clone, Default)]
pub struct InstancedObjectData {
    pub world_models: Vec<[[f32; 4]; 4]>,
    pub world_normals: Vec<[[f32; 4]; 3]>,
    pub local_matrices: Vec<nalgebra_glm::Mat4>,
    pub custom_tints: Vec<[f32; 4]>,
    pub mesh_name: String,
    pub render_layer: u32,
    pub visible: u32,
    pub parent_transform: nalgebra_glm::Mat4,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RenderBounds {
    pub center: nalgebra_glm::Vec3,
    pub sphere_radius: f32,
}
