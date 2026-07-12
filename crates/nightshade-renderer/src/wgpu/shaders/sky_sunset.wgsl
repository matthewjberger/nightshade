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

@fragment
fn fs_sky(in: VertexOutput) -> @location(0) vec4<f32> {
    let dir = normalize(in.world_dir);
    let time = u.time;

    let sun_dir = normalize(vec3<f32>(0.5, 0.12, -0.8));

    let height = dir.y;
    let sun_dot = dot(dir, sun_dir);
    let sun_proximity = max(sun_dot, 0.0);

    let deep_orange = vec3<f32>(0.95, 0.35, 0.08);
    let warm_orange = vec3<f32>(1.0, 0.5, 0.15);
    let salmon = vec3<f32>(0.95, 0.55, 0.45);
    let pink = vec3<f32>(0.85, 0.45, 0.55);
    let purple = vec3<f32>(0.35, 0.2, 0.45);
    let dark_blue = vec3<f32>(0.1, 0.08, 0.25);

    var sky_color: vec3<f32>;

    var base_color: vec3<f32>;
    let abs_height = abs(height);

    if abs_height < 0.1 {
        base_color = mix(deep_orange, warm_orange, abs_height / 0.1);
    } else if abs_height < 0.25 {
        base_color = mix(warm_orange, salmon, (abs_height - 0.1) / 0.15);
    } else if abs_height < 0.45 {
        base_color = mix(salmon, pink, (abs_height - 0.25) / 0.2);
    } else if abs_height < 0.65 {
        base_color = mix(pink, purple, (abs_height - 0.45) / 0.2);
    } else {
        base_color = mix(purple, dark_blue, (abs_height - 0.65) / 0.35);
    }

    let sun_warmth = pow(sun_proximity, 1.5) * 0.4;
    let sun_color_boost = mix(vec3<f32>(1.0, 0.6, 0.3), vec3<f32>(1.0, 0.8, 0.6), abs_height);
    sky_color = mix(base_color, base_color + sun_color_boost * 0.3, sun_warmth);

    if height < 0.0 {
        let ground_darken = 1.0 - smoothstep(-0.4, 0.0, height);
        sky_color *= mix(1.0, 0.15, ground_darken);
        sky_color = mix(sky_color, vec3<f32>(0.01, 0.008, 0.015), 1.0 - smoothstep(-0.8, -0.3, height));
    }

    let sun_angular_radius = 0.9995;
    let sun_core = smoothstep(sun_angular_radius - 0.0003, sun_angular_radius, sun_dot);
    let sun_limb = smoothstep(sun_angular_radius - 0.001, sun_angular_radius - 0.0003, sun_dot) * 0.3;
    let sun_glow_tight = pow(sun_proximity, 32.0) * 0.6;
    let sun_glow_medium = pow(sun_proximity, 8.0) * 0.35;
    let sun_glow_wide = pow(sun_proximity, 2.5) * 0.2;

    let sun_color_core = vec3<f32>(1.0, 0.98, 0.9);
    let sun_color_limb = vec3<f32>(1.0, 0.8, 0.4);
    let sun_color_glow = vec3<f32>(1.0, 0.6, 0.25);

    sky_color += sun_color_core * sun_core;
    sky_color += sun_color_limb * sun_limb;
    sky_color += sun_color_glow * sun_glow_tight;
    sky_color += sun_color_glow * sun_glow_medium;
    sky_color += deep_orange * sun_glow_wide;

    if height > 0.0 && height < 0.6 {
        let cloud_dir = dir + vec3<f32>(time * 0.008, 0.0, time * 0.004);
        let cloud_pos = cloud_dir * 3.0;

        let cloud_noise = fbm(cloud_pos * 0.8, 5);
        let cloud_detail = fbm(cloud_pos * 2.5, 3) * 0.4;
        let cloud_density = cloud_noise + cloud_detail * 0.5;
        let cloud_shape = smoothstep(0.4, 0.7, cloud_density);

        let height_fade_in = smoothstep(0.0, 0.15, height);
        let height_fade_out = 1.0 - smoothstep(0.35, 0.6, height);
        let cloud_amount = cloud_shape * height_fade_in * height_fade_out;

        let cloud_sun_dot = max(dot(dir, sun_dir), 0.0);
        let cloud_lit = pow(cloud_sun_dot, 1.5);

        let cloud_shadow = vec3<f32>(0.25, 0.12, 0.18);
        let cloud_mid = vec3<f32>(0.6, 0.35, 0.3);
        let cloud_bright = vec3<f32>(1.0, 0.65, 0.4);
        let cloud_rim = vec3<f32>(1.0, 0.85, 0.6);

        var cloud_color = mix(cloud_shadow, cloud_mid, smoothstep(0.0, 0.3, cloud_lit));
        cloud_color = mix(cloud_color, cloud_bright, smoothstep(0.3, 0.7, cloud_lit));
        cloud_color = mix(cloud_color, cloud_rim, smoothstep(0.7, 1.0, cloud_lit) * 0.5);

        let edge_highlight = smoothstep(0.5, 0.7, cloud_density) - smoothstep(0.7, 0.9, cloud_density);
        cloud_color += cloud_rim * edge_highlight * cloud_lit * 0.3;

        sky_color = mix(sky_color, cloud_color, cloud_amount * 0.9);
    }

    if height > 0.5 {
        let star_visibility = smoothstep(0.5, 0.8, height);
        let star_noise = snoise(dir * 180.0);
        if star_noise > 0.82 {
            let star_intensity = (star_noise - 0.82) / 0.18;
            let twinkle = 0.75 + 0.25 * sin(time * 2.0 + star_noise * 50.0);
            sky_color += vec3<f32>(1.0, 0.98, 0.95) * pow(star_intensity, 2.5) * twinkle * star_visibility * 0.5;
        }
    }

    return vec4<f32>(sky_color, 1.0);
}
