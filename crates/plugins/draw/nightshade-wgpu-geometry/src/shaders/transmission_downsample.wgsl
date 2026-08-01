@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;

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
    var out: VertexOutput;
    out.clip_position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(uv.x, 1.0 - uv.y);
    return out;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = 1.0 / vec2<f32>(textureDimensions(source_texture));
    let x = texel.x;
    let y = texel.y;

    let a = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(-2.0 * x, 2.0 * y));
    let b = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(0.0, 2.0 * y));
    let c = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(2.0 * x, 2.0 * y));
    let d = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(-2.0 * x, 0.0));
    let e = textureSample(source_texture, source_sampler, in.uv);
    let f = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(2.0 * x, 0.0));
    let g = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(-2.0 * x, -2.0 * y));
    let h = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(0.0, -2.0 * y));
    let i = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(2.0 * x, -2.0 * y));
    let j = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(-x, y));
    let k = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(x, y));
    let l = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(-x, -y));
    let m = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(x, -y));

    var result = e * 0.125;
    result += (a + c + g + i) * 0.03125;
    result += (b + d + f + h) * 0.0625;
    result += (j + k + l + m) * 0.125;
    return result;
}
