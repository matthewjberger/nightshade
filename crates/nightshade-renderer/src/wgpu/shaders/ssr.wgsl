struct SsrParams {
    projection: mat4x4<f32>,
    inv_projection: mat4x4<f32>,
    view: mat4x4<f32>,
    inv_view: mat4x4<f32>,
    screen_size: vec2<f32>,
    max_steps: u32,
    thickness: f32,
    max_distance: f32,
    stride: f32,
    fade_start: f32,
    fade_end: f32,
    intensity: f32,
    enabled: f32,
    _padding: vec2<f32>,
}

@group(0) @binding(0) var depth_texture: texture_depth_2d;
@group(0) @binding(1) var normal_texture: texture_2d<f32>;
@group(0) @binding(2) var scene_color_texture: texture_2d<f32>;
@group(0) @binding(3) var point_sampler: sampler;
@group(0) @binding(4) var linear_sampler: sampler;
@group(0) @binding(5) var<uniform> params: SsrParams;

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

fn ndc_to_uv(ndc: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
}

fn reconstruct_view_position(uv: vec2<f32>, depth: f32) -> vec3<f32> {
    let ndc = vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, depth, 1.0);
    var view_pos = params.inv_projection * ndc;
    view_pos /= view_pos.w;
    return view_pos.xyz;
}

fn view_to_ndc(view_pos: vec3<f32>) -> vec3<f32> {
    let clip = params.projection * vec4<f32>(view_pos, 1.0);
    return clip.xyz / clip.w;
}

fn interleaved_gradient_noise(pixel: vec2<f32>) -> f32 {
    return fract(52.9829189 * fract(0.06711056 * pixel.x + 0.00583715 * pixel.y));
}

fn screen_edge_fade(uv: vec2<f32>) -> f32 {
    let edge = 0.12;
    let fade_x = smoothstep(0.0, edge, uv.x) * smoothstep(1.0, 1.0 - edge, uv.x);
    let fade_y = smoothstep(0.0, edge, uv.y) * smoothstep(1.0, 1.0 - edge, uv.y);
    return fade_x * fade_y;
}

fn depth_texel_clamped(texel: vec2<i32>) -> f32 {
    let dims = textureDimensions(depth_texture);
    let max_coord = vec2<i32>(i32(dims.x) - 1, i32(dims.y) - 1);
    let clamped = clamp(texel, vec2<i32>(0), max_coord);
    return textureLoad(depth_texture, clamped, 0);
}

fn depth_sample_nearest_manual(uv: vec2<f32>, tex_size: vec2<f32>) -> f32 {
    let coord = uv * tex_size - vec2<f32>(0.5);
    return depth_texel_clamped(vec2<i32>(floor(coord + vec2<f32>(0.5))));
}

fn depth_sample_bilinear_manual(uv: vec2<f32>, tex_size: vec2<f32>) -> f32 {
    let coord = uv * tex_size - vec2<f32>(0.5);
    let base = vec2<i32>(floor(coord));
    let frac_part = coord - floor(coord);

    let d00 = depth_texel_clamped(base);
    let d10 = depth_texel_clamped(base + vec2<i32>(1, 0));
    let d01 = depth_texel_clamped(base + vec2<i32>(0, 1));
    let d11 = depth_texel_clamped(base + vec2<i32>(1, 1));

    let d0 = mix(d00, d10, frac_part.x);
    let d1 = mix(d01, d11, frac_part.x);
    return mix(d0, d1, frac_part.y);
}

struct DistanceResult {
    distance: f32,
    valid: bool,
    penetration: f32,
}

fn evaluate_depth_distance(
    ray_point_ndc: vec3<f32>,
    depth_thickness: f32,
    tex_size: vec2<f32>,
) -> DistanceResult {
    let uv = ndc_to_uv(ray_point_ndc.xy);
    let ray_depth = 1.0 / ray_point_ndc.z;

    let linear_depth = 1.0 / depth_sample_bilinear_manual(uv, tex_size);
    let unfiltered_depth = 1.0 / depth_sample_nearest_manual(uv, tex_size);

    let max_depth = max(linear_depth, unfiltered_depth);
    let min_depth = min(linear_depth, unfiltered_depth);

    let bias = 0.000002;

    var result: DistanceResult;
    result.distance = max_depth * (1.0 + bias) - ray_depth;
    result.penetration = ray_depth - min_depth;
    result.valid = result.penetration < depth_thickness;

    return result;
}

struct RayMarchHit {
    hit: bool,
    hit_t: f32,
    hit_uv: vec2<f32>,
}

