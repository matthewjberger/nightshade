use nalgebra_glm::Mat4;
use nightshade::ecs::Entity;
use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::backend::RenderInputs;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::backend::{PassDecl, PassExecutionContext, PassNode, Result};
use nightshade_wgpu::wgpu;

pub(crate) const PORTAL_PASS: &str = "portal_pass";

const UNIFORM_SIZE: u64 = 96;
const COLOR: &str = "color";
const DEPTH: &str = "depth";

pub struct PortalDraw {
    pub camera: Entity,
    pub model: Mat4,
    pub rim: [f32; 4],
    pub view: wgpu::TextureView,
}

pub struct PortalPass {
    shader: wgpu::ShaderModule,
    layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: Option<wgpu::RenderPipeline>,
    target: Option<(wgpu::TextureFormat, u32)>,
    sampler: wgpu::Sampler,
    uniforms: Vec<wgpu::Buffer>,
    draws: Vec<PortalDraw>,
    elapsed: f32,
}

impl PortalPass {
    pub fn new(build: PassBuild) -> Self {
        let device = build.device;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(PORTAL_PASS),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/portal.wgsl").into()),
        });

        let layout = build::bind_layout(
            device,
            PORTAL_PASS,
            &[
                build::painted_at(0),
                build::filtering_at(1),
                build::uniform_at(2, wgpu::ShaderStages::VERTEX_FRAGMENT),
            ],
        );

        let pipeline_layout = build::pipeline_layout(device, PORTAL_PASS, &[&layout]);

        Self {
            shader,
            layout,
            pipeline_layout,
            pipeline: None,
            target: None,
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some(PORTAL_PASS),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }),
            uniforms: Vec::new(),
            draws: Vec::new(),
            elapsed: 0.0,
        }
    }

    pub fn set_portals(&mut self, elapsed: f32, draws: Vec<PortalDraw>) {
        self.elapsed = elapsed;
        self.draws = draws;
    }
}

fn uniform_buffer(device: &wgpu::Device, slot: usize) -> wgpu::Buffer {
    build::uniform(device, &format!("{PORTAL_PASS} {slot}"), UNIFORM_SIZE)
}

fn columns(matrix: &Mat4, rim: [f32; 4], elapsed: f32) -> [f32; 24] {
    let mut flat = [0.0_f32; 24];
    for column in 0..4 {
        for row in 0..4 {
            flat[column * 4 + row] = matrix[(row, column)];
        }
    }
    flat[16..20].copy_from_slice(&rim);
    flat[20] = elapsed;
    flat
}

fn view_projection(configs: &RenderInputs) -> Mat4 {
    let held = configs.scene.view_projection;
    let mut flat = [0.0_f32; 16];
    for column in 0..4 {
        for row in 0..4 {
            flat[column * 4 + row] = held[column][row];
        }
    }
    Mat4::from_column_slice(&flat)
}

fn build_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    target: (wgpu::TextureFormat, u32),
) -> wgpu::RenderPipeline {
    let (format, sample_count) = target;
    build::Pipe::new(device, PORTAL_PASS, shader)
        .layout(layout)
        .target(wgpu::ColorTargetState {
            format,
            blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })
        .depth(build::depth_state(wgpu::TextureFormat::Depth32Float, false))
        .samples(sample_count)
        .make()
}

fn draw(
    pass: &mut PortalPass,
    context: &mut PassExecutionContext<'_, '_, RenderInputs>,
) -> Result<()> {
    let Some(camera) = context.configs.view.active_camera else {
        return Ok(());
    };
    let showing: Vec<usize> = (0..pass.draws.len())
        .filter(|index| pass.draws[*index].camera == camera)
        .collect();
    if showing.is_empty() {
        return Ok(());
    }

    let color = context.get_texture(COLOR)?;
    let target = (color.format(), color.sample_count());
    if pass.target != Some(target) {
        pass.pipeline = Some(build_pipeline(
            context.device,
            &pass.pipeline_layout,
            &pass.shader,
            target,
        ));
        pass.target = Some(target);
    }
    let Some(pipeline) = pass.pipeline.as_ref() else {
        return Ok(());
    };

    while pass.uniforms.len() < showing.len() {
        let slot = pass.uniforms.len();
        pass.uniforms.push(uniform_buffer(context.device, slot));
    }

    let view_projection = view_projection(context.configs);
    let mut bound = Vec::with_capacity(showing.len());
    for (slot, index) in showing.iter().enumerate() {
        let portal = &pass.draws[*index];
        let flat = columns(&(view_projection * portal.model), portal.rim, pass.elapsed);
        context
            .queue
            .write_buffer(&pass.uniforms[slot], 0, bytemuck::cast_slice(&flat));
        bound.push(build::binding(
            context.device,
            PORTAL_PASS,
            &pass.layout,
            vec![
                wgpu::BindingResource::TextureView(&portal.view),
                wgpu::BindingResource::Sampler(&pass.sampler),
                pass.uniforms[slot].as_entire_binding(),
            ],
        ));
    }

    let (color_view, color_load, color_store) = context.get_color_attachment(COLOR)?;
    let (depth_view, depth_load, depth_store) = context.get_depth_attachment(DEPTH)?;
    let mut render_pass = context
        .encoder
        .begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(PORTAL_PASS),
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

    render_pass.set_pipeline(pipeline);
    for group in &bound {
        render_pass.set_bind_group(0, group, &[]);
        render_pass.draw(0..6, 0..1);
    }
    Ok(())
}

const PORTAL_PASS_DECL: PassDecl = PassDecl {
    name: PORTAL_PASS,
    reads_writes: &[COLOR, DEPTH],
    ..PassDecl::EMPTY
};

impl PassNode<RenderInputs> for PortalPass {
    fn decl(&self) -> &PassDecl {
        &PORTAL_PASS_DECL
    }

    fn execute<'r, 'e>(
        &mut self,
        mut context: PassExecutionContext<'r, 'e, RenderInputs>,
    ) -> Result<()> {
        draw(self, &mut context)
    }
}

pub fn install_portal_pass(
    graph: &mut nightshade_wgpu::backend::RenderGraph<nightshade_wgpu::backend::RenderInputs>,
    build: PassBuild,
    _: nightshade_wgpu::wgpu::TextureFormat,
) {
    if let Err(error) = nightshade_wgpu::backend::install_scene_pass(
        graph,
        Box::new(PortalPass::new(build)),
        &[
            ("color", build.targets.scene_color),
            ("depth", build.targets.depth),
        ],
    ) {
        panic!("the portal pass could not be installed: {error}");
    }
}

pub fn sync_portal_pass(
    mut renderer: nightshade_wgpu::pre_render::Renderer,
    frame: Option<nightshade::ecs::system_param::Res<nightshade_portal::contract::PortalFrame>>,
) {
    let Some(frame) = frame else {
        return;
    };
    let mut draws = Vec::new();
    for order in &frame.orders {
        let Some(viewport) = renderer.camera_viewports.get(&order.source) else {
            continue;
        };
        draws.push(PortalDraw {
            camera: order.camera,
            model: order.model,
            rim: order.rim,
            view: viewport.view.clone(),
        });
    }

    let Some(pass) =
        nightshade_wgpu::backend::render_graph_get_pass_mut(&mut renderer.graph, PORTAL_PASS)
    else {
        return;
    };
    let Some(portal_pass) = (pass as &mut dyn std::any::Any).downcast_mut::<PortalPass>() else {
        return;
    };
    portal_pass.set_portals(frame.elapsed, draws);
}
