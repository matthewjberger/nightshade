use crate::renderer::context::Context;
use crate::renderer::memory::{Buffer, create_buffer, destroy_buffer};
use ash::vk;
use gpu_allocator::MemoryLocation;

pub struct Rays {
    meshes: Vec<Option<Level>>,
    posed: Vec<Level>,
    posed_addresses: Vec<u64>,
    pending: Vec<Pending>,
    scratch_used: u64,
    world: Option<Level>,
    addresses: Vec<u64>,
    instances: Vec<Buffer>,
    instance_capacity: Vec<u64>,
    scratch: Vec<Buffer>,
    scratch_capacity: Vec<u64>,
    turn: usize,
    pub count: u32,
    standing: u64,
    replaced: Vec<(u64, Level)>,
    let_go: Vec<(u64, Buffer)>,
    frames: u64,
}

enum Pending {
    Mesh {
        geometry: MeshGeometry,
        target: vk::AccelerationStructureKHR,
        scratch: u64,
        slot: Option<u32>,
    },
    World {
        count: u32,
        target: vk::AccelerationStructureKHR,
        scratch: u64,
    },
}

struct Level {
    handle: vk::AccelerationStructureKHR,
    buffer: Buffer,
}

pub struct MeshGeometry {
    pub vertices: u64,
    pub vertex_count: u32,
    pub vertex_stride: u64,
    pub indices: u64,
    pub index_count: u32,
}

#[derive(Clone, Copy)]
pub struct ArenaAddresses {
    pub vertices: u64,
    pub indices: u64,
}

const UNPLACED: ArenaAddresses = ArenaAddresses {
    vertices: 0,
    indices: 0,
};

pub enum Traced {
    Mesh(u32),
    Posed(u32),
}

pub struct Placement {
    pub transform: [[f32; 4]; 4],
    pub traced: Traced,
    pub object: u32,
    pub masked: bool,
}

pub fn create_rays(context: &Context) -> Result<Rays, Box<dyn std::error::Error>> {
    let mut instances = Vec::new();
    let mut scratch = Vec::new();
    for _ in 0..crate::renderer::context::FRAMES_IN_FLIGHT {
        instances.push(create_instance_buffer(context, 1)?);
        scratch.push(create_scratch(context, 1)?);
    }
    Ok(Rays {
        meshes: Vec::new(),
        posed: Vec::new(),
        posed_addresses: Vec::new(),
        pending: Vec::new(),
        scratch_used: 0,
        world: None,
        addresses: Vec::new(),
        instances,
        instance_capacity: vec![1; crate::renderer::context::FRAMES_IN_FLIGHT],
        scratch,
        scratch_capacity: vec![1; crate::renderer::context::FRAMES_IN_FLIGHT],
        turn: 0,
        count: 0,
        standing: 0,
        replaced: Vec::new(),
        let_go: Vec::new(),
        frames: 0,
    })
}

pub fn destroy_rays(context: &Context, rays: &mut Rays) {
    let Some(acceleration) = context.acceleration.as_ref() else {
        return;
    };
    for (_, level) in &mut rays.replaced {
        release(context, acceleration, level);
    }
    rays.replaced.clear();
    for level in rays.meshes.iter_mut().flatten() {
        release(context, acceleration, level);
    }
    rays.meshes.clear();
    for level in &mut rays.posed {
        release(context, acceleration, level);
    }
    rays.posed.clear();
    rays.posed_addresses.clear();
    if let Some(level) = rays.world.as_mut() {
        release(context, acceleration, level);
    }
    rays.world = None;
    for buffer in rays.instances.iter_mut().chain(rays.scratch.iter_mut()) {
        destroy_buffer(context, buffer);
    }
    for (_, buffer) in &mut rays.let_go {
        destroy_buffer(context, buffer);
    }
    rays.let_go.clear();
}

fn release(
    context: &Context,
    acceleration: &ash::khr::acceleration_structure::Device,
    level: &mut Level,
) {
    unsafe { acceleration.destroy_acceleration_structure(level.handle, None) };
    destroy_buffer(context, &mut level.buffer);
}

pub fn mesh_is_ready(rays: &Rays, slot: u32) -> bool {
    rays.addresses
        .get(slot as usize)
        .is_some_and(|address| *address != 0)
}

