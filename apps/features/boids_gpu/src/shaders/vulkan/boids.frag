#version 460

layout(set = 0, binding = 2) uniform DrawUniforms {
    mat4 view_projection;
    vec4 sun;
    vec4 scale;
    vec4 planes[6];
} uniforms;

layout(location = 0) in VertexInput {
    vec3 normal;
    vec3 tint;
} fragment;

layout(location = 0) out vec4 color;
layout(location = 1) out vec4 out_normal;

void main() {
    out_normal = vec4(0.0);
    vec3 normal = normalize(fragment.normal);
    vec3 sun = normalize(uniforms.sun.xyz);
    float lit = max(dot(normal, sun), 0.0);
    float ambient = 0.25;
    color = vec4(fragment.tint * (ambient + lit * 0.9), 1.0);
}
