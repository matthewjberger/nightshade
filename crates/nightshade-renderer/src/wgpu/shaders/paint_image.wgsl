struct ImageInstance {
    position: vec2<f32>,
    size: vec2<f32>,
    uv_min: vec2<f32>,
    uv_max: vec2<f32>,
    color: vec4<f32>,
    clip_rect: vec4<f32>,
    texture_layer: u32,
    depth: f32,
    _padding0: f32,
    _padding1: f32,
}

struct GlobalUniforms {
    view_projection: mat4x4<f32>,
    _padding: vec4<f32>,
}

@group(0) @binding(0) var<uniform> globals: GlobalUniforms;
@group(0) @binding(1) var<storage, read> instances: array<ImageInstance>;
@group(0) @binding(2) var<storage, read> draw_order: array<u32>;
@group(1) @binding(0) var ui_texture_array: texture_2d_array<f32>;
@group(1) @binding(1) var ui_sampler: sampler;

const NO_TEXTURE_LAYER: u32 = 0xFFFFFFFFu;

struct VertexInput {
    @location(0) corner: vec2<f32>,
    @builtin(instance_index) instance_index: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) screen_pos: vec2<f32>,
    @location(3) @interpolate(flat) clip_rect: vec4<f32>,
    @location(4) @interpolate(flat) texture_layer: u32,
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    let slot = draw_order[vertex.instance_index];
    let instance = instances[slot];
    let world_pos = instance.position + vertex.corner * instance.size;
    let uv = mix(instance.uv_min, instance.uv_max, vertex.corner);

    var out: VertexOutput;
    out.position = globals.view_projection * vec4<f32>(world_pos, 0.0, 1.0);
    out.position.z = instance.depth;
    out.uv = uv;
    out.color = instance.color;
    out.screen_pos = world_pos;
    out.clip_rect = instance.clip_rect;
    out.texture_layer = instance.texture_layer;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if in.screen_pos.x < in.clip_rect.x
        || in.screen_pos.y < in.clip_rect.y
        || in.screen_pos.x > in.clip_rect.z
        || in.screen_pos.y > in.clip_rect.w
    {
        discard;
    }

    if in.texture_layer == NO_TEXTURE_LAYER {
        return in.color;
    }

    let sampled = textureSampleLevel(
        ui_texture_array,
        ui_sampler,
        in.uv,
        i32(in.texture_layer),
        0.0,
    );
    return sampled * in.color;
}
