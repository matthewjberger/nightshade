use crate::resources::{NearGpu, Parts};
use nightshade_near::contract::{Answer, NearAnswers, NearAsks, NearFrame, NearSettings};
use nightshade_wgpu::backend::WgpuRenderer;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;

pub const WORKGROUP: u32 = 64;
pub const MAX_SIDE: u32 = 32;
pub const MAX_POINTS: usize = 16384;
pub const MAX_ASKS: usize = 256;

pub const SETTINGS_BYTES: u64 = 48;
pub const POINT_BYTES: u64 = (MAX_POINTS * 16) as u64;
pub const WEIGHT_BYTES: u64 = (MAX_POINTS * 4) as u64;
pub const ASK_BYTES: u64 = (MAX_ASKS * 16) as u64;
pub const ANSWER_BYTES: u64 = (MAX_ASKS * 16) as u64;
pub const PUSH_BYTES: u64 = (MAX_POINTS * 16) as u64;
pub const READBACK_BYTES: u64 = ANSWER_BYTES + PUSH_BYTES;

pub(crate) fn build_parts(renderer: &mut WgpuRenderer, gpu: &mut NearGpu, capacity: u32) {
    let side = MAX_SIDE;
    let cells = side * side * side;
    if gpu
        .parts
        .as_ref()
        .is_some_and(|parts| parts.capacity >= capacity)
    {
        return;
    }

    let device = &renderer.device;
    let shader = renderer.shader_composer.compile_wgsl(
        device,
        "near.wgsl",
        include_str!("../shaders/near.wgsl"),
    );

    let layout = build::bind_layout(
        device,
        "Near Bind Group Layout",
        &[
            entry(0, uniform()),
            entry(1, storage(true)),
            entry(2, storage(true)),
            entry(3, storage(true)),
            entry(4, storage(false)),
            entry(5, storage(false)),
            entry(6, storage(false)),
            entry(7, storage(false)),
        ],
    );

    let pipeline_layout = build::pipeline_layout(device, "Near Pipeline Layout", &[&layout]);

    let make = |entry_point: &str, label: &str| {
        build::compute(device, label, &shader, entry_point, &pipeline_layout)
    };

    let clear = make("clear_cells", "Near Clear Pipeline");
    let fill = make("fill_cells", "Near Fill Pipeline");
    let answer = make("answer_asks", "Near Answer Pipeline");
    let push = make("push_apart", "Near Push Pipeline");

    let settings = buffer(
        device,
        "Near Settings Buffer",
        SETTINGS_BYTES,
        wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    );
    let points = buffer(
        device,
        "Near Point Buffer",
        POINT_BYTES,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    );
    let weights = buffer(
        device,
        "Near Weight Buffer",
        WEIGHT_BYTES,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    );
    let asks = buffer(
        device,
        "Near Ask Buffer",
        ASK_BYTES,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    );
    let cell_counts = buffer(
        device,
        "Near Cell Count Buffer",
        (cells * 4) as u64,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    );
    let cell_items = buffer(
        device,
        "Near Cell Item Buffer",
        (cells * capacity * 4) as u64,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    );
    let answers = buffer(
        device,
        "Near Answer Buffer",
        ANSWER_BYTES,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let pushes = buffer(
        device,
        "Near Push Buffer",
        PUSH_BYTES,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let staging = buffer(
        device,
        "Near Staging Buffer",
        READBACK_BYTES,
        wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    );

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Near Bind Group"),
        layout: &layout,
        entries: &[
            binding(0, &settings),
            binding(1, &points),
            binding(2, &weights),
            binding(3, &asks),
            binding(4, &cell_counts),
            binding(5, &cell_items),
            binding(6, &answers),
            binding(7, &pushes),
        ],
    });

    gpu.parts = Some(Parts {
        clear,
        fill,
        answer,
        push,
        settings,
        points,
        weights,
        asks,
        answers,
        pushes,
        staging,
        bind_group,
        cells,
        capacity,
    });
}

fn buffer(
    device: &wgpu::Device,
    label: &str,
    size: u64,
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    build::buffer(device, label, size, usage)
}

fn binding(index: u32, held: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding: index,
        resource: held.as_entire_binding(),
    }
}

fn entry(binding: u32, ty: wgpu::BindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty,
        count: None,
    }
}

fn uniform() -> wgpu::BindingType {
    wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
    }
}

fn storage(read_only: bool) -> wgpu::BindingType {
    wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Storage { read_only },
        has_dynamic_offset: false,
        min_binding_size: None,
    }
}

use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use std::sync::atomic::Ordering;

pub fn sync_near_pass(
    mut renderer: nightshade_wgpu::pre_render::Renderer,
    mut gpu: ResMut<NearGpu>,
    answers: Option<ResMut<NearAnswers>>,
    frame: Option<Res<NearFrame>>,
    asks: Option<Res<NearAsks>>,
    settings: Option<Res<NearSettings>>,
) {
    let (Some(mut answers), Some(frame), Some(asks), Some(settings)) =
        (answers, frame, asks, settings)
    else {
        return;
    };
    collect(&mut renderer, &mut gpu, &mut answers);
    dispatch(&mut renderer, &mut gpu, &frame, &asks, &settings);
}

