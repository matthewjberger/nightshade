struct GlobalUniforms {
    projection: mat4x4<f32>,
}

struct RectInstance {
    position_size: vec4<f32>,
    color: vec4<f32>,
    border_color: vec4<f32>,
    clip_rect: vec4<f32>,
    params: vec4<f32>,
    shadow_color: vec4<f32>,
    shadow_params: vec4<f32>,
    effect_params: vec4<f32>,
    quad_corner_01: vec4<f32>,
    quad_corner_23: vec4<f32>,
    effect_kind: u32,
    is_quad: u32,
    _padding0: u32,
    _padding1: u32,
}

@group(0) @binding(0) var<uniform> globals: GlobalUniforms;
@group(1) @binding(0) var<storage, read> instances: array<RectInstance>;
@group(1) @binding(1) var<storage, read> draw_order: array<u32>;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @builtin(instance_index) instance_index: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) rect_size: vec2<f32>,
    @location(2) screen_pos: vec2<f32>,
    @location(3) @interpolate(flat) slot_index: u32,
}

fn shadow_margin(inst: RectInstance) -> f32 {
    let offset = inst.shadow_params.xy;
    let blur = inst.shadow_params.z;
    let spread = inst.shadow_params.w;
    if inst.shadow_color.a <= 0.0 {
        return 0.0;
    }
    return blur * 1.5 + spread + max(abs(offset.x), abs(offset.y));
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let slot = draw_order[vertex.instance_index];
    let inst = instances[slot];

    let rect_pos = inst.position_size.xy;
    let rect_size = inst.position_size.zw;

    var world_pos: vec2<f32>;
    var local: vec2<f32>;
    if inst.is_quad != 0u {
        let p0 = inst.quad_corner_01.xy;
        let p1 = inst.quad_corner_01.zw;
        let p2 = inst.quad_corner_23.xy;
        let p3 = inst.quad_corner_23.zw;
        let top = mix(p0, p1, vertex.position.x);
        let bottom = mix(p3, p2, vertex.position.x);
        world_pos = mix(top, bottom, vertex.position.y);
        local = vertex.position * rect_size;
    } else {
        let margin = shadow_margin(inst);
        let extended_size = rect_size + vec2<f32>(2.0 * margin, 2.0 * margin);
        local = vertex.position * extended_size - vec2<f32>(margin, margin);

        let rotation = inst.params.w;
        let center = rect_size * 0.5;
        let centered = local - center;
        let cos_r = cos(rotation);
        let sin_r = sin(rotation);
        let rotated = vec2<f32>(
            centered.x * cos_r - centered.y * sin_r,
            centered.x * sin_r + centered.y * cos_r,
        );
        world_pos = rect_pos + center + rotated;
    }

    output.position = globals.projection * vec4<f32>(world_pos, 0.0, 1.0);
    output.position.z = inst.params.z;
    output.local_pos = local;
    output.rect_size = rect_size;
    output.screen_pos = world_pos;
    output.slot_index = slot;

    return output;
}

fn rounded_rect_sdf(pos: vec2<f32>, size: vec2<f32>, radius: f32) -> f32 {
    let half_size = size * 0.5;
    let center_pos = pos - half_size;
    let clamped_radius = min(radius, min(half_size.x, half_size.y));
    let q = abs(center_pos) - half_size + clamped_radius;
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - clamped_radius;
}

fn edge_coverage(signed_distance: f32, pixel_width: f32) -> f32 {
    return 1.0 - smoothstep(-pixel_width * 0.5, pixel_width * 0.5, signed_distance);
}

fn gaussian_coverage(signed_distance: f32, sigma: f32) -> f32 {
    let x = signed_distance / max(sigma, 0.00001);
    return 1.0 / (1.0 + exp(1.5976 * x + 0.070565 * x * x * x));
}

fn quantization_dither(fragment_position: vec2<f32>) -> f32 {
    let magic = vec3<f32>(0.06711056, 0.00583715, 52.9829189);
    return fract(magic.z * fract(dot(fragment_position, magic.xy))) - 0.5;
}

fn linear_to_oklab(c: vec3<f32>) -> vec3<f32> {
    let l = 0.4122214708 * c.r + 0.5363325363 * c.g + 0.0514459929 * c.b;
    let m = 0.2119034982 * c.r + 0.6806995451 * c.g + 0.1073969566 * c.b;
    let s = 0.0883024619 * c.r + 0.2817188376 * c.g + 0.6299787005 * c.b;
    let l_ = pow(max(l, 0.0), 1.0 / 3.0);
    let m_ = pow(max(m, 0.0), 1.0 / 3.0);
    let s_ = pow(max(s, 0.0), 1.0 / 3.0);
    return vec3<f32>(
        0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
        1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
        0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
    );
}

