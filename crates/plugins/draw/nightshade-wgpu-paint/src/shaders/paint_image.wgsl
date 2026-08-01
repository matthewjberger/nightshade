struct ImageInstance {
    position: vec2<f32>,
    size: vec2<f32>,
    uv_min: vec2<f32>,
    uv_max: vec2<f32>,
    color: vec4<f32>,
    clip_rect: vec4<f32>,
    texture_layer: u32,
    depth: f32,
    rotation: f32,
    flags: u32,
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
@group(1) @binding(2) var ui_nearest_sampler: sampler;

const NO_TEXTURE_LAYER: u32 = 0xFFFFFFFFu;
const FLAG_NEAREST: u32 = 1u;

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
    @location(5) @interpolate(flat) flags: u32,
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    let slot = draw_order[vertex.instance_index];
    let instance = instances[slot];
    let center = instance.position + instance.size * 0.5;
    let offset = (vertex.corner - vec2<f32>(0.5, 0.5)) * instance.size;
    let cos_r = cos(instance.rotation);
    let sin_r = sin(instance.rotation);
    let turned = vec2<f32>(
        offset.x * cos_r - offset.y * sin_r,
        offset.x * sin_r + offset.y * cos_r,
    );
    let world_pos = center + turned;
    let uv = mix(instance.uv_min, instance.uv_max, vertex.corner);

    var out: VertexOutput;
    out.position = globals.view_projection * vec4<f32>(world_pos, 0.0, 1.0);
    out.position.z = instance.depth;
    out.uv = uv;
    out.color = instance.color;
    out.screen_pos = world_pos;
    out.clip_rect = instance.clip_rect;
    out.texture_layer = instance.texture_layer;
    out.flags = instance.flags;
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

    var sampled: vec4<f32>;
    if (in.flags & FLAG_NEAREST) != 0u {
        sampled = textureSampleLevel(
            ui_texture_array,
            ui_nearest_sampler,
            in.uv,
            i32(in.texture_layer),
            0.0,
        );
    } else {
        sampled = textureSampleLevel(
            ui_texture_array,
            ui_sampler,
            in.uv,
            i32(in.texture_layer),
            0.0,
        );
    }
    return sampled * in.color;
}
