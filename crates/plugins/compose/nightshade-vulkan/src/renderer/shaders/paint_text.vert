#version 460

struct TextInstanceData {
    vec4 color;
    vec4 outline_color;
    vec4 clip_rect;
    vec4 params;
    vec4 position;
};

layout(set = 0, binding = 0) uniform GlobalUniforms {
    mat4 projection;
} globals;

layout(set = 0, binding = 1) readonly buffer TextInstances { TextInstanceData text_instances[]; };

layout(location = 0) in vec3 vertex_position;
layout(location = 1) in vec2 vertex_tex_coords;
layout(location = 2) in uint vertex_character_index;
layout(location = 3) in uint vertex_text_instance_index;
layout(location = 4) in uint vertex_flags;

layout(location = 0) out vec2 tex_coords;
layout(location = 1) out vec2 screen_pos;
layout(location = 2) flat out uint character_index;
layout(location = 3) flat out uint text_instance_index;
layout(location = 4) flat out uint flags;

void main() {
    TextInstanceData instance = text_instances[vertex_text_instance_index];
    vec2 world_pos = vertex_position.xy + instance.position.xy;

    gl_Position = globals.projection * vec4(world_pos, 0.0, 1.0);
    gl_Position.z = instance.params.y;
    tex_coords = vertex_tex_coords;
    screen_pos = world_pos;
    character_index = vertex_character_index;
    text_instance_index = vertex_text_instance_index;
    flags = vertex_flags;
}