fn oklab_to_linear(c: vec3<f32>) -> vec3<f32> {
    let l_ = c.x + 0.3963377774 * c.y + 0.2158037573 * c.z;
    let m_ = c.x - 0.1055613458 * c.y - 0.0638541728 * c.z;
    let s_ = c.x - 0.0894841775 * c.y - 1.2914855480 * c.z;
    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;
    return vec3<f32>(
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    );
}

fn mix_oklab(a: vec4<f32>, b: vec4<f32>, t: f32) -> vec4<f32> {
    let lab_a = linear_to_oklab(a.rgb);
    let lab_b = linear_to_oklab(b.rgb);
    let blended = oklab_to_linear(mix(lab_a, lab_b, t));
    let alpha = mix(a.a, b.a, t);
    return vec4<f32>(blended, alpha);
}

const TAU: f32 = 6.28318530717958647692;

fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> vec3<f32> {
    let chroma = value * saturation;
    let hue_segment = clamp(hue, 0.0, 1.0) * 6.0;
    let secondary = chroma * (1.0 - abs((hue_segment % 2.0) - 1.0));
    var rgb = vec3<f32>(0.0, 0.0, 0.0);
    if hue_segment < 1.0 {
        rgb = vec3<f32>(chroma, secondary, 0.0);
    } else if hue_segment < 2.0 {
        rgb = vec3<f32>(secondary, chroma, 0.0);
    } else if hue_segment < 3.0 {
        rgb = vec3<f32>(0.0, chroma, secondary);
    } else if hue_segment < 4.0 {
        rgb = vec3<f32>(0.0, secondary, chroma);
    } else if hue_segment < 5.0 {
        rgb = vec3<f32>(secondary, 0.0, chroma);
    } else {
        rgb = vec3<f32>(chroma, 0.0, secondary);
    }
    let match_value = value - chroma;
    return rgb + vec3<f32>(match_value, match_value, match_value);
}

