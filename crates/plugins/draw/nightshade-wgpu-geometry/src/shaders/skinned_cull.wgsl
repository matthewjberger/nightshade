#import nightshade_wgpu::cull_common::{DrawIndexedIndirect, sphere_in_frustum}

struct SkinnedCullObject {
    bounds: vec4<f32>,
    command_index: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

struct SkinnedCullUniforms {
    frustum_planes: array<vec4<f32>, 6>,
    object_count: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

@group(0) @binding(0)
var<storage, read> cull_objects: array<SkinnedCullObject>;

@group(0) @binding(1)
var<storage, read_write> indirect_commands: array<DrawIndexedIndirect>;

@group(0) @binding(2)
var<storage, read_write> visible_indices: array<u32>;

@group(0) @binding(3)
var<uniform> cull: SkinnedCullUniforms;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let object_index = global_id.x;
    if object_index >= cull.object_count {
        return;
    }

    let object = cull_objects[object_index];
    if !sphere_in_frustum(cull.frustum_planes, object.bounds.xyz, object.bounds.w) {
        return;
    }

    let command_index = object.command_index;
    if command_index == 0xFFFFFFFFu {
        return;
    }
    let slot = atomicAdd(&indirect_commands[command_index].instance_count, 1u);
    let first_instance = indirect_commands[command_index].first_instance;
    visible_indices[first_instance + slot] = object_index;
}
