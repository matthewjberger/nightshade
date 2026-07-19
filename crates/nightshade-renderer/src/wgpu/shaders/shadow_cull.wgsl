#import nightshade_renderer::cull_common::{DrawIndexedIndirect, sphere_in_frustum}

struct ShadowOccluder {
    transform_index: u32,
    mesh_geo_id: u32,
    batch_id: u32,
    _pad: u32,
};

struct MeshBounds {
    center: vec3<f32>,
    radius: f32,
};

struct CullUniforms {
    frustum_planes: array<vec4<f32>, 6>,
    occluder_count: u32,
    indirect_offset: u32,
    _pad0: u32,
    _pad1: u32,
};

@group(0) @binding(0) var<storage, read> occluders: array<ShadowOccluder>;
@group(0) @binding(1) var<storage, read> transforms: array<mat4x4<f32>>;
@group(0) @binding(2) var<storage, read> mesh_bounds: array<MeshBounds>;
@group(0) @binding(3) var<storage, read_write> indirect_commands: array<DrawIndexedIndirect>;
@group(0) @binding(4) var<storage, read_write> visible_indices: array<u32>;
@group(0) @binding(5) var<uniform> cull: CullUniforms;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let occluder_index = global_id.x;
    if occluder_index >= cull.occluder_count {
        return;
    }

    let occluder = occluders[occluder_index];
    let bounds = mesh_bounds[occluder.mesh_geo_id];
    let model = transforms[occluder.transform_index];

    let world_center = (model * vec4<f32>(bounds.center, 1.0)).xyz;
    let scale_x = length(model[0].xyz);
    let scale_y = length(model[1].xyz);
    let scale_z = length(model[2].xyz);
    let world_radius = bounds.radius * max(scale_x, max(scale_y, scale_z));

    if !sphere_in_frustum(cull.frustum_planes, world_center, world_radius) {
        return;
    }

    let command_index = cull.indirect_offset + occluder.batch_id;
    let slot = atomicAdd(&indirect_commands[command_index].instance_count, 1u);
    let first_instance = indirect_commands[command_index].first_instance;
    visible_indices[first_instance + slot] = occluder.transform_index;
}