fn trace_ray(
    ray_start_ndc: vec3<f32>,
    ray_end_ndc: vec3<f32>,
    jitter: f32,
    depth_thickness: f32,
) -> RayMarchHit {
    var result: RayMarchHit;
    result.hit = false;
    result.hit_t = 0.0;
    result.hit_uv = vec2<f32>(0.0);

    let tex_size = vec2<f32>(textureDimensions(depth_texture));
    let dir = ray_end_ndc - ray_start_ndc;

    let start_uv = ndc_to_uv(ray_start_ndc.xy);
    let end_uv = ndc_to_uv(ray_end_ndc.xy);
    let ray_len_px = (end_uv - start_uv) * params.screen_size;
    let step_count = clamp(
        u32(floor(length(ray_len_px))),
        2u,
        params.max_steps
    );

    var min_t = 0.0;
    var max_t = 1.0;
    var min_d = DistanceResult(0.0, false, 0.0);
    var max_d = DistanceResult(0.0, false, 0.0);
    var intersected = false;

    let first_t = jitter / f32(step_count);
    let first_point = ray_start_ndc + dir * first_t;
    let first_d = evaluate_depth_distance(first_point, depth_thickness, tex_size);

    if first_d.distance < 0.0 && first_d.valid {
        max_t = first_t;
        max_d = first_d;
        intersected = true;
    } else {
        min_t = first_t;
        min_d = first_d;

        for (var step = 1u; step < step_count; step += 1u) {
            let t = (f32(step) + jitter) / f32(step_count);
            let point = ray_start_ndc + dir * t;
            let d = evaluate_depth_distance(point, depth_thickness, tex_size);

            if d.distance < 0.0 && d.valid {
                max_t = t;
                max_d = d;
                intersected = true;
                break;
            }
            min_t = t;
            min_d = d;
        }
    }

    if !intersected {
        result.hit_t = min_t;
        return result;
    }

    for (var step = 0u; step < 5u; step += 1u) {
        let mid_t = (min_t + max_t) * 0.5;
        let mid_point = ray_start_ndc + dir * mid_t;
        let mid_d = evaluate_depth_distance(mid_point, depth_thickness, tex_size);

        if mid_d.distance < 0.0 && mid_d.valid {
            max_t = mid_t;
            max_d = mid_d;
        } else {
            min_t = mid_t;
            min_d = mid_d;
        }
    }

    var final_t = max_t;
    var final_d = max_d;

    let total_d = min_d.distance + (-max_d.distance);
    if total_d > 0.001 {
        let secant_t = mix(min_t, max_t, min_d.distance / total_d);
        let secant_point = ray_start_ndc + dir * secant_t;
        let secant_d = evaluate_depth_distance(secant_point, depth_thickness, tex_size);

        if abs(secant_d.distance) < min_d.distance * 0.9 && secant_d.valid {
            final_t = secant_t;
            final_d = secant_d;
        }
    }

    if final_d.penetration < depth_thickness {
        result.hit = true;
        result.hit_t = final_t;
        result.hit_uv = ndc_to_uv(mix(ray_start_ndc.xy, ray_end_ndc.xy, final_t));
    }

    return result;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if params.enabled < 0.5 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let depth = textureSampleLevel(depth_texture, point_sampler, in.uv, 0);
    if depth == 0.0 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let view_pos = reconstruct_view_position(in.uv, depth);
    let normal_sample = textureSampleLevel(normal_texture, point_sampler, in.uv, 0.0).xyz;
    let normal_len = length(normal_sample);
    if normal_len < 0.001 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    let view_normal = normal_sample / normal_len;

    let view_dir = normalize(view_pos);
    let reflect_dir = reflect(view_dir, view_normal);

    if dot(reflect_dir, view_normal) < 0.01 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let ray_origin = view_pos + view_normal * 0.02;
    var ray_start_ndc = view_to_ndc(ray_origin);

    let clip_dir = params.projection * vec4<f32>(normalize(reflect_dir), 0.0);
    var end_cs = vec4<f32>(ray_start_ndc, 1.0) + clip_dir;
    end_cs /= select(-1.0, 1.0, end_cs.w >= 0.0) * max(1e-10, abs(end_cs.w));
    var ray_end_ndc = end_cs.xyz;

    var delta = ray_end_ndc - ray_start_ndc;
    if length(delta) < 0.0001 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let near_edge = select(vec3<f32>(-1.0, -1.0, 0.0), vec3<f32>(1.0, 1.0, 1.0), delta < vec3<f32>(0.0));
    let dist_to_near = (near_edge - ray_start_ndc) / delta;
    let max_dist_to_near = max(dist_to_near.x, dist_to_near.y);
    ray_start_ndc += delta * max(0.0, max_dist_to_near);

    delta = ray_end_ndc - ray_start_ndc;
    let far_edge = select(vec3<f32>(-1.0, -1.0, 0.0), vec3<f32>(1.0, 1.0, 1.0), delta >= vec3<f32>(0.0));
    let dist_to_far = (far_edge - ray_start_ndc) / delta;
    let min_dist_to_far = min(min(dist_to_far.x, dist_to_far.y), dist_to_far.z);
    delta *= min_dist_to_far;
    ray_end_ndc = ray_start_ndc + delta;

    let near_plane = params.projection[3][2];
    let depth_thickness = params.thickness / max(near_plane, 0.001);

    let pixel_coord = in.uv * params.screen_size;
    let jitter = interleaved_gradient_noise(pixel_coord);

    let march = trace_ray(ray_start_ndc, ray_end_ndc, jitter, depth_thickness);

    if march.hit {
        let hit_uv = march.hit_uv;

        if hit_uv.x < 0.0 || hit_uv.x > 1.0 || hit_uv.y < 0.0 || hit_uv.y > 1.0 {
            return vec4<f32>(0.0, 0.0, 0.0, 0.0);
        }

        let raw_color = textureSampleLevel(scene_color_texture, linear_sampler, hit_uv, 0.0).rgb;
        let reflected_color = min(raw_color, vec3<f32>(2.0));

        let distance_ratio = march.hit_t;
        let fade_range = params.fade_end - params.fade_start;
        var distance_fade = 1.0;
        if fade_range > 0.001 {
            distance_fade = 1.0 - saturate((distance_ratio - params.fade_start) / fade_range);
        }

        let edge_fade = screen_edge_fade(hit_uv);
        let confidence = saturate(distance_fade * edge_fade * params.intensity);

        return vec4<f32>(reflected_color * confidence, confidence);
    }

    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}
