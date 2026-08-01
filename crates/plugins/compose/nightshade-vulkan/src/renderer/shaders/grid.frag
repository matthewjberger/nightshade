#version 460

layout(location = 0) in vec3 world_position;
layout(location = 0) out vec4 out_color;
layout(location = 1) out vec4 out_normal;

layout(push_constant) uniform Push {
    mat4 view_projection;
    vec4 camera;
    vec4 settings;
} push;

float log10(float value) {
    return log2(value) / log2(10.0);
}

float along(float position, float size) {
    return position - size * floor(position / size);
}

float lines_of(float size, vec2 width) {
    vec2 within = vec2(
        along(world_position.x, size),
        along(world_position.z, size)
    ) / width;
    vec2 brightness = vec2(1.0) - abs(clamp(within, 0.0, 1.0) * 2.0 - vec2(1.0));
    return max(brightness.x, brightness.y);
}

void main() {
    out_normal = vec4(0.0);

    vec2 change = vec2(
        length(vec2(dFdx(world_position.x), dFdy(world_position.x))),
        length(vec2(dFdx(world_position.z), dFdy(world_position.z)))
    );
    float across = length(change);

    float parallel = push.settings.z;
    float parallel_scale = push.settings.w;
    float scaled = parallel_scale > 1.0 ? across * parallel_scale : across;
    float step = max(
        0.0,
        log10(scaled * push.settings.x / push.settings.y) + 1.0
    );

    float fine = push.settings.y * pow(10.0, floor(step));
    float middle = fine * 10.0;
    float coarse = middle * 10.0;
    vec2 width = change * 8.0;

    float fine_lines = lines_of(fine, width);
    float middle_lines = lines_of(middle, width);
    float coarse_lines = lines_of(coarse, width);
    float fade = fract(step);

    const vec4 THIN = vec4(0.75, 0.75, 0.75, 0.25);
    const vec4 THICK = vec4(0.2, 0.4, 0.8, 0.4);

    vec4 color;
    if (coarse_lines > 0.0) {
        color = THICK;
        color.a *= coarse_lines * 0.7;
    } else if (middle_lines > 0.0) {
        color = mix(THICK, THIN, smoothstep(0.2, 0.8, fade));
        color.a *= middle_lines * 0.5;
    } else {
        color = THIN;
        color.a *= fine_lines * (1.0 - fade) * 0.4;
    }

    if (parallel <= 0.5) {
        float away = length(world_position.xz - push.camera.xz);
        color.a *= 1.0 - smoothstep(0.8 * push.camera.w, push.camera.w * 3.0, away);
    }

    if (abs(world_position.z) < 0.03) {
        color = mix(color, vec4(0.87, 0.26, 0.24, 0.7), 0.5);
    }
    if (abs(world_position.x) < 0.03) {
        color = mix(color, vec4(0.24, 0.7, 0.29, 0.7), 0.5);
    }

    if (color.a < 0.02) {
        discard;
    }
    out_color = color;
}
