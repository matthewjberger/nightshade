use ash::vk;
use gpu_allocator::MemoryLocation;

use crate::renderer::context::Context;
use crate::renderer::memory::{
    Buffer, Image, begin_one_shot, create_buffer, create_color_image, destroy_buffer,
    destroy_image, end_one_shot, release, retire_buffer, transition,
};
use crate::renderer::pass::{
    Pass, PassRecord, PassSetup, ScopeKind, ScreenTargets, create_shader_module,
};
use crate::renderer::resource::{self, Resource, ResourceStates, Usage};

pub const POST_PASS: &str = "post";

const BLOOM_LEVELS: usize = 6;

const COLOR_LUT_SIZE: u32 = 16;

struct ColorLut {
    image: vk::Image,
    view: vk::ImageView,
    allocation: Option<gpu_allocator::vulkan::Allocation>,
    sampler: vk::Sampler,
}

fn plain_color_lut() -> Vec<u8> {
    let size = COLOR_LUT_SIZE as usize;
    let mut table = Vec::with_capacity(size * size * size * 4);
    let step = 255.0 / (size as f32 - 1.0);
    for blue in 0..size {
        for green in 0..size {
            for red in 0..size {
                table.push((red as f32 * step).round() as u8);
                table.push((green as f32 * step).round() as u8);
                table.push((blue as f32 * step).round() as u8);
                table.push(255);
            }
        }
    }
    table
}

fn create_color_lut(context: &Context) -> Result<ColorLut, Box<dyn std::error::Error>> {
    let create_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_3D)
        .format(vk::Format::R8G8B8A8_UNORM)
        .extent(vk::Extent3D {
            width: COLOR_LUT_SIZE,
            height: COLOR_LUT_SIZE,
            depth: COLOR_LUT_SIZE,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .initial_layout(vk::ImageLayout::UNDEFINED);
    let image = unsafe { context.device.create_image(&create_info, None)? };
    let requirements = unsafe { context.device.get_image_memory_requirements(image) };
    let allocation = context
        .allocator()
        .lock()
        .map_err(|error| error.to_string())?
        .allocate(&gpu_allocator::vulkan::AllocationCreateDesc {
            name: "color lookup table",
            requirements,
            location: MemoryLocation::GpuOnly,
            linear: false,
            allocation_scheme: gpu_allocator::vulkan::AllocationScheme::GpuAllocatorManaged,
        })?;
    unsafe {
        context
            .device
            .bind_image_memory(image, allocation.memory(), allocation.offset())?;
    }
    let view_info = vk::ImageViewCreateInfo::default()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_3D)
        .format(vk::Format::R8G8B8A8_UNORM)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });
    let view = unsafe { context.device.create_image_view(&view_info, None)? };
    let sampler_info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .mipmap_mode(vk::SamplerMipmapMode::NEAREST);
    let sampler = unsafe { context.device.create_sampler(&sampler_info, None)? };

    let lut = ColorLut {
        image,
        view,
        allocation: Some(allocation),
        sampler,
    };
    fill_color_lut(context, &lut, &plain_color_lut())?;
    Ok(lut)
}

fn fill_color_lut(
    context: &Context,
    lut: &ColorLut,
    table: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let size = COLOR_LUT_SIZE as u64;
    let wanted = (size * size * size * 4) as usize;
    if table.len() < wanted {
        return Err("the color lookup table is smaller than the one the pass holds".into());
    }
    let mut staging = create_buffer(
        context,
        "color lookup table upload",
        wanted as u64,
        vk::BufferUsageFlags::TRANSFER_SRC,
        MemoryLocation::CpuToGpu,
    )?;
    if let Some(allocation) = staging.allocation.as_ref()
        && let Some(mapped) = allocation.mapped_ptr()
    {
        unsafe {
            std::ptr::copy_nonoverlapping(table.as_ptr(), mapped.as_ptr() as *mut u8, wanted);
        }
    }

    let command_buffer = begin_one_shot(context)?;
    transition(
        context,
        command_buffer,
        lut.image,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
    );
    let region = vk::BufferImageCopy::default()
        .image_subresource(vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        })
        .image_extent(vk::Extent3D {
            width: COLOR_LUT_SIZE,
            height: COLOR_LUT_SIZE,
            depth: COLOR_LUT_SIZE,
        });
    unsafe {
        context.device.cmd_copy_buffer_to_image(
            command_buffer,
            staging.buffer,
            lut.image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            std::slice::from_ref(&region),
        );
    }
    transition(
        context,
        command_buffer,
        lut.image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );
    end_one_shot(context, command_buffer)?;
    retire_buffer(context, &mut staging);
    Ok(())
}

pub fn set_color_lut(
    pass: &PostPass,
    context: &Context,
    table: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    fill_color_lut(context, &pass.color_lut, table)
}

