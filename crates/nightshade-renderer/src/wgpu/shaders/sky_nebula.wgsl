struct Uniform {
    proj: mat4x4<f32>,
    proj_inv: mat4x4<f32>,
    view: mat4x4<f32>,
    cam_pos: vec4<f32>,
    time: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
};

@group(0) @binding(0)
var<uniform> u: Uniform;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_dir: vec3<f32>,
};

@vertex
fn vs_sky(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let tmp1 = i32(vertex_index) / 2;
    let tmp2 = i32(vertex_index) & 1;
    let pos = vec4<f32>(
        f32(tmp1) * 4.0 - 1.0,
        f32(tmp2) * 4.0 - 1.0,
        0.0,
        1.0
    );
    let inv_model_view = transpose(mat3x3<f32>(u.view[0].xyz, u.view[1].xyz, u.view[2].xyz));
    let unprojected = u.proj_inv * pos;
    var result: VertexOutput;
    result.world_dir = inv_model_view * unprojected.xyz;
    result.position = pos;
    return result;
}

fn mod289_3(x: vec3<f32>) -> vec3<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn mod289_4(x: vec4<f32>) -> vec4<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn permute(x: vec4<f32>) -> vec4<f32> {
    return mod289_4(((x * 34.0) + 1.0) * x);
}

fn taylor_inv_sqrt(r: vec4<f32>) -> vec4<f32> {
    return 1.79284291400159 - 0.85373472095314 * r;
}

fn snoise(v: vec3<f32>) -> f32 {
    let C = vec2<f32>(1.0 / 6.0, 1.0 / 3.0);
    let D = vec4<f32>(0.0, 0.5, 1.0, 2.0);

    var i = floor(v + dot(v, vec3<f32>(C.y, C.y, C.y)));
    let x0 = v - i + dot(i, vec3<f32>(C.x, C.x, C.x));

    let g = step(x0.yzx, x0.xyz);
    let l = 1.0 - g;
    let i1 = min(g.xyz, l.zxy);
    let i2 = max(g.xyz, l.zxy);

    let x1 = x0 - i1 + C.x;
    let x2 = x0 - i2 + C.y;
    let x3 = x0 - D.yyy;

    i = mod289_3(i);
    let p = permute(permute(permute(
        i.z + vec4<f32>(0.0, i1.z, i2.z, 1.0))
        + i.y + vec4<f32>(0.0, i1.y, i2.y, 1.0))
        + i.x + vec4<f32>(0.0, i1.x, i2.x, 1.0));

    let n_ = 0.142857142857;
    let ns = n_ * D.wyz - D.xzx;

    let j = p - 49.0 * floor(p * ns.z * ns.z);

    let x_ = floor(j * ns.z);
    let y_ = floor(j - 7.0 * x_);

    let x = x_ * ns.x + ns.y;
    let y = y_ * ns.x + ns.y;
    let h = 1.0 - abs(x) - abs(y);

    let b0 = vec4<f32>(x.xy, y.xy);
    let b1 = vec4<f32>(x.zw, y.zw);

    let s0 = floor(b0) * 2.0 + 1.0;
    let s1 = floor(b1) * 2.0 + 1.0;
    let sh = -step(h, vec4<f32>(0.0, 0.0, 0.0, 0.0));

    let a0 = b0.xzyw + s0.xzyw * sh.xxyy;
    let a1 = b1.xzyw + s1.xzyw * sh.zzww;

    var p0 = vec3<f32>(a0.xy, h.x);
    var p1 = vec3<f32>(a0.zw, h.y);
    var p2 = vec3<f32>(a1.xy, h.z);
    var p3 = vec3<f32>(a1.zw, h.w);

    let norm = taylor_inv_sqrt(vec4<f32>(dot(p0, p0), dot(p1, p1), dot(p2, p2), dot(p3, p3)));
    p0 *= norm.x;
    p1 *= norm.y;
    p2 *= norm.z;
    p3 *= norm.w;

    var m = max(0.6 - vec4<f32>(dot(x0, x0), dot(x1, x1), dot(x2, x2), dot(x3, x3)), vec4<f32>(0.0));
    m = m * m;
    return 42.0 * dot(m * m, vec4<f32>(dot(p0, x0), dot(p1, x1), dot(p2, x2), dot(p3, x3)));
}

