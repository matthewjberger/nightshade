#import nightshade_wgpu::material_data::{Material}
#import nightshade_wgpu::shadow_cutout::{hidden_by_cutout, NO_SHADOW_MATERIAL}

struct PointShadowUniforms {
    view_projection: mat4x4<f32>,
    light_position: vec3<f32>,
    light_range: f32,
    time: f32,
};

struct ShadowOccluder {
    transform_index: u32,
    mesh_geo_id: u32,
    batch_id: u32,
    material_index: u32,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) tex_coords_1: vec2<f32>,
    @location(4) alpha: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) uv_1: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) @interpolate(flat) material_index: u32,
    @location(5) @interpolate(flat) occluder_index: u32,
};

@group(0) @binding(0)
var<uniform> uniforms: PointShadowUniforms;
@group(1) @binding(0)
var<storage, read> transforms: array<mat4x4<f32>>;
@group(1) @binding(1)
var<storage, read> visible_indices: array<u32>;
@group(1) @binding(2)
var<storage, read> occluders: array<ShadowOccluder>;
@group(1) @binding(3)
var<storage, read> materials: array<Material>;

@vertex
fn vs_main(in: VertexInput, @builtin(instance_index) instance_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let occluder_index = visible_indices[instance_index];
    let occluder = occluders[occluder_index];
    let model_matrix = transforms[occluder.transform_index];
    let world_position = model_matrix * vec4<f32>(in.position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = uniforms.view_projection * world_position;
    out.uv = in.tex_coords;
    out.uv_1 = in.tex_coords_1;
    out.color = vec4<f32>(1.0, 1.0, 1.0, in.alpha);
    out.material_index = occluder.material_index;
    out.occluder_index = occluder_index;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) f32 {
    if in.material_index != NO_SHADOW_MATERIAL {
        let material = materials[in.material_index];
        if hidden_by_cutout(material, in.uv, in.uv_1, in.color, in.clip_position.xy, in.occluder_index, uniforms.time) {
            discard;
        }
    }
    let dist = length(in.world_position - uniforms.light_position);
    return dist / uniforms.light_range;
}

@fragment
fn fs_distance(in: VertexOutput) -> @location(0) f32 {
    let dist = length(in.world_position - uniforms.light_position);
    return dist / uniforms.light_range;
}