fn apply_effect(
    local_pos: vec2<f32>,
    rect_size: vec2<f32>,
    base: vec4<f32>,
    effect_kind: u32,
    params: vec4<f32>,
) -> vec4<f32> {
    if effect_kind == 1u {
        let denom = max(rect_size.y, 1.0);
        let t = clamp(local_pos.y / denom, 0.0, 1.0);
        let secondary = vec4<f32>(params.x, params.y, params.z, params.w);
        return mix_oklab(base, secondary, t);
    }
    if effect_kind == 2u {
        let denom = max(rect_size.x, 1.0);
        let t = clamp(local_pos.x / denom, 0.0, 1.0);
        let secondary = vec4<f32>(params.x, params.y, params.z, params.w);
        return mix_oklab(base, secondary, t);
    }
    if effect_kind == 3u {
        let center = rect_size * 0.5;
        let half_diag = max(length(center), 1.0);
        let dist = length(local_pos - center) / half_diag;
        let glow = vec4<f32>(params.x, params.y, params.z, base.a);
        let intensity = params.w;
        return mix(base, glow, smoothstep(0.0, 0.85, dist) * intensity);
    }
    if effect_kind == 4u {
        let strength = params.x;
        let edge = max(params.y, 0.5);
        let top_edge = smoothstep(0.0, edge, local_pos.y);
        let left_edge = smoothstep(0.0, edge, local_pos.x);
        let bottom_edge = smoothstep(0.0, edge, rect_size.y - local_pos.y);
        let right_edge = smoothstep(0.0, edge, rect_size.x - local_pos.x);
        let inset_dark = (1.0 - top_edge) * 0.6 + (1.0 - left_edge) * 0.4;
        let inset_light = (1.0 - bottom_edge) * 0.4 + (1.0 - right_edge) * 0.3;
        let darkened = base.rgb * (1.0 - inset_dark * strength);
        let lightened = darkened + inset_light * strength * 0.5;
        return vec4<f32>(lightened, base.a);
    }
    if effect_kind == 5u {
        let strength = params.x;
        let edge = max(params.y, 0.5);
        let top_edge = smoothstep(0.0, edge, local_pos.y);
        let left_edge = smoothstep(0.0, edge, local_pos.x);
        let bottom_edge = smoothstep(0.0, edge, rect_size.y - local_pos.y);
        let right_edge = smoothstep(0.0, edge, rect_size.x - local_pos.x);
        let outset_light = (1.0 - top_edge) * 0.5 + (1.0 - left_edge) * 0.3;
        let outset_dark = (1.0 - bottom_edge) * 0.5 + (1.0 - right_edge) * 0.3;
        let lightened = base.rgb + outset_light * strength * 0.6;
        let darkened = lightened * (1.0 - outset_dark * strength * 0.4);
        return vec4<f32>(darkened, base.a);
    }
    if effect_kind == 6u {
        let tint = vec4<f32>(params.x, params.y, params.z, params.w);
        let highlight_band = smoothstep(0.0, max(rect_size.y * 0.5, 1.0), rect_size.y - local_pos.y);
        let frosted = mix(tint, vec4<f32>(1.0, 1.0, 1.0, tint.a * 0.18), highlight_band * 0.25);
        return frosted;
    }
    if effect_kind == 8u {
        let nx = local_pos.x / max(rect_size.x, 1.0);
        let half_h = max(rect_size.y * 0.5, 1.0);
        let ny = abs(local_pos.y - rect_size.y * 0.5) / half_h;
        let edge = nx + ny;
        let triangle_alpha = 1.0 - smoothstep(0.96, 1.04, edge);
        return vec4<f32>(base.rgb, base.a * triangle_alpha);
    }
    if effect_kind == 7u {
        let saturation = clamp(local_pos.x / max(rect_size.x, 1.0), 0.0, 1.0);
        let value = clamp(1.0 - local_pos.y / max(rect_size.y, 1.0), 0.0, 1.0);
        return vec4<f32>(hsv_to_rgb(clamp(params.x, 0.0, 1.0), saturation, value), base.a);
    }
    if effect_kind == 9u {
        let center = rect_size * 0.5;
        let radius = max(min(center.x, center.y), 1.0);
        let offset = local_pos - center;
        let distance = length(offset);
        let saturation = clamp(distance / radius, 0.0, 1.0);
        let hue = fract(atan2(offset.y, offset.x) / TAU + 1.0);
        let disc = 1.0 - smoothstep(radius - 1.0, radius, distance);
        let rgb = hsv_to_rgb(hue, saturation, clamp(params.x, 0.0, 1.0));
        return vec4<f32>(rgb, base.a * disc);
    }
    return base;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let inst = instances[input.slot_index];

    let edge_distance = rounded_rect_sdf(input.local_pos, input.rect_size, inst.params.x);
    let pixel_width = max(fwidth(edge_distance), 0.00001);

    let clip = inst.clip_rect;
    if clip.z > clip.x {
        let frag_pos = input.screen_pos;
        if frag_pos.x < clip.x || frag_pos.x > clip.z ||
           frag_pos.y < clip.y || frag_pos.y > clip.w {
            discard;
        }
    }

    if inst.is_quad != 0u {
        if inst.color.a < 0.001 {
            discard;
        }
        return inst.color;
    }

    let corner_radius = inst.params.x;
    let border_width = inst.params.y;
    let rect_dist = edge_distance;

    var rect_color = apply_effect(
        input.local_pos,
        input.rect_size,
        inst.color,
        inst.effect_kind,
        inst.effect_params,
    );
    rect_color = vec4<f32>(
        rect_color.rgb + quantization_dither(input.position.xy) / 255.0,
        rect_color.a,
    );
    let rect_alpha = edge_coverage(rect_dist, pixel_width);

    if border_width > 0.0 {
        let is_border = rect_alpha - edge_coverage(rect_dist + border_width, pixel_width);
        rect_color = mix(rect_color, inst.border_color, is_border);
    }

    let rect_eff_a = rect_alpha * rect_color.a;

    let shadow_color = inst.shadow_color;
    if shadow_color.a > 0.0 {
        let shadow_offset = inst.shadow_params.xy;
        let shadow_blur = inst.shadow_params.z;
        let shadow_spread = inst.shadow_params.w;

        let shadow_local = input.local_pos - shadow_offset + vec2<f32>(shadow_spread, shadow_spread);
        let shadow_size = input.rect_size + vec2<f32>(2.0 * shadow_spread, 2.0 * shadow_spread);
        let shadow_radius = corner_radius + shadow_spread;
        let shadow_dist = rounded_rect_sdf(shadow_local, shadow_size, shadow_radius);
        let sigma = max(shadow_blur, 0.5) * 0.5;
        let shadow_eff_a = gaussian_coverage(shadow_dist, sigma) * shadow_color.a;

        let visible_shadow_a = shadow_eff_a * (1.0 - rect_alpha);
        let total_a = rect_eff_a + visible_shadow_a;
        if total_a < 0.001 {
            discard;
        }
        let total_rgb = rect_color.rgb * rect_eff_a + shadow_color.rgb * visible_shadow_a;
        return vec4<f32>(total_rgb / total_a, total_a);
    }

    if rect_eff_a < 0.001 {
        discard;
    }
    return vec4<f32>(rect_color.rgb, rect_eff_a);
}
