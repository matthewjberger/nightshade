use nightshade_wgpu::backend::{PassDecl, PassExecutionContext, PassNode, PassPosition};
use nightshade_wgpu::wgpu;

pub trait OverlaySubPass: Send + Sync + 'static {
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &nightshade_wgpu::backend::RenderInputs,
    );

    fn wants(&self) -> &'static [&'static str] {
        &[]
    }

    fn receive(&mut self, _name: &str, _device: &wgpu::Device, _view: &wgpu::TextureView) {}

    fn invalidate(&mut self) {}

    fn encode_compute(&mut self, _encoder: &mut wgpu::CommandEncoder) {}

    fn record(
        &mut self,
        render_pass: &mut wgpu::RenderPass<'_>,
        device: &wgpu::Device,
        configs: &nightshade_wgpu::backend::RenderInputs,
    );

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub struct SceneOverlayPass {
    pub sub_passes: Vec<Box<dyn OverlaySubPass>>,
}

impl SceneOverlayPass {
    pub fn sub_pass_mut<P: OverlaySubPass>(&mut self) -> Option<&mut P> {
        self.sub_passes
            .iter_mut()
            .find_map(|sub_pass| sub_pass.as_any_mut().downcast_mut::<P>())
    }
}

const SCENE_OVERLAY_PASS_DECL: PassDecl = PassDecl {
    name: "scene_overlay_pass",
    position: PassPosition::SceneOverlay,
    reads_writes: &["color", "depth"],
    optional_reads: &["glyph_atlas"],
    ..PassDecl::EMPTY
};

impl PassNode<nightshade_wgpu::backend::RenderInputs> for SceneOverlayPass {
    fn decl(&self) -> &PassDecl {
        &SCENE_OVERLAY_PASS_DECL
    }

    fn invalidate_bind_groups(&mut self) {
        for sub_pass in &mut self.sub_passes {
            sub_pass.invalidate();
        }
    }

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &nightshade_wgpu::backend::RenderInputs,
    ) {
        for sub_pass in &mut self.sub_passes {
            sub_pass.prepare(device, queue, configs);
        }
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, nightshade_wgpu::backend::RenderInputs>,
    ) -> nightshade_wgpu::backend::Result<()> {
        if !context.is_pass_enabled() {
            return Ok(());
        }

        for sub_pass in &mut self.sub_passes {
            for name in sub_pass.wants() {
                if let Ok(view) = context.get_texture_view(name) {
                    sub_pass.receive(name, context.device, view);
                }
            }
        }

        for sub_pass in &mut self.sub_passes {
            sub_pass.encode_compute(context.encoder);
        }

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

        for sub_pass in &mut self.sub_passes {
            sub_pass.record(&mut render_pass, context.device, context.configs);
        }

        drop(render_pass);

        Ok(())
    }
}
