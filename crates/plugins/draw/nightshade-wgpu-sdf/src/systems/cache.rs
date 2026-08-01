use std::collections::HashMap;

use nightshade::ecs::Entity;
use nightshade_render::components::RenderMesh;
use nightshade_sdf::contract::field::{blend_guard, field_doubt};
use nightshade_sdf::contract::render_data::{SdfClipmap, SdfEditInstance};
use nightshade_wgpu::backend::{
    PassBuild, PassDecl, PassExecutionContext, PassNode, RenderInputs, Result, build, caps,
};
use nightshade_wgpu::wgpu;

use crate::systems::sdf::{
    ATLAS_BRICKS, ATLAS_TEXELS, BAKE_ARGUMENTS, CAST_GROUP, CONTROL_WORDS, DISPATCH_WORDS,
    FIRST_EDITS, GpuEdit, GpuNode, GpuUniforms, MAX_BRICKS, MAX_LEVELS, MAX_MOVED, MAX_POINTERS,
    MAX_SIDE, RESET_GROUP, SDF_CACHE_PASS, SHADOW_SHRINK, SdfHandles, WINDOW_GROUP, build_tree,
    march_size, pack_edit, tree_nodes,
};

const STAGES: wgpu::ShaderStages = wgpu::ShaderStages::COMPUTE.union(wgpu::ShaderStages::FRAGMENT);
const MOTION_BYTES: usize = 64;

pub struct SdfCachePass {
    uniforms: wgpu::Buffer,
    edits: wgpu::Buffer,
    tree: wgpu::Buffer,
    motions: wgpu::Buffer,
    edit_capacity: usize,
    moved: wgpu::Buffer,
    pointers: wgpu::Buffer,
    dispatch: wgpu::Buffer,
    cast_size: u32,
    cast_view: wgpu::TextureView,
    shared_layout: wgpu::BindGroupLayout,
    shared_group: wgpu::BindGroup,
    read_group: wgpu::BindGroup,
    reset_group: wgpu::BindGroup,
    mark_group: wgpu::BindGroup,
    scan_group: wgpu::BindGroup,
    work_group: wgpu::BindGroup,
    survey_group: wgpu::BindGroup,
    allocate_group: wgpu::BindGroup,
    indirect_group: wgpu::BindGroup,
    bake_group: wgpu::BindGroup,
    cast_group: wgpu::BindGroup,
    write_group: wgpu::BindGroup,
    reset_pipeline: wgpu::ComputePipeline,
    mark_pipeline: wgpu::ComputePipeline,
    scan_pipeline: wgpu::ComputePipeline,
    work_pipeline: wgpu::ComputePipeline,
    survey_pipeline: wgpu::ComputePipeline,
    allocate_pipeline: wgpu::ComputePipeline,
    indirect_pipeline: wgpu::ComputePipeline,
    bake_pipeline: wgpu::ComputePipeline,
    cast_pipeline: wgpu::ComputePipeline,
    write_pipeline: wgpu::RenderPipeline,
    edit_data: Vec<GpuEdit>,
    edit_entities: Vec<Entity>,
    previous: HashMap<Entity, (u32, GpuEdit)>,
    motion_data: Vec<[[f32; 4]; 4]>,
    moved_data: Vec<[f32; 4]>,
    tree_data: Vec<GpuNode>,
    levels: u32,
    side: u32,
    finest_cell: f32,
    debug: u32,
    scale: f32,
    terrain: [f32; 4],
    frame: u32,
    throw_away: bool,
    casts: bool,
}

fn storage_buffer(device: &wgpu::Device, label: &str, bytes: u64, indirect: bool) -> wgpu::Buffer {
    let mut usage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
    if indirect {
        usage |= wgpu::BufferUsages::INDIRECT;
    }
    build::buffer(device, label, bytes, usage)
}

fn compute_pipeline(
    device: &wgpu::Device,
    label: &str,
    layouts: &[Option<&wgpu::BindGroupLayout>],
    module: &wgpu::ShaderModule,
) -> wgpu::ComputePipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: layouts,
        immediate_size: 0,
    });
    build::compute(device, label, module, "main", &layout)
}

