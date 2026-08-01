#import nightshade_wgpu::pbr_brdf::{getRangeAttenuation, getSpotAttenuation}
#import nightshade_wgpu::mesh_common::{Uniforms, cluster_uniforms, get_cluster_index, PI, LIGHT_TYPE_POINT, LIGHT_TYPE_SPOT, COOKIE_LAYER_NONE, MAX_LIGHTS_PER_CLUSTER, Light, LightGrid, SpotlightShadowData}
#import nightshade_wgpu::material_sampling::sample_srgb_index
#import nightshade_wgpu::fog_common::{FogUniforms, FOG_X, FOG_Y, FOG_Z, fog_depth_of, fog_slice_of}
#import nightshade_wgpu::area_shape::{area_light_count, area_light_at, area_light_in_air}

struct PointLightShadowData {
    position: vec3<f32>,
    range: f32,
    bias: f32,
    shadow_index: i32,
    padding_first: f32,
    padding_second: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(3)
var<storage, read> lights: array<Light>;

@group(1) @binding(7)
var<storage, read> light_grid: array<LightGrid>;

@group(1) @binding(8)
var<storage, read> light_indices: array<u32>;

@group(3) @binding(0)
var shadow_texture: texture_depth_2d;

@group(3) @binding(1)
var shadow_compare_sampler: sampler_comparison;

@group(3) @binding(2)
var spotlight_shadow_atlas: texture_depth_2d;

@group(3) @binding(3)
var<storage, read> spotlight_shadows: array<SpotlightShadowData>;

@group(3) @binding(4)
var point_shadow_cubemap: texture_cube_array<f32>;

@group(3) @binding(5)
var point_shadow_sampler: sampler;

@group(3) @binding(6)
var<storage, read> point_light_shadows: array<PointLightShadowData>;

@group(3) @binding(7)
var irradiance_map: texture_cube<f32>;

@group(3) @binding(8)
var irradiance_map_b: texture_cube<f32>;

@group(3) @binding(9)
var irradiance_sampler: sampler;

@group(3) @binding(10)
var<uniform> fog: FogUniforms;

@group(3) @binding(11)
var fog_scatter: texture_storage_3d<rgba16float, write>;

@group(3) @binding(12)
var fog_history: texture_3d<f32>;

@group(3) @binding(13)
var fog_sampler: sampler;

const HISTORY_WEIGHT: f32 = 0.95;
const LIGHT_WIDTH: f32 = 0.5;

fn world_at(fragment: vec2<f32>, depth: f32) -> vec3<f32> {
    let ndc = vec2<f32>(
        fragment.x / fog.screen.x * 2.0 - 1.0,
        1.0 - fragment.y / fog.screen.y * 2.0
    );
    let view_position = vec3<f32>(
        ndc.x * depth / fog.focal.x,
        ndc.y * depth / fog.focal.y,
        -depth
    );
    let rotation = transpose(mat3x3<f32>(uniforms.view[0].xyz, uniforms.view[1].xyz, uniforms.view[2].xyz));
    return uniforms.camera_position.xyz + rotation * view_position;
}

fn jitter_of(frame: u32) -> vec3<f32> {
    let stride = vec3<f32>(0.8191725134, 0.6710436067, 0.5497004779);
    return fract(stride * f32(frame)) - 0.5;
}

fn select_cascade(view_depth: f32) -> i32 {
    for (var cascade_index = 0; cascade_index < 4; cascade_index = cascade_index + 1) {
        if view_depth < uniforms.cascade_split_distances[cascade_index] {
            return cascade_index;
        }
    }
    return 3;
}

fn sun_reaches(world_position: vec3<f32>, view_depth: f32) -> f32 {
    if uniforms.shadows_enabled < 0.5 {
        return 1.0;
    }
    if view_depth > uniforms.cascade_split_distances[3] {
        return 1.0;
    }
    let cascade = select_cascade(view_depth);
    let to_sun = -normalize(uniforms.directional_light_direction.xyz);
    let texel_world = uniforms.cascade_atlas_offsets[cascade].z;
    let pushed = world_position + to_sun * uniforms.shadow_bias * texel_world;
    let clip = uniforms.cascade_view_projections[cascade] * vec4<f32>(pushed, 1.0);
    let ndc = clip.xyz / clip.w;
    if ndc.z <= 0.0 || ndc.z >= 1.0 {
        return 1.0;
    }
    let offset = uniforms.cascade_atlas_offsets[cascade].xy;
    let scale = uniforms.cascade_atlas_scale.xy;
    let margin = scale * 0.01;
    let uv = clamp(
        (ndc.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5)) * scale + offset,
        offset + margin,
        offset + scale - margin
    );
    return textureSampleCompareLevel(shadow_texture, shadow_compare_sampler, uv, ndc.z);
}

