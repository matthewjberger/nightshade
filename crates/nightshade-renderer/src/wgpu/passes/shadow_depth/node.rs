use crate::skinning::GpuSkinData;
use crate::wgpu::rendergraph::{PassExecutionContext, PassNode};
use wgpu::util::DeviceExt;

use super::atlas::{full_frustum_planes, sphere_in_frustum};
use super::cascade::{
    calculate_cascade_view_projection, get_frustum_corners_world_space, reverse_z_ortho_light,
    reverse_z_perspective,
};
use super::pass::ShadowDepthPass;
use super::types::{
    CASCADE_ATLAS_SLOTS_PER_ROW, CASCADE_SLOT_RESOLUTION, MAX_POINT_LIGHT_SHADOWS,
    MAX_SPOTLIGHT_SHADOWS, NUM_SHADOW_CASCADES, POINT_SHADOW_FACE_SIZE, POINT_SHADOW_NUM_FACES,
    PointLightShadowData, PointLightShadowSlot, PointShadowUniforms, ShadowCullUniforms,
    ShadowDrawIndexedIndirect, ShadowOccluder, ShadowSlotCache, ShadowUniforms,
    SkinnedShadowCullObject, SkinnedShadowObjectData, SpotlightShadowSlot, scale_cascade_splits,
};
use crate::wgpu::passes::grow_storage_buffer;

/// One occluder reduced to a world-space bounding sphere plus a content hash of
/// its geometry id and model matrix, used to decide whether a cached shadow view
/// still matches this frame.
pub(super) struct OccluderCacheEntry {
    pub(super) center: nalgebra_glm::Vec3,
    pub(super) radius: f32,
    pub(super) hash: u64,
}

/// Whether a bounding sphere intersects the axis-aligned cube a point light
/// shadows. The six cube faces each reach `half_extent` along their axis, so the
/// rendered region is exactly this cube; a euclidean range test would miss
/// occluders in the corners that the cubemap still renders.
fn sphere_intersects_cube(
    center: nalgebra_glm::Vec3,
    radius: f32,
    cube_center: nalgebra_glm::Vec3,
    half_extent: f32,
) -> bool {
    let dx = ((center.x - cube_center.x).abs() - half_extent).max(0.0);
    let dy = ((center.y - cube_center.y).abs() - half_extent).max(0.0);
    let dz = ((center.z - cube_center.z).abs() - half_extent).max(0.0);
    dx * dx + dy * dy + dz * dz <= radius * radius
}

fn hash_occluder(mesh_geo_id: u32, model: &[[f32; 4]; 4]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    mesh_geo_id.hash(&mut hasher);
    bytemuck::bytes_of(model).hash(&mut hasher);
    hasher.finish()
}

fn hash_view_key(bytes: &[u8]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

/// Extracts the four side frustum planes (left/right/bottom/top) from a
/// view-projection matrix for occluder culling. The two depth planes are left
/// disabled (a never-reject plane) so occluders between the light and the
/// cascade box, which still cast shadows into it, are never culled. Planes are
/// returned as `(a, b, c, d)` with inside defined as `a*x + b*y + c*z + d >= 0`.
fn cascade_side_planes(view_projection: &[[f32; 4]; 4]) -> [[f32; 4]; 6] {
    let row = |index: usize| {
        [
            view_projection[0][index],
            view_projection[1][index],
            view_projection[2][index],
            view_projection[3][index],
        ]
    };
    let row_x = row(0);
    let row_y = row(1);
    let row_w = row(3);
    let combine = |sign: f32, axis: [f32; 4]| {
        let plane = [
            row_w[0] + sign * axis[0],
            row_w[1] + sign * axis[1],
            row_w[2] + sign * axis[2],
            row_w[3] + sign * axis[3],
        ];
        let length = (plane[0] * plane[0] + plane[1] * plane[1] + plane[2] * plane[2])
            .sqrt()
            .max(1e-8);
        [
            plane[0] / length,
            plane[1] / length,
            plane[2] / length,
            plane[3] / length,
        ]
    };
    let disabled = [0.0, 0.0, 0.0, 1.0e9];
    [
        combine(1.0, row_x),
        combine(-1.0, row_x),
        combine(1.0, row_y),
        combine(-1.0, row_y),
        disabled,
        disabled,
    ]
}

const Y_FLIP_MATRIX: nalgebra_glm::Mat4 = nalgebra_glm::Mat4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
);

const CUBEMAP_FACE_DIRECTIONS: [(nalgebra_glm::Vec3, nalgebra_glm::Vec3); POINT_SHADOW_NUM_FACES] = [
    (
        nalgebra_glm::Vec3::new(1.0, 0.0, 0.0),
        nalgebra_glm::Vec3::new(0.0, -1.0, 0.0),
    ),
    (
        nalgebra_glm::Vec3::new(-1.0, 0.0, 0.0),
        nalgebra_glm::Vec3::new(0.0, -1.0, 0.0),
    ),
    (
        nalgebra_glm::Vec3::new(0.0, 1.0, 0.0),
        nalgebra_glm::Vec3::new(0.0, 0.0, 1.0),
    ),
    (
        nalgebra_glm::Vec3::new(0.0, -1.0, 0.0),
        nalgebra_glm::Vec3::new(0.0, 0.0, -1.0),
    ),
    (
        nalgebra_glm::Vec3::new(0.0, 0.0, 1.0),
        nalgebra_glm::Vec3::new(0.0, -1.0, 0.0),
    ),
    (
        nalgebra_glm::Vec3::new(0.0, 0.0, -1.0),
        nalgebra_glm::Vec3::new(0.0, -1.0, 0.0),
    ),
];

fn caster_visible(
    configs: &crate::wgpu::render_configs::RenderInputs,
    entity: &freecs::Entity,
) -> bool {
    if let Some(visible) = configs
        .scene_world
        .get::<crate::render_world::Visible>(*entity)
    {
        return visible.0;
    }
    if let Some(instanced) = configs
        .scene_world
        .get::<crate::config::InstancedObjectData>(*entity)
    {
        return instanced.visible != 0;
    }
    configs
        .scene
        .render_skinned_dynamic_state
        .get(entity)
        .map(|dynamic| dynamic.visible != 0)
        .unwrap_or(true)
}

impl PassNode<crate::wgpu::render_configs::RenderInputs> for ShadowDepthPass {
    fn name(&self) -> &str {
        "shadow_depth_pass"
    }