fn shared_binding(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    uniforms: &wgpu::Buffer,
    edits: &wgpu::Buffer,
    tree: &wgpu::Buffer,
) -> wgpu::BindGroup {
    build::binding(
        device,
        "Sdf Shared Bind Group",
        layout,
        vec![
            uniforms.as_entire_binding(),
            edits.as_entire_binding(),
            tree.as_entire_binding(),
        ],
    )
}

fn writes(count: u32) -> Vec<wgpu::BindGroupLayoutEntry> {
    (0..count)
        .map(|binding| build::storage_at(binding, true, STAGES))
        .collect()
}

fn balls(
    previous: &HashMap<Entity, (u32, GpuEdit)>,
    entities: &[Entity],
    edits: &[GpuEdit],
) -> Option<Vec<[f32; 4]>> {
    let mut before = previous.clone();
    let mut moved = Vec::new();
    let mut push = |edit: &GpuEdit| -> bool {
        if edit.kind_operation_scale[3] < 0.5 {
            return true;
        }
        if edit.bounds[3] < 0.0 || moved.len() >= MAX_MOVED {
            return false;
        }
        moved.push(edit.bounds);
        true
    };
    for (index, (entity, now)) in entities.iter().zip(edits).enumerate() {
        match before.remove(entity) {
            Some((was_index, was)) => {
                if was == *now && was_index as usize == index {
                    continue;
                }
                let unmoved = was.bounds == now.bounds
                    && was.kind_operation_scale[3] == now.kind_operation_scale[3];
                if !push(&was) || (!unmoved && !push(now)) {
                    return None;
                }
            }
            None => {
                if !push(now) {
                    return None;
                }
            }
        }
    }
    for (_, was) in before.values() {
        if !push(was) {
            return None;
        }
    }
    Some(moved)
}

