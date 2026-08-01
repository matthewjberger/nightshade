#version 460

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 out_wave;
layout(location = 1) out vec4 out_noise;

layout(set = 0, binding = 0) uniform sampler2D wave;
layout(set = 0, binding = 1) uniform sampler2D noise;
layout(set = 0, binding = 2) uniform sampler2D scene_depth;
layout(set = 0, binding = 3) uniform sampler2D scene_normal;

layout(push_constant) uniform Push {
    vec4 screen;
    vec4 edges;
} push;

const float KERNEL[5] = float[5](0.0625, 0.25, 0.375, 0.25, 0.0625);

float luminance(vec3 color) {
    return dot(color, vec3(0.2126, 0.7152, 0.0722));
}

void main() {
    vec4 middle = texture(wave, uv);
    float varies_here = texture(noise, uv).r;
    out_wave = middle;
    out_noise = vec4(varies_here, 0.0, 0.0, 0.0);

    float depth = texture(scene_depth, uv).r;
    vec3 written = texture(scene_normal, uv).xyz;
    if (dot(written, written) < 1.0e-6) {
        return;
    }
    vec3 normal = normalize(written);
    float brightness = luminance(middle.rgb);

    vec2 texel = 1.0 / max(push.screen.xy, vec2(1.0));
    float varies = 0.0;
    float shares = 0.0;
    for (int down = -1; down <= 1; ++down) {
        for (int across = -1; across <= 1; ++across) {
            float share = KERNEL[down + 2] * KERNEL[across + 2];
            varies += texture(noise, uv + vec2(across, down) * texel).r * share;
            shares += share;
        }
    }
    varies = shares > 0.0 ? varies / shares : varies_here;
    float allowed = push.screen.w * sqrt(max(varies, 0.0)) + 1.0e-4;

    float step_size = max(push.screen.z, 1.0);
    float own = KERNEL[2] * KERNEL[2];
    vec4 gathered = middle * own;
    float gathered_noise = varies_here * own * own;
    float total = own;
    float squared_total = own * own;

    for (int down = -2; down <= 2; ++down) {
        for (int across = -2; across <= 2; ++across) {
            if (down == 0 && across == 0) {
                continue;
            }
            vec2 point = uv + vec2(across, down) * step_size * texel;
            if (point.x < 0.0 || point.x > 1.0 || point.y < 0.0 || point.y > 1.0) {
                continue;
            }
            vec3 near_normal = texture(scene_normal, point).xyz;
            if (dot(near_normal, near_normal) < 1.0e-6) {
                continue;
            }
            float near_depth = texture(scene_depth, point).r;
            float apart = abs(near_depth - depth) / max(abs(depth), 1.0e-5);
            float alike = pow(max(dot(normalize(near_normal), normal), 0.0), push.edges.y);
            vec4 near = texture(wave, point);
            float strayed = abs(luminance(near.rgb) - brightness) / allowed;

            float weight = KERNEL[down + 2] * KERNEL[across + 2]
                * exp(-apart / max(push.edges.x, 1.0e-5) - strayed)
                * alike;
            if (weight <= 1.0e-6) {
                continue;
            }
            gathered += near * weight;
            gathered_noise += texture(noise, point).r * weight * weight;
            total += weight;
            squared_total += weight * weight;
        }
    }
    if (total <= 1.0e-6) {
        return;
    }
    out_wave = gathered / total;
    out_noise = vec4(gathered_noise / max(squared_total, 1.0e-8), 0.0, 0.0, 0.0);
}
