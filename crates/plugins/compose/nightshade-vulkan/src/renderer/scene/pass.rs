use crate::renderer::constraints::{self, ConstraintData, Constraints};
use crate::renderer::context::{Context, FRAMES_IN_FLIGHT};
use crate::renderer::grab::{self, Grab};
use crate::renderer::hiz::{self, DepthPyramid};
use crate::renderer::memory::{
    Buffer, create_buffer, destroy_buffer, read_buffer, write_buffer, write_buffer_at,
};
use crate::renderer::pass::{
    Pass, PassRecord, PassSetup, ScopeKind, ScreenTargets, create_shader_module,
};
use crate::renderer::resource::{self, Resource, ResourceStates, Usage};
use crate::renderer::scene::copies::CopySnapshot;
use crate::renderer::scene::fog;
use crate::renderer::scene::geometry::POSE_RUNS;
use crate::renderer::scene::objects::{FLAG_SHADOW_CASTER, FLAG_VISIBLE, GpuObject};
use crate::renderer::scene::probes;
use crate::renderer::scene::shadow::{
    CASCADE_COUNT, DEPTH_FORMAT, LOCAL_ATLAS_SIZE, SLOT_SIZE, ShadowAtlas, slot_origin,
};
use ash::vk;
use gpu_allocator::MemoryLocation;
use nightshade_render::data::light_gpu::LightData;
use nightshade_render::data::skinning::GpuSkinData;

pub const SCENE_PASS: &str = "scene_pass";

pub const MAX_LIGHTS: usize = 1024;
pub const MAX_AREA_LIGHTS: usize = 16;

pub const TASK_BATCH: u32 = 32;

const INITIAL_OBJECT_CAPACITY: u64 = 1 << 12;
const INITIAL_WORK_CAPACITY: u64 = 1 << 14;
const INITIAL_BATCH_CAPACITY: u64 = 1 << 6;
const INITIAL_RUN_CAPACITY: u64 = 1 << 8;
const INITIAL_POSED_CAPACITY: u64 = 1 << 12;
const INITIAL_INSTANCE_CAPACITY: u64 = 1 << 12;
const INITIAL_WORK_LISTS: u64 = 8 + CASCADE_COUNT as u64;

const LIST_COUNT: usize = 8 + CASCADE_COUNT + LOCAL_SHADOW_COUNT;
pub const LOCAL_SHADOW_COUNT: usize = 1024;
const LIST_OPAQUE: u32 = 0;
const LIST_DOUBLE_SIDED: u32 = 1;
const LIST_REFRACTED: u32 = 2;
const LIST_TRANSPARENT: u32 = 3;
const LIST_OVERLAY_BASE: u32 = 4;
const SETTLING_LISTS: usize = 4;
const SOLID_LISTS: usize = 2;
const LIST_CASCADE_BASE: u32 = 8;
const LIST_LOCAL_BASE: u32 = LIST_CASCADE_BASE + CASCADE_COUNT as u32;
const STORAGE_BINDINGS: u32 = 23;
const PYRAMID_BINDING: u32 = 24;
const GRAB_BINDING: u32 = 25;
const LOCAL_SHADOWS_BINDING: u32 = 26;
const RUN_DRAWS_BINDING: u32 = 27;
const RUN_COUNTS_BINDING: u32 = 28;
const INDICES_BINDING: u32 = 29;
const POSED_MESHLETS_BINDING: u32 = 30;
const FURTHER_BINDINGS: u32 = 5;
const FOG_SCATTER_A_BINDING: u32 = 31;
const FOG_MARCHED_BINDING: u32 = 32;
const FOG_VOLUME_BINDING: u32 = 33;
const FOG_SCATTER_B_BINDING: u32 = 34;
const FOG_HISTORY_A_BINDING: u32 = 35;
const FOG_HISTORY_B_BINDING: u32 = 36;
const PROBE_IRRADIANCE_STORE_BINDING: u32 = 37;
const PROBE_DISTANCE_STORE_BINDING: u32 = 38;
const PROBE_IRRADIANCE_BINDING: u32 = 39;
const PROBE_DISTANCE_BINDING: u32 = 40;
const PROBE_RAYS_BINDING: u32 = 41;
const PROBE_OFFSETS_BINDING: u32 = 42;
const DRAWN_COUNTS_BINDING: u32 = 43;
const RUN_DRAW_SIZE: u64 = std::mem::size_of::<vk::DrawIndexedIndirectCommand>() as u64;

const CLUSTER_CELLS: u32 = nightshade_render::data::config::CLUSTER_COUNT;
const CLUSTER_GROUP_SIZE: u32 = 64;

const CLUSTER_LIST_CAPACITY: u32 = CLUSTER_CELLS * 64;
const CLUSTER_WORDS: u32 = 1 + 2 * CLUSTER_CELLS + CLUSTER_LIST_CAPACITY;

const INITIAL_BONE_CAPACITY: u64 = 1 << 12;
const INITIAL_SKIN_CAPACITY: u64 = 1 << 8;
const SKIN_GROUP_SIZE: u32 = 64;
const CULL_GROUP_SIZE: u32 = 64;

const PHASE_EARLY: u32 = 0;
const PHASE_LATE: u32 = 1;
const PHASE_SHADE: u32 = 2;

