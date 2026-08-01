
layout(set = 3, binding = 3) uniform sampler2D area_matrix_table;
layout(set = 3, binding = 4) uniform sampler2D area_terms_table;

#define AREA_TABLE_SCALE 0.984375
#define AREA_TABLE_BIAS 0.0078125

#include "area_shape.glsl"

vec3 area_integrate_edge(vec3 first, vec3 second) {
    float cosine = dot(first, second);
    float magnitude = abs(cosine);
    float upper = 0.8543985 + (0.4965155 + 0.0145206 * magnitude) * magnitude;
    float lower = 3.4175940 + (4.1616724 + magnitude) * magnitude;
    float ratio = upper / lower;
    float weight = cosine > 0.0
        ? ratio
        : 0.5 * inversesqrt(max(1.0 - cosine * cosine, 1.0e-7)) - ratio;
    return cross(first, second) * weight;
}

float area_clipped_form_factor(vec3 summed, bool two_sided) {
    float len = length(summed);
    float upward = two_sided ? abs(summed.z) : summed.z;
    return max((len * len + upward) / (len + 1.0), 0.0);
}

vec3 area_tangent(vec3 surface_normal, vec3 to_view) {
    vec3 along = to_view - surface_normal * dot(to_view, surface_normal);
    if (dot(along, along) < 1.0e-6) {
        vec3 other = abs(surface_normal.y) < 0.99 ? vec3(0.0, 1.0, 0.0) : vec3(1.0, 0.0, 0.0);
        along = other - surface_normal * dot(other, surface_normal);
    }
    return normalize(along);
}

float area_evaluate(
    vec3 surface_normal,
    vec3 to_view,
    vec3 surface,
    vec3 corner_a,
    vec3 corner_b,
    vec3 corner_c,
    vec3 corner_d,
    mat3 inverse_transform,
    bool two_sided
) {
    vec3 tangent = area_tangent(surface_normal, to_view);
    vec3 bitangent = cross(surface_normal, tangent);
    mat3 basis = transpose(mat3(tangent, bitangent, surface_normal));
    mat3 turned = inverse_transform * basis;

    vec3 first = normalize(turned * (corner_a - surface));
    vec3 second = normalize(turned * (corner_b - surface));
    vec3 third = normalize(turned * (corner_c - surface));
    vec3 fourth = normalize(turned * (corner_d - surface));

    vec3 total = area_integrate_edge(first, second)
        + area_integrate_edge(second, third)
        + area_integrate_edge(third, fourth)
        + area_integrate_edge(fourth, first);
    return area_clipped_form_factor(total, two_sided);
}

vec2 area_table_uv(float table_roughness, float normal_view) {
    vec2 uv = vec2(table_roughness, sqrt(clamp(1.0 - normal_view, 0.0, 1.0)));
    return uv * AREA_TABLE_SCALE + AREA_TABLE_BIAS;
}

mat3 area_table_matrix(vec2 uv) {
    vec4 entry = textureLod(area_matrix_table, uv, 0.0);
    return mat3(
        vec3(entry.x, 0.0, entry.y),
        vec3(0.0, 1.0, 0.0),
        vec3(entry.z, 0.0, entry.w)
    );
}

vec3 area_light_contribution(
    Light light,
    AreaLight area,
    vec3 surface,
    vec3 surface_normal,
    vec3 to_view,
    vec3 albedo,
    vec3 reflectance,
    float roughness,
    float metallic
) {
    AreaQuad quad = area_build_quad(light, area, surface);
    bool two_sided = area.two_sided != 0u;

    float distance_to_light = length(quad.center - surface);
    if (light.range > 0.0 && distance_to_light > light.range + quad.extent) {
        return vec3(0.0);
    }

    float facing = dot(normalize(surface - quad.center), quad.normal);
    if (!two_sided && facing <= 0.0) {
        return vec3(0.0);
    }

    vec3 radiance = light.color.rgb * light.color.a
        * area_range_attenuation(light.range, distance_to_light);

    if (area.emissive_layer != AREA_NO_EMISSIVE) {
        vec3 local = surface - quad.center;
        float length_u = max(length(quad.axis_u), 1.0e-5);
        float length_v = max(length(quad.axis_v), 1.0e-5);
        vec2 uv = vec2(
            clamp(dot(local, quad.axis_u / length_u) / length_u, -1.0, 1.0) * 0.5 + 0.5,
            clamp(dot(local, quad.axis_v / length_v) / length_v, -1.0, 1.0) * 0.5 + 0.5
        );
        radiance *= sample_slot(area.emissive_layer & 0xFFFFu, uv).rgb;
    }

    float diffuse_factor = area_evaluate(
        surface_normal,
        to_view,
        surface,
        quad.corner_a,
        quad.corner_b,
        quad.corner_c,
        quad.corner_d,
        mat3(1.0),
        two_sided
    );
    vec3 result = (vec3(1.0) - reflectance)
        * (1.0 - metallic)
        * albedo
        * diffuse_factor
        * radiance;

    float normal_view = max(dot(surface_normal, to_view), 1.0e-4);
    vec2 uv = area_table_uv(clamp(roughness * roughness, 0.001, 1.0), normal_view);
    float specular_factor = area_evaluate(
        surface_normal,
        to_view,
        surface,
        quad.corner_a,
        quad.corner_b,
        quad.corner_c,
        quad.corner_d,
        area_table_matrix(uv),
        two_sided
    );
    vec4 terms = textureLod(area_terms_table, uv, 0.0);
    vec3 specular_color = reflectance * terms.x + (vec3(1.0) - reflectance) * terms.y;
    return result + specular_factor * specular_color * radiance;
}
