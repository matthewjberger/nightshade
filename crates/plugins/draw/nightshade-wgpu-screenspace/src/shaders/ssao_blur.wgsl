struct BlurParams {
    screen_size: vec2<f32>,
    depth_threshold: f32,
    normal_power: f32,
}

@group(0) @binding(0) var ssao_texture: texture_2d<f32>;
@group(0) @binding(1) var depth_texture: texture_depth_2d;
@group(0) @binding(2) var normal_texture: texture_2d<f32>;
@group(0) @binding(3) var linear_sampler: sampler;
@group(0) @binding(4) var point_sampler: sampler;
@group(0) @binding(5) var<uniform> params: BlurParams;

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
fn fragment_main(in: VertexOutput) -> @location(0) f32 {
    let texel_size = 1.0 / params.screen_size;

    let center_ao = textureSampleLevel(ssao_texture, linear_sampler, in.uv, 0.0).r;
    let center_depth = textureSampleLevel(depth_texture, point_sampler, in.uv, 0);
    let center_normal_raw = textureSampleLevel(normal_texture, point_sampler, in.uv, 0.0).xyz;
    let center_normal_len = length(center_normal_raw);
    if center_normal_len < 0.001 {
        return center_ao;
    }
    let center_normal = center_normal_raw / center_normal_len;

    var total_weight = 1.0;
    var total_ao = center_ao;

    for (var x = -2i; x <= 2i; x++) {
        for (var y = -2i; y <= 2i; y++) {
            if x == 0 && y == 0 {
                continue;
            }

            let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
            let sample_uv = in.uv + offset;

            let sample_ao = textureSampleLevel(ssao_texture, linear_sampler, sample_uv, 0.0).r;
            let sample_depth = textureSampleLevel(depth_texture, point_sampler, sample_uv, 0);
            let sample_normal_raw = textureSampleLevel(normal_texture, point_sampler, sample_uv, 0.0).xyz;
            let sample_normal_len = length(sample_normal_raw);

            let depth_diff = abs(center_depth - sample_depth);
            let depth_weight = exp(-depth_diff / params.depth_threshold);

            var normal_weight = 0.0;
            if sample_normal_len >= 0.001 {
                let sample_normal = sample_normal_raw / sample_normal_len;
                normal_weight = pow(max(dot(center_normal, sample_normal), 0.0), params.normal_power);
            }

            let weight = depth_weight * normal_weight;

            total_ao += sample_ao * weight;
            total_weight += weight;
        }
    }

    return total_ao / total_weight;
}
