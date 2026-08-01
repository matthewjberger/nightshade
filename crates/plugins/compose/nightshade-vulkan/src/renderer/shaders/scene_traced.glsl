
layout(set = 2, binding = 1) uniform sampler2DShadow local_shadow_atlas;
layout(set = 3, binding = 0) uniform samplerCube environment_irradiance;
layout(set = 3, binding = 6) uniform samplerCube environment_irradiance_other;

const float PI = 3.14159265359;

vec4 picture_sample(uint slot, vec2 uv) {
    return sample_slot_flat(slot, uv);
}

#include "scene_local_lights.glsl"

vec3 arriving_light(vec3 direction) {
    return mix(
        texture(environment_irradiance, direction).rgb,
        texture(environment_irradiance_other, direction).rgb,
        frame.environment.x
    );
}

struct Met {
    uint drawn;
    uint primitive;
    vec2 weights;
    float travelled;
};

bool trace(vec3 from, vec3 direction, float step_off, float reach, out Met met) {
    rayQueryEXT query;
    rayQueryInitializeEXT(query, world, gl_RayFlagsNoneEXT, 0xffu, from, step_off, direction, reach);
    while (rayQueryProceedEXT(query)) {
        if (rayQueryGetIntersectionTypeEXT(query, false)
                == gl_RayQueryCandidateIntersectionTriangleEXT
            && cut_surface_covers(query)) {
            rayQueryConfirmIntersectionEXT(query);
        }
    }
    if (rayQueryGetIntersectionTypeEXT(query, true) == gl_RayQueryCommittedIntersectionNoneEXT) {
        return false;
    }
    met.drawn = uint(rayQueryGetIntersectionInstanceCustomIndexEXT(query, true));
    met.primitive = uint(rayQueryGetIntersectionPrimitiveIndexEXT(query, true));
    met.weights = rayQueryGetIntersectionBarycentricsEXT(query, true);
    met.travelled = rayQueryGetIntersectionTEXT(query, true);
    return true;
}

float reaches(vec3 from, vec3 normal, vec3 toward, float reach) {
    float step_off = max(length(from - frame.camera_position.xyz) * 1.0e-3, 1.0e-4);
    rayQueryEXT shadow;
    rayQueryInitializeEXT(
        shadow,
        world,
        gl_RayFlagsTerminateOnFirstHitEXT,
        0xffu,
        from + normal * step_off,
        step_off,
        toward,
        reach
    );
    while (rayQueryProceedEXT(shadow)) {
        if (rayQueryGetIntersectionTypeEXT(shadow, false)
                == gl_RayQueryCandidateIntersectionTriangleEXT
            && cut_surface_covers(shadow)) {
            rayQueryConfirmIntersectionEXT(shadow);
        }
    }
    return rayQueryGetIntersectionTypeEXT(shadow, true) == gl_RayQueryCommittedIntersectionNoneEXT
        ? 1.0
        : 0.0;
}

struct Surface {
    vec3 normal;
    vec3 albedo;
    float metallic;
    float roughness;
    vec3 emission;
    float coat;
    float coat_roughness;
    float passes;
};

Surface surface_at_seen(Met met, vec3 direction, out bool from_behind) {
    Object object = objects[met.drawn];
    MeshRange range = mesh_ranges[object.mesh_slot];
    uint base = (object.flags & OBJECT_SKINNED) != 0u ? object.vertex_offset : range.vertex_offset;
    uint first = range.index_offset + met.primitive * 3u;
    Vertex a = vertices[base + indices[first]];
    Vertex b = vertices[base + indices[first + 1u]];
    Vertex c = vertices[base + indices[first + 2u]];
    vec3 share = vec3(1.0 - met.weights.x - met.weights.y, met.weights.x, met.weights.y);
    Surface surface;
    surface.normal = normalize(
        mat3(object.normal_transform) * (a.normal * share.x + b.normal * share.y + c.normal * share.z)
    );
    from_behind = dot(surface.normal, direction) > 0.0;
    if (from_behind) {
        surface.normal = -surface.normal;
    }
    vec2 uv = a.tex_coords * share.x + b.tex_coords * share.y + c.tex_coords * share.z;
    vec2 uv_1 = a.tex_coords_1 * share.x + b.tex_coords_1 * share.y + c.tex_coords_1 * share.z;
    vec4 color = a.color * share.x + b.color * share.y + c.color * share.z;

    surface.albedo = (color * object.tint).rgb;
    surface.metallic = 0.0;
    surface.roughness = 1.0;
    surface.emission = vec3(0.0);
    surface.coat = 0.0;
    surface.coat_roughness = 0.0;
    surface.passes = 0.0;
    if (object.material_index == NO_MATERIAL) {
        return surface;
    }
    GpuMaterial material = materials[object.material_index];
    surface.albedo = (surface_tint(material, color) * object.tint).rgb * material.base_color.rgb;
    uint base_slot = material.textures[TEXTURE_BASE];
    if (base_slot != NO_TEXTURE) {
        surface.albedo *= sample_slot_flat(
            base_slot,
            atlas_uv(material, slot_uv_of(material, TEXTURE_BASE, uv, uv_1), uv_1.x)
        ).rgb;
    }
    surface.metallic = material.factors.x;
    surface.roughness = material.factors.y;
    uint metallic_slot = material.textures[TEXTURE_METALLIC_ROUGHNESS];
    if (metallic_slot != NO_TEXTURE) {
        vec4 sampled = sample_slot_flat(
            metallic_slot,
            slot_uv_of(material, TEXTURE_METALLIC_ROUGHNESS, uv, uv_1)
        );
        surface.roughness *= sampled.g;
        surface.metallic *= sampled.b;
    }
    surface.roughness = clamp(surface.roughness, 0.04, 1.0);
    surface.emission = material.emissive.rgb * material.emissive.a;
    uint emissive_slot = material.textures[TEXTURE_EMISSIVE];
    if (emissive_slot != NO_TEXTURE) {
        surface.emission *= sample_slot_flat(
            emissive_slot,
            slot_uv_of(material, TEXTURE_EMISSIVE, uv, uv_1)
        ).rgb;
    }
    surface.coat = material.sheen.z;
    surface.coat_roughness = clamp(material.sheen.w, 0.04, 1.0);
    surface.passes = material.transmission.x;
    return surface;
}

