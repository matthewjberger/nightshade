use nalgebra_glm::{Mat3, Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashSet;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceModelMatrix {
    pub model: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 3],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceCustomData {
    pub tint: [f32; 4],
}

impl Default for InstanceCustomData {
    fn default() -> Self {
        Self {
            tint: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceObjectData {
    pub transform_index: u32,
    pub mesh_id: u32,
    pub material_id: u32,
    pub visible: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct InstanceTransform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for InstanceTransform {
    fn default() -> Self {
        Self {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }
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

#[derive(Debug, Serialize, Deserialize)]
pub struct InstancedMesh {
    pub mesh_name: String,
    #[serde(skip)]
    pub instances: Vec<InstanceTransform>,
    #[serde(skip)]
    pub custom_data: Vec<InstanceCustomData>,
    #[serde(skip)]
    dirty_custom_data_indices: RefCell<HashSet<usize>>,
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
            custom_data: Vec::new(),
            dirty_custom_data_indices: RefCell::new(HashSet::new()),
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

impl Clone for InstancedMesh {
    fn clone(&self) -> Self {
        Self {
            mesh_name: self.mesh_name.clone(),
            instances: self.instances.clone(),
            custom_data: self.custom_data.clone(),
            dirty_custom_data_indices: RefCell::new(
                self.dirty_custom_data_indices.borrow().clone(),
            ),
            dirty: self.dirty,
            cached_local_matrices: self.cached_local_matrices.clone(),
            cached_world_matrices: self.cached_world_matrices.clone(),
            cached_normal_matrices: self.cached_normal_matrices.clone(),
            cached_parent_transform: self.cached_parent_transform,
            local_matrices_valid: self.local_matrices_valid,
            cached_model_matrices: self.cached_model_matrices.clone(),
        }
    }
}

impl InstancedMesh {
    pub fn new(mesh_name: &str) -> Self {
        Self {
            mesh_name: mesh_name.to_string(),
            instances: Vec::new(),
            custom_data: Vec::new(),
            dirty_custom_data_indices: RefCell::new(HashSet::new()),
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
        let instance_count = instances.len();
        Self {
            mesh_name: mesh_name.to_string(),
            instances,
            custom_data: vec![InstanceCustomData::default(); instance_count],
            dirty_custom_data_indices: RefCell::new(HashSet::new()),
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
        self.custom_data.push(InstanceCustomData::default());
        self.dirty = true;
        self.local_matrices_valid = false;
    }

    pub fn add_instances(&mut self, transforms: impl IntoIterator<Item = InstanceTransform>) {
        let transforms: Vec<_> = transforms.into_iter().collect();
        let count = transforms.len();
        self.instances.extend(transforms);
        self.custom_data
            .extend(std::iter::repeat_n(InstanceCustomData::default(), count));
        self.dirty = true;
        self.local_matrices_valid = false;
    }

    pub fn clear_instances(&mut self) {
        self.instances.clear();
        self.custom_data.clear();
        self.cached_local_matrices.clear();
        self.cached_world_matrices.clear();
        self.cached_normal_matrices.clear();
        self.cached_model_matrices.clear();
        self.dirty_custom_data_indices.borrow_mut().clear();
        self.dirty = true;
        self.local_matrices_valid = false;
    }

    pub fn set_instances(&mut self, instances: Vec<InstanceTransform>) {
        let count = instances.len();
        self.instances = instances;
        self.custom_data = vec![InstanceCustomData::default(); count];
        self.dirty_custom_data_indices.borrow_mut().clear();
        self.dirty = true;
        self.local_matrices_valid = false;
    }

    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    pub fn get_instance(&self, index: usize) -> Option<&InstanceTransform> {
        self.instances.get(index)
    }

    pub fn set_instance_transform(&mut self, index: usize, transform: InstanceTransform) {
        if let Some(existing) = self.instances.get_mut(index)
            && (existing.translation != transform.translation
                || existing.rotation != transform.rotation
                || existing.scale != transform.scale)
        {
            *existing = transform;
            self.dirty = true;
            self.local_matrices_valid = false;
        }
    }

    pub fn remove_instance(&mut self, index: usize) -> Option<InstanceTransform> {
        if index < self.instances.len() {
            self.dirty = true;
            self.local_matrices_valid = false;
            self.dirty_custom_data_indices.borrow_mut().clear();
            self.custom_data.remove(index);
            Some(self.instances.remove(index))
        } else {
            None
        }
    }

    pub fn get_custom_data(&self, index: usize) -> Option<&InstanceCustomData> {
        self.custom_data.get(index)
    }

    pub fn set_instance_tint(&mut self, index: usize, tint: [f32; 4]) {
        if let Some(data) = self.custom_data.get_mut(index)
            && data.tint != tint
        {
            data.tint = tint;
            self.dirty_custom_data_indices.borrow_mut().insert(index);
        }
    }

    pub fn custom_data_slice(&self) -> &[InstanceCustomData] {
        &self.custom_data
    }

    pub fn take_dirty_custom_data_indices(&self) -> HashSet<usize> {
        std::mem::take(&mut *self.dirty_custom_data_indices.borrow_mut())
    }

    pub fn has_dirty_custom_data(&self) -> bool {
        !self.dirty_custom_data_indices.borrow().is_empty()
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
