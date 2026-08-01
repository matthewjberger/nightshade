#import nightshade_wgpu::sdf_common::{
    SDF_CONTROL_BAKE,
    SDF_CONTROL_BOOKED,
    SDF_CONTROL_FREE,
    SDF_CONTROL_WORK,
    SDF_NO_IDENTITY,
}

@group(0) @binding(0)
var<storage, read_write> sdf_slots: array<u32>;

@group(0) @binding(1)
var<storage, read_write> sdf_control: array<atomic<u32>>;

@group(0) @binding(2)
var<storage, read_write> sdf_identities: array<vec2<u32>>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let slots = arrayLength(&sdf_slots) / 2u;
    if id.x < slots {
        sdf_slots[id.x] = id.x;
    }
    if id.x < arrayLength(&sdf_identities) {
        sdf_identities[id.x] = vec2<u32>(SDF_NO_IDENTITY, SDF_NO_IDENTITY);
    }
    if id.x == 0u {
        atomicStore(&sdf_control[SDF_CONTROL_FREE], slots);
        atomicStore(&sdf_control[SDF_CONTROL_BAKE], 0u);
        atomicStore(&sdf_control[SDF_CONTROL_BOOKED], 0u);
        atomicStore(&sdf_control[SDF_CONTROL_WORK], 0u);
    }
}
