use super::super::types::{
    BUFFER_GROWTH_FACTOR, CLUSTER_GRID_X, CLUSTER_GRID_Y, CLUSTER_GRID_Z, ClusterUniforms,
    CullingUniforms, MAX_LIGHTS, MeshUniforms, cluster_slice_near,
};
use super::MeshPass;

impl MeshPass {
    pub(in super::super) fn prepare_uniforms_and_lights(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &crate::wgpu::render_configs::RenderInputs,
    ) {
        use super::super::super::projection::*;
        use crate::bounding_volume::extract_frustum_planes;

        let Some(lighting) = configs.scene.render_lighting.as_ref() else {
            return;
        };
        let mut lights_data = lighting.lights_data.clone();
        let num_directional_lights = lighting.num_directional_lights;
        let directional_light_direction = lighting.directional_light_direction;
        let entity_to_lights_index = lighting.entity_to_lights_index.clone();
        let mut area_lights_data = lighting.area_lights_data.clone();
        let area_entity_to_index = lighting.area_entity_to_index.clone();
        let has_directional_light = lighting.has_directional_light;
        let stashed_cascade_view_projections = lighting.cascade_view_projections;
        let stashed_cascade_diameters = lighting.cascade_diameters;
        let stashed_cascade_split_distances = lighting.cascade_split_distances;
        let stashed_light_view_projection = lighting.light_view_projection;
        let stashed_shadow_bias = lighting.shadow_bias;
        let stashed_shadow_normal_bias = lighting.shadow_normal_bias;
        let stashed_directional_light_size = lighting.directional_light_size;
        let stashed_shadows_enabled = lighting.shadows_enabled;
        {
            let state = self.world_states[self.current_world_id as usize]
                .as_mut()
                .unwrap();
            state.num_directional_lights = num_directional_lights;
            state.num_total_lights = lights_data.len() as u32;
        }

        let shadow_view_required =
            configs.scene.active_view.shadow_depth_enabled && has_directional_light;
        let (
            cascade_view_projections,
            cascade_diameters,
            cascade_split_distances,
            light_view_projection,
            shadow_bias,
            shadow_normal_bias,
            directional_light_size,
            shadows_enabled,
        ) = if shadow_view_required {
            (
                stashed_cascade_view_projections,
                stashed_cascade_diameters,
                stashed_cascade_split_distances,
                stashed_light_view_projection,
                stashed_shadow_bias,
                stashed_shadow_normal_bias,
                stashed_directional_light_size,
                stashed_shadows_enabled,
            )
        } else {
            (
                [[[0.0f32; 4]; 4]; crate::wgpu::passes::NUM_SHADOW_CASCADES],
                [0.0f32; crate::wgpu::passes::NUM_SHADOW_CASCADES],
                CASCADE_SPLIT_DISTANCES,
                [[0.0f32; 4]; 4],
                0.0,
                1.8,
                1.0,
                0.0,
            )
        };

        let cascade_texture_resolution = crate::wgpu::passes::CASCADE_SLOT_RESOLUTION;

        let cascade_atlas_offsets: [[f32; 4]; crate::wgpu::passes::NUM_SHADOW_CASCADES] = [
            [
                0.0,
                0.0,
                cascade_diameters[0] / cascade_texture_resolution,
                0.0,
            ],
            [
                0.5,
                0.0,
                cascade_diameters[1] / cascade_texture_resolution,
                0.0,
            ],
            [
                0.0,
                0.5,
                cascade_diameters[2] / cascade_texture_resolution,
                0.0,
            ],
            [
                0.5,
                0.5,
                cascade_diameters[3] / cascade_texture_resolution,
                0.0,
            ],
        ];

        if let Some(camera_matrices) = configs.scene.render_view.as_ref() {
            let global_unlit = if configs.scene.active_view.unlit_mode {
                1.0
            } else {
                0.0
            };

            let (snap_resolution, snap_enabled) =
                if let Some(ref vertex_snap) = configs.settings.vertex_snap {
                    (vertex_snap.resolution, 1)
                } else {
                    ([320.0, 240.0], 0)
                };

            let affine_enabled = if configs.settings.affine_texture_mapping {
                1
            } else {
                0
            };

            let (fog_color, fog_enabled, fog_start, fog_end) =
                if let Some(ref fog) = configs.scene.active_view.fog {
                    (fog.color, fog.mode.as_u32(), fog.start, fog.end)
                } else {
                    ([0.5, 0.5, 0.6], 0, 5.0, 30.0)
                };

            let time = configs.view.uptime_milliseconds as f32 / 1000.0;

            let camera_z_far = configs
                .scene
                .render_view
                .as_ref()
                .map(|view| view.z_far)
                .unwrap_or(1000.0);
            let oit_z_scale = (camera_z_far * 0.2).max(1.0);

            let uniforms = MeshUniforms {
                view: camera_matrices.view.into(),
                projection: camera_matrices.projection.into(),
                camera_position: [
                    camera_matrices.camera_position.x,
                    camera_matrices.camera_position.y,
                    camera_matrices.camera_position.z,
                    1.0,
                ],
                num_lights: [lights_data.len() as u32, 0, 0, 0],
                ambient_light: configs.scene.active_view.ambient_light,
                light_view_projection,
                shadow_bias,
                shadows_enabled,
                global_unlit,
                shadow_normal_bias,
                snap_resolution,
                snap_enabled,
                affine_enabled,
                fog_color,
                fog_enabled,
                fog_start,
                fog_end,
                cascade_count: crate::wgpu::passes::NUM_SHADOW_CASCADES as u32,
                directional_light_size,
                cascade_view_projections,
                cascade_split_distances,
                cascade_atlas_offsets,
                cascade_atlas_scale: [0.5, 0.5, 0.0, 0.0],
                time,
                pbr_debug_mode: configs.debug_draw.pbr_debug_mode.as_u32(),
                texture_debug_stripes: configs.debug_draw.texture_debug_stripes as u32,
                texture_debug_stripes_speed: configs.debug_draw.texture_debug_stripes_speed,
                directional_light_direction,
                ibl_blend_factor: 0.0,
                oit_z_scale,
                _pad_pre_flat: [0.0; 2],
                flat_color: configs
                    .scene
                    .active_view
                    .flat_shading_color
                    .map(|c| [c.x, c.y, c.z, c.w])
                    .unwrap_or([0.0; 4]),
                prev_view_projection: configs.scene.prev_view_projection,
                cur_view_projection: configs.scene.view_projection,
                material_filtering: configs.debug_draw.material_filtering as u32,
                _padding3: [0.0; 43],
            };
            queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

            let overlay_uniforms = MeshUniforms {
                view: camera_matrices.view.into(),
                projection: camera_matrices.projection.into(),
                camera_position: [
                    camera_matrices.camera_position.x,
                    camera_matrices.camera_position.y,
                    camera_matrices.camera_position.z,
                    1.0,
                ],
                num_lights: [lights_data.len() as u32, 0, 0, 0],
                ambient_light: configs.scene.active_view.ambient_light,
                light_view_projection,
                shadow_bias,
                shadows_enabled: 0.0,
                global_unlit,
                shadow_normal_bias,
                snap_resolution,
                snap_enabled,
                affine_enabled,
                fog_color,
                fog_enabled,
                fog_start,
                fog_end,
                cascade_count: crate::wgpu::passes::NUM_SHADOW_CASCADES as u32,
                directional_light_size,
                cascade_view_projections,
                cascade_split_distances,
                cascade_atlas_offsets,
                cascade_atlas_scale: [0.5, 0.5, 0.0, 0.0],
                time,
                pbr_debug_mode: configs.debug_draw.pbr_debug_mode.as_u32(),
                texture_debug_stripes: configs.debug_draw.texture_debug_stripes as u32,
                texture_debug_stripes_speed: configs.debug_draw.texture_debug_stripes_speed,
                directional_light_direction,
                ibl_blend_factor: 0.0,
                oit_z_scale,
                _pad_pre_flat: [0.0; 2],
                flat_color: configs
                    .scene
                    .active_view
                    .flat_shading_color
                    .map(|c| [c.x, c.y, c.z, c.w])
                    .unwrap_or([0.0; 4]),
                prev_view_projection: configs.scene.prev_view_projection,
                cur_view_projection: configs.scene.view_projection,
                material_filtering: configs.debug_draw.material_filtering as u32,
                _padding3: [0.0; 43],
            };
            queue.write_buffer(
                &self.overlay_uniform_buffer,
                0,
                bytemuck::cast_slice(&[overlay_uniforms]),
            );

            let view_proj = camera_matrices.projection * camera_matrices.view;
            let (culling_frustum_planes, culling_view_proj) =
                if let Some(culling_vp) = configs.scene.culling_camera_view_projection {
                    (extract_frustum_planes(&culling_vp), culling_vp)
                } else {
                    (extract_frustum_planes(&view_proj), view_proj)
                };

            let (screen_width, screen_height) = configs.view.viewport_size.unwrap_or((1, 1));
            let projection_scale_y = camera_matrices.projection[(1, 1)];
            let min_screen_pixel_size = configs.scene.min_screen_pixel_size;

            let object_count = self.world_states[self.current_world_id as usize]
                .as_ref()
                .unwrap()
                .object_count;

            let culling_uniforms = CullingUniforms {
                frustum_planes: culling_frustum_planes.map(|v| [v.x, v.y, v.z, v.w]),
                view_projection: culling_view_proj.into(),
                occluder_view_projection: view_proj.into(),
                screen_size: [screen_width as f32, screen_height as f32],
                object_count,
                min_screen_pixel_size,
                projection_scale_y,
                camera_culling_mask: configs.scene.active_view.culling_mask,
                hiz_mip_count: self.hiz.mip_count() as f32,
                occlusion_enabled: u32::from(
                    self.hiz.hiz_view().is_some()
                        && configs.scene.occlusion_culling_enabled
                        && configs.scene.gpu_culling_enabled,
                ),
                frustum_enabled: u32::from(configs.scene.gpu_culling_enabled),
                _pad_culling: [0; 3],
            };
            queue.write_buffer(
                &self.culling_uniform_buffer,
                0,
                bytemuck::cast_slice(&[culling_uniforms]),
            );

            let world_state = self.world_states[self.current_world_id as usize]
                .as_mut()
                .unwrap();
            let gpu = world_state.gpu_buffers.as_mut().unwrap();
            if gpu.culling_bind_group.is_none() {
                gpu.culling_bind_group =
                    Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Mesh Culling Bind Group"),
                        layout: &self.culling_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: gpu.transform_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: gpu.object_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: self.culling_uniform_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: self.mesh_bounds_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 4,
                                resource: gpu.indirect_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 5,
                                resource: gpu.visible_indices_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 6,
                                resource: self.mesh_lod_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 7,
                                resource: wgpu::BindingResource::TextureView(
                                    self.hiz.hiz_view_or_dummy(),
                                ),
                            },
                        ],
                    }));
            }

            let (z_near, z_far) = configs
                .scene
                .render_view
                .as_ref()
                .map(|view| (view.z_near, view.z_far))
                .unwrap_or((0.1, 1000.0));

            let tile_size_x = (screen_width as f32) / (CLUSTER_GRID_X as f32);
            let tile_size_y = (screen_height as f32) / (CLUSTER_GRID_Y as f32);

            let inverse_projection: [[f32; 4]; 4] =
                nalgebra_glm::inverse(&camera_matrices.projection).into();

            let num_directional_lights = self.world_states[self.current_world_id as usize]
                .as_ref()
                .unwrap()
                .num_directional_lights;
            let cluster_uniforms = ClusterUniforms {
                inverse_projection,
                screen_size: [screen_width as f32, screen_height as f32],
                z_near: cluster_slice_near(z_near),
                z_far,
                cluster_count: [CLUSTER_GRID_X, CLUSTER_GRID_Y, CLUSTER_GRID_Z, 0],
                tile_size: [tile_size_x, tile_size_y],
                num_lights: lights_data.len() as u32,
                num_directional_lights,
            };
            let cluster_uniforms_buffer = &self.world_states[self.current_world_id as usize]
                .as_ref()
                .unwrap()
                .gpu_buffers
                .as_ref()
                .unwrap()
                .cluster_uniforms_buffer;
            queue.write_buffer(
                cluster_uniforms_buffer,
                0,
                bytemuck::cast_slice(&[cluster_uniforms]),
            );

            let view_matrix: [[f32; 4]; 4] = camera_matrices.view.into();
            queue.write_buffer(
                &self.view_matrix_buffer,
                0,
                bytemuck::cast_slice(&[view_matrix]),
            );

            let view_proj_hash = {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                for row in &inverse_projection {
                    for val in row {
                        val.to_bits().hash(&mut hasher);
                    }
                }
                screen_width.hash(&mut hasher);
                screen_height.hash(&mut hasher);
                hasher.finish()
            };

            {
                let state = self.world_states[self.current_world_id as usize]
                    .as_mut()
                    .unwrap();
                state.camera_changed = view_proj_hash != state.last_camera_hash;
                if state.camera_changed {
                    state.last_camera_hash = view_proj_hash;
                }
            }

            let world_state = self.world_states[self.current_world_id as usize]
                .as_mut()
                .unwrap();
            let gpu = world_state.gpu_buffers.as_mut().unwrap();
            if gpu.cluster_assign_bind_group.is_none() {
                gpu.cluster_assign_bind_group =
                    Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Cluster Light Assign Bind Group (Per-World)"),
                        layout: &self.cluster_assign_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: gpu.cluster_uniforms_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: self.cluster_bounds_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: gpu.light_grid_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: gpu.light_indices_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 4,
                                resource: gpu.lights_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 5,
                                resource: self.view_matrix_buffer.as_entire_binding(),
                            },
                        ],
                    }));
            }
        }

        let camera_position = configs
            .scene
            .render_view
            .as_ref()
            .map(|view| view.camera_position)
            .unwrap_or_else(|| nalgebra_glm::vec3(0.0, 0.0, 0.0));

        let spotlight_result = collect_spotlight_shadows(configs);
        apply_spotlight_shadow_indices(
            &mut lights_data,
            &spotlight_result.entity_to_shadow_index,
            &entity_to_lights_index,
        );

        if !spotlight_result.shadow_data.is_empty() {
            queue.write_buffer(
                &self.spotlight_shadow_buffer,
                0,
                bytemuck::cast_slice(&spotlight_result.shadow_data),
            );
        }

        let point_shadow_result = collect_point_light_shadows(
            configs,
            camera_position,
            &mut lights_data,
            &entity_to_lights_index,
        );
        if !point_shadow_result.is_empty() {
            queue.write_buffer(
                &self.point_shadow_buffer,
                0,
                bytemuck::cast_slice(&point_shadow_result),
            );
        }

        if !lights_data.is_empty() {
            let lights_buffer_size = self.world_states[self.current_world_id as usize]
                .as_ref()
                .unwrap()
                .gpu_buffers
                .as_ref()
                .unwrap()
                .lights_buffer_size;
            if lights_data.len() > lights_buffer_size {
                let new_size = (lights_data.len() as f32 * BUFFER_GROWTH_FACTOR).ceil() as usize;
                let new_size = new_size.min(MAX_LIGHTS);

                let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Mesh Lights Buffer (Resized)"),
                    size: (std::mem::size_of::<LightData>() * new_size) as u64,
                    usage: wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                });

                {
                    let world_state = self.world_states[self.current_world_id as usize]
                        .as_mut()
                        .unwrap();
                    let gpu = world_state.gpu_buffers.as_mut().unwrap();
                    gpu.lights_buffer = new_buffer;
                    gpu.lights_buffer_size = new_size;
                }
                self.rebuild_instance_bind_group(device);
            }

            let lights_buffer = &self.world_states[self.current_world_id as usize]
                .as_ref()
                .unwrap()
                .gpu_buffers
                .as_ref()
                .unwrap()
                .lights_buffer;
            queue.write_buffer(lights_buffer, 0, bytemuck::cast_slice(&lights_data));
        }

        apply_area_light_shadow_indices(
            &mut area_lights_data,
            &spotlight_result.entity_to_shadow_index,
            &area_entity_to_index,
        );
        resolve_area_emissive_layers(
            configs,
            &mut area_lights_data,
            &area_entity_to_index,
            &self.material_layer_map,
            &configs.texture_cache.registry,
        );
        {
            let gpu = self.world_states[self.current_world_id as usize]
                .as_ref()
                .unwrap()
                .gpu_buffers
                .as_ref()
                .unwrap();
            if !area_lights_data.is_empty() {
                queue.write_buffer(
                    &gpu.area_lights_buffer,
                    0,
                    bytemuck::cast_slice(&area_lights_data),
                );
            }
            let area_uniforms = AreaUniforms {
                count: area_lights_data.len() as u32,
                _pad0: 0,
                _pad1: 0,
                _pad2: 0,
            };
            queue.write_buffer(
                &gpu.area_uniforms_buffer,
                0,
                bytemuck::cast_slice(&[area_uniforms]),
            );
        }
    }
}
