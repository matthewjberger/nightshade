struct SsgiParams {
    projection: mat4x4<f32>,
    inv_projection: mat4x4<f32>,
    screen_size: vec2<f32>,
    radius: f32,
    intensity: f32,
    max_steps: u32,
    enabled: f32,
    _padding: vec2<f32>,
}

const KERNEL_SIZE: u32 = 16u;
const THICKNESS: f32 = 0.3;

@group(0) @binding(0) var depth_texture: texture_depth_2d;
@group(0) @binding(1) var normal_texture: texture_2d<f32>;
@group(0) @binding(2) var scene_color_texture: texture_2d<f32>;
@group(0) @binding(3) var noise_texture: texture_2d<f32>;
@group(0) @binding(4) var point_sampler: sampler;
@group(0) @binding(5) var linear_sampler: sampler;
@group(0) @binding(6) var noise_sampler: sampler;
@group(0) @binding(7) var<uniform> params: SsgiParams;
@group(0) @binding(8) var<storage, read> kernel_samples: array<vec4<f32>, 16>;

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

fn project_to_uv(view_pos: vec3<f32>) -> vec2<f32> {
    var clip_pos = params.projection * vec4<f32>(view_pos, 1.0);
    clip_pos /= clip_pos.w;
    return vec2<f32>(clip_pos.x * 0.5 + 0.5, 0.5 - clip_pos.y * 0.5);
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if params.enabled < 0.5 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    let depth = textureSampleLevel(depth_texture, point_sampler, in.uv, 0);

    if depth == 0.0 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    let view_pos = reconstruct_view_position(in.uv, depth);
    let normal_sample = textureSampleLevel(normal_texture, point_sampler, in.uv, 0.0).xyz;
    let normal_len = length(normal_sample);
    if normal_len < 0.001 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
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

    var indirect_color = vec3<f32>(0.0);
    let radius = params.radius;
    let step_size = radius / f32(params.max_steps);
    var hit_count = 0.0;

    for (var sample_index = 0u; sample_index < KERNEL_SIZE; sample_index++) {
        let sample_dir = normalize(tbn * kernel_samples[sample_index].xyz);
        let cosine_weight = max(dot(sample_dir, view_normal), 0.0);

        if cosine_weight < 0.01 {
            continue;
        }

        var hit_found = false;
        var hit_uv = vec2<f32>(0.0);
        var hit_distance = 0.0;

        for (var step = 1u; step <= params.max_steps; step++) {
            let march_pos = view_pos + sample_dir * (f32(step) * step_size);
            let sample_uv = project_to_uv(march_pos);

            if sample_uv.x < 0.0 || sample_uv.x > 1.0 || sample_uv.y < 0.0 || sample_uv.y > 1.0 {
                break;
            }

            let sample_depth = textureSampleLevel(depth_texture, point_sampler, sample_uv, 0);

            if sample_depth == 0.0 {
                continue;
            }

            let sample_view_pos = reconstruct_view_position(sample_uv, sample_depth);
            let depth_diff = sample_view_pos.z - march_pos.z;

            if depth_diff > 0.0 && depth_diff < THICKNESS {
                hit_found = true;
                hit_uv = sample_uv;
                hit_distance = f32(step) * step_size;
                break;
            }
        }

        if hit_found {
            let raw_hit_color = textureSampleLevel(scene_color_texture, linear_sampler, hit_uv, 0.0).rgb;
            let hit_color = min(raw_hit_color, vec3<f32>(1.0));
            let distance_attenuation = 1.0 - saturate(hit_distance / radius);
            indirect_color += hit_color * cosine_weight * distance_attenuation;
            hit_count += 1.0;
        }
    }

    if hit_count > 0.0 {
        indirect_color /= f32(KERNEL_SIZE);
    }

    return vec4<f32>(indirect_color, 1.0);
}
