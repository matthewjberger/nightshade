use crate::renderer::context::Context;
use crate::renderer::memory::{Buffer, create_buffer, destroy_buffer, write_buffer};
use crate::renderer::pass::create_shader_module;
use ash::vk;
use gpu_allocator::MemoryLocation;
use nightshade::ecs::Entity;
use nightshade_render::data::config::{SkinnedAnimationSnapshot, SkinnedSkeletonData};
use nightshade_render::data::skinning::SkinningCache;
use std::collections::HashMap;

const MAX_CHAINS: usize = 256;
const MAX_INDICES: usize = 8192;
const MAX_SPRING_SLOTS: usize = 2048;
const GROUP_SIZE: u32 = 64;

const BINDING_BONES: u32 = 0;
const BINDING_CHAINS: u32 = 1;
const BINDING_JOINTS: u32 = 2;
const BINDING_CARRIED: u32 = 3;
const BINDING_REST_AXES: u32 = 4;
const BINDING_BONE_LENGTHS: u32 = 5;
const BINDING_SPRING_STATE: u32 = 6;
const BINDING_COUNT: u32 = 7;

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuLimbChain {
    root: u32,
    middle: u32,
    tip: u32,
    has_pole: u32,
    goal: [f32; 4],
    pole: [f32; 4],
    weight: f32,
    carried_start: u32,
    carried_count: u32,
    padding: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuReachChain {
    joint_start: u32,
    joint_count: u32,
    carried_start: u32,
    carried_count: u32,
    goal: [f32; 4],
    weight: f32,
    widest_swing: f32,
    rounds: u32,
    padding: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuAimChain {
    joint_start: u32,
    joint_count: u32,
    carried_start: u32,
    carried_count: u32,
    goal: [f32; 4],
    forward: [f32; 4],
    weight: f32,
    widest_swing: f32,
    padding_first: u32,
    padding_second: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuSpringChain {
    joint_start: u32,
    joint_count: u32,
    carried_start: u32,
    carried_count: u32,
    state_start: u32,
    restart: u32,
    enabled: u32,
    padding_first: u32,
    gravity: [f32; 4],
    stiffness: f32,
    damping: f32,
    step_seconds: f32,
    padding_second: f32,
}

#[derive(Clone, Default)]
pub struct ConstraintData {
    pub signature: u64,
    pub limbs: Vec<GpuLimbChain>,
    pub limb_carried: Vec<u32>,
    pub reaches: Vec<GpuReachChain>,
    pub reach_joints: Vec<u32>,
    pub reach_carried: Vec<u32>,
    pub aims: Vec<GpuAimChain>,
    pub aim_joints: Vec<u32>,
    pub aim_carried: Vec<u32>,
    pub springs: Vec<GpuSpringChain>,
    pub spring_joints: Vec<u32>,
    pub spring_carried: Vec<u32>,
    pub rest_axes: Vec<[f32; 4]>,
    pub bone_lengths: Vec<f32>,
}

fn collect_carried(
    skeleton: &SkinnedSkeletonData,
    parent_of: &HashMap<u32, Option<u32>>,
    base_bone_index: u32,
    tip_local: u32,
    carried: &mut Vec<u32>,
) -> (u32, u32) {
    let start = carried.len() as u32;
    for joint in &skeleton.joints_ordered {
        if joint.local_index == tip_local {
            continue;
        }
        let mut current = joint.parent_local;
        let mut below_tip = false;
        while let Some(parent) = current {
            if parent == tip_local {
                below_tip = true;
                break;
            }
            current = parent_of.get(&parent).copied().flatten();
        }
        if below_tip {
            carried.push(base_bone_index + joint.local_index);
        }
    }
    (start, carried.len() as u32 - start)
}

fn weight_of(runtime: [f32; 8], value: usize, switch: usize) -> f32 {
    if runtime[switch] > 0.0 {
        runtime[value]
    } else {
        0.0
    }
}

fn goal_of(runtime: [f32; 8]) -> [f32; 4] {
    [runtime[0], runtime[1], runtime[2], 0.0]
}

fn swing_of(runtime: [f32; 8]) -> f32 {
    if runtime[5] > 0.0 {
        runtime[5]
    } else {
        std::f32::consts::PI
    }
}

fn runtime_of(table: &HashMap<(Entity, usize), [f32; 8]>, player: Entity, chain: u32) -> [f32; 8] {
    table
        .get(&(player, chain as usize))
        .copied()
        .unwrap_or([0.0; 8])
}

pub fn build(snapshot: &SkinnedAnimationSnapshot, cache: &SkinningCache) -> ConstraintData {
    let mut data = ConstraintData {
        signature: snapshot.signature,
        ..ConstraintData::default()
    };
    let mut spring_slots: u32 = 0;

    for skeleton in &snapshot.skeletons {
        let Some(&skin_index) = cache.entity_skin_indices.get(&skeleton.skin_entity) else {
            continue;
        };
        let base_bone_index = cache.get_base_bone_index(skin_index);
        let player = skeleton.player_entity;
        let mut parent_of: HashMap<u32, Option<u32>> = HashMap::new();
        for joint in &skeleton.joints_ordered {
            parent_of.insert(joint.local_index, joint.parent_local);
        }

        for chain in &skeleton.ik_chains {
            if data.limbs.len() >= MAX_CHAINS {
                break;
            }
            let (carried_start, carried_count) = collect_carried(
                skeleton,
                &parent_of,
                base_bone_index,
                chain.tip_local,
                &mut data.limb_carried,
            );
            if data.limb_carried.len() > MAX_INDICES {
                data.limb_carried.truncate(carried_start as usize);
                continue;
            }
            let runtime = runtime_of(&snapshot.ik_runtime, player, chain.chain_index);
            data.limbs.push(GpuLimbChain {
                root: base_bone_index + chain.root_local,
                middle: base_bone_index + chain.mid_local,
                tip: base_bone_index + chain.tip_local,
                has_pole: u32::from(chain.has_pole),
                goal: goal_of(runtime),
                pole: [runtime[3], runtime[4], runtime[5], 0.0],
                weight: weight_of(runtime, 6, 7),
                carried_start,
                carried_count,
                padding: 0,
            });
        }

        for chain in &skeleton.ik_multi_chains {
            let joint_count = chain.joint_locals.len() as u32;
            if joint_count < 2 || data.reaches.len() >= MAX_CHAINS {
                continue;
            }
            let joint_start = data.reach_joints.len() as u32;
            for &local in &chain.joint_locals {
                data.reach_joints.push(base_bone_index + local);
            }
            let (carried_start, carried_count) = collect_carried(
                skeleton,
                &parent_of,
                base_bone_index,
                chain.joint_locals[chain.joint_locals.len() - 1],
                &mut data.reach_carried,
            );
            if data.reach_joints.len() > MAX_INDICES || data.reach_carried.len() > MAX_INDICES {
                data.reach_joints.truncate(joint_start as usize);
                data.reach_carried.truncate(carried_start as usize);
                continue;
            }
            let runtime = runtime_of(&snapshot.ik_multi_runtime, player, chain.chain_index);
            data.reaches.push(GpuReachChain {
                joint_start,
                joint_count,
                carried_start,
                carried_count,
                goal: goal_of(runtime),
                weight: weight_of(runtime, 3, 4),
                widest_swing: swing_of(runtime),
                rounds: runtime[6].max(1.0) as u32,
                padding: 0,
            });
        }

        for chain in &skeleton.aim_constraints {
            let joint_count = chain.joint_locals.len() as u32;
            if joint_count == 0 || data.aims.len() >= MAX_CHAINS {
                continue;
            }
            let joint_start = data.aim_joints.len() as u32;
            for &local in &chain.joint_locals {
                data.aim_joints.push(base_bone_index + local);
            }
            let (carried_start, carried_count) = collect_carried(
                skeleton,
                &parent_of,
                base_bone_index,
                chain.joint_locals[chain.joint_locals.len() - 1],
                &mut data.aim_carried,
            );
            if data.aim_joints.len() > MAX_INDICES || data.aim_carried.len() > MAX_INDICES {
                data.aim_joints.truncate(joint_start as usize);
                data.aim_carried.truncate(carried_start as usize);
                continue;
            }
            let runtime = runtime_of(&snapshot.aim_runtime, player, chain.chain_index);
            data.aims.push(GpuAimChain {
                joint_start,
                joint_count,
                carried_start,
                carried_count,
                goal: goal_of(runtime),
                forward: [chain.forward[0], chain.forward[1], chain.forward[2], 0.0],
                weight: weight_of(runtime, 3, 4),
                widest_swing: swing_of(runtime),
                padding_first: 0,
                padding_second: 0,
            });
        }

        for chain in &skeleton.spring_chains {
            let joint_count = chain.joint_locals.len() as u32;
            if joint_count < 2
                || data.springs.len() >= MAX_CHAINS
                || spring_slots + joint_count > MAX_SPRING_SLOTS as u32
            {
                continue;
            }
            let joint_start = data.spring_joints.len() as u32;
            for (along, &local) in chain.joint_locals.iter().enumerate() {
                data.spring_joints.push(base_bone_index + local);
                let axis = chain
                    .rest_axes
                    .get(along)
                    .copied()
                    .unwrap_or([0.0, 1.0, 0.0]);
                data.rest_axes.push([axis[0], axis[1], axis[2], 0.0]);
                data.bone_lengths
                    .push(chain.bone_lengths.get(along).copied().unwrap_or(0.0));
            }
            let (carried_start, carried_count) = collect_carried(
                skeleton,
                &parent_of,
                base_bone_index,
                chain.joint_locals[chain.joint_locals.len() - 1],
                &mut data.spring_carried,
            );
            if data.spring_joints.len() > MAX_INDICES || data.spring_carried.len() > MAX_INDICES {
                data.spring_joints.truncate(joint_start as usize);
                data.spring_carried.truncate(carried_start as usize);
                data.rest_axes.truncate(joint_start as usize);
                data.bone_lengths.truncate(joint_start as usize);
                continue;
            }
            let runtime = runtime_of(&snapshot.spring_runtime, player, chain.chain_index);
            data.springs.push(GpuSpringChain {
                joint_start,
                joint_count,
                carried_start,
                carried_count,
                state_start: spring_slots,
                restart: 0,
                enabled: u32::from(runtime[5] > 0.0),
                padding_first: 0,
                gravity: [runtime[0], runtime[1], runtime[2], 0.0],
                stiffness: runtime[3].clamp(0.0, 1.0),
                damping: runtime[4].clamp(0.0, 1.0),
                step_seconds: runtime[6],
                padding_second: 0.0,
            });
            spring_slots += joint_count;
        }
    }

    data
}

struct SolverFrame {
    set: vk::DescriptorSet,
    chains: Buffer,
    joints: Buffer,
    carried: Buffer,
}

struct Solver {
    pipeline: vk::Pipeline,
    frames: Vec<SolverFrame>,
    count: u32,
}

pub struct Constraints {
    descriptor_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    descriptor_pool: vk::DescriptorPool,
    limb: Solver,
    reach: Solver,
    aim: Solver,
    spring: Solver,
    rest_axes: Buffer,
    bone_lengths: Buffer,
    spring_state: Buffer,
    signature: u64,
    settled: bool,
}

struct SolverSetup<'a> {
    pool: vk::DescriptorPool,
    descriptor_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    frames_in_flight: usize,
    name: &'a str,
    chain_size: u64,
    code: &'a [u8],
}

pub fn create(
    context: &Context,
    frames_in_flight: usize,
) -> Result<Constraints, Box<dyn std::error::Error>> {
    let descriptor_layout = create_descriptor_layout(context)?;
    let layouts = [descriptor_layout];
    let create_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts);
    let pipeline_layout = unsafe { context.device.create_pipeline_layout(&create_info, None)? };

    let size = vk::DescriptorPoolSize::default()
        .ty(vk::DescriptorType::STORAGE_BUFFER)
        .descriptor_count(BINDING_COUNT * (frames_in_flight * 4) as u32);
    let pool_info = vk::DescriptorPoolCreateInfo::default()
        .pool_sizes(std::slice::from_ref(&size))
        .max_sets((frames_in_flight * 4) as u32);
    let descriptor_pool = unsafe { context.device.create_descriptor_pool(&pool_info, None)? };

    let rest_axes = storage(
        context,
        "constraint rest axes",
        (MAX_SPRING_SLOTS * std::mem::size_of::<[f32; 4]>()) as u64,
    )?;
    let bone_lengths = storage(
        context,
        "constraint bone lengths",
        (MAX_SPRING_SLOTS * std::mem::size_of::<f32>()) as u64,
    )?;
    let spring_state = create_buffer(
        context,
        "constraint spring state",
        (MAX_SPRING_SLOTS * 2 * std::mem::size_of::<[f32; 4]>()) as u64,
        vk::BufferUsageFlags::STORAGE_BUFFER,
        MemoryLocation::GpuOnly,
    )?;

    let limb = create_solver(
        context,
        &SolverSetup {
            pool: descriptor_pool,
            descriptor_layout,
            pipeline_layout,
            frames_in_flight,
            name: "constraint limb",
            chain_size: std::mem::size_of::<GpuLimbChain>() as u64,
            code: include_bytes!("shaders/constraint_limb.comp.spv"),
        },
    )?;
    let reach = create_solver(
        context,
        &SolverSetup {
            pool: descriptor_pool,
            descriptor_layout,
            pipeline_layout,
            frames_in_flight,
            name: "constraint reach",
            chain_size: std::mem::size_of::<GpuReachChain>() as u64,
            code: include_bytes!("shaders/constraint_reach.comp.spv"),
        },
    )?;
    let aim = create_solver(
        context,
        &SolverSetup {
            pool: descriptor_pool,
            descriptor_layout,
            pipeline_layout,
            frames_in_flight,
            name: "constraint aim",
            chain_size: std::mem::size_of::<GpuAimChain>() as u64,
            code: include_bytes!("shaders/constraint_aim.comp.spv"),
        },
    )?;
    let spring = create_solver(
        context,
        &SolverSetup {
            pool: descriptor_pool,
            descriptor_layout,
            pipeline_layout,
            frames_in_flight,
            name: "constraint spring",
            chain_size: std::mem::size_of::<GpuSpringChain>() as u64,
            code: include_bytes!("shaders/constraint_spring.comp.spv"),
        },
    )?;

    Ok(Constraints {
        descriptor_layout,
        pipeline_layout,
        descriptor_pool,
        limb,
        reach,
        aim,
        spring,
        rest_axes,
        bone_lengths,
        spring_state,
        signature: u64::MAX,
        settled: false,
    })
}

pub fn upload(
    context: &Context,
    constraints: &mut Constraints,
    data: &ConstraintData,
    bones: vk::Buffer,
    frame_index: usize,
) {
    if constraints.signature != data.signature {
        constraints.signature = data.signature;
        constraints.settled = false;
    }

    constraints.limb.count = data.limbs.len() as u32;
    constraints.reach.count = data.reaches.len() as u32;
    constraints.aim.count = data.aims.len() as u32;
    constraints.spring.count = data.springs.len() as u32;

    write_buffer(&constraints.rest_axes, &data.rest_axes);
    write_buffer(&constraints.bone_lengths, &data.bone_lengths);

    let limb = &constraints.limb.frames[frame_index];
    write_buffer(&limb.chains, &data.limbs);
    write_buffer(&limb.carried, &data.limb_carried);

    let reach = &constraints.reach.frames[frame_index];
    write_buffer(&reach.chains, &data.reaches);
    write_buffer(&reach.joints, &data.reach_joints);
    write_buffer(&reach.carried, &data.reach_carried);

    let aim = &constraints.aim.frames[frame_index];
    write_buffer(&aim.chains, &data.aims);
    write_buffer(&aim.joints, &data.aim_joints);
    write_buffer(&aim.carried, &data.aim_carried);

    let spring = &constraints.spring.frames[frame_index];
    let mut springs = data.springs.clone();
    if !constraints.settled {
        for chain in &mut springs {
            chain.restart = 1;
        }
    }
    write_buffer(&spring.chains, &springs);
    write_buffer(&spring.joints, &data.spring_joints);
    write_buffer(&spring.carried, &data.spring_carried);
    if !springs.is_empty() {
        constraints.settled = true;
    }

    for solver in [
        &constraints.limb,
        &constraints.reach,
        &constraints.aim,
        &constraints.spring,
    ] {
        write_set(
            context,
            &solver.frames[frame_index],
            bones,
            &constraints.rest_axes,
            &constraints.bone_lengths,
            &constraints.spring_state,
        );
    }
}

pub fn dispatch(
    context: &Context,
    constraints: &Constraints,
    command_buffer: vk::CommandBuffer,
    bones: vk::Buffer,
    frame_index: usize,
) {
    let solvers = [
        &constraints.limb,
        &constraints.reach,
        &constraints.aim,
        &constraints.spring,
    ];
    if solvers.iter().all(|solver| solver.count == 0) {
        return;
    }
    let device = &context.device;
    for solver in solvers {
        if solver.count == 0 {
            continue;
        }
        unsafe {
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                solver.pipeline,
            );
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                constraints.pipeline_layout,
                0,
                &[solver.frames[frame_index].set],
                &[],
            );
            device.cmd_dispatch(command_buffer, solver.count.div_ceil(GROUP_SIZE), 1, 1);
        }
        between(device, command_buffer, bones);
    }
}

