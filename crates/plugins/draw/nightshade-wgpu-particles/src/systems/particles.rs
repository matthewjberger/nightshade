use nalgebra_glm::{Mat4, Vec3};
use nightshade::ecs::Entity;
use nightshade_particles::contract::ParticleTextureUpload;
use nightshade_particles::contract::data::{
    emitter_shape_code, emitter_type_code, pack_color_gradient, pack_value_curve,
};
use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::backend::RenderInputs;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;
use nightshade_wgpu_geometry::backend::OverlaySubPass;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

const MAX_PARTICLES: u32 = 1_000_000;
const MAX_EMITTERS: u32 = 512;
const WORKGROUP_SIZE: u32 = 256;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuParticle {
    position: [f32; 4],
    velocity: [f32; 4],
    color: [f32; 4],
    size_lifetime: [f32; 4],
    emitter_data: [f32; 4],
    physics: [f32; 4],
    color_start: [f32; 4],
    color_end: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuEmitterData {
    position: [f32; 4],
    direction: [f32; 4],
    velocity_range: [f32; 4],
    lifetime_range: [f32; 4],
    size_range: [f32; 4],
    gravity: [f32; 4],
    color_gradient: [[f32; 4]; 16],
    gradient_count: u32,
    spawn_count: u32,
    emitter_id: u32,
    shape_type: u32,
    shape_params: [f32; 4],
    turbulence: [f32; 4],
    emissive_strength: f32,
    drag: f32,
    emitter_type: u32,
    texture_index: u32,
    size_curve: [[f32; 4]; 8],
    size_curve_count: u32,
    size_curve_pad: [u32; 3],
    opacity_curve: [[f32; 4]; 8],
    opacity_curve_count: u32,
    opacity_curve_pad: [u32; 3],
}

impl Default for GpuEmitterData {
    fn default() -> Self {
        Self {
            position: [0.0; 4],
            direction: [0.0, 1.0, 0.0, 0.0],
            velocity_range: [1.0, 3.0, 0.3, 0.0],
            lifetime_range: [1.0, 2.0, 0.0, 0.0],
            size_range: [0.1, 0.0, 0.0, 0.0],
            gravity: [0.0, -9.81, 0.0, 0.0],
            color_gradient: [[0.0; 4]; 16],
            gradient_count: 0,
            spawn_count: 0,
            emitter_id: 0,
            shape_type: 0,
            shape_params: [0.0; 4],
            turbulence: [0.0; 4],
            emissive_strength: 1.0,
            drag: 0.1,
            emitter_type: 0,
            texture_index: 0,
            size_curve: [[0.0; 4]; 8],
            size_curve_count: 0,
            size_curve_pad: [0; 3],
            opacity_curve: [[0.0; 4]; 8],
            opacity_curve_count: 0,
            opacity_curve_pad: [0; 3],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct SimParams {
    delta_time: f32,
    time: f32,
    max_particles: u32,
    padding: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct DrawIndirect {
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniforms {
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
    view_projection: [[f32; 4]; 4],
    camera_position: [f32; 4],
    camera_right: [f32; 4],
    camera_up: [f32; 4],
}

const PARTICLE_TEXTURE_SIZE: u32 = 512;

fn layer_pixels(upload: &ParticleTextureUpload) -> Option<std::borrow::Cow<'_, [u8]>> {
    if upload.width == PARTICLE_TEXTURE_SIZE
        && upload.height == PARTICLE_TEXTURE_SIZE
        && upload.rgba_data.len() == (PARTICLE_TEXTURE_SIZE * PARTICLE_TEXTURE_SIZE * 4) as usize
    {
        return Some(std::borrow::Cow::Borrowed(&upload.rgba_data));
    }
    let source = image::RgbaImage::from_raw(upload.width, upload.height, upload.rgba_data.clone())?;
    let scaled = image::imageops::resize(
        &source,
        PARTICLE_TEXTURE_SIZE,
        PARTICLE_TEXTURE_SIZE,
        image::imageops::FilterType::Triangle,
    );
    Some(std::borrow::Cow::Owned(scaled.into_raw()))
}
const MAX_PARTICLE_TEXTURES: u32 = 64;

pub struct ParticlePass {
    emitter_buffer: wgpu::Buffer,
    params_buffer: wgpu::Buffer,
    draw_indirect_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    update_pipeline: wgpu::ComputePipeline,
    spawn_pipeline: wgpu::ComputePipeline,
    reset_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
    compute_bind_group: wgpu::BindGroup,
    render_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group: wgpu::BindGroup,
    texture_array: wgpu::Texture,
    texture_array_view: wgpu::TextureView,
    texture_sampler: wgpu::Sampler,
    max_particles: u32,
    emitter_data: Vec<GpuEmitterData>,
    time: f32,
    emitter_spawn_offsets: HashMap<Entity, f32>,
    loaded_texture_count: usize,
    emitters: Vec<(Entity, nightshade_particles::contract::ParticleEmitter)>,
    pending_textures: Vec<ParticleTextureUpload>,
    stepped_this_frame: bool,
    encoded_this_frame: bool,
}

fn particle_budget(limits: &wgpu::Limits) -> u32 {
    let binding = limits
        .max_storage_buffer_binding_size
        .min(limits.max_buffer_size);
    let stride = std::mem::size_of::<GpuParticle>() as u64;
    let room = (binding / stride).min(u32::MAX as u64) as u32;
    MAX_PARTICLES.min(room)
}

impl ParticlePass {
    pub fn new(build: PassBuild, color_format: wgpu::TextureFormat) -> Self {
        let PassBuild {
            device, composer, ..
        } = build;
        let max_particles = particle_budget(&device.limits());

        let particle_buffer = build::storage(
            device,
            "Particle Buffer",
            (std::mem::size_of::<GpuParticle>() * max_particles as usize) as u64,
        );

        let emitter_buffer = build::storage(
            device,
            "Emitter Buffer",
            (std::mem::size_of::<GpuEmitterData>() * MAX_EMITTERS as usize) as u64,
        );

        let params_buffer = build::uniform(
            device,
            "Particle Sim Params Buffer",
            std::mem::size_of::<SimParams>() as u64,
        );

        let mut free_indices: Vec<u32> = (0..max_particles).collect();
        free_indices.reverse();

        let free_indices_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Free Indices Buffer"),
            contents: bytemuck::cast_slice(&free_indices),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let free_count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Free Count Buffer"),
            contents: bytemuck::cast_slice(&[max_particles]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let alive_indices_buffer = build::storage(
            device,
            "Alive Indices Buffer",
            (std::mem::size_of::<u32>() * max_particles as usize) as u64,
        );

        let alive_count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Alive Count Buffer"),
            contents: bytemuck::cast_slice(&[0u32]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let draw_indirect_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Draw Indirect Buffer"),
            contents: bytemuck::cast_slice(&[DrawIndirect {
                vertex_count: 6,
                instance_count: 0,
                first_vertex: 0,
                first_instance: 0,
            }]),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::COPY_DST,
        });

        let camera_buffer = build::uniform(
            device,
            "Particle Camera Buffer",
            std::mem::size_of::<CameraUniforms>() as u64,
        );

        let compute_bind_group_layout = build::bind_layout(
            device,
            "Particle Compute Bind Group Layout",
            &[
                build::storage_at(0, true, wgpu::ShaderStages::COMPUTE),
                build::storage_at(1, false, wgpu::ShaderStages::COMPUTE),
                build::uniform_at(2, wgpu::ShaderStages::COMPUTE),
                build::storage_at(3, true, wgpu::ShaderStages::COMPUTE),
                build::storage_at(4, true, wgpu::ShaderStages::COMPUTE),
                build::storage_at(5, true, wgpu::ShaderStages::COMPUTE),
                build::storage_at(6, true, wgpu::ShaderStages::COMPUTE),
                build::storage_at(7, true, wgpu::ShaderStages::COMPUTE),
            ],
        );

        let compute_bind_group = build::binding(
            device,
            "Particle Compute Bind Group",
            &compute_bind_group_layout,
            vec![
                particle_buffer.as_entire_binding(),
                emitter_buffer.as_entire_binding(),
                params_buffer.as_entire_binding(),
                free_indices_buffer.as_entire_binding(),
                free_count_buffer.as_entire_binding(),
                alive_indices_buffer.as_entire_binding(),
                alive_count_buffer.as_entire_binding(),
                draw_indirect_buffer.as_entire_binding(),
            ],
        );

        let render_bind_group_layout = build::bind_layout(
            device,
            "Particle Render Bind Group Layout",
            &[
                build::uniform_at(0, wgpu::ShaderStages::VERTEX),
                build::storage_at(1, false, wgpu::ShaderStages::VERTEX),
                build::storage_at(2, false, wgpu::ShaderStages::VERTEX),
            ],
        );

        let render_bind_group = build::binding(
            device,
            "Particle Render Bind Group",
            &render_bind_group_layout,
            vec![
                camera_buffer.as_entire_binding(),
                particle_buffer.as_entire_binding(),
                alive_indices_buffer.as_entire_binding(),
            ],
        );

        let texture_bind_group_layout = build::bind_layout(
            device,
            "Particle Texture Bind Group Layout",
            &[
                build::texture_at(
                    0,
                    wgpu::TextureSampleType::Float { filterable: true },
                    wgpu::TextureViewDimension::D2Array,
                    wgpu::ShaderStages::FRAGMENT,
                ),
                build::filtering_at(1),
            ],
        );

        let texture_array = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Particle Texture Array"),
            size: nightshade_wgpu::backend::texture_extent(
                device,
                PARTICLE_TEXTURE_SIZE,
                PARTICLE_TEXTURE_SIZE,
                MAX_PARTICLE_TEXTURES,
            ),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_array_view = texture_array.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let texture_bind_group = build::binding(
            device,
            "Particle Texture Bind Group",
            &texture_bind_group_layout,
            vec![
                wgpu::BindingResource::TextureView(&texture_array_view),
                wgpu::BindingResource::Sampler(&texture_sampler),
            ],
        );

        let compute_shader = composer.compile_wgsl(
            device,
            "Particle Compute Shader",
            include_str!("../shaders/particle_update.wgsl"),
        );

        let render_shader = composer.compile_wgsl(
            device,
            "Particle Render Shader",
            include_str!("../shaders/particle_render.wgsl"),
        );

        let compute_pipeline_layout = build::pipeline_layout(
            device,
            "Particle Compute Pipeline Layout",
            &[&compute_bind_group_layout],
        );

        let update_pipeline = build::compute(
            device,
            "Particle Update Pipeline",
            &compute_shader,
            "update",
            &compute_pipeline_layout,
        );

        let spawn_pipeline = build::compute(
            device,
            "Particle Spawn Pipeline",
            &compute_shader,
            "spawn",
            &compute_pipeline_layout,
        );

        let reset_pipeline = build::compute(
            device,
            "Particle Reset Pipeline",
            &compute_shader,
            "reset_counters",
            &compute_pipeline_layout,
        );

        let render_pipeline_layout = build::pipeline_layout(
            device,
            "Particle Render Pipeline Layout",
            &[&render_bind_group_layout, &texture_bind_group_layout],
        );

        let render_pipeline = build::Pipe::new(device, "Particle Render Pipeline", &render_shader)
            .layout(&render_pipeline_layout)
            .target(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })
            .depth(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            })
            .vertex("vs_main")
            .fragment("fs_main")
            .make();

        Self {
            emitter_buffer,
            params_buffer,
            draw_indirect_buffer,
            camera_buffer,
            update_pipeline,
            spawn_pipeline,
            reset_pipeline,
            render_pipeline,
            compute_bind_group,
            render_bind_group,
            texture_bind_group_layout,
            texture_bind_group,
            texture_array,
            texture_array_view,
            texture_sampler,
            max_particles,
            emitter_data: Vec::new(),
            time: 0.0,
            emitter_spawn_offsets: HashMap::new(),
            loaded_texture_count: 0,
            emitters: Vec::new(),
            pending_textures: Vec::new(),
            stepped_this_frame: false,
            encoded_this_frame: false,
        }
    }

    pub fn encode_simulation(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if self.encoded_this_frame {
            return;
        }
        self.encoded_this_frame = true;

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Particle Reset Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.reset_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            compute_pass.dispatch_workgroups(1, 1, 1);
        }

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Particle Update Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.update_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            let workgroups = self.max_particles.div_ceil(WORKGROUP_SIZE);
            compute_pass.dispatch_workgroups(workgroups, 1, 1);
        }

        if !self.emitter_data.is_empty() {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Particle Spawn Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.spawn_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            compute_pass.dispatch_workgroups(self.emitter_data.len() as u32, 1, 1);
        }
    }

    pub fn record_billboards(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.render_bind_group, &[]);
        render_pass.set_bind_group(1, &self.texture_bind_group, &[]);
        render_pass.draw_indirect(&self.draw_indirect_buffer, 0);
    }

    pub fn set_emitters(
        &mut self,
        emitters: Vec<(Entity, nightshade_particles::contract::ParticleEmitter)>,
    ) {
        self.emitters = emitters;
        self.stepped_this_frame = false;
        self.encoded_this_frame = false;
    }

    pub fn queue_textures(&mut self, uploads: Vec<ParticleTextureUpload>) {
        self.pending_textures.extend(uploads);
    }

    fn update_emitters(&mut self) {
        self.emitter_data.clear();

        let sources = std::mem::take(&mut self.emitters);

        for (emitter_index, (entity, emitter)) in sources.iter().enumerate() {
            if self.emitter_data.len() >= MAX_EMITTERS as usize {
                break;
            }

            {
                if !emitter.enabled {
                    self.emitter_spawn_offsets
                        .insert(*entity, emitter.accumulated_spawn);
                    continue;
                }

                if emitter.one_shot && emitter.has_fired {
                    let offset = self
                        .emitter_spawn_offsets
                        .get(entity)
                        .copied()
                        .unwrap_or(0.0);
                    if offset >= emitter.accumulated_spawn {
                        continue;
                    }
                }

                let offset = self
                    .emitter_spawn_offsets
                    .get(entity)
                    .copied()
                    .unwrap_or(0.0);
                let spawn_count =
                    (((emitter.accumulated_spawn - offset).max(0.0)) as u32).min(WORKGROUP_SIZE);

                if spawn_count == 0 {
                    continue;
                }

                self.emitter_spawn_offsets
                    .insert(*entity, offset + spawn_count as f32);

                let (shape_type, shape_params) = emitter_shape_code(&emitter.shape);
                let emitter_type_id = emitter_type_code(emitter.emitter_type);
                let (color_gradient, gradient_count) = pack_color_gradient(&emitter.color_gradient);
                let (size_curve, size_curve_count) = pack_value_curve(&emitter.size_curve);
                let (opacity_curve, opacity_curve_count) = pack_value_curve(&emitter.opacity_curve);

                let gpu_emitter = GpuEmitterData {
                    position: [
                        emitter.position.x,
                        emitter.position.y,
                        emitter.position.z,
                        1.0,
                    ],
                    direction: [
                        emitter.direction.x,
                        emitter.direction.y,
                        emitter.direction.z,
                        0.0,
                    ],
                    velocity_range: [
                        emitter.initial_velocity_min,
                        emitter.initial_velocity_max,
                        emitter.velocity_spread,
                        0.0,
                    ],
                    lifetime_range: [
                        emitter.particle_lifetime_min,
                        emitter.particle_lifetime_max,
                        0.0,
                        0.0,
                    ],
                    size_range: [emitter.size_start, emitter.size_end, 0.0, 0.0],
                    gravity: [emitter.gravity.x, emitter.gravity.y, emitter.gravity.z, 0.0],
                    color_gradient,
                    gradient_count,
                    spawn_count,
                    emitter_id: emitter_index as u32,
                    shape_type,
                    shape_params,
                    turbulence: [
                        emitter.turbulence_strength,
                        emitter.turbulence_frequency,
                        0.0,
                        0.0,
                    ],
                    emissive_strength: emitter.emissive_strength,
                    drag: emitter.drag,
                    emitter_type: emitter_type_id,
                    texture_index: emitter.texture_index,
                    size_curve,
                    size_curve_count,
                    size_curve_pad: [0; 3],
                    opacity_curve,
                    opacity_curve_count,
                    opacity_curve_pad: [0; 3],
                };

                self.emitter_data.push(gpu_emitter);
            }
        }

        self.emitters = sources;
    }

    pub fn prepare_frame(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &RenderInputs,
    ) {
        let pending = &self.pending_textures;
        if pending.len() > self.loaded_texture_count {
            for upload in &pending[self.loaded_texture_count..] {
                if upload.slot == 0 || upload.slot >= MAX_PARTICLE_TEXTURES {
                    continue;
                }
                let layer = upload.slot - 1;
                let Some(pixels) = layer_pixels(upload) else {
                    tracing::error!(
                        "a particle texture of {}x{} carried {} bytes, which is not four for each pixel",
                        upload.width,
                        upload.height,
                        upload.rgba_data.len()
                    );
                    continue;
                };
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &self.texture_array,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: 0,
                            y: 0,
                            z: layer,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &pixels,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(PARTICLE_TEXTURE_SIZE * 4),
                        rows_per_image: Some(PARTICLE_TEXTURE_SIZE),
                    },
                    wgpu::Extent3d {
                        width: PARTICLE_TEXTURE_SIZE,
                        height: PARTICLE_TEXTURE_SIZE,
                        depth_or_array_layers: 1,
                    },
                );
            }
            self.loaded_texture_count = pending.len();
            self.texture_array_view = self
                .texture_array
                .create_view(&wgpu::TextureViewDescriptor::default());
            self.texture_bind_group = build::binding(
                device,
                "Particle Texture Bind Group",
                &self.texture_bind_group_layout,
                vec![
                    wgpu::BindingResource::TextureView(&self.texture_array_view),
                    wgpu::BindingResource::Sampler(&self.texture_sampler),
                ],
            );
        }

        if !self.stepped_this_frame {
            self.stepped_this_frame = true;

            let delta_time = configs.view.delta_time;
            self.time += delta_time;

            self.update_emitters();

            if !self.emitter_data.is_empty() {
                queue.write_buffer(
                    &self.emitter_buffer,
                    0,
                    bytemuck::cast_slice(&self.emitter_data),
                );
            }

            let params = SimParams {
                delta_time,
                time: self.time,
                max_particles: self.max_particles,
                padding: 0,
            };
            queue.write_buffer(&self.params_buffer, 0, bytemuck::cast_slice(&[params]));
        }

        if let Some(camera_matrices) = configs.scene.render_view.as_ref() {
            let view: Mat4 = camera_matrices.view;
            let projection: Mat4 = camera_matrices.projection;
            let view_projection = projection * view;

            let view_inverse = nalgebra_glm::inverse(&view);
            let camera_right = Vec3::new(
                view_inverse[(0, 0)],
                view_inverse[(1, 0)],
                view_inverse[(2, 0)],
            );
            let camera_up = Vec3::new(
                view_inverse[(0, 1)],
                view_inverse[(1, 1)],
                view_inverse[(2, 1)],
            );
            let camera_position = Vec3::new(
                view_inverse[(0, 3)],
                view_inverse[(1, 3)],
                view_inverse[(2, 3)],
            );

            let camera_uniforms = CameraUniforms {
                view: view.into(),
                projection: projection.into(),
                view_projection: view_projection.into(),
                camera_position: [camera_position.x, camera_position.y, camera_position.z, 1.0],
                camera_right: [camera_right.x, camera_right.y, camera_right.z, 0.0],
                camera_up: [camera_up.x, camera_up.y, camera_up.z, 0.0],
            };

            queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::cast_slice(&[camera_uniforms]),
            );
        }
    }
}

