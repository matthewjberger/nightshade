use crate::data::SPRITE_PASS;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;

use nalgebra_glm::{Mat4, Vec3};
use nightshade_sprite::contract::SpriteFilter;
use nightshade_sprite::contract::render_data::{
    SpriteAtlasSpec, SpriteAtlasUpload, SpriteInstance,
};
use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::backend::RenderInputs;
use nightshade_wgpu::backend::{PassDecl, PassExecutionContext, PassNode};

const INITIAL_INSTANCE_CAPACITY: usize = 1024;
const VERTICES_PER_SPRITE: u32 = 6;
const FLAG_BILLBOARD: u32 = 1;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuSpriteInstance {
    model: [[f32; 4]; 4],
    uv_min: [f32; 2],
    uv_max: [f32; 2],
    size: [f32; 2],
    anchor: [f32; 2],
    color: [f32; 4],
    flags: u32,
    lean: f32,
    padding: [u32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct SpriteUniforms {
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
    camera_right: [f32; 4],
    camera_up: [f32; 4],
}

struct AtlasTexture {
    size: u32,
    filter: SpriteFilter,
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}

struct SpriteBatch {
    atlas: u32,
    first: u32,
    count: u32,
}

pub struct SpritePass {
    pipeline: wgpu::RenderPipeline,
    atlas_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instance_capacity: usize,
    instance_bind_group: wgpu::BindGroup,
    instance_layout: wgpu::BindGroupLayout,
    linear_sampler: wgpu::Sampler,
    nearest_sampler: wgpu::Sampler,
    atlases: Vec<Option<AtlasTexture>>,
    wanted_atlases: Vec<SpriteAtlasSpec>,
    pending_uploads: Vec<SpriteAtlasUpload>,
    instances: Vec<SpriteInstance>,
    gpu_instances: Vec<GpuSpriteInstance>,
    batches: Vec<SpriteBatch>,
}

impl SpritePass {
    pub fn new(build: PassBuild, color_format: wgpu::TextureFormat) -> Self {
        let PassBuild {
            device, composer, ..
        } = build;
        let instance_layout = build::bind_layout(
            device,
            "Sprite Instance Layout",
            &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<SpriteUniforms>() as u64,
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<GpuSpriteInstance>() as u64,
                        ),
                    },
                    count: None,
                },
            ],
        );

        let atlas_layout = build::bind_layout(
            device,
            "Sprite Atlas Layout",
            &[build::painted_at(0), build::filtering_at(1)],
        );

        let pipeline_layout = build::pipeline_layout(
            device,
            "Sprite Pipeline Layout",
            &[&instance_layout, &atlas_layout],
        );

        let shader = composer.compile_wgsl(
            device,
            "Sprite Shader",
            include_str!("../shaders/sprite.wgsl"),
        );

        let pipeline = build::Pipe::new(device, "Sprite Pipeline", &shader)
            .layout(&pipeline_layout)
            .target(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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

        let uniform_buffer = build::uniform(
            device,
            "Sprite Uniform Buffer",
            std::mem::size_of::<SpriteUniforms>() as u64,
        );

        let instance_buffer = create_instance_buffer(device, INITIAL_INSTANCE_CAPACITY);
        let instance_bind_group =
            build_instance_bind_group(device, &instance_layout, &uniform_buffer, &instance_buffer);

        let linear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sprite Atlas Sampler (Linear)"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let nearest_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sprite Atlas Sampler (Nearest)"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self {
            pipeline,
            atlas_layout,
            uniform_buffer,
            instance_buffer,
            instance_capacity: INITIAL_INSTANCE_CAPACITY,
            instance_bind_group,
            instance_layout,
            linear_sampler,
            nearest_sampler,
            atlases: Vec::new(),
            wanted_atlases: Vec::new(),
            pending_uploads: Vec::new(),
            instances: Vec::new(),
            gpu_instances: Vec::new(),
            batches: Vec::new(),
        }
    }

    pub fn set_atlases(&mut self, atlases: &[SpriteAtlasSpec]) {
        self.wanted_atlases.clear();
        self.wanted_atlases.extend_from_slice(atlases);
    }

    pub fn queue_uploads(&mut self, uploads: Vec<SpriteAtlasUpload>) {
        self.pending_uploads.extend(uploads);
    }

    pub fn set_instances(&mut self, instances: Vec<SpriteInstance>) {
        self.instances = instances;
    }

    fn ensure_atlases(&mut self, device: &wgpu::Device) {
        for spec in &self.wanted_atlases {
            let index = spec.index as usize;
            if index >= self.atlases.len() {
                self.atlases.resize_with(index + 1, || None);
            }
            let stale = self.atlases[index]
                .as_ref()
                .is_some_and(|atlas| atlas.size != spec.size || atlas.filter != spec.filter);
            if self.atlases[index].is_some() && !stale {
                continue;
            }
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Sprite Atlas"),
                size: nightshade_wgpu::backend::texture_extent(device, spec.size, spec.size, 1),
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = match spec.filter {
                SpriteFilter::Nearest => &self.nearest_sampler,
                SpriteFilter::Linear => &self.linear_sampler,
            };
            let bind_group = build::binding(
                device,
                "Sprite Atlas Bind Group",
                &self.atlas_layout,
                vec![
                    wgpu::BindingResource::TextureView(&view),
                    wgpu::BindingResource::Sampler(sampler),
                ],
            );
            self.atlases[index] = Some(AtlasTexture {
                size: spec.size,
                filter: spec.filter,
                texture,
                bind_group,
            });
        }
    }

    fn apply_uploads(&mut self, queue: &wgpu::Queue) {
        let uploads = std::mem::take(&mut self.pending_uploads);
        for upload in uploads {
            let Some(atlas) = self
                .atlases
                .get(upload.atlas as usize)
                .and_then(|slot| slot.as_ref())
            else {
                self.pending_uploads.push(upload);
                continue;
            };
            if upload.width == 0
                || upload.height == 0
                || upload.x + upload.width > atlas.size
                || upload.y + upload.height > atlas.size
                || upload.rgba.len() != (upload.width * upload.height * 4) as usize
            {
                continue;
            }
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &atlas.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: upload.x,
                        y: upload.y,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &upload.rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(upload.width * 4),
                    rows_per_image: Some(upload.height),
                },
                wgpu::Extent3d {
                    width: upload.width,
                    height: upload.height,
                    depth_or_array_layers: 1,
                },
            );
        }
    }

    fn build_batches(&mut self, view: &Mat4) {
        self.gpu_instances.clear();
        self.batches.clear();

        for index in
            nightshade_sprite::contract::render_data::sprite_draw_order(&self.instances, view)
        {
            let instance = &self.instances[index];
            if self
                .atlases
                .get(instance.atlas as usize)
                .is_none_or(|slot| slot.is_none())
            {
                continue;
            }
            let slot = self.gpu_instances.len() as u32;
            self.gpu_instances.push(GpuSpriteInstance {
                model: instance.transform.into(),
                uv_min: [instance.uv_min.x, instance.uv_min.y],
                uv_max: [instance.uv_max.x, instance.uv_max.y],
                size: [instance.size.x, instance.size.y],
                anchor: [instance.anchor.x, instance.anchor.y],
                color: instance.color,
                flags: if instance.billboard {
                    FLAG_BILLBOARD
                } else {
                    0
                },
                lean: instance.lean,
                padding: [0; 2],
            });
            match self.batches.last_mut() {
                Some(batch) if batch.atlas == instance.atlas => batch.count += 1,
                _ => self.batches.push(SpriteBatch {
                    atlas: instance.atlas,
                    first: slot,
                    count: 1,
                }),
            }
        }
    }

    fn ensure_instance_capacity(&mut self, device: &wgpu::Device, required: usize) {
        if required <= self.instance_capacity {
            return;
        }
        let new_capacity = required.next_power_of_two();
        self.instance_buffer = create_instance_buffer(device, new_capacity);
        self.instance_capacity = new_capacity;
        self.instance_bind_group = build_instance_bind_group(
            device,
            &self.instance_layout,
            &self.uniform_buffer,
            &self.instance_buffer,
        );
    }
}

