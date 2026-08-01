use nightshade_wgpu::backend::build;
type Mapping = (
    nightshade_render::data::asset_id::TextureId,
    nightshade_wgpu::backend::MaterialTextureLayer,
);
use nightshade_render::data::skinning::GpuSkinData;
use nightshade_wgpu::backend::{PassDecl, PassExecutionContext, PassNode, PassPosition};
use nightshade_wgpu::wgpu;
use wgpu::util::DeviceExt;

use super::cascade::reverse_z_perspective;
use super::pass::ShadowDepthPass;
use super::types::{
    CASCADE_ATLAS_SLOTS_PER_ROW, CASCADE_SLOT_RESOLUTION, MAX_POINT_LIGHT_SHADOWS,
    MAX_SPOTLIGHT_SHADOWS, NUM_SHADOW_CASCADES, POINT_SHADOW_FACE_SIZE, POINT_SHADOW_NUM_FACES,
    PointLightShadowData, PointLightShadowSlot, PointShadowUniforms, ShadowCullUniforms,
    ShadowDrawIndexedIndirect, ShadowOccluder, ShadowSlotCache, ShadowUniforms,
    SkinnedShadowCullObject, SkinnedShadowObjectData, SpotlightShadowSlot,
};
use nightshade_wgpu::backend::grow_storage_buffer;
use nightshade_wgpu_shading::backend::projection::cascade_view_projection_without_a_camera;
use nightshade_wgpu_shading::backend::projection::{full_frustum_planes, sphere_in_frustum};

pub(super) struct OccluderCacheEntry {
    pub(super) center: nalgebra_glm::Vec3,
    pub(super) radius: f32,
    pub(super) hash: u64,
}

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

