use nightshade_sdf::contract::field::{SCALE_FLOOR, uniform_scale};
use nightshade_sdf::contract::render_data::{SDF_BRICK_SAMPLES, SdfEditInstance};
use nightshade_sdf::contract::{SdfOperation, SdfShape};
use nightshade_wgpu::backend::{
    PassBuild, PassDecl, PassExecutionContext, PassNode, PassPosition, RenderInputs, Result,
    WgpuRenderer, build, placeholder,
};
use nightshade_wgpu::wgpu;

use crate::systems::cache::SdfCachePass;

pub(crate) const SDF_PASS: &str = "sdf_pass";
pub(crate) const SDF_CACHE_PASS: &str = "sdf_cache_pass";

pub(crate) const MAX_LEVELS: u32 = 12;
pub(crate) const MAX_SIDE: u32 = 64;
pub(crate) const ATLAS_BRICKS: u32 = 56;
pub(crate) const ATLAS_TEXELS: u32 = ATLAS_BRICKS * SDF_BRICK_SAMPLES;
pub(crate) const MAX_BRICKS: u32 = ATLAS_BRICKS * ATLAS_BRICKS * ATLAS_BRICKS;
pub(crate) const MAX_POINTERS: u32 = MAX_LEVELS * MAX_SIDE * MAX_SIDE * MAX_SIDE;
pub(crate) const MAX_MOVED: usize = 256;
pub(crate) const FIRST_EDITS: usize = 64;
pub(crate) const LEAF_EDITS: usize = 4;
pub(crate) const REACHES_EVERYWHERE: f32 = 1.0e9;
pub(crate) const WINDOW_GROUP: u32 = 4;
pub(crate) const RESET_GROUP: u32 = 256;
pub(crate) const CAST_GROUP: u32 = 8;
pub(crate) const SHADOW_SHRINK: u32 = 8;
pub(crate) const BAKE_ARGUMENTS: u32 = 4;
pub(crate) const DISPATCH_WORDS: u32 = 8;
pub(crate) const CONTROL_WORDS: u32 = 8;
pub(crate) const TILE: u32 = 8;
pub(crate) const TILE_GROUP: u32 = 8;

const STAGES: wgpu::ShaderStages = wgpu::ShaderStages::COMPUTE.union(wgpu::ShaderStages::FRAGMENT);

#[repr(C)]
#[derive(Copy, Clone, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GpuEdit {
    pub(crate) inverse_transform: [[f32; 4]; 4],
    pub(crate) params: [f32; 4],
    pub(crate) tint_blend: [f32; 4],
    pub(crate) kind_operation_scale: [f32; 4],
    pub(crate) bounds: [f32; 4],
    pub(crate) identity: [u32; 4],
    pub(crate) extra: [f32; 4],
    pub(crate) surface: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GpuNode {
    low: [f32; 4],
    high: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GpuUniforms {
    pub(crate) view_projection: [[f32; 4]; 4],
    pub(crate) inverse_view_projection: [[f32; 4]; 4],
    pub(crate) previous_view_projection: [[f32; 4]; 4],
    pub(crate) view: [[f32; 4]; 4],
    pub(crate) cascade_view_projections: [[[f32; 4]; 4]; 4],
    pub(crate) cascade_inverse_view_projections: [[[f32; 4]; 4]; 4],
    pub(crate) cascade_offsets: [[f32; 4]; 4],
    pub(crate) cascade_splits: [f32; 4],
    pub(crate) camera: [f32; 4],
    pub(crate) sun_direction: [f32; 4],
    pub(crate) sun_color: [f32; 4],
    pub(crate) screen: [f32; 4],
    pub(crate) viewport: [f32; 4],
    pub(crate) counts: [u32; 4],
    pub(crate) flags: [u32; 4],
    pub(crate) sizes: [f32; 4],
    pub(crate) terrain: [f32; 4],
    pub(crate) frame: [u32; 4],
}

#[derive(Clone)]
pub(crate) struct SdfHandles {
    pub(crate) shared_group: wgpu::BindGroup,
    pub(crate) read_group: wgpu::BindGroup,
    pub(crate) cast_view: wgpu::TextureView,
    pub(crate) motions: wgpu::Buffer,
    pub(crate) edits: u32,
}

pub(crate) fn march_size(size: (u32, u32), scale: f32) -> (u32, u32) {
    let shrink = |side: u32| ((side.max(1) as f32) * scale).ceil().max(1.0) as u32;
    (shrink(size.0), shrink(size.1))
}

fn tiles_over(size: (u32, u32)) -> (u32, u32) {
    (size.0.div_ceil(TILE), size.1.div_ceil(TILE))
}

pub(crate) fn shared_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    build::bind_layout(
        device,
        "Sdf Shared Bind Group Layout",
        &[
            build::uniform_at(0, STAGES),
            build::storage_at(1, false, STAGES),
            build::storage_at(2, false, STAGES),
        ],
    )
}

