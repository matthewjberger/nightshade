
layout(std430, set = 0, binding = 0) buffer Vertices { float vertices[]; };

layout(push_constant) uniform Batch {
    uint count;
    uint first_end;
} batch;

void put_end(uint slot, vec3 position, vec4 color) {
    uint base = slot * 7u;
    vertices[base] = position.x;
    vertices[base + 1u] = position.y;
    vertices[base + 2u] = position.z;
    vertices[base + 3u] = color.r;
    vertices[base + 4u] = color.g;
    vertices[base + 5u] = color.b;
    vertices[base + 6u] = color.a;
}
