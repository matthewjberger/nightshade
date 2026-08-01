layout(set = 0, binding = 0) readonly buffer PositionsIn { vec4 positions_in[]; };
layout(set = 0, binding = 1) readonly buffer VelocitiesIn { vec4 velocities_in[]; };
layout(set = 0, binding = 2) writeonly buffer PositionsOut { vec4 positions_out[]; };
layout(set = 0, binding = 3) writeonly buffer VelocitiesOut { vec4 velocities_out[]; };
layout(set = 0, binding = 4) buffer CellCounts { uint cell_counts[]; };
layout(set = 0, binding = 5) buffer CellItems { uint cell_items[]; };

layout(set = 0, binding = 6) uniform Uniforms {
    vec4 step_values;
    vec4 weights;
    uvec4 counts;
    vec4 search;
} uniforms;

uint cell_axis(float value) {
    float bounds = uniforms.step_values.y;
    float cell_size = uniforms.step_values.z;
    int dimension = int(uniforms.counts.y);
    int cell = int(floor((value + bounds) / cell_size));
    return uint(clamp(cell, 0, dimension - 1));
}

uint cell_of(vec3 position) {
    uint dimension = uniforms.counts.y;
    uint x = cell_axis(position.x);
    uint y = cell_axis(position.y);
    uint z = cell_axis(position.z);
    return (z * dimension + y) * dimension + x;
}
