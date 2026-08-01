#version 460

#define TAU 6.28318530717958647692

struct RectInstance {
    vec4 position_size;
    vec4 color;
    vec4 border_color;
    vec4 clip_rect;
    vec4 params;
    vec4 shadow_color;
    vec4 shadow_params;
    vec4 effect_params;
    vec4 quad_corner_01;
    vec4 quad_corner_23;
    uint effect_kind;
    uint is_quad;
    uint padding_0;
    uint padding_1;
};

layout(set = 0, binding = 1) readonly buffer Instances { RectInstance instances[]; };

layout(location = 0) in vec2 local_pos;
layout(location = 1) in vec2 rect_size_in;
layout(location = 2) in vec2 screen_pos;
layout(location = 3) flat in uint slot_index;

layout(location = 0) out vec4 out_color;

float rounded_rect_sdf(vec2 position, vec2 size, float radius) {
    vec2 half_size = size * 0.5;
    vec2 center_pos = position - half_size;
    float clamped_radius = min(radius, min(half_size.x, half_size.y));
    vec2 q = abs(center_pos) - half_size + clamped_radius;
    return length(max(q, vec2(0.0))) + min(max(q.x, q.y), 0.0) - clamped_radius;
}

float edge_coverage(float signed_distance, float pixel_width) {
    return 1.0 - smoothstep(-pixel_width * 0.5, pixel_width * 0.5, signed_distance);
}

float gaussian_coverage(float signed_distance, float sigma) {
    float x = signed_distance / max(sigma, 0.00001);
    return 1.0 / (1.0 + exp(1.5976 * x + 0.070565 * x * x * x));
}

float quantization_dither(vec2 fragment_position) {
    vec3 magic = vec3(0.06711056, 0.00583715, 52.9829189);
    return fract(magic.z * fract(dot(fragment_position, magic.xy))) - 0.5;
}

vec3 linear_to_oklab(vec3 color) {
    float l = 0.4122214708 * color.r + 0.5363325363 * color.g + 0.0514459929 * color.b;
    float m = 0.2119034982 * color.r + 0.6806995451 * color.g + 0.1073969566 * color.b;
    float s = 0.0883024619 * color.r + 0.2817188376 * color.g + 0.6299787005 * color.b;
    float l_root = pow(max(l, 0.0), 1.0 / 3.0);
    float m_root = pow(max(m, 0.0), 1.0 / 3.0);
    float s_root = pow(max(s, 0.0), 1.0 / 3.0);
    return vec3(
        0.2104542553 * l_root + 0.7936177850 * m_root - 0.0040720468 * s_root,
        1.9779984951 * l_root - 2.4285922050 * m_root + 0.4505937099 * s_root,
        0.0259040371 * l_root + 0.7827717662 * m_root - 0.8086757660 * s_root
    );
}

vec3 oklab_to_linear(vec3 color) {
    float l_root = color.x + 0.3963377774 * color.y + 0.2158037573 * color.z;
    float m_root = color.x - 0.1055613458 * color.y - 0.0638541728 * color.z;
    float s_root = color.x - 0.0894841775 * color.y - 1.2914855480 * color.z;
    float l = l_root * l_root * l_root;
    float m = m_root * m_root * m_root;
    float s = s_root * s_root * s_root;
    return vec3(
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s
    );
}

vec4 mix_oklab(vec4 first, vec4 second, float amount) {
    vec3 lab_first = linear_to_oklab(first.rgb);
    vec3 lab_second = linear_to_oklab(second.rgb);
    vec3 blended = oklab_to_linear(mix(lab_first, lab_second, amount));
    return vec4(blended, mix(first.a, second.a, amount));
}

vec3 hsv_to_rgb(float hue, float saturation, float value) {
    float chroma = value * saturation;
    float hue_segment = clamp(hue, 0.0, 1.0) * 6.0;
    float secondary = chroma * (1.0 - abs(mod(hue_segment, 2.0) - 1.0));
    vec3 rgb = vec3(0.0);
    if (hue_segment < 1.0) {
        rgb = vec3(chroma, secondary, 0.0);
    } else if (hue_segment < 2.0) {
        rgb = vec3(secondary, chroma, 0.0);
    } else if (hue_segment < 3.0) {
        rgb = vec3(0.0, chroma, secondary);
    } else if (hue_segment < 4.0) {
        rgb = vec3(0.0, secondary, chroma);
    } else if (hue_segment < 5.0) {
        rgb = vec3(secondary, 0.0, chroma);
    } else {
        rgb = vec3(chroma, 0.0, secondary);
    }
    float match_value = value - chroma;
    return rgb + vec3(match_value);
}

