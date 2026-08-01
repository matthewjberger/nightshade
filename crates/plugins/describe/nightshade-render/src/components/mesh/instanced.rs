use nalgebra_glm::{Mat3, Mat4, Quat, Vec3};
use nightshade::ecs::Entity;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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
        let rotation = nalgebra_glm::quat_to_mat3(&self.rotation);
        let x = rotation.column(0) * self.scale.x;
        let y = rotation.column(1) * self.scale.y;
        let z = rotation.column(2) * self.scale.z;

        Mat4::new(
            x[0],
            y[0],
            z[0],
            self.translation.x,
            x[1],
            y[1],
            z[1],
            self.translation.y,
            x[2],
            y[2],
            z[2],
            self.translation.z,
            0.0,
            0.0,
            0.0,
            1.0,
        )
    }

    pub fn normal_matrix(&self) -> [[f32; 4]; 3] {
        const SMALLEST: f32 = 1.0e-8;

        if self.scale.x.abs() < SMALLEST
            || self.scale.y.abs() < SMALLEST
            || self.scale.z.abs() < SMALLEST
        {
            return [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
            ];
        }

        let rotation = nalgebra_glm::quat_to_mat3(&self.rotation);

        let squared = self.scale.component_mul(&self.scale);
        let uniform =
            (squared.x - squared.y).abs() < 0.001 && (squared.y - squared.z).abs() < 0.001;
        let divisor = if uniform {
            Vec3::new(1.0, 1.0, 1.0)
        } else {
            self.scale
        };

        let x = rotation.column(0) / divisor.x;
        let y = rotation.column(1) / divisor.y;
        let z = rotation.column(2) / divisor.z;

        [
            [x[0], x[1], x[2], 0.0],
            [y[0], y[1], y[2], 0.0],
            [z[0], z[1], z[2], 0.0],
        ]
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

    pub fn add_instance(&mut self, transform: InstanceTransform) {
        self.instances.push(transform);
        self.custom_data.push(InstanceCustomData::default());
        self.dirty = true;
        self.local_matrices_gpu_dirty = true;
    }

    pub fn add_instances(&mut self, transforms: impl IntoIterator<Item = InstanceTransform>) {
        let transforms: Vec<_> = transforms.into_iter().collect();
        let count = transforms.len();
        self.instances.extend(transforms);
        self.custom_data
            .extend(std::iter::repeat_n(InstanceCustomData::default(), count));
        self.dirty = true;
        self.local_matrices_gpu_dirty = true;
    }

    pub fn clear_instances(&mut self) {
        self.instances.clear();
        self.custom_data.clear();
        self.dirty_custom_data_indices.clear();
        self.dirty = true;
        self.local_matrices_gpu_dirty = true;
    }

    pub fn set_instances(&mut self, instances: Vec<InstanceTransform>) {
        let count = instances.len();
        self.instances = instances;
        self.custom_data = vec![InstanceCustomData::default(); count];
        self.dirty_custom_data_indices.clear();
        self.dirty = true;
        self.local_matrices_gpu_dirty = true;
    }

    pub fn mark_instances_changed(&mut self) {
        self.custom_data
            .resize(self.instances.len(), InstanceCustomData::default());
        self.dirty_custom_data_indices.clear();
        self.dirty = true;
        self.local_matrices_gpu_dirty = true;
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
            self.local_matrices_gpu_dirty = true;
        }
    }

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

    pub fn get_custom_data(&self, index: usize) -> Option<&InstanceCustomData> {
        self.custom_data.get(index)
    }

    pub fn set_instance_tint(&mut self, index: usize, tint: [f32; 4]) {
        if let Some(data) = self.custom_data.get_mut(index)
            && data.tint != tint
        {
            data.tint = tint;
            self.dirty_custom_data_indices.insert(index);
        }
    }

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

#[derive(Default)]
pub struct InstancedMeshMatrices {
    entries: HashMap<Entity, InstancedMeshMatrixCache>,
}

impl InstancedMeshMatrices {
    pub fn take_entries(&mut self) -> HashMap<Entity, InstancedMeshMatrixCache> {
        std::mem::take(&mut self.entries)
    }

    pub fn restore_entries(&mut self, entries: HashMap<Entity, InstancedMeshMatrixCache>) {
        self.entries = entries;
    }

    pub fn model_matrices(&self, entity: Entity) -> &[InstanceModelMatrix] {
        self.entries
            .get(&entity)
            .map_or(&[], |cache| &cache.model_matrices)
    }

    pub fn local_matrices(&self, entity: Entity) -> &[Mat4] {
        self.entries
            .get(&entity)
            .map_or(&[], |cache| &cache.local_matrices)
    }
}

#[derive(Default)]
pub struct InstancedMeshMatrixCache {
    local_matrices: Vec<Mat4>,
    model_matrices: Vec<InstanceModelMatrix>,
    parent_transform: Mat4,
    local_valid: bool,
}

