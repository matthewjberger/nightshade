struct TaaParams {
    inv_view_proj: mat4x4<f32>,
    prev_view_proj: mat4x4<f32>,
    resolution: vec2<f32>,
    history_valid: f32,
    blend: f32,
    sharpness: f32,
    use_velocity: f32,
    input_resolution: vec2<f32>,
}

@group(0) @binding(0) var current_texture: texture_2d<f32>;
@group(0) @binding(1) var history_texture: texture_2d<f32>;
@group(0) @binding(2) var depth_texture: texture_depth_2d;
@group(0) @binding(3) var linear_sampler: sampler;
@group(0) @binding(4) var point_sampler: sampler;
@group(0) @binding(5) var<uniform> params: TaaParams;
@group(0) @binding(6) var velocity_texture: texture_2d<f32>;

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

struct FragmentOutput {
    @location(0) display: vec4<f32>,
    @location(1) history: vec4<f32>,
}

fn rgb_to_ycocg(color: vec3<f32>) -> vec3<f32> {
    let luma = dot(color, vec3<f32>(0.25, 0.5, 0.25));
    let orange = dot(color, vec3<f32>(0.5, 0.0, -0.5));
    let green = dot(color, vec3<f32>(-0.25, 0.5, -0.25));
    return vec3<f32>(luma, orange, green);
}

fn ycocg_to_rgb(color: vec3<f32>) -> vec3<f32> {
    let luma = color.x;
    let orange = color.y;
    let green = color.z;
    return vec3<f32>(luma + orange - green, luma + green, luma - orange - green);
}

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn clip_to_aabb(aabb_min: vec3<f32>, aabb_max: vec3<f32>, history: vec3<f32>) -> vec3<f32> {
    let center = 0.5 * (aabb_max + aabb_min);
    let extent = 0.5 * (aabb_max - aabb_min) + vec3<f32>(1e-5);
    let offset = history - center;
    let unit = offset / extent;
    let absolute = abs(unit);
    let largest = max(absolute.x, max(absolute.y, absolute.z));
    if largest > 1.0 {
        return center + offset / largest;
    }
    return history;
}

fn sample_history_catmull_rom(uv: vec2<f32>) -> vec3<f32> {
    let resolution = params.resolution;
    let sample_position = uv * resolution;
    let tex_pos1 = floor(sample_position - 0.5) + 0.5;
    let f = sample_position - tex_pos1;

    let w0 = f * (-0.5 + f * (1.0 - 0.5 * f));
    let w1 = 1.0 + f * f * (-2.5 + 1.5 * f);
    let w2 = f * (0.5 + f * (2.0 - 1.5 * f));
    let w3 = f * f * (-0.5 + 0.5 * f);
    let w12 = w1 + w2;
    let offset12 = w2 / w12;

    let inv_resolution = 1.0 / resolution;
    let pos0 = (tex_pos1 - 1.0) * inv_resolution;
    let pos3 = (tex_pos1 + 2.0) * inv_resolution;
    let pos12 = (tex_pos1 + offset12) * inv_resolution;

    var result = vec3<f32>(0.0);
    result += textureSampleLevel(history_texture, linear_sampler, vec2<f32>(pos0.x, pos0.y), 0.0).rgb * (w0.x * w0.y);
    result += textureSampleLevel(history_texture, linear_sampler, vec2<f32>(pos12.x, pos0.y), 0.0).rgb * (w12.x * w0.y);
    result += textureSampleLevel(history_texture, linear_sampler, vec2<f32>(pos3.x, pos0.y), 0.0).rgb * (w3.x * w0.y);
    result += textureSampleLevel(history_texture, linear_sampler, vec2<f32>(pos0.x, pos12.y), 0.0).rgb * (w0.x * w12.y);
    result += textureSampleLevel(history_texture, linear_sampler, vec2<f32>(pos12.x, pos12.y), 0.0).rgb * (w12.x * w12.y);
    result += textureSampleLevel(history_texture, linear_sampler, vec2<f32>(pos3.x, pos12.y), 0.0).rgb * (w3.x * w12.y);
    result += textureSampleLevel(history_texture, linear_sampler, vec2<f32>(pos0.x, pos3.y), 0.0).rgb * (w0.x * w3.y);
    result += textureSampleLevel(history_texture, linear_sampler, vec2<f32>(pos12.x, pos3.y), 0.0).rgb * (w12.x * w3.y);
    result += textureSampleLevel(history_texture, linear_sampler, vec2<f32>(pos3.x, pos3.y), 0.0).rgb * (w3.x * w3.y);
    return max(result, vec3<f32>(0.0));
}