const DRAWN_WORDS: usize = 2;
const DRAWN_BYTES: u64 = (DRAWN_WORDS * std::mem::size_of::<u32>()) as u64;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuFrame {
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
    pub view_projection: [[f32; 4]; 4],
    pub camera_position: [f32; 4],
    pub frustum_planes: [[f32; 4]; 6],
    pub counts: [u32; 4],
    pub settings: [f32; 4],
    pub cascade_view_projections: [[[f32; 4]; 4]; CASCADE_COUNT],
    pub cascade_planes: [[[f32; 4]; 6]; CASCADE_COUNT],
    pub cascade_splits: [f32; 4],
    pub cascade_texel_sizes: [f32; 4],
    pub shadow_settings: [f32; 4],
    pub sun_shadow: [f32; 4],
    pub previous_view_projection: [[f32; 4]; 4],
    pub environment: [f32; 4],
    pub skin_counts: [u32; 4],
    pub masks: [u32; 4],
    pub debug: [f32; 4],
    pub flat_color: [f32; 4],
    pub cluster: [f32; 4],
    pub screen: [f32; 4],
    pub pyramid: [f32; 4],
    pub fog_color: [f32; 4],
    pub fog: [f32; 4],
    pub style: [f32; 4],
    pub fog_air: [f32; 4],
    pub local_shadow: [f32; 4],
    pub probe_origin: [f32; 4],
    pub probe_spacing: [f32; 4],
    pub probe_counts: [u32; 4],
    pub probe_settings: [f32; 4],
    pub probe_rotation: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuAreaLight {
    pub right: [f32; 4],
    pub up: [f32; 4],
    pub shape: u32,
    pub two_sided: u32,
    pub emissive_layer: u32,
    pub radius: f32,
}

struct RunPipelines {
    solid: vk::Pipeline,
    double_sided: vk::Pipeline,
    blended: vk::Pipeline,
    settled: vk::Pipeline,
    settled_double_sided: vk::Pipeline,
    depth: vk::Pipeline,
    depth_double_sided: vk::Pipeline,
    shadow: vk::Pipeline,
}

impl RunPipelines {
    fn all(&self) -> [vk::Pipeline; 8] {
        [
            self.solid,
            self.double_sided,
            self.blended,
            self.settled,
            self.settled_double_sided,
            self.depth,
            self.depth_double_sided,
            self.shadow,
        ]
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DrawPath {
    Meshed,
    Runs,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuLocalShadow {
    pub view_projection: [[f32; 4]; 4],
    pub planes: [[f32; 4]; 6],
    pub slot: [f32; 4],
    pub light: [f32; 4],
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct SceneHandover {
    pub mesh_ranges: vk::Buffer,
    pub indices: vk::Buffer,
    pub meshlets: vk::Buffer,
    pub meshlet_vertices: vk::Buffer,
    pub meshlet_triangles: vk::Buffer,
    pub vertices: vk::Buffer,
    pub materials: vk::Buffer,
    pub skin_vertices: vk::Buffer,
    pub morph_displacements: vk::Buffer,
    pub textures: vk::DescriptorSet,
    pub shadows: [vk::DescriptorSet; FRAMES_IN_FLIGHT],
    pub environment: [vk::DescriptorSet; FRAMES_IN_FLIGHT],
}

#[derive(Default)]
pub struct SceneData {
    pub objects: Vec<GpuObject>,
    pub copies: CopySnapshot,
    pub lights: Vec<LightData>,
    pub area_lights: Vec<GpuAreaLight>,
    pub local_shadows: Vec<GpuLocalShadow>,
    pub bones: Vec<[[f32; 4]; 4]>,
    pub constraints: ConstraintData,
    pub instance_batches: Vec<GpuInstanceBatch>,
    pub inverse_binds: Vec<[[f32; 4]; 4]>,
    pub skins: Vec<GpuSkinData>,
    pub skin_instances: Vec<GpuSkinInstance>,
    pub joints: u32,
    pub posed_meshlets: u32,
    pub posed_meshlet_width: u32,
    pub frame: GpuFrame,
    pub work_capacity: u32,
    pub run_capacity: u32,
    pub has_overlay: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuInstanceBatch {
    pub parent: [[f32; 4]; 4],
    pub first_object: u32,
    pub count: u32,
    pub first_local: u32,
    pub padding: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuSkinInstance {
    pub source_vertex_offset: u32,
    pub skin_vertex_offset: u32,
    pub posed_vertex_offset: u32,
    pub vertex_count: u32,
    pub joint_offset: u32,
    pub bone_offset: u32,
    pub bone_count: u32,
    pub object_index: u32,
}

struct FrameResources {
    frame: Buffer,
    objects: Buffer,
    staging: Buffer,
    lights: Buffer,
    area_lights: Buffer,
    work_items: Buffer,
    draw_command: Buffer,
    bones: Buffer,
    inverse_binds: Buffer,
    skins: Buffer,
    joint_matrices: Buffer,
    skin_instances: Buffer,
    instance_batches: Buffer,
    instance_locals: Buffer,
    cluster_lights: Buffer,
    visibility: Buffer,
    local_shadows: Buffer,
    run_draws: Buffer,
    run_counts: Buffer,
    run_capacity: u64,
    drawn_counts: Buffer,
    drawn_readback: Buffer,
    posed_meshlets: Buffer,
    posed_capacity: u64,
    cleared: bool,
    copies_written: Option<(u64, u64)>,
    object_capacity: u64,
    batch_capacity: u64,
    instance_capacity: u64,
    bone_capacity: u64,
    skin_capacity: u64,
    work_capacity: u64,
    work_lists: u64,
    set: vk::DescriptorSet,
    bound: Option<SceneHandover>,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct DrawnCounts {
    pub clusters: u32,
    pub triangles: u32,
}

pub struct ScenePass {
    drawn: DrawnCounts,
    descriptor_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    double_sided_pipeline: vk::Pipeline,
    transparent_pipeline: vk::Pipeline,
    composite_pipeline: vk::Pipeline,
    composite_pipeline_layout: vk::PipelineLayout,
    composite_layout: vk::DescriptorSetLayout,
    composite_pool: vk::DescriptorPool,
    composite_sampler: vk::Sampler,
    composite_set: vk::DescriptorSet,
    depth_pipeline: vk::Pipeline,
    depth_double_sided_pipeline: vk::Pipeline,
    settled_pipeline: vk::Pipeline,
    settled_double_sided_pipeline: vk::Pipeline,
    cull_pipeline: vk::Pipeline,
    cluster_pipeline: vk::Pipeline,
    joint_pipeline: vk::Pipeline,
    skin_pipeline: vk::Pipeline,
    bounds_pipeline: vk::Pipeline,
    meshlet_bounds_pipeline: vk::Pipeline,
    instance_pipeline: vk::Pipeline,
    runs: RunPipelines,
    batches_per_dispatch: u32,
    constraints: Constraints,
    shadow_pipeline: vk::Pipeline,
    shadow_view: vk::ImageView,
    local_shadow_view: vk::ImageView,
    descriptor_pool: vk::DescriptorPool,
    frames: Vec<FrameResources>,
    pyramid: Option<DepthPyramid>,
    fog: Option<fog::FogRounds>,
    probes: Option<probes::ProbeRounds>,
    probes_filled: u32,
    probes_standing: [u32; 4],
    settled_view_projection: [[f32; 4]; 4],
    pose_run: u32,
    grab: Option<Grab>,
    handover: SceneHandover,
    data: SceneData,
}

pub fn create_pass(
    setup: &PassSetup<'_>,
    texture_layout: vk::DescriptorSetLayout,
    shadows: &ShadowAtlas,
    environment_layout: vk::DescriptorSetLayout,
    air: &fog::Air,
    grid: &probes::Probes,
) -> Result<ScenePass, Box<dyn std::error::Error>> {
    let context = setup.context;
    let descriptor_layout = create_descriptor_layout(context)?;
    let pipeline_layout = create_pipeline_layout(
        context,
        descriptor_layout,
        texture_layout,
        shadows.layout,
        environment_layout,
    )?;
    let pipeline = create_pipeline(
        setup,
        pipeline_layout,
        Rasterizing::BackFaceCull,
        DrawPath::Meshed,
    )?;
    let double_sided_pipeline = create_pipeline(
        setup,
        pipeline_layout,
        Rasterizing::DoubleSided,
        DrawPath::Meshed,
    )?;
    let transparent_pipeline = create_pipeline(
        setup,
        pipeline_layout,
        Rasterizing::Blended,
        DrawPath::Meshed,
    )?;
    let composite_layout = create_composite_layout(context)?;
    let composite_pool = create_composite_pool(context)?;
    let composite_sampler = create_composite_sampler(context)?;
    let composite_pipeline_layout = {
        let layouts = [composite_layout];
        let create_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts);
        unsafe { context.device.create_pipeline_layout(&create_info, None)? }
    };
    let composite_pipeline = create_composite_pipeline(setup, composite_pipeline_layout)?;
    let composite_set = {
        let layouts = [composite_layout];
        let allocate_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(composite_pool)
            .set_layouts(&layouts);
        unsafe { context.device.allocate_descriptor_sets(&allocate_info)?[0] }
    };
    write_composite_set(
        context,
        composite_set,
        composite_sampler,
        setup.accumulated,
        setup.revealed,
    );
    let depth_pipeline = create_depth_pipeline(setup, pipeline_layout, false, DrawPath::Meshed)?;
    let depth_double_sided_pipeline =
        create_depth_pipeline(setup, pipeline_layout, true, DrawPath::Meshed)?;
    let settled_pipeline = create_pipeline(
        setup,
        pipeline_layout,
        Rasterizing::Settled,
        DrawPath::Meshed,
    )?;
    let settled_double_sided_pipeline = create_pipeline(
        setup,
        pipeline_layout,
        Rasterizing::SettledDoubleSided,
        DrawPath::Meshed,
    )?;
    let cull_pipeline = create_cull_pipeline(context, pipeline_layout)?;
    let cluster_pipeline = create_compute_pipeline(
        context,
        pipeline_layout,
        include_bytes!("../shaders/light_cluster.comp.spv"),
    )?;
    let joint_pipeline = create_compute_pipeline(
        context,
        pipeline_layout,
        include_bytes!("../shaders/skin_joints.comp.spv"),
    )?;
    let skin_pipeline = create_compute_pipeline(
        context,
        pipeline_layout,
        include_bytes!("../shaders/skin_vertices.comp.spv"),
    )?;
    let shadow_pipeline = create_shadow_pipeline(context, pipeline_layout, DrawPath::Meshed)?;
    let descriptor_pool = create_descriptor_pool(context, setup.frames_in_flight)?;

    let mut frames = Vec::with_capacity(setup.frames_in_flight);
    for _ in 0..setup.frames_in_flight {
        frames.push(create_frame(context, descriptor_pool, descriptor_layout)?);
    }

    let pyramid = match hiz::create(context, setup.extent, setup.depth) {
        Ok(pyramid) => {
            bind_pyramid(context, &frames, &pyramid);
            Some(pyramid)
        }
        Err(error) => {
            tracing::error!("the depth pyramid was not made: {error}");
            None
        }
    };

    let instance_pipeline = create_compute_pipeline(
        context,
        pipeline_layout,
        include_bytes!("../shaders/instance_place.comp.spv"),
    )?;
    let bounds_pipeline = create_compute_pipeline(
        context,
        pipeline_layout,
        include_bytes!("../shaders/skin_bounds.comp.spv"),
    )?;
    let meshlet_bounds_pipeline = create_compute_pipeline(
        context,
        pipeline_layout,
        include_bytes!("../shaders/skin_meshlets.comp.spv"),
    )?;

    let constraints = constraints::create(context, setup.frames_in_flight)?;

    let fog = match create_fog(context, pipeline_layout) {
        Ok(fog) => {
            let sets: Vec<vk::DescriptorSet> = frames.iter().map(|frame| frame.set).collect();
            fog::bind(
                context,
                air,
                &sets,
                fog::Bindings {
                    scatter: [FOG_SCATTER_A_BINDING, FOG_SCATTER_B_BINDING],
                    history: [FOG_HISTORY_A_BINDING, FOG_HISTORY_B_BINDING],
                    marched: FOG_MARCHED_BINDING,
                    volume: FOG_VOLUME_BINDING,
                },
            );
            Some(fog)
        }
        Err(error) => {
            tracing::error!("the lit air was not made: {error}");
            None
        }
    };

    let sets: Vec<vk::DescriptorSet> = frames.iter().map(|frame| frame.set).collect();
    probes::bind(
        context,
        grid,
        &sets,
        probes::Bindings {
            irradiance_store: PROBE_IRRADIANCE_STORE_BINDING,
            distance_store: PROBE_DISTANCE_STORE_BINDING,
            irradiance_read: PROBE_IRRADIANCE_BINDING,
            distance_read: PROBE_DISTANCE_BINDING,
            rays: PROBE_RAYS_BINDING,
            offsets: PROBE_OFFSETS_BINDING,
        },
    );
    let probe_rounds = match create_probe_rounds(context, pipeline_layout) {
        Ok(rounds) => Some(rounds),
        Err(error) => {
            tracing::error!("the probes were not made: {error}");
            None
        }
    };

    let grab = match grab::create(context, setup.extent, setup.color) {
        Ok(grab) => {
            bind_grab(context, &frames, &grab);
            Some(grab)
        }
        Err(error) => {
            tracing::error!("the scene grab was not made: {error}");
            None
        }
    };

    Ok(ScenePass {
        drawn: DrawnCounts::default(),
        descriptor_layout,
        pipeline_layout,
        pipeline,
        double_sided_pipeline,
        transparent_pipeline,
        composite_pipeline,
        composite_pipeline_layout,
        composite_layout,
        composite_pool,
        composite_sampler,
        composite_set,
        depth_pipeline,
        depth_double_sided_pipeline,
        settled_pipeline,
        settled_double_sided_pipeline,
        cull_pipeline,
        cluster_pipeline,
        joint_pipeline,
        skin_pipeline,
        bounds_pipeline,
        meshlet_bounds_pipeline,
        instance_pipeline,
        runs: RunPipelines {
            solid: create_pipeline(
                setup,
                pipeline_layout,
                Rasterizing::BackFaceCull,
                DrawPath::Runs,
            )?,
            double_sided: create_pipeline(
                setup,
                pipeline_layout,
                Rasterizing::DoubleSided,
                DrawPath::Runs,
            )?,
            blended: create_pipeline(setup, pipeline_layout, Rasterizing::Blended, DrawPath::Runs)?,
            settled: create_pipeline(setup, pipeline_layout, Rasterizing::Settled, DrawPath::Runs)?,
            settled_double_sided: create_pipeline(
                setup,
                pipeline_layout,
                Rasterizing::SettledDoubleSided,
                DrawPath::Runs,
            )?,
            depth: create_depth_pipeline(setup, pipeline_layout, false, DrawPath::Runs)?,
            depth_double_sided: create_depth_pipeline(
                setup,
                pipeline_layout,
                true,
                DrawPath::Runs,
            )?,
            shadow: create_shadow_pipeline(context, pipeline_layout, DrawPath::Runs)?,
        },
        batches_per_dispatch: batches_per_dispatch(context),
        constraints,
        shadow_pipeline,
        shadow_view: shadows.view,
        local_shadow_view: shadows.local_view,
        descriptor_pool,
        frames,
        pyramid,
        fog,
        probes: probe_rounds,
        probes_filled: 0,
        probes_standing: [0; 4],
        settled_view_projection: [[0.0; 4]; 4],
        pose_run: 0,
        grab,
        handover: SceneHandover::default(),
        data: SceneData::default(),
    })
}

fn create_probe_rounds(
    context: &Context,
    layout: vk::PipelineLayout,
) -> Result<probes::ProbeRounds, Box<dyn std::error::Error>> {
    if context.acceleration.is_none() {
        return Err("the device cannot trace, so the probes send no rays".into());
    }
    let offsets = create_compute_pipeline(
        context,
        layout,
        include_bytes!("../shaders/probe_offsets.comp.spv"),
    )?;
    let trace = match create_compute_pipeline(
        context,
        layout,
        include_bytes!("../shaders/probe_trace.comp.spv"),
    ) {
        Ok(pipeline) => pipeline,
        Err(error) => {
            unsafe { context.device.destroy_pipeline(offsets, None) };
            return Err(error);
        }
    };
    let irradiance = match create_compute_pipeline(
        context,
        layout,
        include_bytes!("../shaders/probe_irradiance.comp.spv"),
    ) {
        Ok(pipeline) => pipeline,
        Err(error) => unsafe {
            context.device.destroy_pipeline(offsets, None);
            context.device.destroy_pipeline(trace, None);
            return Err(error);
        },
    };
    let distance = match create_compute_pipeline(
        context,
        layout,
        include_bytes!("../shaders/probe_distance.comp.spv"),
    ) {
        Ok(pipeline) => pipeline,
        Err(error) => unsafe {
            context.device.destroy_pipeline(offsets, None);
            context.device.destroy_pipeline(trace, None);
            context.device.destroy_pipeline(irradiance, None);
            return Err(error);
        },
    };
    Ok(probes::create_rounds(offsets, trace, irradiance, distance))
}

fn create_fog(
    context: &Context,
    layout: vk::PipelineLayout,
) -> Result<fog::FogRounds, Box<dyn std::error::Error>> {
    let scatter = create_compute_pipeline(
        context,
        layout,
        include_bytes!("../shaders/fog_scatter.comp.spv"),
    )?;
    let march = match create_compute_pipeline(
        context,
        layout,
        include_bytes!("../shaders/fog_march.comp.spv"),
    ) {
        Ok(march) => march,
        Err(error) => {
            unsafe { context.device.destroy_pipeline(scatter, None) };
            return Err(error);
        }
    };
    Ok(fog::create_rounds(scatter, march))
}

fn bind_pyramid(context: &Context, frames: &[FrameResources], pyramid: &DepthPyramid) {
    let info = vk::DescriptorImageInfo::default()
        .image_view(hiz::view(pyramid))
        .sampler(hiz::sampler(pyramid))
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
    let writes: Vec<vk::WriteDescriptorSet> = frames
        .iter()
        .map(|frame| {
            vk::WriteDescriptorSet::default()
                .dst_set(frame.set)
                .dst_binding(PYRAMID_BINDING)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&info))
        })
        .collect();
    unsafe { context.device.update_descriptor_sets(&writes, &[]) };
}

fn bind_grab(context: &Context, frames: &[FrameResources], grab: &Grab) {
    let info = vk::DescriptorImageInfo::default()
        .image_view(grab::view(grab))
        .sampler(grab::sampler(grab))
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
    let writes: Vec<vk::WriteDescriptorSet> = frames
        .iter()
        .map(|frame| {
            vk::WriteDescriptorSet::default()
                .dst_set(frame.set)
                .dst_binding(GRAB_BINDING)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&info))
        })
        .collect();
    unsafe { context.device.update_descriptor_sets(&writes, &[]) };
}

pub struct PosedRun {
    pub mesh_slot: u32,
    pub vertex_offset: u32,
    pub vertex_count: u32,
    pub object_index: u32,
    pub material_index: u32,
    pub casts_shadow: bool,
}

pub fn drawn_counts(pass: &ScenePass) -> DrawnCounts {
    pass.drawn
}

pub fn posed_runs(pass: &ScenePass) -> Vec<PosedRun> {
    pass.data
        .skin_instances
        .iter()
        .filter_map(|instance| {
            let object = pass.data.objects.get(instance.object_index as usize)?;
            if object.flags & FLAG_VISIBLE == 0 {
                return None;
            }
            Some(PosedRun {
                mesh_slot: object.mesh_slot,
                vertex_offset: object.vertex_offset,
                vertex_count: instance.vertex_count,
                object_index: instance.object_index,
                material_index: object.material_index,
                casts_shadow: object.flags & FLAG_SHADOW_CASTER != 0,
            })
        })
        .collect()
}

pub fn traced_inputs(pass: &ScenePass) -> (vk::DescriptorSetLayout, Vec<vk::DescriptorSet>) {
    (
        pass.descriptor_layout,
        pass.frames.iter().map(|frame| frame.set).collect(),
    )
}

pub fn hand_over(
    pass: &mut ScenePass,
    handover: SceneHandover,
    mut data: SceneData,
    unjittered: [[f32; 4]; 4],
) {
    let seen_before = carry_previous_places(&pass.data.objects, &mut data.objects);
    pass.pose_run = (pass.pose_run + 1) % POSE_RUNS;
    settle_pose_runs(&mut data, pass.pose_run, &seen_before);
    if pass.data.frame.counts[0] > 0 {
        data.frame.previous_view_projection = pass.settled_view_projection;
    }
    pass.settled_view_projection = unjittered;
    pass.handover = handover;
    pass.data = data;
}

fn carry_previous_places(before: &[GpuObject], now: &mut [GpuObject]) -> Vec<bool> {
    let mut rows: std::collections::HashMap<u32, usize> =
        std::collections::HashMap::with_capacity(before.len());
    for (index, object) in before.iter().enumerate() {
        rows.insert(object.entity_id, index);
    }
    let mut seen = Vec::with_capacity(now.len());
    for object in now.iter_mut() {
        let was = rows
            .get(&object.entity_id)
            .and_then(|index| before.get(*index));
        object.previous_transform = match was {
            Some(was) => was.transform,
            None => object.transform,
        };
        seen.push(was.is_some());
    }
    seen
}

fn settle_pose_runs(data: &mut SceneData, run: u32, seen_before: &[bool]) {
    for instance in &mut data.skin_instances {
        let reserved = instance.posed_vertex_offset;
        let stride = instance.vertex_count;
        let now = reserved + run * stride;
        let before = reserved + ((run + POSE_RUNS - 1) % POSE_RUNS) * stride;
        instance.posed_vertex_offset = now;
        let Some(object) = data.objects.get_mut(instance.object_index as usize) else {
            continue;
        };
        object.vertex_offset = now;
        object.previous_vertex_offset = match seen_before.get(instance.object_index as usize) {
            Some(true) => before,
            _ => now,
        };
    }
}

impl ScenePass {
    fn take_the_count(&mut self, frame_index: usize) {
        let Some(frame) = self.frames.get(frame_index) else {
            return;
        };
        let words: Vec<u32> = read_buffer(&frame.drawn_readback, 0, DRAWN_WORDS);
        if words.len() == DRAWN_WORDS {
            self.drawn = DrawnCounts {
                clusters: words[0],
                triangles: words[1],
            };
        }
    }

    fn draw_list(
        &self,
        record: &PassRecord,
        frame: &FrameResources,
        list: u32,
        pipeline: vk::Pipeline,
        phase: u32,
    ) {
        let device = &record.context.device;
        unsafe {
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline,
            );
            push_list(
                device,
                record.command_buffer,
                self.pipeline_layout,
                list,
                phase,
            );
            record.context.mesh_shader.cmd_draw_mesh_tasks_indirect(
                record.command_buffer,
                frame.draw_command.buffer,
                (list as usize * std::mem::size_of::<[u32; 4]>()) as u64,
                1,
                std::mem::size_of::<[u32; 4]>() as u32,
            );
        }
        self.draw_runs(record, frame, list, self.run_twin(pipeline));
    }

    fn draw_runs(
        &self,
        record: &PassRecord,
        frame: &FrameResources,
        list: u32,
        pipeline: vk::Pipeline,
    ) {
        let capacity = self.data.run_capacity as u64;
        if capacity == 0 {
            return;
        }
        let device = &record.context.device;
        unsafe {
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline,
            );
            device.cmd_bind_index_buffer(
                record.command_buffer,
                self.handover.indices,
                0,
                vk::IndexType::UINT32,
            );
            device.cmd_draw_indexed_indirect_count(
                record.command_buffer,
                frame.run_draws.buffer,
                list as u64 * capacity * RUN_DRAW_SIZE,
                frame.run_counts.buffer,
                list as u64 * std::mem::size_of::<u32>() as u64,
                capacity as u32,
                RUN_DRAW_SIZE as u32,
            );
        }
    }

    fn run_twin(&self, meshed: vk::Pipeline) -> vk::Pipeline {
        let runs = &self.runs;
        if meshed == self.pipeline {
            runs.solid
        } else if meshed == self.double_sided_pipeline {
            runs.double_sided
        } else if meshed == self.transparent_pipeline {
            runs.blended
        } else if meshed == self.settled_pipeline {
            runs.settled
        } else if meshed == self.settled_double_sided_pipeline {
            runs.settled_double_sided
        } else if meshed == self.depth_pipeline {
            runs.depth
        } else if meshed == self.depth_double_sided_pipeline {
            runs.depth_double_sided
        } else {
            runs.shadow
        }
    }

    fn draw_shadows(&mut self, record: &mut PassRecord) {
        let device = record.context.device.clone();
        record.require(Resource::ShadowAtlas, Usage::DepthAttachment);

        let local_count = self.local_shadow_count();
        if local_count > 0 {
            record.require(Resource::LocalShadowAtlas, Usage::DepthAttachment);
        }
        let atlas = LOCAL_ATLAS_SIZE as f32;
        let placed: Vec<((u32, u32), u32)> = self
            .data
            .local_shadows
            .iter()
            .map(|shadow| {
                (
                    (
                        (shadow.slot[0] * atlas).round() as u32,
                        (shadow.slot[1] * atlas).round() as u32,
                    ),
                    (shadow.slot[2] * atlas).round() as u32,
                )
            })
            .collect();
        for (slot, (origin, size)) in placed.into_iter().enumerate() {
            self.draw_shadow_slot(
                record,
                &device,
                self.local_shadow_view,
                origin,
                size,
                LIST_LOCAL_BASE + slot as u32,
            );
        }
        if local_count > 0 {
            record.require(Resource::LocalShadowAtlas, Usage::Sampled);
        }

        for cascade in 0..CASCADE_COUNT {
            let (x, y) = slot_origin(cascade);
            self.draw_shadow_slot(
                record,
                &device,
                self.shadow_view,
                (x, y),
                SLOT_SIZE,
                LIST_CASCADE_BASE + cascade as u32,
            );
        }

        record.require(Resource::ShadowAtlas, Usage::SampledByComputeAndFragment);
    }

    fn fill_the_probes(&mut self, record: &mut PassRecord) {
        let Some(rounds) = &self.probes else {
            return;
        };
        if self.data.frame.probe_counts[3] == 0 {
            return;
        }
        let counts = self.data.frame.probe_counts;
        let standing = [
            counts[0],
            counts[1],
            counts[2],
            self.data.frame.probe_origin[0].to_bits(),
        ];
        if standing != self.probes_standing {
            self.probes_standing = standing;
            self.probes_filled = 0;
        }
        let probe_count = counts[0].max(1) * counts[1].max(1) * counts[2].max(1);
        let frame_index = record.frame_index % self.frames.len();
        record.require(
            Resource::LocalShadowAtlas,
            Usage::SampledByComputeAndFragment,
        );
        record.require(Resource::ShadowAtlas, Usage::SampledByComputeAndFragment);
        probes::update_probes(
            rounds,
            record.probes,
            record.context,
            record.command_buffer,
            self.pipeline_layout,
            [
                self.frames[frame_index].set,
                self.handover.textures,
                self.handover.shadows[frame_index % FRAMES_IN_FLIGHT],
                self.handover.environment[frame_index % FRAMES_IN_FLIGHT],
            ],
            probe_count.min(probes::max_probe_count()),
            self.probes_filled,
        );
        self.probes_filled = self.probes_filled.saturating_add(1);
    }

    fn light_the_air(&self, record: &mut PassRecord) {
        let Some(fog) = &self.fog else {
            return;
        };
        if self.data.frame.fog[2] == 0.0 {
            return;
        }
        let frame_index = record.frame_index % self.frames.len();
        record.require(
            Resource::LocalShadowAtlas,
            Usage::SampledByComputeAndFragment,
        );
        fog::light_the_air(
            fog,
            record.air,
            record.context,
            record.command_buffer,
            self.pipeline_layout,
            [
                self.frames[frame_index].set,
                self.handover.textures,
                self.handover.shadows[frame_index % FRAMES_IN_FLIGHT],
                self.handover.environment[frame_index % FRAMES_IN_FLIGHT],
            ],
            (self.data.frame.settings[3] as usize) & 1,
        );
    }

    fn ready(&self) -> bool {
        self.data.object_count() > 0 && self.handover.vertices != vk::Buffer::null()
    }

    fn place_instances(&self, record: &PassRecord, frame_index: usize) {
        let widest = self
            .data
            .instance_batches
            .iter()
            .map(|batch| batch.count)
            .max()
            .unwrap_or(0);
        if widest == 0 {
            return;
        }
        let frame = &self.frames[frame_index];
        let device = &record.context.device;
        unsafe {
            device.cmd_bind_descriptor_sets(
                record.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.pipeline_layout,
                0,
                &[frame.set],
                &[],
            );
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.instance_pipeline,
            );
            let total = self.data.instance_batches.len() as u32;
            let mut first = 0;
            while first < total {
                let count = (total - first).min(self.batches_per_dispatch);
                push_list(
                    device,
                    record.command_buffer,
                    self.pipeline_layout,
                    first,
                    0,
                );
                device.cmd_dispatch(
                    record.command_buffer,
                    widest.div_ceil(SKIN_GROUP_SIZE),
                    count,
                    1,
                );
                first += count;
            }
            barrier(
                device,
                record.command_buffer,
                frame.objects.buffer,
                vk::PipelineStageFlags2::COMPUTE_SHADER,
                vk::AccessFlags2::SHADER_STORAGE_WRITE,
                REACHED_AFTER,
                vk::AccessFlags2::SHADER_STORAGE_READ,
            );
        }
    }

    fn pose_skins(&self, record: &PassRecord, frame_index: usize) {
        if self.data.skin_instances.is_empty() {
            return;
        }
        let frame = &self.frames[frame_index];
        let device = &record.context.device;
        constraints::dispatch(
            record.context,
            &self.constraints,
            record.command_buffer,
            frame.bones.buffer,
            frame_index,
        );
        unsafe {
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.bounds_pipeline,
            );
            device.cmd_bind_descriptor_sets(
                record.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.pipeline_layout,
                0,
                &[frame.set],
                &[],
            );
            device.cmd_dispatch(
                record.command_buffer,
                (self.data.skin_instances.len() as u32).div_ceil(SKIN_GROUP_SIZE),
                1,
                1,
            );
            barrier(
                device,
                record.command_buffer,
                frame.objects.buffer,
                vk::PipelineStageFlags2::COMPUTE_SHADER,
                vk::AccessFlags2::SHADER_STORAGE_WRITE,
                REACHED_AFTER,
                vk::AccessFlags2::SHADER_STORAGE_READ,
            );
        }

        let joints = self.data.frame.skin_counts[0];
        let widest = self
            .data
            .skin_instances
            .iter()
            .map(|instance| instance.vertex_count)
            .max()
            .unwrap_or(0);
        if joints == 0 || widest == 0 {
            return;
        }

        unsafe {
            device.cmd_bind_descriptor_sets(
                record.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.pipeline_layout,
                0,
                &[frame.set],
                &[],
            );
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.joint_pipeline,
            );
            device.cmd_dispatch(
                record.command_buffer,
                joints.div_ceil(SKIN_GROUP_SIZE),
                1,
                1,
            );
            barrier(
                device,
                record.command_buffer,
                frame.joint_matrices.buffer,
                vk::PipelineStageFlags2::COMPUTE_SHADER,
                vk::AccessFlags2::SHADER_STORAGE_WRITE,
                vk::PipelineStageFlags2::COMPUTE_SHADER,
                vk::AccessFlags2::SHADER_STORAGE_READ,
            );
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.skin_pipeline,
            );
            device.cmd_dispatch(
                record.command_buffer,
                widest.div_ceil(SKIN_GROUP_SIZE),
                self.data.skin_instances.len() as u32,
                1,
            );
            barrier(
                device,
                record.command_buffer,
                self.handover.vertices,
                vk::PipelineStageFlags2::COMPUTE_SHADER,
                vk::AccessFlags2::SHADER_STORAGE_WRITE,
                REACHED_AFTER,
                vk::AccessFlags2::SHADER_STORAGE_READ,
            );
            if self.data.posed_meshlet_width > 0 {
                device.cmd_bind_pipeline(
                    record.command_buffer,
                    vk::PipelineBindPoint::COMPUTE,
                    self.meshlet_bounds_pipeline,
                );
                device.cmd_dispatch(
                    record.command_buffer,
                    self.data.posed_meshlet_width,
                    self.data.skin_instances.len() as u32,
                    1,
                );
                barrier(
                    device,
                    record.command_buffer,
                    frame.posed_meshlets.buffer,
                    vk::PipelineStageFlags2::COMPUTE_SHADER,
                    vk::AccessFlags2::SHADER_STORAGE_WRITE,
                    REACHED_AFTER,
                    vk::AccessFlags2::SHADER_STORAGE_READ,
                );
            }
        }
    }

    fn assign_lights(&self, record: &PassRecord, frame_index: usize) {
        let frame = &self.frames[frame_index];
        let device = &record.context.device;
        unsafe {
            device.cmd_update_buffer(
                record.command_buffer,
                frame.cluster_lights.buffer,
                0,
                bytemuck::cast_slice(&[0u32]),
            );
            barrier(
                device,
                record.command_buffer,
                frame.cluster_lights.buffer,
                vk::PipelineStageFlags2::TRANSFER,
                vk::AccessFlags2::TRANSFER_WRITE,
                vk::PipelineStageFlags2::COMPUTE_SHADER,
                vk::AccessFlags2::SHADER_STORAGE_READ | vk::AccessFlags2::SHADER_STORAGE_WRITE,
            );
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.cluster_pipeline,
            );
            device.cmd_bind_descriptor_sets(
                record.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.pipeline_layout,
                0,
                &[frame.set],
                &[],
            );
            device.cmd_dispatch(
                record.command_buffer,
                CLUSTER_CELLS.div_ceil(CLUSTER_GROUP_SIZE),
                1,
                1,
            );
            barrier(
                device,
                record.command_buffer,
                frame.cluster_lights.buffer,
                vk::PipelineStageFlags2::COMPUTE_SHADER,
                vk::AccessFlags2::SHADER_STORAGE_WRITE,
                REACHED_AFTER,
                vk::AccessFlags2::SHADER_STORAGE_READ,
            );
        }
    }

    fn splits(&self) -> bool {
        self.ready() && self.pyramid.is_some()
    }

    fn depth_lists(&self) -> [(u32, vk::Pipeline); SETTLING_LISTS] {
        [
            (LIST_OPAQUE, self.depth_pipeline),
            (LIST_DOUBLE_SIDED, self.depth_double_sided_pipeline),
            (LIST_REFRACTED, self.depth_double_sided_pipeline),
            (LIST_TRANSPARENT, self.depth_double_sided_pipeline),
        ]
    }

    fn solid_lists(&self) -> [(u32, vk::Pipeline); SOLID_LISTS] {
        [
            (LIST_OPAQUE, self.settled_pipeline),
            (LIST_DOUBLE_SIDED, self.settled_double_sided_pipeline),
        ]
    }

    fn cull_solids(&self, record: &PassRecord, frame: &FrameResources, phase: u32) {
        let device = &record.context.device;
        let groups = (self.data.object_count() as u32).div_ceil(CULL_GROUP_SIZE);
        let reset = [[0u32, 0, 1, 0]; SETTLING_LISTS];
        let run_reset = [0u32; SETTLING_LISTS];
        unsafe {
            for buffer in [frame.draw_command.buffer, frame.run_counts.buffer] {
                barrier(
                    device,
                    record.command_buffer,
                    buffer,
                    vk::PipelineStageFlags2::DRAW_INDIRECT,
                    vk::AccessFlags2::INDIRECT_COMMAND_READ,
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::AccessFlags2::TRANSFER_WRITE,
                );
            }
            device.cmd_update_buffer(
                record.command_buffer,
                frame.draw_command.buffer,
                0,
                bytemuck::cast_slice(&reset),
            );
            device.cmd_update_buffer(
                record.command_buffer,
                frame.run_counts.buffer,
                0,
                bytemuck::cast_slice(&run_reset),
            );
            for buffer in [frame.draw_command.buffer, frame.run_counts.buffer] {
                barrier(
                    device,
                    record.command_buffer,
                    buffer,
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::AccessFlags2::TRANSFER_WRITE,
                    vk::PipelineStageFlags2::COMPUTE_SHADER,
                    vk::AccessFlags2::SHADER_STORAGE_READ | vk::AccessFlags2::SHADER_STORAGE_WRITE,
                );
            }
            barrier(
                device,
                record.command_buffer,
                frame.run_draws.buffer,
                vk::PipelineStageFlags2::DRAW_INDIRECT,
                vk::AccessFlags2::INDIRECT_COMMAND_READ,
                vk::PipelineStageFlags2::COMPUTE_SHADER,
                vk::AccessFlags2::SHADER_STORAGE_WRITE,
            );
            barrier(
                device,
                record.command_buffer,
                frame.work_items.buffer,
                REACHED_AFTER,
                vk::AccessFlags2::SHADER_STORAGE_READ,
                vk::PipelineStageFlags2::COMPUTE_SHADER,
                vk::AccessFlags2::SHADER_STORAGE_WRITE,
            );
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.cull_pipeline,
            );
            device.cmd_bind_descriptor_sets(
                record.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.pipeline_layout,
                0,
                &[frame.set],
                &[],
            );
            for list in 0..SETTLING_LISTS as u32 {
                push_list(
                    device,
                    record.command_buffer,
                    self.pipeline_layout,
                    list,
                    phase,
                );
                device.cmd_dispatch(record.command_buffer, groups, 1, 1);
            }
            for buffer in [
                frame.draw_command.buffer,
                frame.run_draws.buffer,
                frame.run_counts.buffer,
            ] {
                barrier(
                    device,
                    record.command_buffer,
                    buffer,
                    vk::PipelineStageFlags2::COMPUTE_SHADER,
                    vk::AccessFlags2::SHADER_STORAGE_WRITE,
                    REACHED_AFTER,
                    vk::AccessFlags2::INDIRECT_COMMAND_READ,
                );
            }
            barrier(
                device,
                record.command_buffer,
                frame.work_items.buffer,
                vk::PipelineStageFlags2::COMPUTE_SHADER,
                vk::AccessFlags2::SHADER_STORAGE_WRITE,
                REACHED_AFTER,
                vk::AccessFlags2::SHADER_STORAGE_READ,
            );
        }
    }

    fn bind_graphics(&self, record: &PassRecord, frame: &FrameResources, frame_index: usize) {
        unsafe {
            record.context.device.cmd_bind_descriptor_sets(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &[
                    frame.set,
                    self.handover.textures,
                    self.handover.shadows[frame_index % FRAMES_IN_FLIGHT],
                    self.handover.environment[frame_index % FRAMES_IN_FLIGHT],
                ],
                &[],
            );
        }
    }

    fn put_blended_back(&self, record: &PassRecord) {
        let device = &record.context.device;
        unsafe {
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.composite_pipeline,
            );
            device.cmd_bind_descriptor_sets(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.composite_pipeline_layout,
                0,
                &[self.composite_set],
                &[],
            );
            device.cmd_draw(record.command_buffer, 3, 1, 0, 0);
        }
    }

    fn draw_over_solids(&self, record: &PassRecord, frame: &FrameResources) {
        let device = &record.context.device;
        self.put_blended_back(record);
        if !self.data.has_overlay {
            return;
        }
        clear_depth(device, record.command_buffer, record.extent);
        let overlay = [
            (LIST_OVERLAY_BASE + LIST_OPAQUE, self.pipeline),
            (
                LIST_OVERLAY_BASE + LIST_DOUBLE_SIDED,
                self.double_sided_pipeline,
            ),
            (
                LIST_OVERLAY_BASE + LIST_REFRACTED,
                self.double_sided_pipeline,
            ),
            (
                LIST_OVERLAY_BASE + LIST_TRANSPARENT,
                self.transparent_pipeline,
            ),
        ];
        for (list, pipeline) in overlay {
            self.draw_list(record, frame, list, pipeline, PHASE_EARLY);
        }
    }

    fn draw_shadow_slot(
        &self,
        record: &PassRecord,
        device: &ash::Device,
        atlas: vk::ImageView,
        origin: (u32, u32),
        size: u32,
        list: u32,
    ) {
        let frame_index = record.frame_index % self.frames.len();
        let frame = &self.frames[frame_index];
        let area = vk::Rect2D {
            offset: vk::Offset2D {
                x: origin.0 as i32,
                y: origin.1 as i32,
            },
            extent: vk::Extent2D {
                width: size,
                height: size,
            },
        };
        let depth_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(atlas)
            .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 0.0,
                    stencil: 0,
                },
            });
        let rendering = vk::RenderingInfo::default()
            .render_area(area)
            .layer_count(1)
            .depth_attachment(&depth_attachment);
        let viewport = vk::Viewport {
            x: origin.0 as f32,
            y: origin.1 as f32,
            width: size as f32,
            height: size as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        unsafe {
            device.cmd_begin_rendering(record.command_buffer, &rendering);
            device.cmd_set_viewport(record.command_buffer, 0, std::slice::from_ref(&viewport));
            device.cmd_set_scissor(record.command_buffer, 0, std::slice::from_ref(&area));
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.shadow_pipeline,
            );
            device.cmd_bind_descriptor_sets(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &[frame.set, self.handover.textures],
                &[],
            );
            push_list(
                device,
                record.command_buffer,
                self.pipeline_layout,
                list,
                PHASE_EARLY,
            );
            record.context.mesh_shader.cmd_draw_mesh_tasks_indirect(
                record.command_buffer,
                frame.draw_command.buffer,
                (list as usize * std::mem::size_of::<[u32; 4]>()) as u64,
                1,
                std::mem::size_of::<[u32; 4]>() as u32,
            );
            self.draw_runs(record, frame, list, self.runs.shadow);
            device.cmd_end_rendering(record.command_buffer);
        }
    }

    fn list_count(&self) -> u32 {
        if self.data.frame.shadow_settings[2] == 0.0 {
            LIST_CASCADE_BASE
        } else {
            LIST_LOCAL_BASE + self.local_shadow_count()
        }
    }

    fn local_shadow_count(&self) -> u32 {
        self.data.local_shadows.len() as u32
    }

    fn stage_objects(
        &mut self,
        context: &Context,
        command_buffer: vk::CommandBuffer,
        frame_index: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let object_size = std::mem::size_of::<GpuObject>() as u64;
        let matrix_size = std::mem::size_of::<[[f32; 4]; 4]>() as u64;
        let built = self.data.objects.len() as u64 * object_size;
        let copied = self.data.copies.rows.len() as u64 * object_size;
        let places = self.data.copies.locals.len() as u64 * matrix_size;
        let stamp = (self.data.copies.generation, self.data.objects.len() as u64);
        let frame = &mut self.frames[frame_index];
        let with_copies = copied > 0 && frame.copies_written != Some(stamp);
        let needed = built + if with_copies { copied + places } else { 0 };
        if needed == 0 {
            return Ok(());
        }
        if needed > frame.staging.size {
            let grown = create_buffer(
                context,
                "scene staging",
                needed.next_power_of_two(),
                vk::BufferUsageFlags::TRANSFER_SRC,
                MemoryLocation::CpuToGpu,
            )?;
            destroy_buffer(context, &mut frame.staging);
            frame.staging = grown;
        }
        write_buffer_at(&frame.staging, 0, &self.data.objects);
        let mut objects = built;
        if with_copies {
            write_buffer_at(&frame.staging, built, &self.data.copies.rows[..]);
            write_buffer_at(&frame.staging, built + copied, &self.data.copies.locals[..]);
            objects += copied;
            frame.copies_written = Some(stamp);
        }
        let device = &context.device;
        unsafe {
            if objects > 0 {
                device.cmd_copy_buffer(
                    command_buffer,
                    frame.staging.buffer,
                    frame.objects.buffer,
                    &[vk::BufferCopy::default().size(objects)],
                );
                barrier(
                    device,
                    command_buffer,
                    frame.objects.buffer,
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::AccessFlags2::TRANSFER_WRITE,
                    REACHED_AFTER,
                    vk::AccessFlags2::SHADER_STORAGE_READ | vk::AccessFlags2::SHADER_STORAGE_WRITE,
                );
            }
            if with_copies && places > 0 {
                device.cmd_copy_buffer(
                    command_buffer,
                    frame.staging.buffer,
                    frame.instance_locals.buffer,
                    &[vk::BufferCopy::default()
                        .src_offset(built + copied)
                        .size(places)],
                );
                barrier(
                    device,
                    command_buffer,
                    frame.instance_locals.buffer,
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::AccessFlags2::TRANSFER_WRITE,
                    REACHED_AFTER,
                    vk::AccessFlags2::SHADER_STORAGE_READ,
                );
            }
        }
        Ok(())
    }
}