fn spot_reaches(light: Light, world_position: vec3<f32>) -> f32 {
    if light.shadow_index < 0 || uniforms.shadows_enabled < 0.5 {
        return 1.0;
    }
    let shadow = spotlight_shadows[light.shadow_index];
    let clip = shadow.view_projection * vec4<f32>(world_position, 1.0);
    if clip.w <= 0.0 {
        return 1.0;
    }
    let ndc = clip.xyz / clip.w;
    if any(abs(ndc.xy) > vec2<f32>(1.0)) || ndc.z < 0.0 || ndc.z > 1.0 {
        return 1.0;
    }
    let margin = shadow.atlas_scale * 0.01;
    let uv = clamp(
        (ndc.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5)) * shadow.atlas_scale + shadow.atlas_offset,
        shadow.atlas_offset + margin,
        shadow.atlas_offset + shadow.atlas_scale - margin
    );
    return textureSampleCompareLevel(spotlight_shadow_atlas, shadow_compare_sampler, uv, ndc.z + shadow.bias);
}

fn point_reaches(light: Light, world_position: vec3<f32>) -> f32 {
    if light.shadow_index < 0 || uniforms.shadows_enabled < 0.5 {
        return 1.0;
    }
    let shadow = point_light_shadows[light.shadow_index];
    let from_light = world_position - shadow.position;
    let distance_to_light = length(from_light);
    if distance_to_light > shadow.range {
        return 1.0;
    }
    let direction = from_light / max(distance_to_light, 1.0e-4);
    let stored = textureSampleLevel(point_shadow_cubemap, point_shadow_sampler, direction, light.shadow_index, 0.0).r;
    return select(0.0, 1.0, distance_to_light / shadow.range - shadow.bias <= stored);
}

fn spot_picture(light: Light, world_position: vec3<f32>) -> vec3<f32> {
    if light.cookie_layer == COOKIE_LAYER_NONE || light.shadow_index < 0 {
        return vec3<f32>(1.0);
    }
    let clip = spotlight_shadows[light.shadow_index].view_projection * vec4<f32>(world_position, 1.0);
    if clip.w <= 0.0 {
        return vec3<f32>(0.0);
    }
    let ndc = clip.xyz / clip.w;
    let uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
    if any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) {
        return vec3<f32>(0.0);
    }
    return sample_srgb_index(light.cookie_layer & 0xFFFFu, uv).rgb;
}

fn point_picture(light: Light, world_position: vec3<f32>) -> vec3<f32> {
    if light.cookie_layer == COOKIE_LAYER_NONE {
        return vec3<f32>(1.0);
    }
    let direction = normalize(world_position - light.position.xyz);
    let uv = vec2<f32>(
        atan2(direction.z, direction.x) / (2.0 * PI) + 0.5,
        acos(clamp(direction.y, -1.0, 1.0)) / PI
    );
    return sample_srgb_index(light.cookie_layer & 0xFFFFu, uv).rgb;
}

fn phase(cosine: f32, anisotropy: f32) -> f32 {
    let squared = anisotropy * anisotropy;
    return (1.0 - squared)
        / (4.0 * PI * pow(max(1.0 + squared - 2.0 * anisotropy * cosine, 1.0e-4), 1.5));
}

fn arriving_everywhere() -> vec3<f32> {
    var ways = array<vec3<f32>, 6>(
        vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(-1.0, 0.0, 0.0),
        vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(0.0, -1.0, 0.0),
        vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(0.0, 0.0, -1.0)
    );
    var total = vec3<f32>(0.0);
    for (var way = 0; way < 6; way = way + 1) {
        let first = textureSampleLevel(irradiance_map, irradiance_sampler, ways[way], 0.0).rgb;
        let second = textureSampleLevel(irradiance_map_b, irradiance_sampler, ways[way], 0.0).rgb;
        total = total + mix(first, second, uniforms.ibl_blend_factor);
    }
    return clamp(total / 6.0, vec3<f32>(0.0), vec3<f32>(65000.0));
}

