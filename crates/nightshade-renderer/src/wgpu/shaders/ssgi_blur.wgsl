struct SsgiBlurParams {
    screen_size: vec2<f32>,
    depth_threshold: f32,
    normal_threshold: f32,
}

@group(0) @binding(0) var ssgi_texture: texture_2d<f32>;
@group(0) @binding(1) var depth_texture: texture_depth_2d;
@group(0) @binding(2) var normal_texture: texture_2d<f32>;
@group(0) @binding(3) var linear_sampler: sampler;
@group(0) @binding(4) var point_sampler: sampler;
@group(0) @binding(5) var<uniform> params: SsgiBlurParams;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var output: VertexOutput;
    let x = f32((vertex_index << 1u) & 2u);
    let y = f32(vertex_index & 2u);
    output.position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    output.uv = vec2<f32>(x, 1.0 - y);
    return output;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = 1.0 / params.screen_size;
    let center_depth = textureSampleLevel(depth_texture, point_sampler, in.uv, 0);
    let center_normal = textureSampleLevel(normal_texture, point_sampler, in.uv, 0.0).xyz;

    var result = vec3<f32>(0.0);
    var total_weight = 0.0;

    for (var x = -2i; x <= 2i; x++) {
        for (var y = -2i; y <= 2i; y++) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
            let sample_uv = in.uv + offset;

            let sample_color = textureSampleLevel(ssgi_texture, linear_sampler, sample_uv, 0.0).rgb;
            let sample_depth = textureSampleLevel(depth_texture, point_sampler, sample_uv, 0);
            let sample_normal = textureSampleLevel(normal_texture, point_sampler, sample_uv, 0.0).xyz;

            let depth_diff = abs(center_depth - sample_depth);
            let depth_weight = 1.0 - saturate(depth_diff / params.depth_threshold);

            let normal_similarity = max(dot(center_normal, sample_normal), 0.0);
            let normal_weight = pow(normal_similarity, params.normal_threshold);

            let spatial_dist = length(vec2<f32>(f32(x), f32(y)));
            let spatial_weight = exp(-spatial_dist * spatial_dist / 4.5);

            let weight = depth_weight * normal_weight * spatial_weight;
            result += sample_color * weight;
            total_weight += weight;
        }
    }

    if total_weight > 0.0 {
        result /= total_weight;
    }

    return vec4<f32>(result, 1.0);
}