@fragment
fn fragment_main(in: VertexOutput) -> FragmentOutput {
    let current = textureSampleLevel(current_texture, linear_sampler, in.uv, 0.0);

    var output: FragmentOutput;

    if params.history_valid < 0.5 {
        output.display = current;
        output.history = current;
        return output;
    }

    let texel = 1.0 / params.input_resolution;

    var closest_depth = 0.0;
    var closest_offset = vec2<f32>(0.0);
    for (var offset_y = -1; offset_y <= 1; offset_y += 1) {
        for (var offset_x = -1; offset_x <= 1; offset_x += 1) {
            let sample_offset = vec2<f32>(f32(offset_x), f32(offset_y));
            let sample_uv = in.uv + sample_offset * texel;
            let sampled = textureSampleLevel(depth_texture, point_sampler, sample_uv, 0);
            if sampled > closest_depth {
                closest_depth = sampled;
                closest_offset = sample_offset;
            }
        }
    }
    let dilated_uv = in.uv + closest_offset * texel;

    var prev_uv = in.uv;
    let velocity = textureSampleLevel(velocity_texture, point_sampler, dilated_uv, 0.0).xy;
    if params.use_velocity > 0.5 && abs(velocity.x) < 1.5 && abs(velocity.y) < 1.5 {
        prev_uv = in.uv + velocity;
    } else {
        let pixel_depth = textureSampleLevel(depth_texture, point_sampler, in.uv, 0);
        let reproject_depth = max(pixel_depth, 1e-6);
        let ndc = vec4<f32>(in.uv.x * 2.0 - 1.0, 1.0 - in.uv.y * 2.0, reproject_depth, 1.0);
        var world = params.inv_view_proj * ndc;
        world = world / world.w;
        let prev_clip = params.prev_view_proj * vec4<f32>(world.xyz, 1.0);
        if !(prev_clip.w > 0.0) {
            output.display = current;
            output.history = current;
            return output;
        }
        let prev_ndc = prev_clip.xyz / prev_clip.w;
        prev_uv = vec2<f32>(prev_ndc.x * 0.5 + 0.5, 0.5 - prev_ndc.y * 0.5);
    }

    let in_bounds = prev_uv.x >= 0.0 && prev_uv.x <= 1.0 && prev_uv.y >= 0.0 && prev_uv.y <= 1.0;
    if !in_bounds {
        output.display = current;
        output.history = current;
        return output;
    }

    let current_ycocg = rgb_to_ycocg(current.rgb);
    var neighborhood_min = current_ycocg;
    var neighborhood_max = current_ycocg;
    var moment_first = current_ycocg;
    var moment_second = current_ycocg * current_ycocg;
    for (var offset_y = -1; offset_y <= 1; offset_y += 1) {
        for (var offset_x = -1; offset_x <= 1; offset_x += 1) {
            if offset_x == 0 && offset_y == 0 {
                continue;
            }
            let sample_uv = in.uv + vec2<f32>(f32(offset_x), f32(offset_y)) * texel;
            let neighbor = rgb_to_ycocg(textureSampleLevel(current_texture, point_sampler, sample_uv, 0.0).rgb);
            neighborhood_min = min(neighborhood_min, neighbor);
            neighborhood_max = max(neighborhood_max, neighbor);
            moment_first += neighbor;
            moment_second += neighbor * neighbor;
        }
    }

    let inverse_count = 1.0 / 9.0;
    let mean = moment_first * inverse_count;
    let variance = max(moment_second * inverse_count - mean * mean, vec3<f32>(0.0));
    let deviation = sqrt(variance) * 1.5;
    let clip_min = max(mean - deviation, neighborhood_min);
    let clip_max = min(mean + deviation, neighborhood_max);

    let history_rgb = sample_history_catmull_rom(prev_uv);
    let history_pre = rgb_to_ycocg(history_rgb);
    let history_ycocg = clip_to_aabb(clip_min, clip_max, history_pre);
    let history = ycocg_to_rgb(history_ycocg);

    let clip_extent = max((clip_max - clip_min) * 0.5, vec3<f32>(1e-4));
    let clip_distance = length((history_pre - history_ycocg) / clip_extent);
    let blend = clamp(params.blend + clip_distance * 0.5, params.blend, 0.5);

    let current_weight = blend / (1.0 + luminance(current.rgb));
    let history_weight = (1.0 - blend) / (1.0 + luminance(history));
    let resolved = (current.rgb * current_weight + history * history_weight)
        / max(current_weight + history_weight, 1e-5);

    let low_pass = ycocg_to_rgb(mean);
    let sharpened = max(resolved + (resolved - low_pass) * params.sharpness, vec3<f32>(0.0));

    output.display = vec4<f32>(sharpened, current.a);
    output.history = vec4<f32>(resolved, current.a);
    return output;
}
