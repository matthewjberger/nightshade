#version 460

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform usampler2D entity_ids;
layout(set = 0, binding = 1) uniform Outline {
    vec4 color;
    vec4 shape;
} outline;
layout(std430, set = 0, binding = 2) readonly buffer Named { uint named_bits[]; };

bool named(uint id) {
    if (id == 0u) {
        return false;
    }
    uint word = id / 32u;
    if (word >= uint(outline.shape.y)) {
        return false;
    }
    return ((named_bits[word] >> (id % 32u)) & 1u) != 0u;
}

void main() {
    ivec2 size = textureSize(entity_ids, 0);
    ivec2 point = ivec2(uv * vec2(size));
    ivec2 last = size - ivec2(1);

    if (named(texelFetch(entity_ids, clamp(point, ivec2(0), last), 0).r)) {
        discard;
    }

    int width = int(outline.shape.x);
    for (int y = -width; y <= width; ++y) {
        for (int x = -width; x <= width; ++x) {
            ivec2 at = clamp(point + ivec2(x, y), ivec2(0), last);
            if (named(texelFetch(entity_ids, at, 0).r)) {
                out_color = outline.color;
                return;
            }
        }
    }
    discard;
}
