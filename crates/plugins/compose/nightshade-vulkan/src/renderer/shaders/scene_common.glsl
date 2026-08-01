#extension GL_EXT_scalar_block_layout : require

struct Vertex {
    vec3 position;
    vec3 normal;
    vec2 tex_coords;
    vec2 tex_coords_1;
    vec4 tangent;
    vec4 color;
};

struct MeshRange {
    uint vertex_offset;
    uint vertex_count;
    uint meshlet_offset;
    uint meshlet_count;
    uint morph_offset;
    uint morph_target_count;
    uint index_offset;
    uint index_count;
};

struct MorphDisplacement {
    vec3 position;
    vec3 normal;
    vec3 tangent;
};

struct Meshlet {
    vec4 bounds;
    vec4 cone;
    uint vertex_offset;
    uint triangle_offset;
    uint vertex_count;
    uint triangle_count;
    vec4 group;
    vec4 parent;
    vec4 errors;
};

#define DETAIL_ALLOWANCE 1.0

#define PHASE_EARLY 0u
#define PHASE_LATE 1u

#define PHASE_SHADE 2u

struct Object {
    mat4 transform;
    mat4 previous_transform;
    mat4 normal_transform;
    vec4 bounds;
    uint mesh_slot;
    uint material_index;
    uint culling_mask;
    uint flags;
    uint vertex_offset;
    uint previous_vertex_offset;
    uint entity_id;
    uint run;
    uint posed_meshlets;
    vec4 morph_weights[2];
    vec4 tint;
};

struct GpuMaterial {
    vec4 base_color;
    vec4 emissive;
    vec4 factors;
    vec4 alpha;
    vec4 transmission;
    vec4 attenuation;
    vec4 specular_color;
    vec4 anisotropy;
    vec4 sheen;
    vec4 sheen_color;
    vec4 detail;
    vec4 diffuse_transmission_color;
    vec4 reflection_box_min;
    vec4 reflection_box_max;
    vec4 reveal;
    vec4 reveal_noise;
    vec4 reveal_seam;
    vec4 atlas;
    uint textures[24];
    vec4 transforms[48];
};

#define NO_COOKIE 0xFFFFFFFFu

struct Light {
    vec4 position;
    vec4 direction;
    vec4 color;
    uint light_type;
    float range;
    float inner_cone;
    float outer_cone;
    int shadow_index;
    float light_size;
    uint cookie_layer;
    float casts_shadow;
};

struct AreaLight {
    vec4 right;
    vec4 up;
    uint shape;
    uint two_sided;
    uint emissive_layer;
    float radius;
};

struct SkinData {
    uint joint_count;
    uint base_bone_index;
    uint base_ibm_index;
    uint base_output_index;
};

struct SkinInstance {
    uint source_vertex_offset;
    uint skin_vertex_offset;
    uint posed_vertex_offset;
    uint vertex_count;
    uint joint_offset;
    uint bone_offset;
    uint bone_count;
    uint object_index;
};

struct SkinVertex {
    uvec4 joint_indices;
    vec4 joint_weights;
};

#define OBJECT_VISIBLE 1u
#define OBJECT_SKINNED 2u
#define OBJECT_SHADOW_CASTER 4u
#define OBJECT_DOUBLE_SIDED 8u
#define OBJECT_OVERLAY 16u
#define OBJECT_DEBUG_VIEW_EXEMPT 32u
#define NO_MATERIAL 0xFFFFFFFFu
#define REVEAL_FRONT_DISABLED 1.0e30
#define NO_TEXTURE 0xFFFFFFFFu

#define MATERIAL_DOUBLE_SIDED 1u
#define MATERIAL_UNLIT 2u
#define MATERIAL_NORMAL_FLIP_Y 4u
#define MATERIAL_NORMAL_TWO_COMPONENT 8u
#define MATERIAL_SKY_LIGHT 16u
#define MATERIAL_VERTEX_OCCLUSION 32u

