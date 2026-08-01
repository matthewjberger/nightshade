use nalgebra_glm;
use nalgebra_glm::Mat4;
use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::backend::RenderInputs;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::backend::{PassDecl, PassExecutionContext, PassNode};
use nightshade_wgpu::wgpu;
use wgpu::util::DeviceExt;

use nightshade_decals::contract::render_data::DecalInstance;

pub(crate) const DECAL_PASS: &str = "decal_pass";

const MAX_DECALS: usize = 1024;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniforms {
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
    view_projection: [[f32; 4]; 4],
    inverse_view_projection: [[f32; 4]; 4],
    camera_position: [f32; 4],
    screen_size: [f32; 2],
    near_plane: f32,
    far_plane: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuDecalData {
    model: [[f32; 4]; 4],
    inverse_model: [[f32; 4]; 4],
    color: [f32; 4],
    emissive: [f32; 4],
    size_depth: [f32; 4],
    params: [f32; 4],
}

impl Default for GpuDecalData {
    fn default() -> Self {
        Self {
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            inverse_model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            color: [1.0, 1.0, 1.0, 1.0],
            emissive: [1.0, 1.0, 1.0, 0.0],
            size_depth: [1.0, 1.0, 1.0, 0.0],
            params: [0.0, 50.0, 100.0, 0.5],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct DecalVertex {
    position: [f32; 3],
}

struct DecalBatch {
    texture_name: Option<String>,
    start_index: u32,
    count: u32,
}

pub struct DecalPass {
    pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    decal_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    dummy_texture: wgpu::Texture,
    dummy_texture_view: wgpu::TextureView,
    dummy_sampler: wgpu::Sampler,
    dummy_texture_initialized: bool,
    decal_data: Vec<GpuDecalData>,
    decal_count: u32,
    registered_textures: std::collections::HashMap<String, (wgpu::TextureView, wgpu::Sampler)>,
    batches: Vec<DecalBatch>,
    decals: Vec<DecalInstance>,
}

impl DecalPass {
    pub fn new(build: PassBuild, color_format: wgpu::TextureFormat) -> Self {
        let PassBuild {
            device, composer, ..
        } = build;
        let bind_group_layout = build::bind_layout(
            device,
            "Decal Bind Group Layout",
            &[
                build::uniform_at(0, wgpu::ShaderStages::VERTEX_FRAGMENT),
                build::storage_at(1, false, wgpu::ShaderStages::VERTEX_FRAGMENT),
                build::depth_at(2, wgpu::ShaderStages::FRAGMENT),
                build::painted_at(3),
                build::filtering_at(4),
                build::painted_at(5),
                build::filtering_at(6),
            ],
        );

        let pipeline_layout =
            build::pipeline_layout(device, "Decal Pipeline Layout", &[&bind_group_layout]);

        let shader = composer.compile_wgsl(
            device,
            "Decal Shader",
            include_str!("../shaders/decal.wgsl"),
        );

        let pipeline = build::Pipe::new(device, "Decal Render Pipeline", &shader)
            .layout(&pipeline_layout)
            .buffers(&[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<DecalVertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                }],
            }])
            .target(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })
            .cull_front()
            .vertex("vs_main")
            .fragment("fs_main")
            .make();

        let camera_buffer = build::uniform(
            device,
            "Decal Camera Buffer",
            std::mem::size_of::<CameraUniforms>() as u64,
        );

        let decal_buffer = build::storage(
            device,
            "Decal Data Buffer",
            (std::mem::size_of::<GpuDecalData>() * MAX_DECALS) as u64,
        );

        let vertices = create_unit_cube_vertices();
        let indices = create_unit_cube_indices();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Decal Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Decal Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let dummy_texture = build::surface(
            device,
            "Decal Dummy Texture",
            wgpu::TextureFormat::Rgba8UnormSrgb,
            [1, 1, 1],
            1,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );

        let dummy_texture_view = dummy_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let dummy_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        Self {
            pipeline,
            camera_buffer,
            decal_buffer,
            vertex_buffer,
            index_buffer,
            bind_group_layout,
            dummy_texture,
            dummy_texture_view,
            dummy_sampler,
            dummy_texture_initialized: false,
            decal_data: Vec::with_capacity(MAX_DECALS),
            decal_count: 0,
            registered_textures: std::collections::HashMap::new(),
            batches: Vec::new(),
            decals: Vec::new(),
        }
    }

    pub fn sync_textures(&mut self, texture_store: &nightshade_wgpu::backend::TextureStore) {
        self.registered_textures.retain(|name, _| {
            nightshade_wgpu::backend::texture_store_contains(texture_store, name)
        });
        for name in nightshade_wgpu::backend::texture_store_names(texture_store) {
            if self.registered_textures.contains_key(name) {
                continue;
            }
            if let Some(entry) = nightshade_wgpu::backend::texture_store_get(texture_store, name) {
                self.registered_textures
                    .insert(name.clone(), (entry.view.clone(), entry.sampler.clone()));
            }
        }
    }

    pub fn set_decals(&mut self, decals: Vec<DecalInstance>) {
        self.decals = decals;
    }

    fn update_decals(&mut self) {
        self.decal_data.clear();
        self.batches.clear();

        let mut decals_with_textures: Vec<(GpuDecalData, Option<String>)> = Vec::new();

        for decal in &self.decals {
            if decals_with_textures.len() >= MAX_DECALS {
                break;
            }

            if !decal.visible {
                continue;
            }

            let model_matrix = decal.transform;
            let inverse_model = nalgebra_glm::inverse(&model_matrix);

            let has_emissive_texture = decal.emissive_texture.is_some();

            let model_arr: [[f32; 4]; 4] = model_matrix.into();
            let inverse_arr: [[f32; 4]; 4] = inverse_model.into();

            let gpu_decal = GpuDecalData {
                model: model_arr,
                inverse_model: inverse_arr,
                color: decal.color,
                emissive: [1.0, 1.0, 1.0, decal.emissive_strength],
                size_depth: [decal.size.x, decal.size.y, decal.depth, 0.0],
                params: [
                    if has_emissive_texture { 1.0 } else { 0.0 },
                    decal.fade_start,
                    decal.fade_end,
                    decal.normal_threshold,
                ],
            };

            decals_with_textures.push((gpu_decal, decal.texture.clone()));
        }

        decals_with_textures.sort_by(|a, b| a.1.cmp(&b.1));

        let mut current_texture: Option<String> = None;
        let mut batch_start: u32 = 0;
        let mut batch_count: u32 = 0;

        for (index, (gpu_decal, texture_name)) in decals_with_textures.into_iter().enumerate() {
            if texture_name != current_texture {
                if batch_count > 0 {
                    self.batches.push(DecalBatch {
                        texture_name: current_texture.clone(),
                        start_index: batch_start,
                        count: batch_count,
                    });
                }
                current_texture = texture_name;
                batch_start = index as u32;
                batch_count = 0;
            }
            self.decal_data.push(gpu_decal);
            batch_count += 1;
        }

        if batch_count > 0 {
            self.batches.push(DecalBatch {
                texture_name: current_texture,
                start_index: batch_start,
                count: batch_count,
            });
        }

        self.decal_count = self.decal_data.len() as u32;
    }
}

