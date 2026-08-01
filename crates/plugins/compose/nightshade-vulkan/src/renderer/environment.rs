use ash::vk;
use gpu_allocator::MemoryLocation;

use crate::renderer::context::{Context, FRAMES_IN_FLIGHT};
use crate::renderer::cube::{CUBE_FORMAT, CubeImage, create_cube_image, destroy_cube_image};
use crate::renderer::lookup::{self, AreaTables};
use crate::renderer::memory::{
    Image, begin_one_shot, create_buffer, destroy_buffer, destroy_image, end_one_shot, sent_count,
    wait_for_count, wait_for_sent_work, write_buffer,
};
use crate::renderer::pass::create_shader_module;
use crate::renderer::{Transition, transition};

const SKY_SIZE: u32 = 1024;
const IRRADIANCE_SIZE: u32 = 64;
const PREFILTERED_SIZE: u32 = 512;
const PREFILTERED_LEVELS: u32 = 5;

const IRRADIANCE_SAMPLES: u32 = 1024;
const PREFILTERED_SAMPLES: u32 = 512;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct ProjectParams {
    size: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct PaintParams {
    size: u32,
    atmosphere: u32,
    seconds: f32,
    hour: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct FilterParams {
    size: u32,
    samples: u32,
    roughness: f32,
    source_size: f32,
    diffuse: u32,
}

const SURFACE_TABLE_SIZE: u32 = 256;
const SURFACE_TABLE_SAMPLES: u32 = 1024;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct SurfaceTableParams {
    size: u32,
    samples: u32,
}

pub struct Environment {
    pub sky: CubeImage,
    pub area_tables: AreaTables,
    surface_table: Image,
    pub irradiance: CubeImage,
    pub prefiltered: CubeImage,
    pub irradiance_other: CubeImage,
    pub prefiltered_other: CubeImage,
    held: [Option<f32>; 2],
    pub crossed: f32,
    pub sampler: vk::Sampler,
    pub loaded: bool,
    pub scene_layout: vk::DescriptorSetLayout,
    pub scene_sets: [vk::DescriptorSet; FRAMES_IN_FLIGHT],
    scene_stale: [bool; FRAMES_IN_FLIGHT],
    scene_pool: vk::DescriptorPool,
    bake_done_at: u64,
    project_pipeline: vk::Pipeline,
    project_layout: vk::PipelineLayout,
    paint_pipeline: vk::Pipeline,
    paint_layout: vk::PipelineLayout,
    paint_descriptor_layout: vk::DescriptorSetLayout,
    painted: Option<(u32, i32)>,
    filter_pipeline: vk::Pipeline,
    filter_layout: vk::PipelineLayout,
    surface_table_pipeline: vk::Pipeline,
    surface_table_layout: vk::PipelineLayout,
    surface_table_descriptor_layout: vk::DescriptorSetLayout,
    project_descriptor_layout: vk::DescriptorSetLayout,
    filter_descriptor_layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
}

pub fn create_environment(context: &Context) -> Result<Environment, Box<dyn std::error::Error>> {
    let area_tables = lookup::create(context)?;
    let sky = create_cube_image(
        context,
        SKY_SIZE,
        crate::renderer::cube::level_count(SKY_SIZE),
        "sky cube",
    )?;
    let irradiance = create_cube_image(context, IRRADIANCE_SIZE, 1, "irradiance cube")?;
    let prefiltered = create_cube_image(
        context,
        PREFILTERED_SIZE,
        PREFILTERED_LEVELS,
        "prefiltered cube",
    )?;
    let irradiance_other =
        create_cube_image(context, IRRADIANCE_SIZE, 1, "irradiance cube, the other")?;
    let prefiltered_other = create_cube_image(
        context,
        PREFILTERED_SIZE,
        PREFILTERED_LEVELS,
        "prefiltered cube, the other",
    )?;
    let sampler = create_sampler(context)?;

    let scene_layout = create_scene_layout(context)?;
    let project_descriptor_layout = create_descriptor_layout(context)?;
    let filter_descriptor_layout = create_descriptor_layout(context)?;
    let pool = create_descriptor_pool(context)?;
    let scene_pool = create_scene_pool(context)?;

    let project_layout = create_pipeline_layout(
        context,
        project_descriptor_layout,
        std::mem::size_of::<ProjectParams>() as u32,
    )?;
    let filter_layout = create_pipeline_layout(
        context,
        filter_descriptor_layout,
        std::mem::size_of::<FilterParams>() as u32,
    )?;
    let paint_descriptor_layout = create_paint_layout(context)?;
    let paint_layout = create_pipeline_layout(
        context,
        paint_descriptor_layout,
        std::mem::size_of::<PaintParams>() as u32,
    )?;
    let project_pipeline = create_compute_pipeline(
        context,
        project_layout,
        include_bytes!("shaders/equirect_to_cube.comp.spv"),
    )?;
    let paint_pipeline = create_compute_pipeline(
        context,
        paint_layout,
        include_bytes!("shaders/sky_to_cube.comp.spv"),
    )?;
    let filter_pipeline = create_compute_pipeline(
        context,
        filter_layout,
        include_bytes!("shaders/environment_filter.comp.spv"),
    )?;
    let surface_table_descriptor_layout = create_paint_layout_at(context, 0)?;
    let surface_table_layout = create_pipeline_layout(
        context,
        surface_table_descriptor_layout,
        std::mem::size_of::<SurfaceTableParams>() as u32,
    )?;
    let surface_table_pipeline = create_compute_pipeline(
        context,
        surface_table_layout,
        include_bytes!("shaders/surface_table.comp.spv"),
    )?;
    let surface_table = create_surface_table_image(context)?;

    let mut environment = Environment {
        sky,
        area_tables,
        surface_table,
        irradiance,
        prefiltered,
        irradiance_other,
        prefiltered_other,
        held: [None, None],
        crossed: 0.0,
        sampler,
        loaded: false,
        scene_layout,
        scene_sets: [vk::DescriptorSet::null(); FRAMES_IN_FLIGHT],
        scene_stale: [false; FRAMES_IN_FLIGHT],
        scene_pool,
        bake_done_at: 0,
        project_pipeline,
        project_layout,
        paint_pipeline,
        paint_layout,
        paint_descriptor_layout,
        painted: None,
        filter_pipeline,
        filter_layout,
        surface_table_pipeline,
        surface_table_layout,
        surface_table_descriptor_layout,
        project_descriptor_layout,
        filter_descriptor_layout,
        pool,
    };
    clear_environment(context, &mut environment)?;
    gather_surface_table(context, &environment)?;
    for index in 0..FRAMES_IN_FLIGHT {
        environment.scene_sets[index] =
            allocate_set(context, environment.scene_pool, environment.scene_layout)?;
    }
    write_scene_sets(context, &environment);
    Ok(environment)
}

pub fn destroy_environment(context: &Context, environment: &mut Environment) {
    unsafe {
        context
            .device
            .destroy_descriptor_set_layout(environment.scene_layout, None)
    };
    lookup::destroy(&mut environment.area_tables, context);
    destroy_image(context, &mut environment.surface_table);
    unsafe {
        context
            .device
            .destroy_pipeline(environment.surface_table_pipeline, None);
        context
            .device
            .destroy_pipeline_layout(environment.surface_table_layout, None);
        context
            .device
            .destroy_descriptor_set_layout(environment.surface_table_descriptor_layout, None);
    }
    destroy_cube_image(context, &mut environment.sky);
    destroy_cube_image(context, &mut environment.irradiance);
    destroy_cube_image(context, &mut environment.prefiltered);
    destroy_cube_image(context, &mut environment.irradiance_other);
    destroy_cube_image(context, &mut environment.prefiltered_other);
    unsafe {
        context.device.destroy_sampler(environment.sampler, None);
        context
            .device
            .destroy_pipeline(environment.project_pipeline, None);
        context
            .device
            .destroy_pipeline(environment.paint_pipeline, None);
        context
            .device
            .destroy_pipeline_layout(environment.paint_layout, None);
        context
            .device
            .destroy_descriptor_set_layout(environment.paint_descriptor_layout, None);
        context
            .device
            .destroy_pipeline(environment.filter_pipeline, None);
        context
            .device
            .destroy_pipeline_layout(environment.project_layout, None);
        context
            .device
            .destroy_pipeline_layout(environment.filter_layout, None);
        context
            .device
            .destroy_descriptor_pool(environment.pool, None);
        context
            .device
            .destroy_descriptor_pool(environment.scene_pool, None);
        context
            .device
            .destroy_descriptor_set_layout(environment.project_descriptor_layout, None);
        context
            .device
            .destroy_descriptor_set_layout(environment.filter_descriptor_layout, None);
    }
}

fn bracket(hours: &[f32], hour: f32) -> Option<(f32, f32, f32)> {
    if hours.len() < 2 {
        return None;
    }
    let hour = hour - (hour / 24.0).floor() * 24.0;
    let mut lower = hours[hours.len() - 1];
    let mut upper = hours[0];
    for held in hours {
        if *held <= hour {
            lower = *held;
        }
    }
    for held in hours {
        if *held > hour {
            upper = *held;
            break;
        }
    }
    let span = if upper > lower {
        upper - lower
    } else {
        (24.0 - lower) + upper
    };
    let along = if hour >= lower {
        hour - lower
    } else {
        (24.0 - lower) + hour
    };
    let crossed = if span > 0.0 { along / span } else { 0.0 };
    Some((lower, upper, crossed.clamp(0.0, 1.0)))
}

pub fn cross_the_hours(
    context: &Context,
    environment: &mut Environment,
    atmosphere: u32,
    seconds: f32,
    hour: f32,
    hours: &[f32],
) -> Result<(), Box<dyn std::error::Error>> {
    let Some((lower, upper, crossed)) = bracket(hours, hour) else {
        environment.crossed = 0.0;
        return paint_at(context, environment, atmosphere, seconds, hour, 0).map(|_| ());
    };
    if environment.painted.map(|(painted, _)| painted) != Some(atmosphere) {
        environment.held = [None, None];
    }

    let slot_of = |held: &[Option<f32>; 2], wanted: f32| {
        held.iter()
            .position(|holding| holding.map(|at| at == wanted).unwrap_or(false))
    };
    let mut lower_slot = slot_of(&environment.held, lower);
    let upper_slot = slot_of(&environment.held, upper);
    if lower_slot.is_none() {
        let free = upper_slot.map(|taken| 1 - taken).unwrap_or(0);
        paint_at(context, environment, atmosphere, seconds, lower, free)?;
        environment.held[free] = Some(lower);
        lower_slot = Some(free);
    }
    if upper_slot.is_none() {
        let free = 1 - lower_slot.unwrap_or(0);
        paint_at(context, environment, atmosphere, seconds, upper, free)?;
        environment.held[free] = Some(upper);
    }

    environment.crossed = if lower_slot == Some(0) {
        crossed
    } else {
        1.0 - crossed
    };
    Ok(())
}

fn paint_at(
    context: &Context,
    environment: &mut Environment,
    atmosphere: u32,
    seconds: f32,
    hour: f32,
    slot: usize,
) -> Result<bool, Box<dyn std::error::Error>> {
    let asked = (atmosphere, hour.round() as i32);
    if slot == 0 && environment.held == [None, None] && environment.painted == Some(asked) {
        return Ok(false);
    }

    wait_for_count(context, &context.graphics, environment.bake_done_at)?;
    unsafe {
        context
            .device
            .reset_descriptor_pool(environment.pool, vk::DescriptorPoolResetFlags::empty())?
    };
    let set = allocate_set(
        context,
        environment.pool,
        environment.paint_descriptor_layout,
    )?;
    write_paint_set(context, set, environment.sky.level_views[0]);

    let command_buffer = begin_one_shot(context)?;
    transition_cube(
        context,
        command_buffer,
        &environment.sky,
        LayoutChange {
            old_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            new_layout: vk::ImageLayout::GENERAL,
            source_access: vk::AccessFlags2::SHADER_SAMPLED_READ,
            destination_access: vk::AccessFlags2::SHADER_STORAGE_WRITE,
            source_stage: vk::PipelineStageFlags2::FRAGMENT_SHADER,
            destination_stage: vk::PipelineStageFlags2::COMPUTE_SHADER,
        },
    );
    unsafe {
        context.device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            environment.paint_pipeline,
        );
        context.device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            environment.paint_layout,
            0,
            &[set],
            &[],
        );
        let params = PaintParams {
            size: SKY_SIZE,
            atmosphere,
            seconds,
            hour,
        };
        context.device.cmd_push_constants(
            command_buffer,
            environment.paint_layout,
            vk::ShaderStageFlags::COMPUTE,
            0,
            bytemuck::bytes_of(&params),
        );
        context.device.cmd_dispatch(
            command_buffer,
            SKY_SIZE.div_ceil(16),
            SKY_SIZE.div_ceil(16),
            6,
        );
    }
    end_one_shot(context, command_buffer)?;

    build_levels(context, &environment.sky)?;
    filter_irradiance(context, environment, slot)?;
    filter_prefiltered(context, environment, slot)?;

    environment.scene_stale = [true; FRAMES_IN_FLIGHT];
    environment.bake_done_at = sent_count(&context.graphics);
    environment.painted = Some(asked);
    environment.loaded = true;
    Ok(true)
}