impl SdfCachePass {
    pub(crate) fn new(
        build: PassBuild,
        shared_layout: &wgpu::BindGroupLayout,
        read_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let PassBuild {
            device, composer, ..
        } = build;

        let uniforms = build::uniform(
            device,
            "Sdf Uniform Buffer",
            std::mem::size_of::<GpuUniforms>() as u64,
        );
        let edit_capacity = FIRST_EDITS;
        let edits = storage_buffer(
            device,
            "Sdf Edit Buffer",
            (edit_capacity * std::mem::size_of::<GpuEdit>()) as u64,
            false,
        );
        let tree_data = vec![GpuNode::default(); tree_nodes(edit_capacity)];
        let tree = storage_buffer(
            device,
            "Sdf Edit Tree",
            (tree_data.len() * std::mem::size_of::<GpuNode>()) as u64,
            false,
        );
        let motions = storage_buffer(
            device,
            "Sdf Edit Motions",
            (edit_capacity * MOTION_BYTES) as u64,
            false,
        );
        let moved = build::uniform(device, "Sdf Moved Buffer", (MAX_MOVED as u64) * 16);
        let pointers = storage_buffer(
            device,
            "Sdf Brick Pointer Buffer",
            (MAX_POINTERS as u64) * 4,
            false,
        );
        let slots = storage_buffer(
            device,
            "Sdf Free And Bake Slots",
            (MAX_BRICKS as u64) * 8,
            false,
        );
        let control = storage_buffer(device, "Sdf Control", (CONTROL_WORDS as u64) * 4, false);
        let dispatch = storage_buffer(
            device,
            "Sdf Dispatch Arguments",
            (DISPATCH_WORDS as u64) * 4,
            true,
        );
        let bricks = storage_buffer(device, "Sdf Brick Places", (MAX_BRICKS as u64) * 8, false);
        let stamps = storage_buffer(device, "Sdf Bake Stamps", (MAX_BRICKS as u64) * 4, false);
        let stir = storage_buffer(device, "Sdf Stir Stamps", (MAX_POINTERS as u64) * 4, false);
        let work = storage_buffer(device, "Sdf Work List", (MAX_POINTERS as u64) * 4, false);
        let identities = storage_buffer(
            device,
            "Sdf Cell Identities",
            (MAX_POINTERS as u64) * 8,
            false,
        );

        let atlas = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Sdf Brick Atlas"),
                size: wgpu::Extent3d {
                    width: ATLAS_TEXELS,
                    height: ATLAS_TEXELS,
                    depth_or_array_layers: ATLAS_TEXELS,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D3,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = build::smooth(device, "Sdf Brick Sampler");

        let cast_size = (caps::CASCADE_ATLAS_SIZE / SHADOW_SHRINK).max(1);
        let cast = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Sdf Shadow Cast"),
                size: wgpu::Extent3d {
                    width: cast_size,
                    height: cast_size,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R32Float,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default());

        let shared_group = shared_binding(device, shared_layout, &uniforms, &edits, &tree);
        let read_group = build::binding(
            device,
            "Sdf Cache Read Bind Group",
            read_layout,
            vec![
                pointers.as_entire_binding(),
                wgpu::BindingResource::TextureView(&atlas),
                wgpu::BindingResource::Sampler(&sampler),
                control.as_entire_binding(),
                stamps.as_entire_binding(),
            ],
        );

        let reset_layout = build::bind_layout(device, "Sdf Reset Bind Group Layout", &writes(3));
        let reset_group = build::binding(
            device,
            "Sdf Reset Bind Group",
            &reset_layout,
            vec![
                slots.as_entire_binding(),
                control.as_entire_binding(),
                identities.as_entire_binding(),
            ],
        );
        let reset_shader = composer.compile_wgsl(
            device,
            "sdf_reset.wgsl",
            include_str!("../shaders/sdf_reset.wgsl"),
        );
        let reset_pipeline = compute_pipeline(
            device,
            "Sdf Reset Pipeline",
            &[Some(&reset_layout)],
            &reset_shader,
        );

        let mark_layout = build::bind_layout(
            device,
            "Sdf Mark Bind Group Layout",
            &[
                build::storage_at(0, true, STAGES),
                build::uniform_at(1, STAGES),
            ],
        );
        let mark_group = build::binding(
            device,
            "Sdf Mark Bind Group",
            &mark_layout,
            vec![stir.as_entire_binding(), moved.as_entire_binding()],
        );
        let mark_shader = composer.compile_wgsl(
            device,
            "sdf_mark.wgsl",
            include_str!("../shaders/sdf_mark.wgsl"),
        );
        let mark_pipeline = compute_pipeline(
            device,
            "Sdf Mark Pipeline",
            &[Some(shared_layout), Some(&mark_layout)],
            &mark_shader,
        );

        let mut scan_entries = writes(5);
        scan_entries.push(build::uniform_at(5, STAGES));
        let scan_layout = build::bind_layout(device, "Sdf Scan Bind Group Layout", &scan_entries);
        let scan_group = build::binding(
            device,
            "Sdf Scan Bind Group",
            &scan_layout,
            vec![
                pointers.as_entire_binding(),
                identities.as_entire_binding(),
                stir.as_entire_binding(),
                control.as_entire_binding(),
                work.as_entire_binding(),
                moved.as_entire_binding(),
            ],
        );
        let scan_shader = composer.compile_wgsl(
            device,
            "sdf_scan.wgsl",
            include_str!("../shaders/sdf_scan.wgsl"),
        );
        let scan_pipeline = compute_pipeline(
            device,
            "Sdf Scan Pipeline",
            &[Some(shared_layout), Some(&scan_layout)],
            &scan_shader,
        );

        let work_layout = build::bind_layout(device, "Sdf Work Bind Group Layout", &writes(3));
        let work_group = build::binding(
            device,
            "Sdf Work Bind Group",
            &work_layout,
            vec![
                control.as_entire_binding(),
                work.as_entire_binding(),
                dispatch.as_entire_binding(),
            ],
        );
        let work_shader = composer.compile_wgsl(
            device,
            "sdf_work.wgsl",
            include_str!("../shaders/sdf_work.wgsl"),
        );
        let work_pipeline = compute_pipeline(
            device,
            "Sdf Work Pipeline",
            &[Some(&work_layout)],
            &work_shader,
        );

        let survey_layout = build::bind_layout(device, "Sdf Survey Bind Group Layout", &writes(6));
        let survey_group = build::binding(
            device,
            "Sdf Survey Bind Group",
            &survey_layout,
            vec![
                pointers.as_entire_binding(),
                slots.as_entire_binding(),
                control.as_entire_binding(),
                bricks.as_entire_binding(),
                identities.as_entire_binding(),
                work.as_entire_binding(),
            ],
        );
        let survey_shader = composer.compile_wgsl(
            device,
            "sdf_survey.wgsl",
            include_str!("../shaders/sdf_survey.wgsl"),
        );
        let survey_pipeline = compute_pipeline(
            device,
            "Sdf Survey Pipeline",
            &[Some(shared_layout), Some(&survey_layout)],
            &survey_shader,
        );

        let allocate_layout =
            build::bind_layout(device, "Sdf Allocate Bind Group Layout", &writes(5));
        let allocate_group = build::binding(
            device,
            "Sdf Allocate Bind Group",
            &allocate_layout,
            vec![
                pointers.as_entire_binding(),
                slots.as_entire_binding(),
                control.as_entire_binding(),
                bricks.as_entire_binding(),
                work.as_entire_binding(),
            ],
        );
        let allocate_shader = composer.compile_wgsl(
            device,
            "sdf_allocate.wgsl",
            include_str!("../shaders/sdf_allocate.wgsl"),
        );
        let allocate_pipeline = compute_pipeline(
            device,
            "Sdf Allocate Pipeline",
            &[Some(shared_layout), Some(&allocate_layout)],
            &allocate_shader,
        );

        let indirect_layout =
            build::bind_layout(device, "Sdf Indirect Bind Group Layout", &writes(3));
        let indirect_group = build::binding(
            device,
            "Sdf Indirect Bind Group",
            &indirect_layout,
            vec![
                control.as_entire_binding(),
                slots.as_entire_binding(),
                dispatch.as_entire_binding(),
            ],
        );
        let indirect_shader = composer.compile_wgsl(
            device,
            "sdf_indirect.wgsl",
            include_str!("../shaders/sdf_indirect.wgsl"),
        );
        let indirect_pipeline = compute_pipeline(
            device,
            "Sdf Indirect Pipeline",
            &[Some(&indirect_layout)],
            &indirect_shader,
        );

        let bake_layout = build::bind_layout(
            device,
            "Sdf Bake Bind Group Layout",
            &[
                build::storage_at(0, false, STAGES),
                build::storage_at(1, false, STAGES),
                build::storage_texture_at(
                    2,
                    wgpu::TextureFormat::Rgba8Unorm,
                    wgpu::StorageTextureAccess::WriteOnly,
                    wgpu::TextureViewDimension::D3,
                    wgpu::ShaderStages::COMPUTE,
                ),
                build::storage_at(3, false, STAGES),
                build::storage_at(4, true, STAGES),
            ],
        );
        let bake_group = build::binding(
            device,
            "Sdf Bake Bind Group",
            &bake_layout,
            vec![
                slots.as_entire_binding(),
                bricks.as_entire_binding(),
                wgpu::BindingResource::TextureView(&atlas),
                control.as_entire_binding(),
                stamps.as_entire_binding(),
            ],
        );
        let bake_shader = composer.compile_wgsl(
            device,
            "sdf_bake.wgsl",
            include_str!("../shaders/sdf_bake.wgsl"),
        );
        let bake_pipeline = compute_pipeline(
            device,
            "Sdf Bake Pipeline",
            &[Some(shared_layout), Some(&bake_layout)],
            &bake_shader,
        );

        let cast_layout = build::bind_layout(
            device,
            "Sdf Shadow Cast Bind Group Layout",
            &[build::storage_texture_at(
                0,
                wgpu::TextureFormat::R32Float,
                wgpu::StorageTextureAccess::WriteOnly,
                wgpu::TextureViewDimension::D2,
                wgpu::ShaderStages::COMPUTE,
            )],
        );
        let cast_group = build::binding(
            device,
            "Sdf Shadow Cast Bind Group",
            &cast_layout,
            vec![wgpu::BindingResource::TextureView(&cast)],
        );
        let cast_shader = composer.compile_wgsl(
            device,
            "sdf_shadow_cast.wgsl",
            include_str!("../shaders/sdf_shadow_cast.wgsl"),
        );
        let cast_pipeline = compute_pipeline(
            device,
            "Sdf Shadow Cast Pipeline",
            &[Some(shared_layout), Some(read_layout), Some(&cast_layout)],
            &cast_shader,
        );

        let write_layout = build::bind_layout(
            device,
            "Sdf Shadow Write Bind Group Layout",
            &[build::texture_at(
                0,
                wgpu::TextureSampleType::Float { filterable: false },
                wgpu::TextureViewDimension::D2,
                wgpu::ShaderStages::FRAGMENT,
            )],
        );
        let write_group = build::binding(
            device,
            "Sdf Shadow Write Bind Group",
            &write_layout,
            vec![wgpu::BindingResource::TextureView(&cast)],
        );
        let write_shader = composer.compile_wgsl(
            device,
            "sdf_shadow_write.wgsl",
            include_str!("../shaders/sdf_shadow_write.wgsl"),
        );
        let write_pipeline_layout = build::pipeline_layout(
            device,
            "Sdf Shadow Write Pipeline Layout",
            &[shared_layout, &write_layout],
        );
        let write_pipeline = build::Pipe::new(device, "Sdf Shadow Write Pipeline", &write_shader)
            .layout(&write_pipeline_layout)
            .targets(&[])
            .depth(build::depth_state(wgpu::TextureFormat::Depth32Float, true))
            .vertex("vs_main")
            .fragment("fs_main")
            .make();

        let clipmap = SdfClipmap::default();
        Self {
            uniforms,
            edits,
            tree,
            motions,
            edit_capacity,
            moved,
            pointers,
            dispatch,
            cast_size,
            cast_view: cast,
            shared_layout: shared_layout.clone(),
            shared_group,
            read_group,
            reset_group,
            mark_group,
            scan_group,
            work_group,
            survey_group,
            allocate_group,
            indirect_group,
            bake_group,
            cast_group,
            write_group,
            reset_pipeline,
            mark_pipeline,
            scan_pipeline,
            work_pipeline,
            survey_pipeline,
            allocate_pipeline,
            indirect_pipeline,
            bake_pipeline,
            cast_pipeline,
            write_pipeline,
            edit_data: Vec::new(),
            edit_entities: Vec::new(),
            previous: HashMap::new(),
            motion_data: Vec::new(),
            moved_data: Vec::new(),
            tree_data,
            levels: clipmap.levels,
            side: clipmap.bricks_per_side,
            finest_cell: clipmap.finest_cell,
            debug: 0,
            scale: 1.0,
            terrain: [1.0, 0.0, 0.0, 0.0],
            frame: 0,
            throw_away: true,
            casts: false,
        }
    }

    pub(crate) fn set_edits(
        &mut self,
        device: &wgpu::Device,
        edits: &[SdfEditInstance],
        clipmap: &SdfClipmap,
        debug: u32,
        scale: f32,
    ) {
        let levels = clipmap.levels.clamp(1, MAX_LEVELS);
        let side = clipmap
            .bricks_per_side
            .clamp(2, MAX_SIDE)
            .next_power_of_two()
            .min(MAX_SIDE);
        let finest_cell = clipmap.finest_cell.max(1.0e-4);
        if levels != self.levels || side != self.side || finest_cell != self.finest_cell {
            self.throw_away = true;
        }
        self.levels = levels;
        self.side = side;
        self.finest_cell = finest_cell;
        self.debug = debug;
        self.scale = scale;
        self.edit_data.clear();
        self.edit_entities.clear();
        for edit in edits {
            self.edit_data.push(pack_edit(edit));
            self.edit_entities.push(edit.entity);
        }
        self.terrain = [field_doubt(edits), blend_guard(edits), 0.0, 0.0];
        self.grow(device);
    }

    fn grow(&mut self, device: &wgpu::Device) {
        let needed = self.edit_data.len().max(1);
        if needed <= self.edit_capacity {
            return;
        }
        let capacity = needed.next_power_of_two().max(FIRST_EDITS);
        self.edits = storage_buffer(
            device,
            "Sdf Edit Buffer",
            (capacity * std::mem::size_of::<GpuEdit>()) as u64,
            false,
        );
        self.tree_data = vec![GpuNode::default(); tree_nodes(capacity)];
        self.tree = storage_buffer(
            device,
            "Sdf Edit Tree",
            (self.tree_data.len() * std::mem::size_of::<GpuNode>()) as u64,
            false,
        );
        self.motions = storage_buffer(
            device,
            "Sdf Edit Motions",
            (capacity * MOTION_BYTES) as u64,
            false,
        );
        self.shared_group = shared_binding(
            device,
            &self.shared_layout,
            &self.uniforms,
            &self.edits,
            &self.tree,
        );
        self.edit_capacity = capacity;
    }

    pub(crate) fn handles(&self) -> SdfHandles {
        SdfHandles {
            shared_group: self.shared_group.clone(),
            read_group: self.read_group.clone(),
            cast_view: self.cast_view.clone(),
            motions: self.motions.clone(),
            edits: self.edit_data.len() as u32,
        }
    }

    fn gather_moved(&mut self) {
        self.moved_data.clear();
        if self.throw_away {
            return;
        }
        match balls(&self.previous, &self.edit_entities, &self.edit_data) {
            Some(moved) => self.moved_data = moved,
            None => self.throw_away = true,
        }
    }

    fn commit(&mut self) {
        self.previous = self
            .edit_entities
            .iter()
            .zip(&self.edit_data)
            .enumerate()
            .map(|(index, (entity, edit))| (*entity, (index as u32, *edit)))
            .collect();
    }
}

const SDF_CACHE_PASS_DECL: PassDecl = PassDecl {
    name: SDF_CACHE_PASS,
    reads_writes: &["shadow_depth"],
    ..PassDecl::EMPTY
};

fn matrix(columns: &[[f32; 4]; 4]) -> nalgebra_glm::Mat4 {
    nalgebra_glm::Mat4::from(*columns)
}

impl PassNode<RenderInputs> for SdfCachePass {
    fn decl(&self) -> &PassDecl {
        &SDF_CACHE_PASS_DECL
    }

