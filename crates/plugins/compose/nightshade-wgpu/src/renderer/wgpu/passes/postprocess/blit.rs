use crate::renderer::wgpu::build;
use crate::renderer::wgpu::pass_build::PassBuild;
use crate::renderer::wgpu::rendergraph::{PassDecl, PassExecutionContext, PassNode, PassPosition};
use wgpu::{
    BindGroup, BindGroupLayout, Operations, RenderPassColorAttachment, RenderPipeline, Sampler,
};

const FULLSCREEN_VERTEX_SHADER: &str = "
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((vertex_index & 1u) << 1u);
    let y = f32((vertex_index & 2u));
    out.position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, 1.0 - y);
    return out;
}
";

const BLIT_FRAGMENT_SHADER: &str = "
@group(0) @binding(0)
var input_texture: texture_2d<f32>;

@group(0) @binding(1)
var input_sampler: sampler;

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(input_texture, input_sampler, in.uv);
}
";

pub struct BlitPass {
    enabled_when: Option<fn(&crate::renderer::wgpu::render_configs::RenderInputs) -> bool>,
    pub pipeline: RenderPipeline,
    bind_group_layout: BindGroupLayout,
    sampler: Sampler,
    cached_bind_group: Option<BindGroup>,
    decl: PassDecl,
}

impl BlitPass {
    pub fn when(
        mut self,
        predicate: fn(&crate::renderer::wgpu::render_configs::RenderInputs) -> bool,
    ) -> Self {
        self.enabled_when = Some(predicate);
        self
    }
    pub fn create_pipeline(
        build: PassBuild,
        surface_format: wgpu::TextureFormat,
    ) -> RenderPipeline {
        let PassBuild {
            device, composer, ..
        } = build;
        let shader_source = format!("{}\n{}", FULLSCREEN_VERTEX_SHADER, BLIT_FRAGMENT_SHADER);

        let shader_module = composer.compile_wgsl(device, "Blit Shader", &shader_source);

        let bind_group_layout = build::bind_layout(
            device,
            "Blit Bind Group Layout",
            &[build::painted_at(0), build::filtering_at(1)],
        );

        let pipeline_layout =
            build::pipeline_layout(device, "Blit Pipeline Layout", &[&bind_group_layout]);

        crate::renderer::wgpu::passes::fullscreen_pipeline(
            device,
            "Blit Pipeline",
            &pipeline_layout,
            &shader_module,
            surface_format,
        )
    }

    pub fn new(build: PassBuild, surface_format: wgpu::TextureFormat) -> Self {
        let PassBuild { device, .. } = build;
        let bind_group_layout = build::bind_layout(
            device,
            "Blit Bind Group Layout",
            &[build::painted_at(0), build::filtering_at(1)],
        );

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Blit Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self {
            enabled_when: None,
            pipeline: Self::create_pipeline(build, surface_format),
            bind_group_layout,
            sampler,
            cached_bind_group: None,
            decl: BLIT_PASS_DECL,
        }
    }

    pub fn with_name(mut self, name: &'static str) -> Self {
        self.decl.name = name;
        self
    }

    pub fn with_runs_in_full(mut self, value: bool) -> Self {
        self.decl.full_phase = value;
        self
    }

    pub fn with_runs_in_compose_only(mut self, value: bool) -> Self {
        self.decl.compose_only_phase = value;
        self
    }
}

const BLIT_PASS_DECL: PassDecl = PassDecl {
    name: "blit_pass",
    position: PassPosition::Present,
    reads: &["input"],
    writes: &["output"],
    ..PassDecl::EMPTY
};

impl PassNode<crate::renderer::wgpu::render_configs::RenderInputs> for BlitPass {
    fn decl(&self) -> &PassDecl {
        &self.decl
    }

    fn enabled(&self, configs: &crate::renderer::wgpu::render_configs::RenderInputs) -> bool {
        self.enabled_when.is_none_or(|predicate| predicate(configs))
    }

    fn invalidate_bind_groups(&mut self) {
        self.cached_bind_group = None;
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, crate::renderer::wgpu::render_configs::RenderInputs>,
    ) -> crate::renderer::wgpu::rendergraph::Result<()> {
        if !context.is_pass_enabled() {
            return Ok(());
        }

        if self.cached_bind_group.is_none() {
            let input_view = context.get_texture_view("input")?;

            self.cached_bind_group = Some(build::binding(
                context.device,
                "Blit Bind Group",
                &self.bind_group_layout,
                vec![
                    wgpu::BindingResource::TextureView(input_view),
                    wgpu::BindingResource::Sampler(&self.sampler),
                ],
            ));
        }

        let (color_view, color_load_op, color_store_op) = context.get_color_attachment("output")?;

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Blit Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: Operations {
                        load: color_load_op,
                        store: color_store_op,
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
