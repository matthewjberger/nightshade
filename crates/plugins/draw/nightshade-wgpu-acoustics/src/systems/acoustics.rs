use crate::resources::{AcousticsGpu, Parts};
use nightshade::ecs::Entity;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_acoustics::contract::{
    AcousticsEcho, AcousticsFrame, AcousticsGrid, AcousticsHeard, AcousticsRays, AcousticsRoom,
    AcousticsSettings, EchoPoint, EmitterPlace, EmitterVoice, MAX_ECHOES, MAX_EMITTERS,
    MAX_MATERIALS, MAX_RAY_STEPS, RayKind, RayStep, RoomReading, SoundPath, air_gain_high,
};
use nightshade_wgpu::backend::WgpuRenderer;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;
use std::sync::Arc;

pub const ROOM_BLOCK: usize = 83;
pub const MAX_ZONES: usize = 4;
pub const TALLY_BASE: usize = 417;
pub const TALLY_STRIDE: usize = 8;
pub const TALLY_WORDS: usize = TALLY_BASE + MAX_EMITTERS * TALLY_STRIDE;
pub const TALLY_BYTES: u64 = (TALLY_WORDS * 4) as u64;
pub const ECHO_STRIDE: usize = 8;
pub const ECHO_BYTES: u64 = (MAX_ECHOES * ECHO_STRIDE * 4) as u64;
pub const STEP_STRIDE: usize = 12;
pub const STEP_BYTES: u64 = (MAX_RAY_STEPS * STEP_STRIDE * 4) as u64;
pub const READBACK_BYTES: u64 = TALLY_BYTES + ECHO_BYTES + STEP_BYTES;
pub const EMITTER_FLOATS: usize = MAX_EMITTERS * 16;
pub const EMITTER_BYTES: u64 = (EMITTER_FLOATS * 4) as u64;
pub const SETTINGS_BYTES: u64 = 144;
pub const WORKGROUP: u32 = 64;
pub const MATERIAL_FLOATS: usize = MAX_MATERIALS * 8;

