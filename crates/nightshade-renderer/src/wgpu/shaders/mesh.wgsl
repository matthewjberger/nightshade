const PI: f32 = 3.14159265358979323846;

#import nightshade_renderer::pbr_brdf::{DistributionGGX, GeometrySchlickGGX, GeometrySmith, V_SmithGGXCorrelated, fresnelSchlick, fresnelSchlickRoughness, specularAntiAliasing, fresnel_clearcoat, clearcoat_specular_lobe, d_charlie, v_neubelt, sheen_lobe, fresnel0_to_ior, ior_to_fresnel0_v, ior_to_fresnel0_f, eval_sensitivity, d_ggx_anisotropic, v_ggx_anisotropic, eval_iridescence, schlick_to_f0, getRangeAttenuation, getSpotAttenuation, applyVolumeAttenuation, applyIorToRoughness, specular_occlusion, box_project_reflection, box_projected_roughness}


fn getTransmissionSample(reflection: vec3<f32>, roughness: f32, ior: f32) -> vec3<f32> {
    let transmission_roughness = applyIorToRoughness(roughness, ior);
    let MAX_REFLECTION_LOD = 4.0;
    return textureSampleLevel(prefiltered_env, prefiltered_sampler, reflection, transmission_roughness * MAX_REFLECTION_LOD).rgb;
}

fn getScreenSpaceTransmission(
    world_pos: vec3<f32>,
    refracted_dir: vec3<f32>,
    thickness: f32,
    ior: f32,
    roughness: f32,
    screen_width: f32,
) -> vec3<f32> {
    let exit_world = world_pos + refracted_dir * max(thickness, 0.001);
    let exit_clip = uniforms.projection * uniforms.view * vec4<f32>(exit_world, 1.0);
    let ibl_sample = getTransmissionSample(refracted_dir, roughness, ior);
    if exit_clip.w <= 0.0001 {
        return ibl_sample;
    }
    let exit_ndc = exit_clip.xyz / exit_clip.w;
    let uv = vec2<f32>(exit_ndc.x * 0.5 + 0.5, 0.5 - exit_ndc.y * 0.5);
    let clamped_uv = clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0));
    let refraction_lod = log2(max(screen_width, 2.0)) * applyIorToRoughness(roughness, ior);
    let scene_sample = textureSampleLevel(transmission_color_texture, transmission_color_sampler, clamped_uv, refraction_lod).rgb;
    let edge_dist = min(min(uv.x, uv.y), min(1.0 - uv.x, 1.0 - uv.y));
    let blend = clamp(edge_dist * 8.0, 0.0, 1.0);
    return mix(ibl_sample, scene_sample, blend);
}

fn refract_safe(view: vec3<f32>, normal: vec3<f32>, eta: f32) -> vec3<f32> {
    let refraction = refract(-view, normal, eta);
    let len_sq = dot(refraction, refraction);
    if len_sq > 0.0001 {
        return refraction / sqrt(len_sq);
    }
    return -view;
}

fn getIBLVolumeRefraction(
    normal: vec3<f32>,
    view: vec3<f32>,
    world_pos: vec3<f32>,
    roughness: f32,
    base_color: vec3<f32>,
    F0: vec3<f32>,
    ior: f32,
    dispersion: f32,
    thickness: f32,
    attenuation_color: vec3<f32>,
    attenuation_distance: f32,
    model_scale: vec3<f32>,
    screen_width: f32
) -> vec3<f32> {
    let transmission_distance = thickness * length(model_scale);
    var transmitted_light: vec3<f32>;
    if dispersion > 0.0 {
        let half_spread = (ior - 1.0) * 0.025 * dispersion;
        let ior_r = ior - half_spread;
        let ior_b = ior + half_spread;
        let dir_r = refract_safe(view, normal, 1.0 / ior_r);
        let dir_g = refract_safe(view, normal, 1.0 / ior);
        let dir_b = refract_safe(view, normal, 1.0 / ior_b);
        let r = getScreenSpaceTransmission(world_pos, dir_r, transmission_distance, ior_r, roughness, screen_width).r;
        let g = getScreenSpaceTransmission(world_pos, dir_g, transmission_distance, ior, roughness, screen_width).g;
        let b = getScreenSpaceTransmission(world_pos, dir_b, transmission_distance, ior_b, roughness, screen_width).b;
        transmitted_light = vec3<f32>(r, g, b);
    } else {
        let dir = refract_safe(view, normal, 1.0 / ior);
        transmitted_light = getScreenSpaceTransmission(world_pos, dir, transmission_distance, ior, roughness, screen_width);
    }
    let attenuated_color = applyVolumeAttenuation(transmitted_light, transmission_distance, attenuation_color, attenuation_distance);
    let NdotV = clamp(dot(normal, view), 0.001, 1.0);
    let brdf = textureSampleLevel(brdf_lut, brdf_lut_sampler, vec2<f32>(NdotV, roughness), 0.0).rg;
    let specular_color = F0 * brdf.x + brdf.y;
    return (1.0 - specular_color) * attenuated_color * base_color;
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) tex_coords_1: vec2<f32>,
    @location(4) tangent: vec4<f32>,
    @location(5) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) world_pos: vec3<f32>,
    @location(2) @interpolate(flat) material_id: u32,
    @location(3) tex_coords: vec2<f32>,
    @location(4) tex_coords_1: vec2<f32>,
    @location(5) shadow_pos: vec4<f32>,
    @location(6) instance_tint: vec4<f32>,
    @location(7) vertex_color: vec4<f32>,
    @location(8) @interpolate(linear) tex_coords_affine: vec2<f32>,
    @location(9) @interpolate(flat) world_scale_factor: f32,
    @location(10) prev_world_pos: vec3<f32>,
    @location(11) world_tangent: vec4<f32>,
    @location(12) @interpolate(flat) entity_id: u32,
    @location(13) view_normal: vec3<f32>,
    @location(14) @interpolate(flat) flip_winding: u32,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) entity_id: f32,
    @location(2) view_normal: vec4<f32>,
    @location(3) velocity: vec2<f32>,
};

struct Uniforms {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    camera_position: vec4<f32>,
    num_lights: vec4<u32>,
    ambient_light: vec4<f32>,
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
    texture_debug_stripes: u32,
    texture_debug_stripes_speed: f32,
    directional_light_direction: vec4<f32>,
    ibl_blend_factor: f32,
    oit_z_scale: f32,
    flat_color: vec4<f32>,
    prev_view_projection: mat4x4<f32>,
    cur_view_projection: mat4x4<f32>,
    material_filtering: u32,
};

struct ModelMatrix {
    model: mat4x4<f32>,
    normal_matrix: mat3x3<f32>,
};

#import nightshade_renderer::material_data::{Material, TextureTransform}

const NORMAL_MAP_FLIP_Y: u32 = 1u;
const NORMAL_MAP_TWO_COMPONENT: u32 = 2u;

const ALPHA_MODE_OPAQUE: u32 = 0u;
const ALPHA_MODE_MASK: u32 = 1u;
const ALPHA_MODE_BLEND: u32 = 2u;
const ALPHA_MODE_DITHER: u32 = 3u;

fn interleaved_gradient_noise(p: vec2<f32>) -> f32 {
    return fract(52.9829189 * fract(dot(p, vec2<f32>(0.06711056, 0.00583715))));
}

fn texture_uv(uv0: vec2<f32>, uv1: vec2<f32>, transform: TextureTransform) -> vec2<f32> {
    let uv_set = u32(transform.row_u.w);
    let uv = select(uv0, uv1, uv_set == 1u);
    let homogeneous = vec3<f32>(uv, 1.0);
    let u = dot(transform.row_u.xyz, homogeneous);
    let v = dot(transform.row_v.xyz, homogeneous);
    return vec2<f32>(u, v);
}

struct ObjectData {
    transform_index: u32,
    mesh_id: u32,
    material_id: u32,
    batch_id: u32,
    morph_weights: array<f32, 8>,
    morph_target_count: u32,
    morph_displacement_offset: u32,
    mesh_vertex_offset: u32,
    mesh_vertex_count: u32,
    entity_id: u32,
    is_overlay: u32,
    skip_occlusion: u32,
    flip_winding: u32,
    culling_mask: u32,
    _pad_culling_0: u32,
    _pad_culling_1: u32,
    _pad_culling_2: u32,
};