impl SceneData {
    pub fn object_count(&self) -> usize {
        self.objects.len() + self.copies.rows.len()
    }
}

const BUILDING_PYRAMID: &[(Resource, Usage)] = &[
    (Resource::Depth, Usage::SampledByCompute),
    (Resource::Pyramid, Usage::StorageWrite),
];

const SHADING: &[(Resource, Usage)] = &[
    (Resource::ShadowAtlas, Usage::Sampled),
    (Resource::Grab, Usage::Sampled),
    (Resource::LocalShadowAtlas, Usage::Sampled),
];

const GRABBING: &[(Resource, Usage)] = &[
    (Resource::Hdr, Usage::SampledByCompute),
    (Resource::Grab, Usage::StorageWrite),
];

const OVER_THE_SOLIDS: &[(Resource, Usage)] = &[
    (Resource::ShadowAtlas, Usage::Sampled),
    (Resource::Grab, Usage::Sampled),
    (Resource::LocalShadowAtlas, Usage::Sampled),
    (Resource::Accumulated, Usage::Sampled),
    (Resource::Revealed, Usage::Sampled),
];

impl Pass for ScenePass {
    fn name(&self) -> &'static str {
        SCENE_PASS
    }

    fn usage(&self, scope: ScopeKind) -> &[(Resource, Usage)] {
        match scope {
            ScopeKind::Pyramid if self.pyramid.is_some() => BUILDING_PYRAMID,
            ScopeKind::Grab if self.grab.is_some() => GRABBING,
            ScopeKind::OverSolids => OVER_THE_SOLIDS,
            ScopeKind::Color | ScopeKind::Refracted | ScopeKind::Blended => SHADING,
            _ => &[],
        }
    }

    fn register(&self, states: &mut ResourceStates) {
        if let Some(grab) = &self.grab {
            resource::register(
                states,
                Resource::Grab,
                grab::image(grab),
                vk::ImageAspectFlags::COLOR,
            );
        }
        if let Some(pyramid) = &self.pyramid {
            resource::register(
                states,
                Resource::Pyramid,
                hiz::image(pyramid),
                vk::ImageAspectFlags::COLOR,
            );
        }
    }

    fn compute(&mut self, record: &mut PassRecord) {
        if !self.ready() {
            return;
        }
        let frame_index = record.frame_index % self.frames.len();
        let device = &record.context.device;

        let lists_in_use = self.list_count() as u64;
        if let Err(error) = reserve_frame(
            record.context,
            &mut self.frames[frame_index],
            Wanted {
                objects: self.data.object_count() as u64,
                work_items: self.data.work_capacity as u64,
                work_lists: lists_in_use,
                batches: self.data.instance_batches.len() as u64,
                instances: self.data.copies.locals.len() as u64,
                runs: self.data.run_capacity as u64,
                posed_meshlets: self.data.posed_meshlets as u64,
                bones: self
                    .data
                    .bones
                    .len()
                    .max(self.data.inverse_binds.len())
                    .max(self.data.joints as usize) as u64,
                skins: self.data.skins.len().max(self.data.skin_instances.len()) as u64,
            },
        ) {
            tracing::error!("the scene frame buffers were not grown: {error}");
            return;
        }
        if self.frames[frame_index].bound != Some(self.handover) {
            bind_frame(record.context, &self.frames[frame_index], &self.handover);
            self.frames[frame_index].bound = Some(self.handover);
        }

        self.data.frame.pyramid = match (&self.pyramid, self.splits()) {
            (Some(pyramid), true) => {
                let extent = hiz::extent(pyramid);
                [
                    extent.width as f32,
                    extent.height as f32,
                    hiz::levels(pyramid) as f32,
                    1.0,
                ]
            }
            _ => [0.0; 4],
        };
        if self.pyramid.is_some() {
            record.require(Resource::Pyramid, Usage::SampledByCompute);
        }

        let frame = &self.frames[frame_index];
        if !frame.cleared {
            unsafe {
                device.cmd_fill_buffer(
                    record.command_buffer,
                    frame.visibility.buffer,
                    0,
                    vk::WHOLE_SIZE,
                    0,
                );
            }
            barrier(
                device,
                record.command_buffer,
                frame.visibility.buffer,
                vk::PipelineStageFlags2::TRANSFER,
                vk::AccessFlags2::TRANSFER_WRITE,
                vk::PipelineStageFlags2::COMPUTE_SHADER,
                vk::AccessFlags2::SHADER_STORAGE_READ | vk::AccessFlags2::SHADER_STORAGE_WRITE,
            );
            self.frames[frame_index].cleared = true;
        }
        let frame = &self.frames[frame_index];
        write_buffer(&frame.frame, std::slice::from_ref(&self.data.frame));
        write_buffer(&frame.lights, &self.data.lights);
        write_buffer(&frame.area_lights, &self.data.area_lights);
        write_buffer(&frame.bones, &self.data.bones);
        constraints::upload(
            record.context,
            &mut self.constraints,
            &self.data.constraints,
            frame.bones.buffer,
            frame_index,
        );
        let frame = &self.frames[frame_index];
        write_buffer(&frame.inverse_binds, &self.data.inverse_binds);
        write_buffer(&frame.skins, &self.data.skins);
        write_buffer(&frame.skin_instances, &self.data.skin_instances);
        write_buffer(&frame.instance_batches, &self.data.instance_batches);
        write_buffer(&frame.local_shadows, &self.data.local_shadows);
        if let Err(error) = self.stage_objects(record.context, record.command_buffer, frame_index) {
            tracing::error!("the objects were not staged: {error}");
            return;
        }

        self.place_instances(record, frame_index);

        self.pose_skins(record, frame_index);
        self.assign_lights(record, frame_index);

        self.take_the_count(frame_index);
        let frame = &self.frames[frame_index];
        let groups = (self.data.object_count() as u32).div_ceil(CULL_GROUP_SIZE);
        let reset = [[0u32, 0, 1, 0]; LIST_COUNT];
        unsafe {
            device.cmd_copy_buffer(
                record.command_buffer,
                frame.drawn_counts.buffer,
                frame.drawn_readback.buffer,
                &[vk::BufferCopy::default().size(DRAWN_BYTES)],
            );
            barrier(
                device,
                record.command_buffer,
                frame.drawn_counts.buffer,
                vk::PipelineStageFlags2::TRANSFER,
                vk::AccessFlags2::TRANSFER_READ,
                vk::PipelineStageFlags2::TRANSFER,
                vk::AccessFlags2::TRANSFER_WRITE,
            );
            device.cmd_fill_buffer(
                record.command_buffer,
                frame.drawn_counts.buffer,
                0,
                vk::WHOLE_SIZE,
                0,
            );
            barrier(
                device,
                record.command_buffer,
                frame.drawn_counts.buffer,
                vk::PipelineStageFlags2::TRANSFER,
                vk::AccessFlags2::TRANSFER_WRITE,
                REACHED_AFTER,
                vk::AccessFlags2::SHADER_STORAGE_READ | vk::AccessFlags2::SHADER_STORAGE_WRITE,
            );
            device.cmd_update_buffer(
                record.command_buffer,
                frame.draw_command.buffer,
                0,
                bytemuck::cast_slice(&reset),
            );
            device.cmd_fill_buffer(
                record.command_buffer,
                frame.run_counts.buffer,
                0,
                vk::WHOLE_SIZE,
                0,
            );
            for buffer in [frame.draw_command.buffer, frame.run_counts.buffer] {
                barrier(
                    device,
                    record.command_buffer,
                    buffer,
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::AccessFlags2::TRANSFER_WRITE,
                    vk::PipelineStageFlags2::COMPUTE_SHADER,
                    vk::AccessFlags2::SHADER_STORAGE_READ | vk::AccessFlags2::SHADER_STORAGE_WRITE,
                );
            }
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.cull_pipeline,
            );
            device.cmd_bind_descriptor_sets(
                record.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.pipeline_layout,
                0,
                &[frame.set],
                &[],
            );
            for list in 0..self.list_count() {
                push_list(
                    device,
                    record.command_buffer,
                    self.pipeline_layout,
                    list,
                    PHASE_EARLY,
                );
                device.cmd_dispatch(record.command_buffer, groups, 1, 1);
            }
            for buffer in [
                frame.draw_command.buffer,
                frame.run_draws.buffer,
                frame.run_counts.buffer,
            ] {
                barrier(
                    device,
                    record.command_buffer,
                    buffer,
                    vk::PipelineStageFlags2::COMPUTE_SHADER,
                    vk::AccessFlags2::SHADER_STORAGE_WRITE,
                    vk::PipelineStageFlags2::DRAW_INDIRECT,
                    vk::AccessFlags2::INDIRECT_COMMAND_READ,
                );
            }
            barrier(
                device,
                record.command_buffer,
                frame.work_items.buffer,
                vk::PipelineStageFlags2::COMPUTE_SHADER,
                vk::AccessFlags2::SHADER_STORAGE_WRITE,
                REACHED_AFTER,
                vk::AccessFlags2::SHADER_STORAGE_READ,
            );
        }
    }

    fn offscreen(&mut self, record: &mut PassRecord) {
        if !self.ready() {
            return;
        }
        if self.data.frame.shadow_settings[2] != 0.0 {
            self.draw_shadows(record);
        }
        self.fill_the_probes(record);
        self.light_the_air(record);
    }

    fn draw(&mut self, record: &mut PassRecord) {
        if !self.ready() {
            return;
        }
        let frame_index = record.frame_index % self.frames.len();
        let frame = &self.frames[frame_index];
        self.bind_graphics(record, frame, frame_index);

        match record.scope {
            ScopeKind::EarlyDepth => {
                for (list, pipeline) in self.depth_lists() {
                    self.draw_list(record, frame, list, pipeline, PHASE_EARLY);
                }
            }
            ScopeKind::LateDepth => {
                for (list, pipeline) in self.depth_lists() {
                    self.draw_list(record, frame, list, pipeline, PHASE_LATE);
                }
            }
            ScopeKind::Refracted => {
                self.draw_list(
                    record,
                    frame,
                    LIST_REFRACTED,
                    self.settled_double_sided_pipeline,
                    PHASE_EARLY,
                );
            }
            ScopeKind::Blended => {
                self.draw_list(
                    record,
                    frame,
                    LIST_TRANSPARENT,
                    self.transparent_pipeline,
                    PHASE_EARLY,
                );
            }
            ScopeKind::OverSolids => {
                self.draw_over_solids(record, frame);
            }
            _ => {
                for (list, pipeline) in self.solid_lists() {
                    self.draw_list(record, frame, list, pipeline, PHASE_SHADE);
                }
            }
        }
    }

    fn has_depth_prepass(&self) -> bool {
        self.ready()
    }

    fn between(&mut self, record: &mut PassRecord) {
        if !self.ready() {
            return;
        }
        let frame_index = record.frame_index % self.frames.len();
        match record.scope {
            ScopeKind::Pyramid => {
                if let Some(pyramid) = &self.pyramid {
                    hiz::build(pyramid, record.context, record.command_buffer);
                    record.require(Resource::Pyramid, Usage::SampledByCull);
                }
                self.cull_solids(record, &self.frames[frame_index], PHASE_LATE);
            }
            ScopeKind::Gather => {
                self.cull_solids(record, &self.frames[frame_index], PHASE_EARLY);
            }
            ScopeKind::Grab => {
                if let Some(grab) = &self.grab {
                    grab::build(grab, record.context, record.command_buffer);
                    record.require(Resource::Grab, Usage::Sampled);
                }
            }
            _ => {}
        }
    }

    fn resize(&mut self, context: &Context, targets: ScreenTargets) {
        write_composite_set(
            context,
            self.composite_set,
            self.composite_sampler,
            targets.accumulated,
            targets.revealed,
        );
        if let Some(grab) = &mut self.grab {
            grab::destroy(grab, context);
        }
        self.grab = match grab::create(context, targets.extent, targets.color) {
            Ok(grab) => {
                bind_grab(context, &self.frames, &grab);
                Some(grab)
            }
            Err(error) => {
                tracing::error!("the scene grab was not made again: {error}");
                None
            }
        };
        if let Some(pyramid) = &mut self.pyramid {
            hiz::destroy(pyramid, context);
        }
        self.pyramid = match hiz::create(context, targets.extent, targets.depth) {
            Ok(pyramid) => {
                bind_pyramid(context, &self.frames, &pyramid);
                Some(pyramid)
            }
            Err(error) => {
                tracing::error!("the depth pyramid was not made again: {error}");
                None
            }
        };
        for frame in &mut self.frames {
            frame.cleared = false;
        }
    }

    fn destroy(&mut self, context: &Context) {
        constraints::destroy(context, &mut self.constraints);
        unsafe {
            context
                .device
                .destroy_pipeline(self.composite_pipeline, None);
            context
                .device
                .destroy_pipeline_layout(self.composite_pipeline_layout, None);
            context
                .device
                .destroy_descriptor_pool(self.composite_pool, None);
            context
                .device
                .destroy_descriptor_set_layout(self.composite_layout, None);
            context.device.destroy_sampler(self.composite_sampler, None);
        }
        if let Some(grab) = &mut self.grab {
            grab::destroy(grab, context);
        }
        if let Some(pyramid) = &mut self.pyramid {
            hiz::destroy(pyramid, context);
        }
        unsafe {
            context.device.destroy_pipeline(self.pipeline, None);
            context
                .device
                .destroy_pipeline(self.double_sided_pipeline, None);
            context.device.destroy_pipeline(self.depth_pipeline, None);
            context
                .device
                .destroy_pipeline(self.depth_double_sided_pipeline, None);
            context.device.destroy_pipeline(self.settled_pipeline, None);
            for pipeline in self.runs.all() {
                context.device.destroy_pipeline(pipeline, None);
            }
            context
                .device
                .destroy_pipeline(self.settled_double_sided_pipeline, None);
            context
                .device
                .destroy_pipeline(self.transparent_pipeline, None);
            context.device.destroy_pipeline(self.cull_pipeline, None);
            context.device.destroy_pipeline(self.cluster_pipeline, None);
            context.device.destroy_pipeline(self.joint_pipeline, None);
            context
                .device
                .destroy_pipeline(self.instance_pipeline, None);
            context.device.destroy_pipeline(self.bounds_pipeline, None);
            if let Some(mut fog) = self.fog.take() {
                fog::destroy_rounds(context, &mut fog);
            }
            if let Some(mut rounds) = self.probes.take() {
                probes::destroy_rounds(context, &mut rounds);
            }
            context
                .device
                .destroy_pipeline(self.meshlet_bounds_pipeline, None);
            context.device.destroy_pipeline(self.skin_pipeline, None);
            context.device.destroy_pipeline(self.shadow_pipeline, None);
            context
                .device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            context
                .device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            context
                .device
                .destroy_descriptor_set_layout(self.descriptor_layout, None);
        }
        for frame in &mut self.frames {
            destroy_buffer(context, &mut frame.frame);
            destroy_buffer(context, &mut frame.objects);
            destroy_buffer(context, &mut frame.staging);
            destroy_buffer(context, &mut frame.lights);
            destroy_buffer(context, &mut frame.area_lights);
            destroy_buffer(context, &mut frame.work_items);
            destroy_buffer(context, &mut frame.draw_command);
            destroy_buffer(context, &mut frame.bones);
            destroy_buffer(context, &mut frame.inverse_binds);
            destroy_buffer(context, &mut frame.skins);
            destroy_buffer(context, &mut frame.joint_matrices);
            destroy_buffer(context, &mut frame.skin_instances);
            destroy_buffer(context, &mut frame.instance_batches);
            destroy_buffer(context, &mut frame.instance_locals);
            destroy_buffer(context, &mut frame.cluster_lights);
            destroy_buffer(context, &mut frame.visibility);
            destroy_buffer(context, &mut frame.local_shadows);
            destroy_buffer(context, &mut frame.run_draws);
            destroy_buffer(context, &mut frame.run_counts);
            destroy_buffer(context, &mut frame.drawn_counts);
            destroy_buffer(context, &mut frame.drawn_readback);
            destroy_buffer(context, &mut frame.posed_meshlets);
        }
    }
}

