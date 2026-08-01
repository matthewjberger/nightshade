#define_import_path nightshade_wgpu::mesh_common

#import nightshade_wgpu::material_data::TextureTransform

const PI: f32 = 3.14159265358979323846;
const NORMAL_MAP_FLIP_Y: u32 = 1u;
const NORMAL_MAP_TWO_COMPONENT: u32 = 2u;
const ALPHA_MODE_OPAQUE: u32 = 0u;
const ALPHA_MODE_MASK: u32 = 1u;
const ALPHA_MODE_BLEND: u32 = 2u;
const ALPHA_MODE_DITHER: u32 = 3u;
const LIGHT_TYPE_DIRECTIONAL: u32 = 0u;
const LIGHT_TYPE_POINT: u32 = 1u;
const LIGHT_TYPE_SPOT: u32 = 2u;
const COOKIE_LAYER_NONE: u32 = 0xFFFFFFFFu;
const CLUSTER_GRID_X: u32 = 16u;
const CLUSTER_GRID_Y: u32 = 9u;
const CLUSTER_GRID_Z: u32 = 24u;
const MAX_LIGHTS_PER_CLUSTER: u32 = 256u;
const PBR_DEBUG_NONE: u32 = 0u;
const PBR_DEBUG_BASE_COLOR: u32 = 1u;
const PBR_DEBUG_NORMAL: u32 = 2u;
const PBR_DEBUG_METALLIC: u32 = 3u;
const PBR_DEBUG_ROUGHNESS: u32 = 4u;
const PBR_DEBUG_OCCLUSION: u32 = 5u;
const PBR_DEBUG_EMISSIVE: u32 = 6u;
const PBR_DEBUG_F: u32 = 7u;
const PBR_DEBUG_G: u32 = 8u;
const PBR_DEBUG_D: u32 = 9u;
const PBR_DEBUG_DIFFUSE: u32 = 10u;
const PBR_DEBUG_SPECULAR: u32 = 11u;
const PBR_DEBUG_MIP_RAINBOW: u32 = 12u;
const PBR_DEBUG_SHADOW: u32 = 13u;

struct Uniforms {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    camera_position: vec4<f32>,
    num_lights: vec4<u32>,
    light_view_projection: mat4x4<f32>,
    shadow_bias: f32,
    shadows_enabled: f32,
    global_unlit: f32,
    shadow_normal_bias: f32,
    snap_resolution: vec2<f32>,
    snap_enabled: u32,
    affine_enabled: u32,
    fog_color: vec3<f32>,
    fog_enabled: u32,
    fog_start: f32,
    fog_end: f32,
    cascade_count: u32,
    directional_light_size: f32,
    cascade_view_projections: array<mat4x4<f32>, 4>,
    cascade_split_distances: vec4<f32>,
    cascade_atlas_offsets: array<vec4<f32>, 4>,
    cascade_atlas_scale: vec4<f32>,
    time: f32,
    pbr_debug_mode: u32,
    directional_light_direction: vec4<f32>,
    ibl_blend_factor: f32,
    oit_z_scale: f32,
    flat_color: vec4<f32>,
    prev_view_projection: mat4x4<f32>,
    cur_view_projection: mat4x4<f32>,
    material_filtering: u32,
};

struct Light {
    position: vec4<f32>,
    direction: vec4<f32>,
    color: vec4<f32>,
    light_type: u32,
    range: f32,
    inner_cone: f32,
    outer_cone: f32,
    shadow_index: i32,
    light_size: f32,
    cookie_layer: u32,
    _padding: f32,
};

struct LightGrid {
    offset: u32,
    count: u32,
};

struct ClusterUniforms {
    inverse_projection: mat4x4<f32>,
    screen_size: vec2<f32>,
    z_near: f32,
    z_far: f32,
    cluster_count: vec4<u32>,
    tile_size: vec2<f32>,
    num_lights: u32,
    num_directional_lights: u32,
};

@group(1) @binding(9) var<uniform> cluster_uniforms: ClusterUniforms;

fn get_cluster_index(frag_coord: vec2<f32>, linear_depth: f32) -> u32 {
    let tile = vec2<u32>(
        u32(frag_coord.x / cluster_uniforms.tile_size.x),
        u32(frag_coord.y / cluster_uniforms.tile_size.y)
    );

    let log_ratio = log(cluster_uniforms.z_far / cluster_uniforms.z_near);
    let safe_depth = max(linear_depth, cluster_uniforms.z_near);
    let slice = u32(log(safe_depth / cluster_uniforms.z_near) / log_ratio * f32(cluster_uniforms.cluster_count.z));
    let clamped_slice = clamp(slice, 0u, cluster_uniforms.cluster_count.z - 1u);
    let clamped_tile_x = clamp(tile.x, 0u, cluster_uniforms.cluster_count.x - 1u);
    let clamped_tile_y = clamp(tile.y, 0u, cluster_uniforms.cluster_count.y - 1u);

    return clamped_tile_x +
           clamped_tile_y * cluster_uniforms.cluster_count.x +
           clamped_slice * cluster_uniforms.cluster_count.x * cluster_uniforms.cluster_count.y;
}