fn create_instance_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
    build::storage(
        device,
        "Sprite Instance Buffer",
        (std::mem::size_of::<GpuSpriteInstance>() * capacity) as u64,
    )
}

fn build_instance_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    uniforms: &wgpu::Buffer,
    instances: &wgpu::Buffer,
) -> wgpu::BindGroup {
    build::binding(
        device,
        "Sprite Instance Bind Group",
        layout,
        vec![uniforms.as_entire_binding(), instances.as_entire_binding()],
    )
}

const SPRITE_PASS_DECL: PassDecl = PassDecl {
    name: SPRITE_PASS,
    reads_writes: &["color", "depth"],
    ..PassDecl::EMPTY
};

impl PassNode<RenderInputs> for SpritePass {
    fn decl(&self) -> &PassDecl {
        &SPRITE_PASS_DECL
    }

    fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, configs: &RenderInputs) {
        self.ensure_atlases(device);
        self.apply_uploads(queue);

        let Some(render_view) = configs.scene.render_view.as_ref() else {
            self.gpu_instances.clear();
            self.batches.clear();
            return;
        };

        self.build_batches(&render_view.view);
        if self.gpu_instances.is_empty() {
            return;
        }

        let right: Vec3 = render_view.camera_right;
        let up: Vec3 = render_view.camera_up;
        let uniforms = SpriteUniforms {
            view: render_view.view.into(),
            projection: render_view.projection.into(),
            camera_right: [right.x, right.y, right.z, 0.0],
            camera_up: [up.x, up.y, up.z, 0.0],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        self.ensure_instance_capacity(device, self.gpu_instances.len());
        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&self.gpu_instances),
        );
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, RenderInputs>,
    ) -> nightshade_wgpu::backend::Result<()> {
        if self.batches.is_empty() {
            return Ok(());
        }

        let (color_view, _, color_store) = context.get_color_attachment("color")?;
        let (depth_view, _, depth_store) = context.get_depth_attachment("depth")?;

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Sprite Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: color_store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: depth_store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.instance_bind_group, &[]);

        for batch in &self.batches {
            let Some(atlas) = self
                .atlases
                .get(batch.atlas as usize)
                .and_then(|slot| slot.as_ref())
            else {
                continue;
            };
            render_pass.set_bind_group(1, &atlas.bind_group, &[]);
            render_pass.draw(
                0..VERTICES_PER_SPRITE,
                batch.first..batch.first + batch.count,
            );
        }

        drop(render_pass);
        Ok(())
    }
}

