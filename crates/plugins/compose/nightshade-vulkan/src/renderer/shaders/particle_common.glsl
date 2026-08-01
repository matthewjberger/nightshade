#extension GL_EXT_scalar_block_layout : require

struct Particle {
    vec4 position;
    vec4 velocity;
    vec4 color;
    vec4 size_lifetime;
    vec4 emitter_data;
    vec4 physics;
    vec4 color_start;
    vec4 color_end;
};

struct EmitterData {
    vec4 position;
    vec4 direction;
    vec4 velocity_range;
    vec4 lifetime_range;
    vec4 size_range;
    vec4 gravity;
    vec4 color_gradient[16];
    uint gradient_count;
    uint spawn_count;
    uint emitter_id;
    uint shape_type;
    vec4 shape_params;
    vec4 turbulence;
    float emissive_strength;
    float drag;
    uint emitter_type;
    uint texture_index;
    vec4 size_curve[8];
    uint size_curve_count;
    uint size_curve_padding[3];
    vec4 opacity_curve[8];
    uint opacity_curve_count;
    uint opacity_curve_padding[3];
};

#define EMITTER_FIREWORK 0u
#define EMITTER_FIRE 1u
#define EMITTER_SMOKE 2u
#define EMITTER_SPARKS 3u
#define EMITTER_TRAIL 4u

#define SHAPE_POINT 0u
#define SHAPE_SPHERE 1u
#define SHAPE_CONE 2u
#define SHAPE_BOX 3u