struct MorphDisplacement {
    position: vec3<f32>,
    _pad0: f32,
    normal: vec3<f32>,
    _pad1: f32,
    tangent: vec3<f32>,
    _pad2: f32,
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

const COOKIE_LAYER_NONE: u32 = 0xFFFFFFFFu;

struct SpotlightShadowData {
    view_projection: mat4x4<f32>,
    atlas_offset: vec2<f32>,
    atlas_scale: vec2<f32>,
    bias: f32,
    _padding0: f32,
    _padding1: f32,
    _padding2: f32,
};

struct PointLightShadowData {
    position: vec3<f32>,
    range: f32,
    bias: f32,
    shadow_index: i32,
    _padding0: f32,
    _padding1: f32,
};

const LIGHT_TYPE_DIRECTIONAL: u32 = 0u;
const LIGHT_TYPE_POINT: u32 = 1u;
const LIGHT_TYPE_SPOT: u32 = 2u;

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

fn mip_rainbow_color(uv: vec2<f32>, texture_size: f32) -> vec3<f32> {
    let dx = dpdx(uv) * texture_size;
    let dy = dpdy(uv) * texture_size;
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

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var<storage, read> transforms: array<ModelMatrix>;

@group(1) @binding(12)
var<storage, read> prev_transforms: array<ModelMatrix>;

@group(1) @binding(1)
var<storage, read> materials: array<Material>;

@group(1) @binding(2)
var<storage, read> objects: array<ObjectData>;

@group(1) @binding(3)
var<storage, read> lights: array<Light>;

@group(1) @binding(4)
var<storage, read> visible_indices: array<u32>;

@group(1) @binding(5)
var<storage, read> instance_custom_data: array<vec4<f32>>;

@group(1) @binding(6)
var<storage, read> morph_displacements: array<MorphDisplacement>;

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

const CLUSTER_GRID_X: u32 = 16u;
const CLUSTER_GRID_Y: u32 = 9u;
const CLUSTER_GRID_Z: u32 = 24u;
const MAX_LIGHTS_PER_CLUSTER: u32 = 256u;

@group(1) @binding(7)
var<storage, read> light_grid: array<LightGrid>;

@group(1) @binding(8)
var<storage, read> light_indices: array<u32>;

@group(1) @binding(9)
var<uniform> cluster_uniforms: ClusterUniforms;

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

#import nightshade_renderer::material_sampling::{sample_srgb_layer, sample_linear_layer, sample_srgb_index, NO_LAYER, UvGrad}
#import nightshade_renderer::area_lighting::{area_light_count, area_light_at, evaluate_one_area_light}

var<private> material_uv0_ddx: vec2<f32>;
var<private> material_uv0_ddy: vec2<f32>;
var<private> material_uv1_ddx: vec2<f32>;
var<private> material_uv1_ddy: vec2<f32>;

fn texture_uv_grad(transform: TextureTransform) -> UvGrad {
    let uv_set = u32(transform.row_u.w);
    let ddx = select(material_uv0_ddx, material_uv1_ddx, uv_set == 1u);
    let ddy = select(material_uv0_ddy, material_uv1_ddy, uv_set == 1u);
    var out: UvGrad;
    out.ddx = vec2<f32>(dot(transform.row_u.xy, ddx), dot(transform.row_v.xy, ddx));
    out.ddy = vec2<f32>(dot(transform.row_u.xy, ddy), dot(transform.row_v.xy, ddy));
    return out;
}

fn base_material_uv_grad() -> UvGrad {
    var out: UvGrad;
    out.ddx = material_uv0_ddx;
    out.ddy = material_uv0_ddy;
    return out;
}

fn scaled_material_uv_grad(scale: f32) -> UvGrad {
    var out: UvGrad;
    out.ddx = material_uv0_ddx * scale;
    out.ddy = material_uv0_ddy * scale;
    return out;
}

@group(3) @binding(0)
var shadow_texture: texture_depth_2d;

@group(3) @binding(1)
var shadow_sampler: sampler;

@group(3) @binding(2)
var spotlight_shadow_atlas: texture_depth_2d;

@group(3) @binding(3)
var spotlight_shadow_sampler: sampler;

@group(3) @binding(4)
var<storage, read> spotlight_shadows: array<SpotlightShadowData>;

@group(3) @binding(5)
var brdf_lut: texture_2d<f32>;

@group(3) @binding(6)
var brdf_lut_sampler: sampler;

@group(3) @binding(7)
var irradiance_map: texture_cube<f32>;

@group(3) @binding(8)
var irradiance_sampler: sampler;

@group(3) @binding(9)
var prefiltered_env: texture_cube<f32>;

@group(3) @binding(10)
var prefiltered_sampler: sampler;

@group(3) @binding(11)
var point_shadow_cubemap: texture_cube_array<f32>;

@group(3) @binding(12)
var point_shadow_sampler: sampler;

@group(3) @binding(18)
var shadow_compare_sampler: sampler_comparison;

@group(3) @binding(13)
var<storage, read> point_light_shadows: array<PointLightShadowData>;

@group(3) @binding(14)
var irradiance_map_b: texture_cube<f32>;

@group(3) @binding(15)
var prefiltered_env_b: texture_cube<f32>;

@group(3) @binding(16)
var transmission_color_texture: texture_2d<f32>;

@group(3) @binding(17)
var transmission_color_sampler: sampler;

@vertex
fn vs_main(
    in: VertexInput,
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_id: u32
) -> VertexOutput {
    var out: VertexOutput;

    let object_index = visible_indices[instance_id];
    let object = objects[object_index];

    var position = in.position;
    var normal = in.normal;
    var tangent = in.tangent.xyz;

    if object.morph_target_count > 0u {
        let local_vertex_index = vertex_index - object.mesh_vertex_offset;

        for (var target_index = 0u; target_index < object.morph_target_count; target_index = target_index + 1u) {
            let weight = object.morph_weights[target_index];
            if abs(weight) > 0.0001 {
                let displacement_index = object.morph_displacement_offset
                    + target_index * object.mesh_vertex_count
                    + local_vertex_index;
                let displacement = morph_displacements[displacement_index];
                position = position + displacement.position * weight;
                normal = normal + displacement.normal * weight;
                tangent = tangent + displacement.tangent * weight;
            }
        }
        normal = normalize(normal);
        tangent = normalize(tangent);
    }

    let transform = transforms[object.transform_index];
    let model_matrix = transform.model;
    let normal_matrix = transform.normal_matrix;

    let world_position = model_matrix * vec4<f32>(position, 1.0);
    out.world_pos = world_position.xyz;

    let mat = materials[object.material_id];

    let view_position = uniforms.view * world_position;
    var clip_position = uniforms.projection * view_position;

    if uniforms.snap_enabled == 1u {
        let resolution = uniforms.snap_resolution;
        let snapped_x = round(clip_position.x * resolution.x / clip_position.w) * clip_position.w / resolution.x;
        let snapped_y = round(clip_position.y * resolution.y / clip_position.w) * clip_position.w / resolution.y;
        clip_position.x = snapped_x;
        clip_position.y = snapped_y;
    }

    out.position = clip_position;
    let world_normal = normalize(normal_matrix * normal);
    out.normal = world_normal;
    let model_mat3 = mat3x3<f32>(
        model_matrix[0].xyz,
        model_matrix[1].xyz,
        model_matrix[2].xyz
    );
    out.world_tangent = vec4<f32>(normalize(model_mat3 * tangent), in.tangent.w);
    out.material_id = object.material_id;
    out.tex_coords = in.tex_coords;
    out.tex_coords_1 = in.tex_coords_1;
    out.tex_coords_affine = texture_uv(in.tex_coords, in.tex_coords_1, mat.base_transform);
    out.world_scale_factor = (length(model_mat3[0]) + length(model_mat3[1]) + length(model_mat3[2])) / 3.0;
    out.prev_world_pos = (prev_transforms[object.transform_index].model * vec4<f32>(position, 1.0)).xyz;
    out.instance_tint = instance_custom_data[object.transform_index];
    out.vertex_color = in.color;

    let shadow_offset_position = world_position.xyz + world_normal * uniforms.shadow_normal_bias;
    let light_space_pos = uniforms.light_view_projection * vec4<f32>(shadow_offset_position, 1.0);
    out.shadow_pos = light_space_pos;

    out.entity_id = object.entity_id;
    out.flip_winding = object.flip_winding;

    let view_mat3 = mat3x3<f32>(
        uniforms.view[0].xyz,
        uniforms.view[1].xyz,
        uniforms.view[2].xyz
    );
    out.view_normal = normalize(view_mat3 * world_normal);

    return out;
}

const POISSON_16: array<vec2<f32>, 16> = array<vec2<f32>, 16>(
    vec2<f32>(-0.94201624, -0.39906216),
    vec2<f32>(0.94558609, -0.76890725),
    vec2<f32>(-0.094184101, -0.92938870),
    vec2<f32>(0.34495938, 0.29387760),
    vec2<f32>(-0.91588581, 0.45771432),
    vec2<f32>(-0.81544232, -0.87912464),
    vec2<f32>(-0.38277543, 0.27676845),
    vec2<f32>(0.97484398, 0.75648379),
    vec2<f32>(0.44323325, -0.97511554),
    vec2<f32>(0.53742981, -0.47373420),
    vec2<f32>(-0.26496911, -0.41893023),
    vec2<f32>(0.79197514, 0.19090188),
    vec2<f32>(-0.24188840, 0.99706507),
    vec2<f32>(-0.81409955, 0.91437590),
    vec2<f32>(0.19984126, 0.78641367),
    vec2<f32>(0.14383161, -0.14100790)
);

const POISSON_8: array<vec2<f32>, 8> = array<vec2<f32>, 8>(
    vec2<f32>(-0.7071, -0.7071),
    vec2<f32>(0.7071, -0.7071),
    vec2<f32>(-0.7071, 0.7071),
    vec2<f32>(0.7071, 0.7071),
    vec2<f32>(-1.0, 0.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(0.0, -1.0),
    vec2<f32>(0.0, 1.0)
);

const POISSON_4: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>(-0.5, -0.5),
    vec2<f32>(0.5, -0.5),
    vec2<f32>(-0.5, 0.5),
    vec2<f32>(0.5, 0.5)
);

fn get_shadow_lod(view_depth: f32) -> u32 {
    if view_depth < 30.0 {
        return 0u;
    } else if view_depth < 80.0 {
        return 1u;
    }
    return 2u;
}

struct BlockerSearchResult {
    average_depth: f32,
    num_blockers: u32,
};

fn rotate_offset(offset: vec2<f32>, rotation: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        rotation.x * offset.x - rotation.y * offset.y,
        rotation.y * offset.x + rotation.x * offset.y,
    );
}

fn search_blockers_directional(shadow_uv: vec2<f32>, receiver_depth: f32, texel_size: f32, search_radius: f32, safe_min: vec2<f32>, safe_max: vec2<f32>, rotation: vec2<f32>) -> BlockerSearchResult {
    var blocker_sum = 0.0;
    var num_blockers = 0u;

    for (var sample_index = 0; sample_index < 8; sample_index = sample_index + 1) {
        let offset = rotate_offset(POISSON_8[sample_index], rotation) * texel_size * search_radius;
        let sample_uv = clamp(shadow_uv + offset, safe_min, safe_max);
        let sampled_depth = textureSampleLevel(shadow_texture, shadow_sampler, sample_uv, 0);

        if sampled_depth > receiver_depth {
            blocker_sum = blocker_sum + sampled_depth;
            num_blockers = num_blockers + 1u;
        }
    }

    var result: BlockerSearchResult;
    if num_blockers > 0u {
        result.average_depth = blocker_sum / f32(num_blockers);
    } else {
        result.average_depth = 1.0;
    }
    result.num_blockers = num_blockers;
    return result;
}

fn search_blockers_spotlight(shadow_uv: vec2<f32>, receiver_depth: f32, texel_size: f32, search_radius: f32, safe_min: vec2<f32>, safe_max: vec2<f32>, rotation: vec2<f32>) -> BlockerSearchResult {
    var blocker_sum = 0.0;
    var num_blockers = 0u;

    for (var sample_index = 0; sample_index < 8; sample_index = sample_index + 1) {
        let offset = rotate_offset(POISSON_8[sample_index], rotation) * texel_size * search_radius;
        let sample_uv = clamp(shadow_uv + offset, safe_min, safe_max);
        let sampled_depth = textureSampleLevel(spotlight_shadow_atlas, spotlight_shadow_sampler, sample_uv, 0);

        if sampled_depth > receiver_depth {
            blocker_sum = blocker_sum + sampled_depth;
            num_blockers = num_blockers + 1u;
        }
    }

    var result: BlockerSearchResult;
    if num_blockers > 0u {
        result.average_depth = blocker_sum / f32(num_blockers);
    } else {
        result.average_depth = 1.0;
    }
    result.num_blockers = num_blockers;
    return result;
}

fn select_cascade(view_depth: f32) -> i32 {
    for (var cascade_index = 0; cascade_index < 4; cascade_index = cascade_index + 1) {
        if view_depth < uniforms.cascade_split_distances[cascade_index] {
            return cascade_index;
        }
    }
    return 3;
}

fn sample_shadow_cascade(world_pos: vec3<f32>, world_normal: vec3<f32>, cascade_index: i32, shadow_lod: u32, rotation: vec2<f32>) -> f32 {
    let direction_to_light = -normalize(uniforms.directional_light_direction.xyz);
    let texel_size_world = uniforms.cascade_atlas_offsets[cascade_index].z;
    let slope_scale = clamp(1.0 - dot(world_normal, direction_to_light), 0.0, 1.0);
    let normal_offset = uniforms.shadow_normal_bias * texel_size_world * world_normal * slope_scale;
    let depth_offset = uniforms.shadow_bias * direction_to_light;
    let shadow_offset_position = world_pos + normal_offset + depth_offset;
    let shadow_pos = uniforms.cascade_view_projections[cascade_index] * vec4<f32>(shadow_offset_position, 1.0);

    let shadow_coords_ndc = shadow_pos.xyz / shadow_pos.w;
    var shadow_uv = shadow_coords_ndc.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);

