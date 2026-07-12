use super::super::pass::MeshPass;
use super::super::types::{
    CLUSTER_GRID_X, CLUSTER_GRID_Y, CLUSTER_GRID_Z, DrawIndexedIndirect, LightGrid,
    NUM_DRAW_CLASSES, TOTAL_CLUSTERS,
};
use crate::wgpu::render_configs::RenderInputs;
use crate::wgpu::rendergraph::PassExecutionContext;

impl MeshPass {
    pub(in super::super) fn execute_pass_node<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, RenderInputs>,
    ) -> crate::wgpu::rendergraph::Result<Vec<crate::wgpu::rendergraph::SubGraphRunCommand<'r>>>
    {
        if !context.is_pass_enabled() {
            return Ok(context.into_sub_graph_commands());
        }
        let (shadow_depth_view, _, _) = context.get_depth_attachment("shadow_depth")?;
        let (spotlight_shadow_atlas_view, _, _) =
            context.get_depth_attachment("spotlight_shadow_atlas")?;

        let (color_view, color_load, color_store) = context.get_color_attachment("color")?;
        let scene_color_texture = context.get_texture("color")?;
        let scene_color_size = (
            scene_color_texture.size().width,
            scene_color_texture.size().height,
        );
        if scene_color_size != self.transmission_size {
            let (texture, full_view, mip_views) = Self::create_transmission_textures(
                context.device,
                scene_color_size.0,
                scene_color_size.1,
            );
            self.transmission_color_texture = texture;
            self.transmission_color_view = full_view;
            self.transmission_mip_views = mip_views;
            self.transmission_size = scene_color_size;
            self.scene_bind_group_dirty = true;
        }
        let (depth_view, depth_load, depth_store) = context.get_depth_attachment("depth")?;
        let (entity_id_view, entity_id_load, entity_id_store) =
            context.get_color_attachment("entity_id")?;
        let (view_normals_view, view_normals_load, view_normals_store) =
            context.get_color_attachment("view_normals")?;
        let (velocity_view, velocity_load, velocity_store) =
            context.get_color_attachment("velocity")?;

        let entity_id_discard = entity_id_store == wgpu::StoreOp::Discard;
        let view_normals_discard = view_normals_store == wgpu::StoreOp::Discard;
        let velocity_discard = velocity_store == wgpu::StoreOp::Discard;
        let mut pending_depth_load = Some(depth_load);
        let mut pending_entity_id_load = Some(entity_id_load);
        let mut pending_view_normals_load = Some(view_normals_load);
        let mut pending_velocity_load = Some(velocity_load);

        let workgroup_size = 256;
        let dispatch_size = self.state().object_count.div_ceil(workgroup_size);

        if self.state().instanced_compute_dirty
            && !self.state().instanced_transform_ranges.is_empty()
        {
            self.dispatch_instanced_transform_compute(
                context.encoder,
                context.queue,
                context.configs,
            );
            self.state_mut().instanced_compute_dirty = false;
        }

        if self.state().camera_changed {
            let mut compute_pass =
                context
                    .encoder
                    .begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("Cluster Bounds Pass"),
                        timestamp_writes: None,
                    });

            compute_pass.set_pipeline(&self.cluster_bounds_pipeline);
            compute_pass.set_bind_group(0, &self.gpu().cluster_bounds_bind_group, &[]);
            let dispatch_x = CLUSTER_GRID_X.div_ceil(8);
            let dispatch_y = CLUSTER_GRID_Y.div_ceil(8);
            compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, CLUSTER_GRID_Z);
        }

        let light_grid_size = (std::mem::size_of::<LightGrid>() * TOTAL_CLUSTERS as usize) as u64;
        context.encoder.copy_buffer_to_buffer(
            &self.gpu().light_grid_reset_buffer,
            0,
            &self.gpu().light_grid_buffer,
            0,
            light_grid_size,
        );

        if let Some(ref cluster_assign_bind_group) = self.gpu().cluster_assign_bind_group {
            let mut compute_pass =
                context
                    .encoder
                    .begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("Cluster Light Assignment Pass"),
                        timestamp_writes: None,
                    });

            compute_pass.set_pipeline(&self.cluster_assign_pipeline);
            compute_pass.set_bind_group(0, cluster_assign_bind_group, &[]);
            let dispatch_x = CLUSTER_GRID_X.div_ceil(8);
            let dispatch_y = CLUSTER_GRID_Y.div_ceil(8);
            compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, CLUSTER_GRID_Z);
        }

        if self.state().object_count > 0 && self.state().indirect_reset_count > 0 {
            if self.gpu_batching_enabled {
                let object_count = self.state().object_count;
                let regular_count = self.state().regular_object_count;
                let mesh_count = self.mesh_data.len() as u32;
                let material_count = self.state().cached_materials_data.len() as u32;
                let cap = self.gpu_batch_cap as u32;
                let batch_params: [u32; 8] = [
                    object_count,
                    regular_count,
                    mesh_count,
                    material_count,
                    cap,
                    0,
                    0,
                    0,
                ];
                context.queue.write_buffer(
                    &self.gpu_batch_params_buffer,
                    0,
                    bytemuck::cast_slice(&batch_params),
                );
                if self.gpu().gpu_batch_bind_group.is_none() {
                    let bind_group = context
                        .device
                        .create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Mesh GPU Batch Bind Group"),
                            layout: &self.gpu_batch_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: self.gpu().object_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: self.gpu().material_flags_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 2,
                                    resource: self.gpu().dense_capacity_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 3,
                                    resource: self.gpu_batch_params_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 4,
                                    resource: self.gpu().batch_descs_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 5,
                                    resource: self.gpu().batch_key_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 6,
                                    resource: self.gpu().batch_meta_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 7,
                                    resource: self.mesh_lod_geo_buffer.as_entire_binding(),
                                },
                            ],
                        });
                    self.gpu_mut().gpu_batch_bind_group = Some(bind_group);
                }
                let object_dispatch = object_count.max(1).div_ceil(64);
                let rebuild = self.gpu_batch_dirty;
                if rebuild {
                    let dense_total = 18 * mesh_count.max(1) * material_count.max(1);
                    let mut clear_pass =
                        context
                            .encoder
                            .begin_compute_pass(&wgpu::ComputePassDescriptor {
                                label: Some("Mesh GPU Batch Clear Pass"),
                                timestamp_writes: None,
                            });
                    clear_pass.set_pipeline(&self.gpu_batch_clear_pipeline);
                    clear_pass.set_bind_group(
                        0,
                        self.gpu().gpu_batch_bind_group.as_ref().unwrap(),
                        &[],
                    );
                    clear_pass.dispatch_workgroups(dense_total.div_ceil(64), 1, 1);
                }
                {
                    let mut classify_pass =
                        context
                            .encoder
                            .begin_compute_pass(&wgpu::ComputePassDescriptor {
                                label: Some("Mesh GPU Classify Pass"),
                                timestamp_writes: None,
                            });
                    classify_pass.set_pipeline(&self.gpu_batch_classify_pipeline);
                    classify_pass.set_bind_group(
                        0,
                        self.gpu().gpu_batch_bind_group.as_ref().unwrap(),
                        &[],
                    );
                    classify_pass.dispatch_workgroups(object_dispatch, 1, 1);
                }
                if rebuild {
                    {
                        let mut count_pass =
                            context
                                .encoder
                                .begin_compute_pass(&wgpu::ComputePassDescriptor {
                                    label: Some("Mesh GPU Batch Count Pass"),
                                    timestamp_writes: None,
                                });
                        count_pass.set_pipeline(&self.gpu_batch_count_pipeline);
                        count_pass.set_bind_group(
                            0,
                            self.gpu().gpu_batch_bind_group.as_ref().unwrap(),
                            &[],
                        );
                        count_pass.dispatch_workgroups(object_dispatch, 1, 1);
                    }
                    {
                        let mut build_pass =
                            context
                                .encoder
                                .begin_compute_pass(&wgpu::ComputePassDescriptor {
                                    label: Some("Mesh GPU Batch Build Pass"),
                                    timestamp_writes: None,
                                });
                        build_pass.set_pipeline(&self.gpu_batch_build_pipeline);
                        build_pass.set_bind_group(
                            0,
                            self.gpu().gpu_batch_bind_group.as_ref().unwrap(),
                            &[],
                        );
                        build_pass.dispatch_workgroups(
                            super::super::types::NUM_DRAW_CLASSES as u32,
                            1,
                            1,
                        );
                    }
                    self.gpu_batch_dirty = false;
                }
            }

            let mesh_geo: Vec<super::super::types::MeshGeo> = self
                .mesh_data
                .iter()
                .map(|mesh| super::super::types::MeshGeo {
                    index_count: mesh.index_count,
                    first_index: mesh.index_offset,
                    base_vertex: mesh.vertex_offset as i32,
                    _pad: 0,
                })
                .collect();
            if mesh_geo.len() > self.mesh_geo_buffer_size {
                let new_size = mesh_geo.len().next_power_of_two();
                self.mesh_geo_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Mesh Geometry Table Buffer (Resized)"),
                    size: (std::mem::size_of::<super::super::types::MeshGeo>() * new_size) as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                self.mesh_geo_buffer_size = new_size;
                self.gpu_mut().indirect_build_bind_group = None;
            }
            if !mesh_geo.is_empty() {
                context.queue.write_buffer(
                    &self.mesh_geo_buffer,
                    0,
                    bytemuck::cast_slice(&mesh_geo),
                );
            }

            let build_params: [u32; 4] = [self.state().indirect_reset_count as u32, 0, 0, 0];
            context.queue.write_buffer(
                &self.indirect_build_params_buffer,
                0,
                bytemuck::cast_slice(&build_params),
            );

            if self.gpu().indirect_build_bind_group.is_none() {
                let bind_group = context
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Mesh Indirect Build Bind Group"),
                        layout: &self.indirect_build_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: self.gpu().indirect_reset_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: self.gpu().batch_descs_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: self.mesh_geo_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: self.indirect_build_params_buffer.as_entire_binding(),
                            },
                        ],
                    });
                self.gpu_mut().indirect_build_bind_group = Some(bind_group);
            }

            {
                let mut build_pass =
                    context
                        .encoder
                        .begin_compute_pass(&wgpu::ComputePassDescriptor {
                            label: Some("Mesh Indirect Build Pass"),
                            timestamp_writes: None,
                        });
                build_pass.set_pipeline(&self.indirect_build_pipeline);
                build_pass.set_bind_group(
                    0,
                    self.gpu().indirect_build_bind_group.as_ref().unwrap(),
                    &[],
                );
                build_pass.dispatch_workgroups(1, 1, 1);
            }

            if self.gpu().batch_assign_bind_group.is_none() {
                let bind_group = context
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Mesh Batch Assign Bind Group"),
                        layout: &self.batch_assign_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: self.gpu().object_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: self.gpu().batch_key_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: self.batch_assign_params_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: self.gpu().batch_meta_buffer.as_entire_binding(),
                            },
                        ],
                    });
                self.gpu_mut().batch_assign_bind_group = Some(bind_group);
            }

            {
                let object_count = self.state().object_count;
                let key_count = self.gpu().batch_key_count;
                let mode = u32::from(self.gpu_batching_enabled);
                let cap = self.gpu_batch_cap as u32;
                let assign_params: [u32; 8] =
                    [object_count, key_count, u32::MAX, mode, cap, 0, 0, 0];
                context.queue.write_buffer(
                    &self.batch_assign_params_buffer,
                    0,
                    bytemuck::cast_slice(&assign_params),
                );
                let assign_dispatch = object_count.div_ceil(64);
                let mut assign_pass =
                    context
                        .encoder
                        .begin_compute_pass(&wgpu::ComputePassDescriptor {
                            label: Some("Mesh Batch Assign Pass"),
                            timestamp_writes: None,
                        });
                assign_pass.set_pipeline(&self.batch_assign_pipeline);
                assign_pass.set_bind_group(
                    0,
                    self.gpu().batch_assign_bind_group.as_ref().unwrap(),
                    &[],
                );
                assign_pass.dispatch_workgroups(assign_dispatch, 1, 1);
            }
        }

        let occlusion_culling = context.configs.scene.occlusion_culling_enabled
            && context.configs.scene.gpu_culling_enabled;

        if self.state().object_count > 0 {
            if self.gpu().culling_bind_group.is_none()
                || self.gpu().culling_bind_group_frustum.is_none()
            {
                self.build_culling_bind_groups(context.device);
            }

            if self.state().indirect_reset_count > 0 {
                let copy_size = (self.state().indirect_reset_count
                    * std::mem::size_of::<DrawIndexedIndirect>())
                    as u64;
                context.encoder.copy_buffer_to_buffer(
                    &self.gpu().indirect_reset_buffer,
                    0,
                    &self.gpu().indirect_buffer,
                    0,
                    copy_size,
                );
            }

            {
                let mut compute_pass =
                    context
                        .encoder
                        .begin_compute_pass(&wgpu::ComputePassDescriptor {
                            label: Some("Mesh Frustum Cull Pass"),
                            timestamp_writes: None,
                        });

                compute_pass.set_pipeline(&self.culling_pipeline);
                compute_pass.set_bind_group(
                    0,
                    self.gpu().culling_bind_group_frustum.as_ref().unwrap(),
                    &[],
                );
                compute_pass.dispatch_workgroups(dispatch_size, 1, 1);
            }

            if occlusion_culling
                && (!self.state().opaque_instances.is_empty()
                    || !self.state().opaque_double_sided_instances.is_empty()
                    || !self.state().transparent_instances.is_empty()
                    || !self.state().instanced_opaque_batches.is_empty()
                    || !self
                        .state()
                        .instanced_opaque_double_sided_batches
                        .is_empty())
            {
                let mut depth_prepass =
                    context
                        .encoder
                        .begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Depth Prepass"),
                            color_attachments: &[],
                            depth_stencil_attachment: Some(
                                wgpu::RenderPassDepthStencilAttachment {
                                    view: depth_view,
                                    depth_ops: Some(wgpu::Operations {
                                        load: pending_depth_load
                                            .take()
                                            .unwrap_or(wgpu::LoadOp::Load),
                                        store: wgpu::StoreOp::Store,
                                    }),
                                    stencil_ops: None,
                                },
                            ),
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        });

                depth_prepass.set_pipeline(&self.depth_prepass_pipeline);
                depth_prepass.set_bind_group(0, &self.uniform_bind_group, &[]);
                depth_prepass.set_bind_group(1, &self.gpu().instance_bind_group, &[]);
                depth_prepass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                depth_prepass
                    .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                let indirect_buffer = &self.gpu().indirect_buffer;
                let stride = std::mem::size_of::<DrawIndexedIndirect>() as u64;
                let supports_multi_draw = self.supports_multi_draw_count;

                let prepass_counts = self.state().prepass_batch_counts;
                let opaque_batch_count = self.state().opaque_instances.len();
                let prepass_instanced_offset = self.state().opaque_instances.len()
                    + self.state().opaque_double_sided_instances.len()
                    + self.state().transparent_instances.len()
                    + self.state().overlay_opaque_instances.len()
                    + self.state().overlay_opaque_double_sided_instances.len()
                    + self.state().overlay_transparent_instances.len();
                let instanced_ds_offset =
                    prepass_instanced_offset + self.state().instanced_opaque_batches.len();

                let gpu_batching = self.gpu_batching_enabled;
                let cap = self.gpu_batch_cap as u32;
                let prepass_meta_base = 2 * super::super::types::NUM_DRAW_CLASSES;
                let batch_meta_buffer = &self.gpu().batch_meta_buffer;

                let mut draw_prepass_group = |base_batch: usize, class_index: usize, count: u32| {
                    if gpu_batching {
                        depth_prepass.multi_draw_indexed_indirect_count(
                            indirect_buffer,
                            (base_batch as u64) * stride,
                            batch_meta_buffer,
                            ((prepass_meta_base + class_index) * std::mem::size_of::<u32>()) as u64,
                            cap,
                        );
                        return;
                    }
                    if count == 0 {
                        return;
                    }
                    if supports_multi_draw {
                        depth_prepass.multi_draw_indexed_indirect(
                            indirect_buffer,
                            (base_batch as u64) * stride,
                            count,
                        );
                        return;
                    }
                    for batch_index in 0..count as usize {
                        depth_prepass.draw_indexed_indirect(
                            indirect_buffer,
                            ((base_batch + batch_index) as u64) * stride,
                        );
                    }
                };

                draw_prepass_group(0, 0, prepass_counts[0]);
                draw_prepass_group(opaque_batch_count, 1, prepass_counts[1]);
                draw_prepass_group(prepass_instanced_offset, 6, prepass_counts[6]);
                draw_prepass_group(instanced_ds_offset, 7, prepass_counts[7]);
            }

            if occlusion_culling {
                {
                    let depth_for_hiz = context.get_texture_view("depth")?;
                    self.hiz
                        .resize(context.device, scene_color_size.0, scene_color_size.1);
                    self.hiz.invalidate_bind_groups();
                    self.hiz.rebuild_bind_groups(context.device, depth_for_hiz);
                    self.hiz.execute(context.encoder);
                }
                self.build_culling_bind_groups(context.device);

                if self.state().indirect_reset_count > 0 {
                    let copy_size = (self.state().indirect_reset_count
                        * std::mem::size_of::<DrawIndexedIndirect>())
                        as u64;
                    context.encoder.copy_buffer_to_buffer(
                        &self.gpu().indirect_reset_buffer,
                        0,
                        &self.gpu().indirect_buffer,
                        0,
                        copy_size,
                    );
                }

                let mut compute_pass =
                    context
                        .encoder
                        .begin_compute_pass(&wgpu::ComputePassDescriptor {
                            label: Some("Mesh Occlusion Cull Pass"),
                            timestamp_writes: None,
                        });

                compute_pass.set_pipeline(&self.culling_pipeline);
                compute_pass.set_bind_group(
                    0,
                    self.gpu().culling_bind_group.as_ref().unwrap(),
                    &[],
                );
                compute_pass.dispatch_workgroups(dispatch_size, 1, 1);
            }
        }

        if self.supports_multi_draw_count && self.state().object_count > 0 {
            let class_ranges: [[u32; 2]; NUM_DRAW_CLASSES] = {
                let state = self.state();
                let opaque = state.opaque_instances.len() as u32;
                let opaque_double_sided = state.opaque_double_sided_instances.len() as u32;
                let transparent = state.transparent_instances.len() as u32;
                let overlay_opaque = state.overlay_opaque_instances.len() as u32;
                let overlay_double_sided = state.overlay_opaque_double_sided_instances.len() as u32;
                let overlay_transparent = state.overlay_transparent_instances.len() as u32;
                let instanced_opaque = state.instanced_opaque_batches.len() as u32;
                let instanced_double_sided =
                    state.instanced_opaque_double_sided_batches.len() as u32;
                let instanced_transparent = state.instanced_transparent_batches.len() as u32;
                let instanced_offset = opaque
                    + opaque_double_sided
                    + transparent
                    + overlay_opaque
                    + overlay_double_sided
                    + overlay_transparent;
                [
                    [0, opaque],
                    [opaque, opaque_double_sided],
                    [opaque + opaque_double_sided, transparent],
                    [opaque + opaque_double_sided + transparent, overlay_opaque],
                    [
                        opaque + opaque_double_sided + transparent + overlay_opaque,
                        overlay_double_sided,
                    ],
                    [
                        opaque
                            + opaque_double_sided
                            + transparent
                            + overlay_opaque
                            + overlay_double_sided,
                        overlay_transparent,
                    ],
                    [instanced_offset, instanced_opaque],
                    [instanced_offset + instanced_opaque, instanced_double_sided],
                    [
                        instanced_offset + instanced_opaque + instanced_double_sided,
                        instanced_transparent,
                    ],
                ]
            };
            context.queue.write_buffer(
                &self.gpu().class_ranges_buffer,
                0,
                bytemuck::cast_slice(&class_ranges),
            );

            if self.gpu().compaction_bind_group.is_none() {
                let bind_group = context
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Mesh Indirect Compaction Bind Group"),
                        layout: &self.compaction_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: self.gpu().indirect_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: self.gpu().draw_count_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: self.gpu().class_ranges_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: self.compaction_params_buffer.as_entire_binding(),
                            },
                        ],
                    });
                self.gpu_mut().compaction_bind_group = Some(bind_group);
            }

            let mut compaction_pass =
                context
                    .encoder
                    .begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("Mesh Indirect Compaction Pass"),
                        timestamp_writes: None,
                    });
            compaction_pass.set_pipeline(&self.compaction_pipeline);
            compaction_pass.set_bind_group(
                0,
                self.gpu().compaction_bind_group.as_ref().unwrap(),
                &[],
            );
            compaction_pass.dispatch_workgroups(NUM_DRAW_CLASSES as u32, 1, 1);
        }

        if self.gpu_batching_enabled && self.state().object_count == 0 {
            let zero_draw_counts = [0u32; NUM_DRAW_CLASSES];
            context.queue.write_buffer(
                &self.gpu().draw_count_buffer,
                0,
                bytemuck::cast_slice(&zero_draw_counts),
            );
            let zero_meta = [0u32; NUM_DRAW_CLASSES * 3];
            context.queue.write_buffer(
                &self.gpu().batch_meta_buffer,
                0,
                bytemuck::cast_slice(&zero_meta),
            );
        }

        let world_state = self.state();
        let brdf_lut_view = world_state
            .ibl_brdf_lut_view
            .as_ref()
            .unwrap_or(&self.brdf_lut_view);
        let irradiance_view = world_state
            .ibl_irradiance_view
            .as_ref()
            .unwrap_or(&self.irradiance_view);
        let prefiltered_view = world_state
            .ibl_prefiltered_view
            .as_ref()
            .unwrap_or(&self.prefiltered_view);
        let irradiance_b_view = world_state
            .ibl_irradiance_b_view
            .as_ref()
            .unwrap_or(&self.irradiance_b_view);
        let prefiltered_b_view = world_state
            .ibl_prefiltered_b_view
            .as_ref()
            .unwrap_or(&self.prefiltered_b_view);

        if self.scene_bind_group_dirty {
            self.scene_bind_group = context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Scene Bind Group (Per-World IBL)"),
                    layout: &self.scene_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(shadow_depth_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.shadow_sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(
                                spotlight_shadow_atlas_view,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Sampler(&self.shadow_sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: self.spotlight_shadow_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: wgpu::BindingResource::TextureView(brdf_lut_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: wgpu::BindingResource::Sampler(&self.ibl_sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 7,
                            resource: wgpu::BindingResource::TextureView(irradiance_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 8,
                            resource: wgpu::BindingResource::Sampler(&self.ibl_sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 9,
                            resource: wgpu::BindingResource::TextureView(prefiltered_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 10,
                            resource: wgpu::BindingResource::Sampler(&self.ibl_sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 11,
                            resource: wgpu::BindingResource::TextureView(
                                &self.point_shadow_cubemap_view,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 12,
                            resource: wgpu::BindingResource::Sampler(&self.point_shadow_sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 13,
                            resource: self.point_shadow_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 14,
                            resource: wgpu::BindingResource::TextureView(irradiance_b_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 15,
                            resource: wgpu::BindingResource::TextureView(prefiltered_b_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 16,
                            resource: wgpu::BindingResource::TextureView(
                                &self.transmission_color_view,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 17,
                            resource: wgpu::BindingResource::Sampler(&self.transmission_sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 18,
                            resource: wgpu::BindingResource::Sampler(&self.shadow_pcf_sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 19,
                            resource: wgpu::BindingResource::TextureView(&self.ltc.view_matrix),
                        },
                        wgpu::BindGroupEntry {
                            binding: 20,
                            resource: wgpu::BindingResource::TextureView(&self.ltc.view_terms),
                        },
                        wgpu::BindGroupEntry {
                            binding: 21,
                            resource: wgpu::BindingResource::Sampler(&self.ltc.sampler),
                        },
                    ],
                });
            self.scene_bind_group_dirty = false;
        }

        let instanced_batch_offset = self.state().opaque_instances.len()
            + self.state().opaque_double_sided_instances.len()
            + self.state().transparent_instances.len()
            + self.state().overlay_opaque_instances.len()
            + self.state().overlay_opaque_double_sided_instances.len()
            + self.state().overlay_transparent_instances.len();

        let draw_indirect_buffer = &self.gpu().indirect_buffer;
        let draw_count_buffer = &self.gpu().draw_count_buffer;
        let supports_count = self.supports_multi_draw_count;
        let draw_instance_bind_group = &self.gpu().instance_bind_group;

        let has_opaque = !self.state().opaque_instances.is_empty()
            || !self.state().instanced_opaque_batches.is_empty();
        let has_opaque_double_sided = !self.state().opaque_double_sided_instances.is_empty()
            || !self
                .state()
                .instanced_opaque_double_sided_batches
                .is_empty();

        if has_opaque || has_opaque_double_sided {
            let mut opaque_pass = context
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Opaque Mesh Pass"),
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view: color_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: color_load,
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        }),
                        Some(wgpu::RenderPassColorAttachment {
                            view: entity_id_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: take_aux_load(&mut pending_entity_id_load, entity_id_discard),
                                store: entity_id_store,
                            },
                            depth_slice: None,
                        }),
                        Some(wgpu::RenderPassColorAttachment {
                            view: view_normals_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: take_aux_load(
                                    &mut pending_view_normals_load,
                                    view_normals_discard,
                                ),
                                store: view_normals_store,
                            },
                            depth_slice: None,
                        }),
                        Some(wgpu::RenderPassColorAttachment {
                            view: velocity_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: take_aux_load(&mut pending_velocity_load, velocity_discard),
                                store: velocity_store,
                            },
                            depth_slice: None,
                        }),
                    ],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: pending_depth_load.take().unwrap_or(wgpu::LoadOp::Load),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });

            opaque_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            opaque_pass.set_bind_group(1, draw_instance_bind_group, &[]);
            opaque_pass.set_bind_group(2, self.material_bind_group.as_ref().unwrap(), &[]);
            opaque_pass.set_bind_group(3, &self.scene_bind_group, &[]);
            opaque_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            opaque_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            if has_opaque {
                opaque_pass.set_pipeline(&self.opaque_pipeline);

                MeshPass::draw_batches(
                    &mut opaque_pass,
                    self.state().opaque_instances.len(),
                    0,
                    0,
                    supports_count,
                    draw_indirect_buffer,
                    draw_count_buffer,
                );

                if !self.state().instanced_opaque_batches.is_empty() {
                    MeshPass::draw_batches(
                        &mut opaque_pass,
                        self.state().instanced_opaque_batches.len(),
                        instanced_batch_offset,
                        6,
                        supports_count,
                        draw_indirect_buffer,
                        draw_count_buffer,
                    );
                }
            }

            if has_opaque_double_sided {
                opaque_pass.set_pipeline(&self.opaque_double_sided_pipeline);

                MeshPass::draw_batches(
                    &mut opaque_pass,
                    self.state().opaque_double_sided_instances.len(),
                    self.state().opaque_instances.len(),
                    1,
                    supports_count,
                    draw_indirect_buffer,
                    draw_count_buffer,
                );

                if !self
                    .state()
                    .instanced_opaque_double_sided_batches
                    .is_empty()
                {
                    MeshPass::draw_batches(
                        &mut opaque_pass,
                        self.state().instanced_opaque_double_sided_batches.len(),
                        instanced_batch_offset + self.state().instanced_opaque_batches.len(),
                        7,
                        supports_count,
                        draw_indirect_buffer,
                        draw_count_buffer,
                    );
                }
            }

            drop(opaque_pass);
        }

        let has_transmission_materials = context
            .configs
            .scene
            .render_materials
            .entries
            .iter()
            .any(|entry| entry.material.transmission_factor > 0.0);
        let needs_transmission_copy = has_transmission_materials
            && (!self.state().transparent_instances.is_empty()
                || !self.state().instanced_transparent_batches.is_empty());
        if needs_transmission_copy {
            context.encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: scene_color_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyTextureInfo {
                    texture: &self.transmission_color_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: scene_color_size.0,
                    height: scene_color_size.1,
                    depth_or_array_layers: 1,
                },
            );

            let mip_count = self.transmission_mip_views.len();
            if mip_count > 1 {
                let mut downsample_bind_groups = Vec::with_capacity(mip_count - 1);
                for level in 1..mip_count {
                    downsample_bind_groups.push(context.device.create_bind_group(
                        &wgpu::BindGroupDescriptor {
                            label: Some("Mesh Transmission Downsample Bind Group"),
                            layout: &self.transmission_downsample_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(
                                        &self.transmission_mip_views[level - 1],
                                    ),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(
                                        &self.transmission_downsample_sampler,
                                    ),
                                },
                            ],
                        },
                    ));
                }

                for level in 1..mip_count {
                    let mut downsample_pass =
                        context
                            .encoder
                            .begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("Mesh Transmission Downsample Pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &self.transmission_mip_views[level],
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                        store: wgpu::StoreOp::Store,
                                    },
                                    depth_slice: None,
                                })],
                                depth_stencil_attachment: None,
                                timestamp_writes: None,
                                occlusion_query_set: None,
                                multiview_mask: None,
                            });
                    downsample_pass.set_pipeline(&self.transmission_downsample_pipeline);
                    downsample_pass.set_bind_group(0, &downsample_bind_groups[level - 1], &[]);
                    downsample_pass.draw(0..3, 0..1);
                }
            }
        }

        if !self.state().transparent_instances.is_empty()
            || !self.state().instanced_transparent_batches.is_empty()
        {
            let mut blend_opaque_prepass =
                context
                    .encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Blend Opaque Depth Prepass"),
                        color_attachments: &[],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: depth_view,
                            depth_ops: Some(wgpu::Operations {
                                load: pending_depth_load.take().unwrap_or(wgpu::LoadOp::Load),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });

            blend_opaque_prepass.set_pipeline(&self.blend_opaque_depth_prepass_pipeline);
            blend_opaque_prepass.set_bind_group(0, &self.uniform_bind_group, &[]);
            blend_opaque_prepass.set_bind_group(1, draw_instance_bind_group, &[]);
            blend_opaque_prepass.set_bind_group(2, self.material_bind_group.as_ref().unwrap(), &[]);
            blend_opaque_prepass.set_bind_group(3, &self.scene_bind_group, &[]);
            blend_opaque_prepass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            blend_opaque_prepass
                .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            MeshPass::draw_batches(
                &mut blend_opaque_prepass,
                self.state().transparent_instances.len(),
                self.state().opaque_instances.len()
                    + self.state().opaque_double_sided_instances.len(),
                2,
                supports_count,
                draw_indirect_buffer,
                draw_count_buffer,
            );

            if !self.state().instanced_transparent_batches.is_empty() {
                MeshPass::draw_batches(
                    &mut blend_opaque_prepass,
                    self.state().instanced_transparent_batches.len(),
                    instanced_batch_offset
                        + self.state().instanced_opaque_batches.len()
                        + self.state().instanced_opaque_double_sided_batches.len(),
                    8,
                    supports_count,
                    draw_indirect_buffer,
                    draw_count_buffer,
                );
            }

            drop(blend_opaque_prepass);

            let mut oit_pass = context
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("OIT Mesh Pass"),
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view: &self.oit_accum_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        }),
                        Some(wgpu::RenderPassColorAttachment {
                            view: &self.oit_reveal_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        }),
                        Some(wgpu::RenderPassColorAttachment {
                            view: entity_id_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: take_aux_load(&mut pending_entity_id_load, entity_id_discard),
                                store: entity_id_store,
                            },
                            depth_slice: None,
                        }),
                        Some(wgpu::RenderPassColorAttachment {
                            view: velocity_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: take_aux_load(&mut pending_velocity_load, velocity_discard),
                                store: velocity_store,
                            },
                            depth_slice: None,
                        }),
                    ],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: pending_depth_load.take().unwrap_or(wgpu::LoadOp::Load),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });

            oit_pass.set_pipeline(&self.oit_pipeline);
            oit_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            oit_pass.set_bind_group(1, draw_instance_bind_group, &[]);
            oit_pass.set_bind_group(2, self.material_bind_group.as_ref().unwrap(), &[]);
            oit_pass.set_bind_group(3, &self.scene_bind_group, &[]);
            oit_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            oit_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            MeshPass::draw_batches(
                &mut oit_pass,
                self.state().transparent_instances.len(),
                self.state().opaque_instances.len()
                    + self.state().opaque_double_sided_instances.len(),
                2,
                supports_count,
                draw_indirect_buffer,
                draw_count_buffer,
            );

            if !self.state().instanced_transparent_batches.is_empty() {
                MeshPass::draw_batches(
                    &mut oit_pass,
                    self.state().instanced_transparent_batches.len(),
                    instanced_batch_offset
                        + self.state().instanced_opaque_batches.len()
                        + self.state().instanced_opaque_double_sided_batches.len(),
                    8,
                    supports_count,
                    draw_indirect_buffer,
                    draw_count_buffer,
                );
            }

            drop(oit_pass);

            let mut composite_pass =
                context
                    .encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("OIT Composite Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: color_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: color_store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });

            composite_pass.set_pipeline(&self.oit_composite_pipeline);
            composite_pass.set_bind_group(0, &self.oit_composite_bind_group, &[]);
            composite_pass.draw(0..3, 0..1);

            drop(composite_pass);
        }

        if !self.state().overlay_opaque_instances.is_empty() {
            let mut overlay_opaque_pass =
                context
                    .encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Overlay Opaque Mesh Pass"),
                        color_attachments: &[
                            Some(wgpu::RenderPassColorAttachment {
                                view: color_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: wgpu::StoreOp::Store,
                                },
                                depth_slice: None,
                            }),
                            Some(wgpu::RenderPassColorAttachment {
                                view: entity_id_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: take_aux_load(
                                        &mut pending_entity_id_load,
                                        entity_id_discard,
                                    ),
                                    store: entity_id_store,
                                },
                                depth_slice: None,
                            }),
                            Some(wgpu::RenderPassColorAttachment {
                                view: view_normals_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: take_aux_load(
                                        &mut pending_view_normals_load,
                                        view_normals_discard,
                                    ),
                                    store: view_normals_store,
                                },
                                depth_slice: None,
                            }),
                            Some(wgpu::RenderPassColorAttachment {
                                view: velocity_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: take_aux_load(
                                        &mut pending_velocity_load,
                                        velocity_discard,
                                    ),
                                    store: velocity_store,
                                },
                                depth_slice: None,
                            }),
                        ],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &self.overlay_depth_view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(0.0),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });

            overlay_opaque_pass.set_pipeline(&self.overlay_opaque_pipeline);
            overlay_opaque_pass.set_bind_group(0, &self.overlay_uniform_bind_group, &[]);
            overlay_opaque_pass.set_bind_group(1, draw_instance_bind_group, &[]);
            overlay_opaque_pass.set_bind_group(2, self.material_bind_group.as_ref().unwrap(), &[]);
            overlay_opaque_pass.set_bind_group(3, &self.scene_bind_group, &[]);
            overlay_opaque_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            overlay_opaque_pass
                .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            MeshPass::draw_batches(
                &mut overlay_opaque_pass,
                self.state().overlay_opaque_instances.len(),
                self.state().opaque_instances.len()
                    + self.state().opaque_double_sided_instances.len()
                    + self.state().transparent_instances.len(),
                3,
                supports_count,
                draw_indirect_buffer,
                draw_count_buffer,
            );

            drop(overlay_opaque_pass);
        }

        if !self
            .state()
            .overlay_opaque_double_sided_instances
            .is_empty()
        {
            let mut overlay_opaque_double_sided_pass =
                context
                    .encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Overlay Opaque Double-Sided Mesh Pass"),
                        color_attachments: &[
                            Some(wgpu::RenderPassColorAttachment {
                                view: color_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: wgpu::StoreOp::Store,
                                },
                                depth_slice: None,
                            }),
                            Some(wgpu::RenderPassColorAttachment {
                                view: entity_id_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: take_aux_load(
                                        &mut pending_entity_id_load,
                                        entity_id_discard,
                                    ),
                                    store: entity_id_store,
                                },
                                depth_slice: None,
                            }),
                            Some(wgpu::RenderPassColorAttachment {
                                view: view_normals_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: take_aux_load(
                                        &mut pending_view_normals_load,
                                        view_normals_discard,
                                    ),
                                    store: view_normals_store,
                                },
                                depth_slice: None,
                            }),
                            Some(wgpu::RenderPassColorAttachment {
                                view: velocity_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: take_aux_load(
                                        &mut pending_velocity_load,
                                        velocity_discard,
                                    ),
                                    store: velocity_store,
                                },
                                depth_slice: None,
                            }),
                        ],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &self.overlay_depth_view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });

            overlay_opaque_double_sided_pass
                .set_pipeline(&self.overlay_opaque_double_sided_pipeline);
            overlay_opaque_double_sided_pass.set_bind_group(
                0,
                &self.overlay_uniform_bind_group,
                &[],
            );
            overlay_opaque_double_sided_pass.set_bind_group(1, draw_instance_bind_group, &[]);
            overlay_opaque_double_sided_pass.set_bind_group(
                2,
                self.material_bind_group.as_ref().unwrap(),
                &[],
            );
            overlay_opaque_double_sided_pass.set_bind_group(3, &self.scene_bind_group, &[]);
            overlay_opaque_double_sided_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            overlay_opaque_double_sided_pass
                .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            MeshPass::draw_batches(
                &mut overlay_opaque_double_sided_pass,
                self.state().overlay_opaque_double_sided_instances.len(),
                self.state().opaque_instances.len()
                    + self.state().opaque_double_sided_instances.len()
                    + self.state().transparent_instances.len()
                    + self.state().overlay_opaque_instances.len(),
                4,
                supports_count,
                draw_indirect_buffer,
                draw_count_buffer,
            );

            drop(overlay_opaque_double_sided_pass);
        }

        if !self.state().overlay_transparent_instances.is_empty() {
            let mut overlay_oit_pass =
                context
                    .encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Overlay OIT Mesh Pass"),
                        color_attachments: &[
                            Some(wgpu::RenderPassColorAttachment {
                                view: &self.oit_accum_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                    store: wgpu::StoreOp::Store,
                                },
                                depth_slice: None,
                            }),
                            Some(wgpu::RenderPassColorAttachment {
                                view: &self.oit_reveal_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                                    store: wgpu::StoreOp::Store,
                                },
                                depth_slice: None,
                            }),
                            Some(wgpu::RenderPassColorAttachment {
                                view: entity_id_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: take_aux_load(
                                        &mut pending_entity_id_load,
                                        entity_id_discard,
                                    ),
                                    store: entity_id_store,
                                },
                                depth_slice: None,
                            }),
                            Some(wgpu::RenderPassColorAttachment {
                                view: velocity_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: take_aux_load(
                                        &mut pending_velocity_load,
                                        velocity_discard,
                                    ),
                                    store: velocity_store,
                                },
                                depth_slice: None,
                            }),
                        ],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &self.overlay_depth_view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });

            overlay_oit_pass.set_pipeline(&self.oit_pipeline);
            overlay_oit_pass.set_bind_group(0, &self.overlay_uniform_bind_group, &[]);
            overlay_oit_pass.set_bind_group(1, draw_instance_bind_group, &[]);
            overlay_oit_pass.set_bind_group(2, self.material_bind_group.as_ref().unwrap(), &[]);
            overlay_oit_pass.set_bind_group(3, &self.scene_bind_group, &[]);
            overlay_oit_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            overlay_oit_pass
                .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            MeshPass::draw_batches(
                &mut overlay_oit_pass,
                self.state().overlay_transparent_instances.len(),
                self.state().opaque_instances.len()
                    + self.state().opaque_double_sided_instances.len()
                    + self.state().transparent_instances.len()
                    + self.state().overlay_opaque_instances.len()
                    + self.state().overlay_opaque_double_sided_instances.len(),
                5,
                supports_count,
                draw_indirect_buffer,
                draw_count_buffer,
            );

            drop(overlay_oit_pass);

            let mut overlay_composite_pass =
                context
                    .encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Overlay OIT Composite Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: color_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: color_store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });

            overlay_composite_pass.set_pipeline(&self.oit_composite_pipeline);
            overlay_composite_pass.set_bind_group(0, &self.oit_composite_bind_group, &[]);
            overlay_composite_pass.draw(0..3, 0..1);

            drop(overlay_composite_pass);
        }

        if pending_depth_load.is_some()
            || pending_entity_id_load.is_some()
            || pending_view_normals_load.is_some()
            || pending_velocity_load.is_some()
        {
            context
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Mesh Attachment Clear Pass"),
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view: entity_id_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: pending_entity_id_load.unwrap_or(wgpu::LoadOp::Load),
                                store: entity_id_store,
                            },
                            depth_slice: None,
                        }),
                        Some(wgpu::RenderPassColorAttachment {
                            view: view_normals_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: pending_view_normals_load.unwrap_or(wgpu::LoadOp::Load),
                                store: view_normals_store,
                            },
                            depth_slice: None,
                        }),
                        Some(wgpu::RenderPassColorAttachment {
                            view: velocity_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: pending_velocity_load.unwrap_or(wgpu::LoadOp::Load),
                                store: velocity_store,
                            },
                            depth_slice: None,
                        }),
                    ],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: pending_depth_load.unwrap_or(wgpu::LoadOp::Load),
                            store: depth_store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
        }

        {
            let gpu = self.gpu();
            let copy_size = gpu
                .transform_buffer
                .size()
                .min(gpu.prev_transform_buffer.size());
            context.encoder.copy_buffer_to_buffer(
                &gpu.transform_buffer,
                0,
                &gpu.prev_transform_buffer,
                0,
                copy_size,
            );
        }

        Ok(context.into_sub_graph_commands())
    }
}

/// Picks the load op for the `entity_id`, `view_normals`, and `velocity`
/// attachments of a
/// mesh sub-pass. While the graph discards the attachment's stores, nothing
/// consumes its contents, so every sub-pass clears instead of loading. On
/// tile based GPUs a clear-on-load plus discard-on-store attachment never
/// touches memory, while a load fetches the full attachment into tile memory.
/// Otherwise the first sub-pass to bind the attachment takes the graph
/// provided load op and later sub-passes load the accumulated contents.
fn take_aux_load(
    pending_load: &mut Option<wgpu::LoadOp<wgpu::Color>>,
    discard: bool,
) -> wgpu::LoadOp<wgpu::Color> {
    if discard {
        *pending_load = None;
        return wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT);
    }
    pending_load.take().unwrap_or(wgpu::LoadOp::Load)
}
