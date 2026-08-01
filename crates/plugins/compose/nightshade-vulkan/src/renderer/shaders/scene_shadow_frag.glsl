#extension GL_GOOGLE_include_directive : require
#extension GL_EXT_nonuniform_qualifier : require
#extension GL_EXT_mesh_shader : require

#include "scene_common.glsl"

layout(location = 0) in VertexOutput {
    vec3 normal;
    vec3 world_position;
    vec3 previous_world_position;
    vec2 tex_coords;
    vec2 tex_coords_1;
    vec4 tangent;
    vec4 color;
} vertex_input;

#ifdef RUN_PATH
layout(location = 7) in flat uint material_index;
layout(location = 8) in flat uint object_index;
#else
layout(location = 7) perprimitiveEXT in flat uint material_index;
layout(location = 8) perprimitiveEXT in flat uint object_index;
#endif
layout(location = 9) noperspective in vec2 tex_coords_affine;

layout(set = 1, binding = 0) uniform texture2D scene_textures[];
layout(set = 1, binding = 1) uniform sampler scene_samplers[];

float dither_noise(vec2 fragment, float seconds, uint object) {
    vec2 jitter = vec2(fract(seconds * 91.7), fract(seconds * 47.3)) * 64.0;
    vec2 point = fragment + jitter + float(object & 0xFFu);
    return fract(52.9829189 * fract(dot(point, vec2(0.06711056, 0.00583715))));
}

vec4 sample_slot(uint slot, vec2 uv) {
    uint index = slot & 0xFFFFu;
    uint wrap_u = (slot >> 16u) & 3u;
    uint wrap_v = (slot >> 18u) & 3u;
    uint nearest = (slot >> 20u) & 1u;
    uint sampler_slot = nearest * 9u + wrap_u * 3u + wrap_v;
    return texture(
        sampler2D(scene_textures[nonuniformEXT(index)], scene_samplers[sampler_slot]),
        uv
    );
}

vec2 slot_uv(GpuMaterial material, int slot) {
    vec4 row_u = material.transforms[slot * 2];
    vec4 row_v = material.transforms[slot * 2 + 1];
    vec2 first = frame.style.w != 0.0 ? tex_coords_affine : vertex_input.tex_coords;
    vec2 uv = uint(row_u.w) == 1u ? vertex_input.tex_coords_1 : first;
    vec3 homogeneous = vec3(uv, 1.0);
    return vec2(dot(row_u.xyz, homogeneous), dot(row_v.xyz, homogeneous));
}

void main() {
    if (material_index == NO_MATERIAL) {
        return;
    }
    GpuMaterial material = materials[material_index];
    uint alpha_mode = uint(material.alpha.y);
    if (alpha_mode != ALPHA_MODE_MASK && alpha_mode != ALPHA_MODE_DITHER) {
        return;
    }

    vec4 albedo = material.base_color * surface_tint(material, vertex_input.color);
    uint base_slot = material.textures[TEXTURE_BASE];
    if (base_slot != NO_TEXTURE) {
        albedo *= sample_slot(
            base_slot,
            atlas_uv(material, slot_uv(material, TEXTURE_BASE), vertex_input.tex_coords_1.x)
        );
    }

    if (alpha_mode == ALPHA_MODE_MASK) {
        if (albedo.a < material.alpha.x) {
            discard;
        }
        return;
    }

    float noise = dither_noise(gl_FragCoord.xy, frame.settings.z, object_index);
    if (albedo.a < pow(noise, 2.2)) {
        discard;
    }
}
