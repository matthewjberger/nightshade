struct SkinnedVertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) tex_coords_1: vec2<f32>,
    @location(4) tangent: vec4<f32>,
    @location(5) color: vec4<f32>,
    @location(6) joint_indices: vec4<u32>,
    @location(7) joint_weights: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

struct Uniforms {
    light_view_projection: mat4x4<f32>,
}

struct SkinnedObjectData {
    transform_index: u32,
    mesh_id: u32,
    material_id: u32,
    joint_offset: u32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var<storage, read> objects: array<SkinnedObjectData>;

@group(1) @binding(1)
var<storage, read> joint_matrices: array<mat4x4<f32>>;

@group(1) @binding(2)
var<storage, read> visible_indices: array<u32>;

@vertex
fn vs_main(
    in: SkinnedVertexInput,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    let object = objects[visible_indices[instance_index]];

    var skinned_position = vec3<f32>(0.0, 0.0, 0.0);

    let joint_offset = object.joint_offset;

    for (var index = 0u; index < 4u; index = index + 1u) {
        let joint_index = in.joint_indices[index];
        let joint_weight = in.joint_weights[index];

        if (joint_weight > 0.0) {
            let joint_matrix = joint_matrices[joint_offset + joint_index];

            let transformed_pos = joint_matrix * vec4<f32>(in.position, 1.0);
            skinned_position = skinned_position + transformed_pos.xyz * joint_weight;
        }
    }

    let world_pos = skinned_position;
    let clip_pos = uniforms.light_view_projection * vec4<f32>(world_pos, 1.0);

    var out: VertexOutput;
    out.position = clip_pos;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) {
}
