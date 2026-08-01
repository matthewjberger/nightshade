const PI: f32 = 3.14159265358979323846;

const DISTRIBUTION_LAMBERTIAN: u32 = 0u;
const DISTRIBUTION_GGX: u32 = 1u;

struct FilterParams {
    face: u32,
    output_size: u32,
    roughness: f32,
    sample_count: u32,
    source_width: u32,
    source_height: u32,
    distribution: u32,
    source_mip_count: u32,
}

@group(0) @binding(0)
var source_cubemap: texture_cube<f32>;

@group(0) @binding(1)
var source_sampler: sampler;

@group(0) @binding(2)
var output_texture: texture_storage_2d_array<rgba16float, write>;

@group(0) @binding(3)
var<uniform> params: FilterParams;

@group(0) @binding(4)
var<storage, read_write> sh_coeffs: array<vec4<f32>, 9>;

const SH_FACE_SAMPLES: u32 = 32u;

fn sh_basis(d: vec3<f32>) -> array<f32, 9> {
    var b: array<f32, 9>;
    b[0] = 0.282095;
    b[1] = 0.488603 * d.y;
    b[2] = 0.488603 * d.z;
    b[3] = 0.488603 * d.x;
    b[4] = 1.092548 * d.x * d.y;
    b[5] = 1.092548 * d.y * d.z;
    b[6] = 0.315392 * (3.0 * d.z * d.z - 1.0);
    b[7] = 1.092548 * d.x * d.z;
    b[8] = 0.546274 * (d.x * d.x - d.y * d.y);
    return b;
}

fn sh_area_element(x: f32, y: f32) -> f32 {
    return atan2(x * y, sqrt(x * x + y * y + 1.0));
}

fn sh_texel_solid_angle(u: f32, v: f32, half_texel: f32) -> f32 {
    let x0 = u - half_texel;
    let x1 = u + half_texel;
    let y0 = v - half_texel;
    let y1 = v + half_texel;
    return sh_area_element(x1, y1) - sh_area_element(x0, y1) - sh_area_element(x1, y0) + sh_area_element(x0, y0);
}

fn sh_irradiance(n: vec3<f32>) -> vec3<f32> {
    let b = sh_basis(n);
    let a1 = 2.0 / 3.0;
    let a2 = 1.0 / 4.0;
    var e = sh_coeffs[0].rgb * b[0];
    e = e + sh_coeffs[1].rgb * b[1] * a1;
    e = e + sh_coeffs[2].rgb * b[2] * a1;
    e = e + sh_coeffs[3].rgb * b[3] * a1;
    e = e + sh_coeffs[4].rgb * b[4] * a2;
    e = e + sh_coeffs[5].rgb * b[5] * a2;
    e = e + sh_coeffs[6].rgb * b[6] * a2;
    e = e + sh_coeffs[7].rgb * b[7] * a2;
    e = e + sh_coeffs[8].rgb * b[8] * a2;
    return max(e, vec3<f32>(0.0));
}

@compute @workgroup_size(1, 1, 1)
fn project_sh() {
    var acc: array<vec3<f32>, 9>;
    for (var i = 0u; i < 9u; i = i + 1u) {
        acc[i] = vec3<f32>(0.0);
    }

    let face_texels = SH_FACE_SAMPLES * SH_FACE_SAMPLES;
    let total_texels = face_texels * 6u;
    let half_texel = 1.0 / f32(SH_FACE_SAMPLES);
    let lod = max(log2(f32(params.source_width) / f32(SH_FACE_SAMPLES)), 0.0);

    for (var t = 0u; t < total_texels; t = t + 1u) {
        let face = t / face_texels;
        let within = t % face_texels;
        let px = within % SH_FACE_SAMPLES;
        let py = within / SH_FACE_SAMPLES;
        let uv = vec2<f32>(
            (f32(px) + 0.5) / f32(SH_FACE_SAMPLES),
            (f32(py) + 0.5) / f32(SH_FACE_SAMPLES),
        );
        let dir = cube_to_world(face, uv);
        let u = 2.0 * uv.x - 1.0;
        let v = 2.0 * uv.y - 1.0;
        let solid_angle = sh_texel_solid_angle(u, v, half_texel);
        let radiance = min(textureSampleLevel(source_cubemap, source_sampler, dir, lod).rgb, vec3<f32>(10.0));
        let b = sh_basis(dir);
        for (var i = 0u; i < 9u; i = i + 1u) {
            acc[i] = acc[i] + radiance * b[i] * solid_angle;
        }
    }

    for (var i = 0u; i < 9u; i = i + 1u) {
        sh_coeffs[i] = vec4<f32>(acc[i], 0.0);
    }
}

fn hammersley_2d(index: u32, num_samples: u32) -> vec2<f32> {
    var bits = index;
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    let rdi = f32(bits) * 2.3283064365386963e-10;
    return vec2<f32>(f32(index) / f32(num_samples), rdi);
}