vec2 atlas_uv(GpuMaterial material, vec2 uv, float cell) {
    vec2 cells = material.atlas.xy;
    if (cells.x <= 0.0) {
        return uv;
    }
    float whole = floor(cell + 0.5);
    float row = floor((whole + 0.5) / cells.x);
    float column = whole - row * cells.x;
    return (vec2(column, row) + fract(uv)) / cells;
}

vec4 surface_tint(GpuMaterial material, vec4 color) {
    if ((uint(material.alpha.z) & MATERIAL_VERTEX_OCCLUSION) != 0u) {
        return vec4(1.0, 1.0, 1.0, color.a);
    }
    return color;
}

#define ALPHA_MODE_OPAQUE 0u
#define ALPHA_MODE_MASK 1u
#define ALPHA_MODE_BLEND 2u
#define ALPHA_MODE_DITHER 3u

#define TASK_BATCH 32u
uint run_length_of(Object object) {
    return object.run >> 16u;
}

uint run_position_of(Object object) {
    return object.run & 0xFFFFu;
}

#define TASK_WIDTH 32768u

#define VIEW_COUNT 1029u
#define VIEW_CAMERA 0u
#define CASCADE_COUNT 4
#define LOCAL_SHADOW_COUNT 1024u
#define VIEW_LOCAL_BASE 5u

#define LIST_COUNT 1036u
#define LIST_OPAQUE 0u
#define LIST_DOUBLE_SIDED 1u
#define LIST_REFRACTED 2u
#define LIST_TRANSPARENT 3u
#define LIST_OVERLAY_BASE 4u
#define LIST_CASCADE_BASE 8u
#define LIST_LOCAL_BASE 12u

#define CLUSTER_X 16u
#define CLUSTER_Y 9u
#define CLUSTER_Z 24u
#define CLUSTER_CELLS (CLUSTER_X * CLUSTER_Y * CLUSTER_Z)
#define CLUSTER_HEADER 1u
#define CLUSTER_GRID_BASE CLUSTER_HEADER
#define CLUSTER_LIST_BASE (CLUSTER_HEADER + 2u * CLUSTER_CELLS)
#define CLUSTER_LIST_CAPACITY (CLUSTER_CELLS * 64u)
#define CLUSTER_WORDS (CLUSTER_LIST_BASE + CLUSTER_LIST_CAPACITY)

#define FOG_X 160u
#define FOG_Y 90u
#define FOG_Z 64u

#define LIGHT_DIRECTIONAL 0u
#define LIGHT_POINT 1u
#define LIGHT_SPOT 2u
#define LIGHT_AREA 3u

#define TEXTURE_BASE 0
#define TEXTURE_METALLIC_ROUGHNESS 1
#define TEXTURE_NORMAL 2
#define TEXTURE_OCCLUSION 3
#define TEXTURE_EMISSIVE 4
#define TEXTURE_TRANSMISSION 5
#define TEXTURE_THICKNESS 6
#define TEXTURE_SPECULAR 7
#define TEXTURE_SPECULAR_COLOR 8
#define TEXTURE_IRIDESCENCE 9
#define TEXTURE_IRIDESCENCE_THICKNESS 10
#define TEXTURE_SHEEN_COLOR 11
#define TEXTURE_SHEEN_ROUGHNESS 12
#define TEXTURE_CLEARCOAT 13
#define TEXTURE_CLEARCOAT_ROUGHNESS 14
#define TEXTURE_CLEARCOAT_NORMAL 15
#define TEXTURE_ANISOTROPY 16
#define TEXTURE_DIFFUSE_TRANSMISSION 17
#define TEXTURE_DIFFUSE_TRANSMISSION_COLOR 18
#define TEXTURE_DETAIL_BASE 19
#define TEXTURE_DETAIL_NORMAL 20
#define TEXTURE_HEIGHT 21
#define TEXTURE_SLOT_COUNT 24