pub fn forget_meshes(rays: &mut Rays) {
    let replaced_on = rays.frames;
    for level in rays.meshes.drain(..).flatten() {
        rays.replaced.push((replaced_on, level));
    }
    rays.addresses.clear();
    rays.standing = 0;
    rays.count = 0;
}

pub fn forget_mesh(rays: &mut Rays, slot: u32) {
    let slot = slot as usize;
    if let Some(level) = rays.meshes.get_mut(slot).and_then(Option::take) {
        let replaced_on = rays.frames;
        rays.replaced.push((replaced_on, level));
    }
    if let Some(address) = rays.addresses.get_mut(slot) {
        *address = 0;
    }
}

pub fn traces(context: &Context) -> bool {
    context.acceleration.is_some()
}

pub fn address_of(context: &Context, buffer: &Buffer) -> u64 {
    let info = vk::BufferDeviceAddressInfo::default().buffer(buffer.buffer);
    unsafe { context.device.get_buffer_device_address(&info) }
}

pub fn build_mesh(
    rays: &mut Rays,
    context: &Context,
    slot: u32,
    geometry: &MeshGeometry,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(acceleration) = context.acceleration.as_ref() else {
        return Ok(());
    };
    if geometry.index_count < 3 {
        return Ok(());
    }
    let named = slot;
    let slot = slot as usize;
    if rays.meshes.len() <= slot {
        rays.meshes.resize_with(slot + 1, || None);
        rays.addresses.resize(slot + 1, 0);
    }

    let holds = triangles_of(geometry, UNPLACED);
    let triangle_count = geometry.index_count / 3;

    let info = vk::AccelerationStructureBuildGeometryInfoKHR::default()
        .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
        .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
        .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
        .geometries(std::slice::from_ref(&holds));

    let mut sizes = vk::AccelerationStructureBuildSizesInfoKHR::default();
    unsafe {
        acceleration.get_acceleration_structure_build_sizes(
            vk::AccelerationStructureBuildTypeKHR::DEVICE,
            &info,
            &[triangle_count],
            &mut sizes,
        )
    };

    let level = make_level(
        context,
        acceleration,
        sizes.acceleration_structure_size,
        vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL,
        "mesh to trace",
    )?;
    let scratch = take_scratch(rays, sizes.build_scratch_size);
    reserve_scratch(rays, context, rays.scratch_used)?;
    rays.pending.push(Pending::Mesh {
        geometry: MeshGeometry {
            vertices: geometry.vertices,
            vertex_count: geometry.vertex_count,
            vertex_stride: geometry.vertex_stride,
            indices: geometry.indices,
            index_count: geometry.index_count,
        },
        target: level.handle,
        scratch,
        slot: Some(named),
    });

    if let Some(standing) = rays.meshes[slot].take() {
        let replaced_on = rays.frames;
        rays.replaced.push((replaced_on, standing));
    }
    let address_info = vk::AccelerationStructureDeviceAddressInfoKHR::default()
        .acceleration_structure(level.handle);
    rays.addresses[slot] =
        unsafe { acceleration.get_acceleration_structure_device_address(&address_info) };
    rays.meshes[slot] = Some(level);
    Ok(())
}

pub fn build_posed(
    rays: &mut Rays,
    context: &Context,
    poses: &[MeshGeometry],
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(acceleration) = context.acceleration.as_ref() else {
        return Ok(());
    };
    let replaced_on = rays.frames;
    for level in rays.posed.drain(..) {
        rays.replaced.push((replaced_on, level));
    }
    rays.posed_addresses.clear();
    if poses.is_empty() {
        return Ok(());
    }

    for pose in poses {
        let holds = triangles_of(pose, UNPLACED);
        let count = pose.index_count / 3;
        let info = vk::AccelerationStructureBuildGeometryInfoKHR::default()
            .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
            .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_BUILD)
            .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
            .geometries(std::slice::from_ref(&holds));
        let mut sizes = vk::AccelerationStructureBuildSizesInfoKHR::default();
        unsafe {
            acceleration.get_acceleration_structure_build_sizes(
                vk::AccelerationStructureBuildTypeKHR::DEVICE,
                &info,
                std::slice::from_ref(&count),
                &mut sizes,
            )
        };
        let level = make_level(
            context,
            acceleration,
            sizes.acceleration_structure_size,
            vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL,
            "posed run to trace",
        )?;
        let address_info = vk::AccelerationStructureDeviceAddressInfoKHR::default()
            .acceleration_structure(level.handle);
        rays.posed_addresses
            .push(unsafe { acceleration.get_acceleration_structure_device_address(&address_info) });
        let scratch = take_scratch(rays, sizes.build_scratch_size);
        rays.pending.push(Pending::Mesh {
            geometry: MeshGeometry {
                vertices: pose.vertices,
                vertex_count: pose.vertex_count,
                vertex_stride: pose.vertex_stride,
                indices: pose.indices,
                index_count: pose.index_count,
            },
            target: level.handle,
            scratch,
            slot: None,
        });
        rays.posed.push(level);
    }
    reserve_scratch(rays, context, rays.scratch_used)?;
    rays.standing = 0;
    Ok(())
}

