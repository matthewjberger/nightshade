//! Combined overlay pass for everything drawn over the lit scene with the
//! shared `scene_color` and `depth` attachments: the infinite grid, debug
//! lines, GPU particles, and 3D text.
//!
//! Drawing all four through one render pass matters on tile based GPUs,
//! where every render pass boundary stores the attachments from tile
//! memory to RAM and loads them back for the next pass. As separate
//! passes these cost four full screen load/store round trips per frame.
//! Merged, they cost one.
//!
//! Compute work the sub-passes need (line generation and culling,
//! particle simulation) is encoded before the shared render pass begins.
//! The individual pass types keep their own `PassNode` implementations
//! for graphs that register them standalone.

use super::grid::GridPass;
use super::lines::LinesPass;
use super::particles::ParticlePass;
use super::text::TextPass;
use crate::wgpu::rendergraph::{PassExecutionContext, PassNode};

pub struct SceneOverlayPass {
    pub grid: GridPass,
    pub lines: LinesPass,
    pub particles: ParticlePass,
    pub text: TextPass,
}

impl PassNode<crate::wgpu::render_configs::RenderInputs> for SceneOverlayPass {
    fn name(&self) -> &str {
        "scene_overlay_pass"
    }

    fn reads(&self) -> Vec<&str> {
        vec![]
    }

    fn writes(&self) -> Vec<&str> {
        vec![]
    }

    fn reads_writes(&self) -> Vec<&str> {
        vec!["color", "depth"]
    }

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &crate::wgpu::render_configs::RenderInputs,
    ) {
        self.grid.prepare(device, queue, configs);
        self.lines.prepare(device, queue, configs);
        self.particles.prepare(device, queue, configs);
        self.text.prepare(device, queue, configs);
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, crate::wgpu::render_configs::RenderInputs>,
    ) -> crate::wgpu::rendergraph::Result<Vec<crate::wgpu::rendergraph::SubGraphRunCommand<'r>>>
    {
        if !context.is_pass_enabled() {
            return Ok(context.into_sub_graph_commands());
        }

        let culling_enabled = context.configs.scene.gpu_culling_enabled;
        self.lines.encode_compute(context.encoder, culling_enabled);
        self.particles.encode_simulation(context.encoder);

        let (color_view, color_load, color_store) = context.get_color_attachment("color")?;
        let (depth_view, depth_load, depth_store) = context.get_depth_attachment("depth")?;

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Scene Overlay Pass"),
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

        self.grid.record(&mut render_pass, context.configs);
        self.lines.record(&mut render_pass, culling_enabled);
        self.particles.record(&mut render_pass);
        self.text.record(&mut render_pass, context.device);

        drop(render_pass);

        Ok(context.into_sub_graph_commands())
    }
}
