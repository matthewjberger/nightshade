#version 460
#extension GL_EXT_scalar_block_layout : require
#extension GL_GOOGLE_include_directive : require

#include "sprite_common.glsl"

layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec4 out_color;

const uint FLAG_BILLBOARD = 1u;

const vec2 CORNERS[6] = vec2[6](
    vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(1.0, 1.0),
    vec2(1.0, 1.0), vec2(0.0, 1.0), vec2(0.0, 0.0)
);

void main() {
    Sprite sprite = sprites[gl_InstanceIndex];
    vec2 corner = CORNERS[gl_VertexIndex];
    vec2 square = (corner - sprite.anchor) * sprite.size;
    vec2 local = vec2(square.x + sprite.lean * square.y, square.y);

    vec4 world;
    if ((sprite.flags & FLAG_BILLBOARD) != 0u) {
        float scale_x = length(sprite.model[0].xyz);
        float scale_y = length(sprite.model[1].xyz);
        vec3 center = sprite.model[3].xyz;
        vec3 offset = camera.camera_right.xyz * (local.x * scale_x)
            + camera.camera_up.xyz * (local.y * scale_y);
        world = vec4(center + offset, 1.0);
    } else {
        world = sprite.model * vec4(local, 0.0, 1.0);
    }

    gl_Position = camera.projection * camera.view * world;
    out_uv = mix(sprite.uv_min, sprite.uv_max, vec2(corner.x, 1.0 - corner.y));
    out_color = sprite.color;
}