layout(scalar, set = 0, binding = 0) uniform Frame {
    mat4 view;
    mat4 projection;
    mat4 view_projection;
    vec4 camera_position;
    vec4 frustum_planes[6];
    uvec4 counts;
    vec4 settings;
    mat4 cascade_view_projections[CASCADE_COUNT];
    vec4 cascade_planes[CASCADE_COUNT][6];
    vec4 cascade_splits;
    vec4 cascade_texel_sizes;
    vec4 shadow_settings;
    vec4 sun_shadow;
    mat4 previous_view_projection;
    vec4 environment;
    uvec4 skin_counts;
    uvec4 masks;
    vec4 debug;
    vec4 flat_color;
    vec4 cluster;
    vec4 screen;
    vec4 pyramid;
    vec4 fog_color;
    vec4 fog;
    vec4 style;
    vec4 fog_air;
    vec4 local_shadow;
    vec4 probe_origin;
    vec4 probe_spacing;
    uvec4 probe_counts;
    vec4 probe_settings;
    vec4 probe_rotation;
} frame;

struct LocalShadow {
    mat4 view_projection;
    vec4 planes[6];
    vec4 slot;
    vec4 light;
};

#ifdef OBJECT_WRITE
layout(scalar, set = 0, binding = 1) buffer Objects { Object objects[]; };
#else
layout(scalar, set = 0, binding = 1) readonly buffer Objects { Object objects[]; };
#endif
layout(scalar, set = 0, binding = 2) readonly buffer MeshRanges { MeshRange mesh_ranges[]; };
layout(scalar, set = 0, binding = 3) readonly buffer Meshlets { Meshlet meshlets[]; };
layout(scalar, set = 0, binding = 4) readonly buffer MeshletVertices { uint meshlet_vertices[]; };
layout(scalar, set = 0, binding = 5) readonly buffer MeshletTriangles { uint meshlet_triangles[]; };
layout(scalar, set = 0, binding = 6) readonly buffer Vertices { Vertex vertices[]; };
layout(scalar, set = 0, binding = 7) readonly buffer Materials { GpuMaterial materials[]; };
layout(scalar, set = 0, binding = 8) readonly buffer Lights { Light lights[]; };
layout(scalar, set = 0, binding = 9) readonly buffer AreaLights { AreaLight area_lights[]; };
#ifdef CULL_WRITE
#define CULL_ACCESS buffer
#else
#define CULL_ACCESS readonly buffer
#endif
layout(scalar, set = 0, binding = 10) CULL_ACCESS WorkList { uvec2 work_items[]; };
layout(scalar, set = 0, binding = 11) CULL_ACCESS DrawCommands {
    uvec4 draw_commands[];
};
layout(scalar, set = 0, binding = 12) readonly buffer BoneTransforms { mat4 bone_transforms[]; };
layout(scalar, set = 0, binding = 13) readonly buffer InverseBinds { mat4 inverse_binds[]; };
layout(scalar, set = 0, binding = 14) readonly buffer SkinDataTable { SkinData skin_data[]; };
layout(scalar, set = 0, binding = 15) buffer JointMatrices { mat4 joint_matrices[]; };
layout(scalar, set = 0, binding = 16) readonly buffer SkinInstances { SkinInstance skin_instances[]; };
layout(scalar, set = 0, binding = 17) readonly buffer SkinVertices { SkinVertex skin_vertices[]; };
layout(scalar, set = 0, binding = 18) buffer PosedVertices { Vertex posed_vertices[]; };
layout(scalar, set = 0, binding = 19) readonly buffer MorphDisplacements {
    MorphDisplacement morph_displacements[];
};
#ifdef CLUSTER_WRITE
layout(scalar, set = 0, binding = 20) buffer ClusterLights { uint cluster_lights[]; };
#else
layout(scalar, set = 0, binding = 20) readonly buffer ClusterLights { uint cluster_lights[]; };
#endif

layout(scalar, set = 0, binding = 21) CULL_ACCESS ObjectVisibility { uint object_visible[]; };

struct InstanceBatch {
    mat4 parent;
    uint first_object;
    uint count;
    uint first_local;
    uint padding;
};