pub fn settle_scene_set(context: &Context, environment: &mut Environment, frame_index: usize) {
    if frame_index >= FRAMES_IN_FLIGHT || !environment.scene_stale[frame_index] {
        return;
    }
    write_one_scene_set(context, environment, frame_index);
    environment.scene_stale[frame_index] = false;
}

pub fn load_environment(
    context: &Context,
    environment: &mut Environment,
    bytes: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let (pixels, width, height) = decode_radiance(bytes)?;
    let mut source = upload_equirectangular(context, &pixels, width, height)?;

    unsafe {
        context
            .device
            .reset_descriptor_pool(environment.pool, vk::DescriptorPoolResetFlags::empty())?
    };

    environment.painted = None;
    project_to_cube(context, environment, &source)?;
    build_levels(context, &environment.sky)?;
    filter_irradiance(context, environment, 0)?;
    filter_prefiltered(context, environment, 0)?;

    wait_for_sent_work(context)?;
    destroy_image(context, &mut source);
    write_scene_sets(context, environment);
    environment.bake_done_at = sent_count(&context.graphics);
    environment.loaded = true;
    Ok(())
}

fn clear_environment(
    context: &Context,
    environment: &mut Environment,
) -> Result<(), Box<dyn std::error::Error>> {
    let command_buffer = begin_one_shot(context)?;
    for cube in [
        &environment.sky,
        &environment.irradiance,
        &environment.prefiltered,
        &environment.irradiance_other,
        &environment.prefiltered_other,
    ] {
        transition_cube(
            context,
            command_buffer,
            cube,
            LayoutChange {
                old_layout: vk::ImageLayout::UNDEFINED,
                new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                source_access: vk::AccessFlags2::empty(),
                destination_access: vk::AccessFlags2::TRANSFER_WRITE,
                source_stage: vk::PipelineStageFlags2::TOP_OF_PIPE,
                destination_stage: vk::PipelineStageFlags2::TRANSFER,
            },
        );
        let range = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: cube.levels,
            base_array_layer: 0,
            layer_count: 6,
        };
        unsafe {
            context.device.cmd_clear_color_image(
                command_buffer,
                cube.image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
                std::slice::from_ref(&range),
            )
        };
        transition_cube(
            context,
            command_buffer,
            cube,
            LayoutChange {
                old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                source_access: vk::AccessFlags2::TRANSFER_WRITE,
                destination_access: vk::AccessFlags2::SHADER_SAMPLED_READ,
                source_stage: vk::PipelineStageFlags2::TRANSFER,
                destination_stage: vk::PipelineStageFlags2::FRAGMENT_SHADER,
            },
        );
    }
    end_one_shot(context, command_buffer)?;
    Ok(())
}

