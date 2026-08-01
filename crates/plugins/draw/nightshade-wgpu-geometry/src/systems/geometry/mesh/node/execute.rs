use super::super::pass::MeshPass;
use super::super::types::{
    CLUSTER_GRID_X, CLUSTER_GRID_Y, CLUSTER_GRID_Z, DrawIndexedIndirect, LightGrid,
    NUM_DRAW_CLASSES, TOTAL_CLUSTERS,
};
use nightshade_wgpu::backend::PassExecutionContext;
use nightshade_wgpu::backend::RenderInputs;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;

const CLEAR_GROUPS_PER_ROW: u32 = 32768;

impl MeshPass {
    pub(in super::super) fn execute_pass_node<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, RenderInputs>,
    ) -> nightshade_wgpu::backend::Result<()> {
        if !context.is_pass_enabled() {
            return Ok(());
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
        self.resize(context.device, scene_color_size.0, scene_color_size.1);

        if scene_color_size != self.transmission.size {
            let (texture, full_view, mip_views) = Self::create_transmission_textures(
                context.device,
                scene_color_size.0,
                scene_color_size.1,
            );
            self.transmission.color_texture = texture;
            self.transmission.color_view = full_view;
            self.transmission.publish_pending = true;
            self.transmission.mip_views = mip_views;
            self.transmission.size = scene_color_size;
            self.bindings.scene_group_dirty = true;
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
                context.device,
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

            compute_pass.set_pipeline(&self.clusters.bounds_pipeline);
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

            compute_pass.set_pipeline(&self.clusters.assign_pipeline);
            compute_pass.set_bind_group(0, cluster_assign_bind_group, &[]);
            let dispatch_x = CLUSTER_GRID_X.div_ceil(8);
            let dispatch_y = CLUSTER_GRID_Y.div_ceil(8);
            compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, CLUSTER_GRID_Z);
        }

        if self.state().object_count > 0 && self.state().indirect_reset_count > 0 {
            if self.compute.batching_enabled {
                let object_count = self.state().object_count;
                let regular_count = self.state().regular_object_count;
                let mesh_count = self.registry.data.len() as u32;
                let material_count = self.state().cached_materials_data.len() as u32;
                let cap = self.compute.batch_cap as u32;
                let batch_params: [u32; 8] = [
                    object_count,
                    regular_count,
                    mesh_count,
                    material_count,
                    cap,
                    self.compute.batch_keys_per_lane as u32,
                    self.gpu().dense_capacity_size as u32,
                    0,
                ];
                context.queue.write_buffer(
                    &self.compute.batch_params,
                    0,
                    bytemuck::cast_slice(&batch_params),
                );
                if self.gpu().gpu_batch_bind_group.is_none() {
                    let bind_group = build::binding(
                        context.device,
                        "Mesh GPU Batch Bind Group",
                        &self.compute.batch_layout,
                        vec![
                            self.gpu().object_buffer.as_entire_binding(),
                            self.gpu().material_flags_buffer.as_entire_binding(),
                            self.gpu().dense_capacity_buffer.as_entire_binding(),
                            self.compute.batch_params.as_entire_binding(),
                            self.gpu().batch_descs_buffer.as_entire_binding(),
                            self.gpu().batch_key_buffer.as_entire_binding(),
                            self.gpu().batch_meta_buffer.as_entire_binding(),
                        ],
                    );
                    self.gpu_mut().gpu_batch_bind_group = Some(bind_group);
                }
                let object_dispatch = object_count.max(1).div_ceil(64);
                let rebuild = self.compute.batch_dirty;
                if rebuild && self.gpu().dense_capacity_clear_pending {
                    let table_words = self.gpu().dense_capacity_size as u32;
                    let mut clear_pass =
                        context
                            .encoder
                            .begin_compute_pass(&wgpu::ComputePassDescriptor {
                                label: Some("Mesh GPU Batch Clear Pass"),
                                timestamp_writes: None,
                            });
                    clear_pass.set_pipeline(&self.compute.batch_clear);
                    clear_pass.set_bind_group(
                        0,
                        self.gpu().gpu_batch_bind_group.as_ref().unwrap(),
                        &[],
                    );
                    let clear_groups = table_words.div_ceil(64);
                    clear_pass.dispatch_workgroups(
                        clear_groups.min(CLEAR_GROUPS_PER_ROW),
                        clear_groups.div_ceil(CLEAR_GROUPS_PER_ROW),
                        1,
                    );
                    self.gpu_mut().dense_capacity_clear_pending = false;
                }
                {
                    let mut classify_pass =
                        context
                            .encoder
                            .begin_compute_pass(&wgpu::ComputePassDescriptor {
                                label: Some("Mesh GPU Classify Pass"),
                                timestamp_writes: None,
                            });
                    classify_pass.set_pipeline(&self.compute.batch_classify);
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
                        count_pass.set_pipeline(&self.compute.batch_count);
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
                        build_pass.set_pipeline(&self.compute.batch_build);
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
                    self.compute.batch_dirty = false;
                }
            }

            if self.registry.geo_dirty {
                let mesh_geo: Vec<super::super::types::MeshGeo> = self
                    .registry
                    .data
                    .iter()
                    .map(|mesh| super::super::types::MeshGeo {
                        index_count: mesh.index_count,
                        first_index: mesh.index_offset,
                        base_vertex: mesh.vertex_offset as i32,
                        padding: 0,
                    })
                    .collect();
                if mesh_geo.len() > self.registry.geo_buffer_size {
                    let new_size = mesh_geo.len().next_power_of_two();
                    self.registry.geo_buffer = build::storage(
                        context.device,
                        "Mesh Geometry Table Buffer (Resized)",
                        (std::mem::size_of::<super::super::types::MeshGeo>() * new_size) as u64,
                    );
                    self.registry.geo_buffer_size = new_size;
                    self.gpu_mut().indirect_build_bind_group = None;
                }
                if !mesh_geo.is_empty() {
                    context.queue.write_buffer(
                        &self.registry.geo_buffer,
                        0,
                        bytemuck::cast_slice(&mesh_geo),
                    );
                }
                self.registry.geo_dirty = false;
            }

            let build_params: [u32; 4] = [self.state().indirect_reset_count as u32, 0, 0, 0];
            context.queue.write_buffer(
                &self.compute.indirect_params,
                0,
                bytemuck::cast_slice(&build_params),
            );

            if self.gpu().indirect_build_bind_group.is_none() {
                let bind_group = build::binding(
                    context.device,
                    "Mesh Indirect Build Bind Group",
                    &self.compute.indirect_layout,
                    vec![
                        self.gpu().indirect_reset_buffer.as_entire_binding(),
                        self.gpu().batch_descs_buffer.as_entire_binding(),
                        self.registry.geo_buffer.as_entire_binding(),
                        self.compute.indirect_params.as_entire_binding(),
                    ],
                );
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
                build_pass.set_pipeline(&self.compute.indirect_pipeline);
                build_pass.set_bind_group(
                    0,
                    self.gpu().indirect_build_bind_group.as_ref().unwrap(),
                    &[],
                );
                build_pass.dispatch_workgroups(1, 1, 1);
            }

            if self.gpu().batch_assign_bind_group.is_none() {
                let bind_group = build::binding(
                    context.device,
                    "Mesh Batch Assign Bind Group",
                    &self.compute.assign_layout,
                    vec![
                        self.gpu().object_buffer.as_entire_binding(),
                        self.gpu().batch_key_buffer.as_entire_binding(),
                        self.compute.assign_params.as_entire_binding(),
                        self.gpu().batch_meta_buffer.as_entire_binding(),
                    ],
                );
                self.gpu_mut().batch_assign_bind_group = Some(bind_group);
            }

            {
                let object_count = self.state().object_count;
                let key_count = self.gpu().batch_key_count;
                let mode = u32::from(self.compute.batching_enabled);
                let cap = self.compute.batch_cap as u32;
                let assign_params: [u32; 8] =
                    [object_count, key_count, u32::MAX, mode, cap, 0, 0, 0];
                context.queue.write_buffer(
                    &self.compute.assign_params,
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
                assign_pass.set_pipeline(&self.compute.assign_pipeline);
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

                compute_pass.set_pipeline(&self.compute.cull_pipeline);
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

                depth_prepass.set_pipeline(&self.pipelines.depth_prepass);
                depth_prepass.set_bind_group(0, &self.bindings.uniform_group, &[]);
                depth_prepass.set_bind_group(1, &self.gpu().instance_bind_group, &[]);
                depth_prepass.set_vertex_buffer(0, self.buffers.vertex.slice(..));
                depth_prepass
                    .set_index_buffer(self.buffers.index.slice(..), wgpu::IndexFormat::Uint32);

                let indirect_buffer = &self.gpu().indirect_buffer;
                let stride = std::mem::size_of::<DrawIndexedIndirect>() as u64;
                let supports_multi_draw = self.compute.supports_multi_draw_count;

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

                let gpu_batching = self.compute.batching_enabled;
                let cap = self.compute.batch_cap as u32;
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

                compute_pass.set_pipeline(&self.compute.cull_pipeline);
                compute_pass.set_bind_group(
                    0,
                    self.gpu().culling_bind_group.as_ref().unwrap(),
                    &[],
                );
                compute_pass.dispatch_workgroups(dispatch_size, 1, 1);
            }
        }

        if self.compute.supports_multi_draw_count && self.state().object_count > 0 {
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
                    [
                        instanced_offset
                            + instanced_opaque
                            + instanced_double_sided
                            + instanced_transparent,
                        state.transmissive_instances.len() as u32,
                    ],
                ]
            };
            context.queue.write_buffer(
                &self.gpu().class_ranges_buffer,
                0,
                bytemuck::cast_slice(&class_ranges),
            );

            if self.gpu().compaction_bind_group.is_none() {
                let bind_group = build::binding(
                    context.device,
                    "Mesh Indirect Compaction Bind Group",
                    &self.compute.compaction_layout,
                    vec![
                        self.gpu().indirect_buffer.as_entire_binding(),
                        self.gpu().draw_count_buffer.as_entire_binding(),
                        self.gpu().class_ranges_buffer.as_entire_binding(),
                        self.compute.compaction_params.as_entire_binding(),
                    ],
                );
                self.gpu_mut().compaction_bind_group = Some(bind_group);
            }

            let mut compaction_pass =
                context
                    .encoder
                    .begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("Mesh Indirect Compaction Pass"),
                        timestamp_writes: None,
                    });
            compaction_pass.set_pipeline(&self.compute.compaction_pipeline);
            compaction_pass.set_bind_group(
                0,
                self.gpu().compaction_bind_group.as_ref().unwrap(),
                &[],
            );
            compaction_pass.dispatch_workgroups(NUM_DRAW_CLASSES as u32, 1, 1);
        }

        if self.compute.batching_enabled && self.state().object_count == 0 {
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

        let point_shadow_cubemap = context
            .get_texture_view("point_shadow_cubemap")
            .unwrap_or(&self.image_lighting.point_shadow_cubemap);

        let brdf_lut_view = context
            .get_texture_view("brdf_lut")
            .unwrap_or(&self.image_lighting.brdf_lut);
        let irradiance_view = context
            .get_texture_view("ibl_irradiance")
            .unwrap_or(&self.image_lighting.irradiance);
        let prefiltered_view = context
            .get_texture_view("ibl_prefiltered")
            .unwrap_or(&self.image_lighting.prefiltered);
        let irradiance_b_view = context
            .get_texture_view("ibl_irradiance_b")
            .unwrap_or(&self.image_lighting.irradiance_b);
        let prefiltered_b_view = context
            .get_texture_view("ibl_prefiltered_b")
            .unwrap_or(&self.image_lighting.prefiltered_b);
        let air_irradiance = irradiance_view.clone();
        let air_irradiance_b = irradiance_b_view.clone();
        let scene_was_dirty = self.bindings.scene_group_dirty;

        if self.bindings.scene_group_dirty {
            let air_reads = self.air.reads();
            self.bindings.scene_group = build::binding(
                context.device,
                "Scene Bind Group (Per-World IBL)",
                &self.bindings.scene_layout,
                vec![
                    wgpu::BindingResource::TextureView(shadow_depth_view),
                    wgpu::BindingResource::Sampler(&self.bindings.shadow_sampler),
                    wgpu::BindingResource::TextureView(spotlight_shadow_atlas_view),
                    wgpu::BindingResource::Sampler(&self.bindings.shadow_sampler),
                    self.bindings.spotlight_shadows.as_entire_binding(),
                    wgpu::BindingResource::TextureView(brdf_lut_view),
                    wgpu::BindingResource::Sampler(&self.bindings.ibl_sampler),
                    wgpu::BindingResource::TextureView(irradiance_view),
                    wgpu::BindingResource::Sampler(&self.bindings.ibl_sampler),
                    wgpu::BindingResource::TextureView(prefiltered_view),
                    wgpu::BindingResource::Sampler(&self.bindings.ibl_sampler),
                    wgpu::BindingResource::TextureView(point_shadow_cubemap),
                    wgpu::BindingResource::Sampler(&self.bindings.point_shadow_sampler),
                    self.bindings.point_shadows.as_entire_binding(),
                    wgpu::BindingResource::TextureView(irradiance_b_view),
                    wgpu::BindingResource::TextureView(prefiltered_b_view),
                    wgpu::BindingResource::TextureView(&self.transmission.color_view),
                    wgpu::BindingResource::Sampler(&self.transmission.sampler),
                    wgpu::BindingResource::Sampler(&self.bindings.shadow_pcf_sampler),
                    wgpu::BindingResource::TextureView(&self.ltc.view_matrix),
                    wgpu::BindingResource::TextureView(&self.ltc.view_terms),
                    wgpu::BindingResource::Sampler(&self.ltc.sampler),
                    wgpu::BindingResource::TextureView(&air_reads.volume),
                    wgpu::BindingResource::Sampler(&air_reads.sampler),
                    air_reads.uniforms.as_entire_binding(),
                ],
            );
            self.bindings.scene_group_dirty = false;
        }

        if scene_was_dirty || !self.air.has_groups() {
            self.air.rebuild_scatter_groups(
                context.device,
                nightshade_wgpu_shading::backend::fog::ScatterReads {
                    shadow_depth: shadow_depth_view,
                    spotlight_shadow_atlas: spotlight_shadow_atlas_view,
                    compare_sampler: &self.bindings.shadow_pcf_sampler,
                    spotlight_shadows: &self.bindings.spotlight_shadows,
                    point_shadow_cubemap,
                    point_shadow_sampler: &self.bindings.point_shadow_sampler,
                    point_light_shadows: &self.bindings.point_shadows,
                    irradiance: &air_irradiance,
                    irradiance_b: &air_irradiance_b,
                    ibl_sampler: &self.bindings.ibl_sampler,
                },
            );
        }
        let lit_air = context
            .configs
            .scene
            .active_view
            .fog
            .filter(|fog| fog.volumetric);
        let focal = context
            .configs
            .scene
            .render_view
            .as_ref()
            .map(|view| [view.projection[(0, 0)], view.projection[(1, 1)]])
            .unwrap_or([1.0, 1.0]);
        self.air.write_uniforms(
            context.queue,
            lit_air.as_ref(),
            scene_color_size,
            focal,
            context.configs.settings.anti_aliasing.is_temporal(),
        );
        if lit_air.is_some() {
            let MeshPass {
                air,
                state,
                bindings,
                materials: material,
                ..
            } = self;
            if let (Some(materials), Some(world)) = (material.bind_group.as_ref(), state.as_ref())
                && let Some(gpu) = world.gpu_buffers.as_ref()
            {
                air.light(
                    context.encoder,
                    &bindings.uniform_group,
                    &gpu.instance_bind_group,
                    materials,
                );
            }
        }

        let instanced_batch_offset = self.state().opaque_instances.len()
            + self.state().opaque_double_sided_instances.len()
            + self.state().transparent_instances.len()
            + self.state().overlay_opaque_instances.len()
            + self.state().overlay_opaque_double_sided_instances.len()
            + self.state().overlay_transparent_instances.len();

        let transmissive_batch_offset = instanced_batch_offset
            + self.state().instanced_opaque_batches.len()
            + self.state().instanced_opaque_double_sided_batches.len()
            + self.state().instanced_transparent_batches.len();

        let draw_indirect_buffer = &self.gpu().indirect_buffer;
        let draw_count_buffer = &self.gpu().draw_count_buffer;
        let supports_count = self.compute.supports_multi_draw_count;
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

            opaque_pass.set_bind_group(0, &self.bindings.uniform_group, &[]);
            opaque_pass.set_bind_group(1, draw_instance_bind_group, &[]);
            opaque_pass.set_bind_group(2, self.materials.bind_group.as_ref().unwrap(), &[]);
            opaque_pass.set_bind_group(3, &self.bindings.scene_group, &[]);
            opaque_pass.set_vertex_buffer(0, self.buffers.vertex.slice(..));
            opaque_pass.set_index_buffer(self.buffers.index.slice(..), wgpu::IndexFormat::Uint32);

            if has_opaque {
                opaque_pass.set_pipeline(&self.pipelines.opaque);

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
                opaque_pass.set_pipeline(&self.pipelines.opaque_double_sided);

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

        if lit_air.is_some() {
            self.air.fog_the_sky(
                context.encoder,
                color_view,
                depth_view,
                pending_depth_load.take().unwrap_or(wgpu::LoadOp::Load),
            );
        }

        let has_blended_transparency = !self.state().transparent_instances.is_empty()
            || !self.state().instanced_transparent_batches.is_empty();
        let has_refractive_transparency = !self.state().transmissive_instances.is_empty();

        let has_transmission_materials = context
            .configs
            .scene
            .render_materials
            .entries
            .iter()
            .any(|entry| entry.material.transmission_factor > 0.0);
        let needs_transmission_copy = has_transmission_materials;

        let transparency_rounds = if needs_transmission_copy && has_refractive_transparency {
            2
        } else {
            1
        };

        for transparency_round in 0..transparency_rounds {
            let is_probe_round = transparency_round + 1 < transparency_rounds;

            if needs_transmission_copy {
                context.encoder.copy_texture_to_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: scene_color_texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyTextureInfo {
                        texture: &self.transmission.color_texture,
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

                if transparency_round > 0
                    && let Some(mip_zero) = self.transmission.mip_views.first()
                {
                    let mut grab_composite_pass =
                        context
                            .encoder
                            .begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("Transmission Grab Composite Pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: mip_zero,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Load,
                                        store: wgpu::StoreOp::Store,
                                    },
                                    depth_slice: None,
                                })],
                                depth_stencil_attachment: None,
                                timestamp_writes: None,
                                occlusion_query_set: None,
                                multiview_mask: None,
                            });
                    grab_composite_pass.set_pipeline(&self.oit.composite_pipeline);
                    grab_composite_pass.set_bind_group(0, &self.oit.composite_bind_group, &[]);
                    grab_composite_pass.draw(0..3, 0..1);
                    drop(grab_composite_pass);
                }

                let mip_count = self.transmission.mip_views.len();
                if mip_count > 1 {
                    let mut downsample_bind_groups = Vec::with_capacity(mip_count - 1);
                    for level in 1..mip_count {
                        downsample_bind_groups.push(build::binding(
                            context.device,
                            "Mesh Transmission Downsample Bind Group",
                            &self.transmission.downsample_layout,
                            vec![
                                wgpu::BindingResource::TextureView(
                                    &self.transmission.mip_views[level - 1],
                                ),
                                wgpu::BindingResource::Sampler(
                                    &self.transmission.downsample_sampler,
                                ),
                            ],
                        ));
                    }

                    for level in 1..mip_count {
                        let mut downsample_pass =
                            context
                                .encoder
                                .begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some("Mesh Transmission Downsample Pass"),
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view: &self.transmission.mip_views[level],
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
                        downsample_pass.set_pipeline(&self.transmission.downsample_pipeline);
                        downsample_pass.set_bind_group(0, &downsample_bind_groups[level - 1], &[]);
                        downsample_pass.draw(0..3, 0..1);
                    }
                }
            }

            if has_blended_transparency || has_refractive_transparency {
                let mut blend_opaque_prepass =
                    context
                        .encoder
                        .begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Blend Opaque Depth Prepass"),
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

                blend_opaque_prepass.set_pipeline(&self.pipelines.blend_depth_prepass);
                blend_opaque_prepass.set_bind_group(0, &self.bindings.uniform_group, &[]);
                blend_opaque_prepass.set_bind_group(1, draw_instance_bind_group, &[]);
                blend_opaque_prepass.set_bind_group(
                    2,
                    self.materials.bind_group.as_ref().unwrap(),
                    &[],
                );
                blend_opaque_prepass.set_bind_group(3, &self.bindings.scene_group, &[]);
                blend_opaque_prepass.set_vertex_buffer(0, self.buffers.vertex.slice(..));
                blend_opaque_prepass
                    .set_index_buffer(self.buffers.index.slice(..), wgpu::IndexFormat::Uint32);

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
                                view: &self.oit.accum_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                    store: wgpu::StoreOp::Store,
                                },
                                depth_slice: None,
                            }),
                            Some(wgpu::RenderPassColorAttachment {
                                view: &self.oit.reveal_view,
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

                oit_pass.set_pipeline(&self.oit.pipeline);
                oit_pass.set_bind_group(0, &self.bindings.uniform_group, &[]);
                oit_pass.set_bind_group(1, draw_instance_bind_group, &[]);
                oit_pass.set_bind_group(2, self.materials.bind_group.as_ref().unwrap(), &[]);
                oit_pass.set_bind_group(3, &self.bindings.scene_group, &[]);
                oit_pass.set_vertex_buffer(0, self.buffers.vertex.slice(..));
                oit_pass.set_index_buffer(self.buffers.index.slice(..), wgpu::IndexFormat::Uint32);

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

                MeshPass::draw_batches(
                    &mut oit_pass,
                    self.state().transmissive_instances.len(),
                    transmissive_batch_offset,
                    9,
                    supports_count,
                    draw_indirect_buffer,
                    draw_count_buffer,
                );

                drop(oit_pass);
            }

            if (has_blended_transparency || has_refractive_transparency) && !is_probe_round {
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

                composite_pass.set_pipeline(&self.oit.composite_pipeline);
                composite_pass.set_bind_group(0, &self.oit.composite_bind_group, &[]);
                composite_pass.draw(0..3, 0..1);

                drop(composite_pass);
            }
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
                            view: &self.pipelines.overlay_depth_view,
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

            overlay_opaque_pass.set_pipeline(&self.pipelines.overlay);
            overlay_opaque_pass.set_bind_group(0, &self.bindings.overlay_uniform_group, &[]);
            overlay_opaque_pass.set_bind_group(1, draw_instance_bind_group, &[]);
            overlay_opaque_pass.set_bind_group(2, self.materials.bind_group.as_ref().unwrap(), &[]);
            overlay_opaque_pass.set_bind_group(3, &self.bindings.scene_group, &[]);
            overlay_opaque_pass.set_vertex_buffer(0, self.buffers.vertex.slice(..));
            overlay_opaque_pass
                .set_index_buffer(self.buffers.index.slice(..), wgpu::IndexFormat::Uint32);

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
                            view: &self.pipelines.overlay_depth_view,
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

            overlay_opaque_double_sided_pass.set_pipeline(&self.pipelines.overlay_double_sided);
            overlay_opaque_double_sided_pass.set_bind_group(
                0,
                &self.bindings.overlay_uniform_group,
                &[],
            );
            overlay_opaque_double_sided_pass.set_bind_group(1, draw_instance_bind_group, &[]);
            overlay_opaque_double_sided_pass.set_bind_group(
                2,
                self.materials.bind_group.as_ref().unwrap(),
                &[],
            );
            overlay_opaque_double_sided_pass.set_bind_group(3, &self.bindings.scene_group, &[]);
            overlay_opaque_double_sided_pass.set_vertex_buffer(0, self.buffers.vertex.slice(..));
            overlay_opaque_double_sided_pass
                .set_index_buffer(self.buffers.index.slice(..), wgpu::IndexFormat::Uint32);

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
                                view: &self.oit.accum_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                    store: wgpu::StoreOp::Store,
                                },
                                depth_slice: None,
                            }),
                            Some(wgpu::RenderPassColorAttachment {
                                view: &self.oit.reveal_view,
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
                            view: &self.pipelines.overlay_depth_view,
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

            overlay_oit_pass.set_pipeline(&self.oit.pipeline);
            overlay_oit_pass.set_bind_group(0, &self.bindings.overlay_uniform_group, &[]);
            overlay_oit_pass.set_bind_group(1, draw_instance_bind_group, &[]);
            overlay_oit_pass.set_bind_group(2, self.materials.bind_group.as_ref().unwrap(), &[]);
            overlay_oit_pass.set_bind_group(3, &self.bindings.scene_group, &[]);
            overlay_oit_pass.set_vertex_buffer(0, self.buffers.vertex.slice(..));
            overlay_oit_pass
                .set_index_buffer(self.buffers.index.slice(..), wgpu::IndexFormat::Uint32);

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

            overlay_composite_pass.set_pipeline(&self.oit.composite_pipeline);
            overlay_composite_pass.set_bind_group(0, &self.oit.composite_bind_group, &[]);
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

        Ok(())
    }
}

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