pub(crate) fn read_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    build::bind_layout(
        device,
        "Sdf Cache Read Bind Group Layout",
        &[
            build::storage_at(0, false, STAGES),
            build::texture_at(
                1,
                wgpu::TextureSampleType::Float { filterable: true },
                wgpu::TextureViewDimension::D3,
                STAGES,
            ),
            build::sampler_at(2, wgpu::SamplerBindingType::Filtering, STAGES),
            build::storage_at(3, false, STAGES),
            build::storage_at(4, false, STAGES),
        ],
    )
}

pub(crate) fn tree_nodes(capacity: usize) -> usize {
    let mut depth = 0;
    while capacity.div_ceil(1 << depth) > LEAF_EDITS {
        depth += 1;
    }
    (1 << (depth + 1)) - 1
}

pub(crate) fn build_tree(edits: &[GpuEdit], nodes: &mut [GpuNode]) {
    fill_node(edits, nodes, 0, 0, edits.len());
}

fn fill_node(edits: &[GpuEdit], nodes: &mut [GpuNode], node: usize, first: usize, count: usize) {
    if node >= nodes.len() {
        return;
    }
    if count == 0 {
        nodes[node] = GpuNode {
            low: [REACHES_EVERYWHERE; 4],
            high: [
                -REACHES_EVERYWHERE,
                -REACHES_EVERYWHERE,
                -REACHES_EVERYWHERE,
                0.0,
            ],
        };
        return;
    }
    let mut low = [f32::MAX; 3];
    let mut high = [f32::MIN; 3];
    let mut everywhere = false;
    for edit in &edits[first..first + count] {
        let reach = edit.bounds[3];
        if reach < 0.0 || edit.kind_operation_scale[1] > 1.5 {
            everywhere = true;
            break;
        }
        for axis in 0..3 {
            low[axis] = low[axis].min(edit.bounds[axis] - reach);
            high[axis] = high[axis].max(edit.bounds[axis] + reach);
        }
    }
    if everywhere {
        low = [-REACHES_EVERYWHERE; 3];
        high = [REACHES_EVERYWHERE; 3];
    }
    let leaf = count <= LEAF_EDITS || node * 2 + 2 >= nodes.len();
    nodes[node] = GpuNode {
        low: [low[0], low[1], low[2], first as f32],
        high: [
            high[0],
            high[1],
            high[2],
            if leaf { count as f32 } else { -1.0 },
        ],
    };
    if leaf {
        return;
    }
    let half = count / 2;
    fill_node(edits, nodes, node * 2 + 1, first, half);
    fill_node(edits, nodes, node * 2 + 2, first + half, count - half);
}