fn project_to_cube(
    context: &Context,
    environment: &Environment,
    source: &Image,
) -> Result<(), Box<dyn std::error::Error>> {
    let set = allocate_set(
        context,
        environment.pool,
        environment.project_descriptor_layout,
    )?;
    write_set(
        context,
        set,
        source.view,
        environment.sampler,
        environment.sky.level_views[0],
    );

    let command_buffer = begin_one_shot(context)?;
    transition_cube(
        context,
        command_buffer,
        &environment.sky,
        LayoutChange {
            old_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            new_layout: vk::ImageLayout::GENERAL,
            source_access: vk::AccessFlags2::SHADER_SAMPLED_READ,
            destination_access: vk::AccessFlags2::SHADER_STORAGE_WRITE,
            source_stage: vk::PipelineStageFlags2::FRAGMENT_SHADER,
            destination_stage: vk::PipelineStageFlags2::COMPUTE_SHADER,
        },
    );
    unsafe {
        context.device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            environment.project_pipeline,
        );
        context.device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            environment.project_layout,
            0,
            &[set],
            &[],
        );
        let params = ProjectParams { size: SKY_SIZE };
        context.device.cmd_push_constants(
            command_buffer,
            environment.project_layout,
            vk::ShaderStageFlags::COMPUTE,
            0,
            bytemuck::bytes_of(&params),
        );
        context.device.cmd_dispatch(
            command_buffer,
            SKY_SIZE.div_ceil(16),
            SKY_SIZE.div_ceil(16),
            6,
        );
    }
    end_one_shot(context, command_buffer)?;
    Ok(())
}