fn create_frame(
    context: &Context,
    pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
) -> Result<FrameResources, Box<dyn std::error::Error>> {
    let frame = create_buffer(
        context,
        "scene frame",
        std::mem::size_of::<GpuFrame>() as u64,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        MemoryLocation::CpuToGpu,
    )?;
    let objects = create_device_storage(
        context,
        "scene objects",
        INITIAL_OBJECT_CAPACITY * std::mem::size_of::<GpuObject>() as u64,
        vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
    )?;
    let staging = create_buffer(
        context,
        "scene staging",
        INITIAL_OBJECT_CAPACITY * std::mem::size_of::<GpuObject>() as u64,
        vk::BufferUsageFlags::TRANSFER_SRC,
        MemoryLocation::CpuToGpu,
    )?;
    let lights = create_shared_storage(
        context,
        "scene lights",
        (MAX_LIGHTS * std::mem::size_of::<LightData>()) as u64,
    )?;
    let area_lights = create_shared_storage(
        context,
        "scene area lights",
        (MAX_LIGHTS * std::mem::size_of::<GpuAreaLight>()) as u64,
    )?;
    let work_items = create_device_storage(
        context,
        "scene work list",
        INITIAL_WORK_CAPACITY * INITIAL_WORK_LISTS * std::mem::size_of::<[u32; 2]>() as u64,
        vk::BufferUsageFlags::STORAGE_BUFFER,
    )?;
    let draw_command = create_device_storage(
        context,
        "scene draw commands",
        (LIST_COUNT * std::mem::size_of::<[u32; 4]>()) as u64,
        vk::BufferUsageFlags::STORAGE_BUFFER
            | vk::BufferUsageFlags::INDIRECT_BUFFER
            | vk::BufferUsageFlags::TRANSFER_DST,
    )?;
    let matrix_size = std::mem::size_of::<[[f32; 4]; 4]>() as u64;
    let bones = create_shared_storage(context, "scene bones", INITIAL_BONE_CAPACITY * matrix_size)?;
    let inverse_binds = create_shared_storage(
        context,
        "scene inverse binds",
        INITIAL_BONE_CAPACITY * matrix_size,
    )?;
    let skins = create_shared_storage(
        context,
        "scene skins",
        INITIAL_SKIN_CAPACITY * std::mem::size_of::<GpuSkinData>() as u64,
    )?;
    let joint_matrices = create_device_storage(
        context,
        "scene joint matrices",
        INITIAL_BONE_CAPACITY * matrix_size,
        vk::BufferUsageFlags::STORAGE_BUFFER,
    )?;
    let skin_instances = create_shared_storage(
        context,
        "scene skin instances",
        INITIAL_SKIN_CAPACITY * std::mem::size_of::<GpuSkinInstance>() as u64,
    )?;
    let instance_batches = create_shared_storage(
        context,
        "scene instance batches",
        INITIAL_BATCH_CAPACITY * std::mem::size_of::<GpuInstanceBatch>() as u64,
    )?;
    let instance_locals = create_device_storage(
        context,
        "scene instance places",
        INITIAL_INSTANCE_CAPACITY * matrix_size,
        vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
    )?;
    let cluster_lights = create_device_storage(
        context,
        "scene cluster lights",
        CLUSTER_WORDS as u64 * std::mem::size_of::<u32>() as u64,
        vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
    )?;
    let visibility = create_visibility(context, INITIAL_OBJECT_CAPACITY)?;
    let local_shadows = create_shared_storage(
        context,
        "scene local shadows",
        (LOCAL_SHADOW_COUNT * std::mem::size_of::<GpuLocalShadow>()) as u64,
    )?;
    let run_draws = create_device_storage(
        context,
        "scene run draws",
        INITIAL_RUN_CAPACITY * INITIAL_WORK_LISTS * RUN_DRAW_SIZE,
        vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::INDIRECT_BUFFER,
    )?;
    let run_counts = create_device_storage(
        context,
        "scene run counts",
        (LIST_COUNT * std::mem::size_of::<u32>()) as u64,
        vk::BufferUsageFlags::STORAGE_BUFFER
            | vk::BufferUsageFlags::INDIRECT_BUFFER
            | vk::BufferUsageFlags::TRANSFER_DST,
    )?;
    let drawn_counts = create_device_storage(
        context,
        "scene drawn counts",
        DRAWN_BYTES,
        vk::BufferUsageFlags::STORAGE_BUFFER
            | vk::BufferUsageFlags::TRANSFER_DST
            | vk::BufferUsageFlags::TRANSFER_SRC,
    )?;
    let drawn_readback = create_buffer(
        context,
        "scene drawn readback",
        DRAWN_BYTES,
        vk::BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuToCpu,
    )?;
    let posed_meshlets = create_posed_meshlets(context, INITIAL_POSED_CAPACITY)?;
    let layouts = [layout];
    let allocate_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    let set = unsafe { context.device.allocate_descriptor_sets(&allocate_info)?[0] };
    Ok(FrameResources {
        frame,
        objects,
        staging,
        lights,
        area_lights,
        work_items,
        draw_command,
        bones,
        inverse_binds,
        skins,
        joint_matrices,
        skin_instances,
        instance_batches,
        instance_locals,
        cluster_lights,
        visibility,
        local_shadows,
        run_draws,
        run_counts,
        run_capacity: INITIAL_RUN_CAPACITY,
        drawn_counts,
        drawn_readback,
        posed_meshlets,
        posed_capacity: INITIAL_POSED_CAPACITY,
        cleared: false,
        copies_written: None,
        bone_capacity: INITIAL_BONE_CAPACITY,
        skin_capacity: INITIAL_SKIN_CAPACITY,
        object_capacity: INITIAL_OBJECT_CAPACITY,
        batch_capacity: INITIAL_BATCH_CAPACITY,
        instance_capacity: INITIAL_INSTANCE_CAPACITY,
        work_capacity: INITIAL_WORK_CAPACITY,
        work_lists: INITIAL_WORK_LISTS,
        set,
        bound: None,
    })
}