pub(crate) fn build_parts(
    renderer: &mut WgpuRenderer,
    gpu: &mut AcousticsGpu,
    grid: &AcousticsGrid,
) {
    let words = grid.cells.len().div_ceil(4).max(1);
    if gpu
        .parts
        .as_ref()
        .is_some_and(|parts| parts.cell_words >= words)
    {
        return;
    }

    let device = &renderer.device;
    let shader = renderer.shader_composer.compile_wgsl(
        device,
        "acoustics.wgsl",
        include_str!("../shaders/acoustics.wgsl"),
    );

    let layout = build::bind_layout(
        device,
        "Acoustics Bind Group Layout",
        &[
            entry(0, uniform()),
            entry(1, storage(true)),
            entry(2, storage(true)),
            entry(3, storage(false)),
            entry(4, storage(false)),
            entry(5, storage(false)),
            entry(6, storage(true)),
        ],
    );

    let pipeline_layout = build::pipeline_layout(device, "Acoustics Pipeline Layout", &[&layout]);

    let pipeline = build::compute(
        device,
        "Acoustics Compute Pipeline",
        &shader,
        "main",
        &pipeline_layout,
    );

    let settings = build::uniform(device, "Acoustics Settings Buffer", SETTINGS_BYTES);
    let emitters = build::storage(device, "Acoustics Emitter Buffer", EMITTER_BYTES);
    let cells = build::storage(device, "Acoustics Cell Buffer", (words * 4) as u64);
    let tally = build::buffer(
        device,
        "Acoustics Tally Buffer",
        TALLY_BYTES,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
    );
    let echoes = build::buffer(
        device,
        "Acoustics Echo Buffer",
        ECHO_BYTES,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let steps = build::buffer(
        device,
        "Acoustics Ray Step Buffer",
        STEP_BYTES,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let materials = build::storage(
        device,
        "Acoustics Material Buffer",
        (MATERIAL_FLOATS * 4) as u64,
    );

    let staging = build::buffer(
        device,
        "Acoustics Staging Buffer",
        READBACK_BYTES,
        wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    );

    let bind_group = build::binding(
        device,
        "Acoustics Bind Group",
        &layout,
        vec![
            settings.as_entire_binding(),
            emitters.as_entire_binding(),
            cells.as_entire_binding(),
            tally.as_entire_binding(),
            echoes.as_entire_binding(),
            steps.as_entire_binding(),
            materials.as_entire_binding(),
        ],
    );

    gpu.parts = Some(Parts {
        pipeline,
        settings,
        emitters,
        cells,
        tally,
        echoes,
        steps,
        materials,
        staging,
        bind_group,
        cell_words: words,
    });
    gpu.revision = u64::MAX;
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

use std::sync::atomic::Ordering;

const AMBIENT_LOW: usize = 9;
const AMBIENT_HIGH: usize = 10;
const AMBIENT_THROUGH_LOW: usize = 81;
const AMBIENT_THROUGH_HIGH: usize = 82;
const SCATTER_SLOT: usize = 8;
const BUCKET_LOW: usize = 11;
const BUCKET_HIGH: usize = 43;
const EARLY_DIRECTION: usize = 75;
const LATE_DIRECTION: usize = 78;
const ECHO_BUCKETS: usize = 32;
const BUCKET_SECONDS: f32 = 0.05;
const EARLY_SECONDS: f32 = 0.08;
const SURFACE_SLOT: usize = 6;
const ECHO_SLOT: usize = 415;
const STEP_SLOT: usize = 416;
const CENTIMETRES: f32 = 100.0;
const DIRECTION_SCALE: f32 = 1000.0;
const ENERGY_SCALE: f32 = 100_000.0;
const SHORTEST_DECAY: f32 = 0.1;
const LONGEST_DECAY: f32 = 8.0;

pub fn sync_acoustics_pass(
    mut renderer: nightshade_wgpu::pre_render::Renderer,
    mut gpu: ResMut<AcousticsGpu>,
    heard: Option<ResMut<AcousticsHeard>>,
    room: Option<ResMut<AcousticsRoom>>,
    echo: Option<ResMut<AcousticsEcho>>,
    rays: Option<ResMut<AcousticsRays>>,
    frame: Option<Res<AcousticsFrame>>,
    grid: Option<Res<AcousticsGrid>>,
    settings: Option<Res<AcousticsSettings>>,
) {
    let (
        Some(mut heard),
        Some(mut room),
        Some(mut echo),
        Some(mut rays),
        Some(frame),
        Some(grid),
        Some(settings),
    ) = (heard, room, echo, rays, frame, grid, settings)
    else {
        return;
    };
    collect(
        &mut renderer,
        &mut gpu,
        &mut heard,
        &mut room,
        &mut echo,
        &mut rays,
    );
    dispatch(&mut renderer, &mut gpu, &frame, &grid, &settings);
}

fn collect(
    renderer: &mut WgpuRenderer,
    gpu: &mut AcousticsGpu,
    heard: &mut AcousticsHeard,
    room: &mut AcousticsRoom,
    echo: &mut AcousticsEcho,
    rays: &mut AcousticsRays,
) {
    if !gpu.pending {
        return;
    }
    let _ = renderer.device.poll(wgpu::PollType::Poll);
    if !gpu.ready.load(Ordering::Relaxed) {
        return;
    }

    let words = {
        let Some(parts) = gpu.parts.as_ref() else {
            return;
        };
        let slice = parts.staging.slice(..);
        let data = slice.get_mapped_range();
        let words: Vec<u32> = data
            .as_chunks::<4>()
            .0
            .iter()
            .map(|chunk| u32::from_le_bytes(*chunk))
            .collect();
        drop(data);
        parts.staging.unmap();
        words
    };

    gpu.pending = false;
    gpu.ready.store(false, Ordering::Relaxed);
    apply(gpu, heard, room, &words);
    lend(gpu, heard);
    remember(gpu, heard);
    gather_echoes(echo, &words, &gpu.voices);
    gather_steps(rays, &words);
}

fn gather_steps(rays: &mut AcousticsRays, words: &[u32]) {
    rays.steps.clear();
    rays.revision = rays.revision.wrapping_add(1);
    rays.span = 0.0;
    if words.len() < TALLY_WORDS {
        return;
    }
    let head = TALLY_WORDS + MAX_ECHOES * ECHO_STRIDE;
    let found = (words[STEP_SLOT] as usize).min(MAX_RAY_STEPS);
    for index in 0..found {
        let base = head + index * STEP_STRIDE;
        if base + STEP_STRIDE > words.len() {
            break;
        }
        let step = RayStep {
            start: place_at(words, base),
            end: place_at(words, base + 4),
            from_travel: f32::from_bits(words[base + 3]),
            to_travel: f32::from_bits(words[base + 7]),
            kind: RayKind::from_code(f32::from_bits(words[base + 8])),
        };
        rays.span = rays.span.max(step.to_travel);
        rays.steps.push(step);
    }
}

fn place_at(words: &[u32], base: usize) -> nalgebra_glm::Vec3 {
    nalgebra_glm::Vec3::new(
        f32::from_bits(words[base]),
        f32::from_bits(words[base + 1]),
        f32::from_bits(words[base + 2]),
    )
}

fn gather_echoes(echo: &mut AcousticsEcho, words: &[u32], voices: &[EmitterVoice]) {
    echo.points.clear();
    echo.revision = echo.revision.wrapping_add(1);
    if words.len() < TALLY_WORDS {
        return;
    }
    let found = (words[ECHO_SLOT] as usize).min(MAX_ECHOES);
    for index in 0..found {
        let base = TALLY_WORDS + index * ECHO_STRIDE;
        if base + ECHO_STRIDE > words.len() {
            break;
        }
        let which = f32::from_bits(words[base + 4]) as usize;
        echo.points.push(EchoPoint {
            place: place_at(words, base),
            strength: f32::from_bits(words[base + 3]),
            voice: voices.get(which).copied().unwrap_or_default(),
        });
    }
}

fn read_room(
    gpu: &AcousticsGpu,
    room: &mut RoomReading,
    words: &[u32],
    block: usize,
    counted: [u32; 5],
    seconds: f32,
) {
    let head = block * ROOM_BLOCK;
    let words = &words[head..];
    let rays = counted[0].max(1) as f32;
    let ambient_around = counted[3].max(1) as f32;
    let ambient_through = counted[4].max(1) as f32;
    let returns = words[0] as f32;
    let returned_span = words[1] as f32 / CENTIMETRES;
    let outdoors = words[2] as f32;

    room.loudness = (returns / rays).clamp(0.0, 1.0);
    room.span = if returns > 0.0 {
        returned_span / returns
    } else {
        0.0
    };
    room.outdoors = (outdoors / ambient_around).clamp(0.0, 1.0);
    let ambience = gpu.caps[3];
    room.outdoor_low = reaching(words[AMBIENT_LOW], ambient_around, ambience).max(reaching(
        words[AMBIENT_THROUGH_LOW],
        ambient_through,
        ambience,
    ));
    room.outdoor_high = reaching(words[AMBIENT_HIGH], ambient_around, ambience).max(reaching(
        words[AMBIENT_THROUGH_HIGH],
        ambient_through,
        ambience,
    ));

    let surfaces = words[SURFACE_SLOT + 1] as f32;
    if surfaces > 0.0 {
        room.absorbed = (words[SURFACE_SLOT] as f32 / surfaces / ENERGY_SCALE).clamp(0.0, 1.0);
    }
    room.decay = decay_of(words, BUCKET_LOW, seconds);
    room.decay_high = decay_of(words, BUCKET_HIGH, seconds);
    if surfaces > 0.0 {
        room.scattered = (words[SCATTER_SLOT] as f32 / surfaces / ENERGY_SCALE).clamp(0.0, 1.0);
    }

    let early_until = (EARLY_SECONDS / seconds) as usize;
    let mut early_energy = 0.0;
    let mut late_energy = 0.0;
    let mut early_delay = EARLY_SECONDS;
    let mut late_delay = EARLY_SECONDS;
    for bucket in 0..ECHO_BUCKETS {
        let energy = words[BUCKET_LOW + bucket] as f32 / ENERGY_SCALE;
        if energy <= 0.0 {
            continue;
        }
        let time = (bucket as f32 + 0.5) * seconds;
        if bucket < early_until {
            if early_energy <= 0.0 {
                early_delay = time;
            }
            early_energy += energy;
        } else {
            if late_energy <= 0.0 {
                late_delay = time;
            }
            late_energy += energy;
        }
    }
    room.early_gain = (early_energy / rays / gpu.caps[2].max(1.0e-4)).clamp(0.0, 1.0);
    room.late_gain = (late_energy / rays / gpu.caps[2].max(1.0e-4)).clamp(0.0, 1.0);
    room.early_delay = early_delay;
    room.late_delay = late_delay;
    room.early_from = arrival_of(words, EARLY_DIRECTION);
    room.late_from = arrival_of(words, LATE_DIRECTION);

    let opening = nalgebra_glm::Vec3::new(
        words[3] as i32 as f32 / DIRECTION_SCALE,
        words[4] as i32 as f32 / DIRECTION_SCALE,
        words[5] as i32 as f32 / DIRECTION_SCALE,
    );
    if outdoors > 0.0 && opening.magnitude() > 1.0e-3 {
        room.opening = opening.normalize();
    }
}

fn apply(gpu: &AcousticsGpu, heard: &mut AcousticsHeard, room: &mut AcousticsRoom, words: &[u32]) {
    if words.len() < TALLY_WORDS {
        return;
    }
    read_room(gpu, &mut room.heard, words, 0, gpu.counted, BUCKET_SECONDS);

    room.zones.clear();
    for (block, entity) in gpu.zoned.iter().enumerate() {
        let mut reading = room.heard;
        let seconds = gpu.buckets.get(block).copied().unwrap_or(BUCKET_SECONDS);
        let counted = gpu
            .zone_counted
            .get(block)
            .copied()
            .unwrap_or([1, 0, 0, 0, 0]);
        read_room(gpu, &mut reading, words, block + 1, counted, seconds);
        room.zones.push((*entity, reading));
    }

    heard.paths.clear();
    heard.paths.extend(gpu.parked.iter().copied());
    for (index, entity) in gpu.listened.iter().enumerate() {
        let slot = TALLY_BASE + index * TALLY_STRIDE;
        let crossing = gpu.counted[2].max(1) as f32;
        let spreading = gpu.counted[1].max(1) as f32;
        let through_low = reaching(words[slot + 1], crossing, gpu.caps[1]);
        let through_high = reaching(words[slot + 2], crossing, gpu.caps[1]);
        let around = words[slot + 3] as f32;
        let around_low = reaching(words[slot + 4], spreading, gpu.caps[0]);
        let around_high = reaching(words[slot + 5], spreading, gpu.caps[0]);
        let straight = gpu.travels.get(index).copied().unwrap_or(0.0);
        let travel = if words[slot] > 0 || around <= 0.0 {
            straight
        } else {
            words[slot + 6] as f32 / CENTIMETRES / around
        };
        let outside = gpu.outside.get(index).copied().unwrap_or(false);
        let reachable = if outside { gpu.beyond } else { 1.0 };
        let gain_low = (through_low.max(around_low) * reachable).clamp(0.0, gpu.loudest);
        heard.paths.push((
            *entity,
            SoundPath {
                listener_reverb: gpu.listens.get(index).copied().unwrap_or(true),
                direct: (words[slot] as f32).clamp(0.0, 1.0),
                gain_low,
                gain_high: (through_high.max(around_high) * reachable).clamp(0.0, gain_low),
                travel,
                through: through_low,
                around: around_low,
            },
        ));
    }
}

fn material_bytes(grid: &AcousticsGrid) -> Vec<u8> {
    let mut written = Vec::with_capacity(MATERIAL_FLOATS * 4);
    for entry in grid.materials {
        for value in [
            entry.absorb_low,
            entry.absorb_high,
            entry.crossing_low,
            entry.crossing_high,
            entry.scatter,
            entry.flat_low,
            entry.flat_high,
            entry.solid,
        ] {
            written.extend_from_slice(&value.to_le_bytes());
        }
    }
    written
}

fn reaching(counted: u32, rays: f32, cap: f32) -> f32 {
    let share = counted as f32 / ENERGY_SCALE / rays;
    (share / cap.max(1.0e-4)).min(1.0)
}

fn lend(gpu: &AcousticsGpu, heard: &mut AcousticsHeard) {
    for (leech, parent) in &gpu.leeches {
        if let Some((_, path)) = heard.paths.iter().find(|(entity, _)| entity == parent) {
            let taken = *path;
            heard.paths.push((*leech, taken));
        }
    }
}

fn remember(gpu: &mut AcousticsGpu, heard: &AcousticsHeard) {
    gpu.held.clear();
    gpu.held.extend(heard.paths.iter().copied());
}

fn zone_bounce(asked: u32, fallback: u32) -> u32 {
    if asked > 0 { asked } else { fallback.max(1) }
}

fn zone_spread(place: &EmitterPlace, settings: &AcousticsSettings) -> [u32; 5] {
    let share = settings.zone_rays.max(1);
    [
        if place.reverb_rays > 0 {
            place.reverb_rays
        } else {
            share
        },
        if place.occlusion_rays > 0 {
            place.occlusion_rays
        } else {
            0
        },
        if place.permeation_rays > 0 {
            place.permeation_rays
        } else {
            0
        },
        if place.ambient_occlusion_rays > 0 {
            place.ambient_occlusion_rays
        } else {
            0
        },
        if place.ambient_permeation_rays > 0 {
            place.ambient_permeation_rays
        } else {
            0
        },
    ]
}

fn clamped_into(place: nalgebra_glm::Vec3, grid: &AcousticsGrid) -> nalgebra_glm::Vec3 {
    let span = grid.cell * grid.side as f32;
    nalgebra_glm::Vec3::new(
        place.x.clamp(grid.origin.x, grid.origin.x + span),
        place.y.clamp(grid.origin.y, grid.origin.y + span),
        place.z.clamp(grid.origin.z, grid.origin.z + span),
    )
}

fn arrival_of(words: &[u32], slot: usize) -> nalgebra_glm::Vec3 {
    let gathered = nalgebra_glm::Vec3::new(
        words[slot] as i32 as f32,
        words[slot + 1] as i32 as f32,
        words[slot + 2] as i32 as f32,
    );
    if gathered.magnitude() > 1.0e-3 {
        gathered.normalize()
    } else {
        nalgebra_glm::Vec3::zeros()
    }
}

fn decay_of(words: &[u32], base: usize, seconds: f32) -> f32 {
    let mut loudest = 0.0f32;
    for bucket in 0..ECHO_BUCKETS {
        loudest = loudest.max(words[base + bucket] as f32);
    }
    if loudest <= 0.0 {
        return 0.0;
    }
    let mut times = 0.0;
    let mut levels = 0.0;
    let mut crossed = 0.0;
    let mut squares = 0.0;
    let mut counted = 0.0;
    for bucket in 0..ECHO_BUCKETS {
        let energy = words[base + bucket] as f32;
        if energy <= 0.0 {
            continue;
        }
        let level = 10.0 * (energy / loudest).log10();
        if level < -60.0 {
            continue;
        }
        let time = (bucket as f32 + 0.5) * seconds;
        times += time;
        levels += level;
        crossed += time * level;
        squares += time * time;
        counted += 1.0;
    }
    if counted < 3.0 {
        return 0.0;
    }
    let divisor = counted * squares - times * times;
    if divisor.abs() < 1.0e-6 {
        return 0.0;
    }
    let slope = (counted * crossed - times * levels) / divisor;
    if slope >= -1.0e-3 {
        return LONGEST_DECAY;
    }
    (-60.0 / slope).clamp(SHORTEST_DECAY, LONGEST_DECAY)
}

fn dispatch(
    renderer: &mut WgpuRenderer,
    gpu: &mut AcousticsGpu,
    frame: &AcousticsFrame,
    grid: &AcousticsGrid,
    settings: &AcousticsSettings,
) {
    if gpu.pending {
        return;
    }
    if !frame.heard || frame.emitters.is_empty() || grid.cells.is_empty() {
        return;
    }
    let slices = settings.sweep_slices.max(1);
    if gpu.slice > 0 && (frame.listener - gpu.anchor).magnitude() > settings.refresh_distance {
        gpu.slice = 0;
    }
    if gpu.slice == 0 {
        gpu.anchor = frame.listener;
    }

    build_parts(renderer, gpu, grid);
    let stale = gpu.revision != grid.revision;
    if stale {
        pack_cells(gpu, grid);
    }
    let material_stale = stale || !gpu.materials_written;
    gpu.materials_written = true;

    let parked: Vec<(Entity, SoundPath)> = gpu
        .held
        .iter()
        .filter(|(entity, _)| {
            frame
                .emitters
                .iter()
                .any(|place| place.entity == *entity && place.hold)
        })
        .copied()
        .collect();
    let mut ordered: Vec<&EmitterPlace> = frame
        .emitters
        .iter()
        .filter(|place| !parked.iter().any(|(entity, _)| *entity == place.entity))
        .collect();
    ordered.sort_by_key(|place| !place.own_reverb);
    gpu.parked = parked;
    let listener_spread = [
        settings.reverb_rays,
        settings.occlusion_rays,
        settings.permeation_rays,
        settings.ambient_occlusion_rays,
        settings.ambient_permeation_rays,
    ];
    gpu.counted = listener_spread;
    let count = ordered.len().min(MAX_EMITTERS);
    let ordered = &ordered[..count];
    gpu.leeches = frame.leeches.clone();
    gpu.zoned = ordered
        .iter()
        .filter(|place| place.own_reverb)
        .take(MAX_ZONES)
        .map(|place| place.entity)
        .collect();
    gpu.zone_rays = settings.zone_rays;

    let zone_count = gpu.zoned.len() as u32;
    let listener_total: u32 = listener_spread.iter().sum();
    let zone_total: u32 = gpu
        .zone_counted
        .iter()
        .map(|spread| spread.iter().sum::<u32>())
        .sum();
    let walked = (listener_total + zone_total).max(MAX_ECHOES as u32);
    let listened: Vec<Entity> = ordered.iter().map(|place| place.entity).collect();
    let voices: Vec<EmitterVoice> = ordered.iter().map(|place| place.voice).collect();
    gpu.holds = frame.emitters[..count]
        .iter()
        .map(|place| place.hold)
        .collect();
    let travels: Vec<f32> = ordered
        .iter()
        .map(|place| (place.position - frame.listener).magnitude())
        .collect();

    let mut places = [0.0f32; EMITTER_FLOATS];
    for (index, place) in ordered.iter().enumerate() {
        let base = index * 8;
        let seated = if place.clamped {
            clamped_into(place.position, grid)
        } else {
            place.position
        };
        places[base] = seated.x;
        places[base + 1] = seated.y;
        places[base + 2] = seated.z;
        places[base + 3] = place.reach;
        let spread = zone_spread(place, settings);
        places[base + 4] = spread[4] as f32;
        places[base + 5] = zone_bounce(
            place.ambient_permeation_bounces,
            settings.ambient_permeation_bounces,
        ) as f32;
        places[base + 6] = if place.bucket_seconds > 0.0 {
            place.bucket_seconds
        } else {
            BUCKET_SECONDS
        };
        places[base + 7] = place.seed as f32;
        places[base + 8] = spread[0] as f32;
        places[base + 9] = spread[1] as f32;
        places[base + 10] = spread[2] as f32;
        places[base + 11] = spread[3] as f32;
        places[base + 12] = zone_bounce(place.reverb_bounces, settings.reverb_bounces) as f32;
        places[base + 13] = zone_bounce(place.occlusion_bounces, settings.occlusion_bounces) as f32;
        places[base + 14] =
            zone_bounce(place.permeation_bounces, settings.permeation_bounces) as f32;
        places[base + 15] = zone_bounce(
            place.ambient_occlusion_bounces,
            settings.ambient_occlusion_bounces,
        ) as f32;
    }
    gpu.zone_counted = ordered
        .iter()
        .filter(|place| place.own_reverb)
        .take(MAX_ZONES)
        .map(|place| zone_spread(place, settings))
        .collect();
    gpu.buckets = ordered
        .iter()
        .map(|place| {
            if place.bucket_seconds > 0.0 {
                place.bucket_seconds
            } else {
                BUCKET_SECONDS
            }
        })
        .collect();

    let head = [
        frame.listener.x,
        frame.listener.y,
        frame.listener.z,
        settings.reach,
        grid.origin.x,
        grid.origin.y,
        grid.origin.z,
        grid.cell,
    ];
    let tail = [grid.side, listener_total, 1, count as u32];
    let spread_words = [
        listener_spread[0] as f32,
        listener_spread[1] as f32,
        listener_spread[2] as f32,
        listener_spread[3] as f32,
    ];
    let bounce_words = [
        settings.reverb_bounces.max(1) as f32,
        settings.occlusion_bounces.max(1) as f32,
        settings.permeation_bounces.max(1) as f32,
        settings.ambient_occlusion_bounces.max(1) as f32,
    ];
    let ambient_words = [
        listener_spread[4] as f32,
        settings.ambient_permeation_bounces.max(1) as f32,
        settings.visualisation_rays.max(1) as f32,
        0.0f32,
    ];
    let spokes = (MAX_ECHOES as u32 / (count as u32).max(1)).clamp(1, 256);
    let probes = count as u32 * spokes;
    let sliced = walked.div_ceil(slices);
    let slice_base = gpu.slice * sliced;
    let slice_rays = sliced.min(walked.saturating_sub(slice_base));
    let threads = if gpu.slice == 0 {
        slice_rays.max(probes)
    } else {
        slice_rays
    };
    let settled = gpu.slice + 1 >= slices;
    let slicing = [slice_base, slice_rays, 0u32, 0u32];
    let zoning = [
        zone_count,
        settings.zone_rays,
        u32::from(settings.indoors),
        settings.epsilon.max(1.0e-4).to_bits(),
    ];
    let air = [
        air_gain_high(settings),
        settings.speed_of_sound.max(1.0),
        if settings.outside_is_muffled {
            0.0
        } else {
            1.0
        },
        0.0,
    ];
    gpu.caps = [
        settings.occlusion_cap.clamp(0.0, 1.0),
        settings.permeation_cap.clamp(0.0, 1.0),
        settings.reverb_cap.clamp(0.0, 1.0),
        settings.ambience_cap.clamp(0.0, 1.0),
    ];
    gpu.loudest = settings.loudest.clamp(0.0, 1.0);

    {
        let Some(parts) = gpu.parts.as_ref() else {
            return;
        };
        let queue = &renderer.queue;
        queue.write_buffer(&parts.settings, 0, bytemuck::cast_slice(&head));
        queue.write_buffer(&parts.settings, 32, bytemuck::cast_slice(&tail));
        queue.write_buffer(&parts.settings, 48, bytemuck::cast_slice(&air));
        queue.write_buffer(&parts.settings, 64, bytemuck::cast_slice(&zoning));
        queue.write_buffer(&parts.settings, 80, bytemuck::cast_slice(&spread_words));
        queue.write_buffer(&parts.settings, 96, bytemuck::cast_slice(&bounce_words));
        queue.write_buffer(&parts.settings, 112, bytemuck::cast_slice(&ambient_words));
        queue.write_buffer(&parts.settings, 128, bytemuck::cast_slice(&slicing));
        queue.write_buffer(&parts.emitters, 0, bytemuck::cast_slice(&places));
        if material_stale {
            queue.write_buffer(&parts.materials, 0, &material_bytes(grid));
        }
        if stale {
            queue.write_buffer(&parts.cells, 0, &gpu.packed);
        }

        let mut encoder = renderer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Acoustics Encoder"),
            });
        if gpu.slice == 0 {
            encoder.clear_buffer(&parts.tally, 0, None);
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Acoustics Pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&parts.pipeline);
            pass.set_bind_group(0, &parts.bind_group, &[]);
            pass.dispatch_workgroups(threads.max(1).div_ceil(WORKGROUP), 1, 1);
        }
        if settled {
            encoder.copy_buffer_to_buffer(&parts.tally, 0, &parts.staging, 0, TALLY_BYTES);
            encoder.copy_buffer_to_buffer(
                &parts.echoes,
                0,
                &parts.staging,
                TALLY_BYTES,
                ECHO_BYTES,
            );
            encoder.copy_buffer_to_buffer(
                &parts.steps,
                0,
                &parts.staging,
                TALLY_BYTES + ECHO_BYTES,
                STEP_BYTES,
            );
        }
        renderer.queue.submit(std::iter::once(encoder.finish()));

        if settled {
            let ready = Arc::clone(&gpu.ready);
            parts
                .staging
                .slice(..)
                .map_async(wgpu::MapMode::Read, move |_| {
                    ready.store(true, Ordering::Relaxed);
                });
        }
    }

    gpu.slice = if settled { 0 } else { gpu.slice + 1 };
    gpu.pending = settled;
    gpu.revision = grid.revision;
    gpu.listened = listened;
    gpu.voices = voices;
    gpu.travels = travels;
}

fn pack_cells(gpu: &mut AcousticsGpu, grid: &AcousticsGrid) {
    let words = grid.cells.len().div_ceil(4);
    gpu.packed.clear();
    gpu.packed.resize(words * 4, 0);
    gpu.packed[..grid.cells.len()].copy_from_slice(&grid.cells);
}
