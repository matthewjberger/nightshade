#version 460

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 out_display;
layout(location = 1) out vec4 out_history;

layout(set = 0, binding = 0) uniform sampler2D scene_color;
layout(set = 0, binding = 1) uniform sampler2D history;
layout(set = 0, binding = 2) uniform sampler2D scene_depth;
layout(set = 0, binding = 3) uniform sampler2D scene_motion;

layout(push_constant) uniform Push {
    mat4 reprojection;
    vec4 screen;
    vec4 settings;
} push;

vec3 to_ycocg(vec3 color) {
    return vec3(
        dot(color, vec3(0.25, 0.5, 0.25)),
        dot(color, vec3(0.5, 0.0, -0.5)),
        dot(color, vec3(-0.25, 0.5, -0.25))
    );
}

vec3 to_rgb(vec3 color) {
    return vec3(
        color.x + color.y - color.z,
        color.x + color.z,
        color.x - color.y - color.z
    );
}

float luminance(vec3 color) {
    return dot(color, vec3(0.2126, 0.7152, 0.0722));
}

const float FOLD_KNEE = 4.0;

vec3 compress(vec3 color) {
    return color / (1.0 + luminance(color) / FOLD_KNEE);
}

vec3 expand(vec3 color) {
    return color / max(1.0 - luminance(color) / FOLD_KNEE, 1.0e-3);
}

vec3 clip_to_box(vec3 lowest, vec3 highest, vec3 color) {
    vec3 center = 0.5 * (highest + lowest);
    vec3 extent = 0.5 * (highest - lowest) + vec3(1.0e-5);
    vec3 offset = color - center;
    vec3 units = abs(offset / extent);
    float furthest = max(units.x, max(units.y, units.z));
    return furthest > 1.0 ? center + offset / furthest : color;
}

vec3 history_at(vec2 at) {
    vec2 resolution = push.screen.xy;
    vec2 sample_position = at * resolution;
    vec2 texel_position = floor(sample_position - 0.5) + 0.5;
    vec2 f = sample_position - texel_position;

    vec2 w0 = f * (-0.5 + f * (1.0 - 0.5 * f));
    vec2 w1 = 1.0 + f * f * (-2.5 + 1.5 * f);
    vec2 w2 = f * (0.5 + f * (2.0 - 1.5 * f));
    vec2 w3 = f * f * (-0.5 + 0.5 * f);
    vec2 w12 = w1 + w2;
    vec2 offset12 = w2 / w12;

    vec2 inverse = 1.0 / resolution;
    vec2 position0 = (texel_position - 1.0) * inverse;
    vec2 position3 = (texel_position + 2.0) * inverse;
    vec2 position12 = (texel_position + offset12) * inverse;

    vec3 result = vec3(0.0);
    result += texture(history, vec2(position0.x, position0.y)).rgb * (w0.x * w0.y);
    result += texture(history, vec2(position12.x, position0.y)).rgb * (w12.x * w0.y);
    result += texture(history, vec2(position3.x, position0.y)).rgb * (w3.x * w0.y);
    result += texture(history, vec2(position0.x, position12.y)).rgb * (w0.x * w12.y);
    result += texture(history, vec2(position12.x, position12.y)).rgb * (w12.x * w12.y);
    result += texture(history, vec2(position3.x, position12.y)).rgb * (w3.x * w12.y);
    result += texture(history, vec2(position0.x, position3.y)).rgb * (w0.x * w3.y);
    result += texture(history, vec2(position12.x, position3.y)).rgb * (w12.x * w3.y);
    result += texture(history, vec2(position3.x, position3.y)).rgb * (w3.x * w3.y);
    return max(result, vec3(0.0));
}

void main() {
    vec4 current = texture(scene_color, uv);
    out_display = current;
    out_history = current;

    if (push.settings.x <= 0.0) {
        return;
    }

    vec2 texel = 1.0 / push.screen.xy;

    float nearest = 0.0;
    vec2 nearest_offset = vec2(0.0);
    for (int down = -1; down <= 1; ++down) {
        for (int across = -1; across <= 1; ++across) {
            vec2 offset = vec2(float(across), float(down));
            float found = texture(scene_depth, uv + offset * texel).r;
            if (found > nearest) {
                nearest = found;
                nearest_offset = offset;
            }
        }
    }

    vec2 point = uv + nearest_offset * texel;
    vec2 moved = texture(scene_motion, point).rg;
    vec2 was;
    if (dot(moved, moved) > 0.0) {
        was = uv - moved;
    } else {
        float depth = texture(scene_depth, point).r;
        vec4 clip = vec4(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, max(depth, 1.0e-6), 1.0);
        vec4 before = push.reprojection * clip;
        if (before.w <= 0.0) {
            return;
        }
        vec3 before_ndc = before.xyz / before.w;
        was = vec2(before_ndc.x * 0.5 + 0.5, 0.5 - before_ndc.y * 0.5);
    }
    if (was.x < 0.0 || was.x > 1.0 || was.y < 0.0 || was.y > 1.0) {
        return;
    }

    vec3 now = compress(current.rgb);
    vec3 middle = to_ycocg(now);
    vec3 lowest = middle;
    vec3 highest = middle;
    vec3 total = middle;
    vec3 total_squared = middle * middle;
    vec3 darkest = now;
    vec3 brightest = now;
    for (int down = -1; down <= 1; ++down) {
        for (int across = -1; across <= 1; ++across) {
            if (down == 0 && across == 0) {
                continue;
            }
            vec2 offset = vec2(float(across), float(down)) * texel;
            vec3 near = compress(texture(scene_color, uv + offset).rgb);
            vec3 neighbour = to_ycocg(near);
            lowest = min(lowest, neighbour);
            highest = max(highest, neighbour);
            total += neighbour;
            total_squared += neighbour * neighbour;
            darkest = min(darkest, near);
            brightest = max(brightest, near);
        }
    }

    vec3 mean = total / 9.0;
    vec3 spread = sqrt(max(total_squared / 9.0 - mean * mean, vec3(0.0))) * 1.5;
    vec3 box_low = max(mean - spread, lowest);
    vec3 box_high = min(mean + spread, highest);

    vec3 kept = to_ycocg(compress(history_at(was)));
    vec3 pulled = clip_to_box(box_low, box_high, kept);
    vec3 settled = clamp(to_rgb(pulled), darkest, brightest);

    vec3 half_extent = max((box_high - box_low) * 0.5, vec3(1.0e-4));
    float pulled_by = length((kept - pulled) / half_extent);
    float blend = clamp(push.settings.y + pulled_by * 0.5, push.settings.y, 0.5);

    vec3 resolved = mix(settled, now, blend);

    vec3 low_pass = clamp(to_rgb(mean), darkest, brightest);
    vec3 sharpened = clamp(
        resolved + (resolved - low_pass) * push.settings.z,
        darkest,
        brightest
    );

    out_display = vec4(expand(sharpened), current.a);
    out_history = vec4(expand(resolved), current.a);
}
