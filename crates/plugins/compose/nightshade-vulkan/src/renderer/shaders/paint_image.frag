#version 460
#extension GL_EXT_nonuniform_qualifier : require

#define NO_TEXTURE_LAYER 0xFFFFFFFFu

layout(set = 1, binding = 0) uniform texture2D ui_images[];
layout(set = 1, binding = 1) uniform sampler ui_samplers[];

layout(location = 0) in vec2 uv;
layout(location = 1) in vec4 tint;
layout(location = 2) in vec2 screen_pos;
layout(location = 3) flat in vec4 clip_rect;
layout(location = 4) flat in uint texture_layer;

#define CLAMPED_SAMPLER 8

layout(location = 0) out vec4 out_color;

void main() {
    if (screen_pos.x < clip_rect.x || screen_pos.y < clip_rect.y
        || screen_pos.x > clip_rect.z || screen_pos.y > clip_rect.w) {
        discard;
    }

    if (texture_layer == NO_TEXTURE_LAYER) {
        out_color = tint;
        return;
    }

    out_color = texture(sampler2D(ui_images[nonuniformEXT(texture_layer)], ui_samplers[CLAMPED_SAMPLER]), uv) * tint;
}