layout(scalar, set = 0, binding = 22) readonly buffer InstanceBatches {
    InstanceBatch instance_batches[];
};
layout(scalar, set = 0, binding = 23) readonly buffer InstanceLocals {
    mat4 instance_locals[];
};

layout(set = 0, binding = 24) uniform sampler2D depth_pyramid;
layout(set = 0, binding = 25) uniform sampler2D scene_grab;
layout(scalar, set = 0, binding = 26) readonly buffer LocalShadows {
    LocalShadow local_shadows[];
};

struct RunDraw {
    uint index_count;
    uint instance_count;
    uint first_index;
    int vertex_offset;
    uint first_instance;
};

layout(scalar, set = 0, binding = 27) CULL_ACCESS RunDraws { RunDraw run_draws[]; };
layout(scalar, set = 0, binding = 28) CULL_ACCESS RunCounts { uint run_counts[]; };
#ifdef COUNT_WRITE
layout(scalar, set = 0, binding = 43) buffer DrawnCounts { uint drawn_counts[]; };
#else
layout(scalar, set = 0, binding = 43) readonly buffer DrawnCounts { uint drawn_counts[]; };
#endif
layout(scalar, set = 0, binding = 29) readonly buffer Indices { uint indices[]; };
#ifdef POSED_WRITE
layout(scalar, set = 0, binding = 30) buffer PosedMeshlets { vec4 posed_meshlets[]; };
#else
layout(scalar, set = 0, binding = 30) readonly buffer PosedMeshlets { vec4 posed_meshlets[]; };
#endif
#ifdef FOG_WRITE
layout(set = 0, binding = 31, rgba16f) uniform image3D fog_scatter_a;
layout(set = 0, binding = 32, rgba16f) uniform image3D fog_marched;
layout(set = 0, binding = 34, rgba16f) uniform image3D fog_scatter_b;
layout(set = 0, binding = 35) uniform sampler3D fog_history_a;
layout(set = 0, binding = 36) uniform sampler3D fog_history_b;
#endif
layout(set = 0, binding = 33) uniform sampler3D fog_volume;

#ifdef PROBE_WRITE
layout(set = 0, binding = 37, rgba16f) uniform image2D probe_irradiance_store;
layout(set = 0, binding = 38, rg16f) uniform image2D probe_distance_store;
layout(scalar, set = 0, binding = 41) buffer ProbeRays { vec4 probe_rays[]; };
#endif
layout(set = 0, binding = 39) uniform sampler2D probe_irradiance;
layout(set = 0, binding = 40) uniform sampler2D probe_distance;
#ifdef PROBE_WRITE
layout(scalar, set = 0, binding = 42) buffer ProbeOffsets { vec4 probe_offsets[]; };
#else
layout(scalar, set = 0, binding = 42) readonly buffer ProbeOffsets {
    vec4 probe_offsets[];
};
#endif

#define FOG_NEAR 0.5

uint fog_frame() {
    return uint(frame.settings.w + 0.5);
}

uint fog_parity() {
    return fog_frame() & 1u;
}

float fog_depth_of(float slice) {
    float far = max(frame.fog.y, FOG_NEAR * 2.0);
    float along = slice / float(FOG_Z);
    return FOG_NEAR + (far - FOG_NEAR) * along * along;
}

float fog_slice_dither(vec2 fragment) {
    float noise = fract(52.9829189 * fract(dot(fragment, vec2(0.06711056, 0.00583715))));
    return fract(noise + frame.settings.x * frame.settings.w * 0.6180339887) - 0.5;
}

float fog_slice_of(float view_depth) {
    float far = max(frame.fog.y, FOG_NEAR * 2.0);
    float along = sqrt(max(view_depth - FOG_NEAR, 0.0) / (far - FOG_NEAR));
    return clamp(along * float(FOG_Z), 0.0, float(FOG_Z));
}

float range_attenuation(float distance_to_light, float range) {
    if (range <= 0.0) {
        return 1.0;
    }
    float ratio = distance_to_light / range;
    float window = clamp(1.0 - ratio * ratio * ratio * ratio, 0.0, 1.0);
    float clamped = max(distance_to_light, 0.01);
    return window / (clamped * clamped);
}

