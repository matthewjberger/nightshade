#import nightshade_wgpu::sdf_common::{
    SDF_BLEND_START,
    SDF_CONTROL_BAKE,
    SDF_CONTROL_FREE,
    SDF_COVERED,
    SDF_FIRST_BRICK,
    SDF_GROUPS_PER_ROW,
    SDF_HALF_DIAGONAL,
    SDF_NO_IDENTITY,
    SDF_WANTS_BRICK,
    SDF_WANTS_INSIDE,
    SDF_CONTROL_QUEUED,
    SDF_WORK_GROUP,
    sdf,
    sdf_brick_from_index,
    sdf_covered_by_finer,
    sdf_empty_pointer,
    sdf_pointer_is_brick,
    sdf_doubt,
    sdf_field,
    sdf_level_brick,
    sdf_level_cell,
    sdf_level_reach,
    sdf_never_read,
    sdf_pack_brick,
}

@group(1) @binding(0)
var<storage, read_write> sdf_pointers: array<u32>;

@group(1) @binding(1)
var<storage, read_write> sdf_slots: array<u32>;

@group(1) @binding(2)
var<storage, read_write> sdf_control: array<atomic<u32>>;

@group(1) @binding(3)
var<storage, read_write> sdf_bricks: array<vec2<u32>>;

@group(1) @binding(4)
var<storage, read_write> sdf_identities: array<vec2<u32>>;

@group(1) @binding(5)
var<storage, read_write> sdf_work: array<u32>;

const LATTICE_SIDE: u32 = 5u;
const LATTICE_POINTS: u32 = 125u;
const LATTICE_MIDDLE: u32 = 62u;
const LATTICE_STEP: f32 = 0.25;
const LATTICE_COVER: f32 = 0.2165064;

fn blend_slack(level: u32, low: vec3<f32>, high: vec3<f32>) -> f32 {
    if level + 1u >= sdf.counts.y {
        return 0.0;
    }
    let camera = sdf.camera.xyz;
    let farthest = max(abs(low - camera), abs(high - camera));
    if length(farthest) < sdf_level_reach(level) * SDF_BLEND_START {
        return 0.0;
    }
    return SDF_HALF_DIAGONAL * sdf_level_cell(level + 1u);
}

fn wants_brick(low: vec3<f32>, size: f32, cell: f32, middle: f32, slack: f32) -> bool {
    let doubt = sdf_doubt();
    if abs(middle) > SDF_HALF_DIAGONAL * size * doubt + slack {
        return false;
    }
    let margin = LATTICE_COVER * size * doubt + slack;
    var lowest = middle;
    var highest = middle;
    for (var lattice = 0u; lattice < LATTICE_POINTS; lattice = lattice + 1u) {
        if lattice == LATTICE_MIDDLE {
            continue;
        }
        let at = low + vec3<f32>(
            f32(lattice % LATTICE_SIDE),
            f32((lattice / LATTICE_SIDE) % LATTICE_SIDE),
            f32(lattice / (LATTICE_SIDE * LATTICE_SIDE))
        ) * (size * LATTICE_STEP);
        let value = sdf_field(at, 0.0, cell);
        lowest = min(lowest, value);
        highest = max(highest, value);
        if lowest <= margin && highest >= -margin {
            return true;
        }
    }
    return lowest <= margin && highest >= -margin;
}

fn give_back(pointer: u32) {
    let spot = atomicAdd(&sdf_control[SDF_CONTROL_FREE], 1u);
    sdf_slots[spot] = pointer - SDF_FIRST_BRICK;
}

fn bake_later(slot: u32) {
    let room = arrayLength(&sdf_slots) / 2u;
    let spot = atomicAdd(&sdf_control[SDF_CONTROL_BAKE], 1u);
    if spot < room {
        sdf_slots[room + spot] = slot;
    }
}

@compute @workgroup_size(64)
fn main(@builtin(workgroup_id) group: vec3<u32>, @builtin(local_invocation_index) local: u32) {
    let which = (group.x + group.y * SDF_GROUPS_PER_ROW) * SDF_WORK_GROUP + local;
    if which >= atomicLoad(&sdf_control[SDF_CONTROL_QUEUED]) {
        return;
    }
    let index = sdf_work[which];
    let side = sdf.counts.z;
    let level = index / (side * side * side);
    let brick = sdf_brick_from_index(level, index);

    let size = sdf_level_brick(level);
    let low = vec3<f32>(brick) * size;
    let high = low + vec3<f32>(size);

    let pointer = sdf_pointers[index];
    let holds_brick = sdf_pointer_is_brick(pointer);
    let wanted = sdf_pack_brick(level, brick);

    if sdf_never_read(level, low, high) || sdf_covered_by_finer(level, low, high) {
        if holds_brick {
            give_back(pointer);
        }
        sdf_pointers[index] = SDF_COVERED;
        sdf_identities[index] = vec2<u32>(SDF_NO_IDENTITY, SDF_NO_IDENTITY);
        return;
    }

    sdf_identities[index] = wanted;

    let slack = blend_slack(level, low, high);
    let cell = sdf_level_cell(level);
    let middle = sdf_field(low + vec3<f32>(size * 0.5), SDF_HALF_DIAGONAL * size * sdf_doubt() + slack, cell);
    let inside = middle < 0.0;
    var answer = sdf_empty_pointer(middle, cell);
    if wants_brick(low, size, cell, middle, slack) {
        answer = select(SDF_WANTS_BRICK, SDF_WANTS_INSIDE, inside);
    }

    if answer == SDF_WANTS_BRICK || answer == SDF_WANTS_INSIDE {
        if holds_brick {
            let slot = pointer - SDF_FIRST_BRICK;
            sdf_bricks[slot] = wanted;
            bake_later(slot);
            return;
        }
        sdf_pointers[index] = answer;
        return;
    }

    if holds_brick {
        give_back(pointer);
    }
    sdf_pointers[index] = answer;
}
