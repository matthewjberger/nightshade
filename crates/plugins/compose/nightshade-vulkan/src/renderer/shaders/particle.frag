#version 460
#extension GL_GOOGLE_include_directive : require
#extension GL_EXT_nonuniform_qualifier : require

#include "particle_common.glsl"

layout(set = 1, binding = 0) uniform texture2D particle_textures[];
layout(set = 1, binding = 1) uniform sampler particle_samplers[];

layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec4 in_color;
layout(location = 2) in float in_emissive;
layout(location = 3) flat in uint in_emitter_type;
layout(location = 4) flat in uint in_texture_index;

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec4 out_normal;

float firework_glow(vec2 uv) {
    float distance_from_center = length(uv - vec2(0.5));
    float squared = distance_from_center * distance_from_center;
    return exp(-squared * 120.0)
        + exp(-squared * 40.0) * 0.9
        + exp(-squared * 15.0) * 0.6
        + exp(-squared * 6.0) * 0.35
        + exp(-squared * 2.5) * 0.15;
}

float fire_shape(vec2 uv) {
    vec2 offset = uv - vec2(0.5);
    offset.y *= 0.65;
    float squared = dot(offset, offset);
    return exp(-squared * 60.0) + exp(-squared * 15.0) * 0.75 + exp(-squared * 5.0) * 0.35;
}

float smoke_shape(vec2 uv) {
    float distance_from_center = length(uv - vec2(0.5));
    return exp(-distance_from_center * distance_from_center * 4.0) * 0.85;
}

float spark_shape(vec2 uv) {
    float distance_from_center = length(uv - vec2(0.5));
    float squared = distance_from_center * distance_from_center;
    return exp(-squared * 200.0) + exp(-squared * 60.0) * 0.9 + exp(-squared * 15.0) * 0.5;
}

float shape_of(vec2 uv, uint emitter_type) {
    if (emitter_type == EMITTER_FIRE) {
        return fire_shape(uv);
    }
    if (emitter_type == EMITTER_SMOKE) {
        return smoke_shape(uv);
    }
    if (emitter_type == EMITTER_SPARKS) {
        return spark_shape(uv);
    }
    return firework_glow(uv);
}

void main() {
    out_normal = vec4(0.0);
    float brightness = max(in_emissive - 1.0, 0.0);
    if (in_texture_index > 0u) {
        vec4 sampled = texture(
            sampler2D(
                particle_textures[nonuniformEXT(in_texture_index - 1u)],
                particle_samplers[0]
            ),
            in_uv
        );
        if (sampled.a < 0.01) {
            discard;
        }
        vec4 color = in_color * sampled;
        float intensity = color.a;
        vec3 light = color.rgb * intensity * (1.0 + brightness * 1.2);
        out_color = vec4(light + light * light * 0.3, intensity);
        return;
    }

    float shape = shape_of(in_uv, in_emitter_type);
    if (shape < 0.005) {
        discard;
    }
    float intensity = shape * in_color.a;
    if (in_emitter_type == EMITTER_SMOKE) {
        out_color = vec4(in_color.rgb * intensity * 1.5, intensity * 0.8);
        return;
    }
    vec3 light = in_color.rgb * intensity * (1.0 + brightness * 1.2);
    out_color = vec4(light + light * light * 0.3, intensity);
}