fn build_levels(context: &Context, cube: &CubeImage) -> Result<(), Box<dyn std::error::Error>> {
    let command_buffer = begin_one_shot(context)?;
    transition_level(
        context,
        command_buffer,
        cube,
        0,
        LayoutChange {
            old_layout: vk::ImageLayout::GENERAL,
            new_layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            source_access: vk::AccessFlags2::SHADER_STORAGE_WRITE,
            destination_access: vk::AccessFlags2::TRANSFER_READ,
            source_stage: vk::PipelineStageFlags2::COMPUTE_SHADER,
            destination_stage: vk::PipelineStageFlags2::TRANSFER,
        },
    );
    for level in 1..cube.levels {
        transition_level(
            context,
            command_buffer,
            cube,
            level,
            LayoutChange {
                old_layout: vk::ImageLayout::GENERAL,
                new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                source_access: vk::AccessFlags2::empty(),
                destination_access: vk::AccessFlags2::TRANSFER_WRITE,
                source_stage: vk::PipelineStageFlags2::TOP_OF_PIPE,
                destination_stage: vk::PipelineStageFlags2::TRANSFER,
            },
        );
        let source_size = (cube.size >> (level - 1)).max(1) as i32;
        let target_size = (cube.size >> level).max(1) as i32;
        let blit = vk::ImageBlit::default()
            .src_subresource(vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: level - 1,
                base_array_layer: 0,
                layer_count: 6,
            })
            .src_offsets([
                vk::Offset3D { x: 0, y: 0, z: 0 },
                vk::Offset3D {
                    x: source_size,
                    y: source_size,
                    z: 1,
                },
            ])
            .dst_subresource(vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: level,
                base_array_layer: 0,
                layer_count: 6,
            })
            .dst_offsets([
                vk::Offset3D { x: 0, y: 0, z: 0 },
                vk::Offset3D {
                    x: target_size,
                    y: target_size,
                    z: 1,
                },
            ]);
        unsafe {
            context.device.cmd_blit_image(
                command_buffer,
                cube.image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                cube.image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                std::slice::from_ref(&blit),
                vk::Filter::LINEAR,
            )
        };
        transition_level(
            context,
            command_buffer,
            cube,
            level,
            LayoutChange {
                old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                new_layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                source_access: vk::AccessFlags2::TRANSFER_WRITE,
                destination_access: vk::AccessFlags2::TRANSFER_READ,
                source_stage: vk::PipelineStageFlags2::TRANSFER,
                destination_stage: vk::PipelineStageFlags2::TRANSFER,
            },
        );
    }
    transition_cube(
        context,
        command_buffer,
        cube,
        LayoutChange {
            old_layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            source_access: vk::AccessFlags2::TRANSFER_READ,
            destination_access: vk::AccessFlags2::SHADER_SAMPLED_READ,
            source_stage: vk::PipelineStageFlags2::TRANSFER,
            destination_stage: vk::PipelineStageFlags2::COMPUTE_SHADER,
        },
    );
    end_one_shot(context, command_buffer)?;
    Ok(())
}

