#version 460

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec4 in_color;

layout(location = 0) out vec4 out_color;

layout(push_constant) uniform Params {
    mat4 view_projection;
    uint on_top;
} params;

void main() {
    out_color = in_color;
    vec4 clip = params.view_projection * vec4(in_position, 1.0);
    if (params.on_top != 0u) {
        clip.z = clip.w;
    }
    gl_Position = clip;
}
