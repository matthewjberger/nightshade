#version 460

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform sampler2D accumulated;
layout(set = 0, binding = 1) uniform sampler2D revealed;

void main() {
    float showing = texture(revealed, uv).r;
    if (showing > 0.9999) {
        discard;
    }
    vec4 sum = texture(accumulated, uv);
    float weight = max(sum.a, 1.0e-5);
    out_color = vec4(sum.rgb / weight, 1.0 - showing);
}