float spot_attenuation(vec3 to_surface, vec3 direction, float outer_cone, float inner_cone) {
    float cosine = dot(normalize(direction), normalize(to_surface));
    if (cosine <= outer_cone) {
        return 0.0;
    }
    if (cosine < inner_cone) {
        return smoothstep(outer_cone, inner_cone, cosine);
    }
    return 1.0;
}

uint cluster_offset(uint cell) {
    return cluster_lights[CLUSTER_GRID_BASE + cell * 2u];
}

uint cluster_count(uint cell) {
    return cluster_lights[CLUSTER_GRID_BASE + cell * 2u + 1u];
}

uint cluster_light(uint offset, uint slot) {
    return cluster_lights[CLUSTER_LIST_BASE + offset + slot];
}

uint cluster_cell(vec2 fragment, float view_depth) {
    uint tile_x = min(uint(fragment.x / frame.screen.x * float(CLUSTER_X)), CLUSTER_X - 1u);
    uint tile_y = min(uint(fragment.y / frame.screen.y * float(CLUSTER_Y)), CLUSTER_Y - 1u);
    float slice = log(max(view_depth, frame.cluster.z)) * frame.cluster.x + frame.cluster.y;
    uint depth_slice = uint(clamp(slice, 0.0, float(CLUSTER_Z - 1u)));
    return depth_slice * CLUSTER_X * CLUSTER_Y + tile_y * CLUSTER_X + tile_x;
}

uint triangle_index(uint offset) {
    uint word = meshlet_triangles[offset >> 2u];
    return (word >> ((offset & 3u) * 8u)) & 0xFFu;
}

float largest_scale(mat4 transform) {
    return sqrt(max(
        dot(transform[0].xyz, transform[0].xyz),
        max(dot(transform[1].xyz, transform[1].xyz), dot(transform[2].xyz, transform[2].xyz))
    ));
}

uint view_of(uint list) {
    if (list < LIST_CASCADE_BASE) {
        return VIEW_CAMERA;
    }
    if (list < LIST_LOCAL_BASE) {
        return list - LIST_CASCADE_BASE + 1u;
    }
    return list - LIST_LOCAL_BASE + VIEW_LOCAL_BASE;
}

bool list_is_overlay(uint list) {
    return list >= LIST_OVERLAY_BASE && list < LIST_CASCADE_BASE;
}

uint list_of_material(uint material_index) {
    if (material_index == NO_MATERIAL) {
        return LIST_OPAQUE;
    }
    GpuMaterial material = materials[material_index];
    if (uint(material.alpha.y) == ALPHA_MODE_BLEND) {
        return LIST_TRANSPARENT;
    }
    if (material.transmission.x > 0.0) {
        return LIST_REFRACTED;
    }
    if ((uint(material.alpha.z) & MATERIAL_DOUBLE_SIDED) != 0u) {
        return LIST_DOUBLE_SIDED;
    }
    return LIST_OPAQUE;
}

bool draws_both_faces(Object object) {
    if ((object.flags & OBJECT_DOUBLE_SIDED) != 0u) {
        return true;
    }
    if (object.material_index == NO_MATERIAL) {
        return false;
    }
    GpuMaterial material = materials[object.material_index];
    return uint(material.alpha.y) == ALPHA_MODE_BLEND
        || material.transmission.x > 0.0
        || (uint(material.alpha.z) & MATERIAL_DOUBLE_SIDED) != 0u;
}

uint list_of_object(Object object) {
    uint list = list_of_material(object.material_index);
    if (list == LIST_OPAQUE && (object.flags & OBJECT_DOUBLE_SIDED) != 0u) {
        list = LIST_DOUBLE_SIDED;
    }
    if ((object.flags & OBJECT_OVERLAY) != 0u) {
        list += LIST_OVERLAY_BASE;
    }
    return list;
}

