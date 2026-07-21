use super::super::pass::SkinnedMeshPass;
use crate::wgpu::render_configs::RenderInputs;
use crate::wgpu::rendergraph::PassExecutionContext;

impl SkinnedMeshPass {
    fn transparent_indirect_offset(&self) -> u64 {
        (self.state().opaque_draw_groups.len()
            * std::mem::size_of::<super::super::types::SkinnedDrawIndexedIndirect>()) as u64
    }

    pub(in super::super) fn execute_pass_node<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, RenderInputs>,
    ) -> crate::wgpu::rendergraph::Result<Vec<crate::wgpu::rendergraph::SubGraphRunCommand<'r>>>
    {
        if !context.is_pass_enabled() {
            return Ok(context.into_sub_graph_commands());
        }
        if self.state().skinned_entities.is_empty() {
            return Ok(vec![]);
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

        if self.total_joints_to_dispatch > 0 {
            let mut compute_pass =
                context
                    .encoder
                    .begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("Skinning Compute Pass"),
                        timestamp_writes: None,
                    });

            compute_pass.set_pipeline(&self.skinning_compute_pipeline);
            compute_pass.set_bind_group(0, &self.skinning_compute_bind_group, &[]);

            let num_workgroups = self.total_joints_to_dispatch.div_ceil(64);
            compute_pass.dispatch_workgroups(num_workgroups, 1, 1);
        }

        if self.bounds_dispatch_count > 0
            && let Some(bounds_bind_group) = self.bounds_compute_bind_group.as_ref()
        {
            let mut compute_pass =
                context
                    .encoder
                    .begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("Skinned Bounds Compute Pass"),
                        timestamp_writes: None,
                    });
            compute_pass.set_pipeline(&self.bounds_compute_pipeline);
            compute_pass.set_bind_group(0, bounds_bind_group, &[]);
            compute_pass.dispatch_workgroups(self.bounds_dispatch_count.div_ceil(64), 1, 1);
        }

        if self.gpu_batching_enabled {
            let object_count = self.state().skinned_entities.len() as u32;
            let mesh_count = self.skinned_mesh_data.len().max(1) as u32;
            if self.gpu().batch_bind_group.is_none() {
                let bind_group = context
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Skinned Mesh Batch Bind Group"),
                        layout: &self.batch_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: self.gpu().object_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: self.gpu().cull_objects_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: self.gpu().dense_capacity_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: self.gpu().command_map_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 4,
                                resource: self.gpu().indirect_commands_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 5,
                                resource: self.gpu().batch_meta_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 6,
                                resource: self.batch_mesh_geo_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 7,
                                resource: self.batch_params_buffer.as_entire_binding(),
                            },
                        ],
                    });
                self.gpu_mut().batch_bind_group = Some(bind_group);
            }
            let passes: [(&wgpu::ComputePipeline, u32); 4] = [
                (&self.batch_clear_pipeline, (2 * mesh_count).div_ceil(64)),
                (&self.batch_count_pipeline, object_count.max(1).div_ceil(64)),
                (&self.batch_build_pipeline, 1),
                (
                    &self.batch_assign_pipeline,
                    object_count.max(1).div_ceil(64),
                ),
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
            cull_pass.set_pipeline(&self.cull_pipeline);
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
                compute_pass.set_pipeline(&self.cluster_bounds_pipeline);
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
                compute_pass.set_pipeline(&self.cluster_assign_pipeline);
                compute_pass.set_bind_group(0, cluster_assign_bind_group, &[]);
                compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, dispatch_z);
            }
        }

        let (color_view, color_load, color_store) = context.get_color_attachment("color")?;
        let (depth_view, depth_load, depth_store) = context.get_depth_attachment("depth")?;
        let (shadow_depth_view, _, _) = context.get_depth_attachment("shadow_depth")?;
        let (spotlight_shadow_atlas_view, _, _) =
            context.get_depth_attachment("spotlight_shadow_atlas")?;
        let (entity_id_view, entity_id_load, entity_id_store) =
            context.get_color_attachment("entity_id")?;
        let (velocity_view, velocity_load, velocity_store) =
            context.get_color_attachment("velocity")?;

        let world_state = self.state();
        let brdf_view = world_state
            .ibl_brdf_lut_view
            .as_ref()
            .or(self.brdf_lut_view.as_ref())
            .unwrap_or(&self.dummy_white_view);
        let irradiance_view = world_state
            .ibl_irradiance_view
            .as_ref()
            .or(self.irradiance_map_view.as_ref())
            .unwrap_or(&self.dummy_cube_view);
        let prefiltered_view = world_state
            .ibl_prefiltered_view
            .as_ref()
            .or(self.prefiltered_env_view.as_ref())
            .unwrap_or(&self.dummy_cube_view);
        let irradiance_b_view = world_state
            .ibl_irradiance_b_view
            .as_ref()
            .or(self.irradiance_b_view.as_ref())
            .unwrap_or(&self.dummy_cube_view);
        let prefiltered_b_view = world_state
            .ibl_prefiltered_b_view
            .as_ref()
            .or(self.prefiltered_b_view.as_ref())
            .unwrap_or(&self.dummy_cube_view);

        self.shadow_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Skinned Mesh Shadow Bind Group"),
                layout: &self.shadow_bind_group_layout,
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
                        resource: wgpu::BindingResource::TextureView(spotlight_shadow_atlas_view),
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
                        resource: wgpu::BindingResource::TextureView(brdf_view),
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
                        resource: wgpu::BindingResource::TextureView(&self.dummy_white_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 17,
                        resource: wgpu::BindingResource::Sampler(&self.dummy_sampler),
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

        if self.material_bind_group.is_none() && self.material_bindless_max.is_none() {
            self.rebuild_material_bind_group(context.device);
        }
        let material_bind_group = self.material_bind_group.clone().unwrap();

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
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_bind_group(1, &self.gpu().instance_bind_group, &[]);
            render_pass.set_bind_group(3, &self.shadow_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.skinned_vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                self.skinned_index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );

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
                blend_opaque_prepass.set_bind_group(0, &self.uniform_bind_group, &[]);
                blend_opaque_prepass.set_bind_group(1, &self.gpu().instance_bind_group, &[]);
                blend_opaque_prepass.set_bind_group(3, &self.shadow_bind_group, &[]);

                blend_opaque_prepass.set_vertex_buffer(0, self.skinned_vertex_buffer.slice(..));
                blend_opaque_prepass.set_index_buffer(
                    self.skinned_index_buffer.slice(..),
                    wgpu::IndexFormat::Uint32,
                );

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
                                view: &self.oit_accum_view,
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
                                view: &self.oit_reveal_view,
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

                oit_pass.set_pipeline(&self.oit_render_pipeline);
                oit_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                oit_pass.set_bind_group(1, &self.gpu().instance_bind_group, &[]);
                oit_pass.set_bind_group(3, &self.shadow_bind_group, &[]);

                oit_pass.set_vertex_buffer(0, self.skinned_vertex_buffer.slice(..));
                oit_pass.set_index_buffer(
                    self.skinned_index_buffer.slice(..),
                    wgpu::IndexFormat::Uint32,
                );

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

            let oit_composite_bind_group =
                context
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Skinned Mesh OIT Composite Bind Group"),
                        layout: &self.oit_composite_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(&self.oit_accum_view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(&self.oit_sampler),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: wgpu::BindingResource::TextureView(&self.oit_reveal_view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: wgpu::BindingResource::Sampler(&self.oit_sampler),
                            },
                        ],
                    });

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

            composite_pass.set_pipeline(&self.oit_composite_pipeline);
            composite_pass.set_bind_group(0, &oit_composite_bind_group, &[]);
            composite_pass.draw(0..3, 0..1);
        }

        let joint_copy_size = self
            .joint_matrices_buffer
            .size()
            .min(self.prev_joint_matrices_buffer.size());
        context.encoder.copy_buffer_to_buffer(
            &self.joint_matrices_buffer,
            0,
            &self.prev_joint_matrices_buffer,
            0,
            joint_copy_size,
        );

        Ok(vec![])
    }
}