fn cascade_side_planes(view_projection: &[[f32; 4]; 4]) -> [nalgebra_glm::Vec4; 6] {
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
        nalgebra_glm::vec4(
            plane[0] / length,
            plane[1] / length,
            plane[2] / length,
            plane[3] / length,
        )
    };
    let disabled = nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0e9);
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
    configs: &nightshade_wgpu::backend::RenderInputs,
    entity: &nightshade::ecs::Entity,
) -> bool {
    if let Some(visible) = configs
        .scene_world
        .get::<nightshade_render::components::render_world::Visible>(*entity)
    {
        return visible.0;
    }
    if let Some(instanced) = configs
        .scene_world
        .get::<nightshade_render::data::config::InstancedObjectData>(*entity)
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

const SHADOW_DEPTH_PASS_DECL: PassDecl = PassDecl {
    name: "shadow_depth_pass",
    position: PassPosition::Background,
    writes: &["shadow_depth", "spotlight_shadow_atlas"],
    optional_reads: &["bone_matrices"],
    ..PassDecl::EMPTY
};

impl PassNode<nightshade_wgpu::backend::RenderInputs> for ShadowDepthPass {
    fn decl(&self) -> &PassDecl {
        &SHADOW_DEPTH_PASS_DECL
    }

    fn invalidate_bind_groups(&mut self) {
        self.skinning.bone_bound = false;
    }

    fn consume(&mut self, reader: &nightshade_wgpu::backend::ResourceReader<'_>) {
        if let Some(buffer) = reader.buffer("bone_matrices") {
            self.set_bone_source(buffer.clone());
        }
    }

    fn publish(&mut self, publisher: &mut nightshade_wgpu::backend::ResourcePublisher<'_>) {
        if !self.point_lights.publish_pending {
            return;
        }
        self.point_lights.publish_pending = false;
        publisher.texture(
            "point_shadow_cubemap",
            None,
            self.point_lights.cubemap.clone(),
            POINT_SHADOW_FACE_SIZE,
            POINT_SHADOW_FACE_SIZE,
        );
    }

    fn material_update(
        &mut self,
        device: &wgpu::Device,
        update: &nightshade_wgpu::backend::MaterialUpdate<'_>,
    ) {
        match update {
            nightshade_wgpu::backend::MaterialUpdate::Register {
                name,
                view,
                sampler,
                mapping,
            } => {
                let _ = (name, view, sampler);
                if let Some(pair) = mapping.and_then(|value| value.downcast_ref::<Mapping>()) {
                    self.add_material_layer_mapping(pair.0, pair.1);
                }
            }
            nightshade_wgpu::backend::MaterialUpdate::Unregister { .. } => {}
            nightshade_wgpu::backend::MaterialUpdate::Mapping { mapping } => {
                if let Some(pair) = mapping.downcast_ref::<Mapping>() {
                    self.add_material_layer_mapping(pair.0, pair.1);
                }
            }
            nightshade_wgpu::backend::MaterialUpdate::Rebind { arrays } => {
                if let Some(arrays) =
                    arrays.downcast_ref::<nightshade_wgpu::backend::MaterialTextureArrays>()
                {
                    self.apply_material_textures(device, arrays);
                }
            }
        }
    }

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &nightshade_wgpu::backend::RenderInputs,
    ) {
        if let Some(frame_state) = configs.mesh_frame_state.as_ref() {
            self.shadow_scene_dirty |= frame_state.full_rebuild_needed
                || frame_state.instanced_meshes_changed
                || !frame_state.instanced_local_matrices_dirty.is_empty();
            self.note_dirty_meshes(&configs.dirty_meshes);
            self.note_frame_changes(frame_state);
        }
        let time = configs.view.uptime_milliseconds as f32 / 1000.0;
        let changes = std::mem::take(&mut self.occluders.frame_changes);
        let casting: Option<std::collections::HashSet<String>> =
            (self.compaction_due() || !self.occluders.initialized).then(|| {
                configs
                    .scene_world
                    .query_ref::<(
                        &nightshade_render::components::render_world::MeshName,
                        &nightshade_render::components::render_world::Visible,
                    )>()
                    .with_tag_type::<nightshade_render::components::render_world::ShadowCaster>()
                    .iter()
                    .filter(|(_, (_, visible))| visible.0)
                    .map(|(_, (mesh, _))| mesh.0.clone())
                    .collect()
            });
        let mut wanted_now: Vec<String> = Vec::new();
        for entity in &changes.added {
            if let Some(mesh) =
                configs
                    .scene_world
                    .get::<nightshade_render::components::render_world::MeshName>(*entity)
            {
                wanted_now.push(mesh.0.clone());
            }
        }
        self.sync_meshes(
            device,
            queue,
            &configs.mesh_cache,
            casting.as_ref(),
            &wanted_now,
            configs.settings.named_shadow_casters,
        );

        let directional_light = configs
            .scene_world
            .query_ref::<(
                &nightshade_render::components::light::RenderLightData,
                &nightshade_render::components::render_world::Transform,
            )>()
            .iter()
            .find_map(|(_entity, (light, transform))| {
                if matches!(
                    light.light_type,
                    nightshade_render::data::config::RenderLightType::Directional
                ) && light.cast_shadows
                {
                    Some((light.clone(), transform.0))
                } else {
                    None
                }
            });

        if let Some((_light, light_transform)) = directional_light {
            let light_direction =
                nightshade_wgpu_shading::backend::projection::transform_forward(&light_transform);

            let camera_matrices = configs.scene.render_view.as_ref();

            if let Some(camera_matrices) = camera_matrices {
                let fit = nightshade_render::queries::projection::fit_cascades(
                    &camera_matrices.view,
                    &camera_matrices.projection_params(),
                    &light_direction,
                    CASCADE_SLOT_RESOLUTION,
                );
                self.cascades.data.view_projections = fit.view_projections;
                self.cascades.data.split_distances = fit.split_distances;
            } else {
                for (cascade_index, &cascade_far) in
                    super::types::CASCADE_SPLIT_DISTANCES.iter().enumerate()
                {
                    self.cascades.data.view_projections[cascade_index] =
                        cascade_view_projection_without_a_camera(&light_direction, cascade_far)
                            .into();
                }
            }

            let first_cascade_vp: nalgebra_glm::Mat4 =
                self.cascades.data.view_projections[0].into();
            let uniforms = ShadowUniforms {
                light_view_projection: first_cascade_vp.into(),
                time,
                padding_time: [0.0; 3],
                padding: [[0.0; 4]; 4],
                padding_2: [[0.0; 4]; 4],
                padding_3: [[0.0; 4]; 4],
            };

            queue.write_buffer(
                &self.transforms.uniforms,
                0,
                bytemuck::cast_slice(&[uniforms]),
            );

            for cascade_index in 0..NUM_SHADOW_CASCADES {
                let cascade_vp: nalgebra_glm::Mat4 =
                    self.cascades.data.view_projections[cascade_index].into();
                let cascade_uniforms = ShadowUniforms {
                    light_view_projection: cascade_vp.into(),
                    time,
                    padding_time: [0.0; 3],
                    padding: [[0.0; 4]; 4],
                    padding_2: [[0.0; 4]; 4],
                    padding_3: [[0.0; 4]; 4],
                };
                queue.write_buffer(
                    &self.cascades.uniforms[cascade_index],
                    0,
                    bytemuck::cast_slice(&[cascade_uniforms]),
                );
            }
        }

        let mesh_generation = self.culling.mesh_table_generation;
        let materials_changed = configs.scene.render_materials.generation
            != self.materials.generation
            || self.materials.layer_generation != self.materials.row_layer_generation;
        if materials_changed {
            self.rebuild_shadow_materials(device, queue, &configs.scene.render_materials);
        }
        let classes_changed =
            configs.scene.render_materials.class_generation != self.materials.class_generation;
        let table_reset = mesh_generation != self.occluders.mesh_generation;
        let instanced_touched = changes
            .added
            .iter()
            .chain(&changes.removed)
            .chain(&changes.moved)
            .chain(&changes.state_dirty)
            .any(|entity| {
                self.occluders.instanced_casters.contains(entity)
                    || configs
                        .scene_world
                        .get::<nightshade_render::data::config::InstancedObjectData>(*entity)
                        .is_some()
            });
        let needs_occluder_rebuild = !self.occluders.initialized
            || classes_changed
            || self.shadow_scene_dirty
            || table_reset
            || instanced_touched;
        self.shadow_scene_dirty = false;

        let (upload, transforms_grew) = if needs_occluder_rebuild {
            self.rebuild_occluders(configs);
            self.occluders.initialized = true;
            self.occluders.mesh_generation = mesh_generation;
            self.materials.class_generation = configs.scene.render_materials.class_generation;
            let grew = self.upload_all_occluder_transforms(device, queue);
            (super::types::OccluderUpload::All, grew)
        } else {
            self.apply_occluder_changes(device, queue, configs, &changes)
        };

        self.sync_skinned_meshes(device, queue, &configs.mesh_cache);

        self.skinned_geometry.casters.clear();
        let mut skinned_objects = Vec::new();
        let mut skinned_bounds: Vec<[f32; 4]> = Vec::new();

        let skinning_snapshot = &configs.scene.render_skinning;
        let skinning_cache = &skinning_snapshot.cache;

        let mut buffers_resized = false;

        let max_buffer_size = device.limits().max_buffer_size as usize;
        let matrix_size = std::mem::size_of::<[[f32; 4]; 4]>();

        if !self.skinning.bone_source_external && !skinning_snapshot.bone_transforms.is_empty() {
            let required_size = skinning_snapshot.bone_transforms.len();
            if required_size > self.skinning.bone_transforms_size {
                let new_size = (required_size as f32 * 2.0).ceil() as usize;
                let buffer_bytes = matrix_size * new_size;
                if buffer_bytes > max_buffer_size {
                    tracing::error!(
                        "Shadow bone transforms buffer would exceed GPU limit: {} > {} bytes",
                        buffer_bytes,
                        max_buffer_size
                    );
                } else {
                    self.skinning.bone_transforms = build::storage(
                        device,
                        "Shadow Bone Transforms Buffer (Resized)",
                        buffer_bytes as u64,
                    );
                    self.skinning.bone_transforms_size = new_size;
                    buffers_resized = true;
                }
            }
            if skinning_snapshot.bone_transforms.len() <= self.skinning.bone_transforms_size {
                queue.write_buffer(
                    &self.skinning.bone_transforms,
                    0,
                    bytemuck::cast_slice(&skinning_snapshot.bone_transforms),
                );
            }
        }

        if !skinning_cache.inverse_bind_matrices.is_empty() {
            let required_size = skinning_cache.inverse_bind_matrices.len();
            if required_size > self.skinning.inverse_binds_size {
                let new_size = (required_size as f32 * 2.0).ceil() as usize;
                let buffer_bytes = matrix_size * new_size;
                if buffer_bytes > max_buffer_size {
                    tracing::error!(
                        "Shadow inverse bind matrices buffer would exceed GPU limit: {} > {} bytes",
                        buffer_bytes,
                        max_buffer_size
                    );
                } else {
                    self.skinning.inverse_binds = build::storage(
                        device,
                        "Shadow Inverse Bind Matrices Buffer (Resized)",
                        buffer_bytes as u64,
                    );
                    self.skinning.inverse_binds_size = new_size;
                    buffers_resized = true;
                }
            }
            if skinning_cache.inverse_bind_matrices.len() <= self.skinning.inverse_binds_size {
                queue.write_buffer(
                    &self.skinning.inverse_binds,
                    0,
                    bytemuck::cast_slice(&skinning_cache.inverse_bind_matrices),
                );
            }
        }

        if !skinning_cache.skin_data.is_empty() {
            let required_size = skinning_cache.skin_data.len();
            if required_size > self.skinning.skin_data_size {
                let new_size = (required_size as f32 * 2.0).ceil() as usize;
                let buffer_bytes = std::mem::size_of::<GpuSkinData>() * new_size;
                if buffer_bytes > max_buffer_size {
                    tracing::error!(
                        "Shadow skin data buffer would exceed GPU limit: {} > {} bytes",
                        buffer_bytes,
                        max_buffer_size
                    );
                } else {
                    self.skinning.skin_data = build::storage(
                        device,
                        "Shadow Skin Data Buffer (Resized)",
                        buffer_bytes as u64,
                    );
                    self.skinning.skin_data_size = new_size;
                    buffers_resized = true;
                }
            }
            if skinning_cache.skin_data.len() <= self.skinning.skin_data_size {
                queue.write_buffer(
                    &self.skinning.skin_data,
                    0,
                    bytemuck::cast_slice(&skinning_cache.skin_data),
                );
            }
        }

        let required_joints = skinning_cache.total_joints as usize;
        if required_joints > self.skinning.joints_size {
            let new_size = (required_joints as f32 * 2.0).ceil() as usize;
            let buffer_bytes = matrix_size * new_size;
            if buffer_bytes > max_buffer_size {
                tracing::error!(
                    "Shadow joint matrices buffer would exceed GPU limit: {} > {} bytes",
                    buffer_bytes,
                    max_buffer_size
                );
            } else {
                self.skinning.joints = build::storage(
                    device,
                    "Shadow Joint Matrices Buffer (Resized)",
                    buffer_bytes as u64,
                );
                self.skinning.joints_size = new_size;
                buffers_resized = true;
                self.rebuild_skinned_bind_group(device);
            }
        }

        if buffers_resized || !self.skinning.bone_bound {
            self.rebuild_skinning_compute_bind_group(device);
            self.skinning.bone_bound = true;
        }

        self.skinning.joints_to_dispatch = skinning_cache.total_joints;

        for &entity in &configs.scene.render_shadow_casters.skinned {
            let Some(dynamic) = configs
                .scene
                .render_skinned_dynamic_state
                .get(&entity)
                .copied()
            else {
                continue;
            };

            if dynamic.render_layer == nightshade_render::data::render_layer::RenderLayer::OVERLAY {
                continue;
            }

            if dynamic.visible == 0 {
                continue;
            }

            if let Some(mesh_name) = configs.scene.render_skinned_mesh_names.get(&entity)
                && let Some(&(vertex_offset, _vertex_count, index_offset, index_count)) =
                    self.skinned_geometry.ids.get(mesh_name)
            {
                let skin_index = skinning_cache
                    .entity_skin_indices
                    .get(&entity)
                    .copied()
                    .unwrap_or(0);
                let joint_offset = skinning_cache.get_joint_offset(skin_index);

                skinned_objects.push(SkinnedShadowObjectData {
                    index_offset,
                    index_count,
                    vertex_offset,
                    joint_offset,
                    material_index: self.shadow_material_index(configs, entity),
                    padding: [0; 3],
                });
                let margin = configs
                    .scene_world
                    .get::<nightshade_render::data::config::RenderBounds>(entity)
                    .map(|volume| volume.sphere_radius)
                    .unwrap_or(2.0);
                skinned_bounds.push(nightshade_render::data::skinning::skinned_root_bounds(
                    &dynamic.transform,
                    margin,
                ));
                self.skinned_geometry.casters.push(entity);
            }
        }

        let shadows_enabled = true;
        let was_enabled = self.was_enabled_last_frame;

        self.spotlights.slots.clear();
        self.spotlights.data.clear();

        let assignment = &configs.owned.shadow_atlas;
        self.spotlights.atlas_slot_size = assignment.default_slot_size;

        for slot in &assignment.slots {
            let view_projection_array: [[f32; 4]; 4] = slot.view_projection.into();
            let frustum = full_frustum_planes(&view_projection_array);
            let mut content_hash = hash_view_key(bytemuck::bytes_of(&view_projection_array));
            content_hash = content_hash.wrapping_add(hash_view_key(bytemuck::bytes_of(&[
                slot.atlas_x_pixels,
                slot.atlas_y_pixels,
                slot.atlas_size_pixels,
            ])));
            for entry in &self.occluders.cache {
                if sphere_in_frustum(&entry.center, entry.radius, &frustum) {
                    content_hash = content_hash.wrapping_add(entry.hash);
                }
            }
            let mut skinned_dirty = false;
            for bound in &skinned_bounds {
                let center = nalgebra_glm::vec3(bound[0], bound[1], bound[2]);
                if sphere_in_frustum(&center, bound[3], &frustum) {
                    skinned_dirty = true;
                    break;
                }
            }
            let cache = &mut self.spotlights.slot_cache[slot.slot_index as usize];
            let clean = shadows_enabled
                && was_enabled
                && !skinned_dirty
                && cache.occupant == Some(slot.entity)
                && cache.hash == content_hash;
            *cache = ShadowSlotCache {
                occupant: Some(slot.entity),
                hash: content_hash,
            };

            self.spotlights.slots.push(SpotlightShadowSlot {
                view_projection: slot.view_projection,
                slot_index: slot.slot_index,
                atlas_x_pixels: slot.atlas_x_pixels,
                atlas_y_pixels: slot.atlas_y_pixels,
                atlas_size_pixels: slot.atlas_size_pixels,
                clean,
            });
        }

        self.spotlights.data = assignment.data.clone();

        self.point_lights.slots.clear();
        self.point_lights.data.clear();

        for chosen in &configs.owned.point_shadows {
            let slot_index = chosen.slot_index as usize;
            let entity = &chosen.entity;
            let light = &chosen.light;
            let transform = &chosen.transform;
            let light_position =
                nalgebra_glm::vec3(transform[(0, 3)], transform[(1, 3)], transform[(2, 3)]);

            let range = light.range.max(0.1);
            let point_key = [light_position.x, light_position.y, light_position.z, range];
            let mut content_hash = hash_view_key(bytemuck::bytes_of(&point_key));
            for entry in &self.occluders.cache {
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
            let cache = &mut self.point_lights.slot_cache[slot_index];
            let clean = shadows_enabled
                && was_enabled
                && !skinned_dirty
                && cache.occupant == Some(*entity)
                && cache.hash == content_hash;
            *cache = ShadowSlotCache {
                occupant: Some(*entity),
                hash: content_hash,
            };

            self.point_lights.slots.push(PointLightShadowSlot {
                position: light_position,
                range: light.range.max(0.1),
                slot_index: slot_index as u32,
                clean,
            });

            self.point_lights.data.push(PointLightShadowData {
                position: [light_position.x, light_position.y, light_position.z],
                range: light.range.max(0.1),
                bias: light.shadow_bias,
                shadow_index: slot_index as i32,
                padding: [0.0; 2],
            });
        }

        for index in self.spotlights.slots.len()..MAX_SPOTLIGHT_SHADOWS {
            self.spotlights.slot_cache[index] = ShadowSlotCache::default();
        }
        for index in self.point_lights.slots.len()..MAX_POINT_LIGHT_SHADOWS {
            self.point_lights.slot_cache[index] = ShadowSlotCache::default();
        }
        self.was_enabled_last_frame = shadows_enabled;

        if !self.point_lights.slots.is_empty() {
            let mut all_uniforms: Vec<PointShadowUniforms> = Vec::new();

            for slot in &self.point_lights.slots {
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
                    uniform.time = time;
                    all_uniforms.push(uniform);
                }
            }

            queue.write_buffer(
                &self.point_lights.uniforms,
                0,
                bytemuck::cast_slice(&all_uniforms),
            );
        }

        let num_spotlight_views = self.spotlights.slots.len();
        let num_point_lights = self.point_lights.slots.len();
        let view_count =
            NUM_SHADOW_CASCADES + num_spotlight_views + num_point_lights * POINT_SHADOW_NUM_FACES;

        self.shadow_view_dirty.clear();
        for _ in 0..NUM_SHADOW_CASCADES {
            self.shadow_view_dirty.push(true);
        }
        for slot in &self.spotlights.slots {
            self.shadow_view_dirty.push(!slot.clean);
        }
        for slot in &self.point_lights.slots {
            for _ in 0..POINT_SHADOW_NUM_FACES {
                self.shadow_view_dirty.push(!slot.clean);
            }
        }

        self.culling.prepare_frame(
            device,
            queue,
            &mut self.occluders.rows,
            &self.transforms.buffer,
            &self.materials.buffer,
            view_count,
            upload,
            materials_changed || transforms_grew,
        );

        for cascade in 0..NUM_SHADOW_CASCADES {
            let planes = cascade_side_planes(&self.cascades.data.view_projections[cascade]);
            self.culling.set_view_frustum(queue, cascade, planes);
        }
        for (spotlight_index, slot) in self.spotlights.slots.iter().enumerate() {
            let view_projection: [[f32; 4]; 4] = slot.view_projection.into();
            let planes = full_frustum_planes(&view_projection);
            self.culling
                .set_view_frustum(queue, NUM_SHADOW_CASCADES + spotlight_index, planes);
        }
        for (light_index, slot) in self.point_lights.slots.iter().enumerate() {
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
        context: PassExecutionContext<'r, 'e, nightshade_wgpu::backend::RenderInputs>,
    ) -> nightshade_wgpu::backend::Result<()> {
        let Some(material_bind_group) = self.materials.bind_group.clone() else {
            return Ok(());
        };
        let has_directional_shadow = context
            .configs
            .scene_world
            .query_ref::<&nightshade_render::components::light::RenderLightData>()
            .iter()
            .any(|(_, light)| {
                matches!(
                    light.light_type,
                    nightshade_render::data::config::RenderLightType::Directional
                ) && light.cast_shadows
            });

        let has_spotlight_shadows = !self.spotlights.slots.is_empty();
        let has_point_light_shadows = !self.point_lights.slots.is_empty();

        if !has_directional_shadow && !has_spotlight_shadows && !has_point_light_shadows {
            return Ok(());
        }

        if self.skinning.joints_to_dispatch > 0 && !self.skinned_geometry.casters.is_empty() {
            let mut compute_pass =
                context
                    .encoder
                    .begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("Shadow Skinning Compute Pass"),
                        timestamp_writes: None,
                    });

            compute_pass.set_pipeline(&self.skinning.pipeline);
            compute_pass.set_bind_group(0, &self.skinning.bind_group, &[]);

            let num_workgroups = self.skinning.joints_to_dispatch.div_ceil(64);
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

        if self.skinned_draw.occluder_count > 0 && self.skinned_draw.group_count > 0 {
            let mut cull_pass = context
                .encoder
                .begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Shadow Skinned Cull Pass"),
                    timestamp_writes: None,
                });
            cull_pass.set_pipeline(&self.skinned_draw.cull_pipeline);
            let workgroups = (self.skinned_draw.occluder_count as u32).div_ceil(64);
            for view in 0..self.skinned_draw.cull_view_count {
                cull_pass.set_bind_group(0, &self.skinned_draw.cull_groups[view], &[]);
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
            shadow_pass.set_bind_group(0, &self.cascades.uniform_groups[cascade_index], &[]);
            shadow_pass.set_bind_group(2, &material_bind_group, &[]);
            shadow_pass.set_vertex_buffer(0, self.geometry.vertex.slice(..));
            shadow_pass.set_index_buffer(self.geometry.index.slice(..), wgpu::IndexFormat::Uint32);

            self.culling
                .draw_view(&mut shadow_pass, cascade_index, supports_multi_draw);

            if self.skinned_draw.group_count > 0 {
                shadow_pass.set_pipeline(&self.skinned_draw.pipeline);
                shadow_pass.set_bind_group(0, &self.cascades.uniform_groups[cascade_index], &[]);
                shadow_pass.set_bind_group(1, &self.skinned_draw.bind_group, &[]);
                shadow_pass.set_bind_group(2, &material_bind_group, &[]);
                shadow_pass.set_vertex_buffer(0, self.skinned_geometry.vertex.slice(..));
                shadow_pass.set_index_buffer(
                    self.skinned_geometry.index.slice(..),
                    wgpu::IndexFormat::Uint32,
                );

                let stride = std::mem::size_of::<ShadowDrawIndexedIndirect>() as u64;
                let base = (cascade_index * self.skinned_draw.group_count) as u64 * stride;
                let count = self.skinned_draw.group_count as u32;
                if supports_multi_draw {
                    shadow_pass.multi_draw_indexed_indirect(
                        &self.skinned_draw.indirect,
                        base,
                        count,
                    );
                } else {
                    for command in 0..count as u64 {
                        shadow_pass.draw_indexed_indirect(
                            &self.skinned_draw.indirect,
                            base + command * stride,
                        );
                    }
                }
            }
        }

        if has_spotlight_shadows {
            let (spotlight_atlas_view, _, _) =
                context.get_depth_attachment("spotlight_shadow_atlas")?;

            for slot in &self.spotlights.slots.clone() {
                if slot.clean {
                    continue;
                }
                let uniforms = ShadowUniforms {
                    light_view_projection: slot.view_projection.into(),
                    time: context.configs.view.uptime_milliseconds as f32 / 1000.0,
                    padding_time: [0.0; 3],
                    padding: [[0.0; 4]; 4],
                    padding_2: [[0.0; 4]; 4],
                    padding_3: [[0.0; 4]; 4],
                };

                let slot_uniform_buffer =
                    context
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Spotlight Shadow Uniform Buffer"),
                            contents: bytemuck::cast_slice(&[uniforms]),
                            usage: wgpu::BufferUsages::UNIFORM,
                        });

                let slot_uniform_bind_group = build::binding(
                    context.device,
                    "Spotlight Shadow Uniform Bind Group",
                    &self.transforms.uniform_layout,
                    vec![slot_uniform_buffer.as_entire_binding()],
                );

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
                spotlight_pass.set_bind_group(2, &material_bind_group, &[]);
                spotlight_pass.set_vertex_buffer(0, self.geometry.vertex.slice(..));
                spotlight_pass
                    .set_index_buffer(self.geometry.index.slice(..), wgpu::IndexFormat::Uint32);

                self.culling.draw_view(
                    &mut spotlight_pass,
                    NUM_SHADOW_CASCADES + slot.slot_index as usize,
                    supports_multi_draw,
                );

                let skinned_view_id = NUM_SHADOW_CASCADES + slot.slot_index as usize;
                if self.skinned_draw.group_count > 0
                    && skinned_view_id < self.skinned_draw.cull_view_count
                {
                    spotlight_pass.set_pipeline(&self.skinned_draw.pipeline);
                    spotlight_pass.set_bind_group(0, &slot_uniform_bind_group, &[]);
                    spotlight_pass.set_bind_group(1, &self.skinned_draw.bind_group, &[]);
                    spotlight_pass.set_bind_group(2, &material_bind_group, &[]);
                    spotlight_pass.set_vertex_buffer(0, self.skinned_geometry.vertex.slice(..));
                    spotlight_pass.set_index_buffer(
                        self.skinned_geometry.index.slice(..),
                        wgpu::IndexFormat::Uint32,
                    );

                    let stride = std::mem::size_of::<ShadowDrawIndexedIndirect>() as u64;
                    let base = (skinned_view_id * self.skinned_draw.group_count) as u64 * stride;
                    let count = self.skinned_draw.group_count as u32;
                    if supports_multi_draw {
                        spotlight_pass.multi_draw_indexed_indirect(
                            &self.skinned_draw.indirect,
                            base,
                            count,
                        );
                    } else {
                        for command in 0..count as u64 {
                            spotlight_pass.draw_indexed_indirect(
                                &self.skinned_draw.indirect,
                                base + command * stride,
                            );
                        }
                    }
                }
            }
        }

        if has_point_light_shadows {
            let num_spotlight_views = self.spotlights.slots.len();
            for slot in &self.point_lights.slots {
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
                    let face_view = &self.point_lights.faces[face_view_index];

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
                                        view: &self.point_lights.depth,
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
                    point_pass.set_bind_group(2, &material_bind_group, &[]);
                    point_pass.set_bind_group(
                        0,
                        &self.point_lights.uniform_group,
                        &[dynamic_offset],
                    );
                    point_pass.set_vertex_buffer(0, self.geometry.vertex.slice(..));
                    point_pass
                        .set_index_buffer(self.geometry.index.slice(..), wgpu::IndexFormat::Uint32);

                    let view_id = NUM_SHADOW_CASCADES
                        + num_spotlight_views
                        + slot.slot_index as usize * POINT_SHADOW_NUM_FACES
                        + face_index;
                    self.culling
                        .draw_view(&mut point_pass, view_id, supports_multi_draw);
                }
            }
        }

        Ok(())
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
        self.skinned_draw.group_count = 0;
        self.skinned_draw.occluder_count = 0;
        self.skinned_draw.cull_view_count = 0;
        if skinned_objects.is_empty() {
            return;
        }

        let mut order: Vec<usize> = (0..skinned_objects.len()).collect();
        order.sort_by_key(|&index| {
            (
                skinned_objects[index].index_offset,
                skinned_objects[index].index_count,
                skinned_objects[index].vertex_offset,
            )
        });
        let objects: Vec<SkinnedShadowObjectData> =
            order.iter().map(|&index| skinned_objects[index]).collect();
        let bounds: Vec<[f32; 4]> = order.iter().map(|&index| skinned_bounds[index]).collect();

        let mut command_index_per_object = vec![0u32; objects.len()];
        let mut group_meshes: Vec<(u32, u32, u32, u32)> = Vec::new();
        let mut index = 0;
        while index < objects.len() {
            let index_offset = objects[index].index_offset;
            let index_count = objects[index].index_count;
            let vertex_offset = objects[index].vertex_offset;
            let start = index as u32;
            while index < objects.len()
                && objects[index].index_offset == index_offset
                && objects[index].index_count == index_count
                && objects[index].vertex_offset == vertex_offset
            {
                command_index_per_object[index] = group_meshes.len() as u32;
                index += 1;
            }
            group_meshes.push((index_offset, index_count, vertex_offset, start));
        }

        let group_count = group_meshes.len();
        let occluder_count = objects.len();
        let view_count = NUM_SHADOW_CASCADES + num_spotlight_views;

        let cull_objects: Vec<SkinnedShadowCullObject> = (0..occluder_count)
            .map(|object_index| SkinnedShadowCullObject {
                bounds: bounds[object_index],
                command_index: command_index_per_object[object_index],
                padding: [0; 3],
            })
            .collect();

        let mut commands: Vec<ShadowDrawIndexedIndirect> =
            Vec::with_capacity(view_count * group_count);
        for view in 0..view_count {
            for &(index_offset, index_count, vertex_offset, group_start) in &group_meshes {
                commands.push(ShadowDrawIndexedIndirect {
                    index_count,
                    instance_count: 0,
                    first_index: index_offset,
                    base_vertex: vertex_offset as i32,
                    first_instance: (view * occluder_count) as u32 + group_start,
                });
            }
        }

        let mut bind_groups_dirty = false;
        if grow_storage_buffer::<SkinnedShadowObjectData>(
            device,
            &mut self.skinned_draw.objects,
            &mut self.skinned_draw.objects_size,
            occluder_count,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            "Shadow Skinned Object Buffer (Resized)",
        ) {
            bind_groups_dirty = true;
        }
        if grow_storage_buffer::<SkinnedShadowCullObject>(
            device,
            &mut self.skinned_draw.cull_objects,
            &mut self.skinned_draw.cull_objects_size,
            cull_objects.len(),
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            "Shadow Skinned Cull Objects Buffer (Resized)",
        ) {
            bind_groups_dirty = true;
        }
        if occluder_count * view_count > self.skinned_draw.visible_indices_size {
            let new_size = (occluder_count * view_count * 2).max(64);
            self.skinned_draw.visible_indices = build::storage(
                device,
                "Shadow Skinned Visible Indices Buffer (Resized)",
                (std::mem::size_of::<u32>() * new_size) as u64,
            );
            self.skinned_draw.visible_indices_size = new_size;
            bind_groups_dirty = true;
        }
        if commands.len() > self.skinned_draw.indirect_size {
            let new_size = (commands.len() * 2).max(32);
            self.skinned_draw.indirect = build::buffer(
                device,
                "Shadow Skinned Indirect Buffer (Resized)",
                (std::mem::size_of::<ShadowDrawIndexedIndirect>() * new_size) as u64,
                wgpu::BufferUsages::INDIRECT
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
            );
            self.skinned_draw.indirect_size = new_size;
            bind_groups_dirty = true;
        }
        if bind_groups_dirty {
            self.rebuild_skinned_bind_group(device);
            self.rebuild_skinned_cull_bind_groups(device);
        }

        queue.write_buffer(
            &self.skinned_draw.objects,
            0,
            bytemuck::cast_slice(&objects),
        );
        queue.write_buffer(
            &self.skinned_draw.cull_objects,
            0,
            bytemuck::cast_slice(&cull_objects),
        );
        queue.write_buffer(
            &self.skinned_draw.indirect,
            0,
            bytemuck::cast_slice(&commands),
        );

        for view in 0..view_count {
            let planes = if view < NUM_SHADOW_CASCADES {
                cascade_side_planes(&self.cascades.data.view_projections[view])
            } else {
                let spotlight_index = view - NUM_SHADOW_CASCADES;
                let view_projection: [[f32; 4]; 4] = self.spotlights.slots[spotlight_index]
                    .view_projection
                    .into();
                full_frustum_planes(&view_projection)
            };
            let uniforms = ShadowCullUniforms {
                frustum_planes: planes.map(|plane| [plane.x, plane.y, plane.z, plane.w]),
                occluder_count: occluder_count as u32,
                indirect_offset: (view * group_count) as u32,
                padding_0: 0,
                padding_1: 0,
            };
            queue.write_buffer(
                &self.skinned_draw.cull_uniforms[view],
                0,
                bytemuck::cast_slice(&[uniforms]),
            );
        }

        self.skinned_draw.group_count = group_count;
        self.skinned_draw.occluder_count = occluder_count;
        self.skinned_draw.cull_view_count = view_count;
    }
}

