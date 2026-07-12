struct BloomParams {
    filter_radius: f32,
    _padding0: f32,
    _padding1: f32,
    _padding2: f32,
}

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> params: BloomParams;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let uv = vec2<f32>(
        f32((vertex_index << 1u) & 2u),
        f32(vertex_index & 2u)
    );
    let clip_position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);

    var out: VertexOutput;
    out.clip_position = clip_position;
    out.uv = vec2<f32>(uv.x, 1.0 - uv.y);
    return out;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // The filter kernel is applied with a radius, specified in texture
    // coordinates, so that the radius will vary across mip resolutions.
    let x = params.filter_radius;
    let y = params.filter_radius;

    // Take 9 samples around current texel:
    // a - b - c
    // d - e - f
    // g - h - i
    let a = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(-x, y)).rgb;
    let b = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(0.0, y)).rgb;
    let c = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(x, y)).rgb;

    let d = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(-x, 0.0)).rgb;
    let e = textureSample(source_texture, source_sampler, in.uv).rgb;
    let f = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(x, 0.0)).rgb;

    let g = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(-x, -y)).rgb;
    let h = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(0.0, -y)).rgb;
    let i = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(x, -y)).rgb;

    var upsample = e * 4.0;
    upsample += (b + d + f + h) * 2.0;
    upsample += (a + c + g + i);
    upsample *= 1.0 / 32.0;

    return vec4<f32>(upsample, 1.0);
}
