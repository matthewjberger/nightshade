struct Decal {
    mat4 model;
    mat4 inverse_model;
    vec4 color;
    vec4 emissive;
    vec4 size_depth;
    vec4 params;
};

layout(scalar, set = 0, binding = 0) uniform Camera {
    mat4 view_projection;
    mat4 inverse_view_projection;
    vec4 camera_position;
    vec4 screen;
} camera;

layout(scalar, set = 0, binding = 1) readonly buffer Decals { Decal decals[]; };