impl ShadowDepthPass {
    fn cache_entry(&self, slot: usize) -> OccluderCacheEntry {
        let occluder = &self.occluders.rows[slot];
        let model = self.occluders.transforms[occluder.transform_index as usize];
        let model_matrix: nalgebra_glm::Mat4 = model.into();
        let bounds = self.culling.mesh_bounds[occluder.mesh_geo_id as usize];
        let local_center =
            nalgebra_glm::vec4(bounds.center[0], bounds.center[1], bounds.center[2], 1.0);
        let world_center = model_matrix * local_center;
        let scale_x =
            (model[0][0] * model[0][0] + model[0][1] * model[0][1] + model[0][2] * model[0][2])
                .sqrt();
        let scale_y =
            (model[1][0] * model[1][0] + model[1][1] * model[1][1] + model[1][2] * model[1][2])
                .sqrt();
        let scale_z =
            (model[2][0] * model[2][0] + model[2][1] * model[2][1] + model[2][2] * model[2][2])
                .sqrt();
        let max_scale = scale_x.max(scale_y).max(scale_z);
        OccluderCacheEntry {
            center: nalgebra_glm::vec3(world_center.x, world_center.y, world_center.z),
            radius: bounds.radius * max_scale,
            hash: hash_occluder(occluder.mesh_geo_id, &model),
        }
    }

