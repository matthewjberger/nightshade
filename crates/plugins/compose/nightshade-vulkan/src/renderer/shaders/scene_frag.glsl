#extension GL_GOOGLE_include_directive : require
#extension GL_EXT_nonuniform_qualifier : require
#extension GL_EXT_ray_query : require
#extension GL_EXT_mesh_shader : require

#include "scene_common.glsl"
#include "scene_rays.glsl"
#include "scene_probes.glsl"

layout(location = 0) in VertexOutput {
    vec3 normal;
    vec3 world_position;
    vec3 previous_world_position;
    vec2 tex_coords;
    vec2 tex_coords_1;
    vec4 tangent;
    vec4 color;
} vertex_input;

layout(push_constant) uniform ListPush {
    uint list;
    uint phase;
} push;

#ifdef RUN_PATH
layout(location = 7) in flat uint material_index;
layout(location = 8) in flat uint object_index;
#else
layout(location = 7) perprimitiveEXT in flat uint material_index;
layout(location = 8) perprimitiveEXT in flat uint object_index;
#endif
layout(location = 9) noperspective in vec2 tex_coords_affine;

layout(set = 3, binding = 0) uniform samplerCube environment_irradiance;
layout(set = 3, binding = 1) uniform samplerCube environment_reflection;
layout(set = 3, binding = 5) uniform sampler2D surface_table;
layout(set = 3, binding = 6) uniform samplerCube environment_irradiance_other;
layout(set = 3, binding = 7) uniform samplerCube environment_reflection_other;
layout(set = 2, binding = 0) uniform sampler2DShadow shadow_atlas;
layout(set = 2, binding = 1) uniform sampler2DShadow local_shadow_atlas;
layout(set = 2, binding = 2) uniform sampler2D shadow_depths;
layout(set = 2, binding = 3) uniform sampler2D local_shadow_depths;

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec4 out_normal;
layout(location = 2) out vec2 out_motion;

const float PI = 3.14159265359;
const float DIELECTRIC_REFLECTANCE = 0.04;

vec3 surface_reflectance = vec3(DIELECTRIC_REFLECTANCE);
float grain_strength = 0.0;
vec3 grain_tangent = vec3(1.0, 0.0, 0.0);
vec3 grain_bitangent = vec3(0.0, 1.0, 0.0);
float coat_strength = 0.0;
float coat_roughness = 0.0;
vec3 coat_normal = vec3(0.0, 1.0, 0.0);
vec3 sheen_tint = vec3(0.0);
float sheen_roughness = 0.0;

vec3 gathered_around = vec3(0.0);
float shine_around = 0.0;

vec2 surface_response = vec2(0.0);
float sheen_response = 0.0;

float passes_light_through = 0.0;

float scatters_through = 0.0;
vec3 scatter_tint = vec3(1.0);

float boxed_reflection = 0.0;
vec3 reflection_lowest = vec3(0.0);
vec3 reflection_highest = vec3(0.0);

const vec2 SHADOW_DISC[16] = vec2[16](
    vec2(-0.94201624, -0.39906216), vec2(0.94558609, -0.76890725),
    vec2(-0.09418410, -0.92938870), vec2(0.34495938, 0.29387760),
    vec2(-0.91588581, 0.45771432), vec2(-0.81544232, -0.87912464),
    vec2(-0.38277543, 0.27676845), vec2(0.97484398, 0.75648379),
    vec2(0.44323325, -0.97511554), vec2(0.53742981, -0.47373420),
    vec2(-0.26496911, -0.41893023), vec2(0.79197514, 0.19090188),
    vec2(-0.24188840, 0.99706507), vec2(-0.81409955, 0.91437590),
    vec2(0.19984126, 0.78641367), vec2(0.14383161, -0.14100790)
);

const vec2 SHADOW_RING[8] = vec2[8](
    vec2(-0.7071, -0.7071), vec2(0.7071, -0.7071),
    vec2(-0.7071, 0.7071), vec2(0.7071, 0.7071),
    vec2(-1.0, 0.0), vec2(1.0, 0.0),
    vec2(0.0, -1.0), vec2(0.0, 1.0)
);

const vec2 SHADOW_CROSS[4] = vec2[4](
    vec2(-0.5, -0.5), vec2(0.5, -0.5),
    vec2(-0.5, 0.5), vec2(0.5, 0.5)
);

const float SHADOW_TAPS_NEAR = 30.0;
const float SHADOW_TAPS_FAR = 80.0;

uint shadow_taps(float view_depth) {
    if (view_depth < SHADOW_TAPS_NEAR) {
        return 16u;
    }
    if (view_depth < SHADOW_TAPS_FAR) {
        return 8u;
    }
    return 4u;
}

const float LOCAL_SEARCH_REACH = 15.0;
const float LOCAL_SPREAD_REACH = 30.0;

vec2 turned(vec2 offset, vec2 turn) {
    return vec2(
        turn.x * offset.x - turn.y * offset.y,
        turn.y * offset.x + turn.x * offset.y
    );
}

vec2 filter_turn(vec2 fragment) {
    float noise = fract(52.9829189 * fract(dot(fragment, vec2(0.06711056, 0.00583715))));
    noise = fract(noise + frame.settings.x * frame.settings.w * 0.6180339887);
    float angle = noise * 6.2831853;
    return vec2(cos(angle), sin(angle));
}

struct Blockers {
    float depth;
    uint found;
};

Blockers blockers_of(
    sampler2D atlas,
    vec2 at,
    float depth,
    vec2 texel,
    float reach,
    vec2 lowest,
    vec2 highest,
    vec2 turn,
    float climb
) {
    float total = 0.0;
    uint found = 0u;
    for (int tap = 0; tap < 8; ++tap) {
        vec2 point = clamp(at + turned(SHADOW_RING[tap], turn) * texel * reach, lowest, highest);
        float written = texture(atlas, point).r;
        if (written > depth + climb) {
            total += written;
            found += 1u;
        }
    }
    Blockers result;
    result.found = found;
    result.depth = found > 0u ? total / float(found) : 1.0;
    return result;
}

float spread_of(float blocker, float depth, float size, float reach, float least, float most) {
    float ratio = (blocker - depth) / max(1.0 - blocker, 0.001);
    return clamp(ratio * size * reach, least, most);
}

float dither_noise(vec2 fragment, float seconds, uint object) {
    vec2 jitter = vec2(fract(seconds * 91.7), fract(seconds * 47.3)) * 64.0;
    vec2 point = fragment + jitter + float(object & 0xFFu);
    return fract(52.9829189 * fract(dot(point, vec2(0.06711056, 0.00583715))));
}

