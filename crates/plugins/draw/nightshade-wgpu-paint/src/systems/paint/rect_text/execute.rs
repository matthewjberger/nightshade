use nightshade_wgpu::backend::PassExecutionContext;
use nightshade_wgpu::wgpu;

use super::PaintPass;

impl PaintPass {
    pub(super) fn execute_pass<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, nightshade_wgpu::backend::RenderInputs>,
    ) -> Result<(), nightshade_wgpu::backend::RenderGraphError> {
        if !context.is_pass_enabled() {
            return Ok(());
        }

        let (color_view, color_load, color_store) = context.get_color_attachment("color")?;
        let (depth_view, depth_load, depth_store) = context.get_depth_attachment("depth")?;
        let (render_target_width, render_target_height) = context.get_texture_size("color")?;

        let has_rects = self.rect_count > 0;
        let has_text = !self.text_draw_calls.is_empty();

        if has_rects || has_text {
            let mut render_pass = context
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("UI Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: color_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: color_load,
                            store: color_store,
                        },
                        depth_slice: None,
                    })],
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

            render_pass.set_scissor_rect(0, 0, render_target_width, render_target_height);

            for group in &self.layer_draw_groups {
                if group.rect_count > 0 {
                    render_pass.set_pipeline(&self.rect_pipeline);
                    render_pass.set_bind_group(0, &self.global_uniform_bind_group, &[]);
                    render_pass.set_bind_group(1, &self.rect_instance_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, self.rect_quad_vertex_buffer.slice(..));
                    render_pass.set_index_buffer(
                        self.rect_quad_index_buffer.slice(..),
                        wgpu::IndexFormat::Uint16,
                    );
                    render_pass.draw_indexed(
                        0..6,
                        0,
                        group.rect_start..(group.rect_start + group.rect_count),
                    );
                }

                if group.text_draw_count > 0 {
                    render_pass.set_pipeline(&self.text_pipeline);
                    render_pass.set_bind_group(0, &self.text_data_bind_group, &[]);
                    render_pass.set_bind_group(2, &self.text_global_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, self.text_vertex_buffer.slice(..));
                    render_pass.set_index_buffer(
                        self.text_index_buffer.slice(..),
                        wgpu::IndexFormat::Uint32,
                    );

                    let text_end = group.text_draw_start + group.text_draw_count;
                    if let Some(atlas_bind_group) = self.text_font_bind_groups.first() {
                        render_pass.set_bind_group(1, atlas_bind_group, &[]);
                        for draw_call in &self.text_draw_calls[group.text_draw_start..text_end] {
                            if draw_call.index_count == 0 {
                                continue;
                            }
                            render_pass.draw_indexed(
                                draw_call.index_start
                                    ..(draw_call.index_start + draw_call.index_count),
                                0,
                                0..1,
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