fn take_scratch(rays: &mut Rays, size: u64) -> u64 {
    let at = rays.scratch_used;
    rays.scratch_used += size.next_multiple_of(SCRATCH_STEP);
    at
}

pub fn record_builds(
    rays: &mut Rays,
    context: &Context,
    command_buffer: vk::CommandBuffer,
    arena: ArenaAddresses,
) {
    let Some(acceleration) = context.acceleration.as_ref() else {
        return;
    };
    if rays.pending.is_empty() {
        return;
    }
    let scratch = address_of(context, &rays.scratch[rays.turn]);
    let pending = std::mem::take(&mut rays.pending);
    rays.scratch_used = 0;
    posed_first(context, command_buffer);

    let instances = vk::AccelerationStructureGeometryInstancesDataKHR::default().data(
        vk::DeviceOrHostAddressConstKHR {
            device_address: address_of(context, &rays.instances[rays.turn]),
        },
    );
    let holds: Vec<vk::AccelerationStructureGeometryKHR> = pending
        .iter()
        .map(|build| match build {
            Pending::Mesh { geometry, .. } => triangles_of(geometry, arena),
            Pending::World { .. } => vk::AccelerationStructureGeometryKHR::default()
                .geometry_type(vk::GeometryTypeKHR::INSTANCES)
                .geometry(vk::AccelerationStructureGeometryDataKHR { instances }),
        })
        .collect();

    for over_the_world in [false, true] {
        let mut infos: Vec<vk::AccelerationStructureBuildGeometryInfoKHR> = Vec::new();
        let mut ranges: Vec<vk::AccelerationStructureBuildRangeInfoKHR> = Vec::new();
        for (build, holds) in pending.iter().zip(holds.iter()) {
            let (kind, flags, count, target, at) = match build {
                Pending::Mesh {
                    geometry,
                    target,
                    scratch,
                    slot,
                } => (
                    vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL,
                    if slot.is_some() {
                        vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE
                    } else {
                        vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_BUILD
                    },
                    geometry.index_count / 3,
                    *target,
                    *scratch,
                ),
                Pending::World {
                    count,
                    target,
                    scratch,
                } => (
                    vk::AccelerationStructureTypeKHR::TOP_LEVEL,
                    vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE,
                    *count,
                    *target,
                    *scratch,
                ),
            };
            if (kind == vk::AccelerationStructureTypeKHR::TOP_LEVEL) != over_the_world {
                continue;
            }
            infos.push(
                vk::AccelerationStructureBuildGeometryInfoKHR::default()
                    .ty(kind)
                    .flags(flags)
                    .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
                    .geometries(std::slice::from_ref(holds))
                    .dst_acceleration_structure(target)
                    .scratch_data(vk::DeviceOrHostAddressKHR {
                        device_address: scratch + at,
                    }),
            );
            ranges
                .push(vk::AccelerationStructureBuildRangeInfoKHR::default().primitive_count(count));
        }
        if infos.is_empty() {
            continue;
        }
        let each: Vec<&[vk::AccelerationStructureBuildRangeInfoKHR]> =
            ranges.iter().map(std::slice::from_ref).collect();
        unsafe {
            acceleration.cmd_build_acceleration_structures(command_buffer, &infos, &each);
        }
        settled(context, command_buffer, over_the_world);
    }
}

