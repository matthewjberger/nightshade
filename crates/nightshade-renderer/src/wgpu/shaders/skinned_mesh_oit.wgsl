const PI: f32 = 3.14159265358979323846;

#import nightshade::pbr_brdf::{DistributionGGX, GeometrySchlickGGX, GeometrySmith, fresnelSchlick, fresnelSchlickRoughness, specularAntiAliasing, fresnel_clearcoat, clearcoat_specular_lobe, d_charlie, v_neubelt, sheen_lobe, fresnel0_to_ior, ior_to_fresnel0_v, ior_to_fresnel0_f, eval_sensitivity, d_ggx_anisotropic, v_ggx_anisotropic, eval_iridescence, getRangeAttenuation, getSpotAttenuation, applyVolumeAttenuation, applyIorToRoughness, box_project_reflection, box_projected_roughness}


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
    let scene_sample = textureSampleLevel(transmission_color_texture, transmission_color_sampler, clamped_uv, 0.0).rgb;
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
    model_scale: vec3<f32>
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
        let r = getScreenSpaceTransmission(world_pos, dir_r, transmission_distance, ior_r, roughness).r;
        let g = getScreenSpaceTransmission(world_pos, dir_g, transmission_distance, ior, roughness).g;
        let b = getScreenSpaceTransmission(world_pos, dir_b, transmission_distance, ior_b, roughness).b;
        transmitted_light = vec3<f32>(r, g, b);
    } else {
        let dir = refract_safe(view, normal, 1.0 / ior);
        transmitted_light = getScreenSpaceTransmission(world_pos, dir, transmission_distance, ior, roughness);
    }
    let attenuated_color = applyVolumeAttenuation(transmitted_light, transmission_distance, attenuation_color, attenuation_distance);
    let NdotV = clamp(dot(normal, view), 0.001, 1.0);
    let brdf = textureSampleLevel(brdf_lut, brdf_lut_sampler, vec2<f32>(NdotV, roughness), 0.0).rg;
    let specular_color = F0 * brdf.x + brdf.y;
    return (1.0 - specular_color) * attenuated_color * base_color;
}

struct SkinnedVertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) tex_coords_1: vec2<f32>,
    @location(4) tangent: vec4<f32>,
    @location(5) color: vec4<f32>,
    @location(6) joint_indices: vec4<u32>,
    @location(7) joint_weights: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) world_pos: vec3<f32>,
    @location(2) @interpolate(flat) material_id: u32,
    @location(3) tex_coords: vec2<f32>,
    @location(4) tex_coords_1: vec2<f32>,
    @location(5) instance_tint: vec4<f32>,
    @location(6) vertex_color: vec4<f32>,
    @location(7) world_tangent: vec4<f32>,
    @location(8) @interpolate(linear) view_depth: f32,
    @location(9) @interpolate(flat) flip_winding: u32,
    @location(10) @interpolate(flat) world_scale_factor: f32,
    @location(11) @interpolate(flat) entity_id: u32,
    @location(12) prev_world_pos: vec3<f32>,
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
    _padding2: f32,
    cascade_view_projections: array<mat4x4<f32>, 4>,
    cascade_split_distances: vec4<f32>,
    cascade_atlas_offsets: array<vec4<f32>, 4>,
    cascade_atlas_scale: vec4<f32>,
    time: f32,
    directional_light_direction: vec4<f32>,
    ibl_blend_factor: f32,
    oit_z_scale: f32,
    flat_color: vec4<f32>,
    prev_view_projection: mat4x4<f32>,
    cur_view_projection: mat4x4<f32>,
};

struct TextureTransform {
    row0: vec4<f32>,
    row1: vec4<f32>,
};

