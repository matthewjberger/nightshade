#import nightshade_wgpu::cull_common::{DrawIndexedIndirect, sphere_in_frustum}

struct SkinnedShadowCullObject {
    bounds: vec4<f32>,
    command_index: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

struct SkinnedShadowCullUniforms {
    frustum_planes: array<vec4<f32>, 6>,
    occluder_count: u32,
    command_base: u32,
    _pad0: u32,
    _pad1: u32,
};

@group(0) @binding(0)
var<storage, read> cull_objects: array<SkinnedShadowCullObject>;

@group(0) @binding(1)
var<storage, read_write> indirect_commands: array<DrawIndexedIndirect>;

@group(0) @binding(2)
var<storage, read_write> visible_indices: array<u32>;

@group(0) @binding(3)
var<uniform> cull: SkinnedShadowCullUniforms;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let occluder_index = global_id.x;
    if occluder_index >= cull.occluder_count {
        return;
    }

    let object = cull_objects[occluder_index];
    if !sphere_in_frustum(cull.frustum_planes, object.bounds.xyz, object.bounds.w) {
        return;
    }

    let command_index = cull.command_base + object.command_index;
    let slot = atomicAdd(&indirect_commands[command_index].instance_count, 1u);
    let first_instance = indirect_commands[command_index].first_instance;
    visible_indices[first_instance + slot] = occluder_index;
}
