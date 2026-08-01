#version 460
#extension GL_GOOGLE_include_directive : require

layout(location = 0) in vec2 uv;
layout(location = 0) out float out_occlusion;

layout(set = 0, binding = 0) uniform sampler2D scene_depth;
layout(set = 0, binding = 1) uniform sampler2D scene_normal;

layout(push_constant) uniform Push {
    mat4 inverse_projection;
    vec4 projection;
    vec4 settings;
} push;

const float TWO_PI = 6.28318530718;

#include "screen_sampling.glsl"

vec3 view_position(vec2 point, float depth) {
    vec4 clip = vec4(point.x * 2.0 - 1.0, 1.0 - point.y * 2.0, depth, 1.0);
    vec4 view = push.inverse_projection * clip;
    return view.xyz / view.w;
}

vec2 screen_of(vec3 position) {
    vec2 ndc = vec2(push.projection.x * position.x, push.projection.y * position.y)
        / -position.z;
    return vec2(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
}

float depth_at(vec2 point) {
    return texture(scene_depth, point).r;
}

void main() {
    if (push.settings.z <= 0.0) {
        out_occlusion = 1.0;
        return;
    }
    float depth = depth_at(uv);
    if (depth <= 0.0) {
        out_occlusion = 1.0;
        return;
    }

    vec4 written = texture(scene_normal, uv);
    if (dot(written.xyz, written.xyz) < 1.0e-6) {
        out_occlusion = 1.0;
        return;
    }
    vec3 normal = normalize(written.xyz);
    vec3 center = view_position(uv, depth);

    float turn = hash(floor(mod(gl_FragCoord.xy, 4.0))) * TWO_PI;
    vec3 random = vec3(cos(turn), sin(turn), 0.0);
    vec3 tangent = random - normal * dot(random, normal);
    if (dot(tangent, tangent) < 1.0e-8) {
        tangent = abs(normal.x) > 0.9 ? vec3(0.0, 1.0, 0.0) : vec3(1.0, 0.0, 0.0);
        tangent = tangent - normal * dot(tangent, normal);
    }
    tangent = normalize(tangent);
    mat3 basis = mat3(tangent, cross(normal, tangent), normal);

    uint count = uint(push.settings.w);
    float radius = push.settings.x;
    float bias = push.settings.y;
    float occlusion = 0.0;

    for (uint index = 0u; index < count; ++index) {
        vec2 point = hammersley(index, count);
        float angle = point.x * TWO_PI;
        float spread = sqrt(point.y);
        vec3 direction = vec3(
            cos(angle) * spread,
            sin(angle) * spread,
            sqrt(max(1.0 - point.y, 0.0))
        );
        float step = float(index + 1u) / float(count);
        vec3 offset = basis * direction * mix(0.1, 1.0, step * step);
        vec3 taken = center + offset * radius;

        vec2 point_on_screen = screen_of(taken);
        if (point_on_screen.x < 0.0 || point_on_screen.x > 1.0
            || point_on_screen.y < 0.0 || point_on_screen.y > 1.0) {
            continue;
        }
        float found = depth_at(point_on_screen);
        if (found <= 0.0) {
            continue;
        }
        vec3 other = texture(scene_normal, point_on_screen).xyz;
        if (dot(other, other) > 1.0e-6 && dot(normal, normalize(other)) < 0.3) {
            continue;
        }
        float surface = view_position(point_on_screen, found).z;
        float apart = abs(center.z - surface);
        if (apart < bias) {
            continue;
        }
        if (surface >= taken.z + bias) {
            occlusion += smoothstep(0.0, 1.0, radius / apart);
        }
    }

    occlusion = 1.0 - occlusion / float(count);
    out_occlusion = pow(max(occlusion, 0.0), push.settings.z);
}