fn irradiance_of(environment: &Environment, slot: usize) -> &CubeImage {
    if slot == 0 {
        &environment.irradiance
    } else {
        &environment.irradiance_other
    }
}

fn prefiltered_of(environment: &Environment, slot: usize) -> &CubeImage {
    if slot == 0 {
        &environment.prefiltered
    } else {
        &environment.prefiltered_other
    }
}

fn filter_irradiance(
    context: &Context,
    environment: &Environment,
    slot: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let params = FilterParams {
        size: IRRADIANCE_SIZE,
        samples: IRRADIANCE_SAMPLES,
        roughness: 0.0,
        source_size: SKY_SIZE as f32,
        diffuse: 1,
    };
    let target = irradiance_of(environment, slot);
    run_filter(
        context,
        environment,
        target.level_views[0],
        target,
        0,
        params,
    )
}

fn filter_prefiltered(
    context: &Context,
    environment: &Environment,
    slot: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let target = prefiltered_of(environment, slot);
    for level in 0..PREFILTERED_LEVELS {
        let params = FilterParams {
            size: (PREFILTERED_SIZE >> level).max(1),
            samples: PREFILTERED_SAMPLES,
            roughness: level as f32 / (PREFILTERED_LEVELS - 1) as f32,
            source_size: SKY_SIZE as f32,
            diffuse: 0,
        };
        run_filter(
            context,
            environment,
            target.level_views[level as usize],
            target,
            level,
            params,
        )?;
    }
    Ok(())
}

fn run_filter(
    context: &Context,
    environment: &Environment,
    target_view: vk::ImageView,
    target: &CubeImage,
    level: u32,
    params: FilterParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let set = allocate_set(
        context,
        environment.pool,
        environment.filter_descriptor_layout,
    )?;
    write_set(
        context,
        set,
        environment.sky.view,
        environment.sampler,
        target_view,
    );

    let command_buffer = begin_one_shot(context)?;
    transition_level(
        context,
        command_buffer,
        target,
        level,
        LayoutChange {
            old_layout: vk::ImageLayout::UNDEFINED,
            new_layout: vk::ImageLayout::GENERAL,
            source_access: vk::AccessFlags2::empty(),
            destination_access: vk::AccessFlags2::SHADER_STORAGE_WRITE,
            source_stage: vk::PipelineStageFlags2::TOP_OF_PIPE,
            destination_stage: vk::PipelineStageFlags2::COMPUTE_SHADER,
        },
    );
    unsafe {
        context.device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            environment.filter_pipeline,
        );
        context.device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            environment.filter_layout,
            0,
            &[set],
            &[],
        );
        context.device.cmd_push_constants(
            command_buffer,
            environment.filter_layout,
            vk::ShaderStageFlags::COMPUTE,
            0,
            bytemuck::bytes_of(&params),
        );
        context.device.cmd_dispatch(
            command_buffer,
            params.size.div_ceil(16),
            params.size.div_ceil(16),
            6,
        );
    }
    transition_level(
        context,
        command_buffer,
        target,
        level,
        LayoutChange {
            old_layout: vk::ImageLayout::GENERAL,
            new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            source_access: vk::AccessFlags2::SHADER_STORAGE_WRITE,
            destination_access: vk::AccessFlags2::SHADER_SAMPLED_READ,
            source_stage: vk::PipelineStageFlags2::COMPUTE_SHADER,
            destination_stage: vk::PipelineStageFlags2::FRAGMENT_SHADER,
        },
    );
    end_one_shot(context, command_buffer)?;
    Ok(())
}