pub fn sync_sprite_pass(
    mut renderer: nightshade_wgpu::pre_render::Renderer,
    frame: Option<
        nightshade::ecs::system_param::ResMut<
            nightshade_sprite::contract::render_data::SpriteFrame,
        >,
    >,
) {
    let Some(mut frame) = frame else {
        return;
    };
    let Some(pass) = nightshade_wgpu::backend::render_graph_get_pass_mut(
        &mut renderer.graph,
        crate::data::SPRITE_PASS,
    ) else {
        return;
    };
    let Some(sprite_pass) = (pass as &mut dyn std::any::Any).downcast_mut::<SpritePass>() else {
        return;
    };
    sprite_pass.set_atlases(&frame.atlases);
    sprite_pass.queue_uploads(std::mem::take(&mut frame.uploads));
    sprite_pass.set_instances(frame.instances.clone());
}

use nightshade_wgpu::backend::{RenderGraph, install_lit_pass};
pub fn install_sprite_pass(
    graph: &mut RenderGraph<RenderInputs>,
    build: PassBuild,
    _: wgpu::TextureFormat,
) {
    if let Err(error) = install_lit_pass(
        graph,
        Box::new(SpritePass::new(build, build.hdr_format)),
        &[
            ("color", build.targets.scene_color),
            ("depth", build.targets.depth),
        ],
    ) {
        panic!("the {SPRITE_PASS} could not be installed: {error}");
    }
}
