#version 460

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform sampler2D scene;
layout(set = 0, binding = 1) uniform sampler2D bloom;
layout(set = 0, binding = 2) uniform sampler2D occlusion;
layout(set = 0, binding = 3) uniform sampler2D reflection;
layout(set = 0, binding = 4) uniform sampler2D indirect;
layout(set = 0, binding = 5) uniform sampler2D resolved;
layout(set = 0, binding = 6) uniform sampler2D focused;
layout(set = 0, binding = 8) uniform sampler3D color_lut;
layout(std430, set = 0, binding = 7) readonly buffer Settled {
    float settled_log_luminance;
    float measured_log_luminance;
    uint measured;
    uint padding;
} settled;

layout(push_constant) uniform Params {
    float exposure;
    float gamma;
    float saturation;
    float brightness;
    float contrast;
    float vibrance;
    float bloom_intensity;
    float vignette_intensity;
    float vignette_radius;
    float vignette_smoothness;
    uint tonemap;
    uint bloom_enabled;
    uint occlusion_enabled;
    uint occlusion_only;
    uint reflection_enabled;
    uint indirect_enabled;
    float indirect_intensity;
    uint resolved_enabled;
    uint focus_enabled;
    float auto_exposure_target;
    float color_lut_weight;
    float chromatic_aberration;
    uint world_enabled;
    float exposure_compensation;
    float auto_exposure_lowest;
    float auto_exposure_highest;
    uint output_encoding;
    float paper_white;
    float headroom;
} params;

#define OUTPUT_SRGB 0u
#define OUTPUT_SCRGB 1u
#define OUTPUT_PQ 2u

const float SCRGB_WHITE = 80.0;
const float PQ_PEAK = 10000.0;

vec3 rec709_to_rec2020(vec3 color) {
    const mat3 widen = mat3(
        0.6274, 0.0691, 0.0164,
        0.3293, 0.9195, 0.0880,
        0.0433, 0.0114, 0.8956
    );
    return widen * color;
}

vec3 pq_encode(vec3 nits) {
    const float m1 = 0.1593017578125;
    const float m2 = 78.84375;
    const float c1 = 0.8359375;
    const float c2 = 18.8515625;
    const float c3 = 18.6875;
    vec3 normalized = pow(clamp(nits / PQ_PEAK, 0.0, 1.0), vec3(m1));
    return pow((c1 + c2 * normalized) / (1.0 + c3 * normalized), vec3(m2));
}

vec3 encoded_for_the_screen(vec3 graded) {
    if (params.output_encoding == OUTPUT_SRGB) {
        return clamp(graded, 0.0, 1.0);
    }
    vec3 linear = pow(max(graded, vec3(0.0)), vec3(params.gamma));
    if (params.output_encoding == OUTPUT_SCRGB) {
        return linear * params.paper_white / SCRGB_WHITE;
    }
    return pq_encode(rec709_to_rec2020(linear) * params.paper_white);
}

#define TONEMAP_ACES 0u
#define TONEMAP_REINHARD 1u
#define TONEMAP_REINHARD_EXTENDED 2u
#define TONEMAP_UNCHARTED2 3u
#define TONEMAP_AGX 4u
#define TONEMAP_NEUTRAL 5u
#define TONEMAP_NONE 6u
#define TONEMAP_ACES2 7u

const vec3 LUMINANCE_WEIGHTS = vec3(0.2126, 0.7152, 0.0722);

vec3 tonemap_aces(vec3 color) {
    const float a = 2.51;
    const float b = 0.03;
    const float c = 2.43;
    const float d = 0.59;
    const float e = 0.14;
    return clamp((color * (a * color + b)) / (color * (c * color + d) + e), 0.0, 1.0);
}