pub(crate) fn pack_edit(edit: &SdfEditInstance) -> GpuEdit {
    let terrain = matches!(edit.shape, SdfShape::Terrain { .. });
    let measured = uniform_scale(&edit.transform);
    let sound = terrain || (measured.is_finite() && measured > SCALE_FLOOR);
    let scale = if sound { measured } else { 1.0 };
    let inverse = if sound {
        nalgebra_glm::inverse(&edit.transform)
    } else {
        nalgebra_glm::Mat4::identity()
    };
    let (kind, params) = match edit.shape {
        SdfShape::Sphere { radius } => (0.0, [radius, 0.0, 0.0, 0.0]),
        SdfShape::Cube {
            half_extents,
            rounding,
        } => (
            1.0,
            [half_extents.x, half_extents.y, half_extents.z, rounding],
        ),
        SdfShape::Capsule {
            half_height,
            radius,
        } => (2.0, [half_height, radius, 0.0, 0.0]),
        SdfShape::Torus {
            major_radius,
            minor_radius,
        } => (3.0, [major_radius, minor_radius, 0.0, 0.0]),
        SdfShape::Cylinder {
            half_height,
            radius,
        } => (4.0, [half_height, radius, 0.0, 0.0]),
        SdfShape::Octahedron { radius, rounding } => (5.0, [radius, rounding, 0.0, 0.0]),
        SdfShape::Terrain {
            base,
            size,
            octaves,
            gain,
            ..
        } => (6.0, [base, size, octaves as f32, gain]),
    };
    let smoothings = match edit.shape {
        SdfShape::Terrain {
            clip,
            clip_smooth,
            join_smooth,
            ..
        } => [clip, clip_smooth, join_smooth],
        _ => [0.0; 3],
    };
    let extra = match edit.shape {
        SdfShape::Terrain { skin, dirt, .. } => [dirt.x, dirt.y, dirt.z, skin],
        _ => [0.0; 4],
    };
    let operation = match edit.operation {
        SdfOperation::Union => 0.0,
        SdfOperation::Subtract => 1.0,
        SdfOperation::Intersect => 2.0,
    };
    let middle = edit.transform.column(3).xyz();
    let blend = edit.blend.max(0.0);
    let bounds = match edit.shape.reach() {
        Some(reach) => [
            middle.x,
            middle.y,
            middle.z,
            if sound { reach * scale + blend } else { 0.0 },
        ],
        None => [smoothings[0], smoothings[1], smoothings[2], -1.0],
    };
    let visible = edit.visible && sound;
    GpuEdit {
        inverse_transform: inverse.into(),
        params,
        tint_blend: [edit.tint.x, edit.tint.y, edit.tint.z, blend],
        kind_operation_scale: [kind, operation, scale, f32::from(u8::from(visible))],
        bounds,
        identity: [edit.entity.id, 0, 0, 0],
        extra,
        surface: [edit.roughness, edit.metal, 0.0, 0.0],
    }
}

struct TileStart {
    size: (u32, u32),
    view: wgpu::TextureView,
}

struct LowTargets {
    size: (u32, u32),
    color: wgpu::TextureView,
    entity: wgpu::TextureView,
    normals: wgpu::TextureView,
    velocity: wgpu::TextureView,
    depth: wgpu::TextureView,
}

pub struct SdfMarchPass {
    pipeline: wgpu::RenderPipeline,
    shade_layout: wgpu::BindGroupLayout,
    tile_layout: wgpu::BindGroupLayout,
    tile_pipeline: wgpu::ComputePipeline,
    upscale_layout: wgpu::BindGroupLayout,
    upscale_pipeline: wgpu::RenderPipeline,
    irradiance_sampler: wgpu::Sampler,
    placeholder_cube: wgpu::TextureView,
    hdr_format: wgpu::TextureFormat,
    tile: Option<TileStart>,
    low: Option<LowTargets>,
    scale: f32,
    handles: Option<SdfHandles>,
}