struct History {
    found: bool,
    value: vec4<f32>,
};

fn found_before(world_center: vec3<f32>) -> History {
    var history: History;
    history.found = false;
    history.value = vec4<f32>(0.0);
    if fog.frame == 0u {
        return history;
    }
    let previous = uniforms.prev_view_projection * vec4<f32>(world_center, 1.0);
    if previous.w <= 0.0 {
        return history;
    }
    let ndc = previous.xyz / previous.w;
    let uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
    let slice = fog_slice_of(previous.w, fog.far) / f32(FOG_Z);
    if any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) || slice <= 0.0 || slice >= 1.0 {
        return history;
    }
    history.found = true;
    history.value = textureSampleLevel(fog_history, fog_sampler, vec3<f32>(uv, slice), 0.0);
    return history;
}

@compute @workgroup_size(8, 8, 4)
fn main(@builtin(global_invocation_id) cell: vec3<u32>) {
    if cell.x >= FOG_X || cell.y >= FOG_Y || cell.z >= FOG_Z {
        return;
    }
    let center = vec3<f32>(cell) + 0.5;
    let place = center + jitter_of(fog.frame);
    let fragment = place.xy / vec2<f32>(f32(FOG_X), f32(FOG_Y)) * fog.screen;
    let depth = fog_depth_of(place.z, fog.far);
    let world_position = world_at(fragment, depth);

    var density = fog.air.x;
    if density <= 0.0 {
        density = 3.0 / max(uniforms.fog_end - uniforms.fog_start, 0.001);
    }
    let above = max(world_position.y - fog.air.w, 0.0);
    density = density * exp(-above * fog.air.z);
    let starts = uniforms.fog_start;
    var extinction = density;
    if starts > 0.0 {
        extinction = extinction * smoothstep(starts * 0.8, starts, depth);
    }
    let anisotropy = fog.air.y;
    let to_view = normalize(uniforms.camera_position.xyz - world_position);

    var gathered = arriving_everywhere();
    for (var index = 0u; index < cluster_uniforms.num_directional_lights; index = index + 1u) {
        let light = lights[index];
        let to_sun = normalize(-light.direction.xyz);
        var reaching = 1.0;
        if index == 0u {
            reaching = sun_reaches(world_position, depth);
        }
        gathered = gathered + light.color.rgb
            * reaching
            * phase(-dot(to_sun, to_view), anisotropy);
    }
    let cluster_index = get_cluster_index(fragment, depth);
    let grid = light_grid[cluster_index];
    let base_offset = cluster_index * MAX_LIGHTS_PER_CLUSTER;
    for (var slot = 0u; slot < min(grid.count, MAX_LIGHTS_PER_CLUSTER); slot = slot + 1u) {
        let light = lights[cluster_uniforms.num_directional_lights + light_indices[base_offset + slot]];
        let to_light_vector = light.position.xyz - world_position;
        let distance_to_light = length(to_light_vector);
        let to_light = to_light_vector / max(distance_to_light, 1.0e-4);
        var radiance = light.color.rgb
            * getRangeAttenuation(light.range, sqrt(distance_to_light * distance_to_light + LIGHT_WIDTH * LIGHT_WIDTH));
        if light.light_type == LIGHT_TYPE_SPOT {
            radiance = radiance
                * getSpotAttenuation(to_light_vector, light.direction.xyz, light.outer_cone, light.inner_cone)
                * spot_picture(light, world_position)
                * spot_reaches(light, world_position);
        } else if light.light_type == LIGHT_TYPE_POINT {
            radiance = radiance
                * point_picture(light, world_position)
                * point_reaches(light, world_position);
        }
        gathered = gathered + radiance * phase(-dot(to_light, to_view), anisotropy);
    }

    for (var index = 0u; index < area_light_count(); index = index + 1u) {
        let arriving = area_light_in_air(area_light_at(index), world_position);
        gathered = gathered
            + arriving.radiance * phase(-dot(arriving.to_light, to_view), anisotropy);
    }

    var found = vec4<f32>(uniforms.fog_color * gathered * extinction, extinction);
    let world_center = world_at(center.xy / vec2<f32>(f32(FOG_X), f32(FOG_Y)) * fog.screen, fog_depth_of(center.z, fog.far));
    let history = found_before(world_center);
    if history.found {
        found = mix(found, history.value, HISTORY_WEIGHT);
    }
    textureStore(fog_scatter, vec3<i32>(cell), found);
}