vec3 tonemap_aces2(vec3 color) {
    const mat3 input_matrix = mat3(
        vec3(0.59719, 0.07600, 0.02840),
        vec3(0.35458, 0.90834, 0.13383),
        vec3(0.04823, 0.01566, 0.83777)
    );
    const mat3 output_matrix = mat3(
        vec3(1.60475, -0.10208, -0.00327),
        vec3(-0.53108, 1.10813, -0.07276),
        vec3(-0.07367, -0.00605, 1.07602)
    );
    vec3 value = input_matrix * (color / 0.6);
    vec3 numerator = value * (value + 0.0245786) - 0.000090537;
    vec3 denominator = value * (0.983729 * value + 0.4329510) + 0.238081;
    return clamp(output_matrix * (numerator / denominator), 0.0, 1.0);
}

vec3 tonemap_reinhard(vec3 color) {
    return color / (color + vec3(1.0));
}

vec3 tonemap_reinhard_extended(vec3 color) {
    const float max_white = 4.0;
    return (color * (1.0 + color / (max_white * max_white))) / (1.0 + color);
}

vec3 uncharted2_partial(vec3 value) {
    const float a = 0.15;
    const float b = 0.50;
    const float c = 0.10;
    const float d = 0.20;
    const float e = 0.02;
    const float f = 0.30;
    return ((value * (a * value + c * b) + d * e) / (value * (a * value + b) + d * f)) - e / f;
}

vec3 tonemap_uncharted2(vec3 color) {
    return uncharted2_partial(color * 2.0) / uncharted2_partial(vec3(11.2));
}

vec3 agx_contrast(vec3 value) {
    vec3 squared = value * value;
    vec3 fourth = squared * squared;
    return 15.5 * fourth * squared
        - 40.14 * fourth * value
        + 31.96 * fourth
        - 6.868 * squared * value
        + 0.4298 * squared
        + 0.1191 * value
        - 0.00232;
}

vec3 tonemap_agx(vec3 color) {
    const mat3 forward = mat3(
        vec3(0.842479062253094, 0.0423282422610123, 0.0423756549057051),
        vec3(0.0784335999999992, 0.878468636469772, 0.0784336),
        vec3(0.0792237451477643, 0.0791661274605434, 0.879142973793104)
    );
    const mat3 backward = mat3(
        vec3(1.19687900512017, -0.0528968517574562, -0.0529716355144438),
        vec3(-0.0980208811401368, 1.15190312990417, -0.0980434501171241),
        vec3(-0.0990297440797205, -0.0989611768448433, 1.15107367264116)
    );
    const float minimum_stop = -12.47393;
    const float maximum_stop = 4.026069;

    vec3 value = max(forward * max(color, vec3(0.0)), vec3(1e-10));
    value = clamp(log2(value), vec3(minimum_stop), vec3(maximum_stop));
    value = (value - minimum_stop) / (maximum_stop - minimum_stop);
    return backward * agx_contrast(value);
}

vec3 tonemap_neutral(vec3 color) {
    const float start = 0.8 - 0.04;
    const float desaturation = 0.15;

    float lowest = min(color.r, min(color.g, color.b));
    float offset = lowest < 0.08 ? lowest - 6.25 * lowest * lowest : 0.04;
    vec3 result = color - vec3(offset);

    float peak = max(result.r, max(result.g, result.b));
    if (peak < start) {
        return result;
    }
    float span = 1.0 - start;
    float new_peak = 1.0 - span * span / (peak + span - start);
    result *= new_peak / peak;
    float blend = 1.0 - 1.0 / (desaturation * (peak - new_peak) + 1.0);
    return mix(result, vec3(new_peak), blend);
}

vec3 apply_tonemap(vec3 color) {
    if (params.tonemap == TONEMAP_ACES) {
        return tonemap_aces(color);
    }
    if (params.tonemap == TONEMAP_ACES2) {
        return tonemap_aces2(color);
    }
    if (params.tonemap == TONEMAP_REINHARD) {
        return tonemap_reinhard(color);
    }
    if (params.tonemap == TONEMAP_REINHARD_EXTENDED) {
        return tonemap_reinhard_extended(color);
    }
    if (params.tonemap == TONEMAP_UNCHARTED2) {
        return tonemap_uncharted2(color);
    }
    if (params.tonemap == TONEMAP_AGX) {
        return tonemap_agx(color);
    }
    if (params.tonemap == TONEMAP_NEUTRAL) {
        return tonemap_neutral(color);
    }
    return clamp(color, 0.0, 1.0);
}