Surface surface_at(Met met, vec3 direction) {
    bool from_behind;
    return surface_at_seen(met, direction, from_behind);
}

float distribution_ggx(float normal_half, float roughness) {
    float alpha = roughness * roughness;
    float alpha_squared = alpha * alpha;
    float denominator = normal_half * normal_half * (alpha_squared - 1.0) + 1.0;
    return alpha_squared / max(PI * denominator * denominator, 1.0e-5);
}

float visibility_smith(float normal_view, float normal_light, float roughness) {
    float alpha = roughness * roughness;
    float view_term = normal_light * sqrt(normal_view * normal_view * (1.0 - alpha) + alpha);
    float light_term = normal_view * sqrt(normal_light * normal_light * (1.0 - alpha) + alpha);
    return 0.5 / max(view_term + light_term, 1.0e-5);
}

vec3 fresnel_schlick(float cosine, vec3 reflectance) {
    return reflectance + (1.0 - reflectance) * pow(clamp(1.0 - cosine, 0.0, 1.0), 5.0);
}

vec3 lit_by(Surface surface, vec3 to_view, vec3 to_light, vec3 radiance) {
    float normal_light = max(dot(surface.normal, to_light), 0.0);
    if (normal_light <= 0.0) {
        return vec3(0.0);
    }
    vec3 half_vector = normalize(to_view + to_light);
    float normal_view = max(dot(surface.normal, to_view), 1.0e-4);
    float normal_half = max(dot(surface.normal, half_vector), 0.0);
    float view_half = max(dot(to_view, half_vector), 0.0);
    vec3 reflectance = mix(vec3(0.04), surface.albedo, surface.metallic);
    vec3 fresnel = fresnel_schlick(view_half, reflectance);
    vec3 specular = fresnel
        * distribution_ggx(normal_half, surface.roughness)
        * visibility_smith(normal_view, normal_light, surface.roughness);
    vec3 diffuse = (vec3(1.0) - fresnel) * (1.0 - surface.metallic) * surface.albedo / PI;
    vec3 lit = (diffuse + specular) * normal_light * radiance;
    if (surface.coat > 0.0) {
        float taken = surface.coat * (0.04 + 0.96 * pow(1.0 - view_half, 5.0));
        float coat_highlight = 0.04
            * distribution_ggx(normal_half, surface.coat_roughness)
            * visibility_smith(normal_view, normal_light, surface.coat_roughness);
        lit = lit * (1.0 - taken) + coat_highlight * surface.coat * normal_light * radiance;
    }
    return lit;
}

vec3 shade(Surface surface, vec3 met, vec3 direction, float reach) {
    vec3 to_view = -direction;
    vec3 body = surface.albedo * (1.0 - surface.metallic);
    vec3 lit = surface.emission + body * arriving_light(surface.normal);
    uint light_count = min(frame.counts.y, uint(lights.length()));
    uint directional_count = min(frame.masks.y, light_count);
    for (uint index = 0u; index < directional_count; ++index) {
        Light light = lights[index];
        vec3 to_light = normalize(-light.direction.xyz);
        if (dot(surface.normal, to_light) <= 0.0) {
            continue;
        }
        vec3 radiance = light.color.rgb * light.color.a
            * reaches(met, surface.normal, to_light, reach);
        lit += lit_by(surface, to_view, to_light, radiance);
    }
    for (uint index = directional_count; index < light_count; ++index) {
        Light light = lights[index];
        if (light.light_type == LIGHT_AREA) {
            continue;
        }
        vec3 to_light = light.position.xyz - met;
        float distance_to_light = length(to_light);
        to_light /= max(distance_to_light, 1.0e-4);
        if (dot(surface.normal, to_light) <= 0.0) {
            continue;
        }
        vec3 radiance = light.color.rgb * light.color.a
            * range_attenuation(distance_to_light, light.range);
        if (light.light_type == LIGHT_SPOT) {
            radiance *= spot_attenuation(
                -to_light,
                light.direction.xyz,
                light.outer_cone,
                light.inner_cone
            ) * spot_picture(light, met);
        } else {
            radiance *= point_picture(light, met);
        }
        radiance *= local_reaches(light, met);
        lit += lit_by(surface, to_view, to_light, radiance);
    }
    return lit;
}
