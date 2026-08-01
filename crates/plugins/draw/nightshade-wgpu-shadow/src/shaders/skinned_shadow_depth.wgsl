#import nightshade_wgpu::material_data::{Material}
#import nightshade_wgpu::shadow_cutout::{hidden_by_cutout, NO_SHADOW_MATERIAL}

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
    @location(0) uv: vec2<f32>,
    @location(1) uv_1: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) @interpolate(flat) material_index: u32,
    @location(4) @interpolate(flat) object_index: u32,
};

struct Uniforms {
    light_view_projection: mat4x4<f32>,
    time: f32,
}

struct SkinnedObjectData {
    index_offset: u32,
    index_count: u32,
    vertex_offset: u32,
    joint_offset: u32,
    material_index: u32,
    padding_0: u32,
    padding_1: u32,
    padding_2: u32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var<storage, read> objects: array<SkinnedObjectData>;

@group(1) @binding(1)
var<storage, read> joint_matrices: array<mat4x4<f32>>;

@group(1) @binding(2)
var<storage, read> visible_indices: array<u32>;
@group(1) @binding(3)
var<storage, read> materials: array<Material>;

@vertex
fn vs_main(
    in: SkinnedVertexInput,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    let object_index = visible_indices[instance_index];
    let object = objects[object_index];

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
    out.uv = in.tex_coords;
    out.uv_1 = in.tex_coords_1;
    out.color = in.color;
    out.material_index = object.material_index;
    out.object_index = object_index;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) {
    if in.material_index == NO_SHADOW_MATERIAL {
        return;
    }
    let material = materials[in.material_index];
    if hidden_by_cutout(material, in.uv, in.uv_1, in.color, in.position.xy, in.object_index, uniforms.time) {
        discard;
    }
}