impl OverlaySubPass for ParticlePass {
    fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, configs: &RenderInputs) {
        self.prepare_frame(device, queue, configs);
    }

    fn encode_compute(&mut self, encoder: &mut wgpu::CommandEncoder) {
        self.encode_simulation(encoder);
    }

    fn record(
        &mut self,
        render_pass: &mut wgpu::RenderPass<'_>,
        _device: &wgpu::Device,
        _configs: &RenderInputs,
    ) {
        self.record_billboards(render_pass);
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub fn install_particle_pass(
    graph: &mut nightshade_wgpu::backend::RenderGraph<nightshade_wgpu::backend::RenderInputs>,
    build: PassBuild,
    _: nightshade_wgpu::wgpu::TextureFormat,
) {
    let pass = ParticlePass::new(build, nightshade_wgpu::backend::HDR_FORMAT);
    if !nightshade_wgpu_geometry::backend::add_overlay_sub_pass(graph, Box::new(pass)) {
        panic!("the particle pass found no scene overlay to draw in");
    }
}

pub fn sync_particle_pass(
    mut renderer: nightshade_wgpu::pre_render::Renderer,
    textures: Option<
        nightshade::ecs::system_param::ResMut<nightshade_particles::contract::ParticleTextures>,
    >,
    frame: Option<
        nightshade::ecs::system_param::Res<nightshade_particles::contract::ParticleFrame>,
    >,
) {
    let (Some(mut textures), Some(frame)) = (textures, frame) else {
        return;
    };
    let Some(pass) = nightshade_wgpu_geometry::backend::overlay_sub_pass_mut::<ParticlePass>(
        &mut renderer.graph,
    ) else {
        return;
    };
    pass.set_emitters(frame.emitters.clone());
    pass.queue_textures(std::mem::take(&mut textures.pending));
}