fn posed_first(context: &Context, command_buffer: vk::CommandBuffer) {
    let memory = vk::MemoryBarrier2::default()
        .src_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
        .src_access_mask(vk::AccessFlags2::SHADER_STORAGE_WRITE)
        .dst_stage_mask(vk::PipelineStageFlags2::ACCELERATION_STRUCTURE_BUILD_KHR)
        .dst_access_mask(vk::AccessFlags2::SHADER_READ);
    let dependency = vk::DependencyInfo::default().memory_barriers(std::slice::from_ref(&memory));
    unsafe {
        context
            .device
            .cmd_pipeline_barrier2(command_buffer, &dependency)
    };
}

fn settled(context: &Context, command_buffer: vk::CommandBuffer, over_the_world: bool) {
    let reaches = if over_the_world {
        crate::renderer::pass::REACHED_ON_ANY_QUEUE
    } else {
        vk::PipelineStageFlags2::ACCELERATION_STRUCTURE_BUILD_KHR
    };
    let reads = if over_the_world {
        vk::AccessFlags2::ACCELERATION_STRUCTURE_READ_KHR
    } else {
        vk::AccessFlags2::ACCELERATION_STRUCTURE_READ_KHR
            | vk::AccessFlags2::ACCELERATION_STRUCTURE_WRITE_KHR
    };
    let memory = vk::MemoryBarrier2::default()
        .src_stage_mask(vk::PipelineStageFlags2::ACCELERATION_STRUCTURE_BUILD_KHR)
        .src_access_mask(vk::AccessFlags2::ACCELERATION_STRUCTURE_WRITE_KHR)
        .dst_stage_mask(reaches)
        .dst_access_mask(reads);
    let dependency = vk::DependencyInfo::default().memory_barriers(std::slice::from_ref(&memory));
    unsafe {
        context
            .device
            .cmd_pipeline_barrier2(command_buffer, &dependency)
    };
}

fn triangles_of(
    geometry: &MeshGeometry,
    arena: ArenaAddresses,
) -> vk::AccelerationStructureGeometryKHR<'_> {
    let triangles = vk::AccelerationStructureGeometryTrianglesDataKHR::default()
        .vertex_format(vk::Format::R32G32B32_SFLOAT)
        .vertex_data(vk::DeviceOrHostAddressConstKHR {
            device_address: arena.vertices + geometry.vertices,
        })
        .vertex_stride(geometry.vertex_stride)
        .max_vertex(geometry.vertex_count.saturating_sub(1))
        .index_type(vk::IndexType::UINT32)
        .index_data(vk::DeviceOrHostAddressConstKHR {
            device_address: arena.indices + geometry.indices,
        });
    vk::AccelerationStructureGeometryKHR::default()
        .geometry_type(vk::GeometryTypeKHR::TRIANGLES)
        .geometry(vk::AccelerationStructureGeometryDataKHR { triangles })
}

const SCRATCH_STEP: u64 = 256;

const MASK_STANDING: u8 = 0x01;
const MASK_CUT: u8 = 0x02;

fn records_of(
    rays: &Rays,
    placements: &[Placement],
    first_object: u32,
) -> Vec<vk::AccelerationStructureInstanceKHR> {
    placements
        .iter()
        .filter_map(|placement| {
            let address = match placement.traced {
                Traced::Mesh(slot) => *rays.addresses.get(slot as usize)?,
                Traced::Posed(index) => *rays.posed_addresses.get(index as usize)?,
            };
            if address == 0 {
                return None;
            }
            let mut rows = [[0.0f32; 4]; 3];
            for (row, values) in rows.iter_mut().enumerate() {
                for (column, value) in values.iter_mut().enumerate() {
                    *value = placement.transform[column][row];
                }
            }
            Some(vk::AccelerationStructureInstanceKHR {
                transform: vk::TransformMatrixKHR {
                    matrix: [
                        rows[0][0], rows[0][1], rows[0][2], rows[0][3], rows[1][0], rows[1][1],
                        rows[1][2], rows[1][3], rows[2][0], rows[2][1], rows[2][2], rows[2][3],
                    ],
                },
                instance_custom_index_and_mask: vk::Packed24_8::new(
                    first_object + placement.object,
                    if placement.masked {
                        MASK_STANDING | MASK_CUT
                    } else {
                        MASK_STANDING
                    },
                ),
                instance_shader_binding_table_record_offset_and_flags: vk::Packed24_8::new(
                    0,
                    if placement.masked {
                        vk::GeometryInstanceFlagsKHR::FORCE_NO_OPAQUE.as_raw() as u8
                    } else {
                        vk::GeometryInstanceFlagsKHR::FORCE_OPAQUE.as_raw() as u8
                    },
                ),
                acceleration_structure_reference: vk::AccelerationStructureReferenceKHR {
                    device_handle: address,
                },
            })
        })
        .collect()
}

