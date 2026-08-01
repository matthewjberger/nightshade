#import nightshade_wgpu::sdf_common::{
    SDF_CONTROL_QUEUED,
    SDF_CONTROL_WORK,
    SDF_GROUPS_PER_ROW,
    SDF_WORK_GROUP,
}

@group(0) @binding(0)
var<storage, read_write> sdf_control: array<atomic<u32>>;

@group(0) @binding(1)
var<storage, read_write> sdf_work: array<u32>;

@group(0) @binding(2)
var<storage, read_write> sdf_dispatch: array<u32>;

@compute @workgroup_size(1)
fn main() {
    let queued = min(atomicExchange(&sdf_control[SDF_CONTROL_WORK], 0u), arrayLength(&sdf_work));
    let groups = (queued + SDF_WORK_GROUP - 1u) / SDF_WORK_GROUP;
    atomicStore(&sdf_control[SDF_CONTROL_QUEUED], queued);
    sdf_dispatch[0] = min(groups, SDF_GROUPS_PER_ROW);
    sdf_dispatch[1] = (groups + SDF_GROUPS_PER_ROW - 1u) / SDF_GROUPS_PER_ROW;
    sdf_dispatch[2] = 1u;
    sdf_dispatch[3] = queued;
}