const BRIGHTEST_RADIANCE: f32 = 65000.0;

fn decode_radiance(bytes: &[u8]) -> Result<(Vec<u16>, u32, u32), Box<dyn std::error::Error>> {
    let decoder = image::codecs::hdr::HdrDecoder::new(std::io::Cursor::new(bytes))?;
    let metadata = decoder.metadata();
    let image = image::DynamicImage::from_decoder(decoder)?;
    let pixels = image.to_rgb32f();
    let data = pixels
        .pixels()
        .flat_map(|pixel| {
            let held = |value: f32| {
                let settled = if value.is_finite() { value } else { 0.0 };
                half::f16::from_f32(settled.clamp(0.0, BRIGHTEST_RADIANCE)).to_bits()
            };
            [
                held(pixel.0[0]),
                held(pixel.0[1]),
                held(pixel.0[2]),
                half::f16::from_f32(1.0).to_bits(),
            ]
        })
        .collect();
    Ok((data, metadata.width, metadata.height))
}

fn upload_equirectangular(
    context: &Context,
    pixels: &[u16],
    width: u32,
    height: u32,
) -> Result<Image, Box<dyn std::error::Error>> {
    let create_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .format(CUBE_FORMAT)
        .extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .initial_layout(vk::ImageLayout::UNDEFINED);
    let image = unsafe { context.device.create_image(&create_info, None)? };
    let requirements = unsafe { context.device.get_image_memory_requirements(image) };
    let allocation = context
        .allocator()
        .lock()
        .map_err(|error| error.to_string())?
        .allocate(&gpu_allocator::vulkan::AllocationCreateDesc {
            name: "equirectangular source",
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
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(CUBE_FORMAT)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });
    let view = unsafe { context.device.create_image_view(&view_info, None)? };

    let mut staging = create_buffer(
        context,
        "equirectangular staging",
        std::mem::size_of_val(pixels) as u64,
        vk::BufferUsageFlags::TRANSFER_SRC,
        MemoryLocation::CpuToGpu,
    )?;
    write_buffer(&staging, pixels);

    let command_buffer = begin_one_shot(context)?;
    transition(
        &context.device,
        command_buffer,
        Transition {
            image,
            aspect: vk::ImageAspectFlags::COLOR,
            old_layout: vk::ImageLayout::UNDEFINED,
            new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            source_access: vk::AccessFlags2::empty(),
            destination_access: vk::AccessFlags2::TRANSFER_WRITE,
            source_stage: vk::PipelineStageFlags2::TOP_OF_PIPE,
            destination_stage: vk::PipelineStageFlags2::TRANSFER,
        },
    );
    let region = vk::BufferImageCopy::default()
        .image_subresource(vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        })
        .image_extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        });
    unsafe {
        context.device.cmd_copy_buffer_to_image(
            command_buffer,
            staging.buffer,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            std::slice::from_ref(&region),
        )
    };
    transition(
        &context.device,
        command_buffer,
        Transition {
            image,
            aspect: vk::ImageAspectFlags::COLOR,
            old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            source_access: vk::AccessFlags2::TRANSFER_WRITE,
            destination_access: vk::AccessFlags2::SHADER_SAMPLED_READ,
            source_stage: vk::PipelineStageFlags2::TRANSFER,
            destination_stage: vk::PipelineStageFlags2::COMPUTE_SHADER,
        },
    );
    end_one_shot(context, command_buffer)?;
    wait_for_sent_work(context)?;
    destroy_buffer(context, &mut staging);

    Ok(Image {
        image,
        view,
        allocation: Some(allocation),
        format: CUBE_FORMAT,
        extent: vk::Extent2D { width, height },
    })
}

struct LayoutChange {
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    source_access: vk::AccessFlags2,
    destination_access: vk::AccessFlags2,
    source_stage: vk::PipelineStageFlags2,
    destination_stage: vk::PipelineStageFlags2,
}

fn transition_cube(
    context: &Context,
    command_buffer: vk::CommandBuffer,
    cube: &CubeImage,
    change: LayoutChange,
) {
    cube_barrier(
        context,
        command_buffer,
        cube.image,
        vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: cube.levels,
            base_array_layer: 0,
            layer_count: 6,
        },
        change,
    );
}

fn transition_level(
    context: &Context,
    command_buffer: vk::CommandBuffer,
    cube: &CubeImage,
    level: u32,
    change: LayoutChange,
) {
    cube_barrier(
        context,
        command_buffer,
        cube.image,
        vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: level,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 6,
        },
        change,
    );
}