struct Material {
    base_color: vec4<f32>,
    emissive_factor: vec3<f32>,
    alpha_mode: u32,
    alpha_cutoff: f32,
    base_layer: u32,
    emissive_layer: u32,
    normal_layer: u32,
    metallic_roughness_layer: u32,
    occlusion_layer: u32,
    normal_scale: f32,
    occlusion_strength: f32,
    roughness: f32,
    metallic: f32,
    unlit: u32,
    normal_map_flags: u32,
    transmission_factor: f32,
    transmission_layer: u32,
    thickness: f32,
    thickness_layer: u32,
    attenuation_color: vec3<f32>,
    attenuation_distance: f32,
    ior: f32,
    specular_factor: f32,
    _align_specular: vec2<u32>,
    specular_color_factor: vec3<f32>,
    specular_layer: u32,
    specular_color_layer: u32,
    emissive_strength: f32,
    _pad_before_transforms: vec2<u32>,
    base_transform: TextureTransform,
    emissive_transform: TextureTransform,
    normal_transform: TextureTransform,
    metallic_roughness_transform: TextureTransform,
    occlusion_transform: TextureTransform,
    transmission_transform: TextureTransform,
    thickness_transform: TextureTransform,
    specular_transform: TextureTransform,
    specular_color_transform: TextureTransform,
    clearcoat_factor: f32,
    clearcoat_roughness_factor: f32,
    clearcoat_normal_scale: f32,
    clearcoat_layer: u32,
    clearcoat_roughness_layer: u32,
    clearcoat_normal_layer: u32,
    _pad_clearcoat: vec2<u32>,
    clearcoat_transform: TextureTransform,
    clearcoat_roughness_transform: TextureTransform,
    clearcoat_normal_transform: TextureTransform,
    sheen_color_factor: vec3<f32>,
    sheen_roughness_factor: f32,
    sheen_color_layer: u32,
    _pad_sheen0: u32,
    _pad_sheen: vec2<u32>,
    sheen_color_transform: TextureTransform,
    sheen_roughness_transform: TextureTransform,
    iridescence_factor: f32,
    iridescence_ior: f32,
    iridescence_thickness_minimum: f32,
    iridescence_thickness_maximum: f32,
    iridescence_layer: u32,
    _pad_iridescence0: u32,
    _pad_iridescence: vec2<u32>,
    iridescence_transform: TextureTransform,
    iridescence_thickness_transform: TextureTransform,
    anisotropy_strength: f32,
    anisotropy_rotation_cos: f32,
    anisotropy_rotation_sin: f32,
    anisotropy_layer: u32,
    anisotropy_transform: TextureTransform,
    dispersion: f32,
    diffuse_transmission_factor: f32,
    _pad_diffuse_transmission0: u32,
    diffuse_transmission_color_layer: u32,
    diffuse_transmission_color_factor: vec3<f32>,
    blend_opaque_alpha_threshold: f32,
    diffuse_transmission_transform: TextureTransform,
    diffuse_transmission_color_transform: TextureTransform,
    detail_base_layer: u32,
    detail_normal_layer: u32,
    height_layer: u32,
    detail_uv_scale: f32,
    parallax_scale: f32,
    _pad_detail0: u32,
    _pad_detail1: u32,
    _pad_detail2: u32,
    reflection_box_min: vec3<f32>,
    _pad_box_min: f32,
    reflection_box_max: vec3<f32>,
    _pad_box_max: f32,
};

const NORMAL_MAP_FLIP_Y: u32 = 1u;
const NORMAL_MAP_TWO_COMPONENT: u32 = 2u;

fn texture_uv(uv0: vec2<f32>, uv1: vec2<f32>, transform: TextureTransform) -> vec2<f32> {
    let uv_set = u32(transform.row0.w);
    let uv = select(uv0, uv1, uv_set == 1u);
    let homogeneous = vec3<f32>(uv, 1.0);
    let u = dot(transform.row0.xyz, homogeneous);
    let v = dot(transform.row1.xyz, homogeneous);
    return vec2<f32>(u, v);
}