vec2 slot_uv(GpuMaterial material, int slot) {
    vec2 first = frame.style.w != 0.0 ? tex_coords_affine : vertex_input.tex_coords;
    return slot_uv_of(material, slot, first, vertex_input.tex_coords_1);
}

vec4 sample_slot(uint slot, vec2 uv) {
    uint index = slot & 0xFFFFu;
    uint wrap_u = (slot >> 16u) & 3u;
    uint wrap_v = (slot >> 18u) & 3u;
    uint nearest = (slot >> 20u) & 1u;
    uint sampler_slot = nearest * 9u + wrap_u * 3u + wrap_v;
    float level_bias = frame.debug.w != 0.0 ? 0.0 : -32.0;
    return texture(
        sampler2D(scene_textures[nonuniformEXT(index)], scene_samplers[sampler_slot]),
        uv,
        level_bias
    );
}

vec4 picture_sample(uint slot, vec2 uv) {
    return sample_slot(slot, uv);
}

#include "scene_local_lights.glsl"
#include "area_light.glsl"

float distribution_ggx(float normal_half, float roughness) {
    float alpha = roughness * roughness;
    float alpha_squared = alpha * alpha;
    float denominator = normal_half * normal_half * (alpha_squared - 1.0) + 1.0;
    return alpha_squared / max(PI * denominator * denominator, 1.0e-5);
}

vec3 fresnel_schlick(float cosine, vec3 reflectance) {
    return reflectance + (1.0 - reflectance) * pow(clamp(1.0 - cosine, 0.0, 1.0), 5.0);
}

#include "material_common.glsl"

vec3 fresnel_schlick_roughness(float cosine, vec3 reflectance, float roughness) {
    vec3 ceiling = max(vec3(1.0 - roughness), reflectance);
    return reflectance + (ceiling - reflectance) * pow(clamp(1.0 - cosine, 0.0, 1.0), 5.0);
}

vec3 arriving_light(vec3 direction) {
    return mix(
        texture(environment_irradiance, direction).rgb,
        texture(environment_irradiance_other, direction).rgb,
        frame.environment.x
    );
}

vec3 sky_along(vec3 direction, float level) {
    return mix(
        textureLod(environment_reflection, direction, level).rgb,
        textureLod(environment_reflection_other, direction, level).rgb,
        frame.environment.x
    );
}

vec2 environment_brdf(float normal_view, float roughness) {
    return texture(surface_table, vec2(clamp(normal_view, 0.0, 1.0), clamp(roughness, 0.0, 1.0))).rg;
}

float sheen_albedo(float normal_view, float roughness) {
    return texture(surface_table, vec2(clamp(normal_view, 0.0, 1.0), clamp(roughness, 0.0, 1.0))).b;
}

float luminance_of(vec3 color) {
    return dot(color, vec3(0.2126, 0.7152, 0.0722));
}

vec3 image_based_light(
    vec3 normal,
    vec3 to_view,
    vec3 albedo,
    float metallic,
    float roughness,
    float occlusion
) {
    float normal_view = max(dot(normal, to_view), 0.0);
    vec3 fresnel = fresnel_schlick_roughness(normal_view, surface_reflectance, roughness);
    vec2 response = surface_response;

    vec3 single = fresnel * response.x + response.y;
    float missed = 1.0 - (response.x + response.y);
    vec3 average = surface_reflectance + (vec3(1.0) - surface_reflectance) / 21.0;
    vec3 further = missed * single * average / (vec3(1.0) - average * missed);

    vec3 body = albedo * (1.0 - metallic);
    vec3 kept = body * (vec3(1.0) - single - further);
    vec3 around = arriving_light(normal)
        + probe_arriving_light(vertex_input.world_position, normal, to_view);
    vec3 diffuse = (further + kept) * around;
    if (scatters_through > 0.0) {
        vec3 far_body = scatter_tint * albedo * (1.0 - metallic);
        vec3 far_kept = far_body * (vec3(1.0) - single - further);
        vec3 far_side = (further + far_kept)
            * (arriving_light(-normal)
                + probe_arriving_light(vertex_input.world_position, -normal, to_view));
        diffuse = mix(diffuse, far_side, scatters_through);
    }

    float levels = float(textureQueryLevels(environment_reflection) - 1);
    vec3 facing_reflection = normal;
    if (grain_strength > 0.0) {
        vec3 along_grain = cross(grain_bitangent, to_view);
        vec3 grain_normal = cross(along_grain, grain_bitangent);
        float bend = 1.0 - grain_strength * (1.0 - roughness);
        float squared = bend * bend;
        facing_reflection = normalize(mix(grain_normal, normal, squared * squared));
    }
    vec3 reflected = reflect(-to_view, facing_reflection);
    if (boxed_reflection != 0.0) {
        reflected = reflection_in_a_box(
            reflected,
            vertex_input.world_position,
            reflection_lowest,
            reflection_highest
        );
    }
    vec3 radiance = sky_along(reflected, roughness * levels);
    float horizon = clamp(1.0 + dot(reflected, normal), 0.0, 1.0);
    vec3 specular = radiance * single * horizon * horizon;

    float hides_diffuse = occlusion;
    float hides_reflection = specular_occlusion(normal_view, occlusion, roughness);

    vec3 gathered = diffuse * hides_diffuse + specular * hides_reflection;
    shine_around = luminance_of(specular * hides_reflection);

    if (sheen_tint != vec3(0.0)) {
        float highest = max(sheen_tint.r, max(sheen_tint.g, sheen_tint.b));
        vec3 sheen_radiance = sky_along(reflected, sheen_roughness * levels);
        vec3 sheen_light = sheen_tint * sheen_response * sheen_radiance * hides_reflection;
        gathered = gathered * clamp(1.0 - highest * sheen_response, 0.0, 1.0) + sheen_light;
        shine_around += luminance_of(sheen_light);
    }

    if (coat_strength > 0.0) {
        float coat_view = max(dot(coat_normal, to_view), 0.0);
        float coat_fresnel = 0.04 + 0.96 * pow(1.0 - coat_view, 5.0);
        float coat_taken = coat_strength * coat_fresnel;
        vec3 coat_reflected = reflect(-to_view, coat_normal);
        vec2 coat_response = environment_brdf(coat_view, coat_roughness);
        vec3 coat_radiance = sky_along(coat_reflected, coat_roughness * levels);
        float coat_horizon = clamp(1.0 + dot(coat_reflected, coat_normal), 0.0, 1.0);
        vec3 coat_light = coat_strength
            * (coat_fresnel * coat_response.x + coat_response.y)
            * coat_radiance
            * coat_horizon
            * coat_horizon;
        gathered = gathered * (1.0 - coat_taken) + coat_light * hides_reflection;
        shine_around += luminance_of(coat_light * hides_reflection);
    }

    gathered_around = diffuse * hides_diffuse;
    return gathered;
}