struct MorphDisplacement {
    position: vec3<f32>,
    padding_position: f32,
    normal: vec3<f32>,
    padding_normal: f32,
    tangent: vec3<f32>,
    padding_tangent: f32,
};

struct SpotlightShadowData {
    view_projection: mat4x4<f32>,
    atlas_offset: vec2<f32>,
    atlas_scale: vec2<f32>,
    bias: f32,
    padding_first: f32,
    padding_second: f32,
    padding_third: f32,
};

fn refract_safe(view: vec3<f32>, normal: vec3<f32>, eta: f32) -> vec3<f32> {
    let refraction = refract(-view, normal, eta);
    let len_sq = dot(refraction, refraction);
    if len_sq > 0.0001 {
        return refraction / sqrt(len_sq);
    }
    return -view;
}

fn safe_normalize(v: vec3<f32>) -> vec3<f32> {
    let len_sq = dot(v, v);
    if len_sq < 0.0001 {
        return vec3<f32>(0.0, 1.0, 0.0);
    }
    return v / sqrt(len_sq);
}

fn rotate_offset(offset: vec2<f32>, rotation: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        rotation.x * offset.x - rotation.y * offset.y,
        rotation.y * offset.x + rotation.x * offset.y,
    );
}

struct SnappedVertex {
    clip: vec4<f32>,
    world: vec3<f32>,
}

fn snap_to_the_grid(
    clip: vec4<f32>,
    world: vec3<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    resolution: vec2<f32>,
) -> SnappedVertex {
    let snapped_x = round(clip.x * resolution.x / clip.w) * clip.w / resolution.x;
    let snapped_y = round(clip.y * resolution.y / clip.w) * clip.w / resolution.y;
    let shift = vec3<f32>(
        (snapped_x - clip.x) / projection[0][0],
        (snapped_y - clip.y) / projection[1][1],
        0.0,
    );
    let rotation = mat3x3<f32>(view[0].xyz, view[1].xyz, view[2].xyz);

    var out: SnappedVertex;
    out.clip = vec4<f32>(snapped_x, snapped_y, clip.z, clip.w);
    out.world = world + transpose(rotation) * shift;
    return out;
}

fn interleaved_gradient_noise(p: vec2<f32>) -> f32 {
    return fract(52.9829189 * fract(dot(p, vec2<f32>(0.06711056, 0.00583715))));
}

fn anchored_gradient_noise(world_pos: vec3<f32>) -> f32 {
    let cell = floor(world_pos * 16.0);
    return fract(sin(dot(cell, vec3<f32>(12.9898, 78.233, 37.719))) * 43758.5453);
}

fn mip_rainbow_color(uv_ddx: vec2<f32>, uv_ddy: vec2<f32>, texture_size: f32) -> vec3<f32> {
    let dx = uv_ddx * texture_size;
    let dy = uv_ddy * texture_size;
    let max_dt = max(dot(dx, dx), dot(dy, dy));
    let lod = max(0.0, 0.5 * log2(max_dt));
    let band = u32(floor(lod));
    switch band {
        case 0u: { return vec3<f32>(1.0, 1.0, 1.0); }
        case 1u: { return vec3<f32>(1.0, 0.2, 0.2); }
        case 2u: { return vec3<f32>(0.2, 1.0, 0.2); }
        case 3u: { return vec3<f32>(0.2, 0.4, 1.0); }
        case 4u: { return vec3<f32>(1.0, 0.2, 1.0); }
        default: { return vec3<f32>(1.0, 1.0, 0.2); }
    }
}

fn texture_uv(primary_uv: vec2<f32>, secondary_uv: vec2<f32>, transform: TextureTransform, time: f32) -> vec2<f32> {
    let uv_set = u32(transform.row_u.w);
    let uv = select(primary_uv, secondary_uv, uv_set == 1u);
    let homogeneous = vec3<f32>(uv, 1.0);
    let u = dot(transform.row_u.xyz, homogeneous);
    let v = dot(transform.row_v.xyz, homogeneous);
    return vec2<f32>(u, v) + transform.drift.xy * time;
}

fn get_normal(
    world_normal: vec3<f32>,
    world_tangent: vec4<f32>,
    normal_sample: vec3<f32>,
    has_normal_texture: bool,
    normal_scale: f32,
    flags: u32,
) -> vec3<f32> {
    let N = safe_normalize(world_normal);
    if !has_normal_texture {
        return N;
    }

    var sample_xy = normal_sample.xy * 2.0 - 1.0;
    if (flags & NORMAL_MAP_FLIP_Y) != 0u {
        sample_xy.y = -sample_xy.y;
    }
    var sample_z: f32;
    if (flags & NORMAL_MAP_TWO_COMPONENT) != 0u {
        sample_z = sqrt(max(1.0 - dot(sample_xy, sample_xy), 0.0));
    } else {
        sample_z = normal_sample.z * 2.0 - 1.0;
    }

    let tangent_normal = vec3<f32>(sample_xy * normal_scale, sample_z);
    var T = safe_normalize(world_tangent.xyz);
    T = safe_normalize(T - N * dot(N, T));
    let B = cross(N, T) * world_tangent.w;
    let TBN = mat3x3<f32>(T, B, N);
    return safe_normalize(TBN * tangent_normal);
}