fn between(device: &ash::Device, command_buffer: vk::CommandBuffer, bones: vk::Buffer) {
    let memory = vk::BufferMemoryBarrier2::default()
        .src_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
        .src_access_mask(vk::AccessFlags2::SHADER_STORAGE_WRITE)
        .dst_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
        .dst_access_mask(
            vk::AccessFlags2::SHADER_STORAGE_READ | vk::AccessFlags2::SHADER_STORAGE_WRITE,
        )
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .buffer(bones)
        .size(vk::WHOLE_SIZE);
    let dependency =
        vk::DependencyInfo::default().buffer_memory_barriers(std::slice::from_ref(&memory));
    unsafe { device.cmd_pipeline_barrier2(command_buffer, &dependency) };
}

pub fn destroy(context: &Context, constraints: &mut Constraints) {
    unsafe {
        for solver in [
            &mut constraints.limb,
            &mut constraints.reach,
            &mut constraints.aim,
            &mut constraints.spring,
        ] {
            context.device.destroy_pipeline(solver.pipeline, None);
            for frame in &mut solver.frames {
                destroy_buffer(context, &mut frame.chains);
                destroy_buffer(context, &mut frame.joints);
                destroy_buffer(context, &mut frame.carried);
            }
        }
        destroy_buffer(context, &mut constraints.rest_axes);
        destroy_buffer(context, &mut constraints.bone_lengths);
        destroy_buffer(context, &mut constraints.spring_state);
        context
            .device
            .destroy_descriptor_pool(constraints.descriptor_pool, None);
        context
            .device
            .destroy_pipeline_layout(constraints.pipeline_layout, None);
        context
            .device
            .destroy_descriptor_set_layout(constraints.descriptor_layout, None);
    }
}

