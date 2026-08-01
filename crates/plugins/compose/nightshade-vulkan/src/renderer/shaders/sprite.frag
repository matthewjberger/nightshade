#version 460

layout(set = 1, binding = 0) uniform sampler2D atlas;

layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec4 in_color;
layout(location = 0) out vec4 out_color;

void main() {
    vec4 sampled = texture(atlas, in_uv) * in_color;
    if (sampled.a <= 0.002) {
        discard;
    }
    out_color = sampled;
}
