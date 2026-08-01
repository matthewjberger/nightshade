use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::backend::{PassDecl, PassExecutionContext, PassNode, PassPosition};
use nightshade_wgpu::wgpu;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct SsrParams {
    projection: [[f32; 4]; 4],
    inv_projection: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    inv_view: [[f32; 4]; 4],
    screen_size: [f32; 2],
    max_steps: u32,
    thickness: f32,
    max_distance: f32,
    stride: f32,
    fade_start: f32,
    fade_end: f32,
    intensity: f32,
    enabled: f32,
    padding: [f32; 2],
}

pub struct SsrPass {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    params_buffer: wgpu::Buffer,
    point_sampler: wgpu::Sampler,
    linear_sampler: wgpu::Sampler,
    cached_bind_group: Option<wgpu::BindGroup>,
    cached_params: SsrParams,
}

impl SsrPass {
    pub fn new(build: PassBuild) -> Self {
        let PassBuild {
            device, composer, ..
        } = build;
        let shader = composer.compile_wgsl(device, "ssr.wgsl", include_str!("../shaders/ssr.wgsl"));

        let params = SsrParams {
            projection: [[0.0; 4]; 4],
            inv_projection: [[0.0; 4]; 4],
            view: [[0.0; 4]; 4],
            inv_view: [[0.0; 4]; 4],
            screen_size: [1920.0, 1080.0],
            max_steps: 64,
            thickness: 0.3,
            max_distance: 50.0,
            stride: 1.0,
            fade_start: 0.8,
            fade_end: 1.0,
            intensity: 1.0,
            enabled: 0.0,
            padding: [0.0; 2],
        };

        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("SSR Params Buffer"),
            contents: bytemuck::cast_slice(&[params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let point_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let linear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let bind_group_layout = build::bind_layout(
            device,
            "SSR Bind Group Layout",
            &[
                build::depth_at(0, wgpu::ShaderStages::FRAGMENT),
                build::painted_at(1),
                build::painted_at(2),
                build::sampler_at(
                    3,
                    wgpu::SamplerBindingType::NonFiltering,
                    wgpu::ShaderStages::FRAGMENT,
                ),
                build::filtering_at(4),
                build::uniform_at(5, wgpu::ShaderStages::FRAGMENT),
            ],
        );

        let pipeline_layout =
            build::pipeline_layout(device, "SSR Pipeline Layout", &[&bind_group_layout]);

        let pipeline = build::Pipe::new(device, "SSR Pipeline", &shader)
            .layout(&pipeline_layout)
            .target(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba16Float,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })
            .make();

        Self {
            pipeline,
            bind_group_layout,
            params_buffer,
            point_sampler,
            linear_sampler,
            cached_bind_group: None,
            cached_params: params,
        }
    }
}

const SSR_PASS_DECL: PassDecl = PassDecl {
    name: "ssr_pass",
    position: PassPosition::PostProcess,
    reads: &["depth", "view_normals", "scene_color"],
    writes: &["ssr_raw"],
    ..PassDecl::EMPTY
};

impl PassNode<nightshade_wgpu::backend::RenderInputs> for SsrPass {
    fn decl(&self) -> &PassDecl {
        &SSR_PASS_DECL
    }

    fn invalidate_bind_groups(&mut self) {
        self.cached_bind_group = None;
    }

    fn prepare(
        &mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &nightshade_wgpu::backend::RenderInputs,
    ) {
        let graphics = &configs.scene;
        let enabled = if graphics.active_view.ssr_enabled {
            1.0
        } else {
            0.0
        };

        if let Some(camera_matrices) = configs.scene.render_view.as_ref() {
            let projection: [[f32; 4]; 4] = camera_matrices.projection.into();
            let inv_projection: [[f32; 4]; 4] = camera_matrices
                .projection
                .try_inverse()
                .unwrap_or(nalgebra_glm::Mat4::identity())
                .into();
            let view: [[f32; 4]; 4] = camera_matrices.view.into();
            let inv_view: [[f32; 4]; 4] = camera_matrices
                .view
                .try_inverse()
                .unwrap_or(nalgebra_glm::Mat4::identity())
                .into();

            let (screen_width, screen_height) = configs.view.viewport_size.unwrap_or((1920, 1080));

            let active = &graphics.active_view;
            let params = SsrParams {
                projection,
                inv_projection,
                view,
                inv_view,
                screen_size: [
                    (screen_width / 2).max(1) as f32,
                    (screen_height / 2).max(1) as f32,
                ],
                max_steps: active.ssr_max_steps,
                thickness: active.ssr_thickness,
                max_distance: active.ssr_max_distance,
                stride: active.ssr_stride,
                fade_start: active.ssr_fade_start,
                fade_end: active.ssr_fade_end,
                intensity: active.ssr_intensity,
                enabled,
                padding: [0.0; 2],
            };

            if params != self.cached_params {
                queue.write_buffer(&self.params_buffer, 0, bytemuck::cast_slice(&[params]));
                self.cached_params = params;
            }
        }
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, nightshade_wgpu::backend::RenderInputs>,
    ) -> nightshade_wgpu::backend::Result<()> {
        if !context.is_pass_enabled() {
            return Ok(());
        }
        if !context.configs.scene.active_view.ssr_enabled {
            return Ok(());
        }

        if self.cached_bind_group.is_none() {
            let (depth_view, _, _) = context.get_depth_attachment("depth")?;
            let normal_view = context.get_texture_view("view_normals")?;
            let scene_color_view = context.get_texture_view("scene_color")?;

            self.cached_bind_group = Some(build::binding(
                context.device,
                "SSR Bind Group",
                &self.bind_group_layout,
                vec![
                    wgpu::BindingResource::TextureView(depth_view),
                    wgpu::BindingResource::TextureView(normal_view),
                    wgpu::BindingResource::TextureView(scene_color_view),
                    wgpu::BindingResource::Sampler(&self.point_sampler),
                    wgpu::BindingResource::Sampler(&self.linear_sampler),
                    self.params_buffer.as_entire_binding(),
                ],
            ));
        }

        let (output_view, output_load_op, output_store_op) =
            context.get_color_attachment("ssr_raw")?;

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("SSR Render Pass"),
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
        render_pass.set_bind_group(0, self.cached_bind_group.as_ref().unwrap(), &[]);
        render_pass.draw(0..3, 0..1);

        drop(render_pass);

        Ok(())
    }
}