mat4 view_projection_of(uint view) {
    if (view == VIEW_CAMERA) {
        return frame.view_projection;
    }
    if (view < VIEW_LOCAL_BASE) {
        return frame.cascade_view_projections[view - 1u];
    }
    return local_shadows[view - VIEW_LOCAL_BASE].view_projection;
}

vec4 plane_of(uint view, int plane) {
    if (view == VIEW_CAMERA) {
        return frame.frustum_planes[plane];
    }
    if (view < VIEW_LOCAL_BASE) {
        return frame.cascade_planes[view - 1u][plane];
    }
    return local_shadows[view - VIEW_LOCAL_BASE].planes[plane];
}

bool within_reach(uint view, vec3 center, float radius) {
    if (view < VIEW_LOCAL_BASE) {
        return true;
    }
    vec4 light = local_shadows[view - VIEW_LOCAL_BASE].light;
    return distance(light.xyz, center) - radius <= light.w;
}

vec4 world_bounds(Object object) {
    if ((object.flags & OBJECT_SKINNED) != 0u) {
        return object.bounds;
    }
    return vec4(
        (object.transform * vec4(object.bounds.xyz, 1.0)).xyz,
        object.bounds.w * largest_scale(object.transform)
    );
}

bool sphere_visible(uint view, vec3 center, float radius) {
    for (int plane = 0; plane < 6; ++plane) {
        vec4 bound = plane_of(view, plane);
        if (dot(bound.xyz, center) + bound.w < -radius) {
            return false;
        }
    }
    return true;
}

bool cone_visible(vec3 center, float radius, vec3 axis, float cutoff) {
    if (dot(axis, axis) < 1.0e-6) {
        return true;
    }
    vec3 to_meshlet = center - frame.camera_position.xyz;
    float distance_to_meshlet = length(to_meshlet);
    if (distance_to_meshlet <= radius) {
        return true;
    }
    return dot(to_meshlet / distance_to_meshlet, normalize(axis))
        < cutoff + radius / distance_to_meshlet;
}

bool sphere_occluded(vec3 center, float radius) {
    if (frame.pyramid.w == 0.0) {
        return false;
    }

    vec2 lowest = vec2(1.0);
    vec2 highest = vec2(0.0);
    float nearest = 0.0;
    for (int corner = 0; corner < 8; ++corner) {
        vec3 offset = vec3(
            (corner & 1) != 0 ? radius : -radius,
            (corner & 2) != 0 ? radius : -radius,
            (corner & 4) != 0 ? radius : -radius
        );
        vec4 clip = frame.view_projection * vec4(center + offset, 1.0);
        if (clip.w <= 0.0) {
            return false;
        }
        vec3 ndc = clip.xyz / clip.w;
        vec2 uv = vec2(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
        lowest = min(lowest, uv);
        highest = max(highest, uv);
        nearest = max(nearest, ndc.z);
    }

    vec2 slack = vec2(2.0) / frame.pyramid.xy;
    lowest = clamp(lowest - slack, vec2(0.0), vec2(1.0));
    highest = clamp(highest + slack, vec2(0.0), vec2(1.0));
    if (lowest.x >= highest.x || lowest.y >= highest.y) {
        return false;
    }

    float width = (highest.x - lowest.x) * frame.pyramid.x;
    float height = (highest.y - lowest.y) * frame.pyramid.y;
    int level = int(clamp(
        ceil(log2(max(max(width, height), 1.0))),
        0.0,
        frame.pyramid.z - 1.0
    ));
    vec2 size = vec2(textureSize(depth_pyramid, level));
    ivec2 low = clamp(ivec2(floor(lowest * size)), ivec2(0), ivec2(size) - ivec2(1));
    ivec2 high = clamp(ivec2(floor(highest * size)), ivec2(0), ivec2(size) - ivec2(1));

    float furthest = 1.0;
    for (int y = low.y; y <= high.y; ++y) {
        for (int x = low.x; x <= high.x; ++x) {
            furthest = min(furthest, texelFetch(depth_pyramid, ivec2(x, y), level).r);
        }
    }
    return nearest < furthest - max(furthest * 0.02, 0.0025);
}