struct SkinnedObjectData {
    transform_index: u32,
    mesh_id: u32,
    material_id: u32,
    joint_offset: u32,
    morph_weights: array<f32, 8>,
    morph_target_count: u32,
    morph_displacement_offset: u32,
    mesh_vertex_offset: u32,
    mesh_vertex_count: u32,
    flip_winding: u32,
    entity_id: u32,
    _pad0: u32,
    _pad1: u32,
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

const LIGHT_TYPE_DIRECTIONAL: u32 = 0u;
const LIGHT_TYPE_POINT: u32 = 1u;
const LIGHT_TYPE_SPOT: u32 = 2u;

struct OITOutput {
    @location(0) accum: vec4<f32>,
    @location(1) reveal: f32,
    @location(2) entity_id: f32,
    @location(3) velocity: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var<storage, read> materials: array<Material>;

@group(1) @binding(1)
var<storage, read> objects: array<SkinnedObjectData>;

@group(1) @binding(6)
var<storage, read> visible_indices: array<u32>;

@group(1) @binding(2)
var<storage, read> lights: array<Light>;

@group(1) @binding(3)
var<storage, read> joint_matrices: array<mat4x4<f32>>;

@group(1) @binding(12)
var<storage, read> prev_joint_matrices: array<mat4x4<f32>>;

@group(1) @binding(4)
var<storage, read> instance_custom_data: array<vec4<f32>>;

@group(1) @binding(5)
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

const MAX_LIGHTS_PER_CLUSTER: u32 = 256u;

@group(1) @binding(7)
var<storage, read> light_grid: array<LightGrid>;

@group(1) @binding(8)
var<storage, read> light_indices: array<u32>;

@group(1) @binding(9)
var<uniform> cluster_uniforms: ClusterUniforms;

#import nightshade::material_sampling::{sample_srgb_layer, sample_linear_layer, NO_LAYER, UvGrad}

var<private> material_uv0_ddx: vec2<f32>;
var<private> material_uv0_ddy: vec2<f32>;
var<private> material_uv1_ddx: vec2<f32>;
var<private> material_uv1_ddy: vec2<f32>;

fn texture_uv_grad(transform: TextureTransform) -> UvGrad {
    let uv_set = u32(transform.row0.w);
    let ddx = select(material_uv0_ddx, material_uv1_ddx, uv_set == 1u);
    let ddy = select(material_uv0_ddy, material_uv1_ddy, uv_set == 1u);
    var out: UvGrad;
    out.ddx = vec2<f32>(dot(transform.row0.xy, ddx), dot(transform.row1.xy, ddx));
    out.ddy = vec2<f32>(dot(transform.row0.xy, ddy), dot(transform.row1.xy, ddy));
    return out;
}
#import nightshade::area_lighting::{area_light_count, area_light_at, evaluate_one_area_light}

@group(3) @binding(0)
var shadow_texture: texture_depth_2d;

@group(3) @binding(1)
var shadow_sampler: sampler;

@group(3) @binding(2)
var spotlight_shadow_atlas: texture_depth_2d;

@group(3) @binding(3)
var spotlight_shadow_sampler: sampler;

@group(3) @binding(4)
var<storage, read> spotlight_shadows: array<vec4<f32>>;

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

@group(3) @binding(16)
var transmission_color_texture: texture_2d<f32>;

@group(3) @binding(17)
var transmission_color_sampler: sampler;

@vertex
fn vs_main(
    in: SkinnedVertexInput,
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_id: u32
) -> VertexOutput {
    var out: VertexOutput;

    let object = objects[visible_indices[instance_id]];

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

    var skinned_position = vec3<f32>(0.0, 0.0, 0.0);
    var prev_skinned_position = vec3<f32>(0.0, 0.0, 0.0);
    var skinned_normal = vec3<f32>(0.0, 0.0, 0.0);
    var skinned_tangent = vec3<f32>(0.0, 0.0, 0.0);
    var blended_scale = 0.0;

    let joint_offset = object.joint_offset;

    for (var index = 0u; index < 4u; index = index + 1u) {
        let joint_index = in.joint_indices[index];
        let joint_weight = in.joint_weights[index];

        if (joint_weight > 0.0) {
            let joint_matrix = joint_matrices[joint_offset + joint_index];

            let transformed_pos = joint_matrix * vec4<f32>(position, 1.0);
            skinned_position = skinned_position + transformed_pos.xyz * joint_weight;

            let prev_joint_matrix = prev_joint_matrices[joint_offset + joint_index];
            let prev_transformed_pos = prev_joint_matrix * vec4<f32>(position, 1.0);
            prev_skinned_position = prev_skinned_position + prev_transformed_pos.xyz * joint_weight;

            let normal_matrix = mat3x3<f32>(
                joint_matrix[0].xyz,
                joint_matrix[1].xyz,
                joint_matrix[2].xyz
            );
            let transformed_normal = normal_matrix * normal;
            skinned_normal = skinned_normal + transformed_normal * joint_weight;

            let transformed_tangent = normal_matrix * tangent;
            skinned_tangent = skinned_tangent + transformed_tangent * joint_weight;

            let joint_scale = (length(joint_matrix[0].xyz)
                + length(joint_matrix[1].xyz)
                + length(joint_matrix[2].xyz)) / 3.0;
            blended_scale = blended_scale + joint_scale * joint_weight;
        }
    }

    skinned_normal = normalize(skinned_normal);
    skinned_tangent = normalize(skinned_tangent);

    let world_position = vec4<f32>(skinned_position, 1.0);
    out.world_pos = world_position.xyz;
    out.prev_world_pos = prev_skinned_position;

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
    out.normal = skinned_normal;
    out.world_tangent = vec4<f32>(skinned_tangent, in.tangent.w);
    out.material_id = object.material_id;
    out.flip_winding = object.flip_winding;
    out.world_scale_factor = blended_scale;
    out.tex_coords = in.tex_coords;
    out.tex_coords_1 = in.tex_coords_1;
    out.instance_tint = instance_custom_data[instance_id];
    out.vertex_color = in.color;
    out.view_depth = -view_position.z;
    out.entity_id = object.entity_id;

    return out;
}

fn getLightIntensity(light: Light, point_to_light: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    var range_attenuation = 1.0;
    var spot_attenuation = 1.0;

    if light.light_type != LIGHT_TYPE_DIRECTIONAL {
        range_attenuation = getRangeAttenuation(light.range, length(point_to_light));
    }
    if light.light_type == LIGHT_TYPE_SPOT {
        spot_attenuation = getSpotAttenuation(point_to_light, light.direction.xyz, light.outer_cone, light.inner_cone);
    }

    return range_attenuation * spot_attenuation * light.color.rgb;
}

fn get_normal(
    world_normal: vec3<f32>,
    world_tangent: vec4<f32>,
    normal_sample: vec3<f32>,
    has_normal_texture: bool,
    normal_scale: f32,
    flags: u32,
) -> vec3<f32> {
    let N = normalize(world_normal);
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
    var T = normalize(world_tangent.xyz);
    T = normalize(T - N * dot(N, T));
    let B = cross(N, T) * world_tangent.w;
    let TBN = mat3x3<f32>(T, B, N);
    return normalize(TBN * tangent_normal);
}

fn get_cluster_index(frag_coord: vec2<f32>, linear_depth: f32) -> u32 {
    let tile = vec2<u32>(frag_coord / cluster_uniforms.tile_size);
    let log_ratio = log(cluster_uniforms.z_far / cluster_uniforms.z_near);
    let slice = u32(log(linear_depth / cluster_uniforms.z_near) / log_ratio * f32(cluster_uniforms.cluster_count.z));
    let clamped_slice = clamp(slice, 0u, cluster_uniforms.cluster_count.z - 1u);
    return tile.x + tile.y * cluster_uniforms.cluster_count.x + clamped_slice * cluster_uniforms.cluster_count.x * cluster_uniforms.cluster_count.y;
}

fn weight(view_z: f32, a: f32) -> f32 {
    let z_scale = max(uniforms.oit_z_scale, 1.0);
    return a * clamp(0.03 / (1e-5 + pow(view_z / z_scale, 4.0)), 1e-2, 3e3);
}

@fragment
fn fs_blend_opaque_prepass(in: VertexOutput, @builtin(front_facing) is_front_face: bool) {
    material_uv0_ddx = dpdx(in.tex_coords);
    material_uv0_ddy = dpdy(in.tex_coords);
    material_uv1_ddx = dpdx(in.tex_coords_1);
    material_uv1_ddy = dpdy(in.tex_coords_1);
    if in.flip_winding != 0u && is_front_face {
        discard;
    }
    if uniforms.flat_color.a >= 2.0 {
        discard;
    }
    let material = materials[in.material_id];
    if material.transmission_factor > 0.0 {
        return;
    }
    var alpha = material.base_color.a * in.instance_tint.a * in.vertex_color.a;
    if material.base_layer != NO_LAYER {
        let base_uv = texture_uv(in.tex_coords, in.tex_coords_1, material.base_transform);
        alpha = alpha * sample_srgb_layer(material.base_layer, base_uv, texture_uv_grad(material.base_transform)).a;
    }
    if alpha < material.blend_opaque_alpha_threshold {
        discard;
    }
}

@fragment
fn fs_main(in: VertexOutput, @builtin(front_facing) is_front_face: bool) -> OITOutput {
    material_uv0_ddx = dpdx(in.tex_coords);
    material_uv0_ddy = dpdy(in.tex_coords);
    material_uv1_ddx = dpdx(in.tex_coords_1);
    material_uv1_ddy = dpdy(in.tex_coords_1);
    if in.flip_winding != 0u && is_front_face {
        discard;
    }
    if uniforms.flat_color.a >= 2.0 {
        discard;
    }
    let material = materials[in.material_id];
    let uv0 = in.tex_coords;
    let uv1 = in.tex_coords_1;

    let base_uv = texture_uv(uv0, uv1, material.base_transform);
    let emissive_uv = texture_uv(uv0, uv1, material.emissive_transform);
    let normal_uv = texture_uv(uv0, uv1, material.normal_transform);
    let metallic_roughness_uv = texture_uv(uv0, uv1, material.metallic_roughness_transform);
    let occlusion_uv = texture_uv(uv0, uv1, material.occlusion_transform);
    let transmission_uv = texture_uv(uv0, uv1, material.transmission_transform);
    let thickness_uv = texture_uv(uv0, uv1, material.thickness_transform);
    let specular_uv = texture_uv(uv0, uv1, material.specular_transform);
    let specular_color_uv = texture_uv(uv0, uv1, material.specular_color_transform);

    let normal_tex_sample = select(
        vec3<f32>(0.5, 0.5, 1.0),
        sample_linear_layer(material.normal_layer, normal_uv, texture_uv_grad(material.normal_transform)).xyz,
        material.normal_layer != NO_LAYER,
    );

    let view_dir_initial = normalize(uniforms.camera_position.xyz - in.world_pos);
    var geom_normal = in.normal;
    var geom_tangent = in.world_tangent;
    if dot(geom_normal, view_dir_initial) < 0.0 {
        geom_normal = -geom_normal;
        geom_tangent = -geom_tangent;
    }

    var normal = get_normal(
        geom_normal,
        geom_tangent,
        normal_tex_sample,
        material.normal_layer != NO_LAYER,
        material.normal_scale,
        material.normal_map_flags,
    );
    if uniforms.flat_color.a > 0.0 {
        normal = geom_normal;
    }

    let base_tex_sample = select(
        vec4<f32>(1.0),
        sample_srgb_layer(material.base_layer, base_uv, texture_uv_grad(material.base_transform)),
        material.base_layer != NO_LAYER,
    );
    let emissive_tex_sample = select(
        vec4<f32>(0.0),
        sample_srgb_layer(material.emissive_layer, emissive_uv, texture_uv_grad(material.emissive_transform)),
        material.emissive_layer != NO_LAYER,
    );

    var base_color = material.base_color * in.instance_tint * in.vertex_color;
    if material.base_layer != NO_LAYER {
        base_color = base_color * base_tex_sample;
    }
    if uniforms.flat_color.a > 0.0 {
        base_color = vec4<f32>(uniforms.flat_color.rgb, base_color.a);
    }

    let albedo = base_color.rgb;

    var metallic = material.metallic;
    var roughness = material.roughness;
    if material.metallic_roughness_layer != NO_LAYER {
        let mr_sample = sample_linear_layer(material.metallic_roughness_layer, metallic_roughness_uv, texture_uv_grad(material.metallic_roughness_transform));
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
        occlusion = sample_linear_layer(material.occlusion_layer, occlusion_uv, texture_uv_grad(material.occlusion_transform)).r;
    }

    var transmission_factor = material.transmission_factor;
    if material.transmission_layer != NO_LAYER {
        let transmission_sample = sample_linear_layer(material.transmission_layer, transmission_uv, texture_uv_grad(material.transmission_transform)).r;
        transmission_factor = transmission_factor * transmission_sample;
    }

    var thickness = material.thickness;
    if material.thickness_layer != NO_LAYER {
        let thickness_sample = sample_linear_layer(material.thickness_layer, thickness_uv, texture_uv_grad(material.thickness_transform)).g;
        thickness = thickness * thickness_sample;
    }

    var specular_factor = material.specular_factor;
    if material.specular_layer != NO_LAYER {
        let specular_sample = sample_linear_layer(material.specular_layer, specular_uv, texture_uv_grad(material.specular_transform)).a;
        specular_factor = specular_factor * specular_sample;
    }

    var specular_color_factor = material.specular_color_factor;
    if material.specular_color_layer != NO_LAYER {
        let specular_color_sample = sample_srgb_layer(material.specular_color_layer, specular_color_uv, texture_uv_grad(material.specular_color_transform)).rgb;
        specular_color_factor = specular_color_factor * specular_color_sample;
    }

    var emission = material.emissive_factor * material.emissive_strength;
    if material.emissive_layer != NO_LAYER {
        emission = emission * emissive_tex_sample.rgb;
    }
    if uniforms.flat_color.a > 0.0 {
        emission = vec3<f32>(0.0);
    }

    var final_color: vec3<f32>;

    if material.unlit != 0u || uniforms.global_unlit > 0.5 {
        final_color = albedo + emission;
    } else {
        let N = normal;
        let V = normalize(uniforms.camera_position.xyz - in.world_pos);
        let R = reflect(-V, N);
        let NdotV = max(dot(N, V), 0.0);

        let ior = material.ior;
        let dielectric_f0 = pow((ior - 1.0) / (ior + 1.0), 2.0);
        let dielectric_specular_f0 = min(vec3<f32>(dielectric_f0) * specular_color_factor, vec3<f32>(1.0));
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
            let iridescent_f0 = eval_iridescence(1.0, material.iridescence_ior, NdotV, iridescence_thickness, F0);
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
        let aniso_T_world = normalize(geom_tangent.xyz);
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
            let cc_T = normalize(geom_tangent.xyz);
            let cc_B = cross(geom_normal, cc_T) * geom_tangent.w;
            let cc_TBN = mat3x3<f32>(cc_T, cc_B, geom_normal);
            cc_normal = normalize(cc_TBN * cc_n_tangent);
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

        for (var index = 0u; index < cluster_uniforms.num_directional_lights; index = index + 1u) {
            let light = lights[index];

            let point_to_light = -light.direction.xyz;

            let L = normalize(point_to_light);
            let H = normalize(V + L);

            let radiance = getLightIntensity(light, point_to_light, in.world_pos);

            let F = fresnelSchlick(max(dot(H, V), 0.0), F0);
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
                let aniso_D = d_ggx_anisotropic(aniso_NdotH, aniso_TdotH, aniso_BdotH, at, ab);
                let aniso_V = v_ggx_anisotropic(aniso_NdotL, NdotV, aniso_BdotV, aniso_TdotV, aniso_BdotL, aniso_TdotL, at, ab);
                specular = F * aniso_D * aniso_V;
            } else {
                let NDF = DistributionGGX(N, H, roughness);
                let G = GeometrySmith(N, V, L, roughness);
                let numerator = NDF * G * F;
                let denominator = 4.0 * NdotV * max(dot(N, L), 0.0) + 0.001;
                specular = numerator / denominator;
            }

            let kS = F;
            var kD = vec3<f32>(1.0) - kS;
            kD = kD * (1.0 - metallic);

            let NdotL = max(dot(N, L), 0.0);

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

            Lo = Lo + light_contribution;
        }

        let cluster_index = get_cluster_index(in.position.xy, in.view_depth);
        let grid = light_grid[cluster_index];
        let base_offset = cluster_index * MAX_LIGHTS_PER_CLUSTER;

        for (var index = 0u; index < min(grid.count, MAX_LIGHTS_PER_CLUSTER); index = index + 1u) {
            let light_index = light_indices[base_offset + index];
            let light = lights[cluster_uniforms.num_directional_lights + light_index];

            let point_to_light = light.position.xyz - in.world_pos;

            let L = normalize(point_to_light);
            let H = normalize(V + L);

            let radiance = getLightIntensity(light, point_to_light, in.world_pos);

            let F = fresnelSchlick(max(dot(H, V), 0.0), F0);
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
                let aniso_D = d_ggx_anisotropic(aniso_NdotH, aniso_TdotH, aniso_BdotH, at, ab);
                let aniso_V = v_ggx_anisotropic(aniso_NdotL, NdotV, aniso_BdotV, aniso_TdotV, aniso_BdotL, aniso_TdotL, at, ab);
                specular = F * aniso_D * aniso_V;
            } else {
                let NDF = DistributionGGX(N, H, roughness);
                let G = GeometrySmith(N, V, L, roughness);
                let numerator = NDF * G * F;
                let denominator = 4.0 * NdotV * max(dot(N, L), 0.0) + 0.001;
                specular = numerator / denominator;
            }

            let kS = F;
            var kD = vec3<f32>(1.0) - kS;
            kD = kD * (1.0 - metallic);

            let NdotL = max(dot(N, L), 0.0);

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

            Lo = Lo + light_contribution;
        }