fn destroy_color_lut(context: &Context, lut: &mut ColorLut) {
    unsafe {
        context.device.destroy_sampler(lut.sampler, None);
        context.device.destroy_image_view(lut.view, None);
        context.device.destroy_image(lut.image, None);
    }
    release(context, lut.allocation.take());
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PostParams {
    pub exposure: f32,
    pub gamma: f32,
    pub saturation: f32,
    pub brightness: f32,
    pub contrast: f32,
    pub vibrance: f32,
    pub bloom_intensity: f32,
    pub vignette_intensity: f32,
    pub vignette_radius: f32,
    pub vignette_smoothness: f32,
    pub tonemap: u32,
    pub bloom_enabled: u32,
    pub occlusion_enabled: u32,
    pub occlusion_only: u32,
    pub reflection_enabled: u32,
    pub indirect_enabled: u32,
    pub indirect_intensity: f32,
    pub resolved_enabled: u32,
    pub focus_enabled: u32,
    pub auto_exposure_target: f32,
    pub color_lut_weight: f32,
    pub chromatic_aberration: f32,
    pub world_enabled: u32,
    pub exposure_compensation: f32,
    pub auto_exposure_lowest: f32,
    pub auto_exposure_highest: f32,
    pub output_encoding: u32,
    pub paper_white: f32,
    pub headroom: f32,
}

impl Default for PostParams {
    fn default() -> Self {
        Self {
            exposure: 1.0,
            gamma: 2.2,
            saturation: 1.0,
            brightness: 0.0,
            contrast: 1.0,
            vibrance: 0.0,
            bloom_intensity: 0.0,
            vignette_intensity: 0.0,
            vignette_radius: 0.5,
            vignette_smoothness: 0.5,
            tonemap: 0,
            bloom_enabled: 0,
            occlusion_enabled: 0,
            occlusion_only: 0,
            reflection_enabled: 0,
            indirect_enabled: 0,
            indirect_intensity: 0.0,
            resolved_enabled: 0,
            focus_enabled: 0,
            auto_exposure_target: 0.0,
            color_lut_weight: 0.0,
            chromatic_aberration: 0.0,
            world_enabled: 1,
            exposure_compensation: 1.0,
            auto_exposure_lowest: (-3.0_f32).exp2(),
            auto_exposure_highest: 5.0_f32.exp2(),
            output_encoding: 0,
            paper_white: 200.0,
            headroom: 1.0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ReflectionParams {
    pub inverse_projection: [[f32; 4]; 4],
    pub projection: [f32; 4],
    pub screen: [f32; 4],
    pub fade: [f32; 4],
    pub march: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct IndirectParams {
    pub inverse_projection: [[f32; 4]; 4],
    pub projection: [f32; 4],
    pub screen: [f32; 4],
    pub settings: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct ScreenBlurParams {
    screen: [f32; 4],
    shape: [f32; 4],
}

const REFLECTION_BLUR: (f32, f32) = (2.0, 5.0);
const INDIRECT_BLUR: (f32, f32) = (1.0, 2.0);
const SCREEN_BLUR_THRESHOLD: f32 = 0.05;
const SCREEN_BLUR_NORMAL_POWER: f32 = 16.0;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FocusParams {
    pub focus: [f32; 4],
    pub bokeh: [f32; 4],
    pub screen: [f32; 4],
    pub tilt: [f32; 4],
    pub source: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ResolveParams {
    pub reprojection: [[f32; 4]; 4],
    pub screen: [f32; 4],
    pub settings: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SmoothParams {
    pub texel: [f32; 2],
    pub relative_threshold: f32,
    pub absolute_threshold: f32,
    pub subpixel: f32,
    pub snap: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct OcclusionParams {
    pub inverse_projection: [[f32; 4]; 4],
    pub projection: [f32; 4],
    pub settings: [f32; 4],
    pub traced: [f32; 4],
}

pub struct TracedInputs {
    pub scene_layout: vk::DescriptorSetLayout,
    pub scene_sets: Vec<vk::DescriptorSet>,
    pub texture_layout: vk::DescriptorSetLayout,
    pub texture_set: vk::DescriptorSet,
    pub shadow_layout: vk::DescriptorSetLayout,
    pub shadow_sets: Vec<vk::DescriptorSet>,
    pub environment_layout: vk::DescriptorSetLayout,
    pub environment_sets: Vec<vk::DescriptorSet>,
}

struct Quiet {
    kept: [Image; 2],
    moments: [Image; 2],
    wave: [Image; 2],
    noise: [Image; 2],
    gather_sets: [vk::DescriptorSet; 2],
    variance_sets: [vk::DescriptorSet; 2],
    wavelet_sets: [vk::DescriptorSet; 2],
    gather_layout: vk::PipelineLayout,
    gather_pipeline: vk::Pipeline,
    variance_layout: vk::PipelineLayout,
    variance_pipeline: vk::Pipeline,
    wavelet_layout: vk::PipelineLayout,
    wavelet_pipeline: vk::Pipeline,
    gather_descriptor_layout: vk::DescriptorSetLayout,
    variance_descriptor_layout: vk::DescriptorSetLayout,
    wavelet_descriptor_layout: vk::DescriptorSetLayout,
    turn: usize,
    standing: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuietParams {
    pub reprojection: [[f32; 4]; 4],
    pub screen: [f32; 4],
    pub edges: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct WaveParams {
    screen: [f32; 4],
    edges: [f32; 4],
}

const QUIET_TRUSTED: f32 = 4.0;
const QUIET_STANDING: f32 = 32.0;

const QUIET_SPREAD: f32 = 3.0;

const QUIET_ROUNDS: u32 = 3;

const QUIET_DEPTH: f32 = 0.05;
const QUIET_NORMAL: f32 = 64.0;
const QUIET_BRIGHTNESS: f32 = 4.0;
const QUIET_HISTORY_ROOM: f32 = 3.0;

struct Traced {
    inputs: TracedInputs,
    reflection_layout: vk::PipelineLayout,
    reflection_pipeline: vk::Pipeline,
    occlusion_layout: vk::PipelineLayout,
    occlusion_pipeline: vk::Pipeline,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct BlurParams {
    texel: [f32; 4],
    settings: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct DownsampleParams {
    threshold: f32,
    knee: f32,
    exposure: f32,
    level: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct UpsampleParams {
    filter_radius: f32,
}

struct PostBloom {
    downsample_pipeline: vk::Pipeline,
    upsample_pipeline: vk::Pipeline,
    compose_pipeline: vk::Pipeline,
    single_layout: vk::PipelineLayout,
    compose_layout: vk::PipelineLayout,
    single_descriptor_layout: vk::DescriptorSetLayout,
    compose_descriptor_layout: vk::DescriptorSetLayout,
    images: Vec<Image>,
    downsample_sets: Vec<vk::DescriptorSet>,
    downsample_resolved_set: vk::DescriptorSet,
    downsample_focus_set: vk::DescriptorSet,
    upsample_sets: Vec<vk::DescriptorSet>,
    compose_set: vk::DescriptorSet,
    threshold: f32,
    knee: f32,
    filter_radius: f32,
}

struct PostExposure {
    buffer: Buffer,
    pipeline: vk::Pipeline,
    gather_pipeline: vk::Pipeline,
    cleared: bool,
    layout: vk::PipelineLayout,
    descriptor_layout: vk::DescriptorSetLayout,
    set: vk::DescriptorSet,
    settings: [f32; 4],
}

struct PostOcclusion {
    pipeline: vk::Pipeline,
    blur_pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
    descriptor_layout: vk::DescriptorSetLayout,
    blur_layout: vk::PipelineLayout,
    blur_descriptor_layout: vk::DescriptorSetLayout,
    images: Vec<Image>,
    set: vk::DescriptorSet,
    blur_set: vk::DescriptorSet,
    written: bool,
    params: OcclusionParams,
    blur_threshold: f32,
    blur_sharpness: f32,
}

struct PostReflection {
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
    descriptor_layout: vk::DescriptorSetLayout,
    image: Option<Image>,
    set: vk::DescriptorSet,
    written: bool,
    params: ReflectionParams,
    soft: Option<Image>,
    blur_set: vk::DescriptorSet,
}

struct PostIndirect {
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
    image: Option<Image>,
    set: vk::DescriptorSet,
    written: bool,
    params: IndirectParams,
    soft: Option<Image>,
    blur_set: vk::DescriptorSet,
}

struct PostResolve {
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
    descriptor_layout: vk::DescriptorSetLayout,
    image: Option<Image>,
    kept: Vec<Image>,
    sets: Vec<vk::DescriptorSet>,
    ready: bool,
    written: bool,
    round: usize,
    params: ResolveParams,
}

struct PostFocus {
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
    image: Option<Image>,
    set: vk::DescriptorSet,
    written: bool,
    params: FocusParams,
}

pub struct PostPass {
    focus: PostFocus,
    resolve: PostResolve,
    indirect: PostIndirect,
    reflection: PostReflection,
    occlusion: PostOcclusion,
    exposure: PostExposure,
    bloom: PostBloom,
    pool: vk::DescriptorPool,
    sampler: vk::Sampler,
    point_sampler: vk::Sampler,
    color_lut: ColorLut,
    screen_blur_pipeline: vk::Pipeline,
    screen_blur_layout: vk::PipelineLayout,
    quiet: Option<Quiet>,
    traced: Option<Traced>,
    smooth_pipeline: vk::Pipeline,
    smooth_layout: vk::PipelineLayout,
    composed: Option<Image>,
    composed_set: vk::DescriptorSet,
    smooth: Option<SmoothParams>,
    color_format: vk::Format,
    screen: vk::Extent2D,
    extent: vk::Extent2D,
    hdr_format: vk::Format,
    depth_view: vk::ImageView,
    motion_view: vk::ImageView,
    normal_view: vk::ImageView,
    params: PostParams,
}

const OCCLUSION_FORMAT: vk::Format = vk::Format::R8_UNORM;

pub fn create_pass(
    setup: &PassSetup<'_>,
    extent: vk::Extent2D,
    scene_view: vk::ImageView,
) -> Result<PostPass, Box<dyn std::error::Error>> {
    let hdr_format = setup.hdr_format;
    let context = setup.context;
    let single_descriptor_layout = create_descriptor_layout(context, 1)?;
    let compose_descriptor_layout = create_compose_layout(context)?;
    let exposure = create_buffer(
        context,
        "settled exposure",
        (std::mem::size_of::<[f32; 4]>() + std::mem::size_of::<[u32; EXPOSURE_STEPS]>()) as u64,
        vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )?;
    let exposure_descriptor_layout = create_exposure_layout(context)?;
    let exposure_layout = create_compute_layout(context, exposure_descriptor_layout)?;
    let exposure_pipeline = create_exposure_pipeline(
        context,
        exposure_layout,
        include_bytes!("shaders/auto_exposure.comp.spv"),
    )?;
    let exposure_gather_pipeline = create_exposure_pipeline(
        context,
        exposure_layout,
        include_bytes!("shaders/auto_exposure_gather.comp.spv"),
    )?;
    let pool = create_descriptor_pool(context)?;
    let sampler = create_sampler(context, vk::Filter::LINEAR)?;
    let point_sampler = create_sampler(context, vk::Filter::NEAREST)?;

    let single_layout = create_pipeline_layout(
        context,
        single_descriptor_layout,
        std::mem::size_of::<DownsampleParams>() as u32,
    )?;
    let compose_layout = create_pipeline_layout(
        context,
        compose_descriptor_layout,
        std::mem::size_of::<PostParams>() as u32,
    )?;

    let downsample_pipeline = create_pipeline(
        setup.context,
        single_layout,
        include_bytes!("shaders/post_downsample.frag.spv"),
        setup.hdr_format,
        vk::Format::UNDEFINED,
        false,
    )?;
    let upsample_pipeline = create_pipeline(
        setup.context,
        single_layout,
        include_bytes!("shaders/post_upsample.frag.spv"),
        setup.hdr_format,
        vk::Format::UNDEFINED,
        true,
    )?;
    let compose_pipeline = create_pipeline(
        setup.context,
        compose_layout,
        include_bytes!("shaders/post_compose.frag.spv"),
        setup.color_format,
        vk::Format::UNDEFINED,
        false,
    )?;
    let smooth_layout = create_pipeline_layout(
        context,
        single_descriptor_layout,
        std::mem::size_of::<SmoothParams>() as u32,
    )?;
    let smooth_pipeline = create_pipeline(
        setup.context,
        smooth_layout,
        include_bytes!("shaders/fxaa.frag.spv"),
        setup.color_format,
        vk::Format::UNDEFINED,
        false,
    )?;
    let occlusion_descriptor_layout = create_descriptor_layout(context, 2)?;
    let occlusion_layout = create_pipeline_layout(
        context,
        occlusion_descriptor_layout,
        std::mem::size_of::<OcclusionParams>() as u32,
    )?;
    let occlusion_pipeline = create_pipeline(
        setup.context,
        occlusion_layout,
        include_bytes!("shaders/ssao.frag.spv"),
        OCCLUSION_FORMAT,
        vk::Format::UNDEFINED,
        false,
    )?;
    let blur_descriptor_layout = create_descriptor_layout(context, 3)?;
    let blur_layout = create_pipeline_layout(
        context,
        blur_descriptor_layout,
        std::mem::size_of::<BlurParams>() as u32,
    )?;
    let blur_pipeline = create_pipeline(
        setup.context,
        blur_layout,
        include_bytes!("shaders/ssao_blur.frag.spv"),
        OCCLUSION_FORMAT,
        vk::Format::UNDEFINED,
        false,
    )?;
    let reflection_descriptor_layout = create_descriptor_layout(context, 3)?;
    let reflection_layout = create_pipeline_layout(
        context,
        reflection_descriptor_layout,
        std::mem::size_of::<ReflectionParams>() as u32,
    )?;
    let reflection_pipeline = create_pipeline(
        setup.context,
        reflection_layout,
        include_bytes!("shaders/ssr.frag.spv"),
        setup.hdr_format,
        vk::Format::UNDEFINED,
        false,
    )?;
    let indirect_layout = create_pipeline_layout(
        context,
        reflection_descriptor_layout,
        std::mem::size_of::<IndirectParams>() as u32,
    )?;
    let indirect_pipeline = create_pipeline(
        setup.context,
        indirect_layout,
        include_bytes!("shaders/ssgi.frag.spv"),
        setup.hdr_format,
        vk::Format::UNDEFINED,
        false,
    )?;
    let resolve_descriptor_layout = create_descriptor_layout(context, 4)?;
    let resolve_layout = create_pipeline_layout(
        context,
        resolve_descriptor_layout,
        std::mem::size_of::<ResolveParams>() as u32,
    )?;
    let resolve_pipeline = create_pair_pipeline(
        context,
        resolve_layout,
        include_bytes!("shaders/taa.frag.spv"),
        setup.hdr_format,
    )?;
    let focus_layout = create_pipeline_layout(
        context,
        resolve_descriptor_layout,
        std::mem::size_of::<FocusParams>() as u32,
    )?;
    let focus_pipeline = create_pipeline(
        setup.context,
        focus_layout,
        include_bytes!("shaders/depth_of_field.frag.spv"),
        setup.hdr_format,
        vk::Format::UNDEFINED,
        false,
    )?;
    let screen_blur_layout = create_pipeline_layout(
        context,
        reflection_descriptor_layout,
        std::mem::size_of::<ScreenBlurParams>() as u32,
    )?;
    let screen_blur_pipeline = create_pipeline(
        setup.context,
        screen_blur_layout,
        include_bytes!("shaders/screen_blur.frag.spv"),
        setup.hdr_format,
        vk::Format::UNDEFINED,
        false,
    )?;

    let mut pass = PostPass {
        focus: PostFocus {
            pipeline: focus_pipeline,
            layout: focus_layout,
            image: None,
            set: vk::DescriptorSet::null(),
            written: false,
            params: FocusParams::default(),
        },
        resolve: PostResolve {
            pipeline: resolve_pipeline,
            layout: resolve_layout,
            descriptor_layout: resolve_descriptor_layout,
            image: None,
            kept: Vec::new(),
            sets: Vec::new(),
            ready: false,
            written: false,
            round: 0,
            params: ResolveParams::default(),
        },
        indirect: PostIndirect {
            pipeline: indirect_pipeline,
            layout: indirect_layout,
            image: None,
            set: vk::DescriptorSet::null(),
            written: false,
            params: IndirectParams::default(),
            soft: None,
            blur_set: vk::DescriptorSet::null(),
        },
        reflection: PostReflection {
            pipeline: reflection_pipeline,
            layout: reflection_layout,
            descriptor_layout: reflection_descriptor_layout,
            image: None,
            set: vk::DescriptorSet::null(),
            written: false,
            params: ReflectionParams::default(),
            soft: None,
            blur_set: vk::DescriptorSet::null(),
        },
        occlusion: PostOcclusion {
            pipeline: occlusion_pipeline,
            blur_pipeline,
            layout: occlusion_layout,
            descriptor_layout: occlusion_descriptor_layout,
            blur_layout,
            blur_descriptor_layout,
            images: Vec::new(),
            set: vk::DescriptorSet::null(),
            blur_set: vk::DescriptorSet::null(),
            written: false,
            params: OcclusionParams::default(),
            blur_threshold: 0.005,
            blur_sharpness: 0.0,
        },
        exposure: PostExposure {
            buffer: exposure,
            pipeline: exposure_pipeline,
            gather_pipeline: exposure_gather_pipeline,
            cleared: false,
            layout: exposure_layout,
            descriptor_layout: exposure_descriptor_layout,
            set: vk::DescriptorSet::null(),
            settings: [1.5, 0.016, -3.0, 5.0],
        },
        bloom: PostBloom {
            downsample_pipeline,
            upsample_pipeline,
            compose_pipeline,
            single_layout,
            compose_layout,
            single_descriptor_layout,
            compose_descriptor_layout,
            images: Vec::new(),
            downsample_sets: Vec::new(),
            downsample_resolved_set: vk::DescriptorSet::null(),
            downsample_focus_set: vk::DescriptorSet::null(),
            upsample_sets: Vec::new(),
            compose_set: vk::DescriptorSet::null(),
            threshold: 1.0,
            knee: 0.1,
            filter_radius: 0.005,
        },
        pool,
        sampler,
        point_sampler,
        color_lut: create_color_lut(context)?,
        screen_blur_pipeline,
        screen_blur_layout,
        quiet: None,
        traced: None,
        smooth_pipeline,
        smooth_layout,
        composed: None,
        composed_set: vk::DescriptorSet::null(),
        smooth: None,
        color_format: setup.color_format,
        screen: setup.screen,
        extent,
        hdr_format,
        depth_view: setup.depth,
        motion_view: setup.motion,
        normal_view: setup.normal,
        params: PostParams::default(),
    };
    build_targets(&mut pass, context, extent, scene_view)?;
    Ok(pass)
}

pub fn hand_over(
    pass: &mut PostPass,
    params: PostParams,
    occlusion: OcclusionParams,
    reflection: ReflectionParams,
    indirect: IndirectParams,
    resolve: ResolveParams,
    focus: FocusParams,
    smooth: Option<SmoothParams>,
    exposure: [f32; 4],
    blur_threshold: f32,
    blur_sharpness: f32,
    threshold: f32,
    knee: f32,
    filter_radius: f32,
) {
    pass.params = params;
    pass.smooth = smooth;
    pass.exposure.settings = exposure;
    pass.occlusion.params = occlusion;
    pass.reflection.params = reflection;
    pass.indirect.params = indirect;
    pass.resolve.params = resolve;
    pass.focus.params = focus;
    pass.occlusion.blur_threshold = blur_threshold;
    pass.occlusion.blur_sharpness = blur_sharpness;
    pass.bloom.threshold = threshold;
    pass.bloom.knee = knee;
    pass.bloom.filter_radius = filter_radius;
}

pub fn hand_over_traced(pass: &mut PostPass, context: &Context, inputs: TracedInputs) {
    if let Some(traced) = &mut pass.traced {
        traced.inputs = inputs;
        return;
    }
    match create_traced(context, pass, inputs) {
        Ok(traced) => pass.traced = Some(traced),
        Err(error) => tracing::error!("the gathers that send rays were not made: {error}"),
    }
}

fn create_traced(
    context: &Context,
    pass: &PostPass,
    inputs: TracedInputs,
) -> Result<Traced, Box<dyn std::error::Error>> {
    let reflection_layout = create_traced_layout(
        context,
        &inputs,
        pass.reflection.descriptor_layout,
        std::mem::size_of::<ReflectionParams>() as u32,
    )?;
    let occlusion_layout = create_traced_layout(
        context,
        &inputs,
        pass.occlusion.descriptor_layout,
        std::mem::size_of::<OcclusionParams>() as u32,
    )?;
    let reflection_pipeline = create_pipeline(
        context,
        reflection_layout,
        include_bytes!("shaders/traced_reflection.frag.spv"),
        pass.hdr_format,
        vk::Format::UNDEFINED,
        false,
    )?;
    let occlusion_pipeline = create_pipeline(
        context,
        occlusion_layout,
        include_bytes!("shaders/traced_occlusion.frag.spv"),
        OCCLUSION_FORMAT,
        vk::Format::UNDEFINED,
        false,
    )?;
    Ok(Traced {
        inputs,
        reflection_layout,
        reflection_pipeline,
        occlusion_layout,
        occlusion_pipeline,
    })
}

fn create_traced_layout(
    context: &Context,
    inputs: &TracedInputs,
    own: vk::DescriptorSetLayout,
    push_size: u32,
) -> Result<vk::PipelineLayout, Box<dyn std::error::Error>> {
    let layouts = [
        inputs.scene_layout,
        inputs.texture_layout,
        inputs.shadow_layout,
        inputs.environment_layout,
        own,
    ];
    let range = vk::PushConstantRange::default()
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        .offset(0)
        .size(push_size);
    let create_info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(&layouts)
        .push_constant_ranges(std::slice::from_ref(&range));
    Ok(unsafe { context.device.create_pipeline_layout(&create_info, None)? })
}

fn destroy_traced(context: &Context, traced: &mut Traced) {
    unsafe {
        context
            .device
            .destroy_pipeline(traced.reflection_pipeline, None);
        context
            .device
            .destroy_pipeline_layout(traced.reflection_layout, None);
        context
            .device
            .destroy_pipeline(traced.occlusion_pipeline, None);
        context
            .device
            .destroy_pipeline_layout(traced.occlusion_layout, None);
    }
}

const COMPOSED: &[(Resource, Usage)] = &[
    (Resource::Hdr, Usage::Sampled),
    (Resource::Depth, Usage::Sampled),
    (Resource::Normal, Usage::Sampled),
    (Resource::Motion, Usage::Sampled),
    (Resource::Bloom(0), Usage::Sampled),
    (Resource::Occlusion(1), Usage::Sampled),
    (Resource::ReflectionSoft, Usage::Sampled),
    (Resource::IndirectSoft, Usage::Sampled),
    (Resource::Resolved, Usage::Sampled),
    (Resource::Focus, Usage::Sampled),
    (Resource::Kept(0), Usage::Sampled),
    (Resource::Kept(1), Usage::Sampled),
    (Resource::Composed, Usage::Sampled),
];

impl Pass for PostPass {
    fn name(&self) -> &'static str {
        POST_PASS
    }

    fn usage(&self, scope: ScopeKind) -> &[(Resource, Usage)] {
        match scope {
            ScopeKind::Post => COMPOSED,
            _ => &[],
        }
    }

    fn register(&self, states: &mut ResourceStates) {
        for (level, target) in self.bloom.images.iter().enumerate() {
            register_target(states, Resource::Bloom(level), target);
        }
        for (level, target) in self.occlusion.images.iter().enumerate() {
            register_target(states, Resource::Occlusion(level), target);
        }
        for (level, target) in self.resolve.kept.iter().enumerate() {
            register_target(states, Resource::Kept(level), target);
        }
        if let Some(quiet) = &self.quiet {
            for (turn, target) in quiet.kept.iter().enumerate() {
                register_target(states, Resource::ReflectionKept(turn), target);
            }
            for (turn, target) in quiet.moments.iter().enumerate() {
                register_target(states, Resource::ReflectionMoments(turn), target);
            }
            for (turn, target) in quiet.wave.iter().enumerate() {
                register_target(states, Resource::ReflectionWave(turn), target);
            }
            for (turn, target) in quiet.noise.iter().enumerate() {
                register_target(states, Resource::ReflectionNoise(turn), target);
            }
        }
        let named = [
            (Resource::Reflection, &self.reflection.image),
            (Resource::ReflectionSoft, &self.reflection.soft),
            (Resource::Indirect, &self.indirect.image),
            (Resource::IndirectSoft, &self.indirect.soft),
            (Resource::Resolved, &self.resolve.image),
            (Resource::Focus, &self.focus.image),
            (Resource::Composed, &self.composed),
        ];
        for (resource, target) in named {
            if let Some(target) = target {
                register_target(states, resource, target);
            }
        }
    }

    fn offscreen(&mut self, record: &mut PassRecord) {
        self.measure_brightness(record);
        self.gather_occlusion(record);
        self.trace_reflections(record);
        self.gather_indirect(record);
        self.resolve_over_time(record);
        self.spread_out_of_focus(record);
        self.gather_bloom(record);
        self.compose_apart(record);
    }

    fn draw(&mut self, record: &mut PassRecord) {
        let device = record.context.device.clone();
        match (self.smooth, &self.composed) {
            (Some(smooth), Some(_)) => self.smooth_edges(&device, record.command_buffer, smooth),
            _ => self.compose(&device, record.command_buffer),
        }
    }

    fn retarget(&mut self, setup: &PassSetup<'_>) {
        let compose = match create_pipeline(
            setup.context,
            self.bloom.compose_layout,
            include_bytes!("shaders/post_compose.frag.spv"),
            setup.color_format,
            vk::Format::UNDEFINED,
            false,
        ) {
            Ok(compose) => compose,
            Err(error) => {
                tracing::error!("the compose was not made for the new screen format: {error}");
                return;
            }
        };
        let smooth = match create_pipeline(
            setup.context,
            self.smooth_layout,
            include_bytes!("shaders/fxaa.frag.spv"),
            setup.color_format,
            vk::Format::UNDEFINED,
            false,
        ) {
            Ok(smooth) => smooth,
            Err(error) => {
                tracing::error!("the smoothing was not made for the new screen format: {error}");
                unsafe { setup.context.device.destroy_pipeline(compose, None) };
                return;
            }
        };
        unsafe {
            setup
                .context
                .device
                .destroy_pipeline(self.bloom.compose_pipeline, None);
            setup
                .context
                .device
                .destroy_pipeline(self.smooth_pipeline, None);
        }
        self.bloom.compose_pipeline = compose;
        self.smooth_pipeline = smooth;
        self.color_format = setup.color_format;
        if let Some(target) = &mut self.composed {
            destroy_image(setup.context, target);
        }
        self.composed = None;
    }

    fn resize(&mut self, context: &Context, targets: ScreenTargets) {
        for target in &mut self.bloom.images {
            destroy_image(context, target);
        }
        self.bloom.images.clear();
        for target in &mut self.occlusion.images {
            destroy_image(context, target);
        }
        self.occlusion.images.clear();
        self.occlusion.written = false;
        if let Some(target) = &mut self.reflection.image {
            destroy_image(context, target);
        }
        self.reflection.image = None;
        self.reflection.written = false;
        if let Some(target) = &mut self.indirect.image {
            destroy_image(context, target);
        }
        self.indirect.image = None;
        self.indirect.written = false;
        if let Some(target) = &mut self.resolve.image {
            destroy_image(context, target);
        }
        self.resolve.image = None;
        for target in &mut self.resolve.kept {
            destroy_image(context, target);
        }
        self.resolve.kept.clear();
        self.resolve.sets.clear();
        self.resolve.ready = false;
        self.resolve.written = false;
        self.resolve.round = 0;
        if let Some(target) = &mut self.focus.image {
            destroy_image(context, target);
        }
        self.focus.image = None;
        self.focus.written = false;
        for target in [&mut self.reflection.soft, &mut self.indirect.soft] {
            if let Some(target) = target {
                destroy_image(context, target);
            }
            *target = None;
        }
        if let Some(target) = &mut self.composed {
            destroy_image(context, target);
        }
        self.composed = None;
        if let Some(mut quiet) = self.quiet.take() {
            destroy_quiet(context, &mut quiet);
        }
        self.screen = targets.screen;
        self.depth_view = targets.depth;
        self.motion_view = targets.motion;
        self.normal_view = targets.normal;
        if let Err(error) = unsafe {
            context
                .device
                .reset_descriptor_pool(self.pool, vk::DescriptorPoolResetFlags::empty())
        } {
            tracing::error!("the post descriptor pool was not reset: {error}");
        }
        self.bloom.downsample_sets.clear();
        self.bloom.upsample_sets.clear();
        if let Err(error) = build_targets(self, context, targets.extent, targets.color) {
            tracing::error!("the post targets were not rebuilt: {error}");
        }
    }

    fn destroy(&mut self, context: &Context) {
        unsafe {
            context
                .device
                .destroy_pipeline(self.exposure.pipeline, None);
            context
                .device
                .destroy_pipeline(self.exposure.gather_pipeline, None);
            context
                .device
                .destroy_pipeline_layout(self.exposure.layout, None);
            context
                .device
                .destroy_descriptor_set_layout(self.exposure.descriptor_layout, None);
        }
        destroy_buffer(context, &mut self.exposure.buffer);
        destroy_color_lut(context, &mut self.color_lut);
        if let Some(mut quiet) = self.quiet.take() {
            destroy_quiet(context, &mut quiet);
        }
        for target in &mut self.bloom.images {
            destroy_image(context, target);
        }
        self.bloom.images.clear();
        for target in &mut self.occlusion.images {
            destroy_image(context, target);
        }
        self.occlusion.images.clear();
        if let Some(target) = &mut self.reflection.image {
            destroy_image(context, target);
        }
        self.reflection.image = None;
        if let Some(target) = &mut self.indirect.image {
            destroy_image(context, target);
        }
        self.indirect.image = None;
        if let Some(target) = &mut self.resolve.image {
            destroy_image(context, target);
        }
        self.resolve.image = None;
        for target in &mut self.resolve.kept {
            destroy_image(context, target);
        }
        self.resolve.kept.clear();
        if let Some(target) = &mut self.focus.image {
            destroy_image(context, target);
        }
        self.focus.image = None;
        for target in [&mut self.reflection.soft, &mut self.indirect.soft] {
            if let Some(target) = target {
                destroy_image(context, target);
            }
            *target = None;
        }
        if let Some(target) = &mut self.composed {
            destroy_image(context, target);
        }
        self.composed = None;
        unsafe {
            context
                .device
                .destroy_pipeline(self.screen_blur_pipeline, None);
            context
                .device
                .destroy_pipeline_layout(self.screen_blur_layout, None);
            context.device.destroy_pipeline(self.focus.pipeline, None);
            context
                .device
                .destroy_pipeline_layout(self.focus.layout, None);
            context.device.destroy_pipeline(self.resolve.pipeline, None);
            context
                .device
                .destroy_pipeline_layout(self.resolve.layout, None);
            context
                .device
                .destroy_descriptor_set_layout(self.resolve.descriptor_layout, None);
            context
                .device
                .destroy_pipeline(self.indirect.pipeline, None);
            context
                .device
                .destroy_pipeline_layout(self.indirect.layout, None);
            context
                .device
                .destroy_pipeline(self.reflection.pipeline, None);
            context
                .device
                .destroy_pipeline_layout(self.reflection.layout, None);
            context
                .device
                .destroy_descriptor_set_layout(self.reflection.descriptor_layout, None);
            context
                .device
                .destroy_pipeline(self.occlusion.pipeline, None);
            context
                .device
                .destroy_pipeline(self.occlusion.blur_pipeline, None);
            context
                .device
                .destroy_pipeline_layout(self.occlusion.layout, None);
            context
                .device
                .destroy_pipeline_layout(self.occlusion.blur_layout, None);
            context
                .device
                .destroy_descriptor_set_layout(self.occlusion.descriptor_layout, None);
            context
                .device
                .destroy_descriptor_set_layout(self.occlusion.blur_descriptor_layout, None);
            context.device.destroy_sampler(self.sampler, None);
            context.device.destroy_sampler(self.point_sampler, None);
            context
                .device
                .destroy_pipeline(self.bloom.downsample_pipeline, None);
            context
                .device
                .destroy_pipeline(self.bloom.upsample_pipeline, None);
            context
                .device
                .destroy_pipeline(self.bloom.compose_pipeline, None);
            context.device.destroy_pipeline(self.smooth_pipeline, None);
            context
                .device
                .destroy_pipeline_layout(self.smooth_layout, None);
            if let Some(mut traced) = self.traced.take() {
                destroy_traced(context, &mut traced);
            }
            context
                .device
                .destroy_pipeline_layout(self.bloom.single_layout, None);
            context
                .device
                .destroy_pipeline_layout(self.bloom.compose_layout, None);
            context.device.destroy_descriptor_pool(self.pool, None);
            context
                .device
                .destroy_descriptor_set_layout(self.bloom.single_descriptor_layout, None);
            context
                .device
                .destroy_descriptor_set_layout(self.bloom.compose_descriptor_layout, None);
        }
    }
}

fn create_quiet(
    pass: &PostPass,
    context: &Context,
    extent: vk::Extent2D,
    noisy: vk::ImageView,
) -> Result<Quiet, Box<dyn std::error::Error>> {
    let gather_descriptor_layout = create_descriptor_layout(context, 6)?;
    let variance_descriptor_layout = create_descriptor_layout(context, 4)?;
    let wavelet_descriptor_layout = create_descriptor_layout(context, 4)?;

    let gather_layout = create_pipeline_layout(
        context,
        gather_descriptor_layout,
        std::mem::size_of::<QuietParams>() as u32,
    )?;
    let variance_layout = create_pipeline_layout(
        context,
        variance_descriptor_layout,
        std::mem::size_of::<WaveParams>() as u32,
    )?;
    let wavelet_layout = create_pipeline_layout(
        context,
        wavelet_descriptor_layout,
        std::mem::size_of::<WaveParams>() as u32,
    )?;

    let gather_pipeline = create_pair_pipeline(
        context,
        gather_layout,
        include_bytes!("shaders/svgf_gather.frag.spv"),
        pass.hdr_format,
    )?;
    let variance_pipeline = create_pair_pipeline(
        context,
        variance_layout,
        include_bytes!("shaders/svgf_variance.frag.spv"),
        pass.hdr_format,
    )?;
    let wavelet_pipeline = create_pair_pipeline(
        context,
        wavelet_layout,
        include_bytes!("shaders/svgf_wavelet.frag.spv"),
        pass.hdr_format,
    )?;

    let made = |name: &str| create_color_image(context, extent, pass.hdr_format, name);
    let kept = [made("reflection gathered")?, made("reflection gathered")?];
    let moments = [made("reflection means")?, made("reflection means")?];
    let wave = [made("reflection wave")?, made("reflection wave")?];
    let noise = [made("reflection noise")?, made("reflection noise")?];

    let mut gather_sets = [vk::DescriptorSet::null(); 2];
    let mut variance_sets = [vk::DescriptorSet::null(); 2];
    let mut wavelet_sets = [vk::DescriptorSet::null(); 2];
    for turn in 0..2 {
        let before = 1 - turn;
        gather_sets[turn] = allocate_set(context, pass.pool, gather_descriptor_layout)?;
        write_mixed_set(
            context,
            gather_sets[turn],
            &[
                (noisy, pass.point_sampler),
                (kept[before].view, pass.sampler),
                (moments[before].view, pass.sampler),
                (pass.depth_view, pass.point_sampler),
                (pass.normal_view, pass.point_sampler),
                (pass.motion_view, pass.point_sampler),
            ],
        );
        variance_sets[turn] = allocate_set(context, pass.pool, variance_descriptor_layout)?;
        write_mixed_set(
            context,
            variance_sets[turn],
            &[
                (kept[turn].view, pass.point_sampler),
                (moments[turn].view, pass.point_sampler),
                (pass.depth_view, pass.point_sampler),
                (pass.normal_view, pass.point_sampler),
            ],
        );
        wavelet_sets[turn] = allocate_set(context, pass.pool, wavelet_descriptor_layout)?;
        write_mixed_set(
            context,
            wavelet_sets[turn],
            &[
                (wave[turn].view, pass.point_sampler),
                (noise[turn].view, pass.point_sampler),
                (pass.depth_view, pass.point_sampler),
                (pass.normal_view, pass.point_sampler),
            ],
        );
    }

    Ok(Quiet {
        kept,
        moments,
        wave,
        noise,
        gather_sets,
        variance_sets,
        wavelet_sets,
        gather_layout,
        gather_pipeline,
        variance_layout,
        variance_pipeline,
        wavelet_layout,
        wavelet_pipeline,
        gather_descriptor_layout,
        variance_descriptor_layout,
        wavelet_descriptor_layout,
        turn: 0,
        standing: false,
    })
}

fn destroy_quiet(context: &Context, quiet: &mut Quiet) {
    unsafe {
        context.device.destroy_pipeline(quiet.gather_pipeline, None);
        context
            .device
            .destroy_pipeline(quiet.variance_pipeline, None);
        context
            .device
            .destroy_pipeline(quiet.wavelet_pipeline, None);
        context
            .device
            .destroy_pipeline_layout(quiet.gather_layout, None);
        context
            .device
            .destroy_pipeline_layout(quiet.variance_layout, None);
        context
            .device
            .destroy_pipeline_layout(quiet.wavelet_layout, None);
        context
            .device
            .destroy_descriptor_set_layout(quiet.gather_descriptor_layout, None);
        context
            .device
            .destroy_descriptor_set_layout(quiet.variance_descriptor_layout, None);
        context
            .device
            .destroy_descriptor_set_layout(quiet.wavelet_descriptor_layout, None);
    }
    for image in quiet
        .kept
        .iter_mut()
        .chain(quiet.moments.iter_mut())
        .chain(quiet.wave.iter_mut())
        .chain(quiet.noise.iter_mut())
    {
        destroy_image(context, image);
    }
}

fn build_targets(
    pass: &mut PostPass,
    context: &Context,
    extent: vk::Extent2D,
    scene_view: vk::ImageView,
) -> Result<(), Box<dyn std::error::Error>> {
    pass.extent = extent;

    let mut width = extent.width;
    let mut height = extent.height;
    for _ in 0..BLOOM_LEVELS {
        width = (width / 2).max(1);
        height = (height / 2).max(1);
        if width == 1 && height == 1 {
            break;
        }
        pass.bloom.images.push(create_color_image(
            context,
            vk::Extent2D { width, height },
            pass.hdr_format,
            "bloom",
        )?);
    }

    for level in 0..pass.bloom.images.len() {
        let source = if level == 0 {
            scene_view
        } else {
            pass.bloom.images[level - 1].view
        };
        let set = allocate_set(context, pass.pool, pass.bloom.single_descriptor_layout)?;
        write_set(context, set, &[source], pass.sampler);
        pass.bloom.downsample_sets.push(set);
    }

    for level in 0..pass.bloom.images.len().saturating_sub(1) {
        let set = allocate_set(context, pass.pool, pass.bloom.single_descriptor_layout)?;
        write_set(
            context,
            set,
            &[pass.bloom.images[level + 1].view],
            pass.sampler,
        );
        pass.bloom.upsample_sets.push(set);
    }

    let half = half_of(extent);
    for size in [half, extent] {
        pass.occlusion.images.push(create_color_image(
            context,
            size,
            OCCLUSION_FORMAT,
            "occlusion",
        )?);
    }
    pass.occlusion.set = allocate_set(context, pass.pool, pass.occlusion.descriptor_layout)?;
    write_set(
        context,
        pass.occlusion.set,
        &[pass.depth_view, pass.normal_view],
        pass.point_sampler,
    );
    pass.occlusion.blur_set =
        allocate_set(context, pass.pool, pass.occlusion.blur_descriptor_layout)?;
    write_set(
        context,
        pass.occlusion.blur_set,
        &[
            pass.occlusion.images[0].view,
            pass.depth_view,
            pass.normal_view,
        ],
        pass.point_sampler,
    );

    let reflection = create_color_image(context, half, pass.hdr_format, "reflection")?;
    pass.reflection.set = allocate_set(context, pass.pool, pass.reflection.descriptor_layout)?;
    write_mixed_set(
        context,
        pass.reflection.set,
        &[
            (pass.depth_view, pass.point_sampler),
            (pass.normal_view, pass.point_sampler),
            (scene_view, pass.sampler),
        ],
    );
    let reflection_view = reflection.view;
    pass.reflection.image = Some(reflection);
    let reflection_soft =
        create_color_image(context, extent, pass.hdr_format, "reflection softened")?;
    let reflection_soft_view = reflection_soft.view;
    pass.reflection.soft = Some(reflection_soft);

    let quiet = create_quiet(pass, context, half, reflection_view)?;
    let quieted = quiet.wave[QUIET_ROUNDS as usize % 2].view;
    pass.quiet = Some(quiet);

    pass.reflection.blur_set = allocate_set(context, pass.pool, pass.reflection.descriptor_layout)?;
    write_mixed_set(
        context,
        pass.reflection.blur_set,
        &[
            (quieted, pass.sampler),
            (pass.depth_view, pass.point_sampler),
            (pass.normal_view, pass.point_sampler),
        ],
    );

    let indirect = create_color_image(context, half, pass.hdr_format, "indirect light")?;
    pass.indirect.set = allocate_set(context, pass.pool, pass.reflection.descriptor_layout)?;
    write_mixed_set(
        context,
        pass.indirect.set,
        &[
            (pass.depth_view, pass.point_sampler),
            (pass.normal_view, pass.point_sampler),
            (scene_view, pass.sampler),
        ],
    );
    let indirect_view = indirect.view;
    pass.indirect.image = Some(indirect);
    let indirect_soft = create_color_image(context, extent, pass.hdr_format, "indirect softened")?;
    let indirect_soft_view = indirect_soft.view;
    pass.indirect.soft = Some(indirect_soft);
    pass.indirect.blur_set = allocate_set(context, pass.pool, pass.reflection.descriptor_layout)?;
    write_mixed_set(
        context,
        pass.indirect.blur_set,
        &[
            (indirect_view, pass.sampler),
            (pass.depth_view, pass.point_sampler),
            (pass.normal_view, pass.point_sampler),
        ],
    );

    let resolved = create_color_image(context, extent, pass.hdr_format, "resolved")?;
    let resolved_view = resolved.view;
    pass.resolve.image = Some(resolved);
    for _ in 0..2 {
        pass.resolve.kept.push(create_color_image(
            context,
            extent,
            pass.hdr_format,
            "resolve history",
        )?);
    }
    for round in 0..2 {
        let set = allocate_set(context, pass.pool, pass.resolve.descriptor_layout)?;
        write_mixed_set(
            context,
            set,
            &[
                (scene_view, pass.sampler),
                (pass.resolve.kept[round].view, pass.sampler),
                (pass.depth_view, pass.point_sampler),
                (pass.motion_view, pass.point_sampler),
            ],
        );
        pass.resolve.sets.push(set);
    }

    let focus = create_color_image(context, extent, pass.hdr_format, "focus")?;
    let focus_view = focus.view;
    pass.focus.image = Some(focus);
    pass.focus.set = allocate_set(context, pass.pool, pass.resolve.descriptor_layout)?;
    write_mixed_set(
        context,
        pass.focus.set,
        &[
            (scene_view, pass.sampler),
            (resolved_view, pass.sampler),
            (pass.depth_view, pass.point_sampler),
        ],
    );
    pass.bloom.downsample_resolved_set =
        allocate_set(context, pass.pool, pass.bloom.single_descriptor_layout)?;
    write_set(
        context,
        pass.bloom.downsample_resolved_set,
        &[resolved_view],
        pass.sampler,
    );
    pass.bloom.downsample_focus_set =
        allocate_set(context, pass.pool, pass.bloom.single_descriptor_layout)?;
    write_set(
        context,
        pass.bloom.downsample_focus_set,
        &[focus_view],
        pass.sampler,
    );

    let composed = create_color_image(context, extent, pass.color_format, "composed")?;
    pass.composed_set = allocate_set(context, pass.pool, pass.bloom.single_descriptor_layout)?;
    write_set(context, pass.composed_set, &[composed.view], pass.sampler);
    pass.composed = Some(composed);

    let bloom_view = pass
        .bloom
        .images
        .first()
        .map(|image| image.view)
        .unwrap_or(scene_view);
    let lifting = extent.width < pass.screen.width || extent.height < pass.screen.height;
    let lift = if lifting {
        pass.point_sampler
    } else {
        pass.sampler
    };
    pass.bloom.compose_set =
        allocate_set(context, pass.pool, pass.bloom.compose_descriptor_layout)?;
    write_mixed_set(
        context,
        pass.bloom.compose_set,
        &[
            (scene_view, lift),
            (bloom_view, pass.sampler),
            (pass.occlusion.images[1].view, pass.sampler),
            (reflection_soft_view, pass.sampler),
            (indirect_soft_view, pass.sampler),
            (resolved_view, lift),
            (focus_view, lift),
        ],
    );
    write_exposure_binding(context, pass.bloom.compose_set, 7, &pass.exposure.buffer);
    write_lut_binding(context, pass.bloom.compose_set, 8, &pass.color_lut);

    pass.exposure.set = allocate_set(context, pass.pool, pass.exposure.descriptor_layout)?;
    write_set(context, pass.exposure.set, &[scene_view], pass.sampler);
    write_exposure_binding(context, pass.exposure.set, 1, &pass.exposure.buffer);
    Ok(())
}

fn half_of(extent: vk::Extent2D) -> vk::Extent2D {
    vk::Extent2D {
        width: (extent.width / 2).max(1),
        height: (extent.height / 2).max(1),
    }
}

fn write_lut_binding(context: &Context, set: vk::DescriptorSet, binding: u32, lut: &ColorLut) {
    let info = vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(lut.view)
        .sampler(lut.sampler);
    let write = vk::WriteDescriptorSet::default()
        .dst_set(set)
        .dst_binding(binding)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .image_info(std::slice::from_ref(&info));
    unsafe {
        context
            .device
            .update_descriptor_sets(std::slice::from_ref(&write), &[])
    };
}

fn write_exposure_binding(
    context: &Context,
    set: vk::DescriptorSet,
    binding: u32,
    buffer: &Buffer,
) {
    let info = vk::DescriptorBufferInfo::default()
        .buffer(buffer.buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE);
    let write = vk::WriteDescriptorSet::default()
        .dst_set(set)
        .dst_binding(binding)
        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
        .buffer_info(std::slice::from_ref(&info));
    unsafe {
        context
            .device
            .update_descriptor_sets(std::slice::from_ref(&write), &[])
    };
}

fn write_mixed_set(
    context: &Context,
    set: vk::DescriptorSet,
    views: &[(vk::ImageView, vk::Sampler)],
) {
    let infos: Vec<vk::DescriptorImageInfo> = views
        .iter()
        .map(|(view, sampler)| {
            vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(*view)
                .sampler(*sampler)
        })
        .collect();
    let writes: Vec<vk::WriteDescriptorSet> = infos
        .iter()
        .enumerate()
        .map(|(binding, info)| {
            vk::WriteDescriptorSet::default()
                .dst_set(set)
                .dst_binding(binding as u32)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(info))
        })
        .collect();
    unsafe { context.device.update_descriptor_sets(&writes, &[]) };
}

impl PostPass {
    fn gather_bloom(&mut self, record: &mut PassRecord) {
        if self.params.bloom_enabled == 0 || self.bloom.images.is_empty() {
            return;
        }
        let device = record.context.device.clone();
        for level in 0..self.bloom.images.len() {
            claim_target(record, Resource::Bloom(level));
            begin_target(
                &device,
                record.command_buffer,
                target_of(&self.bloom.images[level]),
            );
            unsafe {
                device.cmd_bind_pipeline(
                    record.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.bloom.downsample_pipeline,
                );
                let source = if level > 0 {
                    self.bloom.downsample_sets[level]
                } else if self.params.focus_enabled != 0 && self.focus.written {
                    self.bloom.downsample_focus_set
                } else if self.params.resolved_enabled != 0 && self.resolve.written {
                    self.bloom.downsample_resolved_set
                } else {
                    self.bloom.downsample_sets[0]
                };
                device.cmd_bind_descriptor_sets(
                    record.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.bloom.single_layout,
                    0,
                    &[source],
                    &[],
                );
                let params = DownsampleParams {
                    threshold: self.bloom.threshold,
                    knee: self.bloom.knee,
                    exposure: self.params.exposure,
                    level: level as u32,
                };
                device.cmd_push_constants(
                    record.command_buffer,
                    self.bloom.single_layout,
                    vk::ShaderStageFlags::FRAGMENT,
                    0,
                    bytemuck::bytes_of(&params),
                );
                device.cmd_draw(record.command_buffer, 3, 1, 0, 0);
            }
            end_target(record, Resource::Bloom(level));
        }

        for level in (0..self.bloom.images.len() - 1).rev() {
            read_to_attachment(record, Resource::Bloom(level));
            begin_target_loading(
                &device,
                record.command_buffer,
                target_of(&self.bloom.images[level]),
            );
            unsafe {
                device.cmd_bind_pipeline(
                    record.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.bloom.upsample_pipeline,
                );
                device.cmd_bind_descriptor_sets(
                    record.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.bloom.single_layout,
                    0,
                    &[self.bloom.upsample_sets[level]],
                    &[],
                );
                let params = UpsampleParams {
                    filter_radius: self.bloom.filter_radius,
                };
                device.cmd_push_constants(
                    record.command_buffer,
                    self.bloom.single_layout,
                    vk::ShaderStageFlags::FRAGMENT,
                    0,
                    bytemuck::bytes_of(&params),
                );
                device.cmd_draw(record.command_buffer, 3, 1, 0, 0);
            }
            end_target(record, Resource::Bloom(level));
        }
    }

    fn compose_apart(&mut self, record: &mut PassRecord) {
        let (Some(composed), true) = (&self.composed, self.smooth.is_some()) else {
            return;
        };
        let device = record.context.device.clone();
        let composed = target_of(composed);
        claim_target(record, Resource::Composed);
        begin_target(&device, record.command_buffer, composed);
        self.compose(&device, record.command_buffer);
        end_target(record, Resource::Composed);
    }

    fn compose(&self, device: &ash::Device, command_buffer: vk::CommandBuffer) {
        unsafe {
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.bloom.compose_pipeline,
            );
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.bloom.compose_layout,
                0,
                &[self.bloom.compose_set],
                &[],
            );
            device.cmd_push_constants(
                command_buffer,
                self.bloom.compose_layout,
                vk::ShaderStageFlags::FRAGMENT,
                0,
                bytemuck::bytes_of(&self.params),
            );
            device.cmd_draw(command_buffer, 3, 1, 0, 0);
        }
    }

    fn smooth_edges(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        smooth: SmoothParams,
    ) {
        unsafe {
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.smooth_pipeline,
            );
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.smooth_layout,
                0,
                &[self.composed_set],
                &[],
            );
            device.cmd_push_constants(
                command_buffer,
                self.smooth_layout,
                vk::ShaderStageFlags::FRAGMENT,
                0,
                bytemuck::bytes_of(&smooth),
            );
            device.cmd_draw(command_buffer, 3, 1, 0, 0);
        }
    }

    fn measure_brightness(&mut self, record: &mut PassRecord) {
        if self.params.auto_exposure_target <= 0.0 {
            return;
        }
        record.require(Resource::Hdr, Usage::SampledByCompute);
        let device = &record.context.device;
        let width = self.resolve.params.screen[0].max(1.0) as u32;
        let height = self.resolve.params.screen[1].max(1.0) as u32;
        if !self.exposure.cleared {
            unsafe {
                device.cmd_fill_buffer(
                    record.command_buffer,
                    self.exposure.buffer.buffer,
                    0,
                    vk::WHOLE_SIZE,
                    0,
                );
            }
            exposure_barrier(
                device,
                record.command_buffer,
                self.exposure.buffer.buffer,
                vk::PipelineStageFlags2::TRANSFER,
                vk::AccessFlags2::TRANSFER_WRITE,
                vk::PipelineStageFlags2::COMPUTE_SHADER,
                vk::AccessFlags2::SHADER_STORAGE_READ | vk::AccessFlags2::SHADER_STORAGE_WRITE,
            );
            self.exposure.cleared = true;
        }
        unsafe {
            device.cmd_bind_descriptor_sets(
                record.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.exposure.layout,
                0,
                &[self.exposure.set],
                &[],
            );
            device.cmd_push_constants(
                record.command_buffer,
                self.exposure.layout,
                vk::ShaderStageFlags::COMPUTE,
                0,
                bytemuck::bytes_of(&self.exposure.settings),
            );
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.exposure.gather_pipeline,
            );
            device.cmd_dispatch(
                record.command_buffer,
                width.div_ceil(EXPOSURE_GATHER_SPAN),
                height.div_ceil(EXPOSURE_GATHER_SPAN),
                1,
            );
        }
        exposure_barrier(
            device,
            record.command_buffer,
            self.exposure.buffer.buffer,
            vk::PipelineStageFlags2::COMPUTE_SHADER,
            vk::AccessFlags2::SHADER_STORAGE_WRITE,
            vk::PipelineStageFlags2::COMPUTE_SHADER,
            vk::AccessFlags2::SHADER_STORAGE_READ | vk::AccessFlags2::SHADER_STORAGE_WRITE,
        );
        unsafe {
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.exposure.pipeline,
            );
            device.cmd_dispatch(record.command_buffer, 1, 1, 1);
        }
        exposure_barrier(
            device,
            record.command_buffer,
            self.exposure.buffer.buffer,
            vk::PipelineStageFlags2::COMPUTE_SHADER,
            vk::AccessFlags2::SHADER_STORAGE_WRITE,
            vk::PipelineStageFlags2::FRAGMENT_SHADER | vk::PipelineStageFlags2::COMPUTE_SHADER,
            vk::AccessFlags2::SHADER_STORAGE_READ | vk::AccessFlags2::SHADER_STORAGE_WRITE,
        );
    }

    fn gather_occlusion(&mut self, record: &mut PassRecord) {
        let wanted = self.params.occlusion_enabled != 0 || self.params.occlusion_only != 0;
        if self.occlusion.images.len() < 2 || (!wanted && self.occlusion.written) {
            return;
        }
        let device = record.context.device.clone();

        claim_target(record, Resource::Occlusion(0));
        begin_target(
            &device,
            record.command_buffer,
            target_of(&self.occlusion.images[0]),
        );
        let traced = self
            .traced
            .as_ref()
            .filter(|_| self.occlusion.params.traced[0] > 0.0);
        unsafe {
            match traced {
                Some(traced) => {
                    device.cmd_bind_pipeline(
                        record.command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        traced.occlusion_pipeline,
                    );
                    bind_traced(
                        &device,
                        record.command_buffer,
                        traced,
                        traced.occlusion_layout,
                        record.frame_index,
                        self.occlusion.set,
                    );
                    device.cmd_push_constants(
                        record.command_buffer,
                        traced.occlusion_layout,
                        vk::ShaderStageFlags::FRAGMENT,
                        0,
                        bytemuck::bytes_of(&self.occlusion.params),
                    );
                }
                None => {
                    device.cmd_bind_pipeline(
                        record.command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.occlusion.pipeline,
                    );
                    device.cmd_bind_descriptor_sets(
                        record.command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.occlusion.layout,
                        0,
                        &[self.occlusion.set],
                        &[],
                    );
                    device.cmd_push_constants(
                        record.command_buffer,
                        self.occlusion.layout,
                        vk::ShaderStageFlags::FRAGMENT,
                        0,
                        bytemuck::bytes_of(&self.occlusion.params),
                    );
                }
            }
            device.cmd_draw(record.command_buffer, 3, 1, 0, 0);
        }
        end_target(record, Resource::Occlusion(0));

        claim_target(record, Resource::Occlusion(1));
        begin_target(
            &device,
            record.command_buffer,
            target_of(&self.occlusion.images[1]),
        );
        unsafe {
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.occlusion.blur_pipeline,
            );
            device.cmd_bind_descriptor_sets(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.occlusion.blur_layout,
                0,
                &[self.occlusion.blur_set],
                &[],
            );
            let gathered = self.occlusion.images[0].extent;
            let params = BlurParams {
                texel: [
                    1.0 / gathered.width.max(1) as f32,
                    1.0 / gathered.height.max(1) as f32,
                    self.extent.width as f32,
                    self.extent.height as f32,
                ],
                settings: [
                    self.occlusion.blur_threshold,
                    self.occlusion.blur_sharpness,
                    0.0,
                    0.0,
                ],
            };
            device.cmd_push_constants(
                record.command_buffer,
                self.occlusion.blur_layout,
                vk::ShaderStageFlags::FRAGMENT,
                0,
                bytemuck::bytes_of(&params),
            );
            device.cmd_draw(record.command_buffer, 3, 1, 0, 0);
        }
        end_target(record, Resource::Occlusion(1));
        self.occlusion.written = true;
    }

    fn trace_reflections(&mut self, record: &mut PassRecord) {
        let Some(target) = &self.reflection.image else {
            return;
        };
        if self.params.reflection_enabled == 0 && self.reflection.written {
            return;
        }
        let device = record.context.device.clone();
        let target = target_of(target);
        let mut params = self.reflection.params;
        params.screen[0] = target.extent.width as f32;
        params.screen[1] = target.extent.height as f32;
        let traced = self.traced.as_ref().filter(|_| params.screen[2] > 0.0);
        claim_target(record, Resource::Reflection);
        begin_target(&device, record.command_buffer, target);
        unsafe {
            match traced {
                Some(traced) => {
                    device.cmd_bind_pipeline(
                        record.command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        traced.reflection_pipeline,
                    );
                    bind_traced(
                        &device,
                        record.command_buffer,
                        traced,
                        traced.reflection_layout,
                        record.frame_index,
                        self.reflection.set,
                    );
                    device.cmd_push_constants(
                        record.command_buffer,
                        traced.reflection_layout,
                        vk::ShaderStageFlags::FRAGMENT,
                        0,
                        bytemuck::bytes_of(&params),
                    );
                }
                None => {
                    device.cmd_bind_pipeline(
                        record.command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.reflection.pipeline,
                    );
                    device.cmd_bind_descriptor_sets(
                        record.command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.reflection.layout,
                        0,
                        &[self.reflection.set],
                        &[],
                    );
                    device.cmd_push_constants(
                        record.command_buffer,
                        self.reflection.layout,
                        vk::ShaderStageFlags::FRAGMENT,
                        0,
                        bytemuck::bytes_of(&params),
                    );
                }
            }
            device.cmd_draw(record.command_buffer, 3, 1, 0, 0);
        }
        end_target(record, Resource::Reflection);
        self.quiet_the_reflection(record, target.extent);
        self.soften(
            record,
            Resource::ReflectionSoft,
            self.reflection.soft.as_ref().map(target_of),
            target.extent,
            self.reflection.blur_set,
            REFLECTION_BLUR,
            true,
        );
        self.reflection.written = true;
    }

    fn quiet_the_reflection(&mut self, record: &mut PassRecord, extent: vk::Extent2D) {
        let Some(quiet) = &mut self.quiet else {
            return;
        };
        let device = record.context.device.clone();
        let turn = quiet.turn;
        let before = 1 - turn;
        let gathered = QuietParams {
            reprojection: self.resolve.params.reprojection,
            screen: [
                extent.width as f32,
                extent.height as f32,
                if quiet.standing { 1.0 } else { 0.0 },
                QUIET_STANDING,
            ],
            edges: [
                QUIET_DEPTH,
                QUIET_NORMAL,
                QUIET_BRIGHTNESS,
                QUIET_HISTORY_ROOM,
            ],
        };

        record.require(Resource::Reflection, Usage::Sampled);
        read_target(record, Resource::ReflectionKept(before));
        read_target(record, Resource::ReflectionMoments(before));
        claim_target(record, Resource::ReflectionKept(turn));
        claim_target(record, Resource::ReflectionMoments(turn));
        begin_pair(
            &device,
            record.command_buffer,
            target_of(&quiet.kept[turn]),
            target_of(&quiet.moments[turn]),
        );
        unsafe {
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                quiet.gather_pipeline,
            );
            device.cmd_bind_descriptor_sets(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                quiet.gather_layout,
                0,
                &[quiet.gather_sets[turn]],
                &[],
            );
            device.cmd_push_constants(
                record.command_buffer,
                quiet.gather_layout,
                vk::ShaderStageFlags::FRAGMENT,
                0,
                bytemuck::bytes_of(&gathered),
            );
            device.cmd_draw(record.command_buffer, 3, 1, 0, 0);
            device.cmd_end_rendering(record.command_buffer);
        }
        read_target(record, Resource::ReflectionKept(turn));
        read_target(record, Resource::ReflectionMoments(turn));

        let wave = WaveParams {
            screen: [
                extent.width as f32,
                extent.height as f32,
                QUIET_TRUSTED,
                QUIET_SPREAD,
            ],
            edges: [QUIET_DEPTH, QUIET_NORMAL, QUIET_BRIGHTNESS, 0.0],
        };
        read_target(record, Resource::ReflectionKept(turn));
        read_target(record, Resource::ReflectionMoments(turn));
        claim_target(record, Resource::ReflectionWave(0));
        claim_target(record, Resource::ReflectionNoise(0));
        begin_pair(
            &device,
            record.command_buffer,
            target_of(&quiet.wave[0]),
            target_of(&quiet.noise[0]),
        );
        unsafe {
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                quiet.variance_pipeline,
            );
            device.cmd_bind_descriptor_sets(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                quiet.variance_layout,
                0,
                &[quiet.variance_sets[turn]],
                &[],
            );
            device.cmd_push_constants(
                record.command_buffer,
                quiet.variance_layout,
                vk::ShaderStageFlags::FRAGMENT,
                0,
                bytemuck::bytes_of(&wave),
            );
            device.cmd_draw(record.command_buffer, 3, 1, 0, 0);
            device.cmd_end_rendering(record.command_buffer);
        }
        read_target(record, Resource::ReflectionWave(0));
        read_target(record, Resource::ReflectionNoise(0));

        for round in 0..QUIET_ROUNDS as usize {
            let from = round % 2;
            let into = 1 - from;
            let mut params = wave;
            params.screen[2] = (1u32 << round) as f32;
            params.screen[3] = QUIET_BRIGHTNESS;
            read_target(record, Resource::ReflectionWave(from));
            read_target(record, Resource::ReflectionNoise(from));
            claim_target(record, Resource::ReflectionWave(into));
            claim_target(record, Resource::ReflectionNoise(into));
            begin_pair(
                &device,
                record.command_buffer,
                target_of(&quiet.wave[into]),
                target_of(&quiet.noise[into]),
            );
            unsafe {
                device.cmd_bind_pipeline(
                    record.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    quiet.wavelet_pipeline,
                );
                device.cmd_bind_descriptor_sets(
                    record.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    quiet.wavelet_layout,
                    0,
                    &[quiet.wavelet_sets[from]],
                    &[],
                );
                device.cmd_push_constants(
                    record.command_buffer,
                    quiet.wavelet_layout,
                    vk::ShaderStageFlags::FRAGMENT,
                    0,
                    bytemuck::bytes_of(&params),
                );
                device.cmd_draw(record.command_buffer, 3, 1, 0, 0);
                device.cmd_end_rendering(record.command_buffer);
            }
            read_target(record, Resource::ReflectionWave(into));
            read_target(record, Resource::ReflectionNoise(into));
        }

        let Some(quiet) = &mut self.quiet else {
            return;
        };
        quiet.turn = before;
        quiet.standing = true;
    }

    fn soften(
        &self,
        record: &mut PassRecord,
        resource: Resource,
        target: Option<Target>,
        source: vk::Extent2D,
        set: vk::DescriptorSet,
        shape: (f32, f32),
        keep_alpha: bool,
    ) {
        let Some(target) = target else {
            return;
        };
        let device = record.context.device.clone();
        claim_target(record, resource);
        begin_target(&device, record.command_buffer, target);
        unsafe {
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.screen_blur_pipeline,
            );
            device.cmd_bind_descriptor_sets(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.screen_blur_layout,
                0,
                &[set],
                &[],
            );
            let params = ScreenBlurParams {
                screen: [
                    source.width as f32,
                    source.height as f32,
                    SCREEN_BLUR_THRESHOLD,
                    SCREEN_BLUR_NORMAL_POWER,
                ],
                shape: [shape.0, shape.1, if keep_alpha { 1.0 } else { 0.0 }, 0.0],
            };
            device.cmd_push_constants(
                record.command_buffer,
                self.screen_blur_layout,
                vk::ShaderStageFlags::FRAGMENT,
                0,
                bytemuck::bytes_of(&params),
            );
            device.cmd_draw(record.command_buffer, 3, 1, 0, 0);
        }
        end_target(record, resource);
    }

    fn gather_indirect(&mut self, record: &mut PassRecord) {
        let Some(target) = &self.indirect.image else {
            return;
        };
        if self.params.indirect_enabled == 0 && self.indirect.written {
            return;
        }
        let device = record.context.device.clone();
        let target = target_of(target);
        claim_target(record, Resource::Indirect);
        begin_target(&device, record.command_buffer, target);
        unsafe {
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.indirect.pipeline,
            );
            device.cmd_bind_descriptor_sets(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.indirect.layout,
                0,
                &[self.indirect.set],
                &[],
            );
            device.cmd_push_constants(
                record.command_buffer,
                self.indirect.layout,
                vk::ShaderStageFlags::FRAGMENT,
                0,
                bytemuck::bytes_of(&self.indirect.params),
            );
            device.cmd_draw(record.command_buffer, 3, 1, 0, 0);
        }
        end_target(record, Resource::Indirect);
        self.soften(
            record,
            Resource::IndirectSoft,
            self.indirect.soft.as_ref().map(target_of),
            target.extent,
            self.indirect.blur_set,
            INDIRECT_BLUR,
            false,
        );
        self.indirect.written = true;
    }

    fn spread_out_of_focus(&mut self, record: &mut PassRecord) {
        let Some(target) = &self.focus.image else {
            return;
        };
        if self.params.focus_enabled == 0 && self.focus.written {
            return;
        }
        let device = record.context.device.clone();
        let target = target_of(target);
        claim_target(record, Resource::Focus);
        begin_target(&device, record.command_buffer, target);
        unsafe {
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.focus.pipeline,
            );
            device.cmd_bind_descriptor_sets(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.focus.layout,
                0,
                &[self.focus.set],
                &[],
            );
            device.cmd_push_constants(
                record.command_buffer,
                self.focus.layout,
                vk::ShaderStageFlags::FRAGMENT,
                0,
                bytemuck::bytes_of(&self.focus.params),
            );
            device.cmd_draw(record.command_buffer, 3, 1, 0, 0);
        }
        end_target(record, Resource::Focus);
        self.focus.written = true;
    }

    fn resolve_over_time(&mut self, record: &mut PassRecord) {
        let wanted = self.params.resolved_enabled != 0;
        let (Some(resolved), 2) = (&self.resolve.image, self.resolve.kept.len()) else {
            return;
        };
        if !wanted && self.resolve.written {
            return;
        }
        let device = record.context.device.clone();
        let round = self.resolve.round;
        let resolved = target_of(resolved);
        let writing = target_of(&self.resolve.kept[1 - round]);

        let mut params = self.resolve.params;
        if !self.resolve.ready || !wanted {
            params.settings[0] = 0.0;
        }
        record.require(Resource::Kept(round), Usage::Sampled);

        claim_target(record, Resource::Resolved);
        claim_target(record, Resource::Kept(1 - round));
        begin_pair(&device, record.command_buffer, resolved, writing);
        unsafe {
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.resolve.pipeline,
            );
            device.cmd_bind_descriptor_sets(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.resolve.layout,
                0,
                &[self.resolve.sets[round]],
                &[],
            );
            device.cmd_push_constants(
                record.command_buffer,
                self.resolve.layout,
                vk::ShaderStageFlags::FRAGMENT,
                0,
                bytemuck::bytes_of(&params),
            );
            device.cmd_draw(record.command_buffer, 3, 1, 0, 0);
            device.cmd_end_rendering(record.command_buffer);
        }
        read_target(record, Resource::Resolved);
        read_target(record, Resource::Kept(1 - round));

        self.resolve.round = 1 - round;
        self.resolve.ready = wanted;
        self.resolve.written = true;
    }
}

