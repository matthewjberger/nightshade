use nalgebra_glm::{Mat3, Mat4, Quat, Vec3};
use nightshade_ecs::Entity;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// GPU instance data containing model and normal matrices.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceModelMatrix {
    /// 4x4 model matrix for this instance.
    pub model: [[f32; 4]; 4],
    /// 3x4 normal matrix (transposed inverse of upper-left 3x3).
    pub normal_matrix: [[f32; 4]; 3],
}

/// Per-instance custom data for rendering variations.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceCustomData {
    /// Color tint applied to this instance.
    pub tint: [f32; 4],
}

impl Default for InstanceCustomData {
    fn default() -> Self {
        Self {
            tint: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

/// Transform data for a single instance within an [`InstancedMesh`].
#[derive(Debug, Clone, Copy)]
pub struct InstanceTransform {
    /// Position offset from the parent entity.
    pub translation: Vec3,
    /// Orientation as a quaternion.
    pub rotation: Quat,
    /// Scale factors.
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
    /// Creates a new instance transform with all components.
    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    /// Creates an instance transform with only position.
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }

    /// Creates an instance transform with position and scale.
    pub fn from_translation_scale(translation: Vec3, scale: Vec3) -> Self {
        Self {
            translation,
            rotation: Quat::identity(),
            scale,
        }
    }

    /// Computes the 4x4 transformation matrix.
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

/// Component for rendering many instances of the same mesh efficiently.
///
/// GPU instancing renders thousands of objects with a single draw call.
/// Each instance has its own transform and optional tint color.
///
/// The component holds only the instance data. The derived per-instance GPU
/// matrices live in the [`InstancedMeshMatrices`] resource, refreshed each
/// frame from the parent entity's
/// [`crate::ecs::transform::GlobalTransform`] and these instance transforms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstancedMesh {
    /// Name of the mesh to instance from the mesh cache.
    pub mesh_name: String,
    #[serde(skip)]
    pub instances: Vec<InstanceTransform>,
    #[serde(skip)]
    pub custom_data: Vec<InstanceCustomData>,
    #[serde(skip)]
    dirty_custom_data_indices: HashSet<usize>,
    #[serde(skip)]
    pub dirty: bool,
    #[serde(skip)]
    local_matrices_gpu_dirty: bool,
}

impl Default for InstancedMesh {
    fn default() -> Self {
        Self {
            mesh_name: "Cube".to_string(),
            instances: Vec::new(),
            custom_data: Vec::new(),
            dirty_custom_data_indices: HashSet::new(),
            dirty: true,
            local_matrices_gpu_dirty: true,
        }
    }
}

impl InstancedMesh {
    /// Creates an empty instanced mesh for the named mesh.
    pub fn new(mesh_name: &str) -> Self {
        Self {
            mesh_name: mesh_name.to_string(),
            ..Default::default()
        }
    }

    pub fn with_instances(mesh_name: &str, instances: Vec<InstanceTransform>) -> Self {
        let instance_count = instances.len();
        Self {
            mesh_name: mesh_name.to_string(),
            instances,
            custom_data: vec![InstanceCustomData::default(); instance_count],
            ..Default::default()
        }
    }

    /// Adds a single instance with the given transform.
    pub fn add_instance(&mut self, transform: InstanceTransform) {
        self.instances.push(transform);
        self.custom_data.push(InstanceCustomData::default());
        self.dirty = true;
        self.local_matrices_gpu_dirty = true;
    }

    /// Adds multiple instances at once.
    pub fn add_instances(&mut self, transforms: impl IntoIterator<Item = InstanceTransform>) {
        let transforms: Vec<_> = transforms.into_iter().collect();
        let count = transforms.len();
        self.instances.extend(transforms);
        self.custom_data
            .extend(std::iter::repeat_n(InstanceCustomData::default(), count));
        self.dirty = true;
        self.local_matrices_gpu_dirty = true;
    }

    /// Removes all instances.
    pub fn clear_instances(&mut self) {
        self.instances.clear();
        self.custom_data.clear();
        self.dirty_custom_data_indices.clear();
        self.dirty = true;
        self.local_matrices_gpu_dirty = true;
    }

    /// Replaces all instances with the given transforms.
    pub fn set_instances(&mut self, instances: Vec<InstanceTransform>) {
        let count = instances.len();
        self.instances = instances;
        self.custom_data = vec![InstanceCustomData::default(); count];
        self.dirty_custom_data_indices.clear();
        self.dirty = true;
        self.local_matrices_gpu_dirty = true;
    }

    /// Returns the number of instances.
    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    /// Returns the transform for an instance by index.
    pub fn get_instance(&self, index: usize) -> Option<&InstanceTransform> {
        self.instances.get(index)
    }

    /// Updates the transform for an instance.
    pub fn set_instance_transform(&mut self, index: usize, transform: InstanceTransform) {
        if let Some(existing) = self.instances.get_mut(index)
            && (existing.translation != transform.translation
                || existing.rotation != transform.rotation
                || existing.scale != transform.scale)
        {
            *existing = transform;
            self.dirty = true;
            self.local_matrices_gpu_dirty = true;
        }
    }

    /// Removes an instance by index, returning its transform.
    pub fn remove_instance(&mut self, index: usize) -> Option<InstanceTransform> {
        if index < self.instances.len() {
            self.dirty = true;
            self.local_matrices_gpu_dirty = true;
            self.dirty_custom_data_indices.clear();
            self.custom_data.remove(index);
            Some(self.instances.remove(index))
        } else {
            None
        }
    }

    /// Returns the custom data for an instance.
    pub fn get_custom_data(&self, index: usize) -> Option<&InstanceCustomData> {
        self.custom_data.get(index)
    }

    /// Sets the tint color for an instance.
    pub fn set_instance_tint(&mut self, index: usize, tint: [f32; 4]) {
        if let Some(data) = self.custom_data.get_mut(index)
            && data.tint != tint
        {
            data.tint = tint;
            self.dirty_custom_data_indices.insert(index);
        }
    }

    /// Returns all custom data as a slice.
    pub fn custom_data_slice(&self) -> &[InstanceCustomData] {
        &self.custom_data
    }

    pub fn take_dirty_custom_data_indices(&mut self) -> HashSet<usize> {
        std::mem::take(&mut self.dirty_custom_data_indices)
    }

    pub fn has_dirty_custom_data(&self) -> bool {
        !self.dirty_custom_data_indices.is_empty()
    }

    pub fn local_matrices_gpu_dirty(&self) -> bool {
        self.local_matrices_gpu_dirty
    }

    pub fn clear_local_matrices_gpu_dirty(&mut self) {
        self.local_matrices_gpu_dirty = false;
    }
}

/// Per-entity derived instance matrices for [`InstancedMesh`] entities.
///
/// The matrices are kept out of the component so the component stays plain
/// instance data. The instanced-mesh update system refreshes one entry per
/// entity each frame; the render sync reads the matrices back by entity.
/// Entries for entities that no longer carry an [`InstancedMesh`] are pruned
/// each frame.
#[derive(Default)]
pub struct InstancedMeshMatrices {
    entries: HashMap<Entity, InstancedMeshMatrixCache>,
}

impl InstancedMeshMatrices {
    /// Takes the cache map out for the update pass, leaving an empty map.
    pub fn take_entries(&mut self) -> HashMap<Entity, InstancedMeshMatrixCache> {
        std::mem::take(&mut self.entries)
    }

    /// Restores the cache map after the update pass.
    pub fn restore_entries(&mut self, entries: HashMap<Entity, InstancedMeshMatrixCache>) {
        self.entries = entries;
    }

    /// GPU-ready model matrices for an entity, or an empty slice.
    pub fn model_matrices(&self, entity: Entity) -> &[InstanceModelMatrix] {
        self.entries
            .get(&entity)
            .map_or(&[], |cache| &cache.model_matrices)
    }

    /// Local instance matrices for an entity, or an empty slice.
    pub fn local_matrices(&self, entity: Entity) -> &[Mat4] {
        self.entries
            .get(&entity)
            .map_or(&[], |cache| &cache.local_matrices)
    }
}

/// One entity's cached instance matrices, derived from its [`InstancedMesh`].
#[derive(Default)]
pub struct InstancedMeshMatrixCache {
    local_matrices: Vec<Mat4>,
    model_matrices: Vec<InstanceModelMatrix>,
    parent_transform: Mat4,
    local_valid: bool,
}

impl InstancedMeshMatrixCache {
    /// Recomputes the cached matrices from the source instances when the
    /// instances changed, the parent moved, or the instance count changed.
    /// Returns `true` when only the parent transform changed (the instances and
    /// count were unchanged), so the caller can trigger a compute dispatch.
    pub fn update(
        &mut self,
        instances: &[InstanceTransform],
        instances_dirty: bool,
        parent_transform: &Mat4,
    ) -> bool {
        if instances_dirty || !self.local_valid || self.local_matrices.len() != instances.len() {
            self.local_matrices.clear();
            self.local_matrices.reserve(instances.len());
            for instance in instances {
                self.local_matrices.push(instance.as_matrix());
            }
            self.local_valid = true;
        }

        let parent_changed = self.parent_transform != *parent_transform;
        let count_changed = self.model_matrices.len() != self.local_matrices.len();

        if !instances_dirty && !count_changed && parent_changed {
            self.parent_transform = *parent_transform;
            return true;
        }

        if instances_dirty || parent_changed || count_changed {
            self.model_matrices.clear();
            self.model_matrices.reserve(self.local_matrices.len());
            for local_matrix in &self.local_matrices {
                let world_matrix = parent_transform * local_matrix;
                let normal_matrix = compute_normal_matrix(&world_matrix);
                self.model_matrices.push(InstanceModelMatrix {
                    model: world_matrix.into(),
                    normal_matrix,
                });
            }
            self.parent_transform = *parent_transform;
        }
        false
    }
}
