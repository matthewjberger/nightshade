#version 460
#extension GL_EXT_scalar_block_layout : require
#extension GL_GOOGLE_include_directive : require

#include "sky_common.glsl"
#include "sky_view.glsl"

layout(location = 0) out vec3 world_direction;

void main() {
    vec2 corner = vec2(
        float(gl_VertexIndex / 2) * 4.0 - 1.0,
        float(gl_VertexIndex & 1) * 4.0 - 1.0
    );
    vec3 view_ray = vec3(
        corner.x / sky.projection[0][0],
        corner.y / sky.projection[1][1],
        -1.0
    );
    world_direction = transpose(mat3(sky.view)) * view_ray;
    gl_Position = vec4(corner, 0.0, 1.0);
}