fn create_unit_cube_vertices() -> Vec<DecalVertex> {
    let h = 0.5;
    vec![
        DecalVertex {
            position: [-h, -h, -h],
        },
        DecalVertex {
            position: [h, -h, -h],
        },
        DecalVertex {
            position: [h, h, -h],
        },
        DecalVertex {
            position: [-h, h, -h],
        },
        DecalVertex {
            position: [-h, -h, h],
        },
        DecalVertex {
            position: [h, -h, h],
        },
        DecalVertex {
            position: [h, h, h],
        },
        DecalVertex {
            position: [-h, h, h],
        },
    ]
}

fn create_unit_cube_indices() -> Vec<u16> {
    vec![
        0, 1, 2, 0, 2, 3, 1, 5, 6, 1, 6, 2, 5, 4, 7, 5, 7, 6, 4, 0, 3, 4, 3, 7, 3, 2, 6, 3, 6, 7,
        4, 5, 1, 4, 1, 0,
    ]
}

const DECAL_PASS_DECL: PassDecl = PassDecl {
    name: DECAL_PASS,
    reads: &["depth"],
    reads_writes: &["color"],
    ..PassDecl::EMPTY
};

impl PassNode<RenderInputs> for DecalPass {
    fn decl(&self) -> &PassDecl {
        &DECAL_PASS_DECL
    }