float surface_reflectivity(
    vec3 normal,
    vec3 to_view,
    vec3 albedo,
    float metallic,
    float roughness
) {
    float normal_view = clamp(dot(normal, to_view), 0.0, 1.0);
    vec3 reflectance = mix(vec3(DIELECTRIC_REFLECTANCE), albedo, metallic);
    vec3 fresnel = reflectance + (vec3(1.0) - reflectance) * pow(1.0 - normal_view, 5.0);
    float strength = luminance_of(fresnel);
    return clamp(strength * (1.0 - roughness), 0.0, 1.0);
}

float debug_sun = 1.0;

vec3 debug_fresnel = vec3(0.0);
float debug_geometry = 0.0;
float debug_distribution = 0.0;
vec3 debug_diffuse = vec3(0.0);
vec3 debug_specular = vec3(0.0);

vec3 shade(
    vec3 normal,
    vec3 to_view,
    vec3 to_light,
    vec3 radiance,
    vec3 albedo,
    float metallic,
    float roughness
) {
    float normal_light = max(dot(normal, to_light), 0.0);
    if (normal_light <= 0.0 && scatters_through <= 0.0) {
        return vec3(0.0);
    }
    vec3 half_vector = normalize(to_view + to_light);
    float normal_view = max(dot(normal, to_view), 1.0e-4);
    float normal_half = max(dot(normal, half_vector), 0.0);
    float view_half = max(dot(to_view, half_vector), 0.0);

    vec3 fresnel = fresnel_schlick(view_half, surface_reflectance);
    float distribution;
    float visibility;
    if (grain_strength > 0.0) {
        float alpha = max(roughness * roughness, 1.0e-4);
        float along = mix(alpha, 1.0, grain_strength * grain_strength);
        float across = alpha;
        distribution = distribution_anisotropic(
            normal_half,
            dot(grain_tangent, half_vector),
            dot(grain_bitangent, half_vector),
            along,
            across
        );
        visibility = visibility_anisotropic(
            normal_light,
            normal_view,
            dot(grain_bitangent, to_view),
            dot(grain_tangent, to_view),
            dot(grain_bitangent, to_light),
            dot(grain_tangent, to_light),
            along,
            across
        );
    } else {
        distribution = distribution_ggx(normal_half, roughness);
        visibility = visibility_smith(normal_view, normal_light, roughness);
    }
    float geometry = visibility * 4.0 * normal_view * normal_light;

    vec3 specular = fresnel * distribution * visibility;
    vec3 diffuse = (vec3(1.0) - fresnel) * (1.0 - metallic) * albedo / PI;

    debug_fresnel = fresnel;
    debug_geometry = geometry;
    debug_distribution = distribution;
    debug_diffuse += diffuse * normal_light * radiance;
    debug_specular += specular * normal_light * radiance;

    vec3 given_back = vec3(1.0)
        + surface_reflectance
            * (1.0 / max(surface_response.x + surface_response.y, 0.001) - 1.0);

    vec3 scattered_back = diffuse * normal_light * (1.0 - passes_light_through);
    if (scatters_through > 0.0) {
        float far_side = max(dot(-normal, to_light), 0.0);
        vec3 through_side = (vec3(1.0) - fresnel)
            * (1.0 - metallic)
            * scatter_tint
            * albedo
            / PI
            * far_side;
        scattered_back = mix(scattered_back, through_side, scatters_through);
    }

    vec3 lit = (scattered_back + specular * given_back * normal_light) * radiance;

    if (sheen_tint != vec3(0.0)) {
        float highest = max(sheen_tint.r, max(sheen_tint.g, sheen_tint.b));
        lit *= clamp(1.0 - highest * sheen_response, 0.0, 1.0);
        lit += sheen_lobe(normal, to_view, to_light, sheen_tint, sheen_roughness) * radiance;
    }
    if (coat_strength > 0.0) {
        float coat_half = max(dot(to_view, half_vector), 0.0);
        float taken = coat_strength * (0.04 + 0.96 * pow(1.0 - coat_half, 5.0));
        lit *= 1.0 - taken;
        lit += clearcoat_lobe(coat_normal, to_view, to_light, coat_roughness)
            * coat_strength
            * radiance;
    }
    return lit;
}

vec3 shading_normal(vec3 geometric, vec4 tangent, uint slot, vec2 uv, float scale, uint flags) {
    if (slot == NO_TEXTURE) {
        return geometric;
    }
    vec3 tangent_direction = tangent.xyz;
    if (dot(tangent_direction, tangent_direction) < 1.0e-6) {
        return geometric;
    }
    tangent_direction = normalize(tangent_direction - geometric * dot(geometric, tangent_direction));
    vec3 bitangent = cross(geometric, tangent_direction) * tangent.w;

    vec3 sampled = sample_slot(slot, uv).xyz;
    vec2 flat_part = sampled.xy * 2.0 - 1.0;
    if ((flags & MATERIAL_NORMAL_FLIP_Y) != 0u) {
        flat_part.y = -flat_part.y;
    }
    float depth_part = (flags & MATERIAL_NORMAL_TWO_COMPONENT) != 0u
        ? sqrt(max(1.0 - dot(flat_part, flat_part), 0.0))
        : sampled.z * 2.0 - 1.0;

    vec3 mapped = vec3(flat_part * scale, depth_part);
    return normalize(mat3(tangent_direction, bitangent, geometric) * mapped);
}

vec2 screen_movement() {
    vec4 now = frame.view_projection * vec4(vertex_input.world_position, 1.0);
    vec4 was = frame.previous_view_projection
        * vec4(vertex_input.previous_world_position, 1.0);
    if (now.w <= 0.0 || was.w <= 0.0) {
        return vec2(0.0);
    }
    vec2 now_at = vec2(now.x / now.w * 0.5 + 0.5, 0.5 - now.y / now.w * 0.5);
    vec2 was_at = vec2(was.x / was.w * 0.5 + 0.5, 0.5 - was.y / was.w * 0.5);
    return now_at - was_at;
}

