#version 460

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform sampler2D scene_color;
layout(set = 0, binding = 1) uniform sampler2D resolved_color;
layout(set = 0, binding = 2) uniform sampler2D scene_depth;

layout(push_constant) uniform Push {
    vec4 focus;
    vec4 bokeh;
    vec4 screen;
    vec4 tilt;
    vec4 source;
} push;

const float GOLDEN_ANGLE = 2.39996323;

vec4 read(vec2 point) {
    return push.source.y != 0.0
        ? texture(resolved_color, point)
        : texture(scene_color, point);
}

float depth_at(vec2 point) {
    ivec2 last = textureSize(scene_depth, 0) - ivec2(1);
    ivec2 texel = clamp(ivec2(point * vec2(textureSize(scene_depth, 0))), ivec2(0), last);
    return texelFetch(scene_depth, texel, 0).r;
}

float distance_of(float depth) {
    return push.bokeh.w / max(depth + push.bokeh.z, 1.0e-6);
}

float spread_of(float distance_away) {
    float outside = abs(distance_away - push.focus.x);
    return clamp(outside / max(push.focus.y, 1.0e-4), 0.0, 1.0) * push.focus.z;
}

float tilted_spread_of(vec2 point) {
    float aspect = push.screen.x / max(push.screen.y, 1.0);
    float down = (point.y - 0.5) * 2.0 - push.tilt.z;
    float across = (point.x - 0.5) * 2.0 * aspect;
    float away = abs(down * cos(push.tilt.y) - across * sin(push.tilt.y));
    float half_band = clamp(push.focus.y * 0.02, 0.01, 0.8);
    float outside = clamp(max(away - half_band, 0.0), 0.0, 1.0);
    return outside * outside * push.focus.z * push.tilt.w;
}

vec2 disk_offset(uint index, uint total) {
    float angle = float(index) * GOLDEN_ANGLE;
    float radius = sqrt(float(index) + 0.5) / sqrt(float(total));
    return vec2(cos(angle), sin(angle)) * radius;
}

float luminance(vec3 color) {
    return dot(color, vec3(0.2126, 0.7152, 0.0722));
}

vec3 show_spread(float spread) {
    return mix(
        vec3(0.0, 1.0, 0.0),
        vec3(1.0, 0.0, 0.0),
        clamp(spread / max(push.focus.z, 1.0e-4), 0.0, 1.0)
    );
}

void main() {
    vec4 middle = read(uv);
    out_color = middle;
    if (push.screen.z <= 0.0) {
        return;
    }

    bool tilted = push.tilt.x != 0.0;
    float depth = depth_at(uv);
    float spread;
    float distance_away = 0.0;
    if (tilted) {
        spread = tilted_spread_of(uv);
    } else {
        distance_away = distance_of(depth);
        spread = spread_of(distance_away);
    }

    if (push.screen.w != 0.0 || (tilted && push.source.x != 0.0)) {
        out_color = vec4(mix(middle.rgb, show_spread(spread), 0.5), middle.a);
        return;
    }
    if (spread < 0.5) {
        return;
    }

    vec2 texel = 1.0 / push.screen.xy;
    uint count = uint(push.focus.w);
    vec3 total = vec3(0.0);
    float weight = 0.0;
    vec3 bright = vec3(0.0);
    float bright_weight = 0.0;

    for (uint index = 0u; index < count; ++index) {
        vec2 offset = disk_offset(index, count);
        vec2 point = uv + offset * spread * texel;
        if (point.x < 0.0 || point.x > 1.0 || point.y < 0.0 || point.y > 1.0) {
            continue;
        }
        vec3 found = read(point).rgb;

        float found_spread;
        float found_distance = distance_away;
        if (tilted) {
            found_spread = tilted_spread_of(point);
        } else {
            found_distance = distance_of(depth_at(point));
            found_spread = spread_of(found_distance);
        }

        float share = 1.0 - length(offset);
        if (!tilted && found_distance < distance_away) {
            share *= smoothstep(0.0, spread * 0.5, found_spread);
        }

        total += found * share;
        weight += share;

        float strength = luminance(found);
        if (strength > push.bokeh.x) {
            float amount = (strength - push.bokeh.x) * share * found_spread;
            bright += found * amount;
            bright_weight += amount;
        }
    }

    vec3 blurred = weight > 0.0 ? total / weight : middle.rgb;
    if (bright_weight > 0.0) {
        blurred = mix(
            blurred,
            bright / bright_weight,
            min(bright_weight * push.bokeh.y * 0.1, 1.0)
        );
    }

    out_color = vec4(
        mix(middle.rgb, blurred, smoothstep(0.0, push.focus.z * 0.3, spread)),
        middle.a
    );
}
