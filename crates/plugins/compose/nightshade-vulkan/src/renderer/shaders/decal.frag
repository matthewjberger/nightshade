#version 460
#extension GL_EXT_scalar_block_layout : require
#extension GL_EXT_nonuniform_qualifier : require
#extension GL_GOOGLE_include_directive : require

#include "decal_common.glsl"

layout(set = 0, binding = 2) uniform sampler2D scene_depth;
layout(set = 1, binding = 0) uniform texture2D decal_textures[];
layout(set = 1, binding = 1) uniform sampler decal_samplers[];

layout(location = 0) flat in uint in_instance;
layout(location = 0) out vec4 out_color;

vec4 sample_slot(uint slot, vec2 uv) {
    return texture(
        sampler2D(decal_textures[nonuniformEXT(slot)], decal_samplers[0]),
        uv
    );
}

vec3 world_position(vec2 uv, float depth) {
    vec4 clip = vec4(uv.x * 2.0 - 1.0, (1.0 - uv.y) * 2.0 - 1.0, depth, 1.0);
    vec4 world = camera.inverse_view_projection * clip;
    return world.xyz / world.w;
}

void main() {
    Decal decal = decals[in_instance];

    vec2 uv = gl_FragCoord.xy / camera.screen.xy;
    float depth = texture(scene_depth, uv).r;
    if (depth <= 0.0) {
        discard;
    }

    vec3 surface = world_position(uv, depth);
    vec4 local = decal.inverse_model * vec4(surface, 1.0);
    vec3 within = local.xyz / decal.size_depth.xyz;
    if (abs(within.x) > 0.5 || abs(within.y) > 0.5 || abs(within.z) > 0.5) {
        discard;
    }

    vec2 decal_uv = vec2(within.x + 0.5, 0.5 - within.y);
    vec4 color = decal.color;
    if (decal.params.w >= 0.0) {
        color *= sample_slot(uint(decal.params.w), decal_uv);
    }
    if (color.a < 0.01) {
        discard;
    }

    vec3 emissive = vec3(0.0);
    float strength = decal.emissive.w;
    if (strength > 0.0) {
        emissive = decal.params.x >= 0.0
            ? sample_slot(uint(decal.params.x), decal_uv).rgb
            : color.rgb;
        emissive *= decal.emissive.rgb * strength;
    }

    float distance_to_camera = length(camera.camera_position.xyz - surface);
    float fade_start = decal.params.y;
    float fade_end = decal.params.z;
    float fade = fade_end > fade_start
        ? 1.0 - clamp((distance_to_camera - fade_start) / (fade_end - fade_start), 0.0, 1.0)
        : 1.0;

    out_color = vec4(color.rgb + emissive, color.a * fade);
}
