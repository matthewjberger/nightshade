use crate::renderer::config::DepthOfFieldQuality;
use crate::renderer::wgpu::build;
use crate::renderer::wgpu::pass_build::PassBuild;
use crate::renderer::wgpu::rendergraph::{PassDecl, PassExecutionContext, PassNode, PassPosition};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct DoFParams {
    focus_distance: f32,
    focus_range: f32,
    max_blur_radius: f32,
    bokeh_threshold: f32,
    bokeh_intensity: f32,
    near_plane: f32,
    far_plane: f32,
    sample_count: u32,
    texture_size: [f32; 2],
    tilt_shift_enabled: u32,
    tilt_shift_angle: f32,
    tilt_shift_center: f32,
    tilt_shift_blur_amount: f32,
    visualize_tilt_shift: u32,
    padding: u32,
}

pub struct DepthOfFieldPass {
    pipeline: wgpu::RenderPipeline,
    coc_visualization_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    params_buffer: wgpu::Buffer,
    bind_group: Option<wgpu::BindGroup>,
    width: u32,
    height: u32,
}

impl DepthOfFieldPass {
    pub fn new(build: PassBuild, format: wgpu::TextureFormat, width: u32, height: u32) -> Self {
        let PassBuild {
            device, composer, ..
        } = build;
        let shader = composer.compile_wgsl(
            device,
            "depth_of_field.wgsl",
            include_str!("../../shaders/depth_of_field.wgsl"),
        );

        let params = DoFParams {
            focus_distance: 10.0,
            focus_range: 5.0,
            max_blur_radius: 8.0,
            bokeh_threshold: 0.8,
            bokeh_intensity: 1.0,
            near_plane: 0.1,
            far_plane: 1000.0,
            sample_count: 16,
            texture_size: [width as f32, height as f32],
            tilt_shift_enabled: 0,
            tilt_shift_angle: 0.0,
            tilt_shift_center: 0.0,
            tilt_shift_blur_amount: 1.0,
            visualize_tilt_shift: 0,
            padding: 0,
        };

        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("DoF Params Buffer"),
            contents: bytemuck::cast_slice(&[params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = build::bind_layout(
            device,
            "DoF Bind Group Layout",
            &[
                build::painted_at(0),
                build::depth_at(1, wgpu::ShaderStages::FRAGMENT),
                build::filtering_at(2),
                build::uniform_at(3, wgpu::ShaderStages::FRAGMENT),
            ],
        );

        let pipeline_layout =
            build::pipeline_layout(device, "DoF Pipeline Layout", &[&bind_group_layout]);

        let pipeline = build::Pipe::new(device, "DoF Pipeline", &shader)
            .layout(&pipeline_layout)
            .target(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })
            .make();

        let coc_visualization_pipeline =
            build::Pipe::new(device, "DoF CoC Visualization Pipeline", &shader)
                .layout(&pipeline_layout)
                .target(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })
                .fragment("fragment_coc_visualization")
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
            coc_visualization_pipeline,
            bind_group_layout,
            sampler,
            params_buffer,
            bind_group: None,
            width,
            height,
        }
    }

    fn quality_to_sample_count(quality: DepthOfFieldQuality) -> u32 {
        quality.sample_count()
    }
}

const DEPTH_OF_FIELD_PASS_DECL: PassDecl = PassDecl {
    name: "depth_of_field_pass",
    position: PassPosition::PostProcess,
    reads: &["hdr", "depth"],
    writes: &["dof_output"],
    ..PassDecl::EMPTY
};

impl PassNode<crate::renderer::wgpu::render_configs::RenderInputs> for DepthOfFieldPass {
    fn decl(&self) -> &PassDecl {
        &DEPTH_OF_FIELD_PASS_DECL
    }

    fn enabled(&self, configs: &crate::renderer::wgpu::render_configs::RenderInputs) -> bool {
        configs.scene.active_view.depth_of_field.enabled
    }

    fn invalidate_bind_groups(&mut self) {
        self.bind_group = None;
    }

    fn prepare(
        &mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &crate::renderer::wgpu::render_configs::RenderInputs,
    ) {
        let dof = &configs.scene.active_view.depth_of_field;

        let (near, far) = configs
            .scene
            .render_view
            .as_ref()
            .map(|view| (view.z_near, view.z_far))
            .unwrap_or((0.1, 1000.0));

        let params = DoFParams {
            focus_distance: dof.focus_distance,
            focus_range: dof.focus_range,
            max_blur_radius: dof.max_blur_radius,
            bokeh_threshold: dof.bokeh_threshold,
            bokeh_intensity: dof.bokeh_intensity,
            near_plane: near,
            far_plane: far,
            sample_count: Self::quality_to_sample_count(dof.quality),
            texture_size: [self.width as f32, self.height as f32],
            tilt_shift_enabled: if dof.tilt_shift_enabled { 1 } else { 0 },
            tilt_shift_angle: dof.tilt_shift_angle.to_radians(),
            tilt_shift_center: dof.tilt_shift_center,
            tilt_shift_blur_amount: dof.tilt_shift_blur_amount,
            visualize_tilt_shift: if dof.visualize_tilt_shift { 1 } else { 0 },
            padding: 0,
        };

        queue.write_buffer(&self.params_buffer, 0, bytemuck::cast_slice(&[params]));
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, crate::renderer::wgpu::render_configs::RenderInputs>,
    ) -> crate::renderer::wgpu::rendergraph::Result<()> {
        if !context.is_pass_enabled() {
            return Ok(());
        }

        if self.bind_group.is_none() {
            let hdr_view = context.get_texture_view("hdr")?;
            let depth_view = context.get_texture_view("depth")?;

            self.bind_group = Some(build::binding(
                context.device,
                "DoF Bind Group",
                &self.bind_group_layout,
                vec![
                    wgpu::BindingResource::TextureView(hdr_view),
                    wgpu::BindingResource::TextureView(depth_view),
                    wgpu::BindingResource::Sampler(&self.sampler),
                    self.params_buffer.as_entire_binding(),
                ],
            ));
        }

        let (output_view, output_load_op, output_store_op) =
            context.get_color_attachment("dof_output")?;

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("DoF Pass"),
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

        let visualize_coc = context.configs.settings.depth_of_field.visualize_coc;
        let pipeline = if visualize_coc {
            &self.coc_visualization_pipeline
        } else {
            &self.pipeline
        };

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
        render_pass.draw(0..3, 0..1);
        drop(render_pass);

        Ok(())
    }
}
