#define_import_path nightshade_wgpu::pbr_brdf

const PI: f32 = 3.14159265358979323846;

fn DistributionGGX(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let NdotH2 = NdotH * NdotH;
    let nom = a2;
    let denom = (NdotH2 * (a2 - 1.0) + 1.0);
    return nom / (PI * denom * denom);
}

fn GeometrySchlickGGX(NdotV: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    let nom = NdotV;
    let denom = NdotV * (1.0 - k) + k;
    return nom / denom;
}

fn GeometrySmith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let ggx2 = GeometrySchlickGGX(NdotV, roughness);
    let ggx1 = GeometrySchlickGGX(NdotL, roughness);
    return ggx1 * ggx2;
}

fn V_SmithGGXCorrelated(NdotV: f32, NdotL: f32, roughness: f32) -> f32 {
    let alpha = roughness * roughness;
    let a2 = alpha * alpha;
    let ggx_v = NdotL * sqrt(NdotV * NdotV * (1.0 - a2) + a2);
    let ggx_l = NdotV * sqrt(NdotL * NdotL * (1.0 - a2) + a2);
    return 0.5 / max(ggx_v + ggx_l, 1e-5);
}

fn fresnelSchlick(cosTheta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (1.0 - F0) * pow(max(1.0 - cosTheta, 0.0), 5.0);
}

fn fresnelSchlickRoughness(cosTheta: f32, F0: vec3<f32>, roughness: f32) -> vec3<f32> {
    return F0 + (max(vec3<f32>(1.0 - roughness), F0) - F0) * pow(max(1.0 - cosTheta, 0.0), 5.0);
}

const SPECULAR_AA_VARIANCE: f32 = 0.25;
const SPECULAR_AA_THRESHOLD: f32 = 0.18;

fn specularAntiAliasing(perceptual_roughness: f32, world_normal: vec3<f32>) -> f32 {
    let du = dpdx(world_normal);
    let dv = dpdy(world_normal);
    let variance = SPECULAR_AA_VARIANCE * (dot(du, du) + dot(dv, dv));
    let roughness = perceptual_roughness * perceptual_roughness;
    let kernel_roughness = min(2.0 * variance, SPECULAR_AA_THRESHOLD);
    let square_roughness = clamp(roughness * roughness + kernel_roughness, 0.0, 1.0);
    return sqrt(sqrt(square_roughness));
}

const CLEARCOAT_F0: f32 = 0.04;

fn fresnel_clearcoat(cos_theta: f32) -> f32 {
    return CLEARCOAT_F0 + (1.0 - CLEARCOAT_F0) * pow(max(1.0 - cos_theta, 0.0), 5.0);
}

fn clearcoat_specular_lobe(
    n_dot_v: f32,
    n_dot_l: f32,
    n_dot_h: f32,
    v_dot_h: f32,
    roughness: f32,
) -> f32 {
    let cc_F = fresnel_clearcoat(v_dot_h);
    let alpha = roughness * roughness;
    let alpha2 = alpha * alpha;
    let denom_d = n_dot_h * n_dot_h * (alpha2 - 1.0) + 1.0;
    let cc_D = alpha2 / (PI * denom_d * denom_d);
    let k = (roughness + 1.0) * (roughness + 1.0) / 8.0;
    let cc_GV = n_dot_v / (n_dot_v * (1.0 - k) + k);
    let cc_GL = n_dot_l / (n_dot_l * (1.0 - k) + k);
    let cc_G = cc_GV * cc_GL;
    return cc_F * cc_D * cc_G / max(4.0 * n_dot_v * n_dot_l, 0.001);
}

fn d_charlie(roughness: f32, n_dot_h: f32) -> f32 {
    let alpha = max(roughness * roughness, 0.0001);
    let inv_alpha = 1.0 / alpha;
    let cos2h = n_dot_h * n_dot_h;
    let sin2h = max(1.0 - cos2h, 0.0078125);
    return (2.0 + inv_alpha) * pow(sin2h, inv_alpha * 0.5) / (2.0 * PI);
}

fn v_neubelt(n_dot_v: f32, n_dot_l: f32) -> f32 {
    return clamp(1.0 / (4.0 * (n_dot_l + n_dot_v - n_dot_l * n_dot_v)), 0.0, 1.0);
}

fn sheen_lobe(
    sheen_color: vec3<f32>,
    n_dot_v: f32,
    n_dot_l: f32,
    n_dot_h: f32,
    roughness: f32,
) -> vec3<f32> {
    return sheen_color * d_charlie(roughness, n_dot_h) * v_neubelt(n_dot_v, n_dot_l);
}