vec3 behind_body(float bending, vec3 normal, vec3 to_view, float thickness, float roughness) {
    vec3 bent = refract(-to_view, normal, 1.0 / max(bending, 1.0e-4));

    float scatter = roughness * clamp(bending * 2.0 - 2.0, 0.0, 1.0);
    float around_levels = float(textureQueryLevels(environment_reflection) - 1);
    vec3 around = sky_along(bent, scatter * around_levels);

    vec4 behind_clip = frame.view_projection
        * vec4(vertex_input.world_position + bent * thickness, 1.0);
    if (behind_clip.w <= 1.0e-4) {
        return around;
    }
    vec2 at = behind_clip.xy / behind_clip.w * 0.5 + 0.5;
    at.y = 1.0 - at.y;
    float grab_levels = float(textureQueryLevels(scene_grab) - 1);
    vec3 drawn = textureLod(
        scene_grab,
        clamp(at, vec2(0.0), vec2(1.0)),
        scatter * grab_levels
    ).rgb;
    float from_edge = min(min(at.x, at.y), min(1.0 - at.x, 1.0 - at.y));
    return mix(around, drawn, clamp(from_edge * 8.0, 0.0, 1.0));
}

int select_cascade(float view_depth) {
    for (int cascade = 0; cascade < CASCADE_COUNT; ++cascade) {
        if (view_depth < frame.cascade_splits[cascade]) {
            return cascade;
        }
    }
    return CASCADE_COUNT - 1;
}

#define SUN_RAYS 8

const float SUN_DISC = 0.00465;

float traced_sun_visibility(vec3 world_position, vec3 normal, vec3 to_sun) {
    float step_off = max(
        length(frame.camera_position.xyz - world_position) * frame.sun_shadow.y,
        1.0e-5
    );
    vec3 from = world_position + normal * step_off;
    float spread = frame.sun_shadow.x * SUN_DISC;
    if (spread <= 0.0) {
        return sun_is_blocked(from, to_sun, step_off) ? 0.0 : 1.0;
    }

    vec3 across = normalize(
        abs(to_sun.y) < 0.99
            ? cross(to_sun, vec3(0.0, 1.0, 0.0))
            : cross(to_sun, vec3(1.0, 0.0, 0.0))
    );
    vec3 up = cross(to_sun, across);

    vec2 turn = filter_turn(gl_FragCoord.xy);
    float reaching = 0.0;
    for (int index = 0; index < SUN_RAYS; ++index) {
        vec2 place = SHADOW_DISC[index];
        vec2 turned = vec2(
            place.x * turn.x - place.y * turn.y,
            place.x * turn.y + place.y * turn.x
        ) * spread;
        vec3 toward = normalize(to_sun + across * turned.x + up * turned.y);
        reaching += sun_is_blocked(from, toward, step_off) ? 0.0 : 1.0;
    }
    return reaching / float(SUN_RAYS);
}

float cascade_visibility(
    int cascade,
    vec3 world_position,
    vec3 normal,
    vec3 to_sun,
    float view_depth
) {
    float texel_world = frame.cascade_texel_sizes[cascade];
    float size = frame.sun_shadow.x;
    float cosine = clamp(dot(normal, to_sun), 0.0, 1.0);
    float slope = sqrt(max(1.0 - cosine * cosine, 0.0));
    float search_texels = clamp(size * 2.0, 1.0, 4.0);
    vec3 offset_position = world_position
        + normal * frame.shadow_settings.y * texel_world * max(slope, 0.35) * search_texels
        + to_sun * frame.shadow_settings.x * texel_world;

    vec4 light_position = frame.cascade_view_projections[cascade] * vec4(offset_position, 1.0);
    vec3 light_ndc = light_position.xyz / light_position.w;
    if (light_ndc.z <= 0.0 || light_ndc.z >= 1.0) {
        return 1.0;
    }

    vec2 slot = vec2(float(cascade % 2), float(cascade / 2)) * 0.5;
    vec2 atlas_uv = (light_ndc.xy * vec2(0.5, 0.5) + vec2(0.5)) * 0.5 + slot;
    vec2 texel = vec2(1.0 / frame.shadow_settings.w);
    vec2 lowest = slot + texel;
    vec2 highest = slot + vec2(0.5) - texel;
    vec2 turn = filter_turn(gl_FragCoord.xy);

    float leaning = slope / max(cosine, 0.15);
    vec3 depth_row = vec3(
        frame.cascade_view_projections[cascade][0][2],
        frame.cascade_view_projections[cascade][1][2],
        frame.cascade_view_projections[cascade][2][2]
    );
    float per_texel = texel_world * leaning * length(depth_row);

    Blockers found = blockers_of(
        shadow_depths,
        atlas_uv,
        light_ndc.z,
        texel,
        search_texels * 2.0,
        lowest,
        highest,
        turn,
        per_texel * search_texels
    );
    if (found.found == 0u) {
        return 1.0;
    }
    float gap = max(found.depth - light_ndc.z, 0.0) / max(length(depth_row), 1.0e-6);
    float spread = clamp(gap * size * SUN_DISC / max(texel_world, 1.0e-6), 1.0, 8.0);

    float reference = light_ndc.z + per_texel * spread;
    float visibility = 0.0;
    uint taps = shadow_taps(view_depth);
    if (taps == 16u) {
        for (int tap = 0; tap < 16; ++tap) {
            vec2 uv = clamp(
                atlas_uv + turned(SHADOW_DISC[tap], turn) * texel * spread,
                lowest,
                highest
            );
            visibility += texture(shadow_atlas, vec3(uv, reference));
        }
        return visibility / 16.0;
    }
    if (taps == 8u) {
        for (int tap = 0; tap < 8; ++tap) {
            vec2 uv = clamp(
                atlas_uv + turned(SHADOW_RING[tap], turn) * texel * spread,
                lowest,
                highest
            );
            visibility += texture(shadow_atlas, vec3(uv, reference));
        }
        return visibility / 8.0;
    }
    for (int tap = 0; tap < 4; ++tap) {
        vec2 uv = clamp(
            atlas_uv + turned(SHADOW_CROSS[tap], turn) * texel * spread,
            lowest,
            highest
        );
        visibility += texture(shadow_atlas, vec3(uv, reference));
    }
    return visibility / 4.0;
}