const REACHED_AFTER: vk::PipelineStageFlags2 = vk::PipelineStageFlags2::ALL_COMMANDS;

fn barrier(
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    buffer: vk::Buffer,
    source_stage: vk::PipelineStageFlags2,
    source_access: vk::AccessFlags2,
    destination_stage: vk::PipelineStageFlags2,
    destination_access: vk::AccessFlags2,
) {
    let memory = vk::BufferMemoryBarrier2::default()
        .src_stage_mask(source_stage)
        .src_access_mask(source_access)
        .dst_stage_mask(destination_stage)
        .dst_access_mask(destination_access)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .buffer(buffer)
        .size(vk::WHOLE_SIZE);
    let dependency =
        vk::DependencyInfo::default().buffer_memory_barriers(std::slice::from_ref(&memory));
    unsafe { device.cmd_pipeline_barrier2(command_buffer, &dependency) };
}

struct Wanted {
    objects: u64,
    work_items: u64,
    work_lists: u64,
    batches: u64,
    instances: u64,
    runs: u64,
    posed_meshlets: u64,
    bones: u64,
    skins: u64,
}

fn batches_per_dispatch(context: &Context) -> u32 {
    let limits = unsafe {
        context
            .instance
            .get_physical_device_properties(context.physical_device)
            .limits
    };
    limits.max_compute_work_group_count[1].max(1)
}