fn create_descriptor_layout(
    context: &Context,
) -> Result<vk::DescriptorSetLayout, Box<dyn std::error::Error>> {
    let bindings: Vec<vk::DescriptorSetLayoutBinding> = (0..BINDING_COUNT)
        .map(|binding| {
            vk::DescriptorSetLayoutBinding::default()
                .binding(binding)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE)
        })
        .collect();
    let create_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
    Ok(unsafe {
        context
            .device
            .create_descriptor_set_layout(&create_info, None)?
    })
}

fn storage(context: &Context, name: &str, size: u64) -> Result<Buffer, Box<dyn std::error::Error>> {
    create_buffer(
        context,
        name,
        size,
        vk::BufferUsageFlags::STORAGE_BUFFER,
        MemoryLocation::CpuToGpu,
    )
}

fn create_solver(
    context: &Context,
    setup: &SolverSetup<'_>,
) -> Result<Solver, Box<dyn std::error::Error>> {
    let pipeline = create_compute_pipeline(context, setup.pipeline_layout, setup.code)?;
    let indices = (MAX_INDICES * std::mem::size_of::<u32>()) as u64;
    let mut frames = Vec::with_capacity(setup.frames_in_flight);
    for _ in 0..setup.frames_in_flight {
        let layouts = [setup.descriptor_layout];
        let allocate_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(setup.pool)
            .set_layouts(&layouts);
        let set = unsafe { context.device.allocate_descriptor_sets(&allocate_info)?[0] };
        frames.push(SolverFrame {
            set,
            chains: storage(context, setup.name, MAX_CHAINS as u64 * setup.chain_size)?,
            joints: storage(context, setup.name, indices)?,
            carried: storage(context, setup.name, indices)?,
        });
    }
    Ok(Solver {
        pipeline,
        frames,
        count: 0,
    })
}

