use nalgebra_glm::{Mat3, Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceModelMatrix {
    pub model: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 3],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceObjectData {
    pub transform_index: u32,
    pub mesh_id: u32,
    pub material_id: u32,
    pub visible: u32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct InstanceTransform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl InstanceTransform {
    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn from_translation_scale(translation: Vec3, scale: Vec3) -> Self {
        Self {
            translation,
            rotation: Quat::identity(),
            scale,
        }
    }

    pub fn as_matrix(&self) -> Mat4 {
        let translation_matrix = nalgebra_glm::translation(&self.translation);
        let rotation_matrix = nalgebra_glm::quat_to_mat4(&self.rotation);
        let scale_matrix = nalgebra_glm::scaling(&self.scale);
        translation_matrix * rotation_matrix * scale_matrix
    }
}

fn compute_normal_matrix(model_matrix: &Mat4) -> [[f32; 4]; 3] {
    let mat3 = nalgebra_glm::mat4_to_mat3(model_matrix);

    let scale_x_sq =
        mat3[(0, 0)] * mat3[(0, 0)] + mat3[(1, 0)] * mat3[(1, 0)] + mat3[(2, 0)] * mat3[(2, 0)];
    let scale_y_sq =
        mat3[(0, 1)] * mat3[(0, 1)] + mat3[(1, 1)] * mat3[(1, 1)] + mat3[(2, 1)] * mat3[(2, 1)];
    let scale_z_sq =
        mat3[(0, 2)] * mat3[(0, 2)] + mat3[(1, 2)] * mat3[(1, 2)] + mat3[(2, 2)] * mat3[(2, 2)];

    let is_uniform_scale =
        (scale_x_sq - scale_y_sq).abs() < 0.001 && (scale_y_sq - scale_z_sq).abs() < 0.001;

    let normal_mat3 = if is_uniform_scale {
        let inv_scale = 1.0 / scale_x_sq.sqrt();
        Mat3::new(
            mat3[(0, 0)] * inv_scale,
            mat3[(0, 1)] * inv_scale,
            mat3[(0, 2)] * inv_scale,
            mat3[(1, 0)] * inv_scale,
            mat3[(1, 1)] * inv_scale,
            mat3[(1, 2)] * inv_scale,
            mat3[(2, 0)] * inv_scale,
            mat3[(2, 1)] * inv_scale,
            mat3[(2, 2)] * inv_scale,
        )
    } else {
        match mat3.try_inverse() {
            Some(inv) => inv.transpose(),
            None => Mat3::identity(),
        }
    };

    [
        [
            normal_mat3[(0, 0)],
            normal_mat3[(1, 0)],
            normal_mat3[(2, 0)],
            0.0,
        ],
        [
            normal_mat3[(0, 1)],
            normal_mat3[(1, 1)],
            normal_mat3[(2, 1)],
            0.0,
        ],
        [
            normal_mat3[(0, 2)],
            normal_mat3[(1, 2)],
            normal_mat3[(2, 2)],
            0.0,
        ],
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstancedMesh {
    pub mesh_name: String,
    #[serde(skip)]
    pub instances: Vec<InstanceTransform>,
    #[serde(skip)]
    pub dirty: bool,
    #[serde(skip)]
    cached_local_matrices: Vec<Mat4>,
    #[serde(skip)]
    cached_world_matrices: Vec<Mat4>,
    #[serde(skip)]
    cached_normal_matrices: Vec<[[f32; 4]; 3]>,
    #[serde(skip)]
    cached_parent_transform: Mat4,
    #[serde(skip)]
    local_matrices_valid: bool,
    #[serde(skip)]
    cached_model_matrices: Vec<InstanceModelMatrix>,
}

impl Default for InstancedMesh {
    fn default() -> Self {
        Self {
            mesh_name: "Cube".to_string(),
            instances: Vec::new(),
            dirty: true,
            cached_local_matrices: Vec::new(),
            cached_world_matrices: Vec::new(),
            cached_normal_matrices: Vec::new(),
            cached_parent_transform: Mat4::identity(),
            local_matrices_valid: false,
            cached_model_matrices: Vec::new(),
        }
    }
}

impl InstancedMesh {
    pub fn new(mesh_name: &str) -> Self {
        Self {
            mesh_name: mesh_name.to_string(),
            instances: Vec::new(),
            dirty: true,
            cached_local_matrices: Vec::new(),
            cached_world_matrices: Vec::new(),
            cached_normal_matrices: Vec::new(),
            cached_parent_transform: Mat4::identity(),
            local_matrices_valid: false,
            cached_model_matrices: Vec::new(),
        }
    }

    pub fn with_instances(mesh_name: &str, instances: Vec<InstanceTransform>) -> Self {
        Self {
            mesh_name: mesh_name.to_string(),
            instances,
            dirty: true,
            cached_local_matrices: Vec::new(),
            cached_world_matrices: Vec::new(),
            cached_normal_matrices: Vec::new(),
            cached_parent_transform: Mat4::identity(),
            local_matrices_valid: false,
            cached_model_matrices: Vec::new(),
        }
    }

    pub fn add_instance(&mut self, transform: InstanceTransform) {
        self.instances.push(transform);
        self.dirty = true;
        self.local_matrices_valid = false;
    }

    pub fn add_instances(&mut self, transforms: impl IntoIterator<Item = InstanceTransform>) {
        self.instances.extend(transforms);
        self.dirty = true;
        self.local_matrices_valid = false;
    }

    pub fn clear_instances(&mut self) {
        self.instances.clear();
        self.cached_local_matrices.clear();
        self.cached_world_matrices.clear();
        self.cached_normal_matrices.clear();
        self.cached_model_matrices.clear();
        self.dirty = true;
        self.local_matrices_valid = false;
    }

    pub fn set_instances(&mut self, instances: Vec<InstanceTransform>) {
        self.instances = instances;
        self.dirty = true;
        self.local_matrices_valid = false;
    }

    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    pub fn get_instance(&self, index: usize) -> Option<&InstanceTransform> {
        self.instances.get(index)
    }

    pub fn get_instance_mut(&mut self, index: usize) -> Option<&mut InstanceTransform> {
        self.dirty = true;
        self.local_matrices_valid = false;
        self.instances.get_mut(index)
    }

    pub fn remove_instance(&mut self, index: usize) -> Option<InstanceTransform> {
        if index < self.instances.len() {
            self.dirty = true;
            self.local_matrices_valid = false;
            Some(self.instances.remove(index))
        } else {
            None
        }
    }

    fn ensure_local_matrices_cached(&mut self) {
        if !self.local_matrices_valid || self.cached_local_matrices.len() != self.instances.len() {
            self.cached_local_matrices.clear();
            self.cached_local_matrices.reserve(self.instances.len());
            for instance in &self.instances {
                self.cached_local_matrices.push(instance.as_matrix());
            }
            self.local_matrices_valid = true;
        }
    }

    pub fn update_world_matrices(&mut self, parent_transform: &Mat4) {
        self.ensure_local_matrices_cached();

        let parent_changed = self.cached_parent_transform != *parent_transform;
        let needs_world_update =
            parent_changed || self.cached_world_matrices.len() != self.cached_local_matrices.len();

        if needs_world_update {
            self.cached_world_matrices.clear();
            self.cached_world_matrices
                .reserve(self.cached_local_matrices.len());
            self.cached_normal_matrices.clear();
            self.cached_normal_matrices
                .reserve(self.cached_local_matrices.len());
            self.cached_model_matrices.clear();
            self.cached_model_matrices
                .reserve(self.cached_local_matrices.len());

            for local_matrix in &self.cached_local_matrices {
                let world_matrix = parent_transform * local_matrix;
                let normal_matrix = compute_normal_matrix(&world_matrix);
                self.cached_world_matrices.push(world_matrix);
                self.cached_normal_matrices.push(normal_matrix);
                self.cached_model_matrices.push(InstanceModelMatrix {
                    model: world_matrix.into(),
                    normal_matrix,
                });
            }
            self.cached_parent_transform = *parent_transform;
        }
        self.dirty = false;
    }

    pub fn cached_world_matrices(&self) -> &[Mat4] {
        &self.cached_world_matrices
    }

    pub fn cached_normal_matrices(&self) -> &[[[f32; 4]; 3]] {
        &self.cached_normal_matrices
    }

    pub fn cached_model_matrices(&self) -> &[InstanceModelMatrix] {
        &self.cached_model_matrices
    }

    pub fn has_valid_cache(&self) -> bool {
        !self.dirty
            && self.local_matrices_valid
            && self.cached_world_matrices.len() == self.instances.len()
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }
}