fn bind_traced(
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    traced: &Traced,
    layout: vk::PipelineLayout,
    frame_index: usize,
    own: vk::DescriptorSet,
) {
    let inputs = &traced.inputs;
    let slot = |sets: &[vk::DescriptorSet]| sets[frame_index % sets.len().max(1)];
    let sets = [
        slot(&inputs.scene_sets),
        inputs.texture_set,
        slot(&inputs.shadow_sets),
        slot(&inputs.environment_sets),
        own,
    ];
    unsafe {
        device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            layout,
            0,
            &sets,
            &[],
        );
    }
}

fn begin_pair(
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    first: Target,
    second: Target,
) {
    let attachments = [
        vk::RenderingAttachmentInfo::default()
            .image_view(first.view)
            .image_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE),
        vk::RenderingAttachmentInfo::default()
            .image_view(second.view)
            .image_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE),
    ];
    let area = vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: first.extent,
    };
    let rendering = vk::RenderingInfo::default()
        .render_area(area)
        .layer_count(1)
        .color_attachments(&attachments);
    let viewport = vk::Viewport {
        x: 0.0,
        y: first.extent.height as f32,
        width: first.extent.width as f32,
        height: -(first.extent.height as f32),
        min_depth: 0.0,
        max_depth: 1.0,
    };
    unsafe {
        device.cmd_begin_rendering(command_buffer, &rendering);
        device.cmd_set_viewport(command_buffer, 0, std::slice::from_ref(&viewport));
        device.cmd_set_scissor(command_buffer, 0, std::slice::from_ref(&area));
    }
}

