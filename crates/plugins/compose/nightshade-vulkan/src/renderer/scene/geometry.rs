use crate::renderer::context::{Context, FRAMES_IN_FLIGHT};
use crate::renderer::memory::{
    Buffer, begin_one_shot, create_buffer, destroy_buffer, end_one_shot, upload_to_buffer,
    wait_for_sent_work,
};
use ash::vk;
use gpu_allocator::MemoryLocation;
use nightshade_render::data::geometry::{Mesh, Vertex};
use nightshade_render::data::meshlet::MeshletMesh;
use nightshade_render::queries::meshlets::build_meshlets;

const INITIAL_VERTEX_CAPACITY: u64 = 1 << 18;
const INITIAL_MESHLET_CAPACITY: u64 = 1 << 14;
const INITIAL_MESHLET_VERTEX_CAPACITY: u64 = 1 << 18;
const INITIAL_MESHLET_TRIANGLE_WORD_CAPACITY: u64 = 1 << 16;
const INITIAL_RANGE_CAPACITY: u64 = 1 << 10;
const INITIAL_SKIN_VERTEX_CAPACITY: u64 = 1 << 16;
const INITIAL_MORPH_CAPACITY: u64 = 1 << 16;

const MAX_MORPH_TARGETS: usize = 8;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshRange {
    pub vertex_offset: u32,
    pub vertex_count: u32,
    pub meshlet_offset: u32,
    pub meshlet_count: u32,
    pub morph_offset: u32,
    pub morph_target_count: u32,
    pub index_offset: u32,
    pub index_count: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MorphDisplacement {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuMeshlet {
    pub bounds: [f32; 4],
    pub cone: [f32; 4],
    pub vertex_offset: u32,
    pub triangle_offset: u32,
    pub vertex_count: u32,
    pub triangle_count: u32,
    pub group: [f32; 4],
    pub parent: [f32; 4],
    pub errors: [f32; 4],
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Span {
    offset: u64,
    length: u64,
}

struct Region {
    buffer: Buffer,
    cursor: u64,
    capacity: u64,
    stride: u64,
    name: &'static str,
    free: Vec<Span>,
}

const REGION_VERTICES: usize = 0;
const REGION_INDICES: usize = 1;
const REGION_MESHLETS: usize = 2;
const REGION_MESHLET_VERTICES: usize = 3;
const REGION_MESHLET_TRIANGLES: usize = 4;
const REGION_SKIN_VERTICES: usize = 5;
const REGION_MORPHS: usize = 6;

const RETIRE_FRAMES: u32 = FRAMES_IN_FLIGHT as u32 + 1;

#[derive(Clone, Copy, Debug, Default)]
struct MeshAllocation {
    vertices: Span,
    indices: Span,
    meshlets: Span,
    meshlet_vertices: Span,
    meshlet_triangles: Span,
    skin_vertices: Span,
    morphs: Span,
}

struct Retiring {
    region: usize,
    span: Span,
    frames: u32,
}

pub struct GeometryArena {
    vertices: Region,
    indices: Region,
    meshlets: Region,
    meshlet_vertices: Region,
    meshlet_triangles: Region,
    range_records: Region,
    skin_vertices: Region,
    morph_displacements: Region,
    pub ranges: Vec<MeshRange>,
    slots: std::collections::HashMap<String, u32>,
    skin_offsets: Vec<Option<u32>>,
    bounds: Vec<[f32; 4]>,
    poses: std::collections::HashMap<nightshade::ecs::Entity, u32>,
    allocations: Vec<Option<MeshAllocation>>,
    free_slots: Vec<u32>,
    retiring: Vec<Retiring>,
    ranges_behind: bool,
    pub clear_generation: u64,
}

pub fn create_arena(context: &Context) -> Result<GeometryArena, Box<dyn std::error::Error>> {
    Ok(GeometryArena {
        vertices: create_region(
            context,
            "scene vertices",
            INITIAL_VERTEX_CAPACITY,
            std::mem::size_of::<Vertex>() as u64,
        )?,
        indices: create_region(
            context,
            "scene indices",
            1 << 16,
            std::mem::size_of::<u32>() as u64,
        )?,
        meshlets: create_region(
            context,
            "scene meshlets",
            INITIAL_MESHLET_CAPACITY,
            std::mem::size_of::<GpuMeshlet>() as u64,
        )?,
        meshlet_vertices: create_region(
            context,
            "scene meshlet vertices",
            INITIAL_MESHLET_VERTEX_CAPACITY,
            4,
        )?,
        meshlet_triangles: create_region(
            context,
            "scene meshlet triangles",
            INITIAL_MESHLET_TRIANGLE_WORD_CAPACITY,
            4,
        )?,
        range_records: create_region(
            context,
            "scene mesh ranges",
            INITIAL_RANGE_CAPACITY,
            std::mem::size_of::<MeshRange>() as u64,
        )?,
        skin_vertices: create_region(
            context,
            "scene skin vertices",
            INITIAL_SKIN_VERTEX_CAPACITY,
            std::mem::size_of::<SkinVertex>() as u64,
        )?,
        morph_displacements: create_region(
            context,
            "scene morph displacements",
            INITIAL_MORPH_CAPACITY,
            std::mem::size_of::<MorphDisplacement>() as u64,
        )?,
        ranges: Vec::new(),
        slots: std::collections::HashMap::new(),
        skin_offsets: Vec::new(),
        bounds: Vec::new(),
        poses: std::collections::HashMap::new(),
        allocations: Vec::new(),
        free_slots: Vec::new(),
        retiring: Vec::new(),
        ranges_behind: false,
        clear_generation: 0,
    })
}

pub fn destroy_arena(context: &Context, arena: &mut GeometryArena) {
    for region in regions(arena) {
        destroy_buffer(context, &mut region.buffer);
    }
    arena.ranges.clear();
    arena.slots.clear();
    arena.skin_offsets.clear();
    arena.bounds.clear();
    arena.poses.clear();
    arena.allocations.clear();
    arena.free_slots.clear();
    arena.retiring.clear();
}

pub fn reset_arena(arena: &mut GeometryArena, clear_generation: u64) {
    arena.ranges.clear();
    arena.slots.clear();
    arena.skin_offsets.clear();
    arena.bounds.clear();
    arena.poses.clear();
    arena.allocations.clear();
    arena.free_slots.clear();
    arena.retiring.clear();
    for region in regions(arena) {
        rewind(region);
    }
    arena.clear_generation = clear_generation;
}

pub fn mesh_slot(arena: &GeometryArena, name: &str) -> Option<u32> {
    arena.slots.get(name).copied()
}

pub fn vertex_buffer(arena: &GeometryArena) -> &Buffer {
    &arena.vertices.buffer
}

pub fn meshlet_buffer(arena: &GeometryArena) -> &Buffer {
    &arena.meshlets.buffer
}

pub fn meshlet_vertex_buffer(arena: &GeometryArena) -> &Buffer {
    &arena.meshlet_vertices.buffer
}

pub fn meshlet_triangle_buffer(arena: &GeometryArena) -> &Buffer {
    &arena.meshlet_triangles.buffer
}

pub fn index_buffer(arena: &GeometryArena) -> &Buffer {
    &arena.indices.buffer
}

pub fn range_buffer(arena: &GeometryArena) -> &Buffer {
    &arena.range_records.buffer
}

pub fn upload_mesh(
    arena: &mut GeometryArena,
    context: &Context,
    name: &str,
    mesh: &Mesh,
    clusters: Option<&MeshletMesh>,
) -> Result<u32, Box<dyn std::error::Error>> {
    if mesh.vertices.is_empty() || mesh.indices.is_empty() {
        return Err(format!("the mesh {name} carries no geometry").into());
    }

    let built;
    let clusters = match clusters {
        Some(ready) => ready,
        None => {
            built = build_meshlets(mesh);
            &built
        }
    };
    if clusters.meshlets.is_empty() {
        return Err(format!("the mesh {name} produced no meshlets").into());
    }

    let mut held = MeshAllocation {
        vertices: reserve(&mut arena.vertices, context, mesh.vertices.len() as u64)?,
        indices: reserve(&mut arena.indices, context, mesh.indices.len() as u64)?,
        meshlets: reserve(&mut arena.meshlets, context, clusters.meshlets.len() as u64)?,
        meshlet_vertices: reserve(
            &mut arena.meshlet_vertices,
            context,
            clusters.vertices.len() as u64,
        )?,
        meshlet_triangles: reserve(
            &mut arena.meshlet_triangles,
            context,
            clusters.triangles.len().div_ceil(4) as u64,
        )?,
        ..MeshAllocation::default()
    };

    let meshlet_vertex_base = held.meshlet_vertices.offset as u32;
    let meshlet_triangle_base = held.meshlet_triangles.offset as u32 * 4;
    let records: Vec<GpuMeshlet> = clusters
        .meshlets
        .iter()
        .map(|meshlet| GpuMeshlet {
            group: meshlet.group,
            parent: meshlet.parent,
            errors: [meshlet.group_error, meshlet.parent_error, 0.0, 0.0],
            bounds: meshlet.bounds,
            cone: meshlet.cone,
            vertex_offset: meshlet_vertex_base + meshlet.vertex_offset,
            triangle_offset: meshlet_triangle_base + meshlet.triangle_offset,
            vertex_count: meshlet.vertex_count,
            triangle_count: meshlet.triangle_count,
        })
        .collect();

    write_at(
        &arena.vertices,
        context,
        held.vertices,
        bytemuck::cast_slice(&mesh.vertices),
    )?;
    write_at(
        &arena.indices,
        context,
        held.indices,
        bytemuck::cast_slice(&mesh.indices),
    )?;
    write_at(
        &arena.meshlets,
        context,
        held.meshlets,
        bytemuck::cast_slice(&records),
    )?;
    write_at(
        &arena.meshlet_vertices,
        context,
        held.meshlet_vertices,
        bytemuck::cast_slice(&clusters.vertices),
    )?;
    write_at(
        &arena.meshlet_triangles,
        context,
        held.meshlet_triangles,
        &clusters.triangles,
    )?;

    let (morph_span, morph_target_count) = upload_morphs(arena, context, mesh)?;
    held.morphs = morph_span;

    let range = MeshRange {
        vertex_offset: held.vertices.offset as u32,
        vertex_count: mesh.vertices.len() as u32,
        meshlet_offset: held.meshlets.offset as u32,
        meshlet_count: records.len() as u32,
        morph_offset: morph_span.offset as u32,
        morph_target_count,
        index_offset: held.indices.offset as u32,
        index_count: mesh.indices.len() as u32,
    };

    let skin_offset = match mesh.skin_data.as_ref() {
        Some(skin) if skin.skinned_vertices.len() == mesh.vertices.len() => {
            let records: Vec<SkinVertex> = skin
                .skinned_vertices
                .iter()
                .map(|vertex| SkinVertex {
                    joint_indices: vertex.joint_indices,
                    joint_weights: vertex.joint_weights,
                })
                .collect();
            let span = reserve(&mut arena.skin_vertices, context, records.len() as u64)?;
            write_at(
                &arena.skin_vertices,
                context,
                span,
                bytemuck::cast_slice(&records),
            )?;
            held.skin_vertices = span;
            Some(span.offset as u32)
        }
        _ => None,
    };

    let sphere = bounding_sphere(mesh);
    let slot = match arena.slots.get(name).copied() {
        Some(slot) => slot,
        None => match arena.free_slots.pop() {
            Some(slot) => {
                arena.slots.insert(name.to_string(), slot);
                slot
            }
            None => {
                let slot = arena.ranges.len() as u32;
                arena.ranges.push(MeshRange::default());
                arena.skin_offsets.push(None);
                arena.bounds.push([0.0; 4]);
                arena.allocations.push(None);
                arena.slots.insert(name.to_string(), slot);
                slot
            }
        },
    };
    if let Some(given) = arena
        .allocations
        .get_mut(slot as usize)
        .and_then(Option::take)
    {
        retire(arena, given);
    }
    arena.ranges[slot as usize] = range;
    arena.skin_offsets[slot as usize] = skin_offset;
    arena.bounds[slot as usize] = sphere;
    arena.allocations[slot as usize] = Some(held);
    arena.ranges_behind = true;
    Ok(slot)
}

pub fn flush_ranges(
    arena: &mut GeometryArena,
    context: &Context,
) -> Result<(), Box<dyn std::error::Error>> {
    if !arena.ranges_behind {
        return Ok(());
    }
    arena.ranges_behind = false;
    upload_ranges(arena, context)
}

fn upload_morphs(
    arena: &mut GeometryArena,
    context: &Context,
    mesh: &Mesh,
) -> Result<(Span, u32), Box<dyn std::error::Error>> {
    let Some(morphs) = mesh.morph_targets.as_ref() else {
        return Ok((Span::default(), 0));
    };
    let vertices = mesh.vertices.len();
    let targets: Vec<&nightshade_render::data::geometry::MorphTarget> = morphs
        .targets
        .iter()
        .filter(|target| target.position_displacements.len() == vertices)
        .take(MAX_MORPH_TARGETS)
        .collect();
    if targets.is_empty() {
        return Ok((Span::default(), 0));
    }

    let mut records = Vec::with_capacity(targets.len() * vertices);
    for target in &targets {
        for vertex in 0..vertices {
            records.push(MorphDisplacement {
                position: target.position_displacements[vertex],
                normal: displacement(target.normal_displacements.as_ref(), vertex),
                tangent: displacement(target.tangent_displacements.as_ref(), vertex),
            });
        }
    }
    let span = reserve(
        &mut arena.morph_displacements,
        context,
        records.len() as u64,
    )?;
    write_at(
        &arena.morph_displacements,
        context,
        span,
        bytemuck::cast_slice(&records),
    )?;
    Ok((span, targets.len() as u32))
}

fn displacement(source: Option<&Vec<[f32; 3]>>, vertex: usize) -> [f32; 3] {
    source
        .and_then(|values| values.get(vertex).copied())
        .unwrap_or([0.0; 3])
}

fn upload_ranges(
    arena: &mut GeometryArena,
    context: &Context,
) -> Result<(), Box<dyn std::error::Error>> {
    let records = std::mem::take(&mut arena.ranges);
    rewind(&mut arena.range_records);
    let result = append(
        &mut arena.range_records,
        context,
        records.len() as u64,
        bytemuck::cast_slice(&records),
    );
    arena.ranges = records;
    result.map(|_| ())
}

fn regions(arena: &mut GeometryArena) -> [&mut Region; 8] {
    [
        &mut arena.vertices,
        &mut arena.indices,
        &mut arena.meshlets,
        &mut arena.meshlet_vertices,
        &mut arena.meshlet_triangles,
        &mut arena.range_records,
        &mut arena.skin_vertices,
        &mut arena.morph_displacements,
    ]
}

fn reserve(
    region: &mut Region,
    context: &Context,
    elements: u64,
) -> Result<Span, Box<dyn std::error::Error>> {
    if elements == 0 {
        return Ok(Span::default());
    }
    if let Some(index) = region
        .free
        .iter()
        .position(|given| given.length >= elements)
    {
        let given = region.free[index];
        if given.length == elements {
            region.free.remove(index);
        } else {
            region.free[index] = Span {
                offset: given.offset + elements,
                length: given.length - elements,
            };
        }
        return Ok(Span {
            offset: given.offset,
            length: elements,
        });
    }
    let end = region.cursor + elements;
    if end > region.capacity {
        grow(region, context, end.next_power_of_two())?;
    }
    let offset = region.cursor;
    region.cursor = end;
    Ok(Span {
        offset,
        length: elements,
    })
}

fn write_at(
    region: &Region,
    context: &Context,
    span: Span,
    bytes: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    if bytes.is_empty() {
        return Ok(());
    }
    upload_to_buffer(context, &region.buffer, span.offset * region.stride, bytes)
}

fn release(region: &mut Region, span: Span) {
    if span.length == 0 {
        return;
    }
    let at = region
        .free
        .partition_point(|held| held.offset < span.offset);
    region.free.insert(at, span);
    let mut merged: Vec<Span> = Vec::with_capacity(region.free.len());
    for held in region.free.drain(..) {
        match merged.last_mut() {
            Some(last) if last.offset + last.length == held.offset => last.length += held.length,
            _ => merged.push(held),
        }
    }
    region.free = merged;
    while let Some(last) = region.free.last().copied() {
        if last.offset + last.length != region.cursor {
            break;
        }
        region.cursor = last.offset;
        region.free.pop();
    }
}

fn rewind(region: &mut Region) {
    region.cursor = 0;
    region.free.clear();
}

fn append(
    region: &mut Region,
    context: &Context,
    elements: u64,
    bytes: &[u8],
) -> Result<u64, Box<dyn std::error::Error>> {
    let span = reserve(region, context, elements)?;
    write_at(region, context, span, bytes)?;
    Ok(span.offset)
}

fn region_at(arena: &mut GeometryArena, index: usize) -> Option<&mut Region> {
    Some(match index {
        REGION_VERTICES => &mut arena.vertices,
        REGION_INDICES => &mut arena.indices,
        REGION_MESHLETS => &mut arena.meshlets,
        REGION_MESHLET_VERTICES => &mut arena.meshlet_vertices,
        REGION_MESHLET_TRIANGLES => &mut arena.meshlet_triangles,
        REGION_SKIN_VERTICES => &mut arena.skin_vertices,
        REGION_MORPHS => &mut arena.morph_displacements,
        _ => return None,
    })
}

fn retire(arena: &mut GeometryArena, held: MeshAllocation) {
    let runs = [
        (REGION_VERTICES, held.vertices),
        (REGION_INDICES, held.indices),
        (REGION_MESHLETS, held.meshlets),
        (REGION_MESHLET_VERTICES, held.meshlet_vertices),
        (REGION_MESHLET_TRIANGLES, held.meshlet_triangles),
        (REGION_SKIN_VERTICES, held.skin_vertices),
        (REGION_MORPHS, held.morphs),
    ];
    for (region, span) in runs {
        if span.length == 0 {
            continue;
        }
        arena.retiring.push(Retiring {
            region,
            span,
            frames: RETIRE_FRAMES,
        });
    }
}

pub fn retire_spans(arena: &mut GeometryArena) {
    if arena.retiring.is_empty() {
        return;
    }
    let mut ready: Vec<(usize, Span)> = Vec::new();
    arena.retiring.retain_mut(|held| {
        if held.frames > 0 {
            held.frames -= 1;
            return true;
        }
        ready.push((held.region, held.span));
        false
    });
    for (index, span) in ready {
        if let Some(region) = region_at(arena, index) {
            release(region, span);
        }
    }
}

pub fn release_mesh(arena: &mut GeometryArena, name: &str) -> Option<u32> {
    let slot = arena.slots.remove(name)?;
    if let Some(held) = arena
        .allocations
        .get_mut(slot as usize)
        .and_then(Option::take)
    {
        retire(arena, held);
    }
    if let Some(range) = arena.ranges.get_mut(slot as usize) {
        *range = MeshRange::default();
    }
    if let Some(offset) = arena.skin_offsets.get_mut(slot as usize) {
        *offset = None;
    }
    if let Some(sphere) = arena.bounds.get_mut(slot as usize) {
        *sphere = [0.0; 4];
    }
    arena.free_slots.push(slot);
    arena.ranges_behind = true;
    Some(slot)
}

fn grow(
    region: &mut Region,
    context: &Context,
    capacity: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    unsafe { context.device.device_wait_idle()? };
    let grown = create_buffer(
        context,
        region.name,
        capacity * region.stride,
        storage_usage(),
        MemoryLocation::GpuOnly,
    )?;
    copy_buffer(
        context,
        &region.buffer,
        &grown,
        region.cursor * region.stride,
    )?;
    wait_for_sent_work(context)?;
    destroy_buffer(context, &mut region.buffer);
    region.buffer = grown;
    region.capacity = capacity;
    Ok(())
}

fn create_region(
    context: &Context,
    name: &'static str,
    capacity: u64,
    stride: u64,
) -> Result<Region, Box<dyn std::error::Error>> {
    Ok(Region {
        buffer: create_buffer(
            context,
            name,
            capacity * stride,
            storage_usage(),
            MemoryLocation::GpuOnly,
        )?,
        cursor: 0,
        capacity,
        stride,
        name,
        free: Vec::new(),
    })
}

fn storage_usage() -> vk::BufferUsageFlags {
    vk::BufferUsageFlags::STORAGE_BUFFER
        | vk::BufferUsageFlags::INDEX_BUFFER
        | vk::BufferUsageFlags::TRANSFER_DST
        | vk::BufferUsageFlags::TRANSFER_SRC
        | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
        | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
}

fn copy_buffer(
    context: &Context,
    source: &Buffer,
    destination: &Buffer,
    length: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    if length == 0 {
        return Ok(());
    }
    let command_buffer = begin_one_shot(context)?;
    let region = vk::BufferCopy::default().size(length);
    unsafe {
        context.device.cmd_copy_buffer(
            command_buffer,
            source.buffer,
            destination.buffer,
            std::slice::from_ref(&region),
        )
    };
    end_one_shot(context, command_buffer)
}

pub fn meshlet_count(arena: &GeometryArena, slot: u32) -> u32 {
    arena
        .ranges
        .get(slot as usize)
        .map(|range| range.meshlet_count)
        .unwrap_or(0)
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkinVertex {
    pub joint_indices: [u32; 4],
    pub joint_weights: [f32; 4],
}

pub fn mesh_bounds(arena: &GeometryArena, slot: u32) -> Option<[f32; 4]> {
    arena.bounds.get(slot as usize).copied()
}

fn bounding_sphere(mesh: &Mesh) -> [f32; 4] {
    if let Some(volume) = mesh.bounding_volume.as_ref() {
        return [
            volume.obb.center.x,
            volume.obb.center.y,
            volume.obb.center.z,
            volume.sphere_radius,
        ];
    }
    let mut lowest = [f32::MAX; 3];
    let mut highest = [f32::MIN; 3];
    for vertex in &mesh.vertices {
        for axis in 0..3 {
            lowest[axis] = lowest[axis].min(vertex.position[axis]);
            highest[axis] = highest[axis].max(vertex.position[axis]);
        }
    }
    let center = [
        (lowest[0] + highest[0]) * 0.5,
        (lowest[1] + highest[1]) * 0.5,
        (lowest[2] + highest[2]) * 0.5,
    ];
    let mut furthest = 0.0f32;
    for vertex in &mesh.vertices {
        let offset = [
            vertex.position[0] - center[0],
            vertex.position[1] - center[1],
            vertex.position[2] - center[2],
        ];
        let distance = offset[0] * offset[0] + offset[1] * offset[1] + offset[2] * offset[2];
        furthest = furthest.max(distance);
    }
    [center[0], center[1], center[2], furthest.sqrt()]
}

pub fn mesh_range(arena: &GeometryArena, slot: u32) -> Option<MeshRange> {
    arena.ranges.get(slot as usize).copied()
}

pub fn skin_vertex_buffer(arena: &GeometryArena) -> &Buffer {
    &arena.skin_vertices.buffer
}

pub fn morph_buffer(arena: &GeometryArena) -> &Buffer {
    &arena.morph_displacements.buffer
}

pub fn skin_offset(arena: &GeometryArena, slot: u32) -> Option<u32> {
    arena.skin_offsets.get(slot as usize).copied().flatten()
}

pub const POSE_RUNS: u32 = FRAMES_IN_FLIGHT as u32 + 1;

pub fn reserve_pose(
    arena: &mut GeometryArena,
    context: &Context,
    entity: nightshade::ecs::Entity,
    vertices: u32,
) -> Result<u32, Box<dyn std::error::Error>> {
    if let Some(offset) = arena.poses.get(&entity) {
        return Ok(*offset);
    }
    let offset = append(
        &mut arena.vertices,
        context,
        vertices as u64 * POSE_RUNS as u64,
        &[],
    )?;
    arena.poses.insert(entity, offset as u32);
    Ok(offset as u32)
}