fn fbm(p: vec3<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;

    for (var index = 0; index < octaves; index++) {
        value += amplitude * (snoise(p * frequency) * 0.5 + 0.5);
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    return value;
}

fn hash_star(p: vec3<f32>) -> f32 {
    let q = fract(p * vec3<f32>(443.8975, 397.2973, 491.1871));
    let r = q + dot(q, q.yxz + 19.19);
    return fract((r.x + r.y) * r.z);
}

fn stars(dir: vec3<f32>, density: f32) -> f32 {
    let n = snoise(dir * density);
    let threshold = 0.75;
    if n > threshold {
        let intensity = (n - threshold) / (1.0 - threshold);
        let twinkle = 0.7 + 0.3 * sin(u.time * (2.0 + hash_star(floor(dir * density)) * 4.0));
        return pow(intensity, 3.0) * twinkle;
    }
    return 0.0;
}

@fragment
fn fs_sky(in: VertexOutput) -> @location(0) vec4<f32> {
    let dir = normalize(in.world_dir);
    let slow_time = u.time * 0.012;

    let nebula_pos = dir * 2.0;
    let drift = vec3<f32>(slow_time * 0.05, slow_time * 0.03, slow_time * 0.04);

    let density1 = fbm(nebula_pos * 1.0 + drift, 6);
    let density2 = fbm(nebula_pos * 1.5 + vec3<f32>(43.0, 17.0, 29.0) + drift * 0.7, 6);
    let density3 = fbm(nebula_pos * 2.2 + vec3<f32>(97.0, 61.0, 83.0) + drift * 0.5, 5);

    let purple = vec3<f32>(0.4, 0.1, 0.6);
    let blue = vec3<f32>(0.1, 0.2, 0.5);
    let pink = vec3<f32>(0.6, 0.15, 0.4);
    let cyan = vec3<f32>(0.1, 0.4, 0.5);
    let magenta = vec3<f32>(0.5, 0.1, 0.5);

    var nebula_color = vec3<f32>(0.0);

    let layer1 = smoothstep(0.3, 0.7, density1);
    nebula_color += mix(purple, blue, density2) * layer1 * 0.5;

    let layer2 = smoothstep(0.35, 0.75, density2);
    nebula_color += mix(pink, cyan, density1) * layer2 * 0.4;

    let layer3 = smoothstep(0.4, 0.8, density3);
    nebula_color += mix(magenta, purple, density2) * layer3 * 0.3;

    let core = smoothstep(0.6, 0.95, density1 * density2);
    nebula_color += vec3<f32>(0.8, 0.6, 0.9) * core * 0.3;

    let dust = fbm(nebula_pos * 3.5 + vec3<f32>(31.0, 47.0, 23.0), 4);
    nebula_color *= 1.0 - smoothstep(0.4, 0.7, dust) * 0.35;

    var star_brightness = 0.0;
    star_brightness += stars(dir, 50.0) * 0.8;
    star_brightness += stars(dir, 100.0) * 0.5;
    star_brightness += stars(dir, 200.0) * 0.3;
    star_brightness += stars(dir, 400.0) * 0.15;

    let nebula_mask = max(layer1, max(layer2, layer3));
    let star_dimming = 1.0 - nebula_mask * 0.5;

    let star_temp = snoise(dir * 80.0);
    var star_color = vec3<f32>(1.0);
    if star_temp < -0.3 {
        star_color = vec3<f32>(0.8, 0.85, 1.0);
    } else if star_temp > 0.3 {
        star_color = vec3<f32>(1.0, 0.95, 0.85);
    }

    let bright_star_noise = snoise(dir * 20.0);
    var bright_star = 0.0;
    if bright_star_noise > 0.85 {
        let intensity = (bright_star_noise - 0.85) / 0.15;
        let glow = pow(intensity, 2.0);
        let twinkle = 0.85 + 0.15 * sin(u.time * 1.5 + bright_star_noise * 10.0);
        bright_star = glow * twinkle * 2.0;
    }

    let background = vec3<f32>(0.01, 0.005, 0.02);

    var final_color = background;
    final_color += nebula_color;
    final_color += star_color * star_brightness * star_dimming;
    final_color += vec3<f32>(1.0, 0.95, 0.9) * bright_star;

    return vec4<f32>(final_color, 1.0);
}
