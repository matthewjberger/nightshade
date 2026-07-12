struct SsaoParams {
    projection: mat4x4<f32>,
    inv_projection: mat4x4<f32>,
    screen_size: vec2<f32>,
    radius: f32,
    bias: f32,
    intensity: f32,
    enabled: f32,
    sample_count: f32,
    _padding: f32,
}

const KERNEL_SIZE: u32 = 64u;

@group(0) @binding(0) var depth_texture: texture_depth_2d;
@group(0) @binding(1) var normal_texture: texture_2d<f32>;
@group(0) @binding(2) var noise_texture: texture_2d<f32>;
@group(0) @binding(3) var point_sampler: sampler;
@group(0) @binding(4) var noise_sampler: sampler;
@group(0) @binding(5) var<uniform> params: SsaoParams;
@group(0) @binding(6) var<storage, read> kernel_samples: array<vec4<f32>, 64>;

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

fn reconstruct_view_position(uv: vec2<f32>, depth: f32) -> vec3<f32> {
    let ndc_x = uv.x * 2.0 - 1.0;
    let ndc_y = 1.0 - uv.y * 2.0;
    let ndc = vec4<f32>(ndc_x, ndc_y, depth, 1.0);
    var view_pos = params.inv_projection * ndc;
    view_pos /= view_pos.w;
    return view_pos.xyz;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) f32 {
    if params.enabled < 0.5 {
        return 1.0;
    }

    let depth = textureSampleLevel(depth_texture, point_sampler, in.uv, 0);

    if depth == 0.0 {
        return 1.0;
    }

    let view_pos = reconstruct_view_position(in.uv, depth);
    let normal_sample = textureSampleLevel(normal_texture, point_sampler, in.uv, 0.0).xyz;
    let normal_len = length(normal_sample);
    if normal_len < 0.001 {
        return 1.0;
    }
    let view_normal = normal_sample / normal_len;

    let noise_scale = params.screen_size / 4.0;
    let noise_sample = textureSampleLevel(noise_texture, noise_sampler, in.uv * noise_scale, 0.0).xyz * 2.0 - 1.0;
    var random_vec = vec3<f32>(noise_sample.x, noise_sample.y, 0.0);
    let random_len = length(random_vec);
    if random_len < 0.001 {
        random_vec = vec3<f32>(1.0, 0.0, 0.0);
    } else {
        random_vec = random_vec / random_len;
    }

    var tangent = random_vec - view_normal * dot(random_vec, view_normal);
    let tangent_len = length(tangent);
    if tangent_len < 0.001 {
        tangent = vec3<f32>(1.0, 0.0, 0.0);
        if abs(view_normal.x) > 0.9 {
            tangent = vec3<f32>(0.0, 1.0, 0.0);
        }
        tangent = normalize(tangent - view_normal * dot(tangent, view_normal));
    } else {
        tangent = tangent / tangent_len;
    }
    let bitangent = cross(view_normal, tangent);
    let tbn = mat3x3<f32>(tangent, bitangent, view_normal);

    var occlusion = 0.0;
    let radius = params.radius;
    let bias = params.bias;

    let active_samples = min(u32(params.sample_count), KERNEL_SIZE);
    for (var sample_index = 0u; sample_index < active_samples; sample_index++) {
        var sample_offset = tbn * kernel_samples[sample_index].xyz;
        let sample_pos = view_pos + sample_offset * radius;

        var clip_pos = params.projection * vec4<f32>(sample_pos, 1.0);
        clip_pos /= clip_pos.w;
        let sample_uv = vec2<f32>(clip_pos.x * 0.5 + 0.5, 0.5 - clip_pos.y * 0.5);

        if sample_uv.x < 0.0 || sample_uv.x > 1.0 || sample_uv.y < 0.0 || sample_uv.y > 1.0 {
            continue;
        }

        let sample_depth = textureSampleLevel(depth_texture, point_sampler, sample_uv, 0);

        if sample_depth == 0.0 {
            continue;
        }

        let sample_view_pos = reconstruct_view_position(sample_uv, sample_depth);

        let sample_normal_raw = textureSampleLevel(normal_texture, point_sampler, sample_uv, 0.0).xyz;
        let sample_normal_len = length(sample_normal_raw);
        if sample_normal_len > 0.001 {
            let sample_normal = sample_normal_raw / sample_normal_len;
            if dot(view_normal, sample_normal) < 0.3 {
                continue;
            }
        }

        let depth_diff = abs(view_pos.z - sample_view_pos.z);
        if depth_diff < bias {
            continue;
        }

        let range_check = smoothstep(0.0, 1.0, radius / depth_diff);

        if sample_view_pos.z >= sample_pos.z + bias {
            occlusion += range_check;
        }
    }

    occlusion = 1.0 - (occlusion / f32(active_samples));
    let final_occlusion = pow(occlusion, params.intensity);

    return final_occlusion;
}
