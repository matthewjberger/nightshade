#version 460

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 out_wave;
layout(location = 1) out vec4 out_noise;

layout(set = 0, binding = 0) uniform sampler2D kept;
layout(set = 0, binding = 1) uniform sampler2D moments;
layout(set = 0, binding = 2) uniform sampler2D scene_depth;
layout(set = 0, binding = 3) uniform sampler2D scene_normal;

layout(push_constant) uniform Push {
    vec4 screen;
    vec4 edges;
} push;

float luminance(vec3 color) {
    return dot(color, vec3(0.2126, 0.7152, 0.0722));
}

float belongs(vec2 point, float depth, vec3 normal) {
    float near_depth = texture(scene_depth, point).r;
    vec3 near_normal = texture(scene_normal, point).xyz;
    if (dot(near_normal, near_normal) < 1.0e-6) {
        return 0.0;
    }
    float apart = abs(near_depth - depth) / max(abs(depth), 1.0e-5);
    float alike = pow(max(dot(normalize(near_normal), normal), 0.0), push.edges.y);
    return exp(-apart / max(push.edges.x, 1.0e-5)) * alike;
}

void main() {
    vec4 gathered = texture(kept, uv);
    vec4 running = texture(moments, uv);
    out_wave = gathered;
    out_noise = vec4(max(running.y - running.x * running.x, 0.0), 0.0, 0.0, 0.0);

    if (running.w >= push.screen.z) {
        return;
    }

    float depth = texture(scene_depth, uv).r;
    vec3 written = texture(scene_normal, uv).xyz;
    if (dot(written, written) < 1.0e-6) {
        return;
    }
    vec3 normal = normalize(written);

    vec2 texel = 1.0 / max(push.screen.xy, vec2(1.0));
    int reach = int(max(push.screen.w, 1.0));
    float mean = 0.0;
    float squared = 0.0;
    float total = 0.0;
    for (int down = -reach; down <= reach; ++down) {
        for (int across = -reach; across <= reach; ++across) {
            vec2 point = uv + vec2(across, down) * texel;
            float weight = belongs(point, depth, normal);
            if (weight <= 0.0) {
                continue;
            }
            float brightness = luminance(texture(kept, point).rgb);
            mean += brightness * weight;
            squared += brightness * brightness * weight;
            total += weight;
        }
    }
    if (total <= 1.0e-5) {
        return;
    }
    mean /= total;
    squared /= total;
    float trust = max(push.screen.z, 1.0);
    float widen = trust / max(running.w, 1.0);
    out_noise.r = max(squared - mean * mean, 0.0) * widen;
}
