use crate::renderer::wgpu::build;
use crate::renderer::wgpu::pass_build::PassBuild;
use crate::renderer::wgpu::rendergraph::{PassDecl, PassExecutionContext, PassNode, PassPosition};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TaaParams {
    inv_view_proj: [[f32; 4]; 4],
    prev_view_proj: [[f32; 4]; 4],
    resolution: [f32; 2],
    history_valid: f32,
    blend: f32,
    sharpness: f32,
    use_velocity: f32,
    input_resolution: [f32; 2],
}

struct CameraHistory {
    views: Vec<wgpu::TextureView>,
    index: usize,
    valid: bool,
    size: (u32, u32),
    bytes: u64,
    last_used: u64,
}

const HISTORY_BUDGET_BYTES: u64 = 128 * 1024 * 1024;

pub struct TaaPass {
    frame: u64,
    resolve_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    linear_sampler: wgpu::Sampler,
    point_sampler: wgpu::Sampler,
    params_buffer: wgpu::Buffer,
    format: wgpu::TextureFormat,
    histories: std::collections::HashMap<Option<nightshade::ecs::Entity>, CameraHistory>,
    dummy_velocity_view: wgpu::TextureView,
}

impl TaaPass {
    pub fn new(build: PassBuild, format: wgpu::TextureFormat) -> Self {
        let PassBuild {
            device,
            queue,
            composer,
            ..
        } = build;
        let shader =
            composer.compile_wgsl(device, "taa.wgsl", include_str!("../../shaders/taa.wgsl"));

        let params = TaaParams {
            inv_view_proj: nalgebra_glm::Mat4::identity().into(),
            prev_view_proj: nalgebra_glm::Mat4::identity().into(),
            resolution: [1920.0, 1080.0],
            history_valid: 0.0,
            blend: 0.12,
            sharpness: 0.5,
            use_velocity: 0.0,
            input_resolution: [1920.0, 1080.0],
        };
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("TAA Params Buffer"),
            contents: bytemuck::cast_slice(&[params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = build::bind_layout(
            device,
            "TAA Bind Group Layout",
            &[
                build::painted_at(0),
                build::painted_at(1),
                build::depth_at(2, wgpu::ShaderStages::FRAGMENT),
                build::filtering_at(3),
                build::sampler_at(
                    4,
                    wgpu::SamplerBindingType::NonFiltering,
                    wgpu::ShaderStages::FRAGMENT,
                ),
                build::uniform_at(5, wgpu::ShaderStages::FRAGMENT),
                build::painted_at(6),
            ],
        );

        let pipeline_layout =
            build::pipeline_layout(device, "TAA Pipeline Layout", &[&bind_group_layout]);

        let target = Some(wgpu::ColorTargetState {
            format,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        });
        let resolve_pipeline = build::Pipe::new(device, "TAA Resolve Pipeline", &shader)
            .layout(&pipeline_layout)
            .targets(&[target.clone(), target])
            .make();

        let linear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("TAA Linear Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let point_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("TAA Point Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let dummy_velocity = crate::renderer::wgpu::placeholder::placeholder_texture(
            device,
            queue,
            "TAA Dummy Velocity",
            wgpu::TextureFormat::Rg16Float,
            1,
            &crate::renderer::wgpu::placeholder::STILL_RG16,
        );
        let dummy_velocity_view =
            dummy_velocity.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            frame: 0,
            resolve_pipeline,
            bind_group_layout,
            linear_sampler,
            point_sampler,
            params_buffer,
            format,
            histories: std::collections::HashMap::new(),
            dummy_velocity_view,
        }
    }

    fn ensure_history(
        &mut self,
        device: &wgpu::Device,
        camera: Option<nightshade::ecs::Entity>,
        width: u32,
        height: u32,
    ) {
        if self
            .histories
            .get(&camera)
            .is_some_and(|history| history.size == (width, height))
        {
            return;
        }

        let mut textures = Vec::new();
        let mut views = Vec::new();
        for _ in 0..2 {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("TAA History Texture"),
                size: crate::renderer::texture_extent::texture_extent(
                    device,
                    width.max(1),
                    height.max(1),
                    1,
                ),
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            textures.push(texture);
            views.push(view);
        }

        let bytes = u64::from(width.max(1))
            * u64::from(height.max(1))
            * u64::from(self.format.block_copy_size(None).unwrap_or(8))
            * 2;
        self.histories.insert(
            camera,
            CameraHistory {
                views,
                index: 0,
                valid: false,
                size: (width, height),
                bytes,
                last_used: self.frame,
            },
        );
        self.evict_cold_histories();
    }

    fn evict_cold_histories(&mut self) {
        let mut held: u64 = self.histories.values().map(|history| history.bytes).sum();
        while held > HISTORY_BUDGET_BYTES {
            let Some(coldest) = self
                .histories
                .iter()
                .filter(|(_, history)| history.last_used != self.frame)
                .min_by_key(|(_, history)| history.last_used)
                .map(|(camera, _)| *camera)
            else {
                return;
            };
            if let Some(dropped) = self.histories.remove(&coldest) {
                held -= dropped.bytes;
            }
        }
    }

    pub fn retain_cameras(&mut self, live: &std::collections::HashSet<nightshade::ecs::Entity>) {
        self.histories
            .retain(|camera, _| camera.is_none_or(|entity| live.contains(&entity)));
    }
}

const TAA_PASS_DECL: PassDecl = PassDecl {
    name: "taa_pass",
    position: PassPosition::Present,
    reads: &["input", "depth"],
    writes: &["output"],
    optional_reads: &["velocity"],
    ..PassDecl::EMPTY
};

impl PassNode<crate::renderer::wgpu::render_configs::RenderInputs> for TaaPass {
    fn decl(&self) -> &PassDecl {
        &TAA_PASS_DECL
    }

    fn enabled(&self, configs: &crate::renderer::wgpu::render_configs::RenderInputs) -> bool {
        configs.settings.anti_aliasing == nightshade_render::data::config::AntiAliasing::Taa
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, crate::renderer::wgpu::render_configs::RenderInputs>,
    ) -> crate::renderer::wgpu::rendergraph::Result<()> {
        let (width, height) = context
            .get_texture("output")
            .map(|texture| (texture.width(), texture.height()))
            .unwrap_or((1920, 1080));

        let camera = context.configs.view.active_camera;
        if !context.configs.settings.anti_aliasing.is_temporal() {
            return Ok(());
        }
        self.frame = self.frame.wrapping_add(1);
        self.ensure_history(context.device, camera, width, height);
        if let Some(history) = self.histories.get_mut(&camera) {
            history.last_used = self.frame;
        }

        let view_projection = nalgebra_glm::Mat4::from(context.configs.scene.view_projection);
        let inv_view_proj = view_projection
            .try_inverse()
            .unwrap_or_else(nalgebra_glm::Mat4::identity);
        let prev_view_proj = context.configs.scene.prev_view_projection;

        let taa_enabled = context.configs.settings.anti_aliasing.is_temporal();
        let taa_blend = context.configs.settings.taa_blend;
        let taa_sharpness = context.configs.settings.taa_sharpness;
        let velocity_slot = context.get_texture_view("velocity").ok();
        let has_velocity = velocity_slot.is_some();
        let velocity_view = velocity_slot.unwrap_or(&self.dummy_velocity_view);
        let (read_index, write_index, history_valid) = self
            .histories
            .get(&camera)
            .map(|history| (history.index, 1 - history.index, history.valid))
            .unwrap_or((0, 1, false));
        let valid = taa_enabled && history_valid;
        let (input_width, input_height) = context
            .get_texture("input")
            .map(|texture| (texture.width(), texture.height()))
            .unwrap_or((width, height));
        let params = TaaParams {
            inv_view_proj: inv_view_proj.into(),
            prev_view_proj,
            resolution: [width.max(1) as f32, height.max(1) as f32],
            history_valid: if valid { 1.0 } else { 0.0 },
            blend: taa_blend,
            sharpness: taa_sharpness,
            use_velocity: if has_velocity && valid { 1.0 } else { 0.0 },
            input_resolution: [input_width.max(1) as f32, input_height.max(1) as f32],
        };
        context
            .queue
            .write_buffer(&self.params_buffer, 0, bytemuck::cast_slice(&[params]));

        let input_view = context.get_texture_view("input")?;
        let (depth_view, _, _) = context.get_depth_attachment("depth")?;

        let bind_group = build::binding(
            context.device,
            "TAA Bind Group",
            &self.bind_group_layout,
            vec![
                wgpu::BindingResource::TextureView(input_view),
                wgpu::BindingResource::TextureView(&self.histories[&camera].views[read_index]),
                wgpu::BindingResource::TextureView(depth_view),
                wgpu::BindingResource::Sampler(&self.linear_sampler),
                wgpu::BindingResource::Sampler(&self.point_sampler),
                self.params_buffer.as_entire_binding(),
                wgpu::BindingResource::TextureView(velocity_view),
            ],
        );

        let (output_view, output_load_op, output_store_op) =
            context.get_color_attachment("output")?;

        {
            let mut render_pass = context
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("TAA Resolve Pass"),
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view: output_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: output_load_op,
                                store: output_store_op,
                            },
                            depth_slice: None,
                        }),
                        Some(wgpu::RenderPassColorAttachment {
                            view: &self.histories[&camera].views[write_index],
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        }),
                    ],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });

            render_pass.set_pipeline(&self.resolve_pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        if let Some(history) = self.histories.get_mut(&camera) {
            history.index = write_index;
            history.valid = true;
        }

        Ok(())
    }
}