fn register_target(states: &mut ResourceStates, resource: Resource, target: &Image) {
    resource::register(states, resource, target.image, vk::ImageAspectFlags::COLOR);
}

fn claim_target(record: &mut PassRecord, resource: Resource) {
    record.discard(resource);
    record.require(resource, Usage::ColorAttachment);
}

fn begin_target(device: &ash::Device, command_buffer: vk::CommandBuffer, target: Target) {
    begin_rendering(device, command_buffer, target, vk::AttachmentLoadOp::CLEAR);
}

fn begin_target_loading(device: &ash::Device, command_buffer: vk::CommandBuffer, target: Target) {
    begin_rendering(device, command_buffer, target, vk::AttachmentLoadOp::LOAD);
}

#[derive(Clone, Copy)]
struct Target {
    view: vk::ImageView,
    extent: vk::Extent2D,
}

fn target_of(image: &Image) -> Target {
    Target {
        view: image.view,
        extent: image.extent,
    }
}

fn begin_rendering(
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    target: Target,
    load: vk::AttachmentLoadOp,
) {
    let attachment = vk::RenderingAttachmentInfo::default()
        .image_view(target.view)
        .image_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL)
        .load_op(load)
        .store_op(vk::AttachmentStoreOp::STORE)
        .clear_value(vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        });
    let area = vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: target.extent,
    };
    let rendering = vk::RenderingInfo::default()
        .render_area(area)
        .layer_count(1)
        .color_attachments(std::slice::from_ref(&attachment));
    let viewport = vk::Viewport {
        x: 0.0,
        y: target.extent.height as f32,
        width: target.extent.width as f32,
        height: -(target.extent.height as f32),
        min_depth: 0.0,
        max_depth: 1.0,
    };
    unsafe {
        device.cmd_begin_rendering(command_buffer, &rendering);
        device.cmd_set_viewport(command_buffer, 0, std::slice::from_ref(&viewport));
        device.cmd_set_scissor(command_buffer, 0, std::slice::from_ref(&area));
    }
}

