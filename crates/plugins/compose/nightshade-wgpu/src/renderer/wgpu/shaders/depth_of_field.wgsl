struct DoFParams {
    focus_distance: f32,
    focus_range: f32,
    max_blur_radius: f32,
    bokeh_threshold: f32,
    bokeh_intensity: f32,
    near_plane: f32,
    far_plane: f32,
    sample_count: u32,
    texture_size: vec2<f32>,
    tilt_shift_enabled: u32,
    tilt_shift_angle: f32,
    tilt_shift_center: f32,
    tilt_shift_blur_amount: f32,
    visualize_tilt_shift: u32,
    _padding: u32,
}

@group(0) @binding(0) var color_texture: texture_2d<f32>;
@group(0) @binding(1) var depth_texture: texture_depth_2d;
@group(0) @binding(2) var color_sampler: sampler;
@group(0) @binding(3) var<uniform> params: DoFParams;

fn sample_depth(uv: vec2<f32>) -> f32 {
    let dims = textureDimensions(depth_texture);
    let max_coord = vec2<i32>(dims) - vec2<i32>(1, 1);
    let tex_coord = clamp(vec2<i32>(uv * vec2<f32>(dims)), vec2<i32>(0, 0), max_coord);
    return textureLoad(depth_texture, tex_coord, 0);
}

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

fn linearize_depth(depth: f32) -> f32 {
    let near = params.near_plane;
    let far = params.far_plane;
    return near * far / (near + depth * (far - near));
}

fn calculate_coc(linear_depth: f32) -> f32 {
    let distance_from_focus = abs(linear_depth - params.focus_distance);
    let coc = clamp(distance_from_focus / params.focus_range, 0.0, 1.0);
    return coc * params.max_blur_radius;
}

fn calculate_tilt_shift_coc(uv: vec2<f32>) -> f32 {
    let aspect_ratio = params.texture_size.x / params.texture_size.y;
    let centered_y = (uv.y - 0.5) * 2.0 - params.tilt_shift_center;
    let centered_x = (uv.x - 0.5) * 2.0 * aspect_ratio;

    let cos_angle = cos(params.tilt_shift_angle);
    let sin_angle = sin(params.tilt_shift_angle);
    let distance_from_focus_line = abs(centered_y * cos_angle - centered_x * sin_angle);

    let focus_half_width = clamp(params.focus_range * 0.02, 0.01, 0.8);
    let falloff_distance = 1.0;
    let distance_outside_band = max(distance_from_focus_line - focus_half_width, 0.0);
    let normalized_distance = clamp(distance_outside_band / falloff_distance, 0.0, 1.0);

    let blur = normalized_distance * normalized_distance * params.max_blur_radius * params.tilt_shift_blur_amount;
    return blur;
}

