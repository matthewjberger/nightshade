#import nightshade_wgpu::sdf_common::{
    SDF_BAKE_ARGUMENTS,
    SDF_CONTROL_BAKE,
    SDF_CONTROL_BAKING,
    SDF_CONTROL_BOOKED,
    SDF_CONTROL_FREE,
    SDF_GROUPS_PER_ROW,
}

@group(0) @binding(0)
var<storage, read_write> sdf_control: array<atomic<u32>>;

@group(0) @binding(1)
var<storage, read_write> sdf_slots: array<u32>;

@group(0) @binding(2)
var<storage, read_write> sdf_dispatch: array<u32>;

@compute @workgroup_size(1)
fn main() {
    let booked = atomicExchange(&sdf_control[SDF_CONTROL_BOOKED], 0u);
    let standing = atomicLoad(&sdf_control[SDF_CONTROL_FREE]);
    atomicStore(&sdf_control[SDF_CONTROL_FREE], standing - min(booked, standing));

    let room = arrayLength(&sdf_slots) / 2u;
    let baking = min(atomicExchange(&sdf_control[SDF_CONTROL_BAKE], 0u), room);
    atomicStore(&sdf_control[SDF_CONTROL_BAKING], baking);
    sdf_dispatch[SDF_BAKE_ARGUMENTS] = min(baking, SDF_GROUPS_PER_ROW);
    sdf_dispatch[SDF_BAKE_ARGUMENTS + 1u] = (baking + SDF_GROUPS_PER_ROW - 1u) / SDF_GROUPS_PER_ROW;
    sdf_dispatch[SDF_BAKE_ARGUMENTS + 2u] = 1u;
    sdf_dispatch[SDF_BAKE_ARGUMENTS + 3u] = baking;
}