    let atlas_offset = uniforms.cascade_atlas_offsets[cascade_index].xy;
    let atlas_scale = uniforms.cascade_atlas_scale.xy;
    shadow_uv = shadow_uv * atlas_scale + atlas_offset;

    let shadow_depth = shadow_coords_ndc.z;

    let slot_min = atlas_offset;
    let slot_max = atlas_offset + atlas_scale;
    let in_bounds = f32(
        shadow_uv.x >= slot_min.x && shadow_uv.x <= slot_max.x &&
        shadow_uv.y >= slot_min.y && shadow_uv.y <= slot_max.y &&
        shadow_depth >= 0.0 && shadow_depth <= 1.0
    );

    let size = f32(textureDimensions(shadow_texture).x);
    let texel_size = 1.0 / size;

    let slot_margin = atlas_scale * 0.01;
    let safe_min = slot_min + slot_margin;
    let safe_max = slot_max - slot_margin;

    let light_size = uniforms.directional_light_size;
    let search_radius = light_size * 10.0;
    let blocker_result = search_blockers_directional(shadow_uv, shadow_depth, texel_size, search_radius, safe_min, safe_max, rotation);

    if blocker_result.num_blockers == 0u {
        return 1.0;
    }

    let penumbra_ratio = (blocker_result.average_depth - shadow_depth) / max(1.0 - blocker_result.average_depth, 0.001);
    let filter_radius = clamp(penumbra_ratio * light_size * 20.0, 1.0, 8.0);

    var visibility = 0.0;

    if shadow_lod == 0u {
        for (var sample_index = 0; sample_index < 16; sample_index = sample_index + 1) {
            let offset = rotate_offset(POISSON_16[sample_index], rotation) * texel_size * filter_radius;
            let sample_uv = clamp(shadow_uv + offset, safe_min, safe_max);
            visibility = visibility + textureSampleCompareLevel(shadow_texture, shadow_compare_sampler, sample_uv, shadow_depth);
        }
        return mix(1.0, visibility / 16.0, in_bounds);
    } else if shadow_lod == 1u {
        for (var sample_index = 0; sample_index < 8; sample_index = sample_index + 1) {
            let offset = rotate_offset(POISSON_8[sample_index], rotation) * texel_size * filter_radius;
            let sample_uv = clamp(shadow_uv + offset, safe_min, safe_max);
            visibility = visibility + textureSampleCompareLevel(shadow_texture, shadow_compare_sampler, sample_uv, shadow_depth);
        }
        return mix(1.0, visibility / 8.0, in_bounds);
    } else {
        for (var sample_index = 0; sample_index < 4; sample_index = sample_index + 1) {
            let offset = rotate_offset(POISSON_4[sample_index], rotation) * texel_size * filter_radius;
            let sample_uv = clamp(shadow_uv + offset, safe_min, safe_max);
            visibility = visibility + textureSampleCompareLevel(shadow_texture, shadow_compare_sampler, sample_uv, shadow_depth);
        }
        return mix(1.0, visibility / 4.0, in_bounds);
    }
}

fn calculate_shadow_factor(world_pos: vec3<f32>, world_normal: vec3<f32>, view_depth: f32, rotation: vec2<f32>) -> f32 {
    if uniforms.shadows_enabled < 0.5 {
        return 1.0;
    }

    let shadow_far = uniforms.cascade_split_distances[3];
    if view_depth > shadow_far {
        return 1.0;
    }
    let far_fade = smoothstep(shadow_far, shadow_far * 0.9, view_depth);

    let shadow_lod = get_shadow_lod(view_depth);
    let cascade_index = select_cascade(view_depth);
    let shadow_factor = sample_shadow_cascade(world_pos, world_normal, cascade_index, shadow_lod, rotation);

    if cascade_index < 3 {
        let cascade_end = uniforms.cascade_split_distances[cascade_index];
        let blend_range = cascade_end * 0.25;
        let blend_start = cascade_end - blend_range;

        if view_depth > blend_start {
            let blend_factor = (view_depth - blend_start) / blend_range;
            let next_shadow_factor = sample_shadow_cascade(world_pos, world_normal, cascade_index + 1, shadow_lod, rotation);
            return mix(shadow_factor, next_shadow_factor, blend_factor);
        }
    }

    return mix(1.0, shadow_factor, far_fade);
}

