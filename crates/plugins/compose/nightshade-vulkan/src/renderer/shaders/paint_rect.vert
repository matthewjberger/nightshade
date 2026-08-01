#version 460

struct RectInstance {
    vec4 position_size;
    vec4 color;
    vec4 border_color;
    vec4 clip_rect;
    vec4 params;
    vec4 shadow_color;
    vec4 shadow_params;
    vec4 effect_params;
    vec4 quad_corner_01;
    vec4 quad_corner_23;
    uint effect_kind;
    uint is_quad;
    uint padding_0;
    uint padding_1;
};

layout(set = 0, binding = 0) uniform GlobalUniforms {
    mat4 projection;
} globals;

layout(set = 0, binding = 1) readonly buffer Instances { RectInstance instances[]; };

layout(location = 0) in vec2 vertex_position;

layout(location = 0) out vec2 local_pos;
layout(location = 1) out vec2 rect_size_out;
layout(location = 2) out vec2 screen_pos;
layout(location = 3) flat out uint slot_index;

float shadow_margin(RectInstance instance) {
    if (instance.shadow_color.a <= 0.0) {
        return 0.0;
    }
    vec2 offset = instance.shadow_params.xy;
    float blur = instance.shadow_params.z;
    float spread = instance.shadow_params.w;
    return blur * 1.5 + spread + max(abs(offset.x), abs(offset.y));
}

void main() {
    uint slot = uint(gl_InstanceIndex);
    RectInstance instance = instances[slot];

    vec2 rect_pos = instance.position_size.xy;
    vec2 rect_size = instance.position_size.zw;

    vec2 world_pos;
    vec2 local;
    if (instance.is_quad != 0u) {
        vec2 corner_0 = instance.quad_corner_01.xy;
        vec2 corner_1 = instance.quad_corner_01.zw;
        vec2 corner_2 = instance.quad_corner_23.xy;
        vec2 corner_3 = instance.quad_corner_23.zw;
        vec2 top = mix(corner_0, corner_1, vertex_position.x);
        vec2 bottom = mix(corner_3, corner_2, vertex_position.x);
        world_pos = mix(top, bottom, vertex_position.y);
        local = vertex_position * rect_size;
    } else {
        float margin = shadow_margin(instance);
        vec2 extended_size = rect_size + vec2(2.0 * margin);
        local = vertex_position * extended_size - vec2(margin);

        float rotation = instance.params.w;
        vec2 center = rect_size * 0.5;
        vec2 centered = local - center;
        float cos_r = cos(rotation);
        float sin_r = sin(rotation);
        vec2 rotated = vec2(
            centered.x * cos_r - centered.y * sin_r,
            centered.x * sin_r + centered.y * cos_r
        );
        world_pos = rect_pos + center + rotated;
    }

    gl_Position = globals.projection * vec4(world_pos, 0.0, 1.0);
    gl_Position.z = instance.params.z;
    local_pos = local;
    rect_size_out = rect_size;
    screen_pos = world_pos;
    slot_index = slot;
}