vec4 apply_effect(vec2 position, vec2 size, vec4 base, uint effect_kind, vec4 params) {
    if (effect_kind == 1u) {
        float amount = clamp(position.y / max(size.y, 1.0), 0.0, 1.0);
        return mix_oklab(base, params, amount);
    }
    if (effect_kind == 2u) {
        float amount = clamp(position.x / max(size.x, 1.0), 0.0, 1.0);
        return mix_oklab(base, params, amount);
    }
    if (effect_kind == 3u) {
        vec2 center = size * 0.5;
        float half_diagonal = max(length(center), 1.0);
        float distance_to_center = length(position - center) / half_diagonal;
        vec4 glow = vec4(params.xyz, base.a);
        return mix(base, glow, smoothstep(0.0, 0.85, distance_to_center) * params.w);
    }
    if (effect_kind == 4u) {
        float strength = params.x;
        float edge = max(params.y, 0.5);
        float top_edge = smoothstep(0.0, edge, position.y);
        float left_edge = smoothstep(0.0, edge, position.x);
        float bottom_edge = smoothstep(0.0, edge, size.y - position.y);
        float right_edge = smoothstep(0.0, edge, size.x - position.x);
        float inset_dark = (1.0 - top_edge) * 0.6 + (1.0 - left_edge) * 0.4;
        float inset_light = (1.0 - bottom_edge) * 0.4 + (1.0 - right_edge) * 0.3;
        vec3 darkened = base.rgb * (1.0 - inset_dark * strength);
        return vec4(darkened + inset_light * strength * 0.5, base.a);
    }
    if (effect_kind == 5u) {
        float strength = params.x;
        float edge = max(params.y, 0.5);
        float top_edge = smoothstep(0.0, edge, position.y);
        float left_edge = smoothstep(0.0, edge, position.x);
        float bottom_edge = smoothstep(0.0, edge, size.y - position.y);
        float right_edge = smoothstep(0.0, edge, size.x - position.x);
        float outset_light = (1.0 - top_edge) * 0.5 + (1.0 - left_edge) * 0.3;
        float outset_dark = (1.0 - bottom_edge) * 0.5 + (1.0 - right_edge) * 0.3;
        vec3 lightened = base.rgb + outset_light * strength * 0.6;
        return vec4(lightened * (1.0 - outset_dark * strength * 0.4), base.a);
    }
    if (effect_kind == 6u) {
        float highlight_band = smoothstep(0.0, max(size.y * 0.5, 1.0), size.y - position.y);
        return mix(params, vec4(1.0, 1.0, 1.0, params.a * 0.18), highlight_band * 0.25);
    }
    if (effect_kind == 8u) {
        float normalized_x = position.x / max(size.x, 1.0);
        float half_height = max(size.y * 0.5, 1.0);
        float normalized_y = abs(position.y - size.y * 0.5) / half_height;
        float edge = normalized_x + normalized_y;
        return vec4(base.rgb, base.a * (1.0 - smoothstep(0.96, 1.04, edge)));
    }
    if (effect_kind == 7u) {
        float saturation = clamp(position.x / max(size.x, 1.0), 0.0, 1.0);
        float value = clamp(1.0 - position.y / max(size.y, 1.0), 0.0, 1.0);
        return vec4(hsv_to_rgb(clamp(params.x, 0.0, 1.0), saturation, value), base.a);
    }
    if (effect_kind == 9u) {
        vec2 center = size * 0.5;
        float radius = max(min(center.x, center.y), 1.0);
        vec2 offset = position - center;
        float distance_to_center = length(offset);
        float saturation = clamp(distance_to_center / radius, 0.0, 1.0);
        float hue = fract(atan(offset.y, offset.x) / TAU + 1.0);
        float disc = 1.0 - smoothstep(radius - 1.0, radius, distance_to_center);
        vec3 rgb = hsv_to_rgb(hue, saturation, clamp(params.x, 0.0, 1.0));
        return vec4(rgb, base.a * disc);
    }
    return base;
}

void main() {
    RectInstance instance = instances[slot_index];

    float edge_distance = rounded_rect_sdf(local_pos, rect_size_in, instance.params.x);
    float pixel_width = max(fwidth(edge_distance), 0.00001);

    vec4 clip = instance.clip_rect;
    if (clip.z > clip.x
        && (screen_pos.x < clip.x || screen_pos.x > clip.z
            || screen_pos.y < clip.y || screen_pos.y > clip.w)) {
        discard;
    }

    if (instance.is_quad != 0u) {
        if (instance.color.a < 0.001) {
            discard;
        }
        out_color = instance.color;
        return;
    }

    float corner_radius = instance.params.x;
    float border_width = instance.params.y;

    vec4 rect_color = apply_effect(
        local_pos,
        rect_size_in,
        instance.color,
        instance.effect_kind,
        instance.effect_params
    );
    rect_color = vec4(
        rect_color.rgb + quantization_dither(gl_FragCoord.xy) / 255.0,
        rect_color.a
    );
    float rect_alpha = edge_coverage(edge_distance, pixel_width);

    if (border_width > 0.0) {
        float is_border = rect_alpha - edge_coverage(edge_distance + border_width, pixel_width);
        rect_color = mix(rect_color, instance.border_color, is_border);
    }

    float rect_effective_alpha = rect_alpha * rect_color.a;

    vec4 shadow_color = instance.shadow_color;
    if (shadow_color.a > 0.0) {
        vec2 shadow_offset = instance.shadow_params.xy;
        float shadow_blur = instance.shadow_params.z;
        float shadow_spread = instance.shadow_params.w;

        vec2 shadow_local = local_pos - shadow_offset + vec2(shadow_spread);
        vec2 shadow_size = rect_size_in + vec2(2.0 * shadow_spread);
        float shadow_radius = corner_radius + shadow_spread;
        float shadow_distance = rounded_rect_sdf(shadow_local, shadow_size, shadow_radius);
        float sigma = max(shadow_blur, 0.5) * 0.5;
        float shadow_effective_alpha = gaussian_coverage(shadow_distance, sigma) * shadow_color.a;

        float visible_shadow_alpha = shadow_effective_alpha * (1.0 - rect_alpha);
        float total_alpha = rect_effective_alpha + visible_shadow_alpha;
        if (total_alpha < 0.001) {
            discard;
        }
        vec3 total_rgb =
            rect_color.rgb * rect_effective_alpha + shadow_color.rgb * visible_shadow_alpha;
        out_color = vec4(total_rgb / total_alpha, total_alpha);
        return;
    }

    if (rect_effective_alpha < 0.001) {
        discard;
    }
    out_color = vec4(rect_color.rgb, rect_effective_alpha);
}