fn calculate_spotlight_shadow_factor(world_pos: vec3<f32>, normal: vec3<f32>, light_pos: vec3<f32>, shadow_index: i32, shadow_lod: u32, light_size: f32, rotation: vec2<f32>) -> f32 {
    if shadow_index < 0 {
        return 1.0;
    }

    let shadow_data = spotlight_shadows[shadow_index];
    let light_space_pos = shadow_data.view_projection * vec4<f32>(world_pos, 1.0);

    if light_space_pos.w <= 0.0 {
        return 1.0;
    }

    let shadow_coords_ndc = light_space_pos.xyz / light_space_pos.w;

    var shadow_uv = shadow_coords_ndc.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    shadow_uv = shadow_uv * shadow_data.atlas_scale + shadow_data.atlas_offset;

    let shadow_depth = shadow_coords_ndc.z;

    let slot_min = shadow_data.atlas_offset;
    let slot_max = shadow_data.atlas_offset + shadow_data.atlas_scale;
    let in_bounds = f32(
        shadow_uv.x >= slot_min.x && shadow_uv.x <= slot_max.x &&
        shadow_uv.y >= slot_min.y && shadow_uv.y <= slot_max.y &&
        shadow_depth >= 0.0 && shadow_depth <= 1.0
    );

    let atlas_size = f32(textureDimensions(spotlight_shadow_atlas).x);
    let texel_size = 1.0 / atlas_size;

    let slot_margin = shadow_data.atlas_scale * 0.01;
    let safe_min = slot_min + slot_margin;
    let safe_max = slot_max - slot_margin;

    let light_dir = normalize(light_pos - world_pos);
    let slope_factor = 1.0 - abs(dot(normal, light_dir));
    let adjusted_bias = shadow_data.bias * (1.0 + slope_factor * 2.0);

    let reference_depth = shadow_depth + adjusted_bias;

    let search_radius = light_size * 15.0;
    let blocker_result = search_blockers_spotlight(shadow_uv, reference_depth, texel_size, search_radius, safe_min, safe_max, rotation);

    if blocker_result.num_blockers == 0u {
        return 1.0;
    }

    let penumbra_ratio = (blocker_result.average_depth - reference_depth) / max(1.0 - blocker_result.average_depth, 0.001);
    let filter_radius = clamp(penumbra_ratio * light_size * 30.0, 0.5, 6.0);

    var visibility = 0.0;

    if shadow_lod == 0u {
        for (var sample_index = 0; sample_index < 16; sample_index = sample_index + 1) {
            let offset = rotate_offset(POISSON_16[sample_index], rotation) * texel_size * filter_radius;
            let sample_uv = clamp(shadow_uv + offset, safe_min, safe_max);
            visibility = visibility + textureSampleCompareLevel(spotlight_shadow_atlas, shadow_compare_sampler, sample_uv, reference_depth);
        }
        return mix(1.0, visibility / 16.0, in_bounds);
    } else if shadow_lod == 1u {
        for (var sample_index = 0; sample_index < 8; sample_index = sample_index + 1) {
            let offset = rotate_offset(POISSON_8[sample_index], rotation) * texel_size * filter_radius;
            let sample_uv = clamp(shadow_uv + offset, safe_min, safe_max);
            visibility = visibility + textureSampleCompareLevel(spotlight_shadow_atlas, shadow_compare_sampler, sample_uv, reference_depth);
        }
        return mix(1.0, visibility / 8.0, in_bounds);
    } else {
        for (var sample_index = 0; sample_index < 4; sample_index = sample_index + 1) {
            let offset = rotate_offset(POISSON_4[sample_index], rotation) * texel_size * filter_radius;
            let sample_uv = clamp(shadow_uv + offset, safe_min, safe_max);
            visibility = visibility + textureSampleCompareLevel(spotlight_shadow_atlas, shadow_compare_sampler, sample_uv, reference_depth);
        }
        return mix(1.0, visibility / 4.0, in_bounds);
    }
}

fn build_tangent_frame(direction: vec3<f32>) -> mat3x3<f32> {
    let up = select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(direction.y) > 0.99);
    let tangent = normalize(cross(up, direction));
    let bitangent = cross(direction, tangent);
    return mat3x3<f32>(tangent, bitangent, direction);
}

const POINT_LIGHT_PCF_OFFSETS: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(1.0, 0.0),
    vec2<f32>(-1.0, 0.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(0.0, -1.0),
    vec2<f32>(0.707, 0.707),
    vec2<f32>(-0.707, -0.707)
);

const PCF_SAMPLES_20: array<vec3<f32>, 20> = array<vec3<f32>, 20>(
    vec3<f32>(1.0, 0.0, 0.0),
    vec3<f32>(-1.0, 0.0, 0.0),
    vec3<f32>(0.0, 1.0, 0.0),
    vec3<f32>(0.0, -1.0, 0.0),
    vec3<f32>(0.0, 0.0, 1.0),
    vec3<f32>(0.0, 0.0, -1.0),
    vec3<f32>(0.707, 0.707, 0.0),
    vec3<f32>(-0.707, 0.707, 0.0),
    vec3<f32>(0.707, -0.707, 0.0),
    vec3<f32>(-0.707, -0.707, 0.0),
    vec3<f32>(0.707, 0.0, 0.707),
    vec3<f32>(-0.707, 0.0, 0.707),
    vec3<f32>(0.707, 0.0, -0.707),
    vec3<f32>(-0.707, 0.0, -0.707),
    vec3<f32>(0.0, 0.707, 0.707),
    vec3<f32>(0.0, -0.707, 0.707),
    vec3<f32>(0.0, 0.707, -0.707),
    vec3<f32>(0.0, -0.707, -0.707),
    vec3<f32>(0.577, 0.577, 0.577),
    vec3<f32>(-0.577, -0.577, -0.577)
);

fn search_blockers_point_light(direction: vec3<f32>, receiver_distance: f32, shadow_index: i32, tangent_frame: mat3x3<f32>, search_radius: f32) -> BlockerSearchResult {
    var blocker_sum = 0.0;
    var num_blockers = 0u;

    for (var sample_index = 0; sample_index < 6; sample_index = sample_index + 1) {
        let offset_2d = POINT_LIGHT_PCF_OFFSETS[sample_index] * search_radius;
        let offset_3d = tangent_frame[0] * offset_2d.x + tangent_frame[1] * offset_2d.y;
        let sample_dir = normalize(direction + offset_3d);

        let sampled_depth = textureSampleLevel(point_shadow_cubemap, point_shadow_sampler, sample_dir, shadow_index, 0.0).r;

        if sampled_depth < receiver_distance {
            blocker_sum = blocker_sum + sampled_depth;
            num_blockers = num_blockers + 1u;
        }
    }

    var result: BlockerSearchResult;
    if num_blockers > 0u {
        result.average_depth = blocker_sum / f32(num_blockers);
    } else {
        result.average_depth = 0.0;
    }
    result.num_blockers = num_blockers;
    return result;
}

fn calculate_point_light_shadow_factor(world_pos: vec3<f32>, world_normal: vec3<f32>, shadow_index: i32, light_size: f32) -> f32 {
    if shadow_index < 0 {
        return 1.0;
    }

    let shadow_data = point_light_shadows[shadow_index];
    let light_pos = shadow_data.position;
    let light_range = shadow_data.range;
    let bias = shadow_data.bias;

    let light_to_frag = world_pos - light_pos;
    let distance = length(light_to_frag);

    if distance > light_range {
        return 1.0;
    }

    let direction = normalize(light_to_frag);
    let normalized_distance = distance / light_range;

    let light_dir = -direction;
    let slope_factor = 1.0 - abs(dot(world_normal, light_dir));
    let adjusted_bias = bias * (1.0 + slope_factor * 2.0);
    let distance_with_bias = normalized_distance - adjusted_bias;

    let disk_radius = light_size * 0.02;

    var shadow = 0.0;
    for (var i = 0; i < 20; i = i + 1) {
        let offset = PCF_SAMPLES_20[i] * disk_radius;
        let sample_dir = normalize(direction + offset);
        let sampled_depth = textureSampleLevel(point_shadow_cubemap, point_shadow_sampler, sample_dir, shadow_index, 0.0).r;
        shadow = shadow + select(0.0, 1.0, distance_with_bias <= sampled_depth);
    }

    return shadow / 20.0;
}

fn sample_spot_cookie(light: Light, world_pos: vec3<f32>) -> vec3<f32> {
    if light.cookie_layer == COOKIE_LAYER_NONE || light.shadow_index < 0 {
        return vec3<f32>(1.0);
    }
    let shadow_data = spotlight_shadows[light.shadow_index];
    let clip = shadow_data.view_projection * vec4<f32>(world_pos, 1.0);
    if clip.w <= 0.0 {
        return vec3<f32>(0.0);
    }
    let ndc = clip.xyz / clip.w;
    let uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
    if any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) {
        return vec3<f32>(0.0);
    }
    let layer = light.cookie_layer & 0xFFFFu;
    return sample_srgb_index(layer, uv).rgb;
}

fn sample_point_cookie(light: Light, world_pos: vec3<f32>) -> vec3<f32> {
    if light.cookie_layer == COOKIE_LAYER_NONE {
        return vec3<f32>(1.0);
    }
    let dir = normalize(world_pos - light.position.xyz);
    let uv = vec2<f32>(
        atan2(dir.z, dir.x) / (2.0 * PI) + 0.5,
        acos(clamp(dir.y, -1.0, 1.0)) / PI,
    );
    let layer = light.cookie_layer & 0xFFFFu;
    return sample_srgb_index(layer, uv).rgb;
}

fn getLightIntensity(light: Light, point_to_light: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    var range_attenuation = 1.0;
    var spot_attenuation = 1.0;
    var cookie_tint = vec3<f32>(1.0);

    if light.light_type != LIGHT_TYPE_DIRECTIONAL {
        range_attenuation = getRangeAttenuation(light.range, length(point_to_light));
    }
    if light.light_type == LIGHT_TYPE_SPOT {
        spot_attenuation = getSpotAttenuation(point_to_light, light.direction.xyz, light.outer_cone, light.inner_cone);
        cookie_tint = sample_spot_cookie(light, world_pos);
    } else if light.light_type == LIGHT_TYPE_POINT {
        cookie_tint = sample_point_cookie(light, world_pos);
    }

    return range_attenuation * spot_attenuation * light.color.rgb * cookie_tint;
}

