struct SpriteInstance {
    model: mat4x4<f32>,
    uv_min: vec2<f32>,
    uv_max: vec2<f32>,
    size: vec2<f32>,
    anchor: vec2<f32>,
    color: vec4<f32>,
    flags: u32,
    lean: f32,
    _padding0: u32,
    _padding1: u32,
}

struct SpriteUniforms {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    camera_right: vec4<f32>,
    camera_up: vec4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: SpriteUniforms;
@group(0) @binding(1) var<storage, read> instances: array<SpriteInstance>;
@group(1) @binding(0) var atlas_texture: texture_2d<f32>;
@group(1) @binding(1) var atlas_sampler: sampler;

const FLAG_BILLBOARD: u32 = 1u;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

fn corner_of(vertex_index: u32) -> vec2<f32> {
    switch vertex_index {
        case 0u: { return vec2<f32>(0.0, 0.0); }
        case 1u: { return vec2<f32>(1.0, 0.0); }
        case 2u: { return vec2<f32>(1.0, 1.0); }
        case 3u: { return vec2<f32>(1.0, 1.0); }
        case 4u: { return vec2<f32>(0.0, 1.0); }
        default: { return vec2<f32>(0.0, 0.0); }
    }
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    let instance = instances[instance_index];
    let corner = corner_of(vertex_index);
    let square = (corner - instance.anchor) * instance.size;
    let local = vec2<f32>(square.x + instance.lean * square.y, square.y);

    var world: vec4<f32>;
    if (instance.flags & FLAG_BILLBOARD) != 0u {
        let scale_x = length(instance.model[0].xyz);
        let scale_y = length(instance.model[1].xyz);
        let center = instance.model[3].xyz;
        let offset = uniforms.camera_right.xyz * (local.x * scale_x)
            + uniforms.camera_up.xyz * (local.y * scale_y);
        world = vec4<f32>(center + offset, 1.0);
    } else {
        world = instance.model * vec4<f32>(local, 0.0, 1.0);
    }

    var out: VertexOutput;
    out.position = uniforms.projection * uniforms.view * world;
    out.uv = mix(instance.uv_min, instance.uv_max, vec2<f32>(corner.x, 1.0 - corner.y));
    out.color = instance.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sampled = textureSample(atlas_texture, atlas_sampler, in.uv) * in.color;
    if sampled.a <= 0.002 {
        discard;
    }
    return sampled;
}
