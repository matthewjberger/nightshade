#version 460

struct ImageInstance {
    vec2 position;
    vec2 size;
    vec2 uv_min;
    vec2 uv_max;
    vec4 color;
    vec4 clip_rect;
    uint texture_layer;
    float depth;
    float rotation;
    uint flags;
};

layout(set = 0, binding = 0) uniform GlobalUniforms {
    mat4 projection;
} globals;

layout(set = 0, binding = 1) readonly buffer Instances { ImageInstance instances[]; };

layout(location = 0) in vec2 corner;

layout(location = 0) out vec2 uv;
layout(location = 1) out vec4 tint;
layout(location = 2) out vec2 screen_pos;
layout(location = 3) flat out vec4 clip_rect;
layout(location = 4) flat out uint texture_layer;

void main() {
    ImageInstance instance = instances[gl_InstanceIndex];
    vec2 center = instance.position + instance.size * 0.5;
    vec2 offset = (corner - vec2(0.5, 0.5)) * instance.size;
    float cos_r = cos(instance.rotation);
    float sin_r = sin(instance.rotation);
    vec2 turned = vec2(
        offset.x * cos_r - offset.y * sin_r,
        offset.x * sin_r + offset.y * cos_r
    );
    vec2 world_pos = center + turned;

    gl_Position = globals.projection * vec4(world_pos, 0.0, 1.0);
    gl_Position.z = instance.depth;
    uv = mix(instance.uv_min, instance.uv_max, corner);
    tint = instance.color;
    screen_pos = world_pos;
    clip_rect = instance.clip_rect;
    texture_layer = instance.texture_layer;
}