fn reserve_frame(
    context: &Context,
    frame: &mut FrameResources,
    wanted: Wanted,
) -> Result<(), Box<dyn std::error::Error>> {
    let Wanted {
        objects,
        work_items,
        work_lists,
        batches,
        instances,
        runs,
        posed_meshlets,
        bones,
        skins,
    } = wanted;
    if objects > frame.object_capacity {
        let capacity = objects.next_power_of_two();
        let grown = create_device_storage(
            context,
            "scene objects",
            capacity * std::mem::size_of::<GpuObject>() as u64,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        )?;
        let seen = create_visibility(context, capacity)?;
        destroy_buffer(context, &mut frame.objects);
        destroy_buffer(context, &mut frame.visibility);
        frame.objects = grown;
        frame.visibility = seen;
        frame.object_capacity = capacity;
        frame.cleared = false;
        frame.copies_written = None;
        frame.bound = None;
    }
    let lists = work_lists.max(frame.work_lists).next_power_of_two();
    let lists_grown = lists > frame.work_lists;
    if work_items > frame.work_capacity || lists_grown {
        let capacity = work_items.max(frame.work_capacity).next_power_of_two();
        let grown = create_device_storage(
            context,
            "scene work list",
            capacity * lists * std::mem::size_of::<[u32; 2]>() as u64,
            vk::BufferUsageFlags::STORAGE_BUFFER,
        )?;
        destroy_buffer(context, &mut frame.work_items);
        frame.work_items = grown;
        frame.work_capacity = capacity;
        frame.work_lists = lists;
        frame.bound = None;
    }
    if runs > frame.run_capacity || lists_grown {
        let capacity = runs.max(frame.run_capacity).next_power_of_two();
        let grown = create_device_storage(
            context,
            "scene run draws",
            capacity * lists * RUN_DRAW_SIZE,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::INDIRECT_BUFFER,
        )?;
        destroy_buffer(context, &mut frame.run_draws);
        frame.run_draws = grown;
        frame.run_capacity = capacity;
        frame.bound = None;
    }
    if batches > frame.batch_capacity {
        let capacity = batches.next_power_of_two();
        let grown = create_shared_storage(
            context,
            "scene instance batches",
            capacity * std::mem::size_of::<GpuInstanceBatch>() as u64,
        )?;
        destroy_buffer(context, &mut frame.instance_batches);
        frame.instance_batches = grown;
        frame.batch_capacity = capacity;
        frame.bound = None;
    }
    if instances > frame.instance_capacity {
        let capacity = instances.next_power_of_two();
        let grown = create_device_storage(
            context,
            "scene instance places",
            capacity * std::mem::size_of::<[[f32; 4]; 4]>() as u64,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        )?;
        destroy_buffer(context, &mut frame.instance_locals);
        frame.instance_locals = grown;
        frame.instance_capacity = capacity;
        frame.copies_written = None;
        frame.bound = None;
    }
    if bones > frame.bone_capacity {
        let capacity = bones.next_power_of_two();
        let matrix_size = std::mem::size_of::<[[f32; 4]; 4]>() as u64;
        let grown_bones = create_shared_storage(context, "scene bones", capacity * matrix_size)?;
        let grown_binds =
            create_shared_storage(context, "scene inverse binds", capacity * matrix_size)?;
        let grown_joints = create_device_storage(
            context,
            "scene joint matrices",
            capacity * matrix_size,
            vk::BufferUsageFlags::STORAGE_BUFFER,
        )?;
        destroy_buffer(context, &mut frame.bones);
        destroy_buffer(context, &mut frame.inverse_binds);
        destroy_buffer(context, &mut frame.joint_matrices);
        frame.bones = grown_bones;
        frame.inverse_binds = grown_binds;
        frame.joint_matrices = grown_joints;
        frame.bone_capacity = capacity;
        frame.bound = None;
    }
    if skins > frame.skin_capacity {
        let capacity = skins.next_power_of_two();
        let grown_skins = create_shared_storage(
            context,
            "scene skins",
            capacity * std::mem::size_of::<GpuSkinData>() as u64,
        )?;
        let grown_instances = create_shared_storage(
            context,
            "scene skin instances",
            capacity * std::mem::size_of::<GpuSkinInstance>() as u64,
        )?;
        destroy_buffer(context, &mut frame.skins);
        destroy_buffer(context, &mut frame.skin_instances);
        frame.skins = grown_skins;
        frame.skin_instances = grown_instances;
        frame.skin_capacity = capacity;
        frame.bound = None;
    }
    if posed_meshlets > frame.posed_capacity {
        let capacity = posed_meshlets.next_power_of_two();
        let grown = create_posed_meshlets(context, capacity)?;
        destroy_buffer(context, &mut frame.posed_meshlets);
        frame.posed_meshlets = grown;
        frame.posed_capacity = capacity;
        frame.bound = None;
    }
    Ok(())
}

