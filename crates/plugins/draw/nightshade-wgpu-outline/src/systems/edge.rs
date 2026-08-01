use nightshade_wgpu::backend::build;
use nightshade_wgpu::backend::{PassBuild, PassDecl, PassExecutionContext, PassNode, PassPosition};
use nightshade_wgpu::wgpu;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct OutlineParams {
    outline_color: [f32; 4],
    outline_width: f32,
    texture_width: f32,
    texture_height: f32,
    padding: f32,
}

pub struct OutlinePass {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    params_buffer: wgpu::Buffer,
    cached_bind_group: Option<wgpu::BindGroup>,
    outline_color: [f32; 4],
    outline_width: f32,
    texture_size: (u32, u32),
    enabled: bool,
}

impl OutlinePass {
    pub fn new(build: PassBuild, color_format: wgpu::TextureFormat) -> Self {
        let PassBuild {
            device, composer, ..
        } = build;
        let shader = composer.compile_wgsl(
            device,
            "outline.wgsl",
            include_str!("../shaders/outline.wgsl"),
        );

        let params = OutlineParams {
            outline_color: nightshade_render::resources::outline::DEFAULT_OUTLINE_COLOR,
            outline_width: 2.0,
            texture_width: 1920.0,
            texture_height: 1080.0,
            padding: 0.0,
        };

        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Outline Params Buffer"),
            contents: bytemuck::cast_slice(&[params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = nightshade_wgpu::backend::sampled_texture_uniform_layout(
            device,
            "Outline Bind Group Layout",
        );

        let pipeline_layout =
            build::pipeline_layout(device, "Outline Pipeline Layout", &[&bind_group_layout]);

        let pipeline = build::Pipe::new(device, "Outline Pipeline", &shader)
            .layout(&pipeline_layout)
            .target(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })
            .make();

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        Self {
            pipeline,
            bind_group_layout,
            sampler,
            params_buffer,
            cached_bind_group: None,
            outline_color: nightshade_render::resources::outline::DEFAULT_OUTLINE_COLOR,
            outline_width: 2.0,
            texture_size: (1920, 1080),
            enabled: false,
        }
    }
}

const OUTLINE_PASS_DECL: PassDecl = PassDecl {
    name: "outline_pass",
    position: PassPosition::PostProcess,
    reads: &["outline_mask"],
    reads_writes: &["color"],
    ..PassDecl::EMPTY
};

impl PassNode<nightshade_wgpu::backend::RenderInputs> for OutlinePass {
    fn decl(&self) -> &PassDecl {
        &OUTLINE_PASS_DECL
    }

    fn invalidate_bind_groups(&mut self) {
        self.cached_bind_group = None;
    }

    fn resize(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue, width: u32, height: u32) {
        self.texture_size = (width, height);
    }

    fn prepare(
        &mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &nightshade_wgpu::backend::RenderInputs,
    ) {
        self.enabled = configs
            .scene_world
            .query_tag_type::<nightshade_render::components::render_world::Outlined>()
            .next()
            .is_some();

        if !self.enabled {
            return;
        }

        self.outline_color = configs.scene.outline_color;

        let params = OutlineParams {
            outline_color: self.outline_color,
            outline_width: self.outline_width,
            texture_width: self.texture_size.0 as f32,
            texture_height: self.texture_size.1 as f32,
            padding: 0.0,
        };
        queue.write_buffer(&self.params_buffer, 0, bytemuck::cast_slice(&[params]));
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, nightshade_wgpu::backend::RenderInputs>,
    ) -> nightshade_wgpu::backend::Result<()> {
        if !self.enabled {
            return Ok(());
        }

        if self.cached_bind_group.is_none() {
            let mask_view = context.get_texture_view("outline_mask")?;

            if let Ok(size) = context.get_texture_size("outline_mask") {
                self.texture_size = size;
            }

            self.cached_bind_group = Some(build::binding(
                context.device,
                "Outline Bind Group",
                &self.bind_group_layout,
                vec![
                    wgpu::BindingResource::TextureView(mask_view),
                    wgpu::BindingResource::Sampler(&self.sampler),
                    self.params_buffer.as_entire_binding(),
                ],
            ));
        }

        let (output_view, _, output_store_op) = context.get_color_attachment("color")?;

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Outline Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: output_store_op,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, self.cached_bind_group.as_ref().unwrap(), &[]);
        render_pass.draw(0..3, 0..1);

        drop(render_pass);

        Ok(())
    }
}
