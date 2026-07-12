const PI: f32 = 3.14159265358979323846;
const NUM_SAMPLES: u32 = 1024u;
const LUT_SIZE: u32 = 256u;

@group(0) @binding(0)
var output_texture: texture_storage_2d<rgba16float, write>;

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

fn random(co: vec2<f32>) -> f32 {
    let a = 12.9898;
    let b = 78.233;
    let c = 43758.5453;
    let dt = dot(co, vec2<f32>(a, b));
    let sn = dt % 3.14;
    return fract(sin(sn) * c);
}

fn importance_sample_ggx(xi: vec2<f32>, roughness: f32, normal: vec3<f32>) -> vec3<f32> {
    let alpha = roughness * roughness;
    let phi = 2.0 * PI * xi.x;
    let cos_theta = sqrt((1.0 - xi.y) / (1.0 + (alpha * alpha - 1.0) * xi.y));
    let sin_theta = sqrt(1.0 - cos_theta * cos_theta);

    let h = vec3<f32>(sin_theta * cos(phi), sin_theta * sin(phi), cos_theta);

    let up = select(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 1.0), abs(normal.z) < 0.999);
    let tangent_x = normalize(cross(up, normal));
    let tangent_y = normalize(cross(normal, tangent_x));

    return normalize(tangent_x * h.x + tangent_y * h.y + normal * h.z);
}

fn g_schlicksmith_ggx(dot_nl: f32, dot_nv: f32, roughness: f32) -> f32 {
    let k = (roughness * roughness) / 2.0;
    let gl = dot_nl / (dot_nl * (1.0 - k) + k);
    let gv = dot_nv / (dot_nv * (1.0 - k) + k);
    return gl * gv;
}

fn v_ashikhmin(dot_nl: f32, dot_nv: f32) -> f32 {
    return clamp(1.0 / (4.0 * (dot_nl + dot_nv - dot_nl * dot_nv)), 0.0, 1.0);
}

fn d_charlie(sheen_roughness: f32, dot_nh: f32) -> f32 {
    let roughness = max(sheen_roughness, 0.000001);
    let inv_r = 1.0 / roughness;
    let cos2h = dot_nh * dot_nh;
    let sin2h = 1.0 - cos2h;
    return (2.0 + inv_r) * pow(sin2h, inv_r * 0.5) / (2.0 * PI);
}

fn importance_sample_charlie(xi: vec2<f32>, roughness: f32, normal: vec3<f32>) -> vec3<f32> {
    let alpha = roughness * roughness;
    let phi = 2.0 * PI * xi.x;
    let sin_theta = pow(xi.y, alpha / (2.0 * alpha + 1.0));
    let cos_theta = sqrt(1.0 - sin_theta * sin_theta);

    let h = vec3<f32>(sin_theta * cos(phi), sin_theta * sin(phi), cos_theta);

    let up = select(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 1.0), abs(normal.z) < 0.999);
    let tangent_x = normalize(cross(up, normal));
    let tangent_y = normalize(cross(normal, tangent_x));

    return normalize(tangent_x * h.x + tangent_y * h.y + normal * h.z);
}

fn integrate_brdf(nov: f32, roughness: f32) -> vec3<f32> {
    let n = vec3<f32>(0.0, 0.0, 1.0);
    let v = vec3<f32>(sqrt(1.0 - nov * nov), 0.0, nov);

    var lut = vec3<f32>(0.0);

    for (var index = 0u; index < NUM_SAMPLES; index = index + 1u) {
        let xi = hammersley_2d(index, NUM_SAMPLES);
        let h = importance_sample_ggx(xi, roughness, n);
        let l = 2.0 * dot(v, h) * h - v;

        let dot_nl = max(dot(n, l), 0.0);
        let dot_nv = max(dot(n, v), 0.0001);
        let dot_vh = max(dot(v, h), 0.0);
        let dot_nh = max(dot(h, n), 0.0001);

        if (dot_nl > 0.0) {
            let g = g_schlicksmith_ggx(dot_nl, dot_nv, roughness);
            let g_vis = (g * dot_vh) / (dot_nh * dot_nv);
            let fc = pow(1.0 - dot_vh, 5.0);
            lut.r = lut.r + (1.0 - fc) * g_vis;
            lut.g = lut.g + fc * g_vis;
        }
    }

    for (var index = 0u; index < NUM_SAMPLES; index = index + 1u) {
        let xi = hammersley_2d(index, NUM_SAMPLES);
        let h = importance_sample_charlie(xi, roughness, n);
        let l = 2.0 * dot(v, h) * h - v;

        let dot_nl = max(dot(n, l), 0.0);
        let dot_nv = max(dot(n, v), 0.0);
        let dot_vh = max(dot(v, h), 0.0);
        let dot_nh = max(dot(h, n), 0.0);

        if (dot_nl > 0.0) {
            let sheen_distribution = d_charlie(roughness, dot_nh);
            let sheen_visibility = v_ashikhmin(dot_nl, dot_nv);
            lut.b = lut.b + sheen_visibility * sheen_distribution * dot_nl * dot_vh;
        }
    }

    return lut / f32(NUM_SAMPLES);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let coords = global_id.xy;
    if (coords.x >= LUT_SIZE || coords.y >= LUT_SIZE) {
        return;
    }

    let uv = vec2<f32>(
        (f32(coords.x) + 0.5) / f32(LUT_SIZE),
        (f32(coords.y) + 0.5) / f32(LUT_SIZE)
    );

    let nov = uv.x;
    let roughness = uv.y;

    let result = integrate_brdf(nov, roughness);

    textureStore(output_texture, coords, vec4<f32>(result, 1.0));
}
