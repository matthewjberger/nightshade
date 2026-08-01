#version 460
#extension GL_GOOGLE_include_directive : require

#include "glyph_distance.glsl"

layout(location = 0) in vec2 tex_coords;
layout(location = 1) in flat uint character;
layout(location = 2) in flat uint flags;

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec4 out_normal;

layout(set = 0, binding = 1) uniform sampler2D glyphs;
layout(std430, set = 0, binding = 2) readonly buffer CharacterColors {
    vec4 character_colors[];
};

layout(push_constant) uniform Push {
    mat4 model;
    vec4 color;
    vec4 outline_color;
    vec4 settings;
} push;

void main() {
    vec2 size = vec2(textureSize(glyphs, 0));
    vec4 sampled = texture(glyphs, tex_coords / size);

    uint slot = uint(push.settings.w) + character;
    vec4 own = character_colors[slot];
    vec4 base = own.a > 0.0 ? own : push.color;

    vec4 shaded;
    if ((flags & GLYPH_COLORED) != 0u) {
        shaded = vec4(base.rgb * sampled.rgb, sampled.a * base.a);
    } else {
        float distance = (sampled.a - 0.5) * 2.0 * SPREAD;
        float pixel = length(fwidth(tex_coords)) * 0.7071;
        shaded = shade_distance(
            distance,
            pixel,
            base,
            push.outline_color,
            push.settings.x,
            push.settings.y
        );
    }

    if (shaded.a < 0.01) {
        discard;
    }
    out_color = shaded;
    out_normal = vec4(0.0);
}
