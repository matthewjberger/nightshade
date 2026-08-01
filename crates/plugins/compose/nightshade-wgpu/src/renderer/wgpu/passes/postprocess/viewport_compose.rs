use crate::renderer::wgpu::build;
use crate::renderer::wgpu::pass_build::PassBuild;
use crate::renderer::wgpu::rendergraph::{PassDecl, PassExecutionContext, PassNode, PassPosition};
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroup, BindGroupLayout, Buffer, Operations, RenderPassColorAttachment, RenderPipeline,
    Sampler,
};

const VERTEX_SHADER: &str = "
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct UvTransform {
    scale: vec2<f32>,
    offset: vec2<f32>,
}

@group(0) @binding(2)
var<uniform> uv_transform: UvTransform;

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((vertex_index & 1u) << 1u);
    let y = f32((vertex_index & 2u));
    out.position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    let raw_uv = vec2<f32>(x, 1.0 - y);
    out.uv = (raw_uv - vec2<f32>(0.5, 0.5)) * uv_transform.scale + vec2<f32>(0.5, 0.5) + uv_transform.offset;
    return out;
}
";

const FRAGMENT_SHADER: &str = "
@group(0) @binding(0)
var input_texture: texture_2d<f32>;

@group(0) @binding(1)
var input_sampler: sampler;

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(input_texture, input_sampler, in.uv);
}
";

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct UvTransform {
    scale: [f32; 2],
    offset: [f32; 2],
}

pub struct ViewportComposePass {
    pipeline: RenderPipeline,
    bind_group_layout: BindGroupLayout,
    sampler: Sampler,
    point_sampler: Sampler,
    uniform_buffer: Buffer,
    cached_bind_group: Option<BindGroup>,
    cached_lifting: bool,
}