pub fn mesh_address(rays: &Rays, slot: u32) -> u64 {
    rays.addresses.get(slot as usize).copied().unwrap_or(0)
}

pub fn build_world(
    rays: &mut Rays,
    context: &Context,
    placements: &[Placement],
    copies: &[Placement],
    copies_signature: u64,
    first_copy: u32,
) -> Result<bool, Box<dyn std::error::Error>> {
    let Some(acceleration) = context.acceleration.as_ref() else {
        return Ok(false);
    };
    let standing = records_of(rays, placements, 0);
    let signature =
        signature_of(&standing) ^ copies_signature ^ (first_copy as u64).rotate_left(23);
    if signature == rays.standing && rays.world.is_some() {
        return Ok(false);
    }
    rays.standing = signature;
    let mut records = standing;
    records.extend(records_of(rays, copies, first_copy));
    rays.count = records.len() as u32;
    reserve_instances(rays, context, records.len().max(1) as u64)?;
    write_instances(&rays.instances[rays.turn], &records);

    let instances = vk::AccelerationStructureGeometryInstancesDataKHR::default().data(
        vk::DeviceOrHostAddressConstKHR {
            device_address: address_of(context, &rays.instances[rays.turn]),
        },
    );
    let holds = vk::AccelerationStructureGeometryKHR::default()
        .geometry_type(vk::GeometryTypeKHR::INSTANCES)
        .geometry(vk::AccelerationStructureGeometryDataKHR { instances });

    let info = vk::AccelerationStructureBuildGeometryInfoKHR::default()
        .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL)
        .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
        .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
        .geometries(std::slice::from_ref(&holds));

    let mut sizes = vk::AccelerationStructureBuildSizesInfoKHR::default();
    unsafe {
        acceleration.get_acceleration_structure_build_sizes(
            vk::AccelerationStructureBuildTypeKHR::DEVICE,
            &info,
            &[rays.count],
            &mut sizes,
        )
    };

    let level = make_level(
        context,
        acceleration,
        sizes.acceleration_structure_size,
        vk::AccelerationStructureTypeKHR::TOP_LEVEL,
        "world to trace",
    )?;
    let scratch = take_scratch(rays, sizes.build_scratch_size);
    reserve_scratch(rays, context, rays.scratch_used)?;
    rays.pending.push(Pending::World {
        count: rays.count,
        target: level.handle,
        scratch,
    });

    if let Some(standing) = rays.world.take() {
        let replaced_on = rays.frames;
        rays.replaced.push((replaced_on, standing));
    }
    rays.world = Some(level);
    Ok(true)
}

pub fn take_a_turn(rays: &mut Rays) {
    rays.turn = (rays.turn + 1) % crate::renderer::context::FRAMES_IN_FLIGHT;
    if rays.pending.is_empty() {
        return;
    }
    let replaced_on = rays.frames;
    for build in rays.pending.drain(..) {
        let Pending::Mesh {
            slot: Some(slot), ..
        } = build
        else {
            continue;
        };
        let slot = slot as usize;
        if let Some(standing) = rays.meshes.get_mut(slot).and_then(Option::take) {
            rays.replaced.push((replaced_on, standing));
        }
        if let Some(address) = rays.addresses.get_mut(slot) {
            *address = 0;
        }
    }
    rays.scratch_used = 0;
    if let Some(level) = rays.world.take() {
        rays.replaced.push((replaced_on, level));
    }
    for level in rays.posed.drain(..) {
        rays.replaced.push((replaced_on, level));
    }
    rays.posed_addresses.clear();
    rays.standing = 0;
    rays.count = 0;
}

