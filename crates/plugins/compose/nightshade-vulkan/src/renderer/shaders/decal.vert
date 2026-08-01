#version 460
#extension GL_EXT_scalar_block_layout : require
#extension GL_GOOGLE_include_directive : require

#include "decal_common.glsl"

layout(location = 0) flat out uint out_instance;

const vec3 CORNERS[36] = vec3[36](
    vec3(-0.5, -0.5, -0.5), vec3(0.5, -0.5, -0.5), vec3(0.5, 0.5, -0.5),
    vec3(-0.5, -0.5, -0.5), vec3(0.5, 0.5, -0.5), vec3(-0.5, 0.5, -0.5),
    vec3(-0.5, -0.5, 0.5), vec3(0.5, 0.5, 0.5), vec3(0.5, -0.5, 0.5),
    vec3(-0.5, -0.5, 0.5), vec3(-0.5, 0.5, 0.5), vec3(0.5, 0.5, 0.5),
    vec3(-0.5, -0.5, -0.5), vec3(-0.5, 0.5, 0.5), vec3(-0.5, -0.5, 0.5),
    vec3(-0.5, -0.5, -0.5), vec3(-0.5, 0.5, -0.5), vec3(-0.5, 0.5, 0.5),
    vec3(0.5, -0.5, -0.5), vec3(0.5, -0.5, 0.5), vec3(0.5, 0.5, 0.5),
    vec3(0.5, -0.5, -0.5), vec3(0.5, 0.5, 0.5), vec3(0.5, 0.5, -0.5),
    vec3(-0.5, -0.5, -0.5), vec3(0.5, -0.5, 0.5), vec3(0.5, -0.5, -0.5),
    vec3(-0.5, -0.5, -0.5), vec3(-0.5, -0.5, 0.5), vec3(0.5, -0.5, 0.5),
    vec3(-0.5, 0.5, -0.5), vec3(0.5, 0.5, -0.5), vec3(0.5, 0.5, 0.5),
    vec3(-0.5, 0.5, -0.5), vec3(0.5, 0.5, 0.5), vec3(-0.5, 0.5, 0.5)
);

void main() {
    Decal decal = decals[gl_InstanceIndex];
    vec3 corner = CORNERS[gl_VertexIndex] * decal.size_depth.xyz;
    out_instance = gl_InstanceIndex;
    gl_Position = camera.view_projection * decal.model * vec4(corner, 1.0);
}
