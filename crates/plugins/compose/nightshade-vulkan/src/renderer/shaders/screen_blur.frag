#version 460

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform sampler2D source;
layout(set = 0, binding = 1) uniform sampler2D scene_depth;
layout(set = 0, binding = 2) uniform sampler2D scene_normal;

layout(push_constant) uniform Push {
    vec4 screen;
    vec4 shape;
} push;

void main() {
    float center_depth = texture(scene_depth, uv).r;
    bool keep_alpha = push.shape.z != 0.0;
    if (center_depth <= 0.0) {
        out_color = keep_alpha ? vec4(0.0) : vec4(0.0, 0.0, 0.0, 1.0);
        return;
    }

    vec3 center_normal = texture(scene_normal, uv).xyz;
    vec2 texel = 1.0 / push.screen.xy;
    int reach = int(push.shape.x);

    vec4 total = vec4(0.0);
    float weight = 0.0;
    for (int down = -reach; down <= reach; ++down) {
        for (int across = -reach; across <= reach; ++across) {
            vec2 step = vec2(float(across), float(down));
            vec2 point = uv + step * texel;
            if (point.x < 0.0 || point.x > 1.0 || point.y < 0.0 || point.y > 1.0) {
                continue;
            }

            float depth = texture(scene_depth, point).r;
            vec3 normal = texture(scene_normal, point).xyz;

            float by_depth = 1.0 - clamp(
                abs(center_depth - depth) / max(push.screen.z, 1.0e-6),
                0.0,
                1.0
            );
            float by_normal = pow(max(dot(center_normal, normal), 0.0), push.screen.w);
            float by_distance = exp(-dot(step, step) / max(push.shape.y, 1.0e-4));

            float share = by_depth * by_normal * by_distance;
            total += texture(source, point) * share;
            weight += share;
        }
    }

    vec4 settled = weight > 0.0 ? total / weight : texture(source, uv);
    out_color = keep_alpha ? settled : vec4(settled.rgb, 1.0);
}