pub fn retire_replaced(rays: &mut Rays, context: &Context, frames_in_flight: u64) {
    let Some(acceleration) = context.acceleration.as_ref() else {
        return;
    };
    rays.frames += 1;
    let reached = rays.frames;
    let mut index = 0;
    while index < rays.replaced.len() {
        if reached.saturating_sub(rays.replaced[index].0) > frames_in_flight {
            let (_, mut level) = rays.replaced.remove(index);
            release(context, acceleration, &mut level);
        } else {
            index += 1;
        }
    }
    let mut index = 0;
    while index < rays.let_go.len() {
        if reached.saturating_sub(rays.let_go[index].0) > frames_in_flight {
            let (_, mut buffer) = rays.let_go.remove(index);
            destroy_buffer(context, &mut buffer);
        } else {
            index += 1;
        }
    }
}

pub fn world_handle(rays: &Rays) -> Option<vk::AccelerationStructureKHR> {
    rays.world.as_ref().map(|level| level.handle)
}

fn write_instances(buffer: &Buffer, records: &[vk::AccelerationStructureInstanceKHR]) {
    let Some(allocation) = buffer.allocation.as_ref() else {
        return;
    };
    let Some(mapped) = allocation.mapped_ptr() else {
        return;
    };
    let bytes = std::mem::size_of_val(records).min(buffer.size as usize);
    unsafe {
        std::ptr::copy_nonoverlapping(
            records.as_ptr().cast::<u8>(),
            mapped.as_ptr().cast::<u8>(),
            bytes,
        );
    }
}

fn signature_of(records: &[vk::AccelerationStructureInstanceKHR]) -> u64 {
    let mut settled: u64 = 0xcbf2_9ce4_8422_2325;
    for record in records {
        let bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                std::ptr::from_ref(record).cast::<u8>(),
                std::mem::size_of::<vk::AccelerationStructureInstanceKHR>(),
            )
        };
        for byte in bytes {
            settled ^= *byte as u64;
            settled = settled.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    settled
}

fn make_level(
    context: &Context,
    acceleration: &ash::khr::acceleration_structure::Device,
    size: u64,
    kind: vk::AccelerationStructureTypeKHR,
    name: &str,
) -> Result<Level, Box<dyn std::error::Error>> {
    let buffer = create_buffer(
        context,
        name,
        size.max(256),
        vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
            | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        MemoryLocation::GpuOnly,
    )?;
    let create_info = vk::AccelerationStructureCreateInfoKHR::default()
        .buffer(buffer.buffer)
        .size(size.max(256))
        .ty(kind);
    let handle = unsafe { acceleration.create_acceleration_structure(&create_info, None)? };
    Ok(Level { handle, buffer })
}

fn reserve_scratch(
    rays: &mut Rays,
    context: &Context,
    size: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let turn = rays.turn;
    if size <= rays.scratch_capacity[turn] {
        return Ok(());
    }
    let capacity = size.next_power_of_two();
    let grown = create_scratch(context, capacity)?;
    let replaced_on = rays.frames;
    rays.let_go.push((
        replaced_on,
        std::mem::replace(&mut rays.scratch[turn], grown),
    ));
    rays.scratch_capacity[turn] = capacity;
    Ok(())
}

fn reserve_instances(
    rays: &mut Rays,
    context: &Context,
    count: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let turn = rays.turn;
    if count <= rays.instance_capacity[turn] {
        return Ok(());
    }
    let capacity = count.next_power_of_two();
    let grown = create_instance_buffer(context, capacity)?;
    let replaced_on = rays.frames;
    rays.let_go.push((
        replaced_on,
        std::mem::replace(&mut rays.instances[turn], grown),
    ));
    rays.instance_capacity[turn] = capacity;
    Ok(())
}

fn create_scratch(context: &Context, size: u64) -> Result<Buffer, Box<dyn std::error::Error>> {
    create_buffer(
        context,
        "structure scratch",
        size.max(256),
        vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        MemoryLocation::GpuOnly,
    )
}

fn create_instance_buffer(
    context: &Context,
    count: u64,
) -> Result<Buffer, Box<dyn std::error::Error>> {
    create_buffer(
        context,
        "structure placements",
        count.max(1) * std::mem::size_of::<vk::AccelerationStructureInstanceKHR>() as u64,
        vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
            | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        MemoryLocation::CpuToGpu,
    )
}
