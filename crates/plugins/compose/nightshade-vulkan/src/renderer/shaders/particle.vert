#version 460
#extension GL_GOOGLE_include_directive : require

#include "particle_common.glsl"

layout(scalar, set = 0, binding = 0) uniform Camera {
    mat4 view_projection;
    vec4 camera_right;
    vec4 camera_up;
} camera;
layout(scalar, set = 0, binding = 1) readonly buffer Particles { Particle particles[]; };
layout(scalar, set = 0, binding = 2) readonly buffer AliveIndices { uint alive_indices[]; };

layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec4 out_color;
layout(location = 2) out float out_emissive;
layout(location = 3) flat out uint out_emitter_type;
layout(location = 4) flat out uint out_texture_index;

const vec2 CORNERS[6] = vec2[6](
    vec2(-0.5, -0.5),
    vec2(0.5, -0.5),
    vec2(0.5, 0.5),
    vec2(-0.5, -0.5),
    vec2(0.5, 0.5),
    vec2(-0.5, 0.5)
);

const vec2 COORDS[6] = vec2[6](
    vec2(0.0, 1.0),
    vec2(1.0, 1.0),
    vec2(1.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, 0.0),
    vec2(0.0, 0.0)
);

void main() {
    Particle particle = particles[alive_indices[gl_InstanceIndex]];
    vec2 corner = CORNERS[gl_VertexIndex];
    float size = particle.emitter_data.y;
    vec3 position = particle.position.xyz
        + camera.camera_right.xyz * corner.x * size
        + camera.camera_up.xyz * corner.y * size;

    out_uv = COORDS[gl_VertexIndex];
    out_color = particle.color;
    out_emissive = particle.emitter_data.z;
    out_emitter_type = uint(particle.emitter_data.w);
    out_texture_index = uint(particle.emitter_data.x);
    gl_Position = camera.view_projection * vec4(position, 1.0);
}