fn bind_frame(context: &Context, frame: &FrameResources, handover: &SceneHandover) {
    let uniform = vk::DescriptorBufferInfo::default()
        .buffer(frame.frame.buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE);
    let storages = [
        frame.objects.buffer,
        handover.mesh_ranges,
        handover.meshlets,
        handover.meshlet_vertices,
        handover.meshlet_triangles,
        handover.vertices,
        handover.materials,
        frame.lights.buffer,
        frame.area_lights.buffer,
        frame.work_items.buffer,
        frame.draw_command.buffer,
        frame.bones.buffer,
        frame.inverse_binds.buffer,
        frame.skins.buffer,
        frame.joint_matrices.buffer,
        frame.skin_instances.buffer,
        handover.skin_vertices,
        handover.vertices,
        handover.morph_displacements,
        frame.cluster_lights.buffer,
        frame.visibility.buffer,
        frame.instance_batches.buffer,
        frame.instance_locals.buffer,
    ];
    let infos: Vec<vk::DescriptorBufferInfo> = storages
        .iter()
        .map(|buffer| {
            vk::DescriptorBufferInfo::default()
                .buffer(*buffer)
                .offset(0)
                .range(vk::WHOLE_SIZE)
        })
        .collect();

    let mut writes = vec![
        vk::WriteDescriptorSet::default()
            .dst_set(frame.set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(std::slice::from_ref(&uniform)),
    ];
    for (index, info) in infos.iter().enumerate() {
        writes.push(
            vk::WriteDescriptorSet::default()
                .dst_set(frame.set)
                .dst_binding(index as u32 + 1)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(std::slice::from_ref(info)),
        );
    }
    let further = [
        (LOCAL_SHADOWS_BINDING, frame.local_shadows.buffer),
        (RUN_DRAWS_BINDING, frame.run_draws.buffer),
        (RUN_COUNTS_BINDING, frame.run_counts.buffer),
        (DRAWN_COUNTS_BINDING, frame.drawn_counts.buffer),
        (INDICES_BINDING, handover.indices),
        (POSED_MESHLETS_BINDING, frame.posed_meshlets.buffer),
    ];
    let further_infos: Vec<vk::DescriptorBufferInfo> = further
        .iter()
        .map(|(_, buffer)| {
            vk::DescriptorBufferInfo::default()
                .buffer(*buffer)
                .offset(0)
                .range(vk::WHOLE_SIZE)
        })
        .collect();
    for ((binding, _), info) in further.iter().zip(&further_infos) {
        writes.push(
            vk::WriteDescriptorSet::default()
                .dst_set(frame.set)
                .dst_binding(*binding)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(std::slice::from_ref(info)),
        );
    }
    unsafe { context.device.update_descriptor_sets(&writes, &[]) };
}

fn create_shared_storage(
    context: &Context,
    name: &str,
    size: u64,
) -> Result<Buffer, Box<dyn std::error::Error>> {
    create_buffer(
        context,
        name,
        size,
        vk::BufferUsageFlags::STORAGE_BUFFER,
        MemoryLocation::CpuToGpu,
    )
}

fn create_device_storage(
    context: &Context,
    name: &str,
    size: u64,
    usage: vk::BufferUsageFlags,
) -> Result<Buffer, Box<dyn std::error::Error>> {
    create_buffer(context, name, size, usage, MemoryLocation::GpuOnly)
}

fn create_posed_meshlets(
    context: &Context,
    meshlets: u64,
) -> Result<Buffer, Box<dyn std::error::Error>> {
    create_device_storage(
        context,
        "scene posed meshlets",
        meshlets * std::mem::size_of::<[f32; 4]>() as u64,
        vk::BufferUsageFlags::STORAGE_BUFFER,
    )
}

fn create_visibility(
    context: &Context,
    objects: u64,
) -> Result<Buffer, Box<dyn std::error::Error>> {
    create_device_storage(
        context,
        "scene object visibility",
        objects * std::mem::size_of::<u32>() as u64,
        vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
    )
}

fn create_descriptor_layout(
    context: &Context,
) -> Result<vk::DescriptorSetLayout, Box<dyn std::error::Error>> {
    let stages = vk::ShaderStageFlags::COMPUTE
        | vk::ShaderStageFlags::TASK_EXT
        | vk::ShaderStageFlags::MESH_EXT
        | vk::ShaderStageFlags::VERTEX
        | vk::ShaderStageFlags::FRAGMENT;
    let mut bindings = vec![
        vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(stages),
    ];
    for binding in (1..=STORAGE_BINDINGS).chain([
        LOCAL_SHADOWS_BINDING,
        RUN_DRAWS_BINDING,
        RUN_COUNTS_BINDING,
        DRAWN_COUNTS_BINDING,
        INDICES_BINDING,
        POSED_MESHLETS_BINDING,
    ]) {
        bindings.push(
            vk::DescriptorSetLayoutBinding::default()
                .binding(binding)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(stages),
        );
    }
    bindings.push(
        vk::DescriptorSetLayoutBinding::default()
            .binding(PROBE_RAYS_BINDING)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::COMPUTE),
    );
    bindings.push(
        vk::DescriptorSetLayoutBinding::default()
            .binding(PROBE_OFFSETS_BINDING)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(stages),
    );
    for binding in [
        PYRAMID_BINDING,
        GRAB_BINDING,
        FOG_VOLUME_BINDING,
        FOG_HISTORY_A_BINDING,
        FOG_HISTORY_B_BINDING,
        PROBE_IRRADIANCE_BINDING,
        PROBE_DISTANCE_BINDING,
    ] {
        bindings.push(
            vk::DescriptorSetLayoutBinding::default()
                .binding(binding)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(stages),
        );
    }
    for binding in [
        FOG_SCATTER_A_BINDING,
        FOG_SCATTER_B_BINDING,
        FOG_MARCHED_BINDING,
        PROBE_IRRADIANCE_STORE_BINDING,
        PROBE_DISTANCE_STORE_BINDING,
    ] {
        bindings.push(
            vk::DescriptorSetLayoutBinding::default()
                .binding(binding)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
        );
    }
    let create_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
    Ok(unsafe {
        context
            .device
            .create_descriptor_set_layout(&create_info, None)?
    })
}

fn create_pipeline_layout(
    context: &Context,
    scene: vk::DescriptorSetLayout,
    textures: vk::DescriptorSetLayout,
    shadows: vk::DescriptorSetLayout,
    environment: vk::DescriptorSetLayout,
) -> Result<vk::PipelineLayout, Box<dyn std::error::Error>> {
    let layouts = [scene, textures, shadows, environment];
    let range = vk::PushConstantRange::default()
        .stage_flags(
            vk::ShaderStageFlags::COMPUTE
                | vk::ShaderStageFlags::TASK_EXT
                | vk::ShaderStageFlags::MESH_EXT
                | vk::ShaderStageFlags::VERTEX
                | vk::ShaderStageFlags::FRAGMENT,
        )
        .offset(0)
        .size(std::mem::size_of::<[u32; 2]>() as u32);
    let create_info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(&layouts)
        .push_constant_ranges(std::slice::from_ref(&range));
    Ok(unsafe { context.device.create_pipeline_layout(&create_info, None)? })
}

fn clear_depth(device: &ash::Device, command_buffer: vk::CommandBuffer, extent: vk::Extent2D) {
    let attachment = vk::ClearAttachment {
        aspect_mask: vk::ImageAspectFlags::DEPTH,
        color_attachment: 0,
        clear_value: vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 0.0,
                stencil: 0,
            },
        },
    };
    let rect = vk::ClearRect {
        rect: vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent,
        },
        base_array_layer: 0,
        layer_count: 1,
    };
    unsafe {
        device.cmd_clear_attachments(
            command_buffer,
            std::slice::from_ref(&attachment),
            std::slice::from_ref(&rect),
        )
    };
}

fn push_list(
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    layout: vk::PipelineLayout,
    list: u32,
    phase: u32,
) {
    let push = [list, phase];
    unsafe {
        device.cmd_push_constants(
            command_buffer,
            layout,
            vk::ShaderStageFlags::COMPUTE
                | vk::ShaderStageFlags::TASK_EXT
                | vk::ShaderStageFlags::MESH_EXT
                | vk::ShaderStageFlags::VERTEX
                | vk::ShaderStageFlags::FRAGMENT,
            0,
            bytemuck::cast_slice(&push),
        )
    };
}

fn create_depth_pipeline(
    setup: &PassSetup<'_>,
    layout: vk::PipelineLayout,
    double_sided: bool,
    path: DrawPath,
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    let context = setup.context;
    let drawn = draw_stages(
        context,
        path,
        include_bytes!("../shaders/scene_depth.frag.spv"),
        include_bytes!("../shaders/scene_depth_run.frag.spv"),
    )?;
    let stages = &drawn.stages;
    let viewport_state = vk::PipelineViewportStateCreateInfo::default()
        .viewport_count(1)
        .scissor_count(1);
    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state =
        vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
    let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(if double_sided {
            vk::CullModeFlags::NONE
        } else {
            vk::CullModeFlags::BACK
        })
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .line_width(1.0);
    let multisample = vk::PipelineMultisampleStateCreateInfo::default()
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);
    let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::GREATER_OR_EQUAL);
    let blend_attachments = [vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::R)];
    let color_blend =
        vk::PipelineColorBlendStateCreateInfo::default().attachments(&blend_attachments);
    let color_formats = [setup.entity_id_format];
    let mut rendering = vk::PipelineRenderingCreateInfo::default()
        .color_attachment_formats(&color_formats)
        .depth_attachment_format(setup.depth_format);

    let vertex_input = vk::PipelineVertexInputStateCreateInfo::default();
    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
    let mut create_info = vk::GraphicsPipelineCreateInfo::default()
        .stages(stages)
        .viewport_state(&viewport_state)
        .dynamic_state(&dynamic_state)
        .rasterization_state(&rasterization)
        .multisample_state(&multisample)
        .depth_stencil_state(&depth_stencil)
        .color_blend_state(&color_blend)
        .layout(layout)
        .push_next(&mut rendering);
    if path == DrawPath::Runs {
        create_info = create_info
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&input_assembly);
    }

    let created = unsafe {
        context
            .device
            .create_graphics_pipelines(
                context.pipeline_cache,
                std::slice::from_ref(&create_info),
                None,
            )
            .map_err(|(_, error)| error)?
    };
    destroy_draw_stages(context, drawn);
    Ok(created[0])
}