fn end_target(record: &mut PassRecord, resource: Resource) {
    unsafe {
        record
            .context
            .device
            .cmd_end_rendering(record.command_buffer)
    };
    record.require(resource, Usage::Sampled);
}

fn read_target(record: &mut PassRecord, resource: Resource) {
    record.require(resource, Usage::Sampled);
}

fn read_to_attachment(record: &mut PassRecord, resource: Resource) {
    record.require(resource, Usage::ColorAttachment);
}

fn allocate_set(
    context: &Context,
    pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
) -> Result<vk::DescriptorSet, Box<dyn std::error::Error>> {
    let layouts = [layout];
    let allocate_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    Ok(unsafe { context.device.allocate_descriptor_sets(&allocate_info)?[0] })
}

fn write_set(
    context: &Context,
    set: vk::DescriptorSet,
    views: &[vk::ImageView],
    sampler: vk::Sampler,
) {
    let infos: Vec<vk::DescriptorImageInfo> = views
        .iter()
        .map(|view| {
            vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(*view)
                .sampler(sampler)
        })
        .collect();
    let writes: Vec<vk::WriteDescriptorSet> = infos
        .iter()
        .enumerate()
        .map(|(binding, info)| {
            vk::WriteDescriptorSet::default()
                .dst_set(set)
                .dst_binding(binding as u32)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(info))
        })
        .collect();
    unsafe { context.device.update_descriptor_sets(&writes, &[]) };
}

