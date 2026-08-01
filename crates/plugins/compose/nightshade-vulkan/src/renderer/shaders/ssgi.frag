#version 460

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 out_indirect;

layout(set = 0, binding = 0) uniform sampler2D scene_depth;
layout(set = 0, binding = 1) uniform sampler2D scene_normal;
layout(set = 0, binding = 2) uniform sampler2D scene_color;

layout(push_constant) uniform Push {
    mat4 inverse_projection;
    vec4 projection;
    vec4 screen;
    vec4 settings;
} push;

const float TWO_PI = 6.28318530718;

const float THICKNESS = 0.3;

vec3 view_position(vec2 point, float depth) {
    vec4 clip = vec4(point.x * 2.0 - 1.0, 1.0 - point.y * 2.0, depth, 1.0);
    vec4 view = push.inverse_projection * clip;
    return view.xyz / view.w;
}

vec2 screen_of(vec3 position) {
    vec2 ndc = vec2(push.projection.x * position.x, push.projection.y * position.y)
        / -position.z;
    return vec2(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
}

vec2 hammersley(uint index, uint count) {
    uint bits = index;
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return vec2(float(index) / float(count), float(bits) * 2.3283064365386963e-10);
}

float hash(vec2 point) {
    return fract(sin(dot(point, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
    out_indirect = vec4(0.0, 0.0, 0.0, 1.0);
    if (push.settings.z <= 0.0) {
        return;
    }

    float depth = texture(scene_depth, uv).r;
    if (depth <= 0.0) {
        return;
    }
    vec4 written = texture(scene_normal, uv);
    if (dot(written.xyz, written.xyz) < 1.0e-6) {
        return;
    }

    vec3 normal = normalize(written.xyz);
    vec3 surface = view_position(uv, depth);

    float turn = hash(floor(mod(gl_FragCoord.xy, 4.0))) * TWO_PI;
    vec3 random = vec3(cos(turn), sin(turn), 0.0);
    vec3 tangent = random - normal * dot(random, normal);
    if (dot(tangent, tangent) < 1.0e-8) {
        tangent = abs(normal.x) > 0.9 ? vec3(0.0, 1.0, 0.0) : vec3(1.0, 0.0, 0.0);
        tangent = tangent - normal * dot(tangent, normal);
    }
    tangent = normalize(tangent);
    mat3 basis = mat3(tangent, cross(normal, tangent), normal);

    uint directions = uint(push.screen.w);
    uint steps = uint(push.settings.x);
    float radius = push.screen.z;
    float step_size = radius / float(steps);
    vec3 gathered = vec3(0.0);

    for (uint index = 0u; index < directions; ++index) {
        vec2 point = hammersley(index, directions);
        float angle = point.x * TWO_PI;
        float spread = sqrt(point.y);
        vec3 direction = normalize(basis * vec3(
            cos(angle) * spread,
            sin(angle) * spread,
            sqrt(max(1.0 - point.y, 0.0))
        ));
        float weight = max(dot(direction, normal), 0.0);
        if (weight < 0.01) {
            continue;
        }

        for (uint step = 1u; step <= steps; ++step) {
            vec3 along = surface + direction * (float(step) * step_size);
            vec2 point_on_screen = screen_of(along);
            if (point_on_screen.x < 0.0 || point_on_screen.x > 1.0
                || point_on_screen.y < 0.0 || point_on_screen.y > 1.0) {
                break;
            }
            float found = texture(scene_depth, point_on_screen).r;
            if (found <= 0.0) {
                continue;
            }
            float behind = view_position(point_on_screen, found).z - along.z;
            if (behind > 0.0 && behind < THICKNESS) {
                vec3 met = min(texture(scene_color, point_on_screen).rgb, vec3(1.0));
                float travelled = float(step) * step_size;
                gathered += met * weight * (1.0 - clamp(travelled / radius, 0.0, 1.0));
                break;
            }
        }
    }

    out_indirect = vec4(gathered / float(directions), 1.0);
}
