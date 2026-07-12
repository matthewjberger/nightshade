struct ProceduralUniform {
    time: f32,
    atmosphere_type: u32,
    _padding0: u32,
    _padding1: u32,
};

@group(0) @binding(0)
var<uniform> params: ProceduralUniform;

@group(0) @binding(1)
var output_texture: texture_storage_2d_array<rgba16float, write>;

const PI: f32 = 3.141592653589793;
const FACE_SIZE: u32 = 1024u;

const ATMOSPHERE_SKY: u32 = 0u;
const ATMOSPHERE_CLOUDY_SKY: u32 = 1u;
const ATMOSPHERE_SPACE: u32 = 2u;
const ATMOSPHERE_NEBULA: u32 = 3u;
const ATMOSPHERE_SUNSET: u32 = 4u;
const ATMOSPHERE_DAY_NIGHT: u32 = 5u;

fn cube_to_world(face: u32, uv: vec2<f32>) -> vec3<f32> {
    var dir: vec3<f32>;
    let x = 2.0 * uv.x - 1.0;
    let y = 2.0 * uv.y - 1.0;

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

fn fbm_detailed(p: vec3<f32>, octaves: i32, lacunarity: f32, gain: f32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;

    for (var index = 0; index < octaves; index++) {
        value += amplitude * (snoise(p * frequency) * 0.5 + 0.5);
        amplitude *= gain;
        frequency *= lacunarity;
    }

    return value;
}

fn hash(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn hash2(p: vec2<f32>) -> vec2<f32> {
    let k = vec2<f32>(0.3183099, 0.3678794);
    var n = p * k + k.yx;
    return fract(16.0 * k * fract(n.x * n.y * (n.x + n.y))) * 2.0 - 1.0;
}

fn hash3(p: vec2<f32>) -> vec3<f32> {
    var p3 = fract(vec3<f32>(p.x, p.y, p.x) * vec3<f32>(0.1031, 0.1030, 0.0973));
    p3 += dot(p3, p3.yxz + 33.33);
    return fract((p3.xxy + p3.yxx) * p3.zyx);
}

fn hash_star(p: vec3<f32>) -> f32 {
    let q = fract(p * vec3<f32>(443.8975, 397.2973, 491.1871));
    let r = q + dot(q, q.yxz + 19.19);
    return fract((r.x + r.y) * r.z);
}

fn dir_to_uv(dir: vec3<f32>) -> vec2<f32> {
    let phi = atan2(dir.z, dir.x);
    let theta = asin(clamp(dir.y, -1.0, 1.0));
    return vec2<f32>(phi / 6.283185 + 0.5, theta / 3.141593 + 0.5);
}

fn sky_color(dir: vec3<f32>, time: f32) -> vec3<f32> {
    let sky_top_color = vec3<f32>(0.385, 0.454, 0.55);
    let sky_horizon_color = vec3<f32>(0.646, 0.656, 0.67);
    let ground_horizon_color = vec3<f32>(0.646, 0.656, 0.67);
    let ground_bottom_color = vec3<f32>(0.2, 0.169, 0.133);

    let height = dir.y;
    let sky_curve = 0.15;
    let ground_curve = 0.02;

    var color: vec3<f32>;
    if height > 0.0 {
        let t = 1.0 - pow(1.0 - height, 1.0 / sky_curve);
        color = mix(sky_horizon_color, sky_top_color, clamp(t, 0.0, 1.0));
    } else {
        let t = 1.0 - pow(1.0 + height, 1.0 / ground_curve);
        color = mix(ground_horizon_color, ground_bottom_color, clamp(t, 0.0, 1.0));
    }

    color = color * 1.3;

    let sun_direction = normalize(vec3<f32>(0.0, 0.5, -1.0));
    let sun_angle = acos(dot(dir, sun_direction));
    let sun_disk = 1.0 - smoothstep(0.0, 0.02, sun_angle);
    let sun_color = vec3<f32>(1.0, 0.95, 0.8);
    color = mix(color, sun_color, sun_disk * 0.5);

    return color;
}

fn cloudy_sky_color(dir: vec3<f32>, time: f32) -> vec3<f32> {
    let height = dir.y;

    let sky_top_color = vec3<f32>(0.25, 0.4, 0.7);
    let sky_horizon_color = vec3<f32>(0.4, 0.55, 0.75);
    let sky_curve = 0.3;

    var color: vec3<f32>;
    if height > 0.0 {
        let t = 1.0 - pow(1.0 - height, 1.0 / sky_curve);
        color = mix(sky_horizon_color, sky_top_color, clamp(t, 0.0, 1.0));
    } else {
        color = sky_horizon_color;
    }

    let sun_direction = normalize(vec3<f32>(0.0, 0.5, -1.0));
    let sun_angle = acos(dot(dir, sun_direction));
    let sun_disk = 1.0 - smoothstep(0.0, 0.02, sun_angle);
    let sun_color = vec3<f32>(1.0, 0.95, 0.8);
    color = mix(color, sun_color, sun_disk * 0.5);

    if height > 0.0 {
        let cloud_drift = vec3<f32>(time * 0.012, 0.0, time * 0.006);
        let cloud_pos = dir * 2.5 + cloud_drift;

        let cloud_base = fbm(cloud_pos * 0.8, 5);
        let cloud_detail = fbm(cloud_pos * 2.0, 4) * 0.5;
        let cloud_density = cloud_base + cloud_detail * 0.4;

        let cloud_coverage = 0.62;
        let cloud_shape = smoothstep(cloud_coverage, cloud_coverage + 0.2, cloud_density);

        let height_fade = smoothstep(0.15, 0.45, height) * (1.0 - smoothstep(0.7, 0.95, height));
        let cloud_amount = cloud_shape * height_fade;

        let cloud_sun_lit = max(dot(dir, sun_direction), 0.0);
        let light_factor = pow(cloud_sun_lit, 1.2);

        let cloud_shadow = vec3<f32>(0.65, 0.7, 0.78);
        let cloud_lit = vec3<f32>(1.0, 0.99, 0.97);
        let cloud_bright = vec3<f32>(1.0, 1.0, 0.98);

        var cloud_color = mix(cloud_shadow, cloud_lit, smoothstep(0.0, 0.5, light_factor));
        cloud_color = mix(cloud_color, cloud_bright, smoothstep(0.5, 1.0, light_factor) * 0.5);

        let cloud_thickness = smoothstep(cloud_coverage, cloud_coverage + 0.35, cloud_density);
        cloud_color = mix(cloud_color, cloud_shadow, cloud_thickness * 0.3);

        color = mix(color, cloud_color, cloud_amount * 0.85);
    }

    return color;
}

fn star_layer_space(uv: vec2<f32>, scale: f32, seed: f32, time: f32) -> f32 {
    let grid_uv = uv * scale;
    let grid_id = floor(grid_uv);
    let grid_fract = fract(grid_uv) - 0.5;

    let rand = hash(grid_id + seed);

    if rand > 0.97 {
        let offset = hash2(grid_id + seed + 100.0) * 0.4;
        let dist = length(grid_fract - offset);

        let star_size = 0.02 + hash(grid_id + seed + 200.0) * 0.03;
        let brightness = smoothstep(star_size, 0.0, dist);

        let twinkle_speed = 1.0 + hash(grid_id + seed + 300.0) * 3.0;
        let twinkle_phase = hash(grid_id + seed + 400.0) * 6.283;
        let twinkle = 0.7 + 0.3 * sin(time * twinkle_speed + twinkle_phase);

        return brightness * twinkle;
    }

    return 0.0;
}

fn space_color(dir: vec3<f32>, time: f32) -> vec3<f32> {
    let uv = dir_to_uv(dir);

    var stars = 0.0;
    stars += star_layer_space(uv, 50.0, 0.0, time);
    stars += star_layer_space(uv, 100.0, 10.0, time);
    stars += star_layer_space(uv, 200.0, 20.0, time) * 0.7;
    stars += star_layer_space(uv, 400.0, 30.0, time) * 0.4;

    let star_hue = hash(floor(uv * 100.0));
    var star_color = vec3<f32>(1.0);
    if star_hue < 0.3 {
        star_color = vec3<f32>(0.8, 0.85, 1.0);
    } else if star_hue > 0.7 {
        star_color = vec3<f32>(1.0, 0.95, 0.85);
    }

    return star_color * stars;
}

fn stars_nebula(dir: vec3<f32>, density: f32, time: f32) -> f32 {
    let n = snoise(dir * density);
    let threshold = 0.75;
    if n > threshold {
        let intensity = (n - threshold) / (1.0 - threshold);
        let twinkle = 0.7 + 0.3 * sin(time * (2.0 + hash_star(floor(dir * density)) * 4.0));
        return pow(intensity, 3.0) * twinkle;
    }
    return 0.0;
}

fn nebula_color(dir: vec3<f32>, time: f32) -> vec3<f32> {
    let slow_time = time * 0.012;

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

    var color = vec3<f32>(0.0);

    let layer1 = smoothstep(0.3, 0.7, density1);
    color += mix(purple, blue, density2) * layer1 * 0.5;

    let layer2 = smoothstep(0.35, 0.75, density2);
    color += mix(pink, cyan, density1) * layer2 * 0.4;

    let layer3 = smoothstep(0.4, 0.8, density3);
    color += mix(magenta, purple, density2) * layer3 * 0.3;

    let core = smoothstep(0.6, 0.95, density1 * density2);
    color += vec3<f32>(0.8, 0.6, 0.9) * core * 0.3;

    let dust = fbm(nebula_pos * 3.5 + vec3<f32>(31.0, 47.0, 23.0), 4);
    color *= 1.0 - smoothstep(0.4, 0.7, dust) * 0.35;

    var star_brightness = 0.0;
    star_brightness += stars_nebula(dir, 50.0, time) * 0.8;
    star_brightness += stars_nebula(dir, 100.0, time) * 0.5;
    star_brightness += stars_nebula(dir, 200.0, time) * 0.3;
    star_brightness += stars_nebula(dir, 400.0, time) * 0.15;

    let nebula_mask = max(layer1, max(layer2, layer3));
    let star_dimming = 1.0 - nebula_mask * 0.5;

    let star_temp = snoise(dir * 80.0);
    var star_col = vec3<f32>(1.0);
    if star_temp < -0.3 {
        star_col = vec3<f32>(0.8, 0.85, 1.0);
    } else if star_temp > 0.3 {
        star_col = vec3<f32>(1.0, 0.95, 0.85);
    }

    let bright_star_noise = snoise(dir * 20.0);
    var bright_star = 0.0;
    if bright_star_noise > 0.85 {
        let intensity = (bright_star_noise - 0.85) / 0.15;
        let glow = pow(intensity, 2.0);
        let twinkle = 0.85 + 0.15 * sin(time * 1.5 + bright_star_noise * 10.0);
        bright_star = glow * twinkle * 2.0;
    }

    let background = vec3<f32>(0.01, 0.005, 0.02);

    var final_color = background;
    final_color += color;
    final_color += star_col * star_brightness * star_dimming;
    final_color += vec3<f32>(1.0, 0.95, 0.9) * bright_star;

    return final_color;
}

fn sunset_color(dir: vec3<f32>, time: f32) -> vec3<f32> {
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

    var color: vec3<f32>;

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
    color = mix(base_color, base_color + sun_color_boost * 0.3, sun_warmth);

    if height < 0.0 {
        let ground_darken = 1.0 - smoothstep(-0.4, 0.0, height);
        color *= mix(1.0, 0.15, ground_darken);
        color = mix(color, vec3<f32>(0.01, 0.008, 0.015), 1.0 - smoothstep(-0.8, -0.3, height));
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

    color += sun_color_core * sun_core;
    color += sun_color_limb * sun_limb;
    color += sun_color_glow * sun_glow_tight;
    color += sun_color_glow * sun_glow_medium;
    color += deep_orange * sun_glow_wide;

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

        color = mix(color, cloud_color, cloud_amount * 0.9);
    }

    if height > 0.5 {
        let star_visibility = smoothstep(0.5, 0.8, height);
        let star_noise = snoise(dir * 180.0);
        if star_noise > 0.82 {
            let star_intensity = (star_noise - 0.82) / 0.18;
            let twinkle = 0.75 + 0.25 * sin(time * 2.0 + star_noise * 50.0);
            color += vec3<f32>(1.0, 0.98, 0.95) * pow(star_intensity, 2.5) * twinkle * star_visibility * 0.5;
        }
    }

    return color;
}

fn stellar_color(temperature: f32) -> vec3<f32> {
    var color: vec3<f32>;
    if temperature < 0.15 {
        color = vec3<f32>(1.0, 0.7, 0.5);
    } else if temperature < 0.3 {
        color = vec3<f32>(1.0, 0.85, 0.7);
    } else if temperature < 0.5 {
        color = vec3<f32>(1.0, 0.97, 0.92);
    } else if temperature < 0.7 {
        color = vec3<f32>(0.95, 0.97, 1.0);
    } else if temperature < 0.85 {
        color = vec3<f32>(0.8, 0.88, 1.0);
    } else {
        color = vec3<f32>(0.7, 0.8, 1.0);
    }
    return color;
}

fn star_field(uv: vec2<f32>, scale: f32, seed: f32, time: f32, threshold: f32) -> vec4<f32> {
    let grid_uv = uv * scale;
    let grid_id = floor(grid_uv);
    let grid_fract = fract(grid_uv) - 0.5;

    let rand_vals = hash3(grid_id + seed);
    let star_prob = rand_vals.x;

    if star_prob > threshold {
        let offset = hash2(grid_id + seed + 100.0) * 0.45;
        let dist = length(grid_fract - offset);

        let magnitude = rand_vals.y;
        let base_size = 0.008 + magnitude * magnitude * 0.025;

        let temperature = rand_vals.z;
        let star_col = stellar_color(temperature);

        let core_brightness = 1.0 - smoothstep(base_size * 0.1, base_size, dist);
        let glow_brightness = (1.0 - smoothstep(0.0, base_size * 3.0, dist)) * 0.3;
        let total_brightness = core_brightness + glow_brightness;

        let twinkle_speed = 0.8 + hash(grid_id + seed + 300.0) * 2.5;
        let twinkle_phase = hash(grid_id + seed + 400.0) * 6.283;
        let twinkle_amount = 0.15 + hash(grid_id + seed + 500.0) * 0.2;
        let twinkle = 1.0 - twinkle_amount + twinkle_amount * (0.5 + 0.5 * sin(time * twinkle_speed + twinkle_phase));

        let final_brightness = total_brightness * twinkle * (0.3 + magnitude * 0.7);

        return vec4<f32>(star_col, final_brightness);
    }

    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}

fn bright_star_with_spikes(uv: vec2<f32>, scale: f32, seed: f32, time: f32) -> vec4<f32> {
    let grid_uv = uv * scale;
    let grid_id = floor(grid_uv);
    let grid_fract = fract(grid_uv) - 0.5;

    let rand_vals = hash3(grid_id + seed);

    if rand_vals.x > 0.992 {
        let offset = hash2(grid_id + seed + 100.0) * 0.35;
        let pos = grid_fract - offset;
        let dist = length(pos);

        let temperature = rand_vals.z;
        let star_col = stellar_color(temperature);

        let core = 1.0 - smoothstep(0.0, 0.025, dist);
        let inner_glow = (1.0 - smoothstep(0.0, 0.08, dist)) * 0.5;
        let outer_glow = (1.0 - smoothstep(0.0, 0.15, dist)) * 0.15;

        let angle = atan2(pos.y, pos.x);
        let spike_pattern = pow(abs(cos(angle * 2.0)), 32.0) + pow(abs(sin(angle * 2.0)), 32.0);
        let spike_falloff = 1.0 - smoothstep(0.01, 0.12, dist);
        let spikes = spike_pattern * spike_falloff * 0.6;

        let twinkle = 0.85 + 0.15 * sin(time * 1.5 + rand_vals.y * 6.283);

        let total = (core + inner_glow + outer_glow + spikes) * twinkle;

        return vec4<f32>(star_col, total);
    }

    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}

fn milky_way(dir: vec3<f32>) -> vec3<f32> {
    let galactic_pole = normalize(vec3<f32>(0.2, 0.9, 0.1));
    let galactic_dist = abs(dot(dir, galactic_pole));

    let band_width = 1.0 - smoothstep(0.0, 0.35, galactic_dist);

    let galactic_center = normalize(vec3<f32>(-0.8, -0.2, 0.5));
    let center_brightness = pow(max(dot(dir, galactic_center), 0.0), 3.0);

    var detail = 0.0;
    detail += snoise(dir * 8.0) * 0.5;
    detail += snoise(dir * 16.0) * 0.25;
    detail += snoise(dir * 32.0) * 0.125;
    detail = detail * 0.5 + 0.5;

    let dust_lanes = snoise(dir * 12.0 + vec3<f32>(0.0, 5.0, 0.0));
    let dust = smoothstep(0.3, 0.6, dust_lanes) * 0.4;

    let base_brightness = band_width * (0.4 + center_brightness * 0.6) * detail;
    let final_brightness = max(base_brightness - dust * band_width, 0.0);

    let warm_color = vec3<f32>(0.06, 0.05, 0.045);
    let cool_color = vec3<f32>(0.04, 0.045, 0.06);
    let milky_color = mix(cool_color, warm_color, center_brightness);

    return milky_color * final_brightness;
}

fn moon_surface(local_pos: vec2<f32>, moon_dir: vec3<f32>) -> vec3<f32> {
    let maria_noise1 = snoise(vec3<f32>(local_pos * 3.0, 0.0));
    let maria_noise2 = snoise(vec3<f32>(local_pos * 1.5 + vec2<f32>(10.0, 5.0), 0.0));
    let maria = smoothstep(0.1, 0.5, maria_noise1 * 0.6 + maria_noise2 * 0.4);

    var crater_detail = 0.0;
    crater_detail += snoise(vec3<f32>(local_pos * 25.0, 1.0)) * 0.3;
    crater_detail += snoise(vec3<f32>(local_pos * 50.0, 2.0)) * 0.15;
    crater_detail += snoise(vec3<f32>(local_pos * 100.0, 3.0)) * 0.08;

    let highland_color = vec3<f32>(0.85, 0.83, 0.78);
    let maria_color = vec3<f32>(0.45, 0.43, 0.42);

    var surface_color = mix(highland_color, maria_color, maria);
    surface_color *= 1.0 + crater_detail * 0.3;

    return surface_color;
}

fn day_night_get_sun_direction(hour: f32) -> vec3<f32> {
    let sun_angle = (hour - 6.0) / 12.0 * PI;
    return normalize(vec3<f32>(-cos(sun_angle), sin(sun_angle), -0.3));
}

fn day_night_get_moon_direction(hour: f32) -> vec3<f32> {
    var h = hour;
    if h < 12.0 {
        h += 24.0;
    }
    let moon_angle = (h - 18.0) / 12.0 * PI;
    return normalize(vec3<f32>(-cos(moon_angle), sin(moon_angle), 0.3));
}

fn day_night_sky_gradient(dir: vec3<f32>, hour: f32) -> vec3<f32> {
    let height = dir.y;

    let night_col = vec3<f32>(0.01, 0.01, 0.03);
    let dawn_horizon = vec3<f32>(0.8, 0.4, 0.15);
    let dawn_sky = vec3<f32>(0.2, 0.25, 0.5);
    let morning_sky = vec3<f32>(0.35, 0.5, 0.75);
    let midday_top = vec3<f32>(0.25, 0.45, 0.8);
    let midday_horizon = vec3<f32>(0.55, 0.65, 0.8);
    let afternoon_sky = vec3<f32>(0.35, 0.5, 0.7);
    let sunset_horizon = vec3<f32>(0.95, 0.35, 0.08);
    let sunset_upper = vec3<f32>(0.35, 0.2, 0.45);
    let dusk_col = vec3<f32>(0.1, 0.08, 0.2);

    var sky_top: vec3<f32>;
    var sky_horiz: vec3<f32>;

    if hour < 5.0 {
        sky_top = night_col;
        sky_horiz = night_col;
    } else if hour < 6.0 {
        let t = hour - 5.0;
        sky_top = mix(night_col, dawn_sky * 0.3, t);
        sky_horiz = mix(night_col, dawn_horizon * 0.3, t);
    } else if hour < 7.0 {
        let t = hour - 6.0;
        sky_top = mix(dawn_sky * 0.3, morning_sky * 0.7, t);
        sky_horiz = mix(dawn_horizon * 0.3, dawn_horizon, t);
    } else if hour < 9.0 {
        let t = (hour - 7.0) / 2.0;
        sky_top = mix(morning_sky * 0.7, midday_top, t);
        sky_horiz = mix(dawn_horizon, midday_horizon, t);
    } else if hour < 15.0 {
        sky_top = midday_top;
        sky_horiz = midday_horizon;
    } else if hour < 17.0 {
        let t = (hour - 15.0) / 2.0;
        sky_top = mix(midday_top, afternoon_sky, t);
        sky_horiz = mix(midday_horizon, vec3<f32>(0.7, 0.55, 0.45), t);
    } else if hour < 18.0 {
        let t = hour - 17.0;
        sky_top = mix(afternoon_sky, sunset_upper, t);
        sky_horiz = mix(vec3<f32>(0.7, 0.55, 0.45), sunset_horizon, t);
    } else if hour < 19.0 {
        let t = hour - 18.0;
        sky_top = mix(sunset_upper, dusk_col, t);
        sky_horiz = mix(sunset_horizon, dusk_col * 1.5, t);
    } else if hour < 20.0 {
        let t = hour - 19.0;
        sky_top = mix(dusk_col, night_col, t);
        sky_horiz = mix(dusk_col * 1.5, night_col, t);
    } else {
        sky_top = night_col;
        sky_horiz = night_col;
    }

    var color: vec3<f32>;
    if height >= 0.0 {
        let sky_curve = 0.15;
        let t = 1.0 - pow(1.0 - height, 1.0 / sky_curve);
        color = mix(sky_horiz, sky_top, clamp(t, 0.0, 1.0));
    } else {
        let ground_darken = 1.0 - smoothstep(-0.4, 0.0, height);
        let ground_color = mix(sky_horiz, night_col, ground_darken);
        color = ground_color * mix(1.0, 0.15, ground_darken);
    }

    return color;
}

fn day_night_color(dir: vec3<f32>, raw_hour: f32) -> vec3<f32> {
    let hour = raw_hour - floor(raw_hour / 24.0) * 24.0;
    var color = day_night_sky_gradient(dir, hour);
    let height = dir.y;
    let time = hour;

    if hour >= 5.5 && hour <= 18.5 {
        let sun_dir = day_night_get_sun_direction(hour);
        let sun_dot = dot(dir, sun_dir);
        let sun_proximity = max(sun_dot, 0.0);

        var sun_intensity = 1.0;
        if hour < 6.5 {
            sun_intensity = smoothstep(5.5, 6.5, hour);
        } else if hour > 17.5 {
            sun_intensity = 1.0 - smoothstep(17.5, 18.5, hour);
        }

        var sun_core_color = vec3<f32>(1.0, 0.98, 0.9);
        var sun_glow_col = vec3<f32>(1.0, 0.8, 0.5);

        if hour < 7.0 || hour > 17.0 {
            sun_core_color = vec3<f32>(1.0, 0.85, 0.6);
            sun_glow_col = vec3<f32>(1.0, 0.5, 0.2);
        }

        let sun_angular_radius = 0.9995;
        let sun_core = smoothstep(sun_angular_radius - 0.0003, sun_angular_radius, sun_dot);
        let sun_limb = smoothstep(sun_angular_radius - 0.001, sun_angular_radius - 0.0003, sun_dot) * 0.3;
        let sun_glow_tight = pow(sun_proximity, 32.0) * 0.6;
        let sun_glow_medium = pow(sun_proximity, 8.0) * 0.35;
        let sun_glow_wide = pow(sun_proximity, 2.5) * 0.2;

        color += sun_core_color * sun_core * sun_intensity;
        color += vec3<f32>(1.0, 0.8, 0.4) * sun_limb * sun_intensity;
        color += sun_glow_col * sun_glow_tight * sun_intensity;
        color += sun_glow_col * sun_glow_medium * sun_intensity;
        color += sun_glow_col * 0.5 * sun_glow_wide * sun_intensity;
    }

    if height > 0.0 && height < 0.6 {
        let sun_dir = day_night_get_sun_direction(clamp(hour, 6.0, 18.0));
        let cloud_pos = dir * 3.0;

        let cloud_noise = fbm(cloud_pos * 0.8, 5);
        let cloud_detail = fbm(cloud_pos * 2.5, 3) * 0.4;
        let cloud_density = cloud_noise + cloud_detail * 0.5;
        let cloud_shape = smoothstep(0.4, 0.7, cloud_density);

        let height_fade_in = smoothstep(0.0, 0.15, height);
        let height_fade_out = 1.0 - smoothstep(0.35, 0.6, height);
        let cloud_amount = cloud_shape * height_fade_in * height_fade_out;

        let cloud_sun_dot = max(dot(dir, sun_dir), 0.0);
        let cloud_lit = pow(cloud_sun_dot, 1.5);

        var cloud_shadow: vec3<f32>;
        var cloud_bright: vec3<f32>;
        var cloud_rim: vec3<f32>;

        if hour >= 17.0 && hour <= 18.0 {
            let sunset_t = hour - 17.0;
            cloud_shadow = mix(vec3<f32>(0.5, 0.5, 0.55), vec3<f32>(0.35, 0.25, 0.3), sunset_t);
            cloud_bright = mix(vec3<f32>(0.95, 0.95, 0.93), vec3<f32>(1.0, 0.65, 0.4), sunset_t);
            cloud_rim = mix(vec3<f32>(1.0, 1.0, 0.98), vec3<f32>(1.0, 0.85, 0.6), sunset_t);
        } else if hour >= 5.0 && hour <= 7.0 {
            let dawn_t = 1.0 - (hour - 5.0) / 2.0;
            cloud_shadow = mix(vec3<f32>(0.5, 0.5, 0.55), vec3<f32>(0.35, 0.25, 0.3), dawn_t);
            cloud_bright = mix(vec3<f32>(0.95, 0.95, 0.93), vec3<f32>(1.0, 0.55, 0.35), dawn_t);
            cloud_rim = mix(vec3<f32>(1.0, 1.0, 0.98), vec3<f32>(1.0, 0.75, 0.5), dawn_t);
        } else {
            cloud_shadow = vec3<f32>(0.5, 0.5, 0.55);
            cloud_bright = vec3<f32>(0.95, 0.95, 0.93);
            cloud_rim = vec3<f32>(1.0, 1.0, 0.98);
        }

        let cloud_mid = mix(cloud_shadow, cloud_bright, 0.4);
        var day_cloud_color = mix(cloud_shadow, cloud_mid, smoothstep(0.0, 0.3, cloud_lit));
        day_cloud_color = mix(day_cloud_color, cloud_bright, smoothstep(0.3, 0.7, cloud_lit));
        day_cloud_color = mix(day_cloud_color, cloud_rim, smoothstep(0.7, 1.0, cloud_lit) * 0.5);

        let edge_highlight = smoothstep(0.5, 0.7, cloud_density) - smoothstep(0.7, 0.9, cloud_density);
        day_cloud_color += cloud_rim * edge_highlight * cloud_lit * 0.3;

        let cloud_moon_dir = day_night_get_moon_direction(hour);
        let cloud_moon_dot = max(dot(dir, cloud_moon_dir), 0.0);
        let cloud_moon_lit = pow(cloud_moon_dot, 2.0);
        let moon_elevation = max(cloud_moon_dir.y, 0.0);

        let night_shadow = vec3<f32>(0.02, 0.02, 0.04);
        let night_bright = vec3<f32>(0.06, 0.07, 0.10);
        let night_rim = vec3<f32>(0.10, 0.12, 0.16);

        let night_mid = mix(night_shadow, night_bright, 0.3);
        var night_cloud_color = mix(night_shadow, night_mid, smoothstep(0.0, 0.3, cloud_moon_lit));
        night_cloud_color = mix(night_cloud_color, night_bright, smoothstep(0.3, 0.7, cloud_moon_lit));
        night_cloud_color = mix(night_cloud_color, night_rim, smoothstep(0.7, 1.0, cloud_moon_lit) * 0.4);
        night_cloud_color += night_rim * edge_highlight * cloud_moon_lit * 0.2;
        night_cloud_color *= 0.5 + 0.5 * moon_elevation;

        var cloud_day_factor = 1.0;
        if hour < 5.0 || hour > 20.0 {
            cloud_day_factor = 0.0;
        } else if hour < 7.0 {
            cloud_day_factor = smoothstep(5.0, 7.0, hour);
        } else if hour > 18.0 {
            cloud_day_factor = 1.0 - smoothstep(18.0, 20.0, hour);
        }

        let final_cloud_color = mix(night_cloud_color, day_cloud_color, cloud_day_factor);
        color = mix(color, final_cloud_color, cloud_amount * 0.9);
    }

    var night_visibility = 0.0;
    if hour < 5.0 || hour > 20.0 {
        night_visibility = 1.0;
    } else if hour < 7.0 {
        night_visibility = 1.0 - smoothstep(5.0, 7.0, hour);
    } else if hour > 18.0 {
        night_visibility = smoothstep(18.0, 20.0, hour);
    }

    if night_visibility > 0.0 && height > -0.1 {
        let uv = dir_to_uv(dir);

        var stars = 0.0;
        stars += star_layer_space(uv, 50.0, 0.0, 0.0);
        stars += star_layer_space(uv, 100.0, 10.0, 0.0);
        stars += star_layer_space(uv, 200.0, 20.0, 0.0) * 0.7;
        stars += star_layer_space(uv, 400.0, 30.0, 0.0) * 0.4;

        let star_hue = hash(floor(uv * 100.0));
        var star_col = vec3<f32>(1.0);
        if star_hue < 0.3 {
            star_col = vec3<f32>(0.8, 0.85, 1.0);
        } else if star_hue > 0.7 {
            star_col = vec3<f32>(1.0, 0.95, 0.85);
        }

        let height_fade = smoothstep(-0.1, 0.1, height);
        color += star_col * stars * night_visibility * height_fade;

        color += milky_way(dir) * night_visibility * height_fade;
    }

    let moon_dir = day_night_get_moon_direction(hour);
    if moon_dir.y > -0.05 {
        let moon_angular_size = 0.03;
        let moon_dist = acos(clamp(dot(dir, moon_dir), -1.0, 1.0));

        if moon_dist < moon_angular_size * 1.5 {
            let moon_horizon_fade = smoothstep(-0.05, 0.05, moon_dir.y);

            let moon_disc = 1.0 - smoothstep(moon_angular_size * 0.95, moon_angular_size, moon_dist);
            let moon_glow = (1.0 - smoothstep(moon_angular_size, moon_angular_size * 1.5, moon_dist)) * 0.15;

            let right = normalize(cross(moon_dir, vec3<f32>(0.0, 1.0, 0.0)));
            let up = normalize(cross(right, moon_dir));
            let local_x = dot(dir - moon_dir, right) / moon_angular_size;
            let local_y = dot(dir - moon_dir, up) / moon_angular_size;
            let local_pos = vec2<f32>(local_x, local_y);

            let surface = moon_surface(local_pos, moon_dir);
            let sun_dir = day_night_get_sun_direction(hour);
            let phase_dot = dot(moon_dir, sun_dir);
            let phase = smoothstep(-0.2, 0.8, local_x * sign(phase_dot) * 0.5 + 0.5);

            let moon_col = surface * phase * 0.8;
            color = mix(color, moon_col, moon_disc * moon_horizon_fade);
            color += vec3<f32>(0.7, 0.75, 0.85) * moon_glow * moon_horizon_fade;
        }
    }

    return color;
}

fn compute_procedural_color(dir: vec3<f32>, time: f32, atmosphere_type: u32) -> vec3<f32> {
    switch atmosphere_type {
        case ATMOSPHERE_SKY: {
            return sky_color(dir, time);
        }
        case ATMOSPHERE_CLOUDY_SKY: {
            return cloudy_sky_color(dir, time);
        }
        case ATMOSPHERE_SPACE: {
            return space_color(dir, time);
        }
        case ATMOSPHERE_NEBULA: {
            return nebula_color(dir, time);
        }
        case ATMOSPHERE_SUNSET: {
            return sunset_color(dir, time);
        }
        case ATMOSPHERE_DAY_NIGHT: {
            return day_night_color(dir, time);
        }
        default: {
            return vec3<f32>(0.0);
        }
    }
}

@compute @workgroup_size(16, 16, 1)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>
) {
    let face = group_id.z;
    if face >= 6u {
        return;
    }

    let coords = vec2<u32>(global_id.xy);
    if coords.x >= FACE_SIZE || coords.y >= FACE_SIZE {
        return;
    }

    let uv = (vec2<f32>(coords) + 0.5) / f32(FACE_SIZE);
    let dir = cube_to_world(face, uv);

    let color = compute_procedural_color(dir, params.time, params.atmosphere_type);

    textureStore(
        output_texture,
        coords,
        face,
        vec4<f32>(color, 1.0)
    );
}
