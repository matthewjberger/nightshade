#version 460

layout(location = 0) in vec2 uv;
layout(location = 0) out float out_occlusion;

layout(set = 0, binding = 0) uniform sampler2D occlusion;
layout(set = 0, binding = 1) uniform sampler2D scene_depth;
layout(set = 0, binding = 2) uniform sampler2D scene_normal;

layout(push_constant) uniform Push {
    vec4 texel;
    vec4 settings;
} push;

vec3 facing_at(vec2 point) {
    vec3 written = texture(scene_normal, point).xyz;
    return dot(written, written) > 1.0e-6 ? normalize(written) : vec3(0.0);
}

void main() {
    float middle = texture(occlusion, uv).r;
    float center = texture(scene_depth, uv).r;
    float threshold = max(push.settings.x, 1.0e-6);
    float sharpness = push.settings.y;
    vec3 middle_facing = facing_at(uv);
    if (dot(middle_facing, middle_facing) == 0.0) {
        out_occlusion = middle;
        return;
    }

    float total = middle;
    float weight = 1.0;
    for (int row = -1; row <= 1; ++row) {
        for (int column = -1; column <= 1; ++column) {
            if (row == 0 && column == 0) {
                continue;
            }
            vec2 point = uv + vec2(float(column), float(row)) * push.texel.xy;
            float depth = texture(scene_depth, point).r;
            float keep = exp(-abs(depth - center) / threshold);
            vec3 facing = facing_at(point);
            keep *= dot(facing, facing) > 0.0
                ? pow(max(dot(middle_facing, facing), 0.0), sharpness)
                : 0.0;
            total += texture(occlusion, point).r * keep;
            weight += keep;
        }
    }

    out_occlusion = total / weight;
}