fn create_compose_layout(
    context: &Context,
) -> Result<vk::DescriptorSetLayout, Box<dyn std::error::Error>> {
    let mut bindings: Vec<vk::DescriptorSetLayoutBinding> = (0..7)
        .map(|binding| {
            vk::DescriptorSetLayoutBinding::default()
                .binding(binding)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        })
        .collect();
    bindings.push(
        vk::DescriptorSetLayoutBinding::default()
            .binding(7)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
    );
    bindings.push(
        vk::DescriptorSetLayoutBinding::default()
            .binding(8)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
    );
    let create_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
    Ok(unsafe {
        context
            .device
            .create_descriptor_set_layout(&create_info, None)?
    })
}

const EXPOSURE_STEPS: usize = 256;
const EXPOSURE_GATHER_SPAN: u32 = 32;

fn exposure_barrier(
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    buffer: vk::Buffer,
    src_stage: vk::PipelineStageFlags2,
    src_access: vk::AccessFlags2,
    dst_stage: vk::PipelineStageFlags2,
    dst_access: vk::AccessFlags2,
) {
    let barrier = vk::BufferMemoryBarrier2::default()
        .buffer(buffer)
        .offset(0)
        .size(vk::WHOLE_SIZE)
        .src_stage_mask(src_stage)
        .src_access_mask(src_access)
        .dst_stage_mask(dst_stage)
        .dst_access_mask(dst_access)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED);
    let dependency =
        vk::DependencyInfo::default().buffer_memory_barriers(std::slice::from_ref(&barrier));
    unsafe { device.cmd_pipeline_barrier2(command_buffer, &dependency) };
}

