#version 460

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform sampler2D source;

layout(push_constant) uniform Params {
    float threshold;
    float knee;
    float exposure;
    uint level;
} params;

float luminance(vec3 color) {
    return dot(color, vec3(0.2126, 0.7152, 0.0722));
}

float karis_average(vec3 color) {
    return 1.0 / (1.0 + luminance(color) * 0.25);
}

vec3 soft_threshold(vec3 color) {
    vec3 exposed = color * params.exposure;
    float brightness = max(exposed.r, max(exposed.g, exposed.b));
    float soft = clamp(brightness - params.threshold + params.knee, 0.0, 2.0 * params.knee);
    float contribution = params.knee > 0.0
        ? soft * soft / (4.0 * params.knee + 0.00001)
        : 0.0;
    float total = max(brightness - params.threshold, contribution);
    return color * (max(total, 0.0) / max(brightness, 0.00001));
}

void main() {
    vec2 texel = 1.0 / vec2(textureSize(source, 0));
    float x = texel.x;
    float y = texel.y;

    vec3 a = texture(source, uv + vec2(-2.0 * x, 2.0 * y)).rgb;
    vec3 b = texture(source, uv + vec2(0.0, 2.0 * y)).rgb;
    vec3 c = texture(source, uv + vec2(2.0 * x, 2.0 * y)).rgb;

    vec3 d = texture(source, uv + vec2(-2.0 * x, 0.0)).rgb;
    vec3 e = texture(source, uv).rgb;
    vec3 f = texture(source, uv + vec2(2.0 * x, 0.0)).rgb;

    vec3 g = texture(source, uv + vec2(-2.0 * x, -2.0 * y)).rgb;
    vec3 h = texture(source, uv + vec2(0.0, -2.0 * y)).rgb;
    vec3 i = texture(source, uv + vec2(2.0 * x, -2.0 * y)).rgb;

    vec3 j = texture(source, uv + vec2(-x, y)).rgb;
    vec3 k = texture(source, uv + vec2(x, y)).rgb;
    vec3 l = texture(source, uv + vec2(-x, -y)).rgb;
    vec3 m = texture(source, uv + vec2(x, -y)).rgb;

    vec3 result;
    if (params.level == 0u) {
        vec3 at = soft_threshold(a);
        vec3 bt = soft_threshold(b);
        vec3 ct = soft_threshold(c);
        vec3 dt = soft_threshold(d);
        vec3 et = soft_threshold(e);
        vec3 ft = soft_threshold(f);
        vec3 gt = soft_threshold(g);
        vec3 ht = soft_threshold(h);
        vec3 it = soft_threshold(i);
        vec3 jt = soft_threshold(j);
        vec3 kt = soft_threshold(k);
        vec3 lt = soft_threshold(l);
        vec3 mt = soft_threshold(m);

        vec3 groups[5];
        groups[0] = (at + bt + dt + et) * (0.125 / 4.0);
        groups[1] = (bt + ct + et + ft) * (0.125 / 4.0);
        groups[2] = (dt + et + gt + ht) * (0.125 / 4.0);
        groups[3] = (et + ft + ht + it) * (0.125 / 4.0);
        groups[4] = (jt + kt + lt + mt) * (0.5 / 4.0);
        result = groups[0] * karis_average(groups[0])
            + groups[1] * karis_average(groups[1])
            + groups[2] * karis_average(groups[2])
            + groups[3] * karis_average(groups[3])
            + groups[4] * karis_average(groups[4]);
    } else {
        result = e * 0.125;
        result += (a + c + g + i) * 0.03125;
        result += (b + d + f + h) * 0.0625;
        result += (j + k + l + m) * 0.125;
    }

    out_color = vec4(max(result, vec3(0.0)), 1.0);
}
