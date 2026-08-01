#version 460

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 out_reflection;

layout(set = 0, binding = 0) uniform sampler2D scene_depth;
layout(set = 0, binding = 1) uniform sampler2D scene_normal;
layout(set = 0, binding = 2) uniform sampler2D scene_color;

layout(push_constant) uniform Push {
    mat4 inverse_projection;
    vec4 projection;
    vec4 screen;
    vec4 fade;
    vec4 march;
} push;

float thickness() {
    return push.screen.w / max(push.projection.w, 0.001);
}

vec2 ndc_to_uv(vec2 ndc) {
    return vec2(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
}

vec3 view_position(vec2 point, float depth) {
    vec4 clip = vec4(point.x * 2.0 - 1.0, 1.0 - point.y * 2.0, depth, 1.0);
    vec4 view = push.inverse_projection * clip;
    return view.xyz / view.w;
}

vec3 view_to_ndc(vec3 position) {
    vec4 clip = vec4(
        push.projection.x * position.x,
        push.projection.y * position.y,
        push.projection.z * position.z + push.projection.w,
        -position.z
    );
    return clip.xyz / clip.w;
}

vec4 direction_to_clip(vec3 direction) {
    return vec4(
        push.projection.x * direction.x,
        push.projection.y * direction.y,
        push.projection.z * direction.z,
        -direction.z
    );
}

float gradient_noise(vec2 pixel) {
    return fract(52.9829189 * fract(0.06711056 * pixel.x + 0.00583715 * pixel.y));
}

float edge_fade(vec2 point) {
    const float edge = 0.12;
    float across = smoothstep(0.0, edge, point.x) * smoothstep(1.0, 1.0 - edge, point.x);
    float down = smoothstep(0.0, edge, point.y) * smoothstep(1.0, 1.0 - edge, point.y);
    return across * down;
}

float depth_at_texel(ivec2 texel) {
    ivec2 last = textureSize(scene_depth, 0) - ivec2(1);
    return texelFetch(scene_depth, clamp(texel, ivec2(0), last), 0).r;
}

float depth_nearest(vec2 point, vec2 size) {
    vec2 coordinate = point * size - vec2(0.5);
    return depth_at_texel(ivec2(floor(coordinate + vec2(0.5))));
}

float depth_smooth(vec2 point, vec2 size) {
    vec2 coordinate = point * size - vec2(0.5);
    ivec2 base = ivec2(floor(coordinate));
    vec2 part = coordinate - floor(coordinate);
    float lower = mix(
        depth_at_texel(base),
        depth_at_texel(base + ivec2(1, 0)),
        part.x
    );
    float upper = mix(
        depth_at_texel(base + ivec2(0, 1)),
        depth_at_texel(base + ivec2(1, 1)),
        part.x
    );
    return mix(lower, upper, part.y);
}

struct Distance {
    float ahead;
    float behind;
    bool touching;
};

Distance distance_at(vec3 point, float depth_thickness, vec2 size) {
    vec2 screen_point = ndc_to_uv(point.xy);
    float ray = 1.0 / point.z;
    float smoothed = 1.0 / depth_smooth(screen_point, size);
    float nearest = 1.0 / depth_nearest(screen_point, size);

    Distance result;
    result.ahead = max(smoothed, nearest) * 1.000002 - ray;
    result.behind = ray - min(smoothed, nearest);
    result.touching = result.behind < depth_thickness;
    return result;
}

struct Hit {
    bool found;
    float travelled;
    vec2 point;
};

Hit trace(vec3 start, vec3 end, float jitter, float depth_thickness) {
    Hit result;
    result.found = false;
    result.travelled = 0.0;
    result.point = vec2(0.0);

    vec2 size = vec2(textureSize(scene_depth, 0));
    vec3 direction = end - start;

    vec2 pixels = (ndc_to_uv(end.xy) - ndc_to_uv(start.xy)) * push.screen.xy;
    float stride = max(push.march.z, 1.0);
    uint steps = clamp(uint(floor(length(pixels) / stride)), 2u, uint(push.march.x));

    float near_side = 0.0;
    float far_side = 1.0;
    Distance near_distance;
    Distance far_distance;
    bool crossed = false;

    float first = jitter / float(steps);
    Distance found = distance_at(start + direction * first, depth_thickness, size);
    if (found.ahead < 0.0 && found.touching) {
        far_side = first;
        far_distance = found;
        crossed = true;
    } else {
        near_side = first;
        near_distance = found;
        for (uint step = 1u; step < steps; ++step) {
            float travelled = (float(step) + jitter) / float(steps);
            Distance here = distance_at(start + direction * travelled, depth_thickness, size);
            if (here.ahead < 0.0 && here.touching) {
                far_side = travelled;
                far_distance = here;
                crossed = true;
                break;
            }
            near_side = travelled;
            near_distance = here;
        }
    }

    if (!crossed) {
        result.travelled = near_side;
        return result;
    }

    for (uint step = 0u; step < 5u; ++step) {
        float middle = (near_side + far_side) * 0.5;
        Distance here = distance_at(start + direction * middle, depth_thickness, size);
        if (here.ahead < 0.0 && here.touching) {
            far_side = middle;
            far_distance = here;
        } else {
            near_side = middle;
            near_distance = here;
        }
    }

    float settled = far_side;
    Distance settled_distance = far_distance;

    float span = near_distance.ahead + (-far_distance.ahead);
    if (span > 0.001) {
        float between = mix(near_side, far_side, near_distance.ahead / span);
        Distance here = distance_at(start + direction * between, depth_thickness, size);
        if (abs(here.ahead) < near_distance.ahead * 0.9 && here.touching) {
            settled = between;
            settled_distance = here;
        }
    }

    if (settled_distance.behind < depth_thickness) {
        result.found = true;
        result.travelled = settled;
        result.point = ndc_to_uv(mix(start.xy, end.xy, settled));
    }
    return result;
}

void main() {
    out_reflection = vec4(0.0);
    if (push.fade.w <= 0.0) {
        return;
    }

    float depth = texture(scene_depth, uv).r;
    if (depth <= 0.0) {
        return;
    }

    vec4 written = texture(scene_normal, uv);
    if (dot(written.xyz, written.xyz) < 1.0e-6) {
        return;
    }
    float reflectivity = clamp(written.w, 0.0, 1.0);
    if (reflectivity < 0.002) {
        return;
    }

    vec3 surface = view_position(uv, depth);
    vec3 normal = normalize(written.xyz);
    vec3 towards = normalize(surface);
    vec3 bounced = reflect(towards, normal);
    if (dot(bounced, normal) < 0.01) {
        return;
    }

    vec3 start = view_to_ndc(surface + normal * 0.02);
    vec4 ending =
        vec4(start, 1.0) + direction_to_clip(normalize(bounced) * max(push.march.y, 0.01));
    float sign_of = ending.w >= 0.0 ? 1.0 : -1.0;
    ending /= sign_of * max(1.0e-10, abs(ending.w));
    vec3 end = ending.xyz;

    vec3 delta = end - start;
    if (length(delta) < 0.0001) {
        return;
    }

    vec3 first_edge = vec3(
        delta.x < 0.0 ? 1.0 : -1.0,
        delta.y < 0.0 ? 1.0 : -1.0,
        delta.z < 0.0 ? 1.0 : 0.0
    );
    vec3 to_first = (first_edge - start) / delta;
    start += delta * max(0.0, max(to_first.x, to_first.y));

    delta = end - start;
    vec3 last_edge = vec3(
        delta.x >= 0.0 ? 1.0 : -1.0,
        delta.y >= 0.0 ? 1.0 : -1.0,
        delta.z >= 0.0 ? 1.0 : 0.0
    );
    vec3 to_last = (last_edge - start) / delta;
    delta *= min(min(to_last.x, to_last.y), to_last.z);
    end = start + delta;

    float jitter = gradient_noise(uv * push.screen.xy);
    Hit hit = trace(start, end, jitter, thickness());
    if (!hit.found) {
        return;
    }
    if (hit.point.x < 0.0 || hit.point.x > 1.0 || hit.point.y < 0.0 || hit.point.y > 1.0) {
        return;
    }

    vec3 reflected = min(texture(scene_color, hit.point).rgb, vec3(2.0));

    float span = push.fade.y - push.fade.x;
    float distance_fade = 1.0;
    if (span > 0.001) {
        distance_fade = 1.0 - clamp((hit.travelled - push.fade.x) / span, 0.0, 1.0);
    }
    float confidence = clamp(
        distance_fade * edge_fade(hit.point) * push.fade.z * reflectivity,
        0.0,
        1.0
    );
    out_reflection = vec4(reflected * confidence, confidence);
}