        for (var area_index = 0u; area_index < area_light_count(); area_index = area_index + 1u) {
            let area_light = area_light_at(area_index);
            Lo = Lo + evaluate_one_area_light(area_light, in.world_pos, N, V, albedo, roughness, metallic, F0);
        }

        let F = fresnelSchlickRoughness(NdotV, F0, roughness);

        let irradiance = textureSampleLevel(irradiance_map, irradiance_sampler, N, 0.0).rgb;

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
        let prefiltered_color = textureSampleLevel(prefiltered_env, prefiltered_sampler, ibl_R, reflection_roughness * MAX_REFLECTION_LOD).rgb;

        let Fss_Ess = F * brdf.x + brdf.y;

        let Ems = 1.0 - (brdf.x + brdf.y);
        let Favg = F0 + (1.0 - F0) / 21.0;
        let Fms_Ems = Ems * Fss_Ess * Favg / (1.0 - Favg * Ems);

        let c_diff = albedo * (1.0 - metallic);
        let kD_ibl = c_diff * (1.0 - Fss_Ess - Fms_Ems);
        var diffuse_ibl = (Fms_Ems + kD_ibl) * irradiance;

        if dt_active {
            let back_irradiance = textureSampleLevel(irradiance_map, irradiance_sampler, -N, 0.0).rgb;
            let c_diff_back = diffuse_transmission_color * albedo * (1.0 - metallic);
            let kD_ibl_back = c_diff_back * (1.0 - Fss_Ess - Fms_Ems);
            let dt_diffuse_ibl = (Fms_Ems + kD_ibl_back) * back_irradiance;
            diffuse_ibl = mix(diffuse_ibl, dt_diffuse_ibl, diffuse_transmission_factor);
        }

