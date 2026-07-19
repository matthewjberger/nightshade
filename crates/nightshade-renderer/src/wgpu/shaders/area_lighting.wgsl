#define_import_path nightshade_renderer::area_lighting

#import nightshade_renderer::material_sampling::sample_srgb_index

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

@group(3) @binding(19)
var ltc_matrix_lut: texture_2d<f32>;

@group(3) @binding(20)
var ltc_terms_lut: texture_2d<f32>;

@group(3) @binding(21)
var ltc_sampler: sampler;

const LTC_LUT_SCALE: f32 = 0.984375;
const LTC_LUT_BIAS: f32 = 0.0078125;

const AREA_PI: f32 = 3.14159265359;
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

fn area_integrate_edge(v1: vec3<f32>, v2: vec3<f32>) -> f32 {
    let x = dot(v1, v2);
    let y = abs(x);
    let a = 0.8543985 + (0.4965155 + 0.0145206 * y) * y;
    let b = 3.4175940 + (4.1616724 + y) * y;
    let v = a / b;
    var theta_sintheta = v;
    if x <= 0.0 {
        theta_sintheta = 0.5 * inverseSqrt(max(1.0 - x * x, 1e-7)) - v;
    }
    return cross(v1, v2).z * theta_sintheta;
}

fn area_ltc_evaluate(
    surface_normal: vec3<f32>,
    view: vec3<f32>,
    surface_pos: vec3<f32>,
    corner_a: vec3<f32>,
    corner_b: vec3<f32>,
    corner_c: vec3<f32>,
    corner_d: vec3<f32>,
    inv_transform: mat3x3<f32>,
    two_sided: bool,
) -> f32 {
    let tangent = normalize(view - surface_normal * dot(view, surface_normal));
    let bitangent = cross(surface_normal, tangent);
    let basis = transpose(mat3x3<f32>(tangent, bitangent, surface_normal));
    let m = inv_transform * basis;

    let l0 = normalize(m * (corner_a - surface_pos));
    let l1 = normalize(m * (corner_b - surface_pos));
    let l2 = normalize(m * (corner_c - surface_pos));
    let l3 = normalize(m * (corner_d - surface_pos));

    var sum = 0.0;
    sum = sum + area_integrate_edge(l0, l1);
    sum = sum + area_integrate_edge(l1, l2);
    sum = sum + area_integrate_edge(l2, l3);
    sum = sum + area_integrate_edge(l3, l0);

    if two_sided {
        sum = abs(sum);
    } else {
        sum = max(0.0, sum);
    }
    return sum / (2.0 * AREA_PI);
}

fn area_polygon_form_factor(
    surface_normal: vec3<f32>,
    view: vec3<f32>,
    surface_pos: vec3<f32>,
    corner_a: vec3<f32>,
    corner_b: vec3<f32>,
    corner_c: vec3<f32>,
    corner_d: vec3<f32>,
    two_sided: bool,
) -> f32 {
    let identity = mat3x3<f32>(
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(0.0, 0.0, 1.0),
    );
    return area_ltc_evaluate(
        surface_normal, view, surface_pos,
        corner_a, corner_b, corner_c, corner_d,
        identity, two_sided,
    );
}

fn area_ltc_uv(lut_roughness: f32, n_dot_v: f32) -> vec2<f32> {
    var uv = vec2<f32>(lut_roughness, sqrt(clamp(1.0 - n_dot_v, 0.0, 1.0)));
    return uv * LTC_LUT_SCALE + LTC_LUT_BIAS;
}

fn area_ltc_matrix(uv: vec2<f32>) -> mat3x3<f32> {
    let t = textureSampleLevel(ltc_matrix_lut, ltc_sampler, uv, 0.0);
    return mat3x3<f32>(
        vec3<f32>(t.x, 0.0, t.y),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(t.z, 0.0, t.w),
    );
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

fn evaluate_one_area_light(
    light: AreaLight,
    surface_pos: vec3<f32>,
    surface_normal: vec3<f32>,
    view: vec3<f32>,
    albedo: vec3<f32>,
    roughness: f32,
    metallic: f32,
    f0: vec3<f32>,
) -> vec3<f32> {
    let quad = area_build_quad(light, surface_pos);
    let two_sided = light.two_sided != 0u;

    let to_center = quad.center - surface_pos;
    let distance = length(to_center);
    if light.range > 0.0 && distance > light.range + quad.extent {
        return vec3<f32>(0.0);
    }

    let facing = dot(normalize(surface_pos - quad.center), quad.normal);
    if !two_sided && facing <= 0.0 {
        return vec3<f32>(0.0);
    }

    let attenuation = area_range_attenuation(light.range, distance);
    var radiance = light.color.rgb * attenuation;

    if light.emissive_layer != AREA_EMISSIVE_LAYER_NONE {
        let local = surface_pos - quad.center;
        let len_u = max(length(quad.axis_u), 1e-5);
        let len_v = max(length(quad.axis_v), 1e-5);
        let uv = vec2<f32>(
            clamp(dot(local, quad.axis_u / len_u) / len_u, -1.0, 1.0) * 0.5 + 0.5,
            clamp(dot(local, quad.axis_v / len_v) / len_v, -1.0, 1.0) * 0.5 + 0.5,
        );
        radiance = radiance * sample_srgb_index(light.emissive_layer & 0xFFFFu, uv).rgb;
    }

    let diffuse_factor = area_polygon_form_factor(
        surface_normal,
        view,
        surface_pos,
        quad.corner_a,
        quad.corner_b,
        quad.corner_c,
        quad.corner_d,
        two_sided,
    );

    let k_diffuse = (vec3<f32>(1.0) - f0) * (1.0 - metallic);
    var result = k_diffuse * albedo * diffuse_factor * radiance;

    let n_dot_v = max(dot(surface_normal, view), 1e-4);
    let lut_roughness = clamp(roughness * roughness, 0.001, 1.0);
    let ltc_uv = area_ltc_uv(lut_roughness, n_dot_v);
    let inv_transform = area_ltc_matrix(ltc_uv);
    let ltc_terms = textureSampleLevel(ltc_terms_lut, ltc_sampler, ltc_uv, 0.0);
    let spec_factor = area_ltc_evaluate(
        surface_normal, view, surface_pos,
        quad.corner_a, quad.corner_b, quad.corner_c, quad.corner_d,
        inv_transform, two_sided,
    );
    let spec_color = f0 * ltc_terms.x + (vec3<f32>(1.0) - f0) * ltc_terms.y;
    result = result + spec_factor * spec_color * radiance;

    return result;
}