    fn reads(&self) -> Vec<&str> {
        vec![]
    }

    fn writes(&self) -> Vec<&str> {
        vec!["shadow_depth", "spotlight_shadow_atlas"]
    }

    fn reads_writes(&self) -> Vec<&str> {
        vec![]
    }

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &crate::wgpu::render_configs::RenderInputs,
    ) {
        self.sync_meshes(device, queue, &configs.mesh_cache);

        let directional_light = configs
            .scene_world
            .query_ref::<(
                &crate::config::RenderLightData,
                &crate::render_world::Transform,
            )>()
            .iter()
            .find_map(|(_entity, (light, transform))| {
                if matches!(
                    light.light_type,
                    crate::config::RenderLightType::Directional
                ) && light.cast_shadows
                {
                    Some((light.clone(), transform.0))
                } else {
                    None
                }
            });

        if let Some((_light, light_transform)) = directional_light {
            let light_direction =
                crate::wgpu::passes::geometry::projection::transform_forward(&light_transform);

            let camera_matrices = configs.scene.render_view.as_ref();

            if let Some(camera_matrices) = camera_matrices {
                let (fov, aspect, camera_near) = if camera_matrices.orthographic.is_some() {
                    (std::f32::consts::FRAC_PI_4, 1.78, 0.1)
                } else {
                    (
                        camera_matrices.y_fov_rad,
                        camera_matrices.aspect,
                        camera_matrices.z_near,
                    )
                };

                let cascade_resolution = CASCADE_SLOT_RESOLUTION;

                let cascade_split_distances = scale_cascade_splits(camera_near);

                for cascade_index in 0..NUM_SHADOW_CASCADES {
                    let cascade_near = if cascade_index == 0 {
                        camera_near
                    } else {
                        cascade_split_distances[cascade_index - 1]
                    };
                    let cascade_far = cascade_split_distances[cascade_index];

                    let frustum_corners = get_frustum_corners_world_space(
                        &camera_matrices.view,
                        fov,
                        aspect,
                        cascade_near,
                        cascade_far,
                    );

                    let view_projection = calculate_cascade_view_projection(
                        &frustum_corners,
                        &light_direction,
                        cascade_resolution,
                        cascade_far,
                    );

                    self.cascade_data.view_projections[cascade_index] = view_projection.into();
                }
                self.cascade_data.split_distances = cascade_split_distances;
            } else {
                let up = if light_direction.y.abs() > 0.99 {
                    nalgebra_glm::vec3(1.0, 0.0, 0.0)
                } else {
                    nalgebra_glm::vec3(0.0, 1.0, 0.0)
                };

                for (cascade_index, &cascade_far) in
                    super::types::CASCADE_SPLIT_DISTANCES.iter().enumerate()
                {
                    let half_size = cascade_far * 0.5;

                    let scene_center = nalgebra_glm::vec3(0.0, 0.0, 0.0);
                    let light_position = scene_center - light_direction * 100.0;
                    let light_view = nalgebra_glm::look_at(&light_position, &scene_center, &up);
                    let light_projection = reverse_z_ortho_light(
                        -half_size,
                        half_size,
                        -half_size,
                        half_size,
                        0.1,
                        cascade_far * 2.0,
                    );

                    self.cascade_data.view_projections[cascade_index] =
                        (light_projection * light_view).into();
                }
            }

            let first_cascade_vp: nalgebra_glm::Mat4 = self.cascade_data.view_projections[0].into();
            let uniforms = ShadowUniforms {
                light_view_projection: first_cascade_vp.into(),
                _padding: [[0.0; 4]; 4],
                _padding2: [[0.0; 4]; 4],
                _padding3: [[0.0; 4]; 4],
                _padding4: [0.0; 4],
            };

            queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

            for cascade_index in 0..NUM_SHADOW_CASCADES {
                let cascade_vp: nalgebra_glm::Mat4 =
                    self.cascade_data.view_projections[cascade_index].into();
                let cascade_uniforms = ShadowUniforms {
                    light_view_projection: cascade_vp.into(),
                    _padding: [[0.0; 4]; 4],
                    _padding2: [[0.0; 4]; 4],
                    _padding3: [[0.0; 4]; 4],
                    _padding4: [0.0; 4],
                };
                queue.write_buffer(
                    &self.cascade_uniform_buffers[cascade_index],
                    0,
                    bytemuck::cast_slice(&[cascade_uniforms]),
                );
            }
        }

        let visibility_signature = {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            for entity in &self.all_caster_entities {
                caster_visible(configs, entity).hash(&mut hasher);
            }
            hasher.finish()
        };
        let needs_occluder_rebuild = !self.occluders_initialized
            || self.shadow_scene_dirty
            || visibility_signature != self.last_visible_signature;
        self.shadow_scene_dirty = false;

        if needs_occluder_rebuild {
            self.shadow_caster_entities.clear();
            self.all_caster_entities.clear();
            self.occluders.clear();
            self.occluder_transforms.clear();

            struct Caster {
                entity: freecs::Entity,
                transform: nalgebra_glm::Mat4,
                visible: bool,
                mesh_name: String,
                overlay: bool,
            }
            let casters: Vec<Caster> = configs
                .scene_world
                .query_ref::<(
                    &crate::render_world::Transform,
                    &crate::render_world::MeshName,
                    &crate::render_world::Visible,
                    &crate::render_world::Layer,
                )>()
                .with_tag_type::<crate::render_world::ShadowCaster>()
                .iter()
                .map(|(entity, (transform, mesh, visible, layer))| Caster {
                    entity,
                    transform: transform.0,
                    visible: visible.0,
                    mesh_name: mesh.0.clone(),
                    overlay: layer.0 == crate::render_layer::RenderLayer::OVERLAY,
                })
                .collect();
            for caster in casters {
                if caster.overlay {
                    continue;
                }
                let Some(&mesh_geo_id) = self.culling.name_to_geo_id.get(&caster.mesh_name) else {
                    continue;
                };
                self.all_caster_entities.push(caster.entity);
                if !caster.visible {
                    continue;
                }
                let model_matrix: [[f32; 4]; 4] = caster.transform.into();
                let transform_index = self.occluder_transforms.len() as u32;
                self.occluder_transforms.push(model_matrix);
                self.occluders.push(ShadowOccluder {
                    transform_index,
                    mesh_geo_id,
                    batch_id: 0,
                    _pad: 0,
                });
                self.shadow_caster_entities.push(caster.entity);
            }

            let instanced_casters: Vec<(freecs::Entity, crate::config::InstancedObjectData)> =
                configs
                    .scene_world
                    .query_ref::<&crate::config::InstancedObjectData>()
                    .with_tag_type::<crate::render_world::ShadowCaster>()
                    .iter()
                    .map(|(entity, instanced)| (entity, instanced.clone()))
                    .collect();
            for (entity, instanced) in &instanced_casters {
                let entity = *entity;
                if instanced.render_layer == crate::render_layer::RenderLayer::OVERLAY {
                    continue;
                }
                let Some(&mesh_geo_id) = self.culling.name_to_geo_id.get(&instanced.mesh_name)
                else {
                    continue;
                };
                self.all_caster_entities.push(entity);
                if instanced.visible == 0 {
                    continue;
                }
                for model in &instanced.world_models {
                    let transform_index = self.occluder_transforms.len() as u32;
                    self.occluder_transforms.push(*model);
                    self.occluders.push(ShadowOccluder {
                        transform_index,
                        mesh_geo_id,
                        batch_id: 0,
                        _pad: 0,
                    });
                    self.shadow_caster_entities.push(entity);
                }
            }

            if !self.occluder_transforms.is_empty() {
                if grow_storage_buffer::<[[f32; 4]; 4]>(
                    device,
                    &mut self.transform_buffer,
                    &mut self.transform_buffer_size,
                    self.occluder_transforms.len(),
                    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    "Shadow Transform Buffer (Resized)",
                ) {
                    self.transform_bind_group =
                        device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Shadow Transform Bind Group"),
                            layout: &self.transform_bind_group_layout,
                            entries: &[wgpu::BindGroupEntry {
                                binding: 0,
                                resource: self.transform_buffer.as_entire_binding(),
                            }],
                        });
                }

                queue.write_buffer(
                    &self.transform_buffer,
                    0,
                    bytemuck::cast_slice(&self.occluder_transforms),
                );
            }

            let mut signature_hasher = std::collections::hash_map::DefaultHasher::new();
            {
                use std::hash::{Hash, Hasher};
                for entity in &self.all_caster_entities {
                    caster_visible(configs, entity).hash(&mut signature_hasher);
                }
                self.last_visible_signature = signature_hasher.finish();
            }
            self.occluders_initialized = true;
        }

        self.sync_skinned_meshes(device, queue, &configs.mesh_cache);

        self.skinned_shadow_caster_entities.clear();
        let mut skinned_objects = Vec::new();
        let mut skinned_bounds: Vec<[f32; 4]> = Vec::new();

        let skinning_snapshot = &configs.scene.render_skinning;
        let skinning_cache = &skinning_snapshot.cache;

        let mut buffers_resized = false;

        let max_buffer_size = device.limits().max_buffer_size as usize;
        let matrix_size = std::mem::size_of::<[[f32; 4]; 4]>();

        if !skinning_snapshot.bone_transforms.is_empty() {
            let required_size = skinning_snapshot.bone_transforms.len();
            if required_size > self.bone_transforms_buffer_size {
                let new_size = (required_size as f32 * 2.0).ceil() as usize;
                let buffer_bytes = matrix_size * new_size;
                if buffer_bytes > max_buffer_size {
                    tracing::error!(
                        "Shadow bone transforms buffer would exceed GPU limit: {} > {} bytes",
                        buffer_bytes,
                        max_buffer_size
                    );
                } else {
                    self.bone_transforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Shadow Bone Transforms Buffer (Resized)"),
                        size: buffer_bytes as u64,
                        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
                    self.bone_transforms_buffer_size = new_size;
                    buffers_resized = true;
                }
            }
            if skinning_snapshot.bone_transforms.len() <= self.bone_transforms_buffer_size {
                queue.write_buffer(
                    &self.bone_transforms_buffer,
                    0,
                    bytemuck::cast_slice(&skinning_snapshot.bone_transforms),
                );
            }
        }

        if !skinning_cache.inverse_bind_matrices.is_empty() {
            let required_size = skinning_cache.inverse_bind_matrices.len();
            if required_size > self.inverse_bind_matrices_buffer_size {
                let new_size = (required_size as f32 * 2.0).ceil() as usize;
                let buffer_bytes = matrix_size * new_size;
                if buffer_bytes > max_buffer_size {
                    tracing::error!(
                        "Shadow inverse bind matrices buffer would exceed GPU limit: {} > {} bytes",
                        buffer_bytes,
                        max_buffer_size
                    );
                } else {
                    self.inverse_bind_matrices_buffer =
                        device.create_buffer(&wgpu::BufferDescriptor {
                            label: Some("Shadow Inverse Bind Matrices Buffer (Resized)"),
                            size: buffer_bytes as u64,
                            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                            mapped_at_creation: false,
                        });
                    self.inverse_bind_matrices_buffer_size = new_size;
                    buffers_resized = true;
                }
            }
            if skinning_cache.inverse_bind_matrices.len() <= self.inverse_bind_matrices_buffer_size
            {
                queue.write_buffer(
                    &self.inverse_bind_matrices_buffer,
                    0,
                    bytemuck::cast_slice(&skinning_cache.inverse_bind_matrices),
                );
            }
        }

        if !skinning_cache.skin_data.is_empty() {
            let required_size = skinning_cache.skin_data.len();
            if required_size > self.skin_data_buffer_size {
                let new_size = (required_size as f32 * 2.0).ceil() as usize;
                let buffer_bytes = std::mem::size_of::<GpuSkinData>() * new_size;
                if buffer_bytes > max_buffer_size {
                    tracing::error!(
                        "Shadow skin data buffer would exceed GPU limit: {} > {} bytes",
                        buffer_bytes,
                        max_buffer_size
                    );
                } else {
                    self.skin_data_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Shadow Skin Data Buffer (Resized)"),
                        size: buffer_bytes as u64,
                        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
                    self.skin_data_buffer_size = new_size;
                    buffers_resized = true;
                }
            }
            if skinning_cache.skin_data.len() <= self.skin_data_buffer_size {
                queue.write_buffer(
                    &self.skin_data_buffer,
                    0,
                    bytemuck::cast_slice(&skinning_cache.skin_data),
                );
            }
        }

        let required_joints = skinning_cache.total_joints as usize;
        if required_joints > self.joint_matrices_buffer_size {
            let new_size = (required_joints as f32 * 2.0).ceil() as usize;
            let buffer_bytes = matrix_size * new_size;
            if buffer_bytes > max_buffer_size {
                tracing::error!(
                    "Shadow joint matrices buffer would exceed GPU limit: {} > {} bytes",
                    buffer_bytes,
                    max_buffer_size
                );
            } else {
                self.joint_matrices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Shadow Joint Matrices Buffer (Resized)"),
                    size: buffer_bytes as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                self.joint_matrices_buffer_size = new_size;
                buffers_resized = true;
                self.rebuild_skinned_bind_group(device);
            }
        }

        if buffers_resized {
            self.rebuild_skinning_compute_bind_group(device);
        }

        self.total_joints_to_dispatch = skinning_cache.total_joints;

        for &entity in &configs.scene.render_shadow_casters.skinned {
            let Some(dynamic) = configs
                .scene
                .render_skinned_dynamic_state
                .get(&entity)
                .copied()
            else {
                continue;
            };

            if dynamic.render_layer == crate::render_layer::RenderLayer::OVERLAY {
                continue;
            }

            if dynamic.visible == 0 {
                continue;
            }

            if let Some(mesh_name) = configs.scene.render_skinned_mesh_names.get(&entity)
                && let Some(&(vertex_offset, _vertex_count, index_offset, index_count)) =
                    self.skinned_meshes.get(mesh_name)
            {
                let skin_index = skinning_cache
                    .entity_skin_indices
                    .get(&entity)
                    .copied()
                    .unwrap_or(0);
                let joint_offset = skinning_cache.get_joint_offset(skin_index);

                skinned_objects.push(SkinnedShadowObjectData {
                    transform_index: 0,
                    mesh_id: (index_offset << 16) | index_count,
                    material_id: vertex_offset,
                    joint_offset,
                });
                let margin = configs
                    .scene_world
                    .get::<crate::config::RenderBounds>(entity)
                    .map(|volume| volume.sphere_radius)
                    .unwrap_or(2.0);
                let bound = skinning_cache
                    .skin_data
                    .get(skin_index as usize)
                    .map(|skin| {
                        crate::skinning::skinned_world_bounds(
                            &skinning_snapshot.bone_transforms,
                            skin.base_bone_index as usize,
                            skin.joint_count as usize,
                            margin,
                        )
                    })
                    .unwrap_or([0.0, 0.0, 0.0, 1.0e9]);
                skinned_bounds.push(bound);
                self.skinned_shadow_caster_entities.push(entity);
            }
        }

        let shadows_enabled = configs.scene.active_view.shadow_depth_enabled;
        let was_enabled = self.was_enabled_last_frame;

        if needs_occluder_rebuild {
            let occluder_transforms = &self.occluder_transforms;
            let mesh_bounds = &self.culling.mesh_bounds;
            let cache: Vec<OccluderCacheEntry> = self
                .occluders
                .iter()
                .map(|occluder| {
                    let model = occluder_transforms[occluder.transform_index as usize];
                    let model_matrix: nalgebra_glm::Mat4 = model.into();
                    let bounds = mesh_bounds[occluder.mesh_geo_id as usize];
                    let local_center = nalgebra_glm::vec4(
                        bounds.center[0],
                        bounds.center[1],
                        bounds.center[2],
                        1.0,
                    );
                    let world_center = model_matrix * local_center;
                    let scale_x = (model[0][0] * model[0][0]
                        + model[0][1] * model[0][1]
                        + model[0][2] * model[0][2])
                        .sqrt();
                    let scale_y = (model[1][0] * model[1][0]
                        + model[1][1] * model[1][1]
                        + model[1][2] * model[1][2])
                        .sqrt();
                    let scale_z = (model[2][0] * model[2][0]
                        + model[2][1] * model[2][1]
                        + model[2][2] * model[2][2])
                        .sqrt();
                    let max_scale = scale_x.max(scale_y).max(scale_z);
                    OccluderCacheEntry {
                        center: nalgebra_glm::vec3(world_center.x, world_center.y, world_center.z),
                        radius: bounds.radius * max_scale,
                        hash: hash_occluder(occluder.mesh_geo_id, &model),
                    }
                })
                .collect();
            self.occluder_cache = cache;
        }

        self.spotlight_shadow_slots.clear();
        self.spotlight_shadow_data.clear();

        let camera_position = configs
            .scene
            .render_view
            .as_ref()
            .map(|view| view.camera_position)
            .unwrap_or_else(|| nalgebra_glm::vec3(0.0, 0.0, 0.0));

        // Demand-driven: drop shadow lights whose lit volume is entirely outside the
        // camera frustum, so a scene can declare far more shadow casters than fit and
        // only the visible ones consume slots. Disabled under tiled multi-viewport,
        // where the shared shadow maps must serve every tile's camera.
        let camera_frustum = if configs.view.camera_tile_rects.is_empty() {
            configs.scene.render_view.as_ref().map(|matrices| {
                let view_projection: [[f32; 4]; 4] = (matrices.projection * matrices.view).into();
                full_frustum_planes(&view_projection)
            })
        } else {
            None
        };

        let assignment = &configs.shadow_atlas;
        self.spotlight_atlas_slot_size = assignment.default_slot_size;

        for slot in &assignment.slots {
            let view_projection_array: [[f32; 4]; 4] = slot.view_projection.into();
            let frustum = full_frustum_planes(&view_projection_array);
            let mut content_hash = hash_view_key(bytemuck::bytes_of(&view_projection_array));
            content_hash = content_hash.wrapping_add(hash_view_key(bytemuck::bytes_of(&[
                slot.atlas_x_pixels,
                slot.atlas_y_pixels,
                slot.atlas_size_pixels,
            ])));
            for entry in &self.occluder_cache {
                if sphere_in_frustum(entry.center, entry.radius, &frustum) {
                    content_hash = content_hash.wrapping_add(entry.hash);
                }
            }
            let mut skinned_dirty = false;
            for bound in &skinned_bounds {
                let center = nalgebra_glm::vec3(bound[0], bound[1], bound[2]);
                if sphere_in_frustum(center, bound[3], &frustum) {
                    skinned_dirty = true;
                    break;
                }
            }
            let cache = &mut self.spotlight_slot_cache[slot.slot_index as usize];
            let clean = shadows_enabled
                && was_enabled
                && !skinned_dirty
                && cache.occupant == Some(slot.entity)
                && cache.hash == content_hash;
            *cache = ShadowSlotCache {
                occupant: Some(slot.entity),
                hash: content_hash,
            };

            self.spotlight_shadow_slots.push(SpotlightShadowSlot {
                entity: slot.entity,
                view_projection: slot.view_projection,
                slot_index: slot.slot_index,
                bias: slot.bias,
                atlas_x_pixels: slot.atlas_x_pixels,
                atlas_y_pixels: slot.atlas_y_pixels,
                atlas_size_pixels: slot.atlas_size_pixels,
                clean,
            });
        }

        self.spotlight_shadow_data = assignment.data.clone();

        self.point_light_shadow_slots.clear();
        self.point_light_shadow_data.clear();

        let mut point_light_candidates: Vec<(
            freecs::Entity,
            f32,
            crate::config::RenderLightData,
            nalgebra_glm::Mat4,
        )> = Vec::new();

        let point_lights: Vec<(
            freecs::Entity,
            crate::config::RenderLightData,
            nalgebra_glm::Mat4,
        )> = configs
            .scene_world
            .query_ref::<(
                &crate::config::RenderLightData,
                &crate::render_world::Transform,
            )>()
            .iter()
            .map(|(entity, (light, transform))| (entity, light.clone(), transform.0))
            .collect();
        for (entity, light, transform) in &point_lights {
            let entity = *entity;
            if matches!(light.light_type, crate::config::RenderLightType::Point)
                && light.cast_shadows
            {
                let light_pos =
                    nalgebra_glm::vec3(transform[(0, 3)], transform[(1, 3)], transform[(2, 3)]);
                let distance_sq = nalgebra_glm::length2(&(light_pos - camera_position));
                if light.shadow_distance > 0.0
                    && distance_sq > light.shadow_distance * light.shadow_distance
                {
                    continue;
                }
                if let Some(planes) = &camera_frustum
                    && !sphere_in_frustum(light_pos, light.range.max(1.0), planes)
                {
                    continue;
                }
                point_light_candidates.push((entity, distance_sq, light.clone(), *transform));
            }
        }

        point_light_candidates
            .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        for (slot_index, (entity, _distance, light, transform)) in point_light_candidates
            .iter()
            .take(MAX_POINT_LIGHT_SHADOWS)
            .enumerate()
        {
            let light_position =
                nalgebra_glm::vec3(transform[(0, 3)], transform[(1, 3)], transform[(2, 3)]);

            let range = light.range.max(0.1);
            let point_key = [light_position.x, light_position.y, light_position.z, range];
            let mut content_hash = hash_view_key(bytemuck::bytes_of(&point_key));
            for entry in &self.occluder_cache {
                if sphere_intersects_cube(entry.center, entry.radius, light_position, range) {
                    content_hash = content_hash.wrapping_add(entry.hash);
                }
            }
            let mut skinned_dirty = false;
            for bound in &skinned_bounds {
                let center = nalgebra_glm::vec3(bound[0], bound[1], bound[2]);
                if sphere_intersects_cube(center, bound[3], light_position, range) {
                    skinned_dirty = true;
                    break;
                }
            }
            let cache = &mut self.point_slot_cache[slot_index];
            let clean = shadows_enabled
                && was_enabled
                && !skinned_dirty
                && cache.occupant == Some(*entity)
                && cache.hash == content_hash;
            *cache = ShadowSlotCache {
                occupant: Some(*entity),
                hash: content_hash,
            };

            self.point_light_shadow_slots.push(PointLightShadowSlot {
                entity: *entity,
                position: light_position,
                range: light.range.max(0.1),
                slot_index: slot_index as u32,
                bias: light.shadow_bias,
                clean,
            });

            self.point_light_shadow_data.push(PointLightShadowData {
                position: [light_position.x, light_position.y, light_position.z],
                range: light.range.max(0.1),
                bias: light.shadow_bias,
                shadow_index: slot_index as i32,
                _padding: [0.0; 2],
            });
        }

        for index in self.spotlight_shadow_slots.len()..MAX_SPOTLIGHT_SHADOWS {
            self.spotlight_slot_cache[index] = ShadowSlotCache::default();
        }
        for index in self.point_light_shadow_slots.len()..MAX_POINT_LIGHT_SHADOWS {
            self.point_slot_cache[index] = ShadowSlotCache::default();
        }
        self.was_enabled_last_frame = shadows_enabled;

        if !self.point_light_shadow_slots.is_empty() {
            let mut all_uniforms: Vec<PointShadowUniforms> = Vec::new();

            for slot in &self.point_light_shadow_slots {
                let light_pos = slot.position;
                let light_range = slot.range;
                let near = 0.1;

                let base_projection =
                    reverse_z_perspective(std::f32::consts::FRAC_PI_2, 1.0, near, light_range);

                let projection = Y_FLIP_MATRIX * base_projection;

                for (direction, up) in &CUBEMAP_FACE_DIRECTIONS {
                    let target = light_pos + *direction;
                    let view = nalgebra_glm::look_at(&light_pos, &target, up);
                    let view_projection = projection * view;

                    let mut uniform: PointShadowUniforms = bytemuck::Zeroable::zeroed();
                    uniform.view_projection = view_projection.into();
                    uniform.light_position = [light_pos.x, light_pos.y, light_pos.z];
                    uniform.light_range = light_range;
                    all_uniforms.push(uniform);
                }
            }

            queue.write_buffer(
                &self.point_light_uniform_buffer,
                0,
                bytemuck::cast_slice(&all_uniforms),
            );
        }

        let num_spotlight_views = self.spotlight_shadow_slots.len();
        let num_point_lights = self.point_light_shadow_slots.len();
        let view_count =
            NUM_SHADOW_CASCADES + num_spotlight_views + num_point_lights * POINT_SHADOW_NUM_FACES;

        self.shadow_view_dirty.clear();
        for _ in 0..NUM_SHADOW_CASCADES {
            self.shadow_view_dirty.push(true);
        }
        for slot in &self.spotlight_shadow_slots {
            self.shadow_view_dirty.push(!slot.clean);
        }
        for slot in &self.point_light_shadow_slots {
            for _ in 0..POINT_SHADOW_NUM_FACES {
                self.shadow_view_dirty.push(!slot.clean);
            }
        }

        self.culling.prepare_frame(
            device,
            queue,
            &mut self.occluders,
            &self.transform_buffer,
            view_count,
            needs_occluder_rebuild,
        );

        for cascade in 0..NUM_SHADOW_CASCADES {
            let planes = cascade_side_planes(&self.cascade_data.view_projections[cascade]);
            self.culling.set_view_frustum(queue, cascade, planes);
        }
        for (spotlight_index, slot) in self.spotlight_shadow_slots.iter().enumerate() {
            let view_projection: [[f32; 4]; 4] = slot.view_projection.into();
            let planes = full_frustum_planes(&view_projection);
            self.culling
                .set_view_frustum(queue, NUM_SHADOW_CASCADES + spotlight_index, planes);
        }
        for (light_index, slot) in self.point_light_shadow_slots.iter().enumerate() {
            let base_projection =
                reverse_z_perspective(std::f32::consts::FRAC_PI_2, 1.0, 0.1, slot.range);
            let projection = Y_FLIP_MATRIX * base_projection;
            for (face_index, (direction, up)) in CUBEMAP_FACE_DIRECTIONS.iter().enumerate() {
                let target = slot.position + *direction;
                let view = nalgebra_glm::look_at(&slot.position, &target, up);
                let view_projection: [[f32; 4]; 4] = (projection * view).into();
                let view_id = NUM_SHADOW_CASCADES
                    + num_spotlight_views
                    + light_index * POINT_SHADOW_NUM_FACES
                    + face_index;
                let planes = full_frustum_planes(&view_projection);
                self.culling.set_view_frustum(queue, view_id, planes);
            }
        }

        self.prepare_skinned_cull(
            device,
            queue,
            skinned_objects,
            skinned_bounds,
            num_spotlight_views,
        );
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, crate::wgpu::render_configs::RenderInputs>,
    ) -> crate::wgpu::rendergraph::Result<Vec<crate::wgpu::rendergraph::SubGraphRunCommand<'r>>>
    {
        if !context.configs.scene.active_view.shadow_depth_enabled {
            return Ok(context.into_sub_graph_commands());
        }

        let has_directional_shadow = context
            .configs
            .scene_world
            .query_ref::<&crate::config::RenderLightData>()
            .iter()
            .any(|(_, light)| {
                matches!(
                    light.light_type,
                    crate::config::RenderLightType::Directional
                ) && light.cast_shadows
            });

        let has_spotlight_shadows = !self.spotlight_shadow_slots.is_empty();
        let has_point_light_shadows = !self.point_light_shadow_slots.is_empty();

        if !has_directional_shadow && !has_spotlight_shadows && !has_point_light_shadows {
            return Ok(vec![]);
        }

        if self.total_joints_to_dispatch > 0 && !self.skinned_shadow_caster_entities.is_empty() {
            let mut compute_pass =
                context
                    .encoder
                    .begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("Shadow Skinning Compute Pass"),
                        timestamp_writes: None,
                    });

            compute_pass.set_pipeline(&self.skinning_compute_pipeline);
            compute_pass.set_bind_group(0, &self.skinning_compute_bind_group, &[]);

            let num_workgroups = self.total_joints_to_dispatch.div_ceil(64);
            compute_pass.dispatch_workgroups(num_workgroups, 1, 1);
        }

        let (shadow_depth_view, _, _) = context.get_depth_attachment("shadow_depth")?;

        let (shadow_atlas_width, _) = context.get_texture_size("shadow_depth")?;
        let atlas_size = shadow_atlas_width as f32;
        let cascade_slot_size = atlas_size / CASCADE_ATLAS_SLOTS_PER_ROW as f32;

        let supports_multi_draw = context
            .device
            .features()
            .contains(wgpu::Features::MULTI_DRAW_INDIRECT_COUNT);
        self.culling
            .dispatch_cull(context.encoder, &self.shadow_view_dirty);

        if self.skinned_occluder_count > 0 && self.skinned_group_count > 0 {
            let mut cull_pass = context
                .encoder
                .begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Shadow Skinned Cull Pass"),
                    timestamp_writes: None,
                });
            cull_pass.set_pipeline(&self.skinned_cull_pipeline);
            let workgroups = (self.skinned_occluder_count as u32).div_ceil(64);
            for view in 0..self.skinned_cull_view_count {
                cull_pass.set_bind_group(0, &self.skinned_cull_bind_groups[view], &[]);
                cull_pass.dispatch_workgroups(workgroups, 1, 1);
            }
        }

        for cascade_index in 0..NUM_SHADOW_CASCADES {
            let slot_x = (cascade_index as u32) % CASCADE_ATLAS_SLOTS_PER_ROW;
            let slot_y = (cascade_index as u32) / CASCADE_ATLAS_SLOTS_PER_ROW;

            let load_op = if cascade_index == 0 {
                wgpu::LoadOp::Clear(0.0)
            } else {
                wgpu::LoadOp::Load
            };

            let mut shadow_pass = context
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Shadow Depth Pass"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: shadow_depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: load_op,
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });

            shadow_pass.set_viewport(
                slot_x as f32 * cascade_slot_size,
                slot_y as f32 * cascade_slot_size,
                cascade_slot_size,
                cascade_slot_size,
                0.0,
                1.0,
            );

            shadow_pass.set_scissor_rect(
                (slot_x as f32 * cascade_slot_size) as u32,
                (slot_y as f32 * cascade_slot_size) as u32,
                cascade_slot_size as u32,
                cascade_slot_size as u32,
            );

            shadow_pass.set_pipeline(&self.culling.indirect_pipeline);
            shadow_pass.set_bind_group(0, &self.cascade_uniform_bind_groups[cascade_index], &[]);
            shadow_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            shadow_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            self.culling
                .draw_view(&mut shadow_pass, cascade_index, supports_multi_draw);

            if self.skinned_group_count > 0 {
                shadow_pass.set_pipeline(&self.skinned_pipeline);
                shadow_pass.set_bind_group(
                    0,
                    &self.cascade_uniform_bind_groups[cascade_index],
                    &[],
                );
                shadow_pass.set_bind_group(1, &self.skinned_bind_group, &[]);
                shadow_pass.set_vertex_buffer(0, self.skinned_vertex_buffer.slice(..));
                shadow_pass.set_index_buffer(
                    self.skinned_index_buffer.slice(..),
                    wgpu::IndexFormat::Uint32,
                );

                let stride = std::mem::size_of::<ShadowDrawIndexedIndirect>() as u64;
                let base = (cascade_index * self.skinned_group_count) as u64 * stride;
                let count = self.skinned_group_count as u32;
                if supports_multi_draw {
                    shadow_pass.multi_draw_indexed_indirect(
                        &self.skinned_indirect_buffer,
                        base,
                        count,
                    );
                } else {
                    for command in 0..count as u64 {
                        shadow_pass.draw_indexed_indirect(
                            &self.skinned_indirect_buffer,
                            base + command * stride,
                        );
                    }
                }
            }
        }

        if has_spotlight_shadows {
            let (spotlight_atlas_view, _, _) =
                context.get_depth_attachment("spotlight_shadow_atlas")?;

            let (spotlight_atlas_width, _) = context.get_texture_size("spotlight_shadow_atlas")?;
            let _ = spotlight_atlas_width;

            for slot in &self.spotlight_shadow_slots.clone() {
                if slot.clean {
                    continue;
                }
                let uniforms = ShadowUniforms {
                    light_view_projection: slot.view_projection.into(),
                    _padding: [[0.0; 4]; 4],
                    _padding2: [[0.0; 4]; 4],
                    _padding3: [[0.0; 4]; 4],
                    _padding4: [0.0; 4],
                };

                let slot_uniform_buffer =
                    context
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Spotlight Shadow Uniform Buffer"),
                            contents: bytemuck::cast_slice(&[uniforms]),
                            usage: wgpu::BufferUsages::UNIFORM,
                        });

                let slot_uniform_bind_group =
                    context
                        .device
                        .create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Spotlight Shadow Uniform Bind Group"),
                            layout: &self.uniform_bind_group_layout,
                            entries: &[wgpu::BindGroupEntry {
                                binding: 0,
                                resource: slot_uniform_buffer.as_entire_binding(),
                            }],
                        });

                let load_op = wgpu::LoadOp::Load;

                let mut spotlight_pass =
                    context
                        .encoder
                        .begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Spotlight Shadow Pass"),
                            color_attachments: &[],
                            depth_stencil_attachment: Some(
                                wgpu::RenderPassDepthStencilAttachment {
                                    view: spotlight_atlas_view,
                                    depth_ops: Some(wgpu::Operations {
                                        load: load_op,
                                        store: wgpu::StoreOp::Store,
                                    }),
                                    stencil_ops: None,
                                },
                            ),
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        });

                spotlight_pass.set_viewport(
                    slot.atlas_x_pixels as f32,
                    slot.atlas_y_pixels as f32,
                    slot.atlas_size_pixels as f32,
                    slot.atlas_size_pixels as f32,
                    0.0,
                    1.0,
                );

                spotlight_pass.set_scissor_rect(
                    slot.atlas_x_pixels,
                    slot.atlas_y_pixels,
                    slot.atlas_size_pixels,
                    slot.atlas_size_pixels,
                );

                spotlight_pass.set_pipeline(&self.depth_clear_pipeline);
                spotlight_pass.draw(0..3, 0..1);

                spotlight_pass.set_pipeline(&self.culling.indirect_pipeline);
                spotlight_pass.set_bind_group(0, &slot_uniform_bind_group, &[]);
                spotlight_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                spotlight_pass
                    .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                self.culling.draw_view(
                    &mut spotlight_pass,
                    NUM_SHADOW_CASCADES + slot.slot_index as usize,
                    supports_multi_draw,
                );

                let skinned_view_id = NUM_SHADOW_CASCADES + slot.slot_index as usize;
                if self.skinned_group_count > 0 && skinned_view_id < self.skinned_cull_view_count {
                    spotlight_pass.set_pipeline(&self.skinned_pipeline);
                    spotlight_pass.set_bind_group(0, &slot_uniform_bind_group, &[]);
                    spotlight_pass.set_bind_group(1, &self.skinned_bind_group, &[]);
                    spotlight_pass.set_vertex_buffer(0, self.skinned_vertex_buffer.slice(..));
                    spotlight_pass.set_index_buffer(
                        self.skinned_index_buffer.slice(..),
                        wgpu::IndexFormat::Uint32,
                    );

                    let stride = std::mem::size_of::<ShadowDrawIndexedIndirect>() as u64;
                    let base = (skinned_view_id * self.skinned_group_count) as u64 * stride;
                    let count = self.skinned_group_count as u32;
                    if supports_multi_draw {
                        spotlight_pass.multi_draw_indexed_indirect(
                            &self.skinned_indirect_buffer,
                            base,
                            count,
                        );
                    } else {
                        for command in 0..count as u64 {
                            spotlight_pass.draw_indexed_indirect(
                                &self.skinned_indirect_buffer,
                                base + command * stride,
                            );
                        }
                    }
                }
            }
        }

        if has_point_light_shadows {
            let num_spotlight_views = self.spotlight_shadow_slots.len();
            for slot in &self.point_light_shadow_slots {
                if slot.clean {
                    continue;
                }
                for face_index in 0..POINT_SHADOW_NUM_FACES {
                    let uniform_index =
                        slot.slot_index as usize * POINT_SHADOW_NUM_FACES + face_index;
                    let dynamic_offset =
                        (uniform_index * std::mem::size_of::<PointShadowUniforms>()) as u32;

                    let face_view_index =
                        slot.slot_index as usize * POINT_SHADOW_NUM_FACES + face_index;
                    let face_view = &self.point_light_face_views[face_view_index];

                    let mut point_pass =
                        context
                            .encoder
                            .begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some(&format!(
                                    "Point Light {} Face {} Shadow Pass",
                                    slot.slot_index, face_index
                                )),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: face_view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color {
                                            r: 1.0,
                                            g: 1.0,
                                            b: 1.0,
                                            a: 1.0,
                                        }),
                                        store: wgpu::StoreOp::Store,
                                    },
                                    depth_slice: None,
                                })],
                                depth_stencil_attachment: Some(
                                    wgpu::RenderPassDepthStencilAttachment {
                                        view: &self.point_light_depth_view,
                                        depth_ops: Some(wgpu::Operations {
                                            load: wgpu::LoadOp::Clear(0.0),
                                            store: wgpu::StoreOp::Discard,
                                        }),
                                        stencil_ops: None,
                                    },
                                ),
                                timestamp_writes: None,
                                occlusion_query_set: None,
                                multiview_mask: None,
                            });

                    point_pass.set_viewport(
                        0.0,
                        0.0,
                        POINT_SHADOW_FACE_SIZE as f32,
                        POINT_SHADOW_FACE_SIZE as f32,
                        0.0,
                        1.0,
                    );

                    point_pass.set_pipeline(&self.culling.point_indirect_pipeline);
                    point_pass.set_bind_group(
                        0,
                        &self.point_light_uniform_bind_group,
                        &[dynamic_offset],
                    );
                    point_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                    point_pass
                        .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                    let view_id = NUM_SHADOW_CASCADES
                        + num_spotlight_views
                        + slot.slot_index as usize * POINT_SHADOW_NUM_FACES
                        + face_index;
                    self.culling
                        .draw_view(&mut point_pass, view_id, supports_multi_draw);
                }
            }
        }

        Ok(vec![])
    }
}