    fn push_occluder(
        &mut self,
        entity: nightshade::ecs::Entity,
        mesh_geo_id: u32,
        model: [[f32; 4]; 4],
        material_index: u32,
    ) -> u32 {
        let slot = self.occluders.rows.len() as u32;
        self.occluders.transforms.push(model);
        self.occluders.rows.push(ShadowOccluder {
            transform_index: slot,
            mesh_geo_id,
            batch_id: 0,
            material_index,
        });
        self.occluders.casters.push(entity);
        let entry = self.cache_entry(slot as usize);
        self.occluders.cache.push(entry);
        slot
    }

    fn remove_occluder(&mut self, entity: nightshade::ecs::Entity) -> Option<Option<u32>> {
        let slot = self.occluders.slot_of.remove(&entity)? as usize;
        let last = self.occluders.rows.len() - 1;
        self.occluders.rows.swap_remove(slot);
        self.occluders.transforms.swap_remove(slot);
        self.occluders.casters.swap_remove(slot);
        self.occluders.cache.swap_remove(slot);
        if slot == last {
            return Some(None);
        }
        self.occluders.rows[slot].transform_index = slot as u32;
        let moved = self.occluders.casters[slot];
        if let Some(held) = self.occluders.slot_of.get_mut(&moved) {
            *held = slot as u32;
        }
        Some(Some(slot as u32))
    }