fn cube_barrier(
    context: &Context,
    command_buffer: vk::CommandBuffer,
    image: vk::Image,
    range: vk::ImageSubresourceRange,
    change: LayoutChange,
) {
    let barrier = vk::ImageMemoryBarrier2::default()
        .image(image)
        .old_layout(change.old_layout)
        .new_layout(change.new_layout)
        .src_access_mask(change.source_access)
        .dst_access_mask(change.destination_access)
        .src_stage_mask(change.source_stage)
        .dst_stage_mask(change.destination_stage)
        .subresource_range(range);
    let dependency =
        vk::DependencyInfo::default().image_memory_barriers(std::slice::from_ref(&barrier));
    unsafe {
        context
            .device
            .cmd_pipeline_barrier2(command_buffer, &dependency)
    };
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

fn create_surface_table_image(context: &Context) -> Result<Image, Box<dyn std::error::Error>> {
    let format = vk::Format::R16G16B16A16_SFLOAT;
    let create_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .format(format)
        .extent(vk::Extent3D {
            width: SURFACE_TABLE_SIZE,
            height: SURFACE_TABLE_SIZE,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .initial_layout(vk::ImageLayout::UNDEFINED);
    let image = unsafe { context.device.create_image(&create_info, None)? };
    let requirements = unsafe { context.device.get_image_memory_requirements(image) };
    let allocation = context
        .allocator()
        .lock()
        .map_err(|error| error.to_string())?
        .allocate(&gpu_allocator::vulkan::AllocationCreateDesc {
            name: "surface answer table",
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
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });
    let view = unsafe { context.device.create_image_view(&view_info, None)? };
    Ok(Image {
        image,
        view,
        allocation: Some(allocation),
        format,
        extent: vk::Extent2D {
            width: SURFACE_TABLE_SIZE,
            height: SURFACE_TABLE_SIZE,
        },
    })
}

fn gather_surface_table(
    context: &Context,
    environment: &Environment,
) -> Result<(), Box<dyn std::error::Error>> {
    let set = allocate_set(
        context,
        environment.pool,
        environment.surface_table_descriptor_layout,
    )?;
    let target = vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::GENERAL)
        .image_view(environment.surface_table.view);
    let write = vk::WriteDescriptorSet::default()
        .dst_set(set)
        .dst_binding(0)
        .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
        .image_info(std::slice::from_ref(&target));
    unsafe {
        context
            .device
            .update_descriptor_sets(std::slice::from_ref(&write), &[])
    };

    let command_buffer = begin_one_shot(context)?;
    crate::renderer::memory::transition(
        context,
        command_buffer,
        environment.surface_table.image,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::GENERAL,
    );
    let params = SurfaceTableParams {
        size: SURFACE_TABLE_SIZE,
        samples: SURFACE_TABLE_SAMPLES,
    };
    unsafe {
        context.device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            environment.surface_table_pipeline,
        );
        context.device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            environment.surface_table_layout,
            0,
            &[set],
            &[],
        );
        context.device.cmd_push_constants(
            command_buffer,
            environment.surface_table_layout,
            vk::ShaderStageFlags::COMPUTE,
            0,
            bytemuck::bytes_of(&params),
        );
        let groups = SURFACE_TABLE_SIZE.div_ceil(16);
        context
            .device
            .cmd_dispatch(command_buffer, groups, groups, 1);
    }
    crate::renderer::memory::transition(
        context,
        command_buffer,
        environment.surface_table.image,
        vk::ImageLayout::GENERAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );
    end_one_shot(context, command_buffer)?;
    wait_for_sent_work(context)
}

fn create_paint_layout(
    context: &Context,
) -> Result<vk::DescriptorSetLayout, Box<dyn std::error::Error>> {
    create_paint_layout_at(context, 1)
}

fn create_paint_layout_at(
    context: &Context,
    slot: u32,
) -> Result<vk::DescriptorSetLayout, Box<dyn std::error::Error>> {
    let binding = vk::DescriptorSetLayoutBinding::default()
        .binding(slot)
        .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::COMPUTE);
    let create_info =
        vk::DescriptorSetLayoutCreateInfo::default().bindings(std::slice::from_ref(&binding));
    Ok(unsafe {
        context
            .device
            .create_descriptor_set_layout(&create_info, None)?
    })
}

fn write_paint_set(context: &Context, set: vk::DescriptorSet, target_view: vk::ImageView) {
    let target = vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::GENERAL)
        .image_view(target_view);
    let write = vk::WriteDescriptorSet::default()
        .dst_set(set)
        .dst_binding(1)
        .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
        .image_info(std::slice::from_ref(&target));
    unsafe {
        context
            .device
            .update_descriptor_sets(std::slice::from_ref(&write), &[])
    };
}

