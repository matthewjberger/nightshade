struct OutlineParams {
    outline_color: vec4<f32>,
    outline_width: f32,
    texture_width: f32,
    texture_height: f32,
    padding: f32,
};

@group(0) @binding(0) var outline_mask: texture_2d<f32>;
@group(0) @binding(1) var mask_sampler: sampler;
@group(0) @binding(2) var<uniform> params: OutlineParams;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

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
    let center_mask = textureSample(outline_mask, mask_sampler, in.uv).r;

    let pixel_size = vec2<f32>(1.0 / params.texture_width, 1.0 / params.texture_height);
    let offset = pixel_size * params.outline_width;

    var max_neighbor: f32 = 0.0;

    max_neighbor = max(max_neighbor, textureSample(outline_mask, mask_sampler, in.uv + vec2<f32>(-offset.x, -offset.y)).r);
    max_neighbor = max(max_neighbor, textureSample(outline_mask, mask_sampler, in.uv + vec2<f32>(0.0, -offset.y)).r);
    max_neighbor = max(max_neighbor, textureSample(outline_mask, mask_sampler, in.uv + vec2<f32>(offset.x, -offset.y)).r);
    max_neighbor = max(max_neighbor, textureSample(outline_mask, mask_sampler, in.uv + vec2<f32>(-offset.x, 0.0)).r);
    max_neighbor = max(max_neighbor, textureSample(outline_mask, mask_sampler, in.uv + vec2<f32>(offset.x, 0.0)).r);
    max_neighbor = max(max_neighbor, textureSample(outline_mask, mask_sampler, in.uv + vec2<f32>(-offset.x, offset.y)).r);
    max_neighbor = max(max_neighbor, textureSample(outline_mask, mask_sampler, in.uv + vec2<f32>(0.0, offset.y)).r);
    max_neighbor = max(max_neighbor, textureSample(outline_mask, mask_sampler, in.uv + vec2<f32>(offset.x, offset.y)).r);

    let is_outline = max_neighbor * (1.0 - center_mask);

    return vec4<f32>(params.outline_color.rgb, is_outline * params.outline_color.a);
}
