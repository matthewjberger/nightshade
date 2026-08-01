#version 460

layout(location = 0) out vec3 world_position;

layout(push_constant) uniform Push {
    mat4 view_projection;
    vec4 camera;
    vec4 settings;
} push;

void main() {
    vec2 corners[6] = vec2[6](
        vec2(-10.0, -10.0),
        vec2(10.0, -10.0),
        vec2(-10.0, 10.0),
        vec2(-10.0, 10.0),
        vec2(10.0, -10.0),
        vec2(10.0, 10.0)
    );

    float parallel = push.settings.z;
    float scale = parallel > 0.5 ? max(10.0, push.settings.w * 100.0) : 1.0;
    vec2 corner = corners[gl_VertexIndex] * push.camera.w * scale;

    world_position = vec3(
        corner.x + push.camera.x,
        -0.01,
        corner.y + push.camera.z
    );

    vec4 clip = push.view_projection * vec4(world_position, 1.0);
    if (parallel > 0.5) {
        clip.z = clamp(clip.z, 0.0, clip.w);
    }
    gl_Position = clip;
}