impl SdfMarchPass {
    pub(crate) fn new(
        build: PassBuild,
        shared: &wgpu::BindGroupLayout,
        read: &wgpu::BindGroupLayout,
    ) -> Self {
        let PassBuild {
            device,
            queue,
            composer,
            hdr_format,
            ..
        } = build;
        let shade_layout = build::bind_layout(
            device,
            "Sdf Shade Bind Group Layout",
            &[
                build::depth_at(0, wgpu::ShaderStages::FRAGMENT),
                build::texture_at(
                    1,
                    wgpu::TextureSampleType::Float { filterable: true },
                    wgpu::TextureViewDimension::Cube,
                    wgpu::ShaderStages::FRAGMENT,
                ),
                build::texture_at(
                    2,
                    wgpu::TextureSampleType::Float { filterable: true },
                    wgpu::TextureViewDimension::Cube,
                    wgpu::ShaderStages::FRAGMENT,
                ),
                build::filtering_at(3),
                build::texture_at(
                    4,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::TextureViewDimension::D2,
                    wgpu::ShaderStages::FRAGMENT,
                ),
                build::texture_at(
                    5,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::TextureViewDimension::D2,
                    wgpu::ShaderStages::FRAGMENT,
                ),
                build::storage_at(6, false, wgpu::ShaderStages::FRAGMENT),
                build::texture_at(
                    7,
                    wgpu::TextureSampleType::Float { filterable: true },
                    wgpu::TextureViewDimension::Cube,
                    wgpu::ShaderStages::FRAGMENT,
                ),
                build::texture_at(
                    8,
                    wgpu::TextureSampleType::Float { filterable: true },
                    wgpu::TextureViewDimension::Cube,
                    wgpu::ShaderStages::FRAGMENT,
                ),
            ],
        );
        let irradiance_sampler = build::smooth(device, "Sdf Irradiance Sampler");
        let placeholder_cube = placeholder::placeholder_texture(
            device,
            queue,
            "Sdf Sky Placeholder",
            wgpu::TextureFormat::Rgba16Float,
            6,
            &placeholder::TRANSPARENT_RGBA16,
        )
        .create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });
        let shader = composer.compile_wgsl(
            device,
            "sdf_march.wgsl",
            include_str!("../shaders/sdf_march.wgsl"),
        );
        let layout = build::pipeline_layout(
            device,
            "Sdf March Pipeline Layout",
            &[shared, read, &shade_layout],
        );
        let targets = [
            Some(build::opaque(hdr_format)),
            Some(build::opaque(wgpu::TextureFormat::R32Float)),
            Some(build::opaque(wgpu::TextureFormat::Rgba16Float)),
            Some(build::opaque(wgpu::TextureFormat::Rg16Float)),
        ];
        let pipeline = build::Pipe::new(device, "Sdf March Pipeline", &shader)
            .layout(&layout)
            .targets(&targets)
            .depth(build::depth_state(wgpu::TextureFormat::Depth32Float, true))
            .vertex("vs_main")
            .fragment("fs_main")
            .make();

        let tile_layout = build::bind_layout(
            device,
            "Sdf Tile Bind Group Layout",
            &[build::storage_texture_at(
                0,
                wgpu::TextureFormat::R32Float,
                wgpu::StorageTextureAccess::WriteOnly,
                wgpu::TextureViewDimension::D2,
                wgpu::ShaderStages::COMPUTE,
            )],
        );
        let tile_shader = composer.compile_wgsl(
            device,
            "sdf_tile.wgsl",
            include_str!("../shaders/sdf_tile.wgsl"),
        );
        let tile_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sdf Tile Pipeline Layout"),
            bind_group_layouts: &[Some(shared), Some(read), Some(&tile_layout)],
            immediate_size: 0,
        });
        let tile_pipeline = build::compute(
            device,
            "Sdf Tile Pipeline",
            &tile_shader,
            "main",
            &tile_pipeline_layout,
        );

        let flat = |binding: u32| {
            build::texture_at(
                binding,
                wgpu::TextureSampleType::Float { filterable: false },
                wgpu::TextureViewDimension::D2,
                wgpu::ShaderStages::FRAGMENT,
            )
        };
        let upscale_layout = build::bind_layout(
            device,
            "Sdf Upscale Bind Group Layout",
            &[
                flat(0),
                flat(1),
                flat(2),
                flat(3),
                build::depth_at(4, wgpu::ShaderStages::FRAGMENT),
            ],
        );
        let upscale_shader = composer.compile_wgsl(
            device,
            "sdf_upscale.wgsl",
            include_str!("../shaders/sdf_upscale.wgsl"),
        );
        let upscale_pipeline_layout = build::pipeline_layout(
            device,
            "Sdf Upscale Pipeline Layout",
            &[shared, &upscale_layout],
        );
        let upscale_pipeline = build::Pipe::new(device, "Sdf Upscale Pipeline", &upscale_shader)
            .layout(&upscale_pipeline_layout)
            .targets(&targets)
            .depth(build::depth_state(wgpu::TextureFormat::Depth32Float, true))
            .vertex("vs_main")
            .fragment("fs_main")
            .make();
        Self {
            pipeline,
            shade_layout,
            tile_layout,
            tile_pipeline,
            upscale_layout,
            upscale_pipeline,
            irradiance_sampler,
            placeholder_cube,
            hdr_format,
            tile: None,
            low: None,
            scale: 1.0,
            handles: None,
        }
    }

    pub(crate) fn receive(&mut self, handles: SdfHandles, scale: f32) {
        self.handles = Some(handles);
        self.scale = scale;
    }

    fn fit(&mut self, device: &wgpu::Device, viewport: (u32, u32)) {
        let march = march_size(viewport, self.scale);
        let tiles = tiles_over(march);
        if self.tile.as_ref().is_none_or(|tile| tile.size != tiles) {
            self.tile = Some(TileStart {
                size: tiles,
                view: build::view_of(
                    device,
                    "Sdf Tile Start",
                    wgpu::TextureFormat::R32Float,
                    tiles.0,
                    tiles.1,
                    wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
                ),
            });
        }
        if march == viewport {
            self.low = None;
            return;
        }
        if self.low.as_ref().is_some_and(|low| low.size == march) {
            return;
        }
        let usage = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        let target = |label: &str, format: wgpu::TextureFormat| {
            build::view_of(device, label, format, march.0, march.1, usage)
        };
        self.low = Some(LowTargets {
            size: march,
            color: target("Sdf Low Color", self.hdr_format),
            entity: target("Sdf Low Entity Id", wgpu::TextureFormat::R32Float),
            normals: target("Sdf Low Normals", wgpu::TextureFormat::Rgba16Float),
            velocity: target("Sdf Low Velocity", wgpu::TextureFormat::Rg16Float),
            depth: target("Sdf Low Depth", wgpu::TextureFormat::Depth32Float),
        });
    }
}