float sun_visibility(vec3 world_position, vec3 normal, vec3 to_sun) {
    if (frame.shadow_settings.z == 0.0) {
        return 1.0;
    }
    if (frame.sun_shadow.w > 0.0) {
        return traced_sun_visibility(world_position, normal, to_sun);
    }
    float depth = -(frame.view * vec4(world_position, 1.0)).z;
    float shadow_far = frame.cascade_splits[CASCADE_COUNT - 1];
    if (depth > shadow_far) {
        return 1.0;
    }
    int cascade = select_cascade(depth);
    float visibility = cascade_visibility(cascade, world_position, normal, to_sun, depth);
    if (cascade < CASCADE_COUNT - 1) {
        float ends = frame.cascade_splits[cascade];
        float mixing = ends * 0.25;
        float starts = ends - mixing;
        if (depth > starts) {
            float next =
                cascade_visibility(cascade + 1, world_position, normal, to_sun, depth);
            visibility = mix(visibility, next, (depth - starts) / mixing);
        }
    }
    float fade = smoothstep(shadow_far, shadow_far * 0.9, depth);
    return mix(1.0, visibility, fade);
}

vec3 with_fog(vec3 color, vec3 world_position) {
    uint curve = uint(frame.fog_color.w);
    if (curve == 0u) {
        return color;
    }
    if (frame.fog.z > 0.0) {
        float view_depth = -(frame.view * vec4(world_position, 1.0)).z;
        vec3 at = vec3(
            gl_FragCoord.xy / frame.screen.xy,
            (fog_slice_of(view_depth) - 0.5 + fog_slice_dither(gl_FragCoord.xy)) / float(FOG_Z)
        );
        vec4 air = texture(fog_volume, at);
        return color * air.a + air.rgb;
    }
    float depth = length(frame.camera_position.xyz - world_position);
    float start = frame.fog.x;
    float end = frame.fog.y;
    float amount;
    if (curve == 1u) {
        amount = clamp((depth - start) / max(end - start, 0.001), 0.0, 1.0);
    } else {
        float beyond = max(depth - start, 0.0) * 3.0 / max(end - start, 0.001);
        amount = curve == 2u
            ? 1.0 - exp(-beyond)
            : 1.0 - exp(-beyond * beyond);
    }
    return mix(color, frame.fog_color.rgb, amount);
}

vec3 mip_rainbow(vec2 uv, float size) {
    vec2 texel = uv * size;
    float spread = max(
        length(dFdx(texel)),
        length(dFdy(texel))
    );
    float level = clamp(log2(max(spread, 1.0e-4)), 0.0, 6.0);
    const vec3 LEVELS[7] = vec3[7](
        vec3(1.0, 0.0, 0.0),
        vec3(1.0, 0.5, 0.0),
        vec3(1.0, 1.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 1.0, 1.0),
        vec3(0.0, 0.0, 1.0),
        vec3(1.0, 0.0, 1.0)
    );
    int lower = int(floor(level));
    int upper = min(lower + 1, 6);
    return mix(LEVELS[lower], LEVELS[upper], fract(level));
}

float local_visibility(
    Light light,
    vec3 world_position,
    vec3 normal,
    vec3 to_light,
    float view_depth
) {
    LocalShadow shadow;
    vec3 settled;
    if (!local_shadow_of(light, world_position, shadow, settled)) {
        return 1.0;
    }
    vec2 across = settled.xy * 0.5 + 0.5;
    vec4 place = shadow.slot;
    vec2 at = place.xy + across * place.z;
    float slope = 1.0 - clamp(dot(normal, to_light), 0.0, 1.0);
    float step_off = place.w * (1.0 + slope * 2.0);
    float depth = settled.z + step_off;

    vec2 texel = vec2(1.0 / frame.local_shadow.x);
    vec2 step = texel * (place.z * frame.local_shadow.x / frame.local_shadow.y);
    vec2 lowest = place.xy + texel;
    vec2 highest = place.xy + vec2(place.z) - texel;
    vec2 turn = filter_turn(gl_FragCoord.xy);
    float size = max(light.light_size, 0.0);
    if (size <= 0.0) {
        return texture(local_shadow_atlas, vec3(at, depth));
    }

    Blockers found = blockers_of(
        local_shadow_depths,
        at,
        depth,
        step,
        size * LOCAL_SEARCH_REACH,
        lowest,
        highest,
        turn,
        0.0
    );
    if (found.found == 0u) {
        return 1.0;
    }
    float spread = spread_of(found.depth, depth, size, LOCAL_SPREAD_REACH, 0.5, 6.0);

    float reference = depth + step_off * spread;

    float visibility = 0.0;
    uint taps = shadow_taps(view_depth);
    if (taps == 16u) {
        for (int tap = 0; tap < 16; ++tap) {
            vec2 point = clamp(
                at + turned(SHADOW_DISC[tap], turn) * step * spread,
                lowest,
                highest
            );
            visibility += texture(local_shadow_atlas, vec3(point, reference));
        }
        return visibility / 16.0;
    }
    if (taps == 8u) {
        for (int tap = 0; tap < 8; ++tap) {
            vec2 point = clamp(
                at + turned(SHADOW_RING[tap], turn) * step * spread,
                lowest,
                highest
            );
            visibility += texture(local_shadow_atlas, vec3(point, reference));
        }
        return visibility / 8.0;
    }
    for (int tap = 0; tap < 4; ++tap) {
        vec2 point = clamp(
            at + turned(SHADOW_CROSS[tap], turn) * step * spread,
            lowest,
            highest
        );
        visibility += texture(local_shadow_atlas, vec3(point, reference));
    }
    return visibility / 4.0;
}

