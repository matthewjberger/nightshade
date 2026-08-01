#version 460

layout(location = 0) out vec2 uv;

void main() {
    vec2 corner = vec2(
        float((gl_VertexIndex << 1) & 2),
        float(gl_VertexIndex & 2)
    );
    uv = vec2(corner.x, 1.0 - corner.y);
    gl_Position = vec4(corner * 2.0 - 1.0, 0.0, 1.0);
}
