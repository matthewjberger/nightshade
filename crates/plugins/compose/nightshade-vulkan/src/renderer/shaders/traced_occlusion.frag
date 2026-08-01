#version 460
#extension GL_GOOGLE_include_directive : require
#extension GL_EXT_nonuniform_qualifier : require
#extension GL_EXT_ray_query : require

#include "scene_common.glsl"
#include "scene_rays.glsl"

layout(location = 0) in vec2 uv;
layout(location = 0) out float out_occlusion;

layout(set = 4, binding = 0) uniform sampler2D scene_depth;
layout(set = 4, binding = 1) uniform sampler2D scene_normal;

layout(push_constant) uniform Push {
    mat4 inverse_projection;
    vec4 projection;
    vec4 settings;
    vec4 traced;
} push;

const float TWO_PI = 6.28318530718;

#include "screen_sampling.glsl"

vec3 view_position(vec2 point, float depth) {
    vec4 clip = vec4(point.x * 2.0 - 1.0, 1.0 - point.y * 2.0, depth, 1.0);
    vec4 view = push.inverse_projection * clip;
    return view.xyz / view.w;
}

void main() {
    if (push.settings.z <= 0.0) {
        out_occlusion = 1.0;
        return;
    }
    float depth = texture(scene_depth, uv).r;
    if (depth <= 0.0) {
        out_occlusion = 1.0;
        return;
    }
    vec4 written = texture(scene_normal, uv);
    if (dot(written.xyz, written.xyz) < 1.0e-6) {
        out_occlusion = 1.0;
        return;
    }

    vec3 center = view_position(uv, depth);
    vec3 facing = normalize(written.xyz);
    mat3 to_world = transpose(mat3(frame.view));
    vec3 origin = frame.camera_position.xyz + to_world * center;
    vec3 normal = normalize(to_world * facing);

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
    float step_off = max(length(center) * 1.0e-3, 1.0e-4);
    float occlusion = 0.0;

    for (uint index = 0u; index < count; ++index) {
        vec2 point = hammersley(index, count);
        float angle = point.x * TWO_PI;
        float spread = sqrt(point.y);
        vec3 direction = basis * vec3(
            cos(angle) * spread,
            sin(angle) * spread,
            sqrt(max(1.0 - point.y, 0.0))
        );
        rayQueryEXT query;
        rayQueryInitializeEXT(
            query,
            world,
            gl_RayFlagsNoneEXT,
            0xffu,
            origin + normal * step_off,
            step_off,
            direction,
            radius
        );
        while (rayQueryProceedEXT(query)) {
            if (rayQueryGetIntersectionTypeEXT(query, false)
                    == gl_RayQueryCandidateIntersectionTriangleEXT
                && cut_surface_covers(query)) {
                rayQueryConfirmIntersectionEXT(query);
            }
        }
        if (rayQueryGetIntersectionTypeEXT(query, true)
            != gl_RayQueryCommittedIntersectionNoneEXT) {
            float travelled = rayQueryGetIntersectionTEXT(query, true);
            occlusion += 1.0 - clamp(travelled / radius, 0.0, 1.0);
        }
    }

    occlusion = 1.0 - occlusion / float(max(count, 1u));
    out_occlusion = pow(max(occlusion, 0.0), push.settings.z);
}