        if transmission_factor > 0.0 {
            let model_scale = vec3<f32>(in.world_scale_factor);
            let transmission = getIBLVolumeRefraction(
                N, V, in.world_pos, roughness, albedo, F0, ior, material.dispersion,
                thickness, material.attenuation_color, material.attenuation_distance, model_scale
            );
            let transmission_attenuated = transmission * (vec3<f32>(1.0) - Fss_Ess);
            diffuse_ibl = mix(diffuse_ibl, transmission_attenuated, transmission_factor);
        }

        let ibl_horizon = clamp(1.0 + dot(ibl_R, N), 0.0, 1.0);
        let specular_ibl = prefiltered_color * Fss_Ess * specular_factor * ibl_horizon * ibl_horizon;

        var ambient = diffuse_ibl + specular_ibl;

        if sheen_active {
            let sheen_prefiltered = textureSampleLevel(prefiltered_env, prefiltered_sampler, R, sheen_roughness * MAX_REFLECTION_LOD).rgb;
            let sheen_ibl = sheen_color * sheen_E * sheen_prefiltered;
            ambient = ambient * sheen_albedo_scale + sheen_ibl;
        }

        if cc_factor > 0.0 {
            let cc_F_ibl = fresnel_clearcoat(cc_NdotV);
            let cc_prefiltered = textureSampleLevel(prefiltered_env, prefiltered_sampler, cc_R, cc_roughness * MAX_REFLECTION_LOD).rgb;
            let cc_horizon = clamp(1.0 + dot(cc_R, cc_normal), 0.0, 1.0);
            let cc_ibl_specular = cc_factor * (cc_F_ibl * cc_brdf.x + cc_brdf.y) * cc_prefiltered * cc_horizon * cc_horizon;
            ambient = ambient * (1.0 - cc_factor * cc_F_ibl) + cc_ibl_specular;
        }

