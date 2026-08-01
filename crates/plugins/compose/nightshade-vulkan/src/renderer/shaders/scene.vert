#version 460
#extension GL_GOOGLE_include_directive : require

#include "scene_common.glsl"

layout(push_constant) uniform ListPush {
    uint list;
} push;

layout(location = 0) out VertexOutput {
    vec3 normal;
    vec3 world_position;
    vec3 previous_world_position;
    vec2 tex_coords;
    vec2 tex_coords_1;
    vec4 tangent;
    vec4 color;
} vertex_output;

layout(location = 7) out flat uint material_index;
layout(location = 8) out flat uint object_index;
layout(location = 9) noperspective out vec2 tex_coords_affine;

void main() {
    uint drawn = uint(gl_InstanceIndex);
    Object object = objects[drawn];
    Vertex vertex = vertices[uint(gl_VertexIndex)];

    vec4 world = object.transform * vec4(vertex.position, 1.0);
    vec4 clip = view_projection_of(view_of(push.list)) * world;
    if (frame.style.z != 0.0) {
        vec2 grid = max(frame.style.xy, vec2(1.0));
        clip.xy = round(clip.xy * grid / clip.w) * clip.w / grid;
    }
    gl_Position = clip;
    vertex_output.normal = normalize(mat3(object.normal_transform) * vertex.normal);
    vertex_output.world_position = world.xyz;
    vertex_output.previous_world_position =
        (object.previous_transform * vec4(vertex.position, 1.0)).xyz;
    vertex_output.tex_coords = vertex.tex_coords;
    vertex_output.tex_coords_1 = vertex.tex_coords_1;
    tex_coords_affine = vertex.tex_coords;
    vertex_output.tangent = vec4(
        normalize(mat3(object.transform) * vertex.tangent.xyz),
        vertex.tangent.w
    );
    vertex_output.color = vertex.color * object.tint;
    material_index = object.material_index;
    object_index = drawn;
}