    fn prepare(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, configs: &RenderInputs) {
        if self.edit_data.is_empty() {
            return;
        }
        let Some(view) = configs.scene.render_view.as_ref() else {
            return;
        };
        self.gather_moved();
        self.frame = self.frame.wrapping_add(1);
        queue.write_buffer(&self.edits, 0, bytemuck::cast_slice(&self.edit_data));
        self.motion_data.clear();
        for (entity, now) in self.edit_entities.iter().zip(&self.edit_data) {
            let motion = match self.previous.get(entity) {
                Some((_, was)) => (nalgebra_glm::inverse(&matrix(&was.inverse_transform))
                    * matrix(&now.inverse_transform))
                .into(),
                None => nalgebra_glm::Mat4::identity().into(),
            };
            self.motion_data.push(motion);
        }
        queue.write_buffer(&self.motions, 0, bytemuck::cast_slice(&self.motion_data));
        if !self.moved_data.is_empty() {
            queue.write_buffer(&self.moved, 0, bytemuck::cast_slice(&self.moved_data));
        }
        build_tree(&self.edit_data, &mut self.tree_data);
        queue.write_buffer(&self.tree, 0, bytemuck::cast_slice(&self.tree_data));

        let (width, height) = configs.view.viewport_size.unwrap_or((1920, 1080));
        let (march_width, march_height) = march_size((width, height), self.scale);
        let lighting = configs.scene.render_lighting.as_ref();
        let (sun_direction, sun_color, lit) = match lighting {
            Some(lighting) if lighting.has_directional_light => {
                (lighting.sun_direction, lighting.sun_color, 1.0)
            }
            _ => (
                nalgebra_glm::Vec3::new(0.0, 1.0, 0.0),
                nalgebra_glm::Vec3::new(1.0, 0.98, 0.92),
                0.0,
            ),
        };
        let mut cascade_view_projections = [[[0.0_f32; 4]; 4]; 4];
        let mut cascade_inverse_view_projections = [[[0.0_f32; 4]; 4]; 4];
        let mut cascade_offsets = [[0.0_f32; 4]; 4];
        let mut cascade_splits = [0.0_f32; 4];
        let mut shadow_bias = 0.0;
        let mut shadow_normal_bias = 0.0;
        let mut light_size = 1.0;
        self.casts = false;
        let meshes = configs
            .scene_world
            .query_entities::<(RenderMesh,)>()
            .next()
            .is_some();
        if let Some(lighting) = lighting
            && lighting.has_directional_light
            && lighting.shadows_enabled > 0.5
        {
            self.casts = meshes;
            cascade_view_projections = lighting.cascade_view_projections;
            for (inverse, forward) in cascade_inverse_view_projections
                .iter_mut()
                .zip(&lighting.cascade_view_projections)
            {
                *inverse = nalgebra_glm::inverse(&matrix(forward)).into();
            }
            cascade_offsets = nightshade_wgpu_shading::backend::projection::cascade_atlas_offsets(
                &lighting.cascade_diameters,
            );
            cascade_splits = lighting.cascade_split_distances;
            shadow_bias = lighting.shadow_bias;
            shadow_normal_bias = lighting.shadow_normal_bias;
            light_size = lighting.directional_light_size;
        }
        let uniforms = GpuUniforms {
            view_projection: view.view_projection.into(),
            inverse_view_projection: view.inverse_view_projection.into(),
            previous_view_projection: configs.scene.prev_view_projection,
            view: view.view.into(),
            cascade_view_projections,
            cascade_inverse_view_projections,
            cascade_offsets,
            cascade_splits,
            camera: [
                view.camera_position.x,
                view.camera_position.y,
                view.camera_position.z,
                view.z_near,
            ],
            sun_direction: [sun_direction.x, sun_direction.y, sun_direction.z, lit],
            sun_color: [
                sun_color.x,
                sun_color.y,
                sun_color.z,
                configs.settings.ibl_blend_factor,
            ],
            screen: [
                march_width as f32,
                march_height as f32,
                1.0 / march_width as f32,
                1.0 / march_height as f32,
            ],
            viewport: [
                width as f32,
                height as f32,
                1.0 / width.max(1) as f32,
                1.0 / height.max(1) as f32,
            ],
            counts: [
                self.edit_data.len() as u32,
                self.levels,
                self.side,
                self.debug,
            ],
            flags: [
                self.moved_data.len() as u32,
                ATLAS_BRICKS,
                u32::from(self.casts),
                SHADOW_SHRINK,
            ],
            sizes: [
                self.finest_cell,
                shadow_bias,
                shadow_normal_bias,
                light_size,
            ],
            terrain: self.terrain,
            frame: [self.frame, 0, 0, 0],
        };
        queue.write_buffer(&self.uniforms, 0, bytemuck::bytes_of(&uniforms));
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, RenderInputs>,
    ) -> Result<()> {
        if !context.is_pass_enabled()
            || self.edit_data.is_empty()
            || context.configs.scene.render_view.is_none()
        {
            return Ok(());
        }

        let window = (
            self.side.div_ceil(WINDOW_GROUP),
            self.side.div_ceil(WINDOW_GROUP),
            (self.side * self.levels).div_ceil(WINDOW_GROUP),
        );

        if self.throw_away {
            let pointer_bytes = (MAX_POINTERS as u64) * 4;
            context
                .encoder
                .clear_buffer(&self.pointers, 0, Some(pointer_bytes));
            let mut start = context
                .encoder
                .begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Sdf Reset Pass"),
                    timestamp_writes: None,
                });
            start.set_pipeline(&self.reset_pipeline);
            start.set_bind_group(0, &self.reset_group, &[]);
            start.dispatch_workgroups(MAX_BRICKS.max(MAX_POINTERS).div_ceil(RESET_GROUP), 1, 1);
            drop(start);
        }

