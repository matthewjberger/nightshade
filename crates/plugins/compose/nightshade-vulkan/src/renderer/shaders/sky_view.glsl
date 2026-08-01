layout(scalar, set = 0, binding = 0) uniform Sky {
    mat4 projection;
    mat4 view;
    vec4 settings;
    vec4 air;
    vec4 fog_color;
} sky;
