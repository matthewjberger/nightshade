#ifndef COMMON_PI
#define COMMON_PI
const float PI = 3.141592653589793;
#endif

vec3 cube_direction(uint face, vec2 uv) {
    float x = 2.0 * uv.x - 1.0;
    float y = 2.0 * uv.y - 1.0;
    vec3 direction;
    if (face == 0u) {
        direction = vec3(1.0, -y, -x);
    } else if (face == 1u) {
        direction = vec3(-1.0, -y, x);
    } else if (face == 2u) {
        direction = vec3(x, 1.0, y);
    } else if (face == 3u) {
        direction = vec3(x, -1.0, -y);
    } else if (face == 4u) {
        direction = vec3(x, -y, 1.0);
    } else {
        direction = vec3(-x, -y, -1.0);
    }
    return normalize(direction);
}

vec2 hammersley(uint index, uint count) {
    uint bits = index;
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return vec2(float(index) / float(count), float(bits) * 2.3283064365386963e-10);
}

mat3 orthonormal_basis(vec3 normal) {
    float sign_z = normal.z >= 0.0 ? 1.0 : -1.0;
    float a = -1.0 / (sign_z + normal.z);
    float b = normal.x * normal.y * a;
    return mat3(
        vec3(1.0 + sign_z * normal.x * normal.x * a, sign_z * b, -sign_z * normal.x),
        vec3(b, sign_z + normal.y * normal.y * a, -normal.y),
        normal
    );
}
