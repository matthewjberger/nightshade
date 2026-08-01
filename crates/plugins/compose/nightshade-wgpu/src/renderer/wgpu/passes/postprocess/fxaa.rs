use crate::renderer::wgpu::build;
use crate::renderer::wgpu::pass_build::PassBuild;
use crate::renderer::wgpu::rendergraph::{PassDecl, PassExecutionContext, PassNode, PassPosition};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct FxaaParams {
    texel: [f32; 2],
    relative_threshold: f32,
    absolute_threshold: f32,
    subpixel: f32,
    padding: [f32; 3],
}

const RELATIVE_THRESHOLD: f32 = 0.125;
const ABSOLUTE_THRESHOLD: f32 = 0.0312;
const SUBPIXEL: f32 = 0.75;

pub struct FxaaPass {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    params_buffer: wgpu::Buffer,
    bind_group: Option<wgpu::BindGroup>,
}

impl FxaaPass {
    pub fn new(build: PassBuild, format: wgpu::TextureFormat) -> Self {
        let PassBuild {
            device, composer, ..
        } = build;
        let shader =
            composer.compile_wgsl(device, "fxaa.wgsl", include_str!("../../shaders/fxaa.wgsl"));

        let params = FxaaParams {
            texel: [1.0 / 1920.0, 1.0 / 1080.0],
            relative_threshold: RELATIVE_THRESHOLD,
            absolute_threshold: ABSOLUTE_THRESHOLD,
            subpixel: SUBPIXEL,
            padding: [0.0; 3],
        };
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("FXAA Params Buffer"),
            contents: bytemuck::cast_slice(&[params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = build::bind_layout(
            device,
            "FXAA Bind Group Layout",
            &[
                build::painted_at(0),
                build::filtering_at(1),
                build::uniform_at(2, wgpu::ShaderStages::FRAGMENT),
            ],
        );

        let pipeline_layout =
            build::pipeline_layout(device, "FXAA Pipeline Layout", &[&bind_group_layout]);

        let pipeline = crate::renderer::wgpu::passes::fullscreen_pipeline(
            device,
            "FXAA Pipeline",
            &pipeline_layout,
            &shader,
            format,
        );

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("FXAA Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self {
            pipeline,
            bind_group_layout,
            sampler,
            params_buffer,
            bind_group: None,
        }
    }
}

const FXAA_PASS_DECL: PassDecl = PassDecl {
    name: "fxaa_pass",
    position: PassPosition::Present,
    reads: &["input"],
    writes: &["output"],
    ..PassDecl::EMPTY
};

impl PassNode<crate::renderer::wgpu::render_configs::RenderInputs> for FxaaPass {
    fn decl(&self) -> &PassDecl {
        &FXAA_PASS_DECL
    }

    fn enabled(&self, configs: &crate::renderer::wgpu::render_configs::RenderInputs) -> bool {
        configs.settings.anti_aliasing == nightshade_render::data::config::AntiAliasing::Fxaa
    }

    fn invalidate_bind_groups(&mut self) {
        self.bind_group = None;
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, crate::renderer::wgpu::render_configs::RenderInputs>,
    ) -> crate::renderer::wgpu::rendergraph::Result<()> {
        if !context.is_pass_enabled() {
            return Ok(());
        }

        let (width, height) = context
            .get_texture("input")
            .map(|texture| (texture.width(), texture.height()))
            .unwrap_or((1920, 1080));
        let params = FxaaParams {
            texel: [1.0 / width.max(1) as f32, 1.0 / height.max(1) as f32],
            relative_threshold: RELATIVE_THRESHOLD,
            absolute_threshold: ABSOLUTE_THRESHOLD,
            subpixel: SUBPIXEL,
            padding: [0.0; 3],
        };
        context
            .queue
            .write_buffer(&self.params_buffer, 0, bytemuck::cast_slice(&[params]));

        if self.bind_group.is_none() {
            let input_view = context.get_texture_view("input")?;
            self.bind_group = Some(build::binding(
                context.device,
                "FXAA Bind Group",
                &self.bind_group_layout,
                vec![
                    wgpu::BindingResource::TextureView(input_view),
                    wgpu::BindingResource::Sampler(&self.sampler),
                    self.params_buffer.as_entire_binding(),
                ],
            ));
        }

        let (output_view, output_load_op, output_store_op) =
            context.get_color_attachment("output")?;

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("FXAA Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: output_load_op,
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
        if let Some(bind_group) = self.bind_group.as_ref() {
            render_pass.set_bind_group(0, bind_group, &[]);
        }
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}