    fn prepare(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, configs: &RenderInputs) {
        if !self.dummy_texture_initialized {
            let white_pixel: [u8; 4] = [255, 255, 255, 255];
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.dummy_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &white_pixel,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4),
                    rows_per_image: Some(1),
                },
                wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
            );
            self.dummy_texture_initialized = true;
        }

        self.sync_textures(&configs.owned.texture_store);
        self.update_decals();

        if self.decal_count == 0 {
            return;
        }

        queue.write_buffer(
            &self.decal_buffer,
            0,
            bytemuck::cast_slice(&self.decal_data),
        );

        if let Some(camera_matrices) = configs.scene.render_view.as_ref() {
            let view: Mat4 = camera_matrices.view;
            let projection: Mat4 = camera_matrices.projection;
            let view_projection = camera_matrices.view_projection;
            let inverse_view_projection = camera_matrices.inverse_view_projection;

            let camera_position = camera_matrices.camera_position;

            let (screen_width, screen_height) =
                if let Some((width, height)) = configs.view.viewport_size {
                    (width as f32, height as f32)
                } else {
                    (1920.0, 1080.0)
                };

            let (near_plane, far_plane) = (camera_matrices.z_near, camera_matrices.z_far);

            let camera_uniforms = CameraUniforms {
                view: view.into(),
                projection: projection.into(),
                view_projection: view_projection.into(),
                inverse_view_projection: inverse_view_projection.into(),
                camera_position: [camera_position.x, camera_position.y, camera_position.z, 1.0],
                screen_size: [screen_width, screen_height],
                near_plane,
                far_plane,
            };

            queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::cast_slice(&[camera_uniforms]),
            );
        }
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, RenderInputs>,
    ) -> nightshade_wgpu::backend::Result<()> {
        if self.decal_count == 0 || self.batches.is_empty() {
            return Ok(());
        }

        let depth_view = context.get_texture_view("depth")?;
        let (color_view, _, color_store) = context.get_color_attachment("color")?;

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Decal Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: color_store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        for batch in &self.batches {
            let (texture_view, sampler) = if let Some(texture_name) = &batch.texture_name {
                if let Some((view, sampler)) = self.registered_textures.get(texture_name) {
                    (view, sampler)
                } else {
                    (&self.dummy_texture_view, &self.dummy_sampler)
                }
            } else {
                (&self.dummy_texture_view, &self.dummy_sampler)
            };

            let bind_group = build::binding(
                context.device,
                "Decal Bind Group",
                &self.bind_group_layout,
                vec![
                    self.camera_buffer.as_entire_binding(),
                    self.decal_buffer.as_entire_binding(),
                    wgpu::BindingResource::TextureView(depth_view),
                    wgpu::BindingResource::TextureView(texture_view),
                    wgpu::BindingResource::Sampler(sampler),
                    wgpu::BindingResource::TextureView(&self.dummy_texture_view),
                    wgpu::BindingResource::Sampler(&self.dummy_sampler),
                ],
            );

            render_pass.set_bind_group(0, &bind_group, &[]);
            let instance_range = batch.start_index..(batch.start_index + batch.count);
            render_pass.draw_indexed(0..36, 0, instance_range);
        }

        drop(render_pass);

        Ok(())
    }
}

pub fn install_decal_pass(
    graph: &mut nightshade_wgpu::backend::RenderGraph<nightshade_wgpu::backend::RenderInputs>,
    build: PassBuild,
    _: nightshade_wgpu::wgpu::TextureFormat,
) {
    if let Err(error) = nightshade_wgpu::backend::install_lit_pass(
        graph,
        Box::new(DecalPass::new(build, build.hdr_format)),
        &[
            ("color", build.targets.scene_color),
            ("depth", build.targets.depth),
        ],
    ) {
        panic!("the decal pass could not be installed: {error}");
    }
}

pub fn sync_decal_pass(
    mut renderer: nightshade_wgpu::pre_render::Renderer,
    frame: Option<nightshade::ecs::system_param::Res<nightshade_decals::contract::DecalFrame>>,
) {
    let Some(frame) = frame else {
        return;
    };
    let Some(pass) =
        nightshade_wgpu::backend::render_graph_get_pass_mut(&mut renderer.graph, DECAL_PASS)
    else {
        return;
    };
    let Some(decal_pass) = (pass as &mut dyn std::any::Any).downcast_mut::<DecalPass>() else {
        return;
    };
    decal_pass.set_decals(frame.decals.clone());
}
