#version 460

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform sampler2D source;

layout(push_constant) uniform Params {
    float filter_radius;
} params;

void main() {
    float x = params.filter_radius;
    float y = params.filter_radius;

    vec3 a = texture(source, uv + vec2(-x, y)).rgb;
    vec3 b = texture(source, uv + vec2(0.0, y)).rgb;
    vec3 c = texture(source, uv + vec2(x, y)).rgb;

    vec3 d = texture(source, uv + vec2(-x, 0.0)).rgb;
    vec3 e = texture(source, uv).rgb;
    vec3 f = texture(source, uv + vec2(x, 0.0)).rgb;

    vec3 g = texture(source, uv + vec2(-x, -y)).rgb;
    vec3 h = texture(source, uv + vec2(0.0, -y)).rgb;
    vec3 i = texture(source, uv + vec2(x, -y)).rgb;

    vec3 result = e * 4.0;
    result += (b + d + f + h) * 2.0;
    result += a + c + g + i;
    out_color = vec4(result * (1.0 / 32.0), 1.0);
}
