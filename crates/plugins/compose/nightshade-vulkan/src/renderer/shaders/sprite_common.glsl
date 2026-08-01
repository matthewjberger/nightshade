struct Sprite {
    mat4 model;
    vec2 uv_min;
    vec2 uv_max;
    vec2 size;
    vec2 anchor;
    vec4 color;
    uint flags;
    float lean;
    uvec2 padding;
};

layout(scalar, set = 0, binding = 0) uniform Camera {
    mat4 view;
    mat4 projection;
    vec4 camera_right;
    vec4 camera_up;
} camera;

layout(scalar, set = 0, binding = 1) readonly buffer Sprites { Sprite sprites[]; };