    fn try_add_caster(
        &mut self,
        configs: &nightshade_wgpu::backend::RenderInputs,
        entity: nightshade::ecs::Entity,
    ) -> Option<u32> {
        if self.occluders.slot_of.contains_key(&entity) {
            return None;
        }
        let world = &configs.scene_world;
        if !world.has_tag_type::<nightshade_render::components::render_world::ShadowCaster>(entity)
        {
            return None;
        }
        let (Some(transform), Some(mesh), Some(visible), Some(layer)) = (
            world.get::<nightshade_render::components::render_world::Transform>(entity),
            world.get::<nightshade_render::components::render_world::MeshKey>(entity),
            world.get::<nightshade_render::components::render_world::Visible>(entity),
            world.get::<nightshade_render::components::render_world::Layer>(entity),
        ) else {
            return None;
        };
        if layer.0 == nightshade_render::data::render_layer::RenderLayer::OVERLAY || !visible.0 {
            return None;
        }
        let Some(&mesh_geo_id) = self.culling.geo_id_of.get(&mesh.0) else {
            self.occluders.pending_casters.push(entity);
            return None;
        };
        let material_index = self.shadow_material_index(configs, entity);
        let model: [[f32; 4]; 4] = transform.0.into();
        let slot = self.push_occluder(entity, mesh_geo_id, model, material_index);
        self.occluders.slot_of.insert(entity, slot);
        Some(slot)
    }

