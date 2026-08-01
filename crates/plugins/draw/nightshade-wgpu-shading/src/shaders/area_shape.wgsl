#define_import_path nightshade_wgpu::area_shape

#import nightshade_wgpu::material_sampling::sample_srgb_index

struct AreaLight {
    position: vec4<f32>,
    right: vec4<f32>,
    up: vec4<f32>,
    color: vec4<f32>,
    shape: u32,
    range: f32,
    radius: f32,
    two_sided: u32,
    shadow_index: i32,
    emissive_layer: u32,
    area_light_pad_a: f32,
    area_light_pad_b: f32,
};

struct AreaUniforms {
    count: u32,
    area_uniforms_pad_a: u32,
    area_uniforms_pad_b: u32,
    area_uniforms_pad_c: u32,
};

@group(1) @binding(10)
var<storage, read> area_lights: array<AreaLight>;

@group(1) @binding(11)
var<uniform> area_uniforms: AreaUniforms;

const AREA_SHAPE_RECTANGLE: u32 = 0u;
const AREA_SHAPE_DISK: u32 = 1u;
const AREA_SHAPE_SPHERE: u32 = 2u;
const AREA_SHAPE_TUBE: u32 = 3u;
const AREA_EMISSIVE_LAYER_NONE: u32 = 0xFFFFFFFFu;

fn area_light_count() -> u32 {
    return area_uniforms.count;
}

fn area_light_at(index: u32) -> AreaLight {
    return area_lights[index];
}

fn area_normal(light: AreaLight) -> vec3<f32> {
    return vec3<f32>(light.position.w, light.right.w, light.up.w);
}

fn area_range_attenuation(range: f32, distance: f32) -> f32 {
    if range <= 0.0 {
        return 1.0;
    }
    let window = clamp(1.0 - pow(distance / range, 4.0), 0.0, 1.0);
    return window * window;
}

struct AreaQuad {
    center: vec3<f32>,
    axis_u: vec3<f32>,
    axis_v: vec3<f32>,
    normal: vec3<f32>,
    corner_a: vec3<f32>,
    corner_b: vec3<f32>,
    corner_c: vec3<f32>,
    corner_d: vec3<f32>,
    extent: f32,
};

fn area_build_quad(light: AreaLight, surface_pos: vec3<f32>) -> AreaQuad {
    var quad: AreaQuad;
    let center = light.position.xyz;
    quad.center = center;

    var axis_u = light.right.xyz;
    var axis_v = light.up.xyz;
    var normal = area_normal(light);

    if light.shape == AREA_SHAPE_SPHERE {
        let face = normalize(surface_pos - center);
        let tangent = normalize(cross(select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(face.y) > 0.99), face));
        let bitangent = cross(tangent, face);
        axis_u = tangent * light.radius;
        axis_v = bitangent * light.radius;
        normal = face;
    }

    if light.shape == AREA_SHAPE_TUBE {
        let axis = normalize(light.right.xyz);
        let to_surface = surface_pos - center;
        var perp = to_surface - axis * dot(to_surface, axis);
        if dot(perp, perp) < 1e-5 {
            perp = area_normal(light);
        }
        perp = normalize(perp);
        normal = perp;
        axis_u = light.right.xyz;
        axis_v = normalize(cross(axis, perp)) * light.radius;
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

struct AreaInAir {
    radiance: vec3<f32>,
    to_light: vec3<f32>,
};

fn area_light_in_air(light: AreaLight, place: vec3<f32>) -> AreaInAir {
    var found: AreaInAir;
    let quad = area_build_quad(light, place);
    let toward = quad.center - place;
    let distance_to_light = length(toward);
    found.to_light = toward / max(distance_to_light, 1.0e-4);
    found.radiance = vec3<f32>(0.0);
    if light.range > 0.0 && distance_to_light > light.range + quad.extent {
        return found;
    }
    if light.two_sided == 0u && dot(-found.to_light, quad.normal) <= 0.0 {
        return found;
    }
    var radiance = light.color.rgb
        * area_range_attenuation(
            light.range,
            sqrt(distance_to_light * distance_to_light + quad.extent * quad.extent)
        );
    if light.emissive_layer != AREA_EMISSIVE_LAYER_NONE {
        let local = place - quad.center;
        let len_u = max(length(quad.axis_u), 1e-5);
        let len_v = max(length(quad.axis_v), 1e-5);
        let uv = vec2<f32>(
            clamp(dot(local, quad.axis_u / len_u) / len_u, -1.0, 1.0) * 0.5 + 0.5,
            clamp(dot(local, quad.axis_v / len_v) / len_v, -1.0, 1.0) * 0.5 + 0.5,
        );
        radiance = radiance * sample_srgb_index(light.emissive_layer & 0xFFFFu, uv).rgb;
    }
    found.radiance = radiance;
    return found;
}
