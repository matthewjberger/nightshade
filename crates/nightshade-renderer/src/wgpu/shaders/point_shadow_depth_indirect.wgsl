struct PointShadowUniforms {
    view_projection: mat4x4<f32>,
    light_position: vec3<f32>,
    light_range: f32,
};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: PointShadowUniforms;

@group(1) @binding(0)
var<storage, read> transforms: array<mat4x4<f32>>;

@group(1) @binding(1)
var<storage, read> visible_indices: array<u32>;

@vertex
fn vs_main(in: VertexInput, @builtin(instance_index) instance_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let transform_index = visible_indices[instance_index];
    let model_matrix = transforms[transform_index];
    let world_position = model_matrix * vec4<f32>(in.position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = uniforms.view_projection * world_position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) f32 {
    let dist = length(in.world_position - uniforms.light_position);
    return dist / uniforms.light_range;
}