        ambient = mix(ambient, ambient * occlusion, material.occlusion_strength);

        let color = ambient + Lo;

        final_color = color + emission;
    }

    var alpha = base_color.a;
    if transmission_factor > 0.0 {
        alpha = 1.0;
    }
    let w = weight(in.view_depth, alpha);

    var out: OITOutput;
    final_color = clamp(select(vec3<f32>(0.0), final_color, final_color == final_color), vec3<f32>(0.0), vec3<f32>(65000.0));
    out.accum = vec4<f32>(final_color, 1.0) * w;
    out.reveal = 1.0 - alpha;
    out.entity_id = bitcast<f32>(in.entity_id);

    let cur_clip = uniforms.cur_view_projection * vec4<f32>(in.world_pos, 1.0);
    let prev_clip = uniforms.prev_view_projection * vec4<f32>(in.prev_world_pos, 1.0);
    let cur_ndc = cur_clip.xy / cur_clip.w;
    let prev_ndc = prev_clip.xy / prev_clip.w;
    let cur_uv = vec2<f32>(cur_ndc.x * 0.5 + 0.5, 0.5 - cur_ndc.y * 0.5);
    let prev_uv = vec2<f32>(prev_ndc.x * 0.5 + 0.5, 0.5 - prev_ndc.y * 0.5);
    out.velocity = prev_uv - cur_uv;

    return out;
}