    fn rebuild_occluders(&mut self, configs: &nightshade_wgpu::backend::RenderInputs) {
        self.occluders.casters.clear();
        self.occluders.rows.clear();
        self.occluders.transforms.clear();
        self.occluders.cache.clear();
        self.occluders.slot_of.clear();
        self.occluders.pending_casters.clear();
        self.occluders.instanced_casters.clear();
        self.occluders.seen_geo_count = self.culling.mesh_geo.len();

        struct Caster {
            entity: nightshade::ecs::Entity,
            transform: nalgebra_glm::Mat4,
            visible: bool,
            mesh_key: u64,
            overlay: bool,
        }
        let casters: Vec<Caster> = configs
            .scene_world
            .query_ref::<(
                &nightshade_render::components::render_world::Transform,
                &nightshade_render::components::render_world::MeshKey,
                &nightshade_render::components::render_world::Visible,
                &nightshade_render::components::render_world::Layer,
            )>()
            .with_tag_type::<nightshade_render::components::render_world::ShadowCaster>()
            .iter()
            .map(|(entity, (transform, mesh, visible, layer))| Caster {
                entity,
                transform: transform.0,
                visible: visible.0,
                mesh_key: mesh.0,
                overlay: layer.0 == nightshade_render::data::render_layer::RenderLayer::OVERLAY,
            })
            .collect();
        for caster in casters {
            if caster.overlay {
                continue;
            }
            let Some(&mesh_geo_id) = self.culling.geo_id_of.get(&caster.mesh_key) else {
                self.occluders.pending_casters.push(caster.entity);
                continue;
            };
            if !caster.visible {
                continue;
            }
            let model_matrix: [[f32; 4]; 4] = caster.transform.into();
            let material_index = self.shadow_material_index(configs, caster.entity);
            let slot = self.push_occluder(caster.entity, mesh_geo_id, model_matrix, material_index);
            self.occluders.slot_of.insert(caster.entity, slot);
        }

        let instanced_casters: Vec<(
            nightshade::ecs::Entity,
            nightshade_render::data::config::InstancedObjectData,
        )> = configs
            .scene_world
            .query_ref::<&nightshade_render::data::config::InstancedObjectData>()
            .with_tag_type::<nightshade_render::components::render_world::ShadowCaster>()
            .iter()
            .map(|(entity, instanced)| (entity, instanced.clone()))
            .collect();
        for (entity, instanced) in &instanced_casters {
            let entity = *entity;
            self.occluders.instanced_casters.insert(entity);
            if instanced.render_layer == nightshade_render::data::render_layer::RenderLayer::OVERLAY
            {
                continue;
            }
            let Some(&mesh_geo_id) = self.culling.geo_id_of.get(
                &nightshade_render::data::generational_registry::registry_name_hash(
                    &instanced.mesh_name,
                ),
            ) else {
                continue;
            };
            if instanced.visible == 0 {
                continue;
            }
            let material_index = self.shadow_material_index(configs, entity);
            for model in &instanced.world_models {
                self.push_occluder(entity, mesh_geo_id, *model, material_index);
            }
        }
    }