fn cleared(view: &wgpu::TextureView) -> Option<wgpu::RenderPassColorAttachment<'_>> {
    attachment(
        view,
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        wgpu::StoreOp::Store,
    )
}

const SDF_PASS_DECL: PassDecl = PassDecl {
    name: SDF_PASS,
    reads: &["shadow_depth"],
    reads_writes: &["color", "depth", "entity_id", "view_normals", "velocity"],
    optional_reads: &[
        "ibl_irradiance",
        "ibl_irradiance_b",
        "ibl_prefiltered",
        "ibl_prefiltered_b",
    ],
    ..PassDecl::EMPTY
};

fn attachment(
    view: &wgpu::TextureView,
    load: wgpu::LoadOp<wgpu::Color>,
    store: wgpu::StoreOp,
) -> Option<wgpu::RenderPassColorAttachment<'_>> {
    Some(wgpu::RenderPassColorAttachment {
        view,
        resolve_target: None,
        ops: wgpu::Operations { load, store },
        depth_slice: None,
    })
}

impl PassNode<RenderInputs> for SdfMarchPass {
    fn decl(&self) -> &PassDecl {
        &SDF_PASS_DECL
    }

    fn prepare(&mut self, device: &wgpu::Device, _queue: &wgpu::Queue, configs: &RenderInputs) {
        let viewport = configs.view.viewport_size.unwrap_or((1920, 1080));
        self.fit(device, viewport);
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, RenderInputs>,
    ) -> Result<()> {
        let Some(handles) = self.handles.as_ref() else {
            return Ok(());
        };
        if !context.is_pass_enabled()
            || handles.edits == 0
            || context.configs.scene.render_view.is_none()
        {
            return Ok(());
        }

        let Some(tile) = self.tile.as_ref() else {
            return Ok(());
        };
        let (shadow_view, _, _) = context.get_depth_attachment("shadow_depth")?;
        let irradiance = context
            .get_texture_view("ibl_irradiance")
            .unwrap_or(&self.placeholder_cube);
        let irradiance_other = context
            .get_texture_view("ibl_irradiance_b")
            .unwrap_or(&self.placeholder_cube);
        let prefiltered = context
            .get_texture_view("ibl_prefiltered")
            .unwrap_or(&self.placeholder_cube);
        let prefiltered_other = context
            .get_texture_view("ibl_prefiltered_b")
            .unwrap_or(&self.placeholder_cube);
        let shade_group = build::binding(
            context.device,
            "Sdf Shade Bind Group",
            &self.shade_layout,
            vec![
                wgpu::BindingResource::TextureView(shadow_view),
                wgpu::BindingResource::TextureView(irradiance),
                wgpu::BindingResource::TextureView(irradiance_other),
                wgpu::BindingResource::Sampler(&self.irradiance_sampler),
                wgpu::BindingResource::TextureView(&handles.cast_view),
                wgpu::BindingResource::TextureView(&tile.view),
                handles.motions.as_entire_binding(),
                wgpu::BindingResource::TextureView(prefiltered),
                wgpu::BindingResource::TextureView(prefiltered_other),
            ],
        );
        let tile_group = build::binding(
            context.device,
            "Sdf Tile Bind Group",
            &self.tile_layout,
            vec![wgpu::BindingResource::TextureView(&tile.view)],
        );
        let mut tiles = context
            .encoder
            .begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Sdf Tile Pass"),
                timestamp_writes: None,
            });
        tiles.set_pipeline(&self.tile_pipeline);
        tiles.set_bind_group(0, &handles.shared_group, &[]);
        tiles.set_bind_group(1, &handles.read_group, &[]);
        tiles.set_bind_group(2, &tile_group, &[]);
        tiles.dispatch_workgroups(
            tile.size.0.div_ceil(TILE_GROUP),
            tile.size.1.div_ceil(TILE_GROUP),
            1,
        );
        drop(tiles);

        let (color_view, color_load, color_store) = context.get_color_attachment("color")?;
        let (entity_view, entity_load, entity_store) = context.get_color_attachment("entity_id")?;
        let (normals_view, normals_load, normals_store) =
            context.get_color_attachment("view_normals")?;
        let (velocity_view, velocity_load, velocity_store) =
            context.get_color_attachment("velocity")?;
        let (depth_view, depth_load, depth_store) = context.get_depth_attachment("depth")?;

        if let Some(low) = self.low.as_ref() {
            let mut march = context
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Sdf March Pass"),
                    color_attachments: &[
                        cleared(&low.color),
                        cleared(&low.entity),
                        cleared(&low.normals),
                        cleared(&low.velocity),
                    ],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &low.depth,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(0.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
            march.set_pipeline(&self.pipeline);
            march.set_bind_group(0, &handles.shared_group, &[]);
            march.set_bind_group(1, &handles.read_group, &[]);
            march.set_bind_group(2, &shade_group, &[]);
            march.draw(0..3, 0..1);
            drop(march);

            let upscale_group = build::binding(
                context.device,
                "Sdf Upscale Bind Group",
                &self.upscale_layout,
                vec![
                    wgpu::BindingResource::TextureView(&low.color),
                    wgpu::BindingResource::TextureView(&low.entity),
                    wgpu::BindingResource::TextureView(&low.normals),
                    wgpu::BindingResource::TextureView(&low.velocity),
                    wgpu::BindingResource::TextureView(&low.depth),
                ],
            );
            let mut upscale = context
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Sdf Upscale Pass"),
                    color_attachments: &[
                        attachment(color_view, color_load, color_store),
                        attachment(entity_view, entity_load, entity_store),
                        attachment(normals_view, normals_load, normals_store),
                        attachment(velocity_view, velocity_load, velocity_store),
                    ],
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
            upscale.set_pipeline(&self.upscale_pipeline);
            upscale.set_bind_group(0, &handles.shared_group, &[]);
            upscale.set_bind_group(1, &upscale_group, &[]);
            upscale.draw(0..3, 0..1);
            drop(upscale);
            return Ok(());
        }

        let mut march = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Sdf March Pass"),
                color_attachments: &[
                    attachment(color_view, color_load, color_store),
                    attachment(entity_view, entity_load, entity_store),
                    attachment(normals_view, normals_load, normals_store),
                    attachment(velocity_view, velocity_load, velocity_store),
                ],
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
        march.set_pipeline(&self.pipeline);
        march.set_bind_group(0, &handles.shared_group, &[]);
        march.set_bind_group(1, &handles.read_group, &[]);
        march.set_bind_group(2, &shade_group, &[]);
        march.draw(0..3, 0..1);
        drop(march);
        Ok(())
    }
}

