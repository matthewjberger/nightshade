#version 460

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 out_kept;
layout(location = 1) out vec4 out_moments;

layout(set = 0, binding = 0) uniform sampler2D traced;
layout(set = 0, binding = 1) uniform sampler2D kept_history;
layout(set = 0, binding = 2) uniform sampler2D moments_history;
layout(set = 0, binding = 3) uniform sampler2D scene_depth;
layout(set = 0, binding = 4) uniform sampler2D scene_normal;
layout(set = 0, binding = 5) uniform sampler2D scene_motion;

layout(push_constant) uniform Push {
    mat4 reprojection;
    vec4 screen;
    vec4 edges;
} push;

float luminance(vec3 color) {
    return dot(color, vec3(0.2126, 0.7152, 0.0722));
}

void main() {
    vec4 now = texture(traced, uv);
    float brightness = luminance(now.rgb);
    out_kept = vec4(now.rgb, now.a);
    out_moments = vec4(brightness, brightness * brightness, now.a, 1.0);

    if (push.screen.z <= 0.0) {
        return;
    }

    vec2 moved = texture(scene_motion, uv).rg;
    vec2 was;
    if (dot(moved, moved) > 0.0) {
        was = uv - moved;
    } else {
        float depth = texture(scene_depth, uv).r;
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

    vec4 had = texture(kept_history, was);
    vec4 had_moments = texture(moments_history, was);

    vec2 texel = 1.0 / max(push.screen.xy, vec2(1.0));
    vec3 lowest = now.rgb;
    vec3 highest = now.rgb;
    for (int down = -1; down <= 1; ++down) {
        for (int across = -1; across <= 1; ++across) {
            vec3 near = texture(traced, uv + vec2(across, down) * texel).rgb;
            lowest = min(lowest, near);
            highest = max(highest, near);
        }
    }
    vec3 middle = (lowest + highest) * 0.5;
    vec3 spread = max((highest - lowest) * 0.5 * max(push.edges.w, 1.0), vec3(1.0e-4));
    vec3 held = clamp(had.rgb, middle - spread, middle + spread);

    float standing = min(had_moments.w + 1.0, max(push.screen.w, 1.0));
    float share = max(1.0 / standing, 0.02);
    out_kept = vec4(mix(held, now.rgb, share), mix(had.a, now.a, share));
    out_moments = vec4(
        mix(had_moments.x, brightness, share),
        mix(had_moments.y, brightness * brightness, share),
        mix(had_moments.z, now.a, share),
        standing
    );
}
