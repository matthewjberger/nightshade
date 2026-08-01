#import nightshade_wgpu::material_data::{Material}
#import nightshade_wgpu::shadow_cutout::{hidden_by_cutout, NO_SHADOW_MATERIAL}

struct Uniforms {
    light_view_projection: mat4x4<f32>,
    time: f32,
}

struct ShadowOccluder {
    transform_index: u32,
    mesh_geo_id: u32,
    batch_id: u32,
    material_index: u32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var<storage, read> transforms: array<mat4x4<f32>>;
@group(1) @binding(1) var<storage, read> visible_indices: array<u32>;
@group(1) @binding(2) var<storage, read> occluders: array<ShadowOccluder>;
@group(1) @binding(3) var<storage, read> materials: array<Material>;

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) uv_1: vec2<f32>,
    @location(4) alpha: f32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) uv_1: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) @interpolate(flat) material_index: u32,
    @location(4) @interpolate(flat) occluder_index: u32,
}

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    let occluder_index = visible_indices[input.instance_index];
    let occluder = occluders[occluder_index];
    let model_matrix = transforms[occluder.transform_index];
    let world_position = model_matrix * vec4<f32>(input.position, 1.0);

    var out: VertexOutput;
    out.position = uniforms.light_view_projection * world_position;
    out.uv = input.uv;
    out.uv_1 = input.uv_1;
    out.color = vec4<f32>(1.0, 1.0, 1.0, input.alpha);
    out.material_index = occluder.material_index;
    out.occluder_index = occluder_index;
    return out;
}

@fragment
fn fragment_main(in: VertexOutput) {
    if in.material_index == NO_SHADOW_MATERIAL {
        return;
    }
    let material = materials[in.material_index];
    if hidden_by_cutout(material, in.uv, in.uv_1, in.color, in.position.xy, in.occluder_index, uniforms.time) {
        discard;
    }
}