fn create_shadow_pipeline(
    context: &Context,
    layout: vk::PipelineLayout,
    path: DrawPath,
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    let drawn = draw_stages(
        context,
        path,
        include_bytes!("../shaders/scene_shadow.frag.spv"),
        include_bytes!("../shaders/scene_shadow_run.frag.spv"),
    )?;
    let stages = &drawn.stages;
    let viewport_state = vk::PipelineViewportStateCreateInfo::default()
        .viewport_count(1)
        .scissor_count(1);
    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state =
        vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
    let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::FRONT)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_bias_enable(true)
        .depth_bias_constant_factor(-1.25)
        .depth_bias_slope_factor(-1.75)
        .line_width(1.0);
    let multisample = vk::PipelineMultisampleStateCreateInfo::default()
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);
    let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::GREATER_OR_EQUAL);
    let color_blend = vk::PipelineColorBlendStateCreateInfo::default();
    let mut rendering =
        vk::PipelineRenderingCreateInfo::default().depth_attachment_format(DEPTH_FORMAT);

    let vertex_input = vk::PipelineVertexInputStateCreateInfo::default();
    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
    let mut create_info = vk::GraphicsPipelineCreateInfo::default()
        .stages(stages)
        .viewport_state(&viewport_state)
        .dynamic_state(&dynamic_state)
        .rasterization_state(&rasterization)
        .multisample_state(&multisample)
        .depth_stencil_state(&depth_stencil)
        .color_blend_state(&color_blend)
        .layout(layout)
        .push_next(&mut rendering);
    if path == DrawPath::Runs {
        create_info = create_info
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&input_assembly);
    }

    let created = unsafe {
        context
            .device
            .create_graphics_pipelines(
                context.pipeline_cache,
                std::slice::from_ref(&create_info),
                None,
            )
            .map_err(|(_, error)| error)?
    };
    destroy_draw_stages(context, drawn);
    Ok(created[0])
}

fn create_descriptor_pool(
    context: &Context,
    frames: usize,
) -> Result<vk::DescriptorPool, Box<dyn std::error::Error>> {
    let count = frames as u32;
    let sizes = [
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(count),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(count * (STORAGE_BINDINGS + FURTHER_BINDINGS + 2)),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(count * 7),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_IMAGE)
            .descriptor_count(count * 5),
    ];
    let create_info = vk::DescriptorPoolCreateInfo::default()
        .pool_sizes(&sizes)
        .max_sets(count);
    Ok(unsafe { context.device.create_descriptor_pool(&create_info, None)? })
}

fn create_cull_pipeline(
    context: &Context,
    layout: vk::PipelineLayout,
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    create_compute_pipeline(
        context,
        layout,
        include_bytes!("../shaders/scene_cull.comp.spv"),
    )
}

fn create_compute_pipeline(
    context: &Context,
    layout: vk::PipelineLayout,
    code: &[u8],
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    let module = create_shader_module(context, code)?;
    let stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::COMPUTE)
        .module(module)
        .name(c"main");
    let create_info = vk::ComputePipelineCreateInfo::default()
        .stage(stage)
        .layout(layout);
    let created = unsafe {
        context
            .device
            .create_compute_pipelines(
                context.pipeline_cache,
                std::slice::from_ref(&create_info),
                None,
            )
            .map_err(|(_, error)| error)?
    };
    unsafe { context.device.destroy_shader_module(module, None) };
    Ok(created[0])
}

enum Rasterizing {
    Settled,
    SettledDoubleSided,
    BackFaceCull,
    DoubleSided,
    Blended,
}

struct DrawStages {
    modules: Vec<vk::ShaderModule>,
    stages: Vec<vk::PipelineShaderStageCreateInfo<'static>>,
}

fn draw_stages(
    context: &Context,
    path: DrawPath,
    meshed_fragment: &[u8],
    run_fragment: &[u8],
) -> Result<DrawStages, Box<dyn std::error::Error>> {
    let mut modules = Vec::new();
    let mut stages = Vec::new();
    let mut stage = |flag: vk::ShaderStageFlags, module: vk::ShaderModule| {
        modules.push(module);
        stages.push(
            vk::PipelineShaderStageCreateInfo::default()
                .stage(flag)
                .module(module)
                .name(c"main"),
        );
    };
    match path {
        DrawPath::Meshed => {
            stage(
                vk::ShaderStageFlags::TASK_EXT,
                create_shader_module(context, include_bytes!("../shaders/scene.task.spv"))?,
            );
            stage(
                vk::ShaderStageFlags::MESH_EXT,
                create_shader_module(context, include_bytes!("../shaders/scene.mesh.spv"))?,
            );
            stage(
                vk::ShaderStageFlags::FRAGMENT,
                create_shader_module(context, meshed_fragment)?,
            );
        }
        DrawPath::Runs => {
            stage(
                vk::ShaderStageFlags::VERTEX,
                create_shader_module(context, include_bytes!("../shaders/scene.vert.spv"))?,
            );
            stage(
                vk::ShaderStageFlags::FRAGMENT,
                create_shader_module(context, run_fragment)?,
            );
        }
    }
    Ok(DrawStages { modules, stages })
}

fn destroy_draw_stages(context: &Context, drawn: DrawStages) {
    for module in drawn.modules {
        unsafe { context.device.destroy_shader_module(module, None) };
    }
}

fn create_pipeline(
    setup: &PassSetup<'_>,
    layout: vk::PipelineLayout,
    rasterizing: Rasterizing,
    path: DrawPath,
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    let context = setup.context;
    let drawn = draw_stages(
        context,
        path,
        include_bytes!("../shaders/scene.frag.spv"),
        include_bytes!("../shaders/scene_run.frag.spv"),
    )?;
    let stages = &drawn.stages;

    let viewport_state = vk::PipelineViewportStateCreateInfo::default()
        .viewport_count(1)
        .scissor_count(1);
    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state =
        vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
    let cull_mode = match rasterizing {
        Rasterizing::BackFaceCull | Rasterizing::Settled => vk::CullModeFlags::BACK,
        Rasterizing::DoubleSided | Rasterizing::SettledDoubleSided | Rasterizing::Blended => {
            vk::CullModeFlags::NONE
        }
    };
    let settled = matches!(
        rasterizing,
        Rasterizing::Settled | Rasterizing::SettledDoubleSided
    );
    let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(cull_mode)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .line_width(1.0);
    let multisample = vk::PipelineMultisampleStateCreateInfo::default()
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);
    let blended = matches!(rasterizing, Rasterizing::Blended);
    let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
        .depth_test_enable(true)
        .depth_write_enable(!blended && !settled)
        .depth_compare_op(if settled {
            vk::CompareOp::EQUAL
        } else {
            vk::CompareOp::GREATER_OR_EQUAL
        });
    let mut blend_attachment = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::RGBA);
    if blended {
        blend_attachment = blend_attachment
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD);
    }
    if blended {
        blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ONE)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE)
            .alpha_blend_op(vk::BlendOp::ADD);
    }
    let normal_blend = if blended {
        vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::ZERO)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_COLOR)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ZERO)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD)
    } else {
        vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
    };
    let motion_blend = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::R | vk::ColorComponentFlags::G);
    let blended_attachments = [
        blend_attachment,
        normal_blend,
        vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::empty()),
    ];
    let solid_attachments = [blend_attachment, normal_blend, motion_blend];
    let attachments: &[vk::PipelineColorBlendAttachmentState] = if blended {
        &blended_attachments
    } else {
        &solid_attachments
    };
    let color_blend = vk::PipelineColorBlendStateCreateInfo::default().attachments(attachments);

    let blended_formats = [
        setup.accumulated_format,
        setup.revealed_format,
        setup.motion_format,
    ];
    let solid_formats = [setup.hdr_format, setup.normal_format, setup.motion_format];
    let color_formats: &[vk::Format] = if blended {
        &blended_formats
    } else {
        &solid_formats
    };
    let mut rendering = vk::PipelineRenderingCreateInfo::default()
        .color_attachment_formats(color_formats)
        .depth_attachment_format(setup.depth_format);

    let vertex_input = vk::PipelineVertexInputStateCreateInfo::default();
    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
    let mut create_info = vk::GraphicsPipelineCreateInfo::default()
        .stages(stages)
        .viewport_state(&viewport_state)
        .dynamic_state(&dynamic_state)
        .rasterization_state(&rasterization)
        .multisample_state(&multisample)
        .depth_stencil_state(&depth_stencil)
        .color_blend_state(&color_blend)
        .layout(layout)
        .push_next(&mut rendering);
    if path == DrawPath::Runs {
        create_info = create_info
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&input_assembly);
    }

    let created = unsafe {
        context
            .device
            .create_graphics_pipelines(
                context.pipeline_cache,
                std::slice::from_ref(&create_info),
                None,
            )
            .map_err(|(_, error)| error)?
    };
    destroy_draw_stages(context, drawn);
    Ok(created[0])
}

fn create_composite_layout(
    context: &Context,
) -> Result<vk::DescriptorSetLayout, Box<dyn std::error::Error>> {
    let bindings = [
        vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
        vk::DescriptorSetLayoutBinding::default()
            .binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
    ];
    let create_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
    Ok(unsafe {
        context
            .device
            .create_descriptor_set_layout(&create_info, None)?
    })
}

fn create_composite_pool(
    context: &Context,
) -> Result<vk::DescriptorPool, Box<dyn std::error::Error>> {
    let size = vk::DescriptorPoolSize::default()
        .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(2);
    let create_info = vk::DescriptorPoolCreateInfo::default()
        .pool_sizes(std::slice::from_ref(&size))
        .max_sets(1);
    Ok(unsafe { context.device.create_descriptor_pool(&create_info, None)? })
}

fn create_composite_sampler(context: &Context) -> Result<vk::Sampler, Box<dyn std::error::Error>> {
    let create_info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::NEAREST)
        .min_filter(vk::Filter::NEAREST)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE);
    Ok(unsafe { context.device.create_sampler(&create_info, None)? })
}

fn write_composite_set(
    context: &Context,
    set: vk::DescriptorSet,
    sampler: vk::Sampler,
    accumulated: vk::ImageView,
    revealed: vk::ImageView,
) {
    let sum = vk::DescriptorImageInfo::default()
        .image_view(accumulated)
        .sampler(sampler)
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
    let showing = vk::DescriptorImageInfo::default()
        .image_view(revealed)
        .sampler(sampler)
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
    let writes = [
        vk::WriteDescriptorSet::default()
            .dst_set(set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&sum)),
        vk::WriteDescriptorSet::default()
            .dst_set(set)
            .dst_binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&showing)),
    ];
    unsafe { context.device.update_descriptor_sets(&writes, &[]) };
}

fn create_composite_pipeline(
    setup: &PassSetup<'_>,
    layout: vk::PipelineLayout,
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    let context = setup.context;
    let vertex = create_shader_module(context, include_bytes!("../shaders/fullscreen.vert.spv"))?;
    let fragment =
        create_shader_module(context, include_bytes!("../shaders/oit_composite.frag.spv"))?;
    let stages = [
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertex)
            .name(c"main"),
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragment)
            .name(c"main"),
    ];
    let vertex_input = vk::PipelineVertexInputStateCreateInfo::default();
    let assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
    let viewport_state = vk::PipelineViewportStateCreateInfo::default()
        .viewport_count(1)
        .scissor_count(1);
    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state =
        vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
    let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .line_width(1.0);
    let multisample = vk::PipelineMultisampleStateCreateInfo::default()
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);
    let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default();
    let blend_attachments = [
        vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD),
        vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::empty()),
        vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::empty()),
    ];
    let color_blend =
        vk::PipelineColorBlendStateCreateInfo::default().attachments(&blend_attachments);
    let color_formats = [setup.hdr_format, setup.normal_format, setup.motion_format];
    let mut rendering = vk::PipelineRenderingCreateInfo::default()
        .color_attachment_formats(&color_formats)
        .depth_attachment_format(setup.depth_format);

    let create_info = vk::GraphicsPipelineCreateInfo::default()
        .stages(&stages)
        .vertex_input_state(&vertex_input)
        .input_assembly_state(&assembly)
        .viewport_state(&viewport_state)
        .dynamic_state(&dynamic_state)
        .rasterization_state(&rasterization)
        .multisample_state(&multisample)
        .depth_stencil_state(&depth_stencil)
        .color_blend_state(&color_blend)
        .layout(layout)
        .push_next(&mut rendering);

    let created = unsafe {
        context
            .device
            .create_graphics_pipelines(
                context.pipeline_cache,
                std::slice::from_ref(&create_info),
                None,
            )
            .map_err(|(_, error)| error)?
    };
    unsafe {
        context.device.destroy_shader_module(vertex, None);
        context.device.destroy_shader_module(fragment, None);
    }
    Ok(created[0])
}
