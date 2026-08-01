#define_import_path nightshade_wgpu::area_lighting

#import nightshade_wgpu::material_sampling::sample_srgb_index

#import nightshade_wgpu::area_shape::{AreaLight, AreaQuad, area_build_quad, area_range_attenuation, AREA_EMISSIVE_LAYER_NONE}

@group(3) @binding(19)
var ltc_matrix_lut: texture_2d<f32>;

@group(3) @binding(20)
var ltc_terms_lut: texture_2d<f32>;

@group(3) @binding(21)
var ltc_sampler: sampler;

const LTC_LUT_SCALE: f32 = 0.984375;
const LTC_LUT_BIAS: f32 = 0.0078125;

fn area_integrate_edge(v1: vec3<f32>, v2: vec3<f32>) -> vec3<f32> {
    let x = dot(v1, v2);
    let y = abs(x);
    let a = 0.8543985 + (0.4965155 + 0.0145206 * y) * y;
    let b = 3.4175940 + (4.1616724 + y) * y;
    let v = a / b;
    var theta_sintheta = v;
    if x <= 0.0 {
        theta_sintheta = 0.5 * inverseSqrt(max(1.0 - x * x, 1e-7)) - v;
    }
    return cross(v1, v2) * theta_sintheta;
}

fn area_clipped_form_factor(summed: vec3<f32>, two_sided: bool) -> f32 {
    let len = length(summed);
    let upward = select(summed.z, abs(summed.z), two_sided);
    return max((len * len + upward) / (len + 1.0), 0.0);
}

fn area_tangent(surface_normal: vec3<f32>, view: vec3<f32>) -> vec3<f32> {
    var along = view - surface_normal * dot(view, surface_normal);
    if dot(along, along) < 1.0e-6 {
        let other = select(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), abs(surface_normal.y) < 0.99);
        along = other - surface_normal * dot(other, surface_normal);
    }
    return normalize(along);
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
    let tangent = area_tangent(surface_normal, view);
    let bitangent = cross(surface_normal, tangent);
    let basis = transpose(mat3x3<f32>(tangent, bitangent, surface_normal));
    let m = inv_transform * basis;

    let l0 = normalize(m * (corner_a - surface_pos));
    let l1 = normalize(m * (corner_b - surface_pos));
    let l2 = normalize(m * (corner_c - surface_pos));
    let l3 = normalize(m * (corner_d - surface_pos));

    let sum = area_integrate_edge(l0, l1)
        + area_integrate_edge(l1, l2)
        + area_integrate_edge(l2, l3)
        + area_integrate_edge(l3, l0);
    return area_clipped_form_factor(sum, two_sided);
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