vec3 read_the_world(vec2 point) {
    if (params.focus_enabled != 0u) {
        return texture(focused, point).rgb;
    }
    if (params.resolved_enabled != 0u) {
        return texture(resolved, point).rgb;
    }
    return texture(scene, point).rgb;
}

vec3 looked_up(vec3 color) {
    const float steps = 16.0;
    float scale = (steps - 1.0) / steps;
    float offset = 0.5 / steps;
    vec3 paper = clamp(color, vec3(0.0), vec3(1.0));
    vec3 at = paper * scale + offset;
    return texture(color_lut, at).rgb + (color - paper);
}

void main() {
    if (params.world_enabled == 0u) {
        out_color = vec4(0.0, 0.0, 0.0, 0.0);
        return;
    }
    vec3 color;
    if (params.chromatic_aberration > 0.0) {
        vec2 from_middle = uv - vec2(0.5);
        vec2 spread = from_middle * params.chromatic_aberration * dot(from_middle, from_middle);
        color = vec3(
            read_the_world(uv + spread).r,
            read_the_world(uv).g,
            read_the_world(uv - spread).b
        );
    } else {
        color = read_the_world(uv);
    }

    if (params.occlusion_only != 0u) {
        out_color = vec4(vec3(texture(occlusion, uv).r), 1.0);
        return;
    }
    if (params.occlusion_enabled != 0u) {
        color *= texture(occlusion, uv).r;
    }

    if (params.indirect_enabled != 0u) {
        color += texture(indirect, uv).rgb * params.indirect_intensity;
    }

    if (params.reflection_enabled != 0u) {
        vec4 bounced = texture(reflection, uv);
        float confidence = clamp(bounced.a, 0.0, 1.0);
        color = mix(color, bounced.rgb / max(confidence, 0.001), confidence);
    }

    if (params.bloom_enabled != 0u) {
        color += texture(bloom, uv).rgb * params.bloom_intensity;
    }

    float exposure = params.exposure;
    if (params.auto_exposure_target > 0.0 && settled.measured != 0u) {
        float average = exp2(settled.settled_log_luminance);
        exposure *= clamp(
            params.auto_exposure_target / max(average, 1.0e-4),
            params.auto_exposure_lowest,
            params.auto_exposure_highest
        );
    }
    color *= exposure * params.exposure_compensation;

    float luminance = dot(color, LUMINANCE_WEIGHTS);
    color = mix(vec3(luminance), color, params.saturation);

    float highest = max(color.r, max(color.g, color.b));
    float lowest = min(color.r, min(color.g, color.b));
    float amount = params.vibrance * (1.0 - clamp(highest - lowest, 0.0, 1.0));
    color = mix(vec3(dot(color, LUMINANCE_WEIGHTS)), color, 1.0 + amount);

    color += vec3(params.brightness);
    color = (color - 0.5) * params.contrast + 0.5;

    float headroom = max(params.headroom, 1.0);
    if (headroom > 1.0) {
        const vec3 gray = vec3(0.18);
        vec3 settled = apply_tonemap(gray);
        vec3 raised = apply_tonemap(gray / headroom) * headroom;
        color = apply_tonemap(color / headroom) * headroom * settled / max(raised, vec3(1.0e-4));
        color = min(color, vec3(headroom));
    } else {
        color = apply_tonemap(color);
    }
    color = pow(max(color, vec3(0.0)), vec3(1.0 / params.gamma));

    if (params.color_lut_weight > 0.0) {
        color = mix(color, looked_up(color), params.color_lut_weight);
    }

    if (params.vignette_intensity > 0.0) {
        float distance_from_center = length(uv - vec2(0.5)) * 1.41421356;
        float vignette = 1.0 - params.vignette_intensity * smoothstep(
            params.vignette_radius,
            params.vignette_radius + params.vignette_smoothness,
            distance_from_center
        );
        color *= vignette;
    }

    out_color = vec4(encoded_for_the_screen(color), 1.0);
}