fn cube_to_world(face: u32, uv: vec2<f32>) -> vec3<f32> {
    let x = 2.0 * uv.x - 1.0;
    let y = 2.0 * uv.y - 1.0;

    var dir: vec3<f32>;
    switch face {
        case 0u: {
            dir = vec3<f32>(1.0, -y, -x);
        }
        case 1u: {
            dir = vec3<f32>(-1.0, -y, x);
        }
        case 2u: {
            dir = vec3<f32>(x, 1.0, y);
        }
        case 3u: {
            dir = vec3<f32>(x, -1.0, -y);
        }
        case 4u: {
            dir = vec3<f32>(x, -y, 1.0);
        }
        default: {
            dir = vec3<f32>(-x, -y, -1.0);
        }
    }
    return normalize(dir);
}

fn orthonormal_basis(n: vec3<f32>) -> mat3x3<f32> {
    let sign = select(-1.0, 1.0, n.z >= 0.0);
    let a = -1.0 / (sign + n.z);
    let b = n.x * n.y * a;
    let tangent = vec3<f32>(1.0 + sign * n.x * n.x * a, sign * b, -sign * n.x);
    let bitangent = vec3<f32>(b, sign + n.y * n.y * a, -n.y);
    return mat3x3<f32>(tangent, bitangent, n);
}

fn pcg_hash(input: u32) -> u32 {
    let state = input * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn rand_f32(seed: u32) -> f32 {
    return f32(pcg_hash(seed)) / 4294967295.0;
}

fn rand_vec2(seed: u32) -> vec2<f32> {
    return vec2<f32>(rand_f32(seed), rand_f32(seed + 1u));
}

fn D_GGX_prefilter(dotNH: f32, roughness: f32) -> f32 {
    let alpha = roughness * roughness;
    let alpha2 = alpha * alpha;
    let denom = dotNH * dotNH * (alpha2 - 1.0) + 1.0;
    return alpha2 / (PI * denom * denom);
}

fn importance_sample_ggx(xi: vec2<f32>, roughness: f32, normal: vec3<f32>) -> vec3<f32> {
    let alpha = roughness * roughness;
    let phi = 2.0 * PI * xi.x;
    let cos_theta = sqrt((1.0 - xi.y) / (1.0 + (alpha * alpha - 1.0) * xi.y));
    let sin_theta = sqrt(1.0 - cos_theta * cos_theta);

    let local_dir = vec3<f32>(
        sin_theta * cos(phi),
        sin_theta * sin(phi),
        cos_theta
    );

    let tbn = orthonormal_basis(normal);
    return normalize(tbn * local_dir);
}

fn prefilter_env_map(normal: vec3<f32>, roughness: f32, pixel_seed: u32) -> vec3<f32> {
    if roughness == 0.0 {
        let mirror = textureSampleLevel(source_cubemap, source_sampler, normal, 0.0).rgb;
        return min(mirror, vec3<f32>(4.0));
    }

    let n = normal;
    let v = normal;

    var color = vec3<f32>(0.0);
    var total_weight = 0.0;
    let num_samples = params.sample_count;
    let clamped_roughness = max(roughness, 0.001);
    let random_offset = rand_vec2(pixel_seed);

    let env_map_dim = f32(params.source_width);

    for (var sample_index = 0u; sample_index < num_samples; sample_index = sample_index + 1u) {
        var xi = hammersley_2d(sample_index, num_samples);
        xi = fract(xi + random_offset);
        let h = importance_sample_ggx(xi, clamped_roughness, n);
        let l = normalize(2.0 * dot(v, h) * h - v);

        let dot_nl = clamp(dot(n, l), 0.0, 1.0);
        if (dot_nl > 0.0) {
            let dot_nh = clamp(dot(n, h), 0.0, 1.0);
            let dot_vh = clamp(dot(v, h), 0.0, 1.0);

            let pdf = D_GGX_prefilter(dot_nh, clamped_roughness) * dot_nh / (4.0 * dot_vh) + 0.0001;
            let omega_s = 1.0 / (f32(num_samples) * pdf);
            let omega_p = 4.0 * PI / (6.0 * env_map_dim * env_map_dim);
            var mip_level = 0.0;
            if params.roughness > 0.0 {
                mip_level = max(0.5 * log2(omega_s / omega_p) + 1.0, 0.0);
            }

            let sample_color = textureSampleLevel(source_cubemap, source_sampler, l, mip_level).rgb;
            color = color + min(sample_color, vec3<f32>(4.0)) * dot_nl;
            total_weight = total_weight + dot_nl;
        }
    }

    if (total_weight > 0.0) {
        return color / total_weight;
    }
    return textureSampleLevel(source_cubemap, source_sampler, normal, 0.0).rgb;
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let coords = global_id.xy;
    let face = global_id.z;

    if (coords.x >= params.output_size || coords.y >= params.output_size) {
        return;
    }

    let uv = vec2<f32>(
        (f32(coords.x) + 0.5) / f32(params.output_size),
        (f32(coords.y) + 0.5) / f32(params.output_size)
    );

    let direction = cube_to_world(face, uv);
    let pixel_seed = (coords.x * 2131358057u) ^ (coords.y * 3416869721u) ^ (face * 1199786941u);

    var color: vec3<f32>;
    if (params.distribution == DISTRIBUTION_LAMBERTIAN) {
        color = sh_irradiance(direction);
    } else {
        color = prefilter_env_map(direction, params.roughness, pixel_seed);
    }

    textureStore(output_texture, coords, face, vec4<f32>(color, 1.0));
}
