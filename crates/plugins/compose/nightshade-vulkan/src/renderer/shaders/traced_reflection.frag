#version 460
#extension GL_GOOGLE_include_directive : require
#extension GL_EXT_nonuniform_qualifier : require
#extension GL_EXT_ray_query : require

#include "scene_common.glsl"
#include "scene_rays.glsl"
#include "scene_traced.glsl"

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 out_reflection;

layout(set = 4, binding = 0) uniform sampler2D scene_depth;
layout(set = 4, binding = 1) uniform sampler2D scene_normal;
layout(set = 4, binding = 2) uniform sampler2D scene_color;

layout(push_constant) uniform Push {
    mat4 inverse_projection;
    vec4 projection;
    vec4 screen;
    vec4 fade;
    vec4 march;
} push;

vec3 view_position(vec2 point, float depth) {
    vec4 clip = vec4(point.x * 2.0 - 1.0, 1.0 - point.y * 2.0, depth, 1.0);
    vec4 view = push.inverse_projection * clip;
    return view.xyz / view.w;
}

float edge_fade(vec2 point) {
    const float edge = 0.12;
    float across = smoothstep(0.0, edge, point.x) * smoothstep(1.0, 1.0 - edge, point.x);
    float down = smoothstep(0.0, edge, point.y) * smoothstep(1.0, 1.0 - edge, point.y);
    return across * down;
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
    vec3 facing = normalize(written.xyz);
    vec3 bounced = reflect(normalize(surface), facing);
    if (dot(bounced, facing) < 0.01) {
        return;
    }

    mat3 to_world = transpose(mat3(frame.view));
    vec3 origin = frame.camera_position.xyz + to_world * surface;
    vec3 normal = normalize(to_world * facing);
    vec3 direction = normalize(to_world * bounced);
    float step_off = max(length(surface) * 1.0e-3, 1.0e-4);
    float reach = max(push.march.y, 0.01);

    Met met;
    if (!trace(origin + normal * step_off, direction, step_off, reach, met)) {
        return;
    }
    float travelled = met.travelled;
    vec3 at = origin + direction * travelled;
    Surface stood = surface_at(met, direction);
    if (stood.passes > 0.5) {
        Met behind;
        float remaining = max(reach - travelled, 0.01);
        if (trace(at + direction * step_off, direction, step_off, remaining, behind)) {
            travelled += behind.travelled;
            at = at + direction * behind.travelled;
            stood = surface_at(behind, direction);
        }
    }

    vec3 seen = vec3(0.0);
    float from_screen = 0.0;
    vec4 clip = frame.view_projection * vec4(at, 1.0);
    if (clip.w > 0.0) {
        vec3 ndc = clip.xyz / clip.w;
        vec2 point = vec2(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
        if (point.x >= 0.0 && point.x <= 1.0 && point.y >= 0.0 && point.y <= 1.0) {
            float shown = texture(scene_depth, point).r;
            if (shown <= ndc.z + push.screen.w * 1.0e-3) {
                seen = texture(scene_color, point).rgb;
                from_screen = edge_fade(point);
            }
        }
    }
    vec3 reflected = from_screen < 1.0
        ? mix(shade(stood, at, direction, reach), seen, from_screen)
        : seen;
    reflected = min(reflected, vec3(2.0));

    float span = push.fade.y - push.fade.x;
    float distance_fade = 1.0;
    if (span > 0.001) {
        distance_fade = 1.0 - clamp((travelled / reach - push.fade.x) / span, 0.0, 1.0);
    }
    float confidence = clamp(distance_fade * push.fade.z * reflectivity, 0.0, 1.0);
    out_reflection = vec4(reflected * confidence, confidence);
}
