struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0)
var accum_texture: texture_2d<f32>;

@group(0) @binding(1)
var accum_sampler: sampler;

@group(0) @binding(2)
var reveal_texture: texture_2d<f32>;

@group(0) @binding(3)
var reveal_sampler: sampler;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    let x = f32((vertex_index << 1u) & 2u);
    let y = f32(vertex_index & 2u);

    out.position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, 1.0 - y);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let accum = textureSample(accum_texture, accum_sampler, in.uv);
    let reveal = textureSample(reveal_texture, reveal_sampler, in.uv).r;

    if accum.a == 0.0 {
        discard;
    }

    let avg_color = accum.rgb / max(accum.a, 0.00001);

    return vec4<f32>(avg_color, 1.0 - reveal);
}