fn fresnel0_to_ior(f0: vec3<f32>) -> vec3<f32> {
    let sqrt_f0 = sqrt(clamp(f0, vec3<f32>(0.0), vec3<f32>(0.9999)));
    return (vec3<f32>(1.0) + sqrt_f0) / (vec3<f32>(1.0) - sqrt_f0);
}

fn ior_to_fresnel0_v(transmitted: vec3<f32>, incident: f32) -> vec3<f32> {
    let r = (transmitted - vec3<f32>(incident)) / (transmitted + vec3<f32>(incident));
    return r * r;
}

fn ior_to_fresnel0_f(transmitted: f32, incident: f32) -> f32 {
    let r = (transmitted - incident) / (transmitted + incident);
    return r * r;
}

fn eval_sensitivity(opd: f32, shift: vec3<f32>) -> vec3<f32> {
    let phase = 2.0 * PI * opd * 1.0e-9;
    let val = vec3<f32>(5.4856e-13, 4.4201e-13, 5.2481e-13);
    let pos = vec3<f32>(1.6810e+06, 1.7953e+06, 2.2084e+06);
    let var_ = vec3<f32>(4.3278e+09, 9.3046e+09, 6.6121e+09);
    var xyz = val * sqrt(2.0 * PI * var_) * cos(pos * phase + shift) * exp(-phase * phase * var_);
    xyz.x = xyz.x + 9.7470e-14 * sqrt(2.0 * PI * 4.5282e+09)
        * cos(2.2399e+06 * phase + shift.x) * exp(-4.5282e+09 * phase * phase);
    xyz = xyz / 1.0685e-7;
    let xyz_to_rec709 = mat3x3<f32>(
        vec3<f32>(3.2404542, -0.9692660, 0.0556434),
        vec3<f32>(-1.5371385, 1.8760108, -0.2040259),
        vec3<f32>(-0.4985314, 0.0415560, 1.0572252),
    );
    return xyz_to_rec709 * xyz;
}

fn d_ggx_anisotropic(n_dot_h: f32, t_dot_h: f32, b_dot_h: f32, at: f32, ab: f32) -> f32 {
    let a2 = at * ab;
    let f = vec3<f32>(ab * t_dot_h, at * b_dot_h, a2 * n_dot_h);
    let w2 = a2 / max(dot(f, f), 1.0e-6);
    return a2 * w2 * w2 / PI;
}

fn v_ggx_anisotropic(
    n_dot_l: f32,
    n_dot_v: f32,
    b_dot_v: f32,
    t_dot_v: f32,
    b_dot_l: f32,
    t_dot_l: f32,
    at: f32,
    ab: f32,
) -> f32 {
    let ggx_v = n_dot_l * length(vec3<f32>(at * t_dot_v, ab * b_dot_v, n_dot_v));
    let ggx_l = n_dot_v * length(vec3<f32>(at * t_dot_l, ab * b_dot_l, n_dot_l));
    let v = 0.5 / max(ggx_v + ggx_l, 1.0e-4);
    return clamp(v, 0.0, 1.0);
}

fn eval_iridescence(
    outside_ior: f32,
    eta2: f32,
    cos_theta1: f32,
    thin_film_thickness: f32,
    base_f0: vec3<f32>,
) -> vec3<f32> {
    let iridescence_ior_smooth = mix(outside_ior, eta2, smoothstep(0.0, 0.03, thin_film_thickness));
    let sin_theta2_sq = (outside_ior / iridescence_ior_smooth) * (outside_ior / iridescence_ior_smooth)
        * (1.0 - cos_theta1 * cos_theta1);
    let cos_theta2_sq = 1.0 - sin_theta2_sq;
    if cos_theta2_sq < 0.0 {
        return vec3<f32>(1.0);
    }
    let cos_theta2 = sqrt(cos_theta2_sq);

    let r0_film = ior_to_fresnel0_f(iridescence_ior_smooth, outside_ior);
    let r12 = r0_film + (1.0 - r0_film) * pow(max(1.0 - cos_theta1, 0.0), 5.0);
    let t121 = 1.0 - r12;
    var phi12 = 0.0;
    if iridescence_ior_smooth < outside_ior {
        phi12 = PI;
    }
    let phi21 = PI - phi12;

    let base_ior = fresnel0_to_ior(base_f0);
    let r1 = ior_to_fresnel0_v(base_ior, iridescence_ior_smooth);
    let r23 = r1 + (vec3<f32>(1.0) - r1) * pow(max(1.0 - cos_theta2, 0.0), 5.0);
    var phi23 = vec3<f32>(0.0);
    if base_ior.x < iridescence_ior_smooth { phi23.x = PI; }
    if base_ior.y < iridescence_ior_smooth { phi23.y = PI; }
    if base_ior.z < iridescence_ior_smooth { phi23.z = PI; }

    let opd = 2.0 * iridescence_ior_smooth * thin_film_thickness * cos_theta2;
    let phi = vec3<f32>(phi21) + phi23;

    let r123 = clamp(vec3<f32>(r12) * r23, vec3<f32>(1.0e-5), vec3<f32>(0.9999));
    let r123_sqrt = sqrt(r123);
    let rs = (t121 * t121) * r23 / (vec3<f32>(1.0) - r123);

    let c0 = vec3<f32>(r12) + rs;
    var i = c0;

    var cm = rs - vec3<f32>(t121);
    cm = cm * r123_sqrt;
    let sm1 = 2.0 * eval_sensitivity(opd, phi);
    i = i + cm * sm1;

    cm = cm * r123_sqrt;
    let sm2 = 2.0 * eval_sensitivity(2.0 * opd, 2.0 * phi);
    i = i + cm * sm2;

    return max(i, vec3<f32>(0.0));
}