impl InstancedMeshMatrixCache {
    pub fn update(
        &mut self,
        instances: &[InstanceTransform],
        instances_dirty: bool,
        parent_transform: &Mat4,
    ) -> bool {
        let InstancedMeshMatrixCache {
            local_matrices,
            model_matrices,
            parent_transform: cached_parent,
            local_valid,
        } = self;

        if instances_dirty || !*local_valid || local_matrices.len() != instances.len() {
            local_matrices.clear();
            local_matrices.resize(instances.len(), Mat4::identity());
            fill_by_index(local_matrices, |index| instances[index].as_matrix());
            *local_valid = true;
        }

        let parent_changed = *cached_parent != *parent_transform;
        let count_changed = model_matrices.len() != local_matrices.len();

        if !instances_dirty && !count_changed && parent_changed {
            *cached_parent = *parent_transform;
            return true;
        }

        if instances_dirty || parent_changed || count_changed {
            model_matrices.clear();
            model_matrices.resize(
                local_matrices.len(),
                InstanceModelMatrix {
                    model: [[0.0; 4]; 4],
                    normal_matrix: [[0.0; 4]; 3],
                },
            );

            if parent_transform == &Mat4::identity() && instances.len() == local_matrices.len() {
                fill_by_index(model_matrices, |index| InstanceModelMatrix {
                    model: local_matrices[index].into(),
                    normal_matrix: instances[index].normal_matrix(),
                });
            } else {
                let parent = *parent_transform;
                fill_by_index(model_matrices, |index| {
                    let world_matrix = parent * local_matrices[index];
                    InstanceModelMatrix {
                        model: world_matrix.into(),
                        normal_matrix: compute_normal_matrix(&world_matrix),
                    }
                });
            }
            *cached_parent = *parent_transform;
        }
        false
    }
}

fn fill_by_index<T, F>(slots: &mut [T], fill: F)
where
    T: Send,
    F: Fn(usize) -> T + Sync + Send,
{
    #[cfg(not(target_family = "wasm"))]
    {
        const FILL_CHUNK: usize = 4096;

        use nightshade::ecs::rayon::prelude::*;

        slots
            .par_chunks_mut(FILL_CHUNK)
            .enumerate()
            .for_each(|(chunk, slice)| {
                let base = chunk * FILL_CHUNK;
                for (offset, slot) in slice.iter_mut().enumerate() {
                    *slot = fill(base + offset);
                }
            });
    }

    #[cfg(target_family = "wasm")]
    for (index, slot) in slots.iter_mut().enumerate() {
        *slot = fill(index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> InstanceTransform {
        InstanceTransform {
            translation: Vec3::new(3.0, -4.0, 5.0),
            rotation: nalgebra_glm::quat_angle_axis(
                0.9,
                &nalgebra_glm::normalize(&Vec3::new(0.3, 1.0, -0.6)),
            ),
            scale: Vec3::new(1.6, 5.0, 1.6),
        }
    }

    fn composed(instance: &InstanceTransform) -> Mat4 {
        nalgebra_glm::translation(&instance.translation)
            * nalgebra_glm::quat_to_mat4(&instance.rotation)
            * nalgebra_glm::scaling(&instance.scale)
    }

    #[test]
    fn the_built_matrix_matches_the_composed_one() {
        let instance = sample();

        let built = instance.as_matrix();
        let expected = composed(&instance);

        for index in 0..16 {
            assert!((built[index] - expected[index]).abs() < 1.0e-4);
        }
    }

    #[test]
    fn a_default_instance_builds_the_identity() {
        let built = InstanceTransform::default().as_matrix();

        for index in 0..16 {
            assert!((built[index] - Mat4::identity()[index]).abs() < 1.0e-6);
        }
    }

    #[test]
    fn the_worked_out_normal_matrix_matches_the_inverted_one() {
        let instance = sample();

        let worked = instance.normal_matrix();
        let inverted = compute_normal_matrix(&composed(&instance));

        for column in 0..3 {
            for row in 0..4 {
                assert!((worked[column][row] - inverted[column][row]).abs() < 1.0e-4);
            }
        }
    }

    #[test]
    fn a_uniform_scale_agrees_with_the_inverted_one_too() {
        let mut instance = sample();
        instance.scale = Vec3::new(2.0, 2.0, 2.0);

        let worked = instance.normal_matrix();
        let inverted = compute_normal_matrix(&composed(&instance));

        for column in 0..3 {
            for row in 0..4 {
                assert!((worked[column][row] - inverted[column][row]).abs() < 1.0e-4);
            }
        }
    }

    #[test]
    fn a_flattened_instance_falls_back_rather_than_dividing_by_nothing() {
        let mut instance = sample();
        instance.scale = Vec3::new(1.0, 0.0, 1.0);

        let worked = instance.normal_matrix();

        assert_eq!(worked[0], [1.0, 0.0, 0.0, 0.0]);
        assert_eq!(worked[1], [0.0, 1.0, 0.0, 0.0]);
    }
}
