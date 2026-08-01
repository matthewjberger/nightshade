struct BoneTransform {
    transform: mat4x4<f32>,
}

struct SkinData {
    joint_count: u32,
    base_bone_index: u32,
    base_ibm_index: u32,
    base_output_index: u32,
}

@group(0) @binding(0)
var<storage, read> bone_transforms: array<BoneTransform>;

@group(0) @binding(1)
var<storage, read> inverse_bind_matrices: array<mat4x4<f32>>;

@group(0) @binding(2)
var<storage, read> skin_data: array<SkinData>;

@group(0) @binding(3)
var<storage, read_write> joint_matrices: array<mat4x4<f32>>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let output_index = global_id.x;

    let num_skins = arrayLength(&skin_data);
    if (num_skins == 0u) {
        return;
    }

    var skin_index = 0u;
    var local_joint_index = output_index;

    for (var search_index = 0u; search_index < num_skins; search_index = search_index + 1u) {
        let skin = skin_data[search_index];
        let skin_end = skin.base_output_index + skin.joint_count;
        if (output_index >= skin.base_output_index && output_index < skin_end) {
            skin_index = search_index;
            local_joint_index = output_index - skin.base_output_index;
            break;
        }
    }

    let skin = skin_data[skin_index];

    if (local_joint_index >= skin.joint_count) {
        return;
    }

    let bone_index = skin.base_bone_index + local_joint_index;
    let inverse_bind_index = skin.base_ibm_index + local_joint_index;

    let bone_transform = bone_transforms[bone_index].transform;
    let inverse_bind = inverse_bind_matrices[inverse_bind_index];

    joint_matrices[output_index] = bone_transform * inverse_bind;
}
