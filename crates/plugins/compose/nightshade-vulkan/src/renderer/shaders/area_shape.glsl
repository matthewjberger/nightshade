
#define AREA_SHAPE_RECTANGLE 0u
#define AREA_SHAPE_DISK 1u
#define AREA_SHAPE_SPHERE 2u
#define AREA_SHAPE_TUBE 3u
#define AREA_NO_EMISSIVE 0xFFFFFFFFu

float area_range_attenuation(float range, float distance_to_light) {
    if (range <= 0.0) {
        return 1.0;
    }
    float window = clamp(1.0 - pow(distance_to_light / range, 4.0), 0.0, 1.0);
    return window * window;
}

struct AreaQuad {
    vec3 center;
    vec3 axis_u;
    vec3 axis_v;
    vec3 normal;
    vec3 corner_a;
    vec3 corner_b;
    vec3 corner_c;
    vec3 corner_d;
    float extent;
};

AreaQuad area_build_quad(Light light, AreaLight area, vec3 surface) {
    AreaQuad quad;
    vec3 center = light.position.xyz;
    quad.center = center;

    vec3 axis_u = area.right.xyz;
    vec3 axis_v = area.up.xyz;
    vec3 normal = light.direction.xyz;

    if (area.shape == AREA_SHAPE_SPHERE) {
        vec3 face = normalize(surface - center);
        vec3 along = abs(face.y) > 0.99 ? vec3(1.0, 0.0, 0.0) : vec3(0.0, 1.0, 0.0);
        vec3 tangent = normalize(cross(along, face));
        vec3 bitangent = cross(tangent, face);
        axis_u = tangent * area.radius;
        axis_v = bitangent * area.radius;
        normal = face;
    }

    if (area.shape == AREA_SHAPE_TUBE) {
        vec3 axis = normalize(area.right.xyz);
        vec3 to_surface = surface - center;
        vec3 across = to_surface - axis * dot(to_surface, axis);
        if (dot(across, across) < 1.0e-5) {
            across = light.direction.xyz;
        }
        across = normalize(across);
        normal = across;
        axis_u = area.right.xyz;
        axis_v = normalize(cross(axis, across)) * area.radius;
    }

    quad.axis_u = axis_u;
    quad.axis_v = axis_v;
    quad.normal = normal;
    quad.corner_a = center - axis_u - axis_v;
    quad.corner_b = center + axis_u - axis_v;
    quad.corner_c = center + axis_u + axis_v;
    quad.corner_d = center - axis_u + axis_v;
    quad.extent = max(length(axis_u), length(axis_v));
    return quad;
}

vec3 area_light_in_air(Light light, AreaLight area, vec3 place, out vec3 to_light) {
    AreaQuad quad = area_build_quad(light, area, place);
    vec3 toward = quad.center - place;
    float distance_to_light = length(toward);
    to_light = toward / max(distance_to_light, 1.0e-4);
    if (light.range > 0.0 && distance_to_light > light.range + quad.extent) {
        return vec3(0.0);
    }
    if (area.two_sided == 0u && dot(-to_light, quad.normal) <= 0.0) {
        return vec3(0.0);
    }
    vec3 radiance = light.color.rgb * light.color.a
        * area_range_attenuation(
            light.range,
            sqrt(distance_to_light * distance_to_light + quad.extent * quad.extent)
        );
    if (area.emissive_layer != AREA_NO_EMISSIVE) {
        vec3 local = place - quad.center;
        float length_u = max(length(quad.axis_u), 1.0e-5);
        float length_v = max(length(quad.axis_v), 1.0e-5);
        vec2 uv = vec2(
            clamp(dot(local, quad.axis_u / length_u) / length_u, -1.0, 1.0) * 0.5 + 0.5,
            clamp(dot(local, quad.axis_v / length_v) / length_v, -1.0, 1.0) * 0.5 + 0.5
        );
        radiance *= picture_sample(area.emissive_layer & 0xFFFFu, uv).rgb;
    }
    return radiance;
}
