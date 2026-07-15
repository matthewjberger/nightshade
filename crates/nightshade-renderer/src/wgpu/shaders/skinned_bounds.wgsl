struct BoneTransform {
    transform: mat4x4<f32>,
}

struct BoundsInput {
    base_bone: u32,
    joint_count: u32,
    rest_radius: f32,
    pad: u32,
}

struct SkinnedCullObject {
    bounds: vec4<f32>,
    command_index: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
}

@group(0) @binding(0) var<storage, read> bone_transforms: array<BoneTransform>;
@group(0) @binding(1) var<storage, read> bounds_inputs: array<BoundsInput>;
@group(0) @binding(2) var<storage, read_write> cull_objects: array<SkinnedCullObject>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if index >= arrayLength(&bounds_inputs) {
        return;
    }
    let input = bounds_inputs[index];
    let bone_count = arrayLength(&bone_transforms);
    if input.joint_count == 0u || input.base_bone + input.joint_count > bone_count {
        cull_objects[index].bounds = vec4<f32>(0.0, 0.0, 0.0, 1.0e9);
        return;
    }
    var centroid = vec3<f32>(0.0, 0.0, 0.0);
    for (var j = 0u; j < input.joint_count; j = j + 1u) {
        centroid = centroid + bone_transforms[input.base_bone + j].transform[3].xyz;
    }
    centroid = centroid / f32(input.joint_count);
    var max_distance_squared = 0.0;
    for (var j = 0u; j < input.joint_count; j = j + 1u) {
        let position = bone_transforms[input.base_bone + j].transform[3].xyz;
        let delta = position - centroid;
        max_distance_squared = max(max_distance_squared, dot(delta, delta));
    }
    let radius = sqrt(max_distance_squared) + input.rest_radius;
    cull_objects[index].bounds = vec4<f32>(centroid, radius);
}