fn safe_normalize(v: vec3<f32>) -> vec3<f32> {
    let len_sq = dot(v, v);
    if len_sq < 0.0001 {
        return vec3<f32>(0.0, 1.0, 0.0);
    }
    return v / sqrt(len_sq);
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


@fragment
fn fs_main(in: VertexOutput, @builtin(front_facing) is_front_face: bool) -> FragmentOutput {
    let material_filter_scale = select(0.0, 1.0, uniforms.material_filtering != 0u);
    material_uv0_ddx = dpdx(in.tex_coords) * material_filter_scale;
    material_uv0_ddy = dpdy(in.tex_coords) * material_filter_scale;
    material_uv1_ddx = dpdx(in.tex_coords_1) * material_filter_scale;
    material_uv1_ddy = dpdy(in.tex_coords_1) * material_filter_scale;
    if in.flip_winding != 0u && is_front_face {
        discard;
    }
    if uniforms.flat_color.a >= 2.0 {
        discard;
    }
    var output: FragmentOutput;
    let computed_view_depth = -(uniforms.view * vec4<f32>(in.world_pos, 1.0)).z;
    let velocity_cur_clip = uniforms.cur_view_projection * vec4<f32>(in.world_pos, 1.0);
    let velocity_prev_clip = uniforms.prev_view_projection * vec4<f32>(in.prev_world_pos, 1.0);
    let velocity_cur_ndc = velocity_cur_clip.xy / velocity_cur_clip.w;
    let velocity_prev_ndc = velocity_prev_clip.xy / velocity_prev_clip.w;
    let velocity_cur_uv = vec2<f32>(velocity_cur_ndc.x * 0.5 + 0.5, 0.5 - velocity_cur_ndc.y * 0.5);
    let velocity_prev_uv = vec2<f32>(velocity_prev_ndc.x * 0.5 + 0.5, 0.5 - velocity_prev_ndc.y * 0.5);
    output.velocity = velocity_prev_uv - velocity_cur_uv;
    let material = materials[in.material_id];

    let view_dir = normalize(uniforms.camera_position.xyz - in.world_pos);
    var geom_normal = in.normal;
    var geom_tangent = in.world_tangent;
    if !is_front_face {
        geom_normal = -geom_normal;
        geom_tangent = -geom_tangent;
    }

    var uv0 = in.tex_coords;
    let uv1 = in.tex_coords_1;

    if material.height_layer != NO_LAYER && material.parallax_scale > 0.0 {
        let parallax_tangent = normalize(geom_tangent.xyz);
        let parallax_bitangent = cross(geom_normal, parallax_tangent) * geom_tangent.w;
        var view_tangent = vec3<f32>(
            dot(view_dir, parallax_tangent),
            dot(view_dir, parallax_bitangent),
            dot(view_dir, geom_normal),
        );
        view_tangent.z = view_tangent.z + 0.42;
        let raw_height = sample_linear_layer(material.height_layer, uv0, base_material_uv_grad()).r;
        let parallax_height = raw_height * material.parallax_scale - material.parallax_scale * 0.5;
        uv0 = uv0 + parallax_height * view_tangent.xy / view_tangent.z;
    }

    let base_uv = texture_uv(uv0, uv1, material.base_transform);
    let emissive_uv = texture_uv(uv0, uv1, material.emissive_transform);
    let normal_uv = texture_uv(uv0, uv1, material.normal_transform);

    var normal_tex_sample = select(
        vec3<f32>(0.5, 0.5, 1.0),
        sample_linear_layer(material.normal_layer, normal_uv, texture_uv_grad(material.normal_transform)).xyz,
        material.normal_layer != NO_LAYER,
    );
    if material.detail_normal_layer != NO_LAYER {
        let detail_uv = uv0 * material.detail_uv_scale;
        let detail_raw = sample_linear_layer(material.detail_normal_layer, detail_uv, scaled_material_uv_grad(material.detail_uv_scale)).xyz;
        let n1 = (normal_tex_sample * 2.0 - 1.0) + vec3<f32>(0.0, 0.0, 1.0);
        let n2 = (detail_raw * 2.0 - 1.0) * vec3<f32>(-1.0, -1.0, 1.0);
        normal_tex_sample = normalize(n1 * dot(n1, n2) / n1.z - n2) * 0.5 + 0.5;
    }
    let base_tex_sample = select(
        vec4<f32>(1.0),
        sample_srgb_layer(material.base_layer, select(base_uv, in.tex_coords_affine, uniforms.affine_enabled > 0u), texture_uv_grad(material.base_transform)),
        material.base_layer != NO_LAYER,
    );
    let emissive_tex_sample = select(
        vec4<f32>(0.0),
        sample_srgb_layer(material.emissive_layer, emissive_uv, texture_uv_grad(material.emissive_transform)),
        material.emissive_layer != NO_LAYER,
    );

    let has_normal_map = material.normal_layer != NO_LAYER || material.detail_normal_layer != NO_LAYER;
    var normal = get_normal(
        geom_normal,
        geom_tangent,
        normal_tex_sample,
        has_normal_map,
        material.normal_scale,
        material.normal_map_flags,
    );
    if uniforms.flat_color.a > 0.0 {
        normal = geom_normal;
    }

    var base_color = material.base_color * in.instance_tint * in.vertex_color;
    if material.base_layer != NO_LAYER {
        base_color = base_color * base_tex_sample;
    }
    if uniforms.flat_color.a > 0.0 {
        base_color = vec4<f32>(uniforms.flat_color.rgb, base_color.a);
    }

    var albedo = base_color.rgb;
    if material.detail_base_layer != NO_LAYER {
        let detail_uv = uv0 * material.detail_uv_scale;
        let detail_col = sample_srgb_layer(material.detail_base_layer, detail_uv, scaled_material_uv_grad(material.detail_uv_scale)).rgb;
        let screen_blend = 1.0 - (1.0 - albedo) * (1.0 - detail_col);
        let multiply_blend = 2.0 * albedo * detail_col;
        albedo = select(screen_blend, multiply_blend, albedo < vec3<f32>(0.5));
    }

    var metallic = material.metallic;
    var roughness = material.roughness;
    if material.metallic_roughness_layer != NO_LAYER {
        let mr_uv = texture_uv(uv0, uv1, material.metallic_roughness_transform);
        let mr_sample = sample_linear_layer(material.metallic_roughness_layer, mr_uv, texture_uv_grad(material.metallic_roughness_transform));
        roughness = roughness * mr_sample.g;
        metallic = metallic * mr_sample.b;
    }
    if uniforms.flat_color.a > 0.0 {
        metallic = 0.0;
        roughness = 1.0;
    }

    roughness = specularAntiAliasing(roughness, normal);

    var occlusion = 1.0;
    if material.occlusion_layer != NO_LAYER {
        let occlusion_uv = texture_uv(uv0, uv1, material.occlusion_transform);
        occlusion = sample_linear_layer(material.occlusion_layer, occlusion_uv, texture_uv_grad(material.occlusion_transform)).r;
    }

    var transmission_factor = material.transmission_factor;
    if material.transmission_layer != NO_LAYER {
        let transmission_uv = texture_uv(uv0, uv1, material.transmission_transform);
        let transmission_sample = sample_linear_layer(material.transmission_layer, transmission_uv, texture_uv_grad(material.transmission_transform)).r;
        transmission_factor = transmission_factor * transmission_sample;
    }

    var thickness = material.thickness;
    if material.thickness_layer != NO_LAYER {
        let thickness_uv = texture_uv(uv0, uv1, material.thickness_transform);
        let thickness_sample = sample_linear_layer(material.thickness_layer, thickness_uv, texture_uv_grad(material.thickness_transform)).g;
        thickness = thickness * thickness_sample;
    }

    var specular_factor = material.specular_factor;
    if material.specular_layer != NO_LAYER {
        let specular_uv = texture_uv(uv0, uv1, material.specular_transform);
        let specular_sample = sample_linear_layer(material.specular_layer, specular_uv, texture_uv_grad(material.specular_transform)).a;
        specular_factor = specular_factor * specular_sample;
    }

    var specular_color_factor = material.specular_color_factor;
    if material.specular_color_layer != NO_LAYER {
        let specular_color_uv = texture_uv(uv0, uv1, material.specular_color_transform);
        let specular_color_sample = sample_srgb_layer(material.specular_color_layer, specular_color_uv, texture_uv_grad(material.specular_color_transform)).rgb;
        specular_color_factor = specular_color_factor * specular_color_sample;
    }

    var alpha = base_color.a;

    if material.alpha_mode == ALPHA_MODE_OPAQUE {
        alpha = 1.0;
    } else if material.alpha_mode == ALPHA_MODE_MASK {
        if alpha < material.alpha_cutoff {
            discard;
        }
        alpha = 1.0;
    } else if material.alpha_mode == ALPHA_MODE_DITHER {
        let dither_jitter = vec2<f32>(fract(uniforms.time * 91.7), fract(uniforms.time * 47.3)) * 64.0;
        let dither_offset = f32(in.entity_id & 0xFFu);
        let dither_noise = interleaved_gradient_noise(in.position.xy + dither_jitter + dither_offset);
        if alpha < pow(dither_noise, 2.2) {
            discard;
        }
        alpha = 1.0;
    }

    var emission = material.emissive_factor * material.emissive_strength;
    if material.emissive_layer != NO_LAYER {
        emission = emission * emissive_tex_sample.rgb;
    }
    if uniforms.flat_color.a > 0.0 {
        emission = vec3<f32>(0.0);
    }

    let shadow_rotation_angle = interleaved_gradient_noise(in.position.xy) * 6.2831853;
    let shadow_rotation = vec2<f32>(cos(shadow_rotation_angle), sin(shadow_rotation_angle));
    let shadow_factor = calculate_shadow_factor(in.world_pos, geom_normal, computed_view_depth, shadow_rotation);

    var final_color: vec3<f32>;
    var debug_F = vec3<f32>(0.0);
    var debug_G = 0.0;
    var debug_D = 0.0;
    var debug_specular = vec3<f32>(0.0);
    var debug_diffuse = vec3<f32>(0.0);

    if material.unlit != 0u || uniforms.global_unlit > 0.5 {
        final_color = albedo + emission;
    } else {
        let N = normal;
        let V = normalize(uniforms.camera_position.xyz - in.world_pos);
        let R = reflect(-V, N);
        let NdotV = max(dot(N, V), 0.0);

        let ior = material.ior;
        let dielectric_f0 = pow((ior - 1.0) / (ior + 1.0), 2.0);
        let dielectric_specular_f0 = min(vec3<f32>(dielectric_f0) * specular_color_factor * specular_factor, vec3<f32>(1.0));
        var F0 = mix(dielectric_specular_f0, albedo, metallic);

        var iridescence_factor = clamp(material.iridescence_factor, 0.0, 1.0);
        var iridescence_thickness = material.iridescence_thickness_maximum;
        if material.iridescence_layer != NO_LAYER {
            let irid_uv = texture_uv(uv0, uv1, material.iridescence_transform);
            let irid_sample = sample_linear_layer(material.iridescence_layer, irid_uv, texture_uv_grad(material.iridescence_transform));
            iridescence_factor = iridescence_factor * irid_sample.r;
            iridescence_thickness = mix(material.iridescence_thickness_minimum, material.iridescence_thickness_maximum, irid_sample.g);
        }
        if iridescence_factor > 0.0 && iridescence_thickness > 0.0 {
            let iridescence_fresnel = eval_iridescence(1.0, material.iridescence_ior, NdotV, iridescence_thickness, F0);
            let iridescent_f0 = schlick_to_f0(iridescence_fresnel, vec3<f32>(1.0), NdotV);
            F0 = mix(F0, iridescent_f0, iridescence_factor);
        }

        var anisotropy_strength = clamp(material.anisotropy_strength, 0.0, 1.0);
        var anisotropy_dir = vec2<f32>(material.anisotropy_rotation_cos, material.anisotropy_rotation_sin);
        if material.anisotropy_layer != NO_LAYER {
            let aniso_uv = texture_uv(uv0, uv1, material.anisotropy_transform);
            let aniso_sample = sample_linear_layer(material.anisotropy_layer, aniso_uv, texture_uv_grad(material.anisotropy_transform));
            let tex_dir = aniso_sample.xy * 2.0 - 1.0;
            let rot_x = material.anisotropy_rotation_cos * tex_dir.x - material.anisotropy_rotation_sin * tex_dir.y;
            let rot_y = material.anisotropy_rotation_sin * tex_dir.x + material.anisotropy_rotation_cos * tex_dir.y;
            anisotropy_dir = vec2<f32>(rot_x, rot_y);
            anisotropy_strength = anisotropy_strength * aniso_sample.b;
        }
        let aniso_T_world = safe_normalize(geom_tangent.xyz);
        let aniso_B_world = cross(N, aniso_T_world) * geom_tangent.w;
        let aniso_T = anisotropy_dir.x * aniso_T_world + anisotropy_dir.y * aniso_B_world;
        let aniso_B = cross(N, aniso_T);

        var diffuse_transmission_factor = clamp(material.diffuse_transmission_factor, 0.0, 1.0);
        var diffuse_transmission_color = material.diffuse_transmission_color_factor;
        if material.diffuse_transmission_color_layer != NO_LAYER {
            let dtc_uv = texture_uv(uv0, uv1, material.diffuse_transmission_color_transform);
            let dtc_sample = sample_srgb_layer(material.diffuse_transmission_color_layer, dtc_uv, texture_uv_grad(material.diffuse_transmission_color_transform));
            diffuse_transmission_color = diffuse_transmission_color * dtc_sample.rgb;
            diffuse_transmission_factor = diffuse_transmission_factor * dtc_sample.a;
        }
        let dt_active = diffuse_transmission_factor > 0.0;

        let brdf = textureSampleLevel(brdf_lut, brdf_lut_sampler, vec2<f32>(NdotV, roughness), 0.0).rg;
        let energy_compensation = 1.0 + F0 * (1.0 / max(brdf.x + brdf.y, 0.001) - 1.0);

        var cc_factor = clamp(material.clearcoat_factor, 0.0, 1.0);
        var cc_roughness = clamp(material.clearcoat_roughness_factor, 0.0, 1.0);
        if material.clearcoat_layer != NO_LAYER {
            let cc_uv = texture_uv(uv0, uv1, material.clearcoat_transform);
            let cc_sample = sample_linear_layer(material.clearcoat_layer, cc_uv, texture_uv_grad(material.clearcoat_transform));
            cc_factor = cc_factor * cc_sample.r;
        }
        if material.clearcoat_roughness_layer != NO_LAYER {
            let ccr_uv = texture_uv(uv0, uv1, material.clearcoat_roughness_transform);
            let ccr_sample = sample_linear_layer(material.clearcoat_roughness_layer, ccr_uv, texture_uv_grad(material.clearcoat_roughness_transform));
            cc_roughness = cc_roughness * ccr_sample.g;
        }
        cc_roughness = clamp(cc_roughness, 0.04, 1.0);
        var cc_normal = N;
        if material.clearcoat_normal_layer != NO_LAYER {
            let cc_n_uv = texture_uv(uv0, uv1, material.clearcoat_normal_transform);
            let cc_n_sample = sample_linear_layer(material.clearcoat_normal_layer, cc_n_uv, texture_uv_grad(material.clearcoat_normal_transform)).xyz;
            let cc_n_tangent_xy = (cc_n_sample.xy * 2.0 - 1.0) * material.clearcoat_normal_scale;
            let cc_n_tangent = vec3<f32>(cc_n_tangent_xy, sqrt(max(1.0 - dot(cc_n_tangent_xy, cc_n_tangent_xy), 0.0)));
            let cc_T = safe_normalize(geom_tangent.xyz);
            let cc_B = cross(geom_normal, cc_T) * geom_tangent.w;
            let cc_TBN = mat3x3<f32>(cc_T, cc_B, geom_normal);
            cc_normal = safe_normalize(cc_TBN * cc_n_tangent);
        }
        let cc_NdotV = max(dot(cc_normal, V), 0.0);
        let cc_R = reflect(-V, cc_normal);
        let cc_brdf = textureSampleLevel(brdf_lut, brdf_lut_sampler, vec2<f32>(cc_NdotV, cc_roughness), 0.0).rg;

        var sheen_color = material.sheen_color_factor;
        var sheen_roughness = clamp(material.sheen_roughness_factor, 0.0, 1.0);
        if material.sheen_color_layer != NO_LAYER {
            let sheen_uv = texture_uv(uv0, uv1, material.sheen_color_transform);
            let sheen_sample = sample_srgb_layer(material.sheen_color_layer, sheen_uv, texture_uv_grad(material.sheen_color_transform));
            sheen_color = sheen_color * sheen_sample.rgb;
            sheen_roughness = sheen_roughness * sheen_sample.a;
        }
        sheen_roughness = clamp(sheen_roughness, 0.07, 1.0);
        let sheen_max = max(sheen_color.r, max(sheen_color.g, sheen_color.b));
        let sheen_active = sheen_max > 0.0;
        let sheen_E = textureSampleLevel(brdf_lut, brdf_lut_sampler, vec2<f32>(NdotV, sheen_roughness), 0.0).b;
        let sheen_albedo_scale = clamp(1.0 - sheen_max * sheen_E, 0.0, 1.0);

        var Lo = vec3<f32>(0.0);
        var captured_debug = false;

        for (var i = 0u; i < cluster_uniforms.num_directional_lights; i = i + 1u) {
            let light = lights[i];

            let point_to_light = -light.direction.xyz;

            let L = normalize(point_to_light);
            let H = normalize(V + L);

            let radiance = getLightIntensity(light, point_to_light, in.world_pos);

            let F_light = fresnelSchlick(max(dot(H, V), 0.0), F0);
            var NDF = 0.0;
            var G = 0.0;
            var specular: vec3<f32>;
            if anisotropy_strength > 0.0 {
                let alpha = roughness * roughness;
                let at = mix(alpha, 1.0, anisotropy_strength * anisotropy_strength);
                let ab = alpha;
                let aniso_NdotL = max(dot(N, L), 0.0);
                let aniso_NdotH = max(dot(N, H), 0.0);
                let aniso_TdotH = dot(aniso_T, H);
                let aniso_BdotH = dot(aniso_B, H);
                let aniso_TdotV = dot(aniso_T, V);
                let aniso_BdotV = dot(aniso_B, V);
                let aniso_TdotL = dot(aniso_T, L);
                let aniso_BdotL = dot(aniso_B, L);
                NDF = d_ggx_anisotropic(aniso_NdotH, aniso_TdotH, aniso_BdotH, at, ab);
                let aniso_V = v_ggx_anisotropic(aniso_NdotL, NdotV, aniso_BdotV, aniso_TdotV, aniso_BdotL, aniso_TdotL, at, ab);
                G = aniso_V * 4.0 * NdotV * max(dot(N, L), 0.001);
                specular = F_light * NDF * aniso_V;
            } else {
                NDF = DistributionGGX(N, H, roughness);
                let vis = V_SmithGGXCorrelated(NdotV, max(dot(N, L), 0.0), roughness);
                G = vis * 4.0 * NdotV * max(dot(N, L), 0.001);
                specular = NDF * vis * F_light;
            }

            let kS = F_light;
            var kD = vec3<f32>(1.0) - kS;
            kD = kD * (1.0 - metallic);

            let NdotL = max(dot(N, L), 0.0);

            if !captured_debug {
                debug_F = F_light;
                debug_G = G;
                debug_D = NDF;
                debug_diffuse = kD * albedo / PI * radiance * NdotL;
                debug_specular = specular * radiance * NdotL;
                captured_debug = true;
            }

            let front_diffuse = kD * albedo / PI * NdotL;
            var diffuse_term = front_diffuse;
            if dt_active {
                let back_NdotL = max(dot(-N, L), 0.0);
                let back_diffuse = kD * diffuse_transmission_color * albedo / PI * back_NdotL;
                diffuse_term = mix(front_diffuse, back_diffuse, diffuse_transmission_factor);
            }
            var light_contribution = (diffuse_term + specular * energy_compensation * NdotL) * radiance;

            if sheen_active && NdotL > 0.0 {
                let NdotH = max(dot(N, H), 0.0);
                let sheen_brdf = sheen_lobe(sheen_color, NdotV, NdotL, NdotH, sheen_roughness);
                light_contribution = light_contribution * sheen_albedo_scale + sheen_brdf * radiance * NdotL;
            }

            if cc_factor > 0.0 {
                let cc_NdotL = max(dot(cc_normal, L), 0.0);
                if cc_NdotL > 0.0 {
                    let cc_H = normalize(V + L);
                    let cc_NdotH = max(dot(cc_normal, cc_H), 0.0);
                    let cc_VdotH = max(dot(V, cc_H), 0.0);
                    let cc_lobe = clearcoat_specular_lobe(cc_NdotV, cc_NdotL, cc_NdotH, cc_VdotH, cc_roughness);
                    let cc_Fc = fresnel_clearcoat(cc_VdotH);
                    light_contribution = light_contribution * (1.0 - cc_factor * cc_Fc) + cc_factor * cc_lobe * radiance * cc_NdotL;
                }
            }

            light_contribution = light_contribution * shadow_factor;

            Lo = Lo + light_contribution;
        }

        let cluster_idx = get_cluster_index(in.position.xy, computed_view_depth);
        let grid = light_grid[cluster_idx];
        let base = cluster_idx * MAX_LIGHTS_PER_CLUSTER;
        let cluster_shadow_lod = get_shadow_lod(computed_view_depth);

        for (var i = 0u; i < min(grid.count, MAX_LIGHTS_PER_CLUSTER); i = i + 1u) {
            let light_idx = light_indices[base + i];
            let light = lights[cluster_uniforms.num_directional_lights + light_idx];

            let point_to_light = light.position.xyz - in.world_pos;

            let L = normalize(point_to_light);
            let H = normalize(V + L);

            let radiance = getLightIntensity(light, point_to_light, in.world_pos);

            let F_light = fresnelSchlick(max(dot(H, V), 0.0), F0);
            var NDF = 0.0;
            var G = 0.0;
            var specular: vec3<f32>;
            if anisotropy_strength > 0.0 {
                let alpha = roughness * roughness;
                let at = mix(alpha, 1.0, anisotropy_strength * anisotropy_strength);
                let ab = alpha;
                let aniso_NdotL = max(dot(N, L), 0.0);
                let aniso_NdotH = max(dot(N, H), 0.0);
                let aniso_TdotH = dot(aniso_T, H);
                let aniso_BdotH = dot(aniso_B, H);
                let aniso_TdotV = dot(aniso_T, V);
                let aniso_BdotV = dot(aniso_B, V);
                let aniso_TdotL = dot(aniso_T, L);
                let aniso_BdotL = dot(aniso_B, L);
                NDF = d_ggx_anisotropic(aniso_NdotH, aniso_TdotH, aniso_BdotH, at, ab);
                let aniso_V = v_ggx_anisotropic(aniso_NdotL, NdotV, aniso_BdotV, aniso_TdotV, aniso_BdotL, aniso_TdotL, at, ab);
                G = aniso_V * 4.0 * NdotV * max(dot(N, L), 0.001);
                specular = F_light * NDF * aniso_V;
            } else {
                NDF = DistributionGGX(N, H, roughness);
                let vis = V_SmithGGXCorrelated(NdotV, max(dot(N, L), 0.0), roughness);
                G = vis * 4.0 * NdotV * max(dot(N, L), 0.001);
                specular = NDF * vis * F_light;
            }

            let kS = F_light;
            var kD = vec3<f32>(1.0) - kS;
            kD = kD * (1.0 - metallic);

            let NdotL = max(dot(N, L), 0.0);

            if !captured_debug {
                debug_F = F_light;
                debug_G = G;
                debug_D = NDF;
                debug_diffuse = kD * albedo / PI * radiance * NdotL;
                debug_specular = specular * radiance * NdotL;
                captured_debug = true;
            }

            let front_diffuse = kD * albedo / PI * NdotL;
            var diffuse_term = front_diffuse;
            if dt_active {
                let back_NdotL = max(dot(-N, L), 0.0);
                let back_diffuse = kD * diffuse_transmission_color * albedo / PI * back_NdotL;
                diffuse_term = mix(front_diffuse, back_diffuse, diffuse_transmission_factor);
            }
            var light_contribution = (diffuse_term + specular * energy_compensation * NdotL) * radiance;

            if sheen_active && NdotL > 0.0 {
                let NdotH = max(dot(N, H), 0.0);
                let sheen_brdf = sheen_lobe(sheen_color, NdotV, NdotL, NdotH, sheen_roughness);
                light_contribution = light_contribution * sheen_albedo_scale + sheen_brdf * radiance * NdotL;
            }

            if cc_factor > 0.0 {
                let cc_NdotL = max(dot(cc_normal, L), 0.0);
                if cc_NdotL > 0.0 {
                    let cc_H = normalize(V + L);
                    let cc_NdotH = max(dot(cc_normal, cc_H), 0.0);
                    let cc_VdotH = max(dot(V, cc_H), 0.0);
                    let cc_lobe = clearcoat_specular_lobe(cc_NdotV, cc_NdotL, cc_NdotH, cc_VdotH, cc_roughness);
                    let cc_Fc = fresnel_clearcoat(cc_VdotH);
                    light_contribution = light_contribution * (1.0 - cc_factor * cc_Fc) + cc_factor * cc_lobe * radiance * cc_NdotL;
                }
            }

            if light.light_type == LIGHT_TYPE_SPOT {
                let spotlight_shadow = calculate_spotlight_shadow_factor(in.world_pos, N, light.position.xyz, light.shadow_index, cluster_shadow_lod, light.light_size, shadow_rotation);
                light_contribution = light_contribution * spotlight_shadow;
            } else if light.light_type == LIGHT_TYPE_POINT {
                let point_shadow = calculate_point_light_shadow_factor(in.world_pos, N, light.shadow_index, light.light_size);
                light_contribution = light_contribution * point_shadow;
            }

            Lo = Lo + light_contribution;
        }

        for (var area_index = 0u; area_index < area_light_count(); area_index = area_index + 1u) {
            let area_light = area_light_at(area_index);
            var area_contribution = evaluate_one_area_light(area_light, in.world_pos, N, V, albedo, roughness, metallic, F0);
            if area_light.shadow_index >= 0 {
                area_contribution = area_contribution * calculate_spotlight_shadow_factor(in.world_pos, N, area_light.position.xyz, area_light.shadow_index, cluster_shadow_lod, area_light.radius, shadow_rotation);
            }
            Lo = Lo + area_contribution;
        }

        let F = fresnelSchlickRoughness(NdotV, F0, roughness);

        let irradiance_a = clamp(textureSampleLevel(irradiance_map, irradiance_sampler, N, 0.0).rgb, vec3<f32>(0.0), vec3<f32>(65000.0));
        let irradiance_b = clamp(textureSampleLevel(irradiance_map_b, irradiance_sampler, N, 0.0).rgb, vec3<f32>(0.0), vec3<f32>(65000.0));
        let irradiance = mix(irradiance_a, irradiance_b, uniforms.ibl_blend_factor);

        let MAX_REFLECTION_LOD = 4.0;
        var ibl_R = R;
        if anisotropy_strength > 0.0 {
            let anisotropic_tangent_iblad = cross(aniso_B, V);
            let anisotropic_normal_iblad = cross(anisotropic_tangent_iblad, aniso_B);
            let bend_factor = 1.0 - anisotropy_strength * (1.0 - roughness);
            let bend_factor_sq = bend_factor * bend_factor;
            let bent_normal = normalize(mix(anisotropic_normal_iblad, N, bend_factor_sq * bend_factor_sq));
            ibl_R = reflect(-V, bent_normal);
        }
        var reflection_roughness = roughness;
        if material.reflection_box_max.x > material.reflection_box_min.x {
            let box_projection = box_project_reflection(ibl_R, in.world_pos, material.reflection_box_min, material.reflection_box_max);
            ibl_R = box_projection.xyz;
            reflection_roughness = box_projected_roughness(roughness, box_projection.w, length(box_projection.xyz));
        }
        let prefiltered_color_a = clamp(textureSampleLevel(prefiltered_env, prefiltered_sampler, ibl_R, reflection_roughness * MAX_REFLECTION_LOD).rgb, vec3<f32>(0.0), vec3<f32>(65000.0));
        let prefiltered_color_b = clamp(textureSampleLevel(prefiltered_env_b, prefiltered_sampler, ibl_R, reflection_roughness * MAX_REFLECTION_LOD).rgb, vec3<f32>(0.0), vec3<f32>(65000.0));
        let prefiltered_color = mix(prefiltered_color_a, prefiltered_color_b, uniforms.ibl_blend_factor);

        let Fss_Ess = F * brdf.x + brdf.y;

        let Ems = 1.0 - (brdf.x + brdf.y);
        let Favg = F0 + (1.0 - F0) / 21.0;
        let Fms_Ems = Ems * Fss_Ess * Favg / (1.0 - Favg * Ems);

        let c_diff = albedo * (1.0 - metallic);
        let kD_ibl = c_diff * (1.0 - Fss_Ess - Fms_Ems);
        var diffuse_ibl = (Fms_Ems + kD_ibl) * irradiance;

        if dt_active {
            let back_irradiance_a = clamp(textureSampleLevel(irradiance_map, irradiance_sampler, -N, 0.0).rgb, vec3<f32>(0.0), vec3<f32>(65000.0));
            let back_irradiance_b = clamp(textureSampleLevel(irradiance_map_b, irradiance_sampler, -N, 0.0).rgb, vec3<f32>(0.0), vec3<f32>(65000.0));
            let back_irradiance = mix(back_irradiance_a, back_irradiance_b, uniforms.ibl_blend_factor);
            let c_diff_back = diffuse_transmission_color * albedo * (1.0 - metallic);
            let kD_ibl_back = c_diff_back * (1.0 - Fss_Ess - Fms_Ems);
            let dt_diffuse_ibl = (Fms_Ems + kD_ibl_back) * back_irradiance;
            diffuse_ibl = mix(diffuse_ibl, dt_diffuse_ibl, diffuse_transmission_factor);
        }

        if transmission_factor > 0.0 {
            let model_scale = vec3<f32>(in.world_scale_factor);
            let transmission = getIBLVolumeRefraction(
                N, V, in.world_pos, roughness, albedo, F0, ior, material.dispersion,
                thickness, material.attenuation_color, material.attenuation_distance, model_scale,
                cluster_uniforms.screen_size.x
            );
            diffuse_ibl = mix(diffuse_ibl, transmission, transmission_factor);
        }

        let ibl_horizon = clamp(1.0 + dot(ibl_R, N), 0.0, 1.0);
        let specular_ibl = prefiltered_color * Fss_Ess * ibl_horizon * ibl_horizon;

        let ao_diffuse = mix(1.0, occlusion, material.occlusion_strength);
        let ao_specular = mix(1.0, specular_occlusion(NdotV, occlusion, roughness), material.occlusion_strength);

        var ambient = diffuse_ibl * ao_diffuse + specular_ibl * ao_specular;

        if sheen_active {
            let sheen_prefiltered_a = clamp(textureSampleLevel(prefiltered_env, prefiltered_sampler, R, sheen_roughness * MAX_REFLECTION_LOD).rgb, vec3<f32>(0.0), vec3<f32>(65000.0));
            let sheen_prefiltered_b = clamp(textureSampleLevel(prefiltered_env_b, prefiltered_sampler, R, sheen_roughness * MAX_REFLECTION_LOD).rgb, vec3<f32>(0.0), vec3<f32>(65000.0));
            let sheen_prefiltered = mix(sheen_prefiltered_a, sheen_prefiltered_b, uniforms.ibl_blend_factor);
            let sheen_ibl = sheen_color * sheen_E * sheen_prefiltered;
            ambient = ambient * sheen_albedo_scale + sheen_ibl * ao_specular;
        }

        if cc_factor > 0.0 {
            let cc_F_ibl = fresnel_clearcoat(cc_NdotV);
            let cc_prefiltered_a = clamp(textureSampleLevel(prefiltered_env, prefiltered_sampler, cc_R, cc_roughness * MAX_REFLECTION_LOD).rgb, vec3<f32>(0.0), vec3<f32>(65000.0));
            let cc_prefiltered_b = clamp(textureSampleLevel(prefiltered_env_b, prefiltered_sampler, cc_R, cc_roughness * MAX_REFLECTION_LOD).rgb, vec3<f32>(0.0), vec3<f32>(65000.0));
            let cc_prefiltered = mix(cc_prefiltered_a, cc_prefiltered_b, uniforms.ibl_blend_factor);
            let cc_horizon = clamp(1.0 + dot(cc_R, cc_normal), 0.0, 1.0);
            let cc_ibl_specular = cc_factor * (cc_F_ibl * cc_brdf.x + cc_brdf.y) * cc_prefiltered * cc_horizon * cc_horizon;
            ambient = ambient * (1.0 - cc_factor * cc_F_ibl) + cc_ibl_specular * ao_specular;
        }

        let color = ambient + Lo;

        final_color = color + emission;
    }

    if uniforms.fog_enabled != 0u {
        var fog_factor: f32;
        if uniforms.fog_enabled == 1u {
            fog_factor = clamp((computed_view_depth - uniforms.fog_start) / (uniforms.fog_end - uniforms.fog_start), 0.0, 1.0);
        } else {
            let d = max(computed_view_depth - uniforms.fog_start, 0.0);
            let x = d * 3.0 / max(uniforms.fog_end - uniforms.fog_start, 0.001);
            if uniforms.fog_enabled == 2u {
                fog_factor = 1.0 - exp(-x);
            } else {
                fog_factor = 1.0 - exp(-x * x);
            }
        }
        final_color = mix(final_color, uniforms.fog_color, fog_factor);
    }

    final_color = clamp(select(vec3<f32>(0.0), final_color, final_color == final_color), vec3<f32>(0.0), vec3<f32>(65000.0));

    if uniforms.texture_debug_stripes != 0u {
        let num_stripes = 7u;
        let line_width = 3.0;
        let max_diagonal = 2560.0 + 1440.0;

        let diagonal = in.position.x + in.position.y - uniforms.time * uniforms.texture_debug_stripes_speed;
        let band_size = max_diagonal / f32(num_stripes);
        let wrapped_pos = ((diagonal % max_diagonal) + max_diagonal) % max_diagonal;
        let stripe_index = u32(floor(wrapped_pos / band_size)) % num_stripes;
        let pos_in_band = fract(wrapped_pos / band_size);

        let is_line = pos_in_band < (line_width / band_size) || pos_in_band > (1.0 - line_width / band_size);

        var stripe_color: vec3<f32>;
        switch stripe_index {
            case 0u: {
                stripe_color = vec3<f32>(occlusion);
            }
            case 1u: {
                stripe_color = emission;
            }
            case 2u: {
                stripe_color = vec3<f32>(metallic);
            }
            case 3u: {
                stripe_color = albedo;
            }
            case 4u: {
                stripe_color = vec3<f32>(roughness);
            }
            case 5u: {
                stripe_color = normal * 0.5 + 0.5;
            }
            case 6u: {
                stripe_color = final_color;
            }
            default: {
                stripe_color = final_color;
            }
        }

        if is_line {
            stripe_color = vec3<f32>(1.0);
        }

        output.color = vec4<f32>(stripe_color, 1.0);
        output.entity_id = bitcast<f32>(in.entity_id);
        output.view_normal = vec4<f32>(normalize(in.view_normal), 1.0);
        return output;
    }

    if uniforms.pbr_debug_mode != PBR_DEBUG_NONE {
        var debug_color = vec3<f32>(0.0);
        switch uniforms.pbr_debug_mode {
            case PBR_DEBUG_BASE_COLOR: {
                debug_color = albedo;
            }
            case PBR_DEBUG_NORMAL: {
                debug_color = normal * 0.5 + 0.5;
            }
            case PBR_DEBUG_METALLIC: {
                debug_color = vec3<f32>(metallic);
            }
            case PBR_DEBUG_ROUGHNESS: {
                debug_color = vec3<f32>(roughness);
            }
            case PBR_DEBUG_OCCLUSION: {
                debug_color = vec3<f32>(occlusion);
            }
            case PBR_DEBUG_EMISSIVE: {
                debug_color = emission;
            }
            case PBR_DEBUG_F: {
                debug_color = debug_F;
            }
            case PBR_DEBUG_G: {
                debug_color = vec3<f32>(debug_G);
            }
            case PBR_DEBUG_D: {
                debug_color = vec3<f32>(debug_D);
            }
            case PBR_DEBUG_DIFFUSE: {
                debug_color = debug_diffuse;
            }
            case PBR_DEBUG_SPECULAR: {
                debug_color = debug_specular;
            }
            case PBR_DEBUG_MIP_RAINBOW: {
                debug_color = mip_rainbow_color(base_uv, 1024.0);
            }
            default: {
                debug_color = final_color;
            }
        }
        output.color = vec4<f32>(debug_color, 1.0);
        output.entity_id = bitcast<f32>(in.entity_id);
        output.view_normal = vec4<f32>(normalize(in.view_normal), 1.0);
        return output;
    }

    output.color = vec4<f32>(final_color, alpha);
    output.entity_id = bitcast<f32>(in.entity_id);
    output.view_normal = vec4<f32>(normalize(in.view_normal), 1.0);
    return output;
}