fn create_exposure_layout(
    context: &Context,
) -> Result<vk::DescriptorSetLayout, Box<dyn std::error::Error>> {
    let bindings = [
        vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::COMPUTE),
        vk::DescriptorSetLayoutBinding::default()
            .binding(1)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::COMPUTE),
    ];
    let create_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
    Ok(unsafe {
        context
            .device
            .create_descriptor_set_layout(&create_info, None)?
    })
}

fn create_compute_layout(
    context: &Context,
    descriptor_layout: vk::DescriptorSetLayout,
) -> Result<vk::PipelineLayout, Box<dyn std::error::Error>> {
    let layouts = [descriptor_layout];
    let range = vk::PushConstantRange::default()
        .stage_flags(vk::ShaderStageFlags::COMPUTE)
        .offset(0)
        .size(std::mem::size_of::<[f32; 4]>() as u32);
    let create_info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(&layouts)
        .push_constant_ranges(std::slice::from_ref(&range));
    Ok(unsafe { context.device.create_pipeline_layout(&create_info, None)? })
}

fn create_exposure_pipeline(
    context: &Context,
    layout: vk::PipelineLayout,
    spirv: &[u8],
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    let module = create_shader_module(context, spirv)?;
    let stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::COMPUTE)
        .module(module)
        .name(c"main");
    let create_info = vk::ComputePipelineCreateInfo::default()
        .stage(stage)
        .layout(layout);
    let created = unsafe {
        context.device.create_compute_pipelines(
            context.pipeline_cache,
            std::slice::from_ref(&create_info),
            None,
        )
    };
    unsafe { context.device.destroy_shader_module(module, None) };
    match created {
        Ok(pipelines) => Ok(pipelines[0]),
        Err((_, error)) => Err(error.into()),
    }
}