fn write_set(
    context: &Context,
    frame: &SolverFrame,
    bones: vk::Buffer,
    rest_axes: &Buffer,
    bone_lengths: &Buffer,
    spring_state: &Buffer,
) {
    let bound = [
        (BINDING_BONES, bones, vk::WHOLE_SIZE),
        (BINDING_CHAINS, frame.chains.buffer, frame.chains.size),
        (BINDING_JOINTS, frame.joints.buffer, frame.joints.size),
        (BINDING_CARRIED, frame.carried.buffer, frame.carried.size),
        (BINDING_REST_AXES, rest_axes.buffer, rest_axes.size),
        (BINDING_BONE_LENGTHS, bone_lengths.buffer, bone_lengths.size),
        (BINDING_SPRING_STATE, spring_state.buffer, spring_state.size),
    ];
    let infos: Vec<vk::DescriptorBufferInfo> = bound
        .iter()
        .map(|(_, buffer, size)| {
            vk::DescriptorBufferInfo::default()
                .buffer(*buffer)
                .offset(0)
                .range(*size)
        })
        .collect();
    let writes: Vec<vk::WriteDescriptorSet> = bound
        .iter()
        .zip(infos.iter())
        .map(|((binding, _, _), info)| {
            vk::WriteDescriptorSet::default()
                .dst_set(frame.set)
                .dst_binding(*binding)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(std::slice::from_ref(info))
        })
        .collect();
    unsafe { context.device.update_descriptor_sets(&writes, &[]) };
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