impl ShadowDepthPass {
    fn prepare_skinned_cull(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        skinned_objects: Vec<SkinnedShadowObjectData>,
        skinned_bounds: Vec<[f32; 4]>,
        num_spotlight_views: usize,
    ) {
        self.skinned_group_count = 0;
        self.skinned_occluder_count = 0;
        self.skinned_cull_view_count = 0;
        if skinned_objects.is_empty() {
            return;
        }

        let mut order: Vec<usize> = (0..skinned_objects.len()).collect();
        order.sort_by_key(|&index| {
            (
                skinned_objects[index].mesh_id,
                skinned_objects[index].material_id,
            )
        });
        let objects: Vec<SkinnedShadowObjectData> =
            order.iter().map(|&index| skinned_objects[index]).collect();
        let bounds: Vec<[f32; 4]> = order.iter().map(|&index| skinned_bounds[index]).collect();

        let mut command_index_per_object = vec![0u32; objects.len()];
        let mut group_meshes: Vec<(u32, u32, u32)> = Vec::new();
        let mut index = 0;
        while index < objects.len() {
            let mesh_id = objects[index].mesh_id;
            let material_id = objects[index].material_id;
            let start = index as u32;
            while index < objects.len()
                && objects[index].mesh_id == mesh_id
                && objects[index].material_id == material_id
            {
                command_index_per_object[index] = group_meshes.len() as u32;
                index += 1;
            }
            group_meshes.push((mesh_id, material_id, start));
        }

        let group_count = group_meshes.len();
        let occluder_count = objects.len();
        let view_count = NUM_SHADOW_CASCADES + num_spotlight_views;

        let cull_objects: Vec<SkinnedShadowCullObject> = (0..occluder_count)
            .map(|object_index| SkinnedShadowCullObject {
                bounds: bounds[object_index],
                command_index: command_index_per_object[object_index],
                _pad: [0; 3],
            })
            .collect();

        let mut commands: Vec<ShadowDrawIndexedIndirect> =
            Vec::with_capacity(view_count * group_count);
        for view in 0..view_count {
            for &(mesh_id, material_id, group_start) in &group_meshes {
                commands.push(ShadowDrawIndexedIndirect {
                    index_count: mesh_id & 0xFFFF,
                    instance_count: 0,
                    first_index: mesh_id >> 16,
                    base_vertex: material_id as i32,
                    first_instance: (view * occluder_count) as u32 + group_start,
                });
            }
        }

        let mut bind_groups_dirty = false;
        if grow_storage_buffer::<SkinnedShadowObjectData>(
            device,
            &mut self.skinned_object_buffer,
            &mut self.skinned_object_buffer_size,
            occluder_count,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            "Shadow Skinned Object Buffer (Resized)",
        ) {
            bind_groups_dirty = true;
        }
        if grow_storage_buffer::<SkinnedShadowCullObject>(
            device,
            &mut self.skinned_cull_objects_buffer,
            &mut self.skinned_cull_objects_buffer_size,
            cull_objects.len(),
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            "Shadow Skinned Cull Objects Buffer (Resized)",
        ) {
            bind_groups_dirty = true;
        }
        if occluder_count * view_count > self.skinned_visible_indices_buffer_size {
            let new_size = (occluder_count * view_count * 2).max(64);
            self.skinned_visible_indices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Shadow Skinned Visible Indices Buffer (Resized)"),
                size: (std::mem::size_of::<u32>() * new_size) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.skinned_visible_indices_buffer_size = new_size;
            bind_groups_dirty = true;
        }
        if commands.len() > self.skinned_indirect_buffer_size {
            let new_size = (commands.len() * 2).max(32);
            self.skinned_indirect_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Shadow Skinned Indirect Buffer (Resized)"),
                size: (std::mem::size_of::<ShadowDrawIndexedIndirect>() * new_size) as u64,
                usage: wgpu::BufferUsages::INDIRECT
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.skinned_indirect_buffer_size = new_size;
            bind_groups_dirty = true;
        }
        if bind_groups_dirty {
            self.rebuild_skinned_bind_group(device);
            self.rebuild_skinned_cull_bind_groups(device);
        }

        queue.write_buffer(
            &self.skinned_object_buffer,
            0,
            bytemuck::cast_slice(&objects),
        );
        queue.write_buffer(
            &self.skinned_cull_objects_buffer,
            0,
            bytemuck::cast_slice(&cull_objects),
        );
        queue.write_buffer(
            &self.skinned_indirect_buffer,
            0,
            bytemuck::cast_slice(&commands),
        );

        for view in 0..view_count {
            let planes = if view < NUM_SHADOW_CASCADES {
                cascade_side_planes(&self.cascade_data.view_projections[view])
            } else {
                let spotlight_index = view - NUM_SHADOW_CASCADES;
                let view_projection: [[f32; 4]; 4] = self.spotlight_shadow_slots[spotlight_index]
                    .view_projection
                    .into();
                full_frustum_planes(&view_projection)
            };
            let uniforms = ShadowCullUniforms {
                frustum_planes: planes,
                occluder_count: occluder_count as u32,
                indirect_offset: (view * group_count) as u32,
                _pad0: 0,
                _pad1: 0,
            };
            queue.write_buffer(
                &self.skinned_cull_uniform_buffers[view],
                0,
                bytemuck::cast_slice(&[uniforms]),
            );
        }

        self.skinned_group_count = group_count;
        self.skinned_occluder_count = occluder_count;
        self.skinned_cull_view_count = view_count;
    }
}