pub fn install_sdf_pass(
    graph: &mut nightshade_wgpu::backend::RenderGraph<RenderInputs>,
    build: PassBuild,
    _: wgpu::TextureFormat,
) {
    build.composer.publish_module(
        include_str!("../shaders/sdf_common.wgsl"),
        "sdf_common.wgsl",
    );
    build
        .composer
        .publish_module(include_str!("../shaders/sdf_cache.wgsl"), "sdf_cache.wgsl");
    let shared = shared_layout(build.device);
    let read = read_layout(build.device);
    let targets = build.targets;
    if let Err(error) = nightshade_wgpu::backend::render_graph_add_pass_at(
        graph,
        Box::new(SdfCachePass::new(build, &shared, &read)),
        &[("shadow_depth", targets.shadow_depth)],
        PassPosition::Background,
    ) {
        panic!("the sdf cache pass could not be installed: {error}");
    }
    if let Err(error) = nightshade_wgpu::backend::install_lit_pass(
        graph,
        Box::new(SdfMarchPass::new(build, &shared, &read)),
        &[
            ("color", targets.scene_color),
            ("depth", targets.depth),
            ("entity_id", targets.entity_id),
            ("view_normals", targets.view_normals),
            ("velocity", targets.velocity),
            ("shadow_depth", targets.shadow_depth),
            ("ibl_irradiance", targets.ibl_irradiance),
            ("ibl_irradiance_b", targets.ibl_irradiance_b),
            ("ibl_prefiltered", targets.ibl_prefiltered),
            ("ibl_prefiltered_b", targets.ibl_prefiltered_b),
        ],
    ) {
        panic!("the sdf pass could not be installed: {error}");
    }
}

