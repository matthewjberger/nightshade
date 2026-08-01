#version 460
#extension GL_GOOGLE_include_directive : require

#include "glyph_distance.glsl"

struct TextInstanceData {
    vec4 color;
    vec4 outline_color;
    vec4 clip_rect;
    vec4 params;
    vec4 position;
};

layout(set = 0, binding = 1) readonly buffer TextInstances { TextInstanceData text_instances[]; };
layout(set = 0, binding = 2) readonly buffer CharacterColors { vec4 character_colors[]; };
layout(set = 0, binding = 3) readonly buffer CharacterBackgrounds { vec4 character_backgrounds[]; };
layout(set = 0, binding = 4) uniform sampler2D font_texture;

layout(location = 0) in vec2 tex_coords;
layout(location = 1) in vec2 screen_pos;
layout(location = 2) flat in uint character_index;
layout(location = 3) flat in uint text_instance_index;
layout(location = 4) flat in uint flags;

layout(location = 0) out vec4 out_color;

void main() {
    TextInstanceData instance = text_instances[text_instance_index];

    vec4 clip = instance.clip_rect;
    if (clip.z > clip.x
        && (screen_pos.x < clip.x || screen_pos.x > clip.z
            || screen_pos.y < clip.y || screen_pos.y > clip.w)) {
        discard;
    }

    vec2 atlas_size = vec2(textureSize(font_texture, 0));
    vec4 sampled = texture(font_texture, tex_coords / atlas_size);

    vec4 character_color = character_colors[character_index];
    vec4 base_color = character_color.a > 0.0 ? character_color : instance.color;

    vec4 foreground;
    if ((flags & GLYPH_COLORED) != 0u) {
        foreground = vec4(base_color.rgb * sampled.rgb, sampled.a * base_color.a);
    } else {
        float distance = (sampled.a - 0.5) * 2.0 * SPREAD;
        float pixel = length(fwidth(tex_coords)) * 0.7071;
        foreground = shade_distance(
            distance,
            pixel,
            base_color,
            instance.outline_color,
            instance.params.x,
            instance.params.z
        );
    }

    vec4 background = character_backgrounds[character_index];
    if (background.a > 0.0) {
        out_color = vec4(
            mix(background.rgb, foreground.rgb, foreground.a),
            max(background.a, foreground.a)
        );
        return;
    }

    out_color = foreground;
}
