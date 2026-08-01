#import nightshade_wgpu::sdf_common::{
    SDF_CONTROL_BAKE,
    SDF_CONTROL_BOOKED,
    SDF_CONTROL_FREE,
    SDF_FIRST_BRICK,
    SDF_GROUPS_PER_ROW,
    SDF_WANTS_BRICK,
    SDF_WANTS_INSIDE,
    SDF_CONTROL_QUEUED,
    SDF_WORK_GROUP,
    sdf,
    sdf_brick_from_index,
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
var<storage, read_write> sdf_work: array<u32>;

@compute @workgroup_size(64)
fn main(@builtin(workgroup_id) group: vec3<u32>, @builtin(local_invocation_index) local: u32) {
    let which = (group.x + group.y * SDF_GROUPS_PER_ROW) * SDF_WORK_GROUP + local;
    if which >= atomicLoad(&sdf_control[SDF_CONTROL_QUEUED]) {
        return;
    }
    let index = sdf_work[which];
    let pointer = sdf_pointers[index];
    if pointer != SDF_WANTS_BRICK && pointer != SDF_WANTS_INSIDE {
        return;
    }
    let side = sdf.counts.z;
    let level = index / (side * side * side);
    let brick = sdf_brick_from_index(level, index);

    let booked = atomicAdd(&sdf_control[SDF_CONTROL_BOOKED], 1u);
    let standing = atomicLoad(&sdf_control[SDF_CONTROL_FREE]);
    if booked >= standing {
        return;
    }
    let slot = sdf_slots[standing - 1u - booked];
    sdf_bricks[slot] = sdf_pack_brick(level, brick);
    sdf_pointers[index] = slot + SDF_FIRST_BRICK;

    let room = arrayLength(&sdf_slots) / 2u;
    let spot = atomicAdd(&sdf_control[SDF_CONTROL_BAKE], 1u);
    if spot < room {
        sdf_slots[room + spot] = slot;
    }
}