fn create_descriptor_layout(
    context: &Context,
    count: u32,
) -> Result<vk::DescriptorSetLayout, Box<dyn std::error::Error>> {
    let bindings: Vec<vk::DescriptorSetLayoutBinding> = (0..count)
        .map(|binding| {
            vk::DescriptorSetLayoutBinding::default()
                .binding(binding)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        })
        .collect();
    let create_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
    Ok(unsafe {
        context
            .device
            .create_descriptor_set_layout(&create_info, None)?
    })
}

fn create_descriptor_pool(
    context: &Context,
) -> Result<vk::DescriptorPool, Box<dyn std::error::Error>> {
    let count = (BLOOM_LEVELS * 2 + 20) as u32;
    let sizes = [
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(count * 8),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(count),
    ];
    let create_info = vk::DescriptorPoolCreateInfo::default()
        .pool_sizes(&sizes)
        .max_sets(count);
    Ok(unsafe { context.device.create_descriptor_pool(&create_info, None)? })
}

fn create_sampler(
    context: &Context,
    filter: vk::Filter,
) -> Result<vk::Sampler, Box<dyn std::error::Error>> {
    let create_info = vk::SamplerCreateInfo::default()
        .mag_filter(filter)
        .min_filter(filter)
        .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE);
    Ok(unsafe { context.device.create_sampler(&create_info, None)? })
}

fn create_pipeline_layout(
    context: &Context,
    descriptor_layout: vk::DescriptorSetLayout,
    push_size: u32,
) -> Result<vk::PipelineLayout, Box<dyn std::error::Error>> {
    let layouts = [descriptor_layout];
    let range = vk::PushConstantRange::default()
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        .offset(0)
        .size(push_size);
    let create_info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(&layouts)
        .push_constant_ranges(std::slice::from_ref(&range));
    Ok(unsafe { context.device.create_pipeline_layout(&create_info, None)? })
}

fn create_pair_pipeline(
    context: &Context,
    layout: vk::PipelineLayout,
    fragment_code: &[u8],
    format: vk::Format,
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    let vertex = create_shader_module(context, include_bytes!("shaders/fullscreen.vert.spv"))?;
    let fragment = create_shader_module(context, fragment_code)?;
    let entry = c"main";
    let stages = [
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertex)
            .name(entry),
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragment)
            .name(entry),
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
    let blend_attachment = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::RGBA);
    let attachments = [blend_attachment, blend_attachment];
    let color_blend = vk::PipelineColorBlendStateCreateInfo::default().attachments(&attachments);
    let formats = [format, format];
    let mut rendering =
        vk::PipelineRenderingCreateInfo::default().color_attachment_formats(&formats);

    let create_info = vk::GraphicsPipelineCreateInfo::default()
        .stages(&stages)
        .vertex_input_state(&vertex_input)
        .input_assembly_state(&assembly)
        .viewport_state(&viewport_state)
        .dynamic_state(&dynamic_state)
        .rasterization_state(&rasterization)
        .multisample_state(&multisample)
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

fn create_pipeline(
    context: &Context,
    layout: vk::PipelineLayout,
    fragment_code: &[u8],
    format: vk::Format,
    depth_format: vk::Format,
    additive: bool,
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    let vertex = create_shader_module(context, include_bytes!("shaders/fullscreen.vert.spv"))?;
    let fragment = create_shader_module(context, fragment_code)?;
    let entry = std::ffi::CString::new("main")?;
    let stages = [
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertex)
            .name(&entry),
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragment)
            .name(&entry),
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
    let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
        .depth_test_enable(false)
        .depth_write_enable(false);
    let mut blend_attachment = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::RGBA);
    if additive {
        blend_attachment = blend_attachment
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ONE)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE)
            .alpha_blend_op(vk::BlendOp::ADD);
    }
    let color_blend = vk::PipelineColorBlendStateCreateInfo::default()
        .attachments(std::slice::from_ref(&blend_attachment));

    let color_formats = [format];
    let mut rendering = vk::PipelineRenderingCreateInfo::default()
        .color_attachment_formats(&color_formats)
        .depth_attachment_format(depth_format);

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

    let pipelines = unsafe {
        context
            .device
            .create_graphics_pipelines(context.pipeline_cache, &[create_info], None)
    };
    unsafe {
        context.device.destroy_shader_module(vertex, None);
        context.device.destroy_shader_module(fragment, None);
    }
    Ok(pipelines.map_err(|(_, error)| error)?[0])
}