fn schlick_to_f0(fresnel: vec3<f32>, f90: vec3<f32>, cos_theta: f32) -> vec3<f32> {
    let x = clamp(1.0 - cos_theta, 0.0, 1.0);
    let x5 = clamp(x * x * x * x * x, 0.0, 0.9999);
    return (fresnel - f90 * x5) / (1.0 - x5);
}

fn getRangeAttenuation(range: f32, distance: f32) -> f32 {
    if range <= 0.0 {
        return 1.0;
    }
    let clamped_distance = max(distance, 0.01);
    return max(min(1.0 - pow(distance / range, 4.0), 1.0), 0.0) / (clamped_distance * clamped_distance);
}

fn getSpotAttenuation(pointToLight: vec3<f32>, spotDirection: vec3<f32>, outerConeCos: f32, innerConeCos: f32) -> f32 {
    let actualCos = dot(normalize(spotDirection), normalize(-pointToLight));
    if actualCos > outerConeCos {
        if actualCos < innerConeCos {
            return smoothstep(outerConeCos, innerConeCos, actualCos);
        }
        return 1.0;
    }
    return 0.0;
}

fn applyVolumeAttenuation(radiance: vec3<f32>, transmission_distance: f32, attenuation_color: vec3<f32>, attenuation_distance: f32) -> vec3<f32> {
    if attenuation_distance <= 0.0 {
        return radiance;
    }
    let attenuation_coefficient = -log(max(attenuation_color, vec3<f32>(0.0001))) / attenuation_distance;
    let transmittance = exp(-attenuation_coefficient * transmission_distance);
    return transmittance * radiance;
}

fn applyIorToRoughness(roughness: f32, ior: f32) -> f32 {
    return roughness * clamp(ior * 2.0 - 2.0, 0.0, 1.0);
}

fn specular_occlusion(n_dot_v: f32, ao: f32, roughness: f32) -> f32 {
    return clamp(pow(n_dot_v + ao, exp2(-16.0 * roughness - 1.0)) - 1.0 + ao, 0.0, 1.0);
}

fn box_project_reflection(dir: vec3<f32>, world_pos: vec3<f32>, box_min: vec3<f32>, box_max: vec3<f32>) -> vec4<f32> {
    let box_center = (box_min + box_max) * 0.5;
    let rbmax = (box_max - world_pos) / dir;
    let rbmin = (box_min - world_pos) / dir;
    let rbminmax = select(rbmin, rbmax, dir > vec3<f32>(0.0));
    let fa = min(min(rbminmax.x, rbminmax.y), rbminmax.z);
    let pos_on_box = world_pos + dir * fa;
    return vec4<f32>(pos_on_box - box_center, fa);
}

fn box_projected_roughness(perceptual_roughness: f32, dist_to_wall: f32, dist_to_center: f32) -> f32 {
    let hardened = clamp(dist_to_wall / max(dist_to_center, 0.001) * perceptual_roughness, 0.0, perceptual_roughness);
    return mix(hardened, perceptual_roughness, perceptual_roughness);
}

fn surface_reflectivity(
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    ior: f32,
    normal: vec3<f32>,
    world_pos: vec3<f32>,
    camera_position: vec3<f32>
) -> f32 {
    let view = normalize(camera_position - world_pos);
    let n_dot_v = clamp(dot(normal, view), 0.0, 1.0);
    let dielectric_f0 = pow((ior - 1.0) / (ior + 1.0), 2.0);
    let f0 = mix(vec3<f32>(dielectric_f0), albedo, metallic);
    let fresnel = f0 + (vec3<f32>(1.0) - f0) * pow(1.0 - n_dot_v, 5.0);
    let strength = dot(fresnel, vec3<f32>(0.2126, 0.7152, 0.0722));
    return saturate(strength * (1.0 - roughness));
}