    fn upload_all_occluder_transforms(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> bool {
        if self.occluders.transforms.is_empty() {
            return false;
        }
        let grew = grow_storage_buffer::<[[f32; 4]; 4]>(
            device,
            &mut self.transforms.buffer,
            &mut self.transforms.buffer_size,
            self.occluders.transforms.len(),
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            "Shadow Transform Buffer (Resized)",
        );
        if grew {
            self.transforms.bind_group = build::binding(
                device,
                "Shadow Transform Bind Group",
                &self.transforms.layout,
                vec![self.transforms.buffer.as_entire_binding()],
            );
        }
        queue.write_buffer(
            &self.transforms.buffer,
            0,
            bytemuck::cast_slice(&self.occluders.transforms),
        );
        grew
    }

    fn apply_occluder_changes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &nightshade_wgpu::backend::RenderInputs,
        changes: &super::types::ShadowFrameChanges,
    ) -> (super::types::OccluderUpload, bool) {
        let mut touched: Vec<u32> = Vec::new();
        let mut set_changed = false;
        for entity in &changes.removed {
            if let Some(filled) = self.remove_occluder(*entity) {
                set_changed = true;
                touched.extend(filled);
            }
        }
        for entity in &changes.state_dirty {
            if caster_visible(configs, entity) {
                if let Some(slot) = self.try_add_caster(configs, *entity) {
                    set_changed = true;
                    touched.push(slot);
                }
            } else if let Some(filled) = self.remove_occluder(*entity) {
                set_changed = true;
                touched.extend(filled);
            }
        }
        for entity in &changes.added {
            if let Some(slot) = self.try_add_caster(configs, *entity) {
                set_changed = true;
                touched.push(slot);
            }
        }
        if self.culling.mesh_geo.len() != self.occluders.seen_geo_count {
            self.occluders.seen_geo_count = self.culling.mesh_geo.len();
            for entity in std::mem::take(&mut self.occluders.pending_casters) {
                if let Some(slot) = self.try_add_caster(configs, entity) {
                    set_changed = true;
                    touched.push(slot);
                }
            }
        }
        for entity in &changes.moved {
            if let Some(&slot) = self.occluders.slot_of.get(entity)
                && let Some(transform) =
                    configs
                        .scene_world
                        .get::<nightshade_render::components::render_world::Transform>(*entity)
            {
                self.occluders.transforms[slot as usize] = transform.0.into();
                self.occluders.cache[slot as usize] = self.cache_entry(slot as usize);
                touched.push(slot);
            }
        }
        for entity in &changes.material_dirty {
            if let Some(&slot) = self.occluders.slot_of.get(entity) {
                self.occluders.rows[slot as usize].material_index =
                    self.shadow_material_index(configs, *entity);
                touched.push(slot);
            }
        }
        touched.sort_unstable();
        touched.dedup();
        let count = self.occluders.rows.len();
        touched.retain(|slot| (*slot as usize) < count);

        let grew = if self.occluders.transforms.len() > self.transforms.buffer_size {
            self.upload_all_occluder_transforms(device, queue)
        } else {
            let stride = std::mem::size_of::<[[f32; 4]; 4]>() as u64;
            for &slot in &touched {
                queue.write_buffer(
                    &self.transforms.buffer,
                    slot as u64 * stride,
                    bytemuck::bytes_of(&self.occluders.transforms[slot as usize]),
                );
            }
            false
        };
        (
            super::types::OccluderUpload::Touched {
                slots: touched,
                set_changed,
            },
            grew,
        )
    }
}
