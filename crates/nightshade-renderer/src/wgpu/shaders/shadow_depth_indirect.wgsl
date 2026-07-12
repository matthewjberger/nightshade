struct Uniforms {
    light_view_projection: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var<storage, read> transforms: array<mat4x4<f32>>;
@group(1) @binding(1) var<storage, read> visible_indices: array<u32>;

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

@vertex
fn vertex_main(input: VertexInput) -> @builtin(position) vec4<f32> {
    let transform_index = visible_indices[input.instance_index];
    let model_matrix = transforms[transform_index];
    let world_position = model_matrix * vec4<f32>(input.position, 1.0);
    return uniforms.light_view_projection * world_position;
}
