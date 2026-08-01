use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;

pub struct GridPass {
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl GridPass {
    pub fn new(
        build: PassBuild,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        let PassBuild {
            device, composer, ..
        } = build;
        let uniform_buffer = build::uniform(
            device,
            "Grid Uniform Buffer",
            std::mem::size_of::<GridUniform>() as u64,
        );

        let bind_group_layout = build::bind_layout(
            device,
            "Grid Bind Group Layout",
            &[build::uniform_at(0, wgpu::ShaderStages::VERTEX_FRAGMENT)],
        );

        let bind_group = build::binding(
            device,
            "Grid Bind Group",
            &bind_group_layout,
            vec![uniform_buffer.as_entire_binding()],
        );

        let shader =
            composer.compile_wgsl(device, "grid.wgsl", include_str!("../shaders/grid.wgsl"));

        let pipeline_layout =
            build::pipeline_layout(device, "Grid Pipeline Layout", &[&bind_group_layout]);

        let pipeline = build::Pipe::new(device, "Grid Pipeline", &shader)
            .layout(&pipeline_layout)
            .target(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent::OVER,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })
            .depth(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: -2,
                    slope_scale: -2.0,
                    clamp: 0.0,
                },
            })
            .make();

        Self {
            uniform_buffer,
            bind_group,
            pipeline,
        }
    }

    pub fn record(
        &self,
        render_pass: &mut wgpu::RenderPass<'_>,
        configs: &nightshade_wgpu::backend::RenderInputs,
    ) {
        if !configs.debug_draw.show_grid {
            return;
        }

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}

impl nightshade_wgpu_geometry::backend::OverlaySubPass for GridPass {
    fn prepare(
        &mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &nightshade_wgpu::backend::RenderInputs,
    ) {
        if let Some(camera_matrices) = configs.scene.render_view.as_ref() {
            let (is_orthographic, orthographic_scale) =
                if let Some((x_mag, y_mag)) = camera_matrices.orthographic {
                    let avg_mag = (x_mag + y_mag) * 0.5;
                    let scale = if avg_mag > 100.0 {
                        avg_mag * 0.001
                    } else {
                        avg_mag * 0.1
                    };
                    (true, scale)
                } else {
                    (false, 1.0)
                };

            let uniform = GridUniform::new(
                camera_matrices.projection,
                camera_matrices.view,
                camera_matrices.camera_position,
                is_orthographic,
                orthographic_scale,
            );

            queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniform]));
        }
    }

    fn record(
        &mut self,
        render_pass: &mut wgpu::RenderPass<'_>,
        _device: &wgpu::Device,
        configs: &nightshade_wgpu::backend::RenderInputs,
    ) {
        GridPass::record(self, render_pass, configs);
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GridUniform {
    view_proj: [[f32; 4]; 4],
    camera_world_pos: [f32; 3],
    grid_size: f32,
    grid_min_pixels: f32,
    grid_cell_size: f32,
    orthographic_scale: f32,
    is_orthographic: f32,
}

impl GridUniform {
    pub fn new(
        projection: nalgebra_glm::Mat4,
        view: nalgebra_glm::Mat4,
        camera_position: nalgebra_glm::Vec3,
        is_orthographic: bool,
        orthographic_scale: f32,
    ) -> Self {
        let view_proj: [[f32; 4]; 4] = (projection * view).into();

        Self {
            view_proj,
            camera_world_pos: [camera_position.x, camera_position.y, camera_position.z],
            grid_size: 500.0,
            grid_min_pixels: 2.0,
            grid_cell_size: 1.0,
            orthographic_scale,
            is_orthographic: if is_orthographic { 1.0 } else { 0.0 },
        }
    }
}