fn get_disk_offset(index: u32, total: u32) -> vec2<f32> {
    let golden_angle = 2.39996323;
    let angle = f32(index) * golden_angle;
    let radius = sqrt(f32(index) + 0.5) / sqrt(f32(total));
    return vec2<f32>(cos(angle), sin(angle)) * radius;
}

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn get_tilt_shift_visualization(uv: vec2<f32>, scene_color: vec3<f32>) -> vec3<f32> {
    let aspect_ratio = params.texture_size.x / params.texture_size.y;
    let centered_y = (uv.y - 0.5) * 2.0 - params.tilt_shift_center;
    let centered_x = (uv.x - 0.5) * 2.0 * aspect_ratio;

    let cos_angle = cos(params.tilt_shift_angle);
    let sin_angle = sin(params.tilt_shift_angle);
    let distance_from_focus_line = abs(centered_y * cos_angle - centered_x * sin_angle);

    let focus_half_width = clamp(params.focus_range * 0.02, 0.01, 0.8);
    let falloff_distance = 1.0;
    let distance_outside_band = max(distance_from_focus_line - focus_half_width, 0.0);
    let normalized_distance = clamp(distance_outside_band / falloff_distance, 0.0, 1.0);

    let in_focus = vec3<f32>(0.0, 1.0, 0.0);
    let transition = vec3<f32>(1.0, 1.0, 0.0);
    let out_of_focus = vec3<f32>(1.0, 0.0, 0.0);

    var overlay_color: vec3<f32>;
    if distance_from_focus_line < focus_half_width {
        overlay_color = in_focus;
    } else if normalized_distance < 0.5 {
        let t = normalized_distance / 0.5;
        overlay_color = mix(in_focus, transition, t);
    } else {
        let t = (normalized_distance - 0.5) / 0.5;
        overlay_color = mix(transition, out_of_focus, t);
    }

    return mix(scene_color, overlay_color, 0.5);
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let center_color = textureSampleLevel(color_texture, color_sampler, in.uv, 0.0);
    let center_depth_raw = sample_depth(in.uv);

    var center_coc: f32;
    var center_depth: f32 = 0.0;
    let use_tilt_shift = params.tilt_shift_enabled != 0u;
    let visualize_ts = params.visualize_tilt_shift != 0u;

    if visualize_ts && use_tilt_shift {
        let vis_color = get_tilt_shift_visualization(in.uv, center_color.rgb);
        return vec4<f32>(vis_color, center_color.a);
    }

    if use_tilt_shift {
        center_coc = calculate_tilt_shift_coc(in.uv);
    } else {
        if center_depth_raw >= 1.0 {
            return center_color;
        }
        center_depth = linearize_depth(center_depth_raw);
        center_coc = calculate_coc(center_depth);
    }

    if center_coc < 0.5 {
        return center_color;
    }

    let color_dims = textureDimensions(color_texture);
    let texel_size = 1.0 / vec2<f32>(color_dims);
    let sample_count = params.sample_count;

    var color_sum = vec3<f32>(0.0);
    var weight_sum = 0.0;
    var bokeh_sum = vec3<f32>(0.0);
    var bokeh_weight_sum = 0.0;

    for (var index = 0u; index < sample_count; index = index + 1u) {
        let offset = get_disk_offset(index, sample_count);
        let sample_offset = offset * center_coc * texel_size;
        let sample_uv = in.uv + sample_offset;

        if sample_uv.x < 0.0 || sample_uv.x > 1.0 || sample_uv.y < 0.0 || sample_uv.y > 1.0 {
            continue;
        }

        let sample_color = textureSampleLevel(color_texture, color_sampler, sample_uv, 0.0).rgb;

        var sample_coc: f32;
        var sample_depth_val: f32 = params.far_plane;

        if use_tilt_shift {
            sample_coc = calculate_tilt_shift_coc(sample_uv);
        } else {
            let sample_depth_raw = sample_depth(sample_uv);
            if sample_depth_raw < 1.0 {
                sample_depth_val = linearize_depth(sample_depth_raw);
            }
            sample_coc = calculate_coc(sample_depth_val);
        }

        let distance_factor = 1.0 - length(offset);
        var weight: f32;
        if use_tilt_shift {
            weight = distance_factor;
        } else {
            let depth_weight = select(1.0, smoothstep(0.0, center_coc * 0.5, sample_coc), sample_depth_val < center_depth);
            weight = distance_factor * depth_weight;
        }

        color_sum = color_sum + sample_color * weight;
        weight_sum = weight_sum + weight;

        let sample_lum = luminance(sample_color);
        if sample_lum > params.bokeh_threshold {
            let bokeh_weight = (sample_lum - params.bokeh_threshold) * weight * sample_coc;
            bokeh_sum = bokeh_sum + sample_color * bokeh_weight;
            bokeh_weight_sum = bokeh_weight_sum + bokeh_weight;
        }
    }

    var blurred_color = center_color.rgb;
    if weight_sum > 0.0 {
        blurred_color = color_sum / weight_sum;
    }

    if bokeh_weight_sum > 0.0 {
        let bokeh_color = bokeh_sum / bokeh_weight_sum;
        let bokeh_strength = min(bokeh_weight_sum * params.bokeh_intensity * 0.1, 1.0);
        blurred_color = mix(blurred_color, bokeh_color, bokeh_strength);
    }

    let blend_factor = smoothstep(0.0, params.max_blur_radius * 0.3, center_coc);
    let final_color = mix(center_color.rgb, blurred_color, blend_factor);

    return vec4<f32>(final_color, center_color.a);
}

@fragment
fn fragment_coc_visualization(in: VertexOutput) -> @location(0) vec4<f32> {
    var coc: f32;
    let use_tilt_shift = params.tilt_shift_enabled != 0u;

    if use_tilt_shift {
        coc = calculate_tilt_shift_coc(in.uv);
    } else {
        let depth_raw = sample_depth(in.uv);
        if depth_raw >= 1.0 {
            return vec4<f32>(0.0, 0.0, 1.0, 1.0);
        }
        let linear_depth = linearize_depth(depth_raw);
        coc = calculate_coc(linear_depth);
    }

    let normalized_coc = coc / params.max_blur_radius;

    let in_focus = vec3<f32>(0.0, 1.0, 0.0);
    let out_of_focus = vec3<f32>(1.0, 0.0, 0.0);
    let color = mix(in_focus, out_of_focus, normalized_coc);

    return vec4<f32>(color, 1.0);
}
