#import nightshade_wgpu::sdf_common::{
    SDF_CONTROL_WORK,
    SDF_COVERED,
    SDF_WANTS_BRICK,
    SDF_WANTS_INSIDE,
    sdf,
    sdf_covered_by_finer,
    sdf_empty_reach,
    sdf_level_base,
    sdf_level_brick,
    sdf_level_cell,
    sdf_never_read,
    sdf_pack_brick,
    sdf_pointer_index,
    sdf_pointer_is_empty,
}

@group(1) @binding(0)
var<storage, read_write> sdf_pointers: array<u32>;

@group(1) @binding(1)
var<storage, read_write> sdf_identities: array<vec2<u32>>;

@group(1) @binding(2)
var<storage, read_write> sdf_stir: array<u32>;

@group(1) @binding(3)
var<storage, read_write> sdf_control: array<atomic<u32>>;

@group(1) @binding(4)
var<storage, read_write> sdf_work: array<u32>;

const SDF_MOVED_ROOM: u32 = 256u;

@group(1) @binding(5)
var<uniform> sdf_moved: array<vec4<f32>, SDF_MOVED_ROOM>;

fn queue(index: u32) {
    let spot = atomicAdd(&sdf_control[SDF_CONTROL_WORK], 1u);
    if spot < arrayLength(&sdf_work) {
        sdf_work[spot] = index;
    }
}

fn leap_touches_a_moved_ball(pointer: u32, level: u32, center: vec3<f32>) -> bool {
    if !sdf_pointer_is_empty(pointer) {
        return false;
    }
    let reach = sdf_empty_reach(pointer, sdf_level_cell(level));
    let moved = min(sdf.flags.x, SDF_MOVED_ROOM);
    for (var which = 0u; which < moved; which = which + 1u) {
        let ball = sdf_moved[which];
        if distance(center, ball.xyz) - ball.w < reach {
            return true;
        }
    }
    return false;
}

@compute @workgroup_size(4, 4, 4)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let side = sdf.counts.z;
    let level = id.z / side;
    if id.x >= side || id.y >= side || level >= sdf.counts.y {
        return;
    }
    let brick = sdf_level_base(level)
        + vec3<i32>(i32(id.x), i32(id.y), i32(id.z % side));
    let index = sdf_pointer_index(level, brick);
    let size = sdf_level_brick(level);
    let low = vec3<f32>(brick) * size;
    let high = low + vec3<f32>(size);
    let pointer = sdf_pointers[index];

    if sdf_never_read(level, low, high) || sdf_covered_by_finer(level, low, high) {
        if pointer != SDF_COVERED {
            queue(index);
        }
        return;
    }
    if pointer == SDF_COVERED || pointer == SDF_WANTS_BRICK || pointer == SDF_WANTS_INSIDE {
        queue(index);
        return;
    }
    let held = sdf_identities[index];
    let wanted = sdf_pack_brick(level, brick);
    if held.x != wanted.x
        || held.y != wanted.y
        || sdf_stir[index] == sdf.frame.x
        || leap_touches_a_moved_ball(pointer, level, low + vec3<f32>(size * 0.5)) {
        queue(index);
    }
}