void main() {
    GpuMaterial material;
    if (material_index == NO_MATERIAL) {
        material.base_color = vec4(1.0);
        material.emissive = vec4(0.0);
        material.factors = vec4(0.0, 1.0, 1.0, 1.0);
        material.alpha = vec4(0.5, 0.0, 0.0, 1.5);
        material.transmission = vec4(0.0, 0.0, 0.0, 0.0);
        material.attenuation = vec4(1.0, 1.0, 1.0, 1.0);
        material.specular_color = vec4(1.0, 1.0, 1.0, 0.0);
        material.anisotropy = vec4(0.0, 0.0, 1.3, 100.0);
        material.sheen = vec4(400.0, 0.0, 0.0, 0.0);
        material.sheen_color = vec4(0.0, 0.0, 0.0, 1.0);
        material.detail = vec4(0.0, 1.0, 0.0, 0.0);
        material.diffuse_transmission_color = vec4(1.0, 1.0, 1.0, 0.0);
        material.reflection_box_min = vec4(0.0);
        material.reflection_box_max = vec4(0.0);
        material.reveal = vec4(0.0, 1.0, 0.0, REVEAL_FRONT_DISABLED);
        material.reveal_noise = vec4(1.0, 1.0, 1.0, 0.0);
        material.reveal_seam = vec4(1.0, 1.0, 1.0, 0.0);
        material.atlas = vec4(0.0);
        for (int slot = 0; slot < TEXTURE_SLOT_COUNT; ++slot) {
            material.textures[slot] = NO_TEXTURE;
        }
        for (int row = 0; row < TEXTURE_SLOT_COUNT * 2; row += 2) {
            material.transforms[row] = vec4(1.0, 0.0, 0.0, 0.0);
            material.transforms[row + 1] = vec4(0.0, 1.0, 0.0, 0.0);
        }
    } else {
        material = materials[material_index];
    }

    vec3 toward_camera = normalize(frame.camera_position.xyz - vertex_input.world_position);
    uint height_slot = material.textures[TEXTURE_HEIGHT];
    vec2 parallax = vec2(0.0);
    if (height_slot != NO_TEXTURE && material.detail.z != 0.0) {
        vec3 surface = normalize(vertex_input.normal);
        vec3 along = vertex_input.tangent.xyz;
        if (dot(along, along) > 1.0e-6) {
            along = normalize(along - surface * dot(surface, along));
            vec3 across = cross(surface, along) * vertex_input.tangent.w;
            vec2 in_tangent = vec2(dot(toward_camera, along), dot(toward_camera, across));
            float depth = sample_slot(height_slot, slot_uv(material, TEXTURE_BASE)).r;
            parallax = in_tangent * (depth - 0.5) * material.detail.z;
        }
    }

    uint surface_flags = uint(material.alpha.z);
    float vertex_occlusion = (surface_flags & MATERIAL_VERTEX_OCCLUSION) != 0u
        ? vertex_input.color.r
        : 1.0;
    float sky_access = (surface_flags & MATERIAL_SKY_LIGHT) != 0u
        ? clamp(vertex_input.tex_coords_1.y, 0.0, 1.0)
        : 1.0;
    vec4 albedo = material.base_color * surface_tint(material, vertex_input.color);
    uint base_slot = material.textures[TEXTURE_BASE];
    if (base_slot != NO_TEXTURE) {
        albedo *= sample_slot(
            base_slot,
            atlas_uv(material, slot_uv(material, TEXTURE_BASE) + parallax, vertex_input.tex_coords_1.x)
        );
    }

    float detail_scale = max(material.detail.y, 1.0e-4);
    vec2 detail_uv = (slot_uv(material, TEXTURE_BASE) + parallax) * detail_scale;
    uint detail_base_slot = material.textures[TEXTURE_DETAIL_BASE];
    if (detail_base_slot != NO_TEXTURE) {
        vec3 detail = sample_slot(detail_base_slot, detail_uv).rgb;
        albedo.rgb *= detail * 2.0;
    }

    float reveal_front = material.reveal.w;
    float seam_amount = 0.0;
    if (reveal_front < REVEAL_FRONT_DISABLED) {
        vec3 place = vertex_input.world_position * material.reveal_noise.xyz;
        float roughened = (reveal_noise_at(floor(place)) - 0.5)
            * material.reveal_noise.w;
        float along = dot(vertex_input.world_position, normalize(material.reveal.xyz))
            + roughened;
        if (along > reveal_front) {
            discard;
        }
        float width = max(material.reveal_seam.w, 1.0e-4);
        float toward_edge = clamp((along - (reveal_front - width)) / width, 0.0, 1.0);
        seam_amount = toward_edge * toward_edge * material.reflection_box_min.w;
    }

    uint flags = uint(material.alpha.z);
    uint alpha_mode = uint(material.alpha.y);
    if (alpha_mode == ALPHA_MODE_MASK) {
        if (albedo.a < material.alpha.x) {
            discard;
        }
        albedo.a = 1.0;
    } else if (alpha_mode == ALPHA_MODE_DITHER) {
        float noise = dither_noise(gl_FragCoord.xy, frame.settings.z, object_index);
        if (albedo.a < pow(noise, 2.2)) {
            discard;
        }
        albedo.a = 1.0;
    } else if (alpha_mode == ALPHA_MODE_OPAQUE) {
        albedo.a = 1.0;
    }

    vec3 geometric = normalize(vertex_input.normal);
    if ((flags & MATERIAL_DOUBLE_SIDED) != 0u && !gl_FrontFacing) {
        geometric = -geometric;
    }

    out_normal = vec4(mat3(frame.view) * geometric, 0.0);
    out_motion = screen_movement();

    vec3 geometric_for_debug = normalize(vertex_input.normal);
    if (frame.debug.y != 0.0) {
        out_color = vec4(geometric_for_debug * 0.5 + 0.5, 1.0);
        return;
    }
    if (frame.flat_color.a >= 0.0) {
        out_color = frame.flat_color;
        return;
    }
    vec3 emission = material.emissive.rgb * material.emissive.a;
    uint emissive_slot = material.textures[TEXTURE_EMISSIVE];
    if (emissive_slot != NO_TEXTURE) {
        emission *= sample_slot(emissive_slot, slot_uv(material, TEXTURE_EMISSIVE)).rgb;
    }

    if ((flags & MATERIAL_UNLIT) != 0u || frame.debug.x != 0.0) {
        out_color = vec4(
            with_fog(albedo.rgb + emission, vertex_input.world_position),
            albedo.a
        );
        return;
    }

    float metallic = material.factors.x;
    float roughness = clamp(material.factors.y, 0.04, 1.0);
    uint metallic_roughness_slot = material.textures[TEXTURE_METALLIC_ROUGHNESS];
    if (metallic_roughness_slot != NO_TEXTURE) {
        vec4 sampled = sample_slot(
            metallic_roughness_slot,
            slot_uv(material, TEXTURE_METALLIC_ROUGHNESS)
        );
        roughness = clamp(roughness * sampled.g, 0.04, 1.0);
        metallic *= sampled.b;
    }

    float specular_strength = material.attenuation.w;
    uint specular_slot = material.textures[TEXTURE_SPECULAR];
    if (specular_slot != NO_TEXTURE) {
        specular_strength *= sample_slot(specular_slot, slot_uv(material, TEXTURE_SPECULAR)).a;
    }
    vec3 specular_color = material.specular_color.rgb;
    uint specular_color_slot = material.textures[TEXTURE_SPECULAR_COLOR];
    if (specular_color_slot != NO_TEXTURE) {
        specular_color *= sample_slot(
            specular_color_slot,
            slot_uv(material, TEXTURE_SPECULAR_COLOR)
        ).rgb;
    }
    surface_reflectance = mix(
        min(
            vec3(reflectance_from_ior(material.alpha.w)) * specular_color * specular_strength,
            vec3(1.0)
        ),
        albedo.rgb,
        metallic
    );

    vec3 normal = shading_normal(
        geometric,
        vertex_input.tangent,
        material.textures[TEXTURE_NORMAL],
        slot_uv(material, TEXTURE_NORMAL) + parallax,
        material.factors.z,
        flags
    );
    roughness = roughness_over_a_pixel(roughness, normal);
    uint detail_normal_slot = material.textures[TEXTURE_DETAIL_NORMAL];
    if (detail_normal_slot != NO_TEXTURE) {
        vec3 detail = shading_normal(
            normal,
            vertex_input.tangent,
            detail_normal_slot,
            detail_uv,
            1.0,
            flags
        );
        normal = normalize(normal + detail);
    }

    float grain = material.anisotropy.x;
    float grain_turn = material.specular_color.w;
    uint grain_slot = material.textures[TEXTURE_ANISOTROPY];
    if (grain_slot != NO_TEXTURE) {
        vec3 sampled = sample_slot(grain_slot, slot_uv(material, TEXTURE_ANISOTROPY)).rgb;
        vec2 direction = sampled.xy * 2.0 - 1.0;
        grain *= sampled.z;
        grain_turn += atan(direction.y, direction.x);
    }
    if (grain > 0.0) {
        vec3 along = vertex_input.tangent.xyz;
        if (dot(along, along) > 1.0e-6) {
            along = normalize(along - normal * dot(normal, along));
            vec3 across = cross(normal, along) * vertex_input.tangent.w;
            float turn_cos = cos(grain_turn);
            float turn_sin = sin(grain_turn);
            grain_tangent = along * turn_cos + across * turn_sin;
            grain_bitangent = across * turn_cos - along * turn_sin;
            grain_strength = clamp(grain, 0.0, 1.0);
        }
    }

    float film_strength = clamp(material.anisotropy.y, 0.0, 1.0);
    float film_thickness = material.sheen.x;
    uint film_slot = material.textures[TEXTURE_IRIDESCENCE];
    if (film_slot != NO_TEXTURE) {
        vec2 sampled = sample_slot(film_slot, slot_uv(material, TEXTURE_IRIDESCENCE)).rg;
        film_strength *= sampled.r;
        film_thickness = mix(material.anisotropy.w, material.sheen.x, sampled.g);
    }
    if (film_strength > 0.0 && film_thickness > 0.0) {
        float facing_film = max(dot(normal, toward_camera), 0.0);
        vec3 reflected_by_film = thin_film(
            facing_film,
            film_thickness,
            material.anisotropy.z,
            surface_reflectance
        );
        vec3 shifted = reflectance_from_fresnel(
            reflected_by_film,
            vec3(1.0),
            facing_film
        );
        surface_reflectance = mix(surface_reflectance, shifted, film_strength);
    }

    coat_strength = material.sheen.z;
    uint coat_slot = material.textures[TEXTURE_CLEARCOAT];
    if (coat_slot != NO_TEXTURE) {
        coat_strength *= sample_slot(coat_slot, slot_uv(material, TEXTURE_CLEARCOAT)).r;
    }
    if (coat_strength > 0.0) {
        coat_roughness = clamp(material.sheen.w, 0.04, 1.0);
        uint coat_roughness_slot = material.textures[TEXTURE_CLEARCOAT_ROUGHNESS];
        if (coat_roughness_slot != NO_TEXTURE) {
            coat_roughness = clamp(
                coat_roughness
                    * sample_slot(
                        coat_roughness_slot,
                        slot_uv(material, TEXTURE_CLEARCOAT_ROUGHNESS)
                    ).g,
                0.04,
                1.0
            );
        }
        coat_normal = shading_normal(
            geometric,
            vertex_input.tangent,
            material.textures[TEXTURE_CLEARCOAT_NORMAL],
            slot_uv(material, TEXTURE_CLEARCOAT_NORMAL),
            material.sheen_color.w,
            flags
        );
    }

    sheen_tint = material.sheen_color.rgb;
    sheen_roughness = material.sheen.y;
    uint sheen_slot = material.textures[TEXTURE_SHEEN_COLOR];
    if (sheen_slot != NO_TEXTURE) {
        vec4 sampled = sample_slot(sheen_slot, slot_uv(material, TEXTURE_SHEEN_COLOR));
        sheen_tint *= sampled.rgb;
        sheen_roughness *= sampled.a;
    }
    sheen_roughness = clamp(sheen_roughness, 0.07, 1.0);

    passes_light_through = material.transmission.x;
    uint transmission_slot = material.textures[TEXTURE_TRANSMISSION];
    if (transmission_slot != NO_TEXTURE) {
        passes_light_through *= sample_slot(
            transmission_slot,
            slot_uv(material, TEXTURE_TRANSMISSION)
        ).r;
    }

    scatters_through = material.detail.x;
    scatter_tint = material.diffuse_transmission_color.rgb;
    uint scattered_slot = material.textures[TEXTURE_DIFFUSE_TRANSMISSION_COLOR];
    if (scattered_slot != NO_TEXTURE) {
        vec4 sampled = sample_slot(
            scattered_slot,
            slot_uv(material, TEXTURE_DIFFUSE_TRANSMISSION_COLOR)
        );
        scatter_tint *= sampled.rgb;
        scatters_through *= sampled.a;
    }

    boxed_reflection = material.detail.w;
    reflection_lowest = material.reflection_box_min.xyz;
    reflection_highest = material.reflection_box_max.xyz;
    vec3 to_view = normalize(frame.camera_position.xyz - vertex_input.world_position);
    surface_response = environment_brdf(max(dot(normal, to_view), 0.0), roughness);
    sheen_response = sheen_tint == vec3(0.0)
        ? 0.0
        : sheen_albedo(max(dot(normal, to_view), 0.0), sheen_roughness);
    out_normal = vec4(
        mat3(frame.view) * normal,
        surface_reflectivity(normal, to_view, albedo.rgb, metallic, roughness)
    );

    float occlusion = 1.0;
    uint occlusion_slot = material.textures[TEXTURE_OCCLUSION];
    if (occlusion_slot != NO_TEXTURE) {
        float sampled = sample_slot(occlusion_slot, slot_uv(material, TEXTURE_OCCLUSION)).r;
        occlusion = mix(1.0, sampled, material.factors.w);
    }

    vec3 color = vec3(0.0);

    uint directional_count = min(frame.masks.y, uint(lights.length()));
    for (uint index = 0u; index < directional_count; ++index) {
        Light light = lights[index];
        vec3 to_light = normalize(-light.direction.xyz);
        vec3 radiance = light.color.rgb * light.color.a;
        float visibility = index == 0u
            ? sun_visibility(vertex_input.world_position, geometric, to_light)
            : 1.0;
        if (index == 0u) {
            debug_sun = visibility;
        }
        color += shade(normal, to_view, to_light, radiance, albedo.rgb, metallic, roughness)
            * visibility * sky_access;
    }

    float view_depth = -(frame.view * vec4(vertex_input.world_position, 1.0)).z;
    uint cell = cluster_cell(gl_FragCoord.xy, view_depth);
    uint offset = cluster_offset(cell);
    uint light_count = cluster_count(cell);
    for (uint slot = 0u; slot < light_count; ++slot) {
        Light light = lights[cluster_light(offset, slot)];
        if (light.light_type == LIGHT_AREA) {
            color += area_light_contribution(
                light,
                area_lights[uint(light.shadow_index)],
                vertex_input.world_position,
                normal,
                to_view,
                albedo.rgb,
                surface_reflectance,
                roughness,
                metallic
            );
            continue;
        }
        vec3 to_light = light.position.xyz - vertex_input.world_position;
        float distance_to_light = length(to_light);
        if (distance_to_light < 1.0e-5) {
            continue;
        }
        to_light /= distance_to_light;
        vec3 radiance = light.color.rgb * light.color.a
            * range_attenuation(distance_to_light, light.range);
        if (light.light_type == LIGHT_SPOT) {
            radiance *= spot_attenuation(
                -to_light,
                light.direction.xyz,
                light.outer_cone,
                light.inner_cone
            );
            radiance *= spot_picture(light, vertex_input.world_position);
        } else if (light.light_type == LIGHT_POINT) {
            radiance *= point_picture(light, vertex_input.world_position);
        }
        radiance *=
            local_visibility(light, vertex_input.world_position, normal, to_light, view_depth);
        color += shade(normal, to_view, to_light, radiance, albedo.rgb, metallic, roughness);
    }

    color += image_based_light(normal, to_view, albedo.rgb, metallic, roughness, occlusion)
        * sky_access * vertex_occlusion;
    color += emission;

    uint mode = uint(frame.debug.z);
    if ((objects[object_index].flags & OBJECT_DEBUG_VIEW_EXEMPT) != 0u) {
        mode = 0u;
    }
    if (mode != 0u) {
        vec3 shown;
        switch (mode) {
            case 1u: shown = albedo.rgb; break;
            case 2u: shown = normal * 0.5 + 0.5; break;
            case 3u: shown = vec3(metallic); break;
            case 4u: shown = vec3(roughness); break;
            case 5u: shown = vec3(occlusion); break;
            case 6u: shown = emission; break;
            case 7u: shown = debug_fresnel; break;
            case 8u: shown = vec3(debug_geometry); break;
            case 9u: shown = vec3(debug_distribution); break;
            case 10u: shown = debug_diffuse; break;
            case 11u: shown = debug_specular; break;
            case 12u: shown = mip_rainbow(slot_uv(material, TEXTURE_BASE), 1024.0); break;
            case 13u: shown = vec3(debug_sun); break;
            default: shown = color; break;
        }
        out_color = vec4(shown, 1.0);
        return;
    }

    if (passes_light_through > 0.0) {
        float thickness = material.transmission.y;
        uint thickness_slot = material.textures[TEXTURE_THICKNESS];
        if (thickness_slot != NO_TEXTURE) {
            thickness *= sample_slot(
                thickness_slot,
                slot_uv(material, TEXTURE_THICKNESS)
            ).g;
        }

        mat3 stood = mat3(objects[object_index].transform);
        float grown = (length(stood[0]) + length(stood[1]) + length(stood[2])) / 3.0;
        float reaches = thickness * length(vec3(grown));

        float bending = max(material.alpha.w, 1.0e-4);
        float spread = material.transmission.w;
        vec3 behind;
        if (spread > 0.0) {
            float apart = (bending - 1.0) * 0.025 * spread;
            behind = vec3(
                behind_body(bending - apart, normal, to_view, reaches, roughness).r,
                behind_body(bending, normal, to_view, reaches, roughness).g,
                behind_body(bending + apart, normal, to_view, reaches, roughness).b
            );
        } else {
            behind = behind_body(bending, normal, to_view, reaches, roughness);
        }

        float reach = material.transmission.z;
        if (reach > 0.0 && reaches > 0.0) {
            vec3 absorbed = -log(clamp(material.attenuation.rgb, vec3(1.0e-4), vec3(1.0)))
                / reach;
            behind *= exp(-absorbed * reaches);
        }

        float facing = clamp(dot(normal, to_view), 0.001, 1.0);
        vec2 response = environment_brdf(facing, roughness);
        vec3 reflected = surface_reflectance * response.x + response.y;

        vec3 kept = vec3(1.0) - reflected;
        vec3 through = kept * kept * behind * albedo.rgb;
        color += (through - gathered_around) * passes_light_through;
        albedo.a = 1.0;
    }

    if (seam_amount > 0.0) {
        color = mix(
            color,
            material.reveal_seam.rgb,
            clamp(seam_amount, 0.0, 1.0)
        );
    }

    color = with_fog(color, vertex_input.world_position);

    if (push.list == LIST_TRANSPARENT) {
        float hides = clamp(albedo.a + shine_around, 0.0, 1.0);
        float reach = max(frame.cluster.w * 0.2, 1.0);
        float weight = hides * clamp(
            0.03 / (1.0e-5 + pow(view_depth / reach, 4.0)),
            1.0e-2,
            3.0e3
        );
        color = clamp(mix(color, vec3(0.0), vec3(isnan(color))), vec3(0.0), vec3(65000.0));
        out_color = vec4(color, 1.0) * weight;
        out_normal = vec4(hides);
        return;
    }

    out_color = vec4(color, albedo.a);
}