fn write_set(
    context: &Context,
    set: vk::DescriptorSet,
    source_view: vk::ImageView,
    sampler: vk::Sampler,
    target_view: vk::ImageView,
) {
    let source = vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(source_view)
        .sampler(sampler);
    let target = vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::GENERAL)
        .image_view(target_view);
    let writes = [
        vk::WriteDescriptorSet::default()
            .dst_set(set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&source)),
        vk::WriteDescriptorSet::default()
            .dst_set(set)
            .dst_binding(1)
            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
            .image_info(std::slice::from_ref(&target)),
    ];
    unsafe { context.device.update_descriptor_sets(&writes, &[]) };
}

fn write_scene_sets(context: &Context, environment: &Environment) {
    for index in 0..FRAMES_IN_FLIGHT {
        write_one_scene_set(context, environment, index);
    }
}

fn write_one_scene_set(context: &Context, environment: &Environment, index: usize) {
    let infos = [
        vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(environment.irradiance.view)
            .sampler(environment.sampler),
        vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(environment.prefiltered.view)
            .sampler(environment.sampler),
        vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(environment.sky.view)
            .sampler(environment.sampler),
        vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(environment.area_tables.matrix.view)
            .sampler(environment.area_tables.sampler),
        vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(environment.area_tables.terms.view)
            .sampler(environment.area_tables.sampler),
        vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(environment.surface_table.view)
            .sampler(environment.area_tables.sampler),
        vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(environment.irradiance_other.view)
            .sampler(environment.sampler),
        vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(environment.prefiltered_other.view)
            .sampler(environment.sampler),
    ];
    let writes: Vec<vk::WriteDescriptorSet> = infos
        .iter()
        .enumerate()
        .map(|(binding, info)| {
            vk::WriteDescriptorSet::default()
                .dst_set(environment.scene_sets[index])
                .dst_binding(binding as u32)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(info))
        })
        .collect();
    unsafe { context.device.update_descriptor_sets(&writes, &[]) };
}

fn create_scene_layout(
    context: &Context,
) -> Result<vk::DescriptorSetLayout, Box<dyn std::error::Error>> {
    let bindings: Vec<vk::DescriptorSetLayoutBinding> = (0..8)
        .map(|binding| {
            vk::DescriptorSetLayoutBinding::default()
                .binding(binding)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::COMPUTE)
        })
        .collect();
    let create_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
    Ok(unsafe {
        context
            .device
            .create_descriptor_set_layout(&create_info, None)?
    })
}

fn create_descriptor_layout(
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
            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
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

fn create_scene_pool(context: &Context) -> Result<vk::DescriptorPool, Box<dyn std::error::Error>> {
    let sets = FRAMES_IN_FLIGHT as u32;
    let sizes = [vk::DescriptorPoolSize::default()
        .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(sets * 8)];
    let create_info = vk::DescriptorPoolCreateInfo::default()
        .max_sets(sets)
        .pool_sizes(&sizes);
    Ok(unsafe { context.device.create_descriptor_pool(&create_info, None)? })
}

fn create_descriptor_pool(
    context: &Context,
) -> Result<vk::DescriptorPool, Box<dyn std::error::Error>> {
    let sets = PREFILTERED_LEVELS + 8;
    let sizes = [
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(sets * 5),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_IMAGE)
            .descriptor_count(sets),
    ];
    let create_info = vk::DescriptorPoolCreateInfo::default()
        .max_sets(sets)
        .pool_sizes(&sizes);
    Ok(unsafe { context.device.create_descriptor_pool(&create_info, None)? })
}

fn create_sampler(context: &Context) -> Result<vk::Sampler, Box<dyn std::error::Error>> {
    let create_info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .max_lod(vk::LOD_CLAMP_NONE);
    Ok(unsafe { context.device.create_sampler(&create_info, None)? })
}

fn create_pipeline_layout(
    context: &Context,
    descriptor_layout: vk::DescriptorSetLayout,
    push_size: u32,
) -> Result<vk::PipelineLayout, Box<dyn std::error::Error>> {
    let layouts = [descriptor_layout];
    let range = vk::PushConstantRange::default()
        .stage_flags(vk::ShaderStageFlags::COMPUTE)
        .offset(0)
        .size(push_size);
    let create_info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(&layouts)
        .push_constant_ranges(std::slice::from_ref(&range));
    Ok(unsafe { context.device.create_pipeline_layout(&create_info, None)? })
}

fn create_compute_pipeline(
    context: &Context,
    layout: vk::PipelineLayout,
    code: &[u8],
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    let module = create_shader_module(context, code)?;
    let entry = std::ffi::CString::new("main")?;
    let stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::COMPUTE)
        .module(module)
        .name(&entry);
    let create_info = vk::ComputePipelineCreateInfo::default()
        .stage(stage)
        .layout(layout);
    let pipelines = unsafe {
        context
            .device
            .create_compute_pipelines(context.pipeline_cache, &[create_info], None)
    };
    unsafe { context.device.destroy_shader_module(module, None) };
    Ok(pipelines.map_err(|(_, error)| error)?[0])
}
