#version 460

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec2 in_tex_coords;
layout(location = 2) in uint in_character;
layout(location = 3) in uint in_flags;

layout(location = 0) out vec2 tex_coords;
layout(location = 1) out flat uint character;
layout(location = 2) out flat uint flags;

layout(set = 0, binding = 0) uniform Frame {
    mat4 view_projection;
    vec4 camera_position;
    vec4 camera_right;
    vec4 camera_up;
} frame;

layout(push_constant) uniform Push {
    mat4 model;
    vec4 color;
    vec4 outline_color;
    vec4 settings;
} push;

void main() {
    tex_coords = in_tex_coords;
    character = in_character;
    flags = in_flags;

    vec4 world;
    if (push.settings.z != 0.0) {
        vec3 origin = (push.model * vec4(0.0, 0.0, 0.0, 1.0)).xyz;
        vec3 offset = mat3(push.model) * in_position;
        world = vec4(
            origin
                + frame.camera_right.xyz * offset.x
                + frame.camera_up.xyz * offset.y,
            1.0
        );
    } else {
        world = push.model * vec4(in_position, 1.0);
    }
    gl_Position = frame.view_projection * world;
}
