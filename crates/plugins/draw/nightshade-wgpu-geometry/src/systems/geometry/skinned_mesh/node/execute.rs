use super::super::pass::SkinnedMeshPass;
use nightshade_wgpu::backend::PassExecutionContext;
use nightshade_wgpu::backend::RenderInputs;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;

impl SkinnedMeshPass {
    fn transparent_indirect_offset(&self) -> u64 {
        (self.state().opaque_draw_groups.len()
            * std::mem::size_of::<super::super::types::SkinnedDrawIndexedIndirect>()) as u64
    }

    pub(in super::super) fn execute_pass_node<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, RenderInputs>,
    ) -> nightshade_wgpu::backend::Result<()> {
        if !context.is_pass_enabled() {
            return Ok(());
        }
        if self.state().skinned_entities.is_empty() {
            return Ok(());
        }

        if self.animation.skeleton_count() > 0 {
            let mut animation_pass =
                context
                    .encoder
                    .begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("Animation Compute Pass"),
                        timestamp_writes: None,
                    });
            self.animation.dispatch(&mut animation_pass);
        }

        if self.spring.chain_count() > 0 {
            let mut spring_pass =
                context
                    .encoder
                    .begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("Skinned Spring Compute Pass"),
                        timestamp_writes: None,
                    });
            self.spring.dispatch(&mut spring_pass);
        }

        if self.ik.chain_count() > 0 {
            let mut ik_pass = context
                .encoder
                .begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Skinned IK Compute Pass"),
                    timestamp_writes: None,
                });
            self.ik.dispatch(&mut ik_pass);
        }

        if self.multi_ik.chain_count() > 0 {
            let mut multi_ik_pass =
                context
                    .encoder
                    .begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("Skinned CCD IK Compute Pass"),
                        timestamp_writes: None,
                    });
            self.multi_ik.dispatch(&mut multi_ik_pass);
        }

        if self.aim.constraint_count() > 0 {
            let mut aim_pass = context
                .encoder
                .begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Skinned Aim Compute Pass"),
                    timestamp_writes: None,
                });
            self.aim.dispatch(&mut aim_pass);
        }

        if self.skinning.joints_to_dispatch > 0 {
            let mut compute_pass =
                context
                    .encoder
                    .begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("Skinning Compute Pass"),
                        timestamp_writes: None,
                    });

            compute_pass.set_pipeline(&self.skinning.pipeline);
            compute_pass.set_bind_group(0, &self.skinning.bind_group, &[]);

            let num_workgroups = self.skinning.joints_to_dispatch.div_ceil(64);
            compute_pass.dispatch_workgroups(num_workgroups, 1, 1);
        }

        if self.compute.bounds_dispatch > 0
            && let Some(bounds_bind_group) = self.compute.bounds_group.as_ref()
        {
            let mut compute_pass =
                context
                    .encoder
                    .begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("Skinned Bounds Compute Pass"),
                        timestamp_writes: None,
                    });
            compute_pass.set_pipeline(&self.compute.bounds_pipeline);
            compute_pass.set_bind_group(0, bounds_bind_group, &[]);
            compute_pass.dispatch_workgroups(self.compute.bounds_dispatch.div_ceil(64), 1, 1);
        }

        if self.compute.batching_enabled {
            let object_count = self.state().skinned_entities.len() as u32;
            let mesh_count = self.buffers.data.len().max(1) as u32;
            if self.gpu().batch_bind_group.is_none() {
                let bind_group = build::binding(
                    context.device,
                    "Skinned Mesh Batch Bind Group",
                    &self.compute.batch_layout,
                    vec![
                        self.gpu().object_buffer.as_entire_binding(),
                        self.gpu().cull_objects_buffer.as_entire_binding(),
                        self.gpu().dense_capacity_buffer.as_entire_binding(),
                        self.gpu().command_map_buffer.as_entire_binding(),
                        self.gpu().indirect_commands_buffer.as_entire_binding(),
                        self.gpu().batch_meta_buffer.as_entire_binding(),
                        self.compute.batch_geo.as_entire_binding(),
                        self.compute.batch_params.as_entire_binding(),
                    ],
                );
                self.gpu_mut().batch_bind_group = Some(bind_group);
            }
            let passes: [(&wgpu::ComputePipeline, u32); 4] = [
                (&self.compute.batch_clear, (2 * mesh_count).div_ceil(64)),
                (&self.compute.batch_count, object_count.max(1).div_ceil(64)),
                (&self.compute.batch_build, 1),
                (&self.compute.batch_assign, object_count.max(1).div_ceil(64)),
            ];
            for (pipeline, dispatch) in passes {
                let mut pass = context
                    .encoder
                    .begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("Skinned Mesh Batch Pass"),
                        timestamp_writes: None,
                    });
                pass.set_pipeline(pipeline);
                pass.set_bind_group(0, self.gpu().batch_bind_group.as_ref().unwrap(), &[]);
                pass.dispatch_workgroups(dispatch, 1, 1);
            }
        }

        {
            let object_count = self.state().skinned_entities.len() as u32;
            let mut cull_pass = context
                .encoder
                .begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Skinned Mesh Cull Pass"),
                    timestamp_writes: None,
                });
            cull_pass.set_pipeline(&self.compute.cull_pipeline);
            cull_pass.set_bind_group(0, &self.gpu().cull_bind_group, &[]);
            cull_pass.dispatch_workgroups(object_count.div_ceil(64), 1, 1);
        }

        {
            let dispatch_x = super::super::types::CLUSTER_GRID_X.div_ceil(8);
            let dispatch_y = super::super::types::CLUSTER_GRID_Y.div_ceil(8);
            let dispatch_z = super::super::types::CLUSTER_GRID_Z;
            {
                let mut compute_pass =
                    context
                        .encoder
                        .begin_compute_pass(&wgpu::ComputePassDescriptor {
                            label: Some("Skinned Cluster Bounds Pass"),
                            timestamp_writes: None,
                        });
                compute_pass.set_pipeline(&self.clusters.bounds_pipeline);
                compute_pass.set_bind_group(0, &self.gpu().cluster_bounds_bind_group, &[]);
                compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, dispatch_z);
            }

            let light_grid_size = (std::mem::size_of::<super::super::types::LightGrid>()
                * super::super::types::TOTAL_CLUSTERS as usize)
                as u64;
            context.encoder.copy_buffer_to_buffer(
                &self.gpu().light_grid_reset_buffer,
                0,
                &self.gpu().light_grid_buffer,
                0,
                light_grid_size,
            );

            if let Some(cluster_assign_bind_group) = self.gpu().cluster_assign_bind_group.as_ref() {
                let mut compute_pass =
                    context
                        .encoder
                        .begin_compute_pass(&wgpu::ComputePassDescriptor {
                            label: Some("Skinned Cluster Light Assignment Pass"),
                            timestamp_writes: None,
                        });
                compute_pass.set_pipeline(&self.clusters.assign_pipeline);
                compute_pass.set_bind_group(0, cluster_assign_bind_group, &[]);
                compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, dispatch_z);
            }
        }

        let (color_view, color_load, color_store) = context.get_color_attachment("color")?;
        let (depth_view, depth_load, depth_store) = context.get_depth_attachment("depth")?;
        let (shadow_depth_view, _, _) = context.get_depth_attachment("shadow_depth")?;
        let (spotlight_shadow_atlas_view, _, _) =
            context.get_depth_attachment("spotlight_shadow_atlas")?;
        let point_shadow_cubemap = context
            .get_texture_view("point_shadow_cubemap")
            .unwrap_or(&self.bindings.point_shadow_cubemap);
        let transmission_grab = context
            .get_texture_view("transmission_grab")
            .unwrap_or(&self.images.white);
        let (entity_id_view, entity_id_load, entity_id_store) =
            context.get_color_attachment("entity_id")?;
        let (view_normals_view, view_normals_load, view_normals_store) =
            context.get_color_attachment("view_normals")?;
        let (velocity_view, velocity_load, velocity_store) =
            context.get_color_attachment("velocity")?;

        let brdf_view = context
            .get_texture_view("brdf_lut")
            .unwrap_or(&self.images.brdf_lut);
        let irradiance_view = context
            .get_texture_view("ibl_irradiance")
            .ok()
            .or(self.images.irradiance.as_ref())
            .unwrap_or(&self.images.cube);
        let prefiltered_view = context
            .get_texture_view("ibl_prefiltered")
            .ok()
            .or(self.images.prefiltered.as_ref())
            .unwrap_or(&self.images.cube);
        let irradiance_b_view = context
            .get_texture_view("ibl_irradiance_b")
            .ok()
            .or(self.images.irradiance_b.as_ref())
            .unwrap_or(&self.images.cube);
        let prefiltered_b_view = context
            .get_texture_view("ibl_prefiltered_b")
            .ok()
            .or(self.images.prefiltered_b.as_ref())
            .unwrap_or(&self.images.cube);

        self.bindings.shadow_group = build::binding(
            context.device,
            "Skinned Mesh Shadow Bind Group",
            &self.bindings.shadow_layout,
            vec![
                wgpu::BindingResource::TextureView(shadow_depth_view),
                wgpu::BindingResource::Sampler(&self.bindings.shadow_sampler),
                wgpu::BindingResource::TextureView(spotlight_shadow_atlas_view),
                wgpu::BindingResource::Sampler(&self.bindings.shadow_sampler),
                self.bindings.spotlight_shadows.as_entire_binding(),
                wgpu::BindingResource::TextureView(brdf_view),
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
                wgpu::BindingResource::TextureView(transmission_grab),
                wgpu::BindingResource::Sampler(&self.images.sampler),
                wgpu::BindingResource::Sampler(&self.bindings.shadow_pcf_sampler),
                wgpu::BindingResource::TextureView(&self.ltc.view_matrix),
                wgpu::BindingResource::TextureView(&self.ltc.view_terms),
                wgpu::BindingResource::Sampler(&self.ltc.sampler),
                wgpu::BindingResource::TextureView(&self.air.volume),
                wgpu::BindingResource::Sampler(&self.air.sampler),
                self.air.uniforms.as_entire_binding(),
            ],
        );

        if self.materials.bind_group.is_none() && self.materials.bindless_max.is_none() {
            self.rebuild_material_bind_group(context.device);
        }
        let material_bind_group = self.materials.bind_group.clone().unwrap();

        if !self.state().opaque_draw_groups.is_empty() {
            let mut render_pass = context
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Skinned Mesh Opaque Pass"),
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view: color_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: color_load,
                                store: color_store,
                            },
                            depth_slice: None,
                        }),
                        Some(wgpu::RenderPassColorAttachment {
                            view: entity_id_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: entity_id_load,
                                store: entity_id_store,
                            },
                            depth_slice: None,
                        }),
                        Some(wgpu::RenderPassColorAttachment {
                            view: view_normals_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: view_normals_load,
                                store: view_normals_store,
                            },
                            depth_slice: None,
                        }),
                        Some(wgpu::RenderPassColorAttachment {
                            view: velocity_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: velocity_load,
                                store: velocity_store,
                            },
                            depth_slice: None,
                        }),
                    ],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: depth_load,
                            store: depth_store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bindings.uniform_group, &[]);
            render_pass.set_bind_group(1, &self.gpu().instance_bind_group, &[]);
            render_pass.set_bind_group(3, &self.bindings.shadow_group, &[]);

            render_pass.set_vertex_buffer(0, self.buffers.vertex.slice(..));
            render_pass.set_index_buffer(self.buffers.index.slice(..), wgpu::IndexFormat::Uint32);

            render_pass.set_bind_group(2, &material_bind_group, &[]);
            let opaque_command_count = self.state().opaque_draw_groups.len() as u32;
            if opaque_command_count > 0 {
                render_pass.multi_draw_indexed_indirect(
                    &self.gpu().indirect_commands_buffer,
                    0,
                    opaque_command_count,
                );
            }

            drop(render_pass);
        }

        if !self.state().transparent_draw_groups.is_empty() {
            let (width, height) = context.get_texture_size("color")?;
            self.resize_oit_textures(context.device, width, height);

            {
                let mut blend_opaque_prepass =
                    context
                        .encoder
                        .begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Skinned Mesh Blend Opaque Depth Prepass"),
                            color_attachments: &[],
                            depth_stencil_attachment: Some(
                                wgpu::RenderPassDepthStencilAttachment {
                                    view: depth_view,
                                    depth_ops: Some(wgpu::Operations {
                                        load: wgpu::LoadOp::Load,
                                        store: wgpu::StoreOp::Store,
                                    }),
                                    stencil_ops: None,
                                },
                            ),
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        });

                blend_opaque_prepass.set_pipeline(&self.blend_opaque_depth_prepass_pipeline);
                blend_opaque_prepass.set_bind_group(0, &self.bindings.uniform_group, &[]);
                blend_opaque_prepass.set_bind_group(1, &self.gpu().instance_bind_group, &[]);
                blend_opaque_prepass.set_bind_group(3, &self.bindings.shadow_group, &[]);

                blend_opaque_prepass.set_vertex_buffer(0, self.buffers.vertex.slice(..));
                blend_opaque_prepass
                    .set_index_buffer(self.buffers.index.slice(..), wgpu::IndexFormat::Uint32);

                blend_opaque_prepass.set_bind_group(2, &material_bind_group, &[]);
                let transparent_command_count = self.state().transparent_draw_groups.len() as u32;
                if transparent_command_count > 0 {
                    blend_opaque_prepass.multi_draw_indexed_indirect(
                        &self.gpu().indirect_commands_buffer,
                        self.transparent_indirect_offset(),
                        transparent_command_count,
                    );
                }

                drop(blend_opaque_prepass);
            }

            {
                let mut oit_pass = context
                    .encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Skinned Mesh OIT Pass"),
                        color_attachments: &[
                            Some(wgpu::RenderPassColorAttachment {
                                view: &self.oit.accum_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color {
                                        r: 0.0,
                                        g: 0.0,
                                        b: 0.0,
                                        a: 0.0,
                                    }),
                                    store: wgpu::StoreOp::Store,
                                },
                                depth_slice: None,
                            }),
                            Some(wgpu::RenderPassColorAttachment {
                                view: &self.oit.reveal_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color {
                                        r: 1.0,
                                        g: 0.0,
                                        b: 0.0,
                                        a: 1.0,
                                    }),
                                    store: wgpu::StoreOp::Store,
                                },
                                depth_slice: None,
                            }),
                            Some(wgpu::RenderPassColorAttachment {
                                view: entity_id_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: entity_id_store,
                                },
                                depth_slice: None,
                            }),
                            Some(wgpu::RenderPassColorAttachment {
                                view: velocity_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: velocity_load,
                                    store: velocity_store,
                                },
                                depth_slice: None,
                            }),
                        ],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: depth_view,
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

                oit_pass.set_pipeline(&self.oit.render_pipeline);
                oit_pass.set_bind_group(0, &self.bindings.uniform_group, &[]);
                oit_pass.set_bind_group(1, &self.gpu().instance_bind_group, &[]);
                oit_pass.set_bind_group(3, &self.bindings.shadow_group, &[]);

                oit_pass.set_vertex_buffer(0, self.buffers.vertex.slice(..));
                oit_pass.set_index_buffer(self.buffers.index.slice(..), wgpu::IndexFormat::Uint32);

                oit_pass.set_bind_group(2, &material_bind_group, &[]);
                let transparent_command_count = self.state().transparent_draw_groups.len() as u32;
                if transparent_command_count > 0 {
                    oit_pass.multi_draw_indexed_indirect(
                        &self.gpu().indirect_commands_buffer,
                        self.transparent_indirect_offset(),
                        transparent_command_count,
                    );
                }
            }

            let oit_composite_bind_group = build::binding(
                context.device,
                "Skinned Mesh OIT Composite Bind Group",
                &self.oit.composite_layout,
                vec![
                    wgpu::BindingResource::TextureView(&self.oit.accum_view),
                    wgpu::BindingResource::Sampler(&self.oit.sampler),
                    wgpu::BindingResource::TextureView(&self.oit.reveal_view),
                    wgpu::BindingResource::Sampler(&self.oit.sampler),
                ],
            );

            let mut composite_pass =
                context
                    .encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Skinned Mesh OIT Composite Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: color_view,
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

            composite_pass.set_pipeline(&self.oit.composite_pipeline);
            composite_pass.set_bind_group(0, &oit_composite_bind_group, &[]);
            composite_pass.draw(0..3, 0..1);
        }

        let joint_copy_size = self
            .skinning
            .joints
            .size()
            .min(self.skinning.previous_joints.size());
        context.encoder.copy_buffer_to_buffer(
            &self.skinning.joints,
            0,
            &self.skinning.previous_joints,
            0,
            joint_copy_size,
        );

        Ok(())
    }
}