        if !self.moved_data.is_empty() {
            let mut mark = context
                .encoder
                .begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Sdf Mark Pass"),
                    timestamp_writes: None,
                });
            mark.set_pipeline(&self.mark_pipeline);
            mark.set_bind_group(0, &self.shared_group, &[]);
            mark.set_bind_group(1, &self.mark_group, &[]);
            mark.dispatch_workgroups(self.moved_data.len() as u32, self.levels, 1);
            drop(mark);
        }

        let mut scan = context
            .encoder
            .begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Sdf Scan Pass"),
                timestamp_writes: None,
            });
        scan.set_pipeline(&self.scan_pipeline);
        scan.set_bind_group(0, &self.shared_group, &[]);
        scan.set_bind_group(1, &self.scan_group, &[]);
        scan.dispatch_workgroups(window.0, window.1, window.2);
        drop(scan);

        let mut work = context
            .encoder
            .begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Sdf Work Pass"),
                timestamp_writes: None,
            });
        work.set_pipeline(&self.work_pipeline);
        work.set_bind_group(0, &self.work_group, &[]);
        work.dispatch_workgroups(1, 1, 1);
        drop(work);

        let mut survey = context
            .encoder
            .begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Sdf Survey Pass"),
                timestamp_writes: None,
            });
        survey.set_pipeline(&self.survey_pipeline);
        survey.set_bind_group(0, &self.shared_group, &[]);
        survey.set_bind_group(1, &self.survey_group, &[]);
        survey.dispatch_workgroups_indirect(&self.dispatch, 0);
        drop(survey);

        let mut allocate = context
            .encoder
            .begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Sdf Allocate Pass"),
                timestamp_writes: None,
            });
        allocate.set_pipeline(&self.allocate_pipeline);
        allocate.set_bind_group(0, &self.shared_group, &[]);
        allocate.set_bind_group(1, &self.allocate_group, &[]);
        allocate.dispatch_workgroups_indirect(&self.dispatch, 0);
        drop(allocate);

        let mut counted = context
            .encoder
            .begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Sdf Dispatch Pass"),
                timestamp_writes: None,
            });
        counted.set_pipeline(&self.indirect_pipeline);
        counted.set_bind_group(0, &self.indirect_group, &[]);
        counted.dispatch_workgroups(1, 1, 1);
        drop(counted);

        let mut bake = context
            .encoder
            .begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Sdf Bake Pass"),
                timestamp_writes: None,
            });
        bake.set_pipeline(&self.bake_pipeline);
        bake.set_bind_group(0, &self.shared_group, &[]);
        bake.set_bind_group(1, &self.bake_group, &[]);
        bake.dispatch_workgroups_indirect(&self.dispatch, (BAKE_ARGUMENTS as u64) * 4);
        drop(bake);

        if self.casts {
            let mut cast = context
                .encoder
                .begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Sdf Shadow Cast Pass"),
                    timestamp_writes: None,
                });
            cast.set_pipeline(&self.cast_pipeline);
            cast.set_bind_group(0, &self.shared_group, &[]);
            cast.set_bind_group(1, &self.read_group, &[]);
            cast.set_bind_group(2, &self.cast_group, &[]);
            let groups = self.cast_size.div_ceil(CAST_GROUP);
            cast.dispatch_workgroups(groups, groups, 1);
            drop(cast);

            let (shadow_view, shadow_load, shadow_store) =
                context.get_depth_attachment("shadow_depth")?;
            let mut write = context
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Sdf Shadow Write Pass"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: shadow_view,
                        depth_ops: Some(wgpu::Operations {
                            load: shadow_load,
                            store: shadow_store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
            write.set_pipeline(&self.write_pipeline);
            write.set_bind_group(0, &self.shared_group, &[]);
            write.set_bind_group(1, &self.write_group, &[]);
            write.draw(0..3, 0..1);
            drop(write);
        }

        self.throw_away = false;
        self.commit();
        Ok(())
    }
}