fn collect(
    renderer: &mut nightshade_wgpu::pre_render::Renderer,
    gpu: &mut NearGpu,
    answers: &mut NearAnswers,
) {
    if !gpu.pending {
        return;
    }
    let _ = renderer.device.poll(wgpu::PollType::Poll);
    if !gpu.ready.load(Ordering::Relaxed) {
        return;
    }
    let Some(parts) = gpu.parts.as_ref() else {
        return;
    };
    {
        let view = parts.staging.slice(..).get_mapped_range();
        let read: &[f32] = bytemuck::cast_slice(&view);
        let asked = gpu.tags.len();
        answers.answers.clear();
        for (slot, tag) in gpu.tags.iter().enumerate() {
            let base = slot * 4;
            if base + 3 >= read.len() {
                break;
            }
            let count = read[base] as u32;
            let index = read[base + 1];
            let reach = read[base + 2];
            let nearest = if index < 0.0 {
                None
            } else {
                gpu.held.get(index as usize).copied()
            };
            answers.answers.push(Answer {
                tag: *tag,
                count,
                nearest,
                reach,
            });
        }
        let pushes_at = (ANSWER_BYTES / 4) as usize;
        answers.pushes.clear();
        for (slot, entity) in gpu.held.iter().enumerate() {
            let base = pushes_at + slot * 4;
            if base + 3 >= read.len() {
                break;
            }
            let push = Vec3::new(read[base], read[base + 1], read[base + 2]);
            if push.magnitude() <= 0.0 {
                continue;
            }
            answers.pushes.push((*entity, push));
        }
        let _ = asked;
    }
    parts.staging.unmap();
    gpu.pending = false;
    gpu.ready.store(false, Ordering::Relaxed);
}

fn dispatch(
    renderer: &mut nightshade_wgpu::pre_render::Renderer,
    gpu: &mut NearGpu,
    frame: &NearFrame,
    asks: &NearAsks,
    settings: &NearSettings,
) {
    if gpu.pending || frame.points.is_empty() {
        return;
    }
    let capacity = settings.cell_capacity.clamp(4, 64);
    build_parts(renderer, gpu, capacity);
    let Some(parts) = gpu.parts.as_ref() else {
        return;
    };

    let mut lowest = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut highest = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
    gpu.held.clear();
    gpu.packed_points.clear();
    gpu.packed_weights.clear();
    for (entity, spot, bulk, weight) in frame.points.iter().take(MAX_POINTS) {
        gpu.held.push(*entity);
        gpu.packed_points.push([spot.x, spot.y, spot.z, *bulk]);
        gpu.packed_weights.push(*weight);
        lowest = Vec3::new(
            lowest.x.min(spot.x),
            lowest.y.min(spot.y),
            lowest.z.min(spot.z),
        );
        highest = Vec3::new(
            highest.x.max(spot.x),
            highest.y.max(spot.y),
            highest.z.max(spot.z),
        );
    }

    gpu.tags.clear();
    gpu.packed_asks.clear();
    for ask in asks.asks.iter().take(MAX_ASKS) {
        gpu.tags.push(ask.tag);
        gpu.packed_asks
            .push([ask.point.x, ask.point.y, ask.point.z, ask.radius]);
    }
    if gpu.packed_asks.is_empty() {
        gpu.packed_asks.push([0.0, 0.0, 0.0, 0.0]);
    }

    let widest = asks
        .asks
        .iter()
        .map(|ask| ask.radius)
        .fold(settings.room, f32::max);
    let span = (highest - lowest).magnitude().max(1.0) + widest * 2.0;
    let cell_size = settings.cell_size.max(span / MAX_SIDE as f32).max(0.01);
    let origin = lowest - Vec3::new(widest, widest, widest);
    let count = gpu.held.len() as u32;

    let settings_bytes: [f32; 12] = [
        origin.x,
        origin.y,
        origin.z,
        0.0,
        cell_size,
        settings.room,
        0.0,
        0.0,
        f32::from_bits(count),
        f32::from_bits(MAX_SIDE),
        f32::from_bits(parts.cells),
        f32::from_bits(capacity),
    ];

    let queue = renderer.queue.clone();
    queue.write_buffer(&parts.settings, 0, bytemuck::cast_slice(&settings_bytes));
    queue.write_buffer(
        &parts.points,
        0,
        bytemuck::cast_slice(gpu.packed_points.as_slice()),
    );
    queue.write_buffer(
        &parts.weights,
        0,
        bytemuck::cast_slice(gpu.packed_weights.as_slice()),
    );
    queue.write_buffer(
        &parts.asks,
        0,
        bytemuck::cast_slice(gpu.packed_asks.as_slice()),
    );

    let mut encoder = renderer
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Near Encoder"),
        });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Near Pass"),
            timestamp_writes: None,
        });
        pass.set_bind_group(0, &parts.bind_group, &[]);
        let asked = gpu.tags.len().max(1) as u32;
        let work = [
            (&parts.clear, parts.cells.div_ceil(WORKGROUP)),
            (&parts.fill, count.div_ceil(WORKGROUP)),
            (&parts.answer, asked.div_ceil(WORKGROUP)),
            (&parts.push, count.div_ceil(WORKGROUP)),
        ];
        for (pipeline, groups) in work {
            pass.set_pipeline(pipeline);
            pass.dispatch_workgroups(groups.max(1), 1, 1);
        }
    }
    encoder.copy_buffer_to_buffer(&parts.answers, 0, &parts.staging, 0, ANSWER_BYTES);
    encoder.copy_buffer_to_buffer(
        &parts.pushes,
        0,
        &parts.staging,
        ANSWER_BYTES,
        READBACK_BYTES - ANSWER_BYTES,
    );
    queue.submit(std::iter::once(encoder.finish()));

    gpu.pending = true;
    let ready = gpu.ready.clone();
    parts
        .staging
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |_| {
            ready.store(true, Ordering::Relaxed);
        });
}