impl ViewportComposePass {
    pub fn new(build: PassBuild, surface_format: wgpu::TextureFormat) -> Self {
        let PassBuild {
            device, composer, ..
        } = build;
        let shader_source = format!("{VERTEX_SHADER}\n{FRAGMENT_SHADER}");
        let shader_module =
            composer.compile_wgsl(device, "Viewport Compose Shader", &shader_source);

        let bind_group_layout = build::bind_layout(
            device,
            "Viewport Compose Bind Group Layout",
            &[
                build::painted_at(0),
                build::filtering_at(1),
                build::uniform_at(2, wgpu::ShaderStages::VERTEX),
            ],
        );

        let pipeline_layout = build::pipeline_layout(
            device,
            "Viewport Compose Pipeline Layout",
            &[&bind_group_layout],
        );

        let pipeline = crate::renderer::wgpu::passes::fullscreen_pipeline(
            device,
            "Viewport Compose Pipeline",
            &pipeline_layout,
            &shader_module,
            surface_format,
        );

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Viewport Compose Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let point_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Viewport Compose Point Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Viewport Compose Uniform"),
            contents: bytemuck::cast_slice(&[UvTransform {
                scale: [1.0, 1.0],
                offset: [0.0, 0.0],
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            pipeline,
            bind_group_layout,
            sampler,
            point_sampler,
            uniform_buffer,
            cached_bind_group: None,
            cached_lifting: false,
        }
    }
}

const VIEWPORT_COMPOSE_PASS_DECL: PassDecl = PassDecl {
    name: "viewport_compose_pass",
    position: PassPosition::Present,
    reads: &["input"],
    writes: &["output"],
    compose_only_phase: true,
    ..PassDecl::EMPTY
};

impl PassNode<crate::renderer::wgpu::render_configs::RenderInputs> for ViewportComposePass {
    fn decl(&self) -> &PassDecl {
        &VIEWPORT_COMPOSE_PASS_DECL
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

        let (output_width, output_height) = context.get_texture_size("output")?;
        let (input_width, input_height) = context.get_texture_size("input")?;
        let lifting = input_width < output_width || input_height < output_height;
        if self.cached_bind_group.is_none() || self.cached_lifting != lifting {
            let input_view = context.get_texture_view("input")?;
            let sampler = if lifting {
                &self.point_sampler
            } else {
                &self.sampler
            };
            self.cached_lifting = lifting;
            self.cached_bind_group = Some(build::binding(
                context.device,
                "Viewport Compose Bind Group",
                &self.bind_group_layout,
                vec![
                    wgpu::BindingResource::TextureView(input_view),
                    wgpu::BindingResource::Sampler(sampler),
                    self.uniform_buffer.as_entire_binding(),
                ],
            ));
        }

        let (output_view, _load_op, store_op) = context.get_color_attachment("output")?;

        let configs = context.configs;
        let active_camera = configs.view.active_camera;
        let render_iteration = configs.view.camera_tile_render_iteration;
        let viewport_rect = active_camera
            .and_then(|camera| configs.view.camera_tile_rects.get(&camera).copied())
            .or(configs.view.active_viewport_rect);

        let clear_color = configs
            .frame
            .background_clear_color
            .map(|color| wgpu::Color {
                r: color.x as f64,
                g: color.y as f64,
                b: color.z as f64,
                a: color.w as f64,
            })
            .unwrap_or(wgpu::Color {
                r: 0.04,
                g: 0.04,
                b: 0.06,
                a: 1.0,
            });

        let load_op = if render_iteration == 0 {
            wgpu::LoadOp::Clear(clear_color)
        } else {
            wgpu::LoadOp::Load
        };

        let constrained_aspect = configs
            .scene
            .render_view
            .as_ref()
            .and_then(|view| view.constrained_aspect);
        let inner_rect = viewport_rect.map(|rect| {
            if let Some(aspect) = constrained_aspect {
                let aspect = aspect.max(0.0001);
                let rect_aspect = rect.width.max(1.0) / rect.height.max(1.0);
                if rect_aspect > aspect {
                    let inner_width = rect.height * aspect;
                    let pad = (rect.width - inner_width) * 0.5;
                    crate::renderer::config::ViewportRect {
                        x: rect.x + pad,
                        y: rect.y,
                        width: inner_width,
                        height: rect.height,
                    }
                } else {
                    let inner_height = rect.width / aspect;
                    let pad = (rect.height - inner_height) * 0.5;
                    crate::renderer::config::ViewportRect {
                        x: rect.x,
                        y: rect.y + pad,
                        width: rect.width,
                        height: inner_height,
                    }
                }
            } else {
                rect
            }
        });

        let dest_width = inner_rect
            .map(|rect| rect.width)
            .unwrap_or(output_width as f32);
        let dest_height = inner_rect
            .map(|rect| rect.height)
            .unwrap_or(output_height as f32);
        let input_aspect = (input_width as f32) / (input_height.max(1) as f32);
        let dest_aspect = dest_width / dest_height.max(1.0);
        let (uv_scale_x, uv_scale_y) = if input_aspect > dest_aspect {
            (dest_aspect / input_aspect, 1.0)
        } else if input_aspect < dest_aspect {
            (1.0, input_aspect / dest_aspect)
        } else {
            (1.0, 1.0)
        };
        let uv_transform = UvTransform {
            scale: [uv_scale_x, uv_scale_y],
            offset: [0.0, 0.0],
        };
        context.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[uv_transform]),
        );

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Viewport Compose Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    ops: Operations {
                        load: load_op,
                        store: store_op,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

        if let Some(rect) = inner_rect {
            let max_x = (output_width as f32).max(1.0);
            let max_y = (output_height as f32).max(1.0);
            let x = rect.x.clamp(0.0, max_x);
            let y = rect.y.clamp(0.0, max_y);
            let width = rect.width.min(max_x - x).max(0.0);
            let height = rect.height.min(max_y - y).max(0.0);
            if width > 0.5 && height > 0.5 {
                render_pass.set_viewport(x, y, width, height, 0.0, 1.0);
            }
        }

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, self.cached_bind_group.as_ref().unwrap(), &[]);
        render_pass.draw(0..3, 0..1);

        drop(render_pass);

        Ok(())
    }
}