pub fn sync_sdf_pass(
    mut renderer: nightshade_wgpu::pre_render::Renderer,
    frame: Option<
        nightshade::ecs::system_param::Res<nightshade_sdf::contract::render_data::SdfFrame>,
    >,
    clipmap: Option<
        nightshade::ecs::system_param::Res<nightshade_sdf::contract::render_data::SdfClipmap>,
    >,
    view: Option<
        nightshade::ecs::system_param::Res<nightshade_sdf::contract::render_data::SdfDebugView>,
    >,
    scale: Option<
        nightshade::ecs::system_param::Res<nightshade_sdf::contract::render_data::SdfMarchScale>,
    >,
) {
    let (Some(frame), Some(clipmap), Some(view), Some(scale)) = (frame, clipmap, view, scale)
    else {
        return;
    };
    let scale = scale.0.clamp(0.25, 1.0);
    let debug = match *view {
        nightshade_sdf::contract::render_data::SdfDebugView::Lit => 0,
        nightshade_sdf::contract::render_data::SdfDebugView::Bricks => 1,
        nightshade_sdf::contract::render_data::SdfDebugView::Levels => 2,
        nightshade_sdf::contract::render_data::SdfDebugView::Steps => 3,
        nightshade_sdf::contract::render_data::SdfDebugView::Baked => 4,
    };
    let renderer: &mut WgpuRenderer = &mut renderer;
    let device = &renderer.device;
    let graph = &mut renderer.graph;
    let Some(cache) = nightshade_wgpu::backend::render_graph_get_pass_mut(graph, SDF_CACHE_PASS)
        .and_then(|pass| (pass as &mut dyn std::any::Any).downcast_mut::<SdfCachePass>())
    else {
        return;
    };
    cache.set_edits(device, &frame.edits, &clipmap, debug, scale);
    let handles = cache.handles();
    let Some(march) = nightshade_wgpu::backend::render_graph_get_pass_mut(graph, SDF_PASS)
        .and_then(|pass| (pass as &mut dyn std::any::Any).downcast_mut::<SdfMarchPass>())
    else {
        return;
    };
    march.receive(handles, scale);
}
