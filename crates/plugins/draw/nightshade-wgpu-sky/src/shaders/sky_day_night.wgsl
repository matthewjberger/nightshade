struct Uniform {
    proj: mat4x4<f32>,
    proj_inv: mat4x4<f32>,
    view: mat4x4<f32>,
    cam_pos: vec4<f32>,
    time: f32,
    _pad0: f32,
    hour: f32,
    _pad1: f32,
};

@group(0) @binding(0)
var<uniform> u: Uniform;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_dir: vec3<f32>,
};

const PI: f32 = 3.141592653589793;

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

fn dir_to_uv(dir: vec3<f32>) -> vec2<f32> {
    let phi = atan2(dir.z, dir.x);
    let theta = asin(clamp(dir.y, -1.0, 1.0));
    return vec2<f32>(phi / 6.283185 + 0.5, theta / 3.141593 + 0.5);
}

fn star_layer(uv: vec2<f32>, scale: f32, seed: f32, time: f32) -> f32 {
    let grid_uv = uv * scale;
    let grid_id = floor(grid_uv);
    let grid_fract = fract(grid_uv) - 0.5;

    let rand = hash(grid_id + seed);

    if rand > 0.97 {
        let offset = hash2(grid_id + seed + 100.0) * 0.4;
        let dist = length(grid_fract - offset);

        let star_size = 0.02 + hash(grid_id + seed + 200.0) * 0.03;
        let brightness = (1.0 - smoothstep(0.0, star_size, dist));

        let twinkle_speed = 1.0 + hash(grid_id + seed + 300.0) * 3.0;
        let twinkle_phase = hash(grid_id + seed + 400.0) * 6.283;
        let twinkle = 0.7 + 0.3 * sin(time * twinkle_speed + twinkle_phase);

        return brightness * twinkle;
    }

    return 0.0;
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

fn moon_surface(local_pos: vec2<f32>) -> vec3<f32> {
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

fn get_sun_direction(hour: f32) -> vec3<f32> {
    let sun_angle = (hour - 6.0) / 12.0 * PI;
    return normalize(vec3<f32>(-cos(sun_angle), sin(sun_angle), -0.3));
}

fn get_moon_direction(hour: f32) -> vec3<f32> {
    var h = hour;
    if h < 12.0 {
        h += 24.0;
    }
    let moon_angle = (h - 18.0) / 12.0 * PI;
    return normalize(vec3<f32>(-cos(moon_angle), sin(moon_angle), 0.3));
}

fn is_sun_visible(hour: f32) -> bool {
    return hour >= 5.5 && hour <= 18.5;
}

fn get_sky_color(dir: vec3<f32>, hour: f32) -> vec3<f32> {
    let height = dir.y;
    let abs_height = abs(height);

    let night_color = vec3<f32>(0.01, 0.01, 0.03);
    let dawn_horizon = vec3<f32>(0.8, 0.4, 0.15);
    let dawn_sky = vec3<f32>(0.2, 0.25, 0.5);
    let morning_sky = vec3<f32>(0.35, 0.5, 0.75);
    let midday_sky_top = vec3<f32>(0.25, 0.45, 0.8);
    let midday_sky_horizon = vec3<f32>(0.55, 0.65, 0.8);
    let afternoon_sky = vec3<f32>(0.35, 0.5, 0.7);
    let sunset_horizon = vec3<f32>(0.95, 0.35, 0.08);
    let sunset_mid = vec3<f32>(0.85, 0.45, 0.55);
    let sunset_upper = vec3<f32>(0.35, 0.2, 0.45);
    let dusk_color = vec3<f32>(0.1, 0.08, 0.2);

    var sky_top: vec3<f32>;
    var sky_horizon: vec3<f32>;

    if hour < 5.0 {
        sky_top = night_color;
        sky_horizon = night_color;
    } else if hour < 6.0 {
        let t = hour - 5.0;
        sky_top = mix(night_color, dawn_sky * 0.3, t);
        sky_horizon = mix(night_color, dawn_horizon * 0.3, t);
    } else if hour < 7.0 {
        let t = hour - 6.0;
        sky_top = mix(dawn_sky * 0.3, morning_sky * 0.7, t);
        sky_horizon = mix(dawn_horizon * 0.3, dawn_horizon, t);
    } else if hour < 9.0 {
        let t = (hour - 7.0) / 2.0;
        sky_top = mix(morning_sky * 0.7, midday_sky_top, t);
        sky_horizon = mix(dawn_horizon, midday_sky_horizon, t);
    } else if hour < 15.0 {
        sky_top = midday_sky_top;
        sky_horizon = midday_sky_horizon;
    } else if hour < 17.0 {
        let t = (hour - 15.0) / 2.0;
        sky_top = mix(midday_sky_top, afternoon_sky, t);
        sky_horizon = mix(midday_sky_horizon, vec3<f32>(0.7, 0.55, 0.45), t);
    } else if hour < 18.0 {
        let t = hour - 17.0;
        sky_top = mix(afternoon_sky, sunset_upper, t);
        sky_horizon = mix(vec3<f32>(0.7, 0.55, 0.45), sunset_horizon, t);
    } else if hour < 19.0 {
        let t = hour - 18.0;
        sky_top = mix(sunset_upper, dusk_color, t);
        sky_horizon = mix(sunset_horizon, dusk_color * 1.5, t);
    } else if hour < 20.0 {
        let t = hour - 19.0;
        sky_top = mix(dusk_color, night_color, t);
        sky_horizon = mix(dusk_color * 1.5, night_color, t);
    } else {
        sky_top = night_color;
        sky_horizon = night_color;
    }

    var color: vec3<f32>;
    if height >= 0.0 {
        let sky_curve = 0.15;
        let t = 1.0 - pow(1.0 - height, 1.0 / sky_curve);
        color = mix(sky_horizon, sky_top, clamp(t, 0.0, 1.0));
    } else {
        let ground_darken = 1.0 - smoothstep(-0.4, 0.0, height);
        let ground_color = mix(sky_horizon, night_color, ground_darken);
        color = ground_color * mix(1.0, 0.15, ground_darken);
    }

    return color;
}

@fragment
fn fs_sky(in: VertexOutput) -> @location(0) vec4<f32> {
    let dir = normalize(in.world_dir);
    let time = u.time;
    let hour = u.hour - floor(u.hour / 24.0) * 24.0;

    var color = get_sky_color(dir, hour);
    let height = dir.y;

    if is_sun_visible(hour) {
        let sun_dir = get_sun_direction(hour);
        let sun_dot = dot(dir, sun_dir);
        let sun_proximity = max(sun_dot, 0.0);

        var sun_intensity = 1.0;
        if hour < 6.5 {
            sun_intensity = smoothstep(5.5, 6.5, hour);
        } else if hour > 17.5 {
            sun_intensity = 1.0 - smoothstep(17.5, 18.5, hour);
        }

        var sun_core_color = vec3<f32>(1.0, 0.98, 0.9);
        var sun_glow_color = vec3<f32>(1.0, 0.8, 0.5);

        if hour < 7.0 || hour > 17.0 {
            sun_core_color = vec3<f32>(1.0, 0.85, 0.6);
            sun_glow_color = vec3<f32>(1.0, 0.5, 0.2);
        }

        let sun_angular_radius = 0.9995;
        let sun_core = smoothstep(sun_angular_radius - 0.0003, sun_angular_radius, sun_dot);
        let sun_limb = smoothstep(sun_angular_radius - 0.001, sun_angular_radius - 0.0003, sun_dot) * 0.3;
        let sun_glow_tight = pow(sun_proximity, 32.0) * 0.6;
        let sun_glow_medium = pow(sun_proximity, 8.0) * 0.35;
        let sun_glow_wide = pow(sun_proximity, 2.5) * 0.2;

        color += sun_core_color * sun_core * sun_intensity;
        color += vec3<f32>(1.0, 0.8, 0.4) * sun_limb * sun_intensity;
        color += sun_glow_color * sun_glow_tight * sun_intensity;
        color += sun_glow_color * sun_glow_medium * sun_intensity;
        color += sun_glow_color * 0.5 * sun_glow_wide * sun_intensity;
    }

    if height > 0.0 && height < 0.6 {
        let sun_dir = get_sun_direction(clamp(hour, 6.0, 18.0));
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

        let cloud_moon_dir = get_moon_direction(hour);
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
        stars += star_layer(uv, 50.0, 0.0, time);
        stars += star_layer(uv, 100.0, 10.0, time);
        stars += star_layer(uv, 200.0, 20.0, time) * 0.7;
        stars += star_layer(uv, 400.0, 30.0, time) * 0.4;

        let star_hue = hash(floor(uv * 100.0));
        var star_color = vec3<f32>(1.0);
        if star_hue < 0.3 {
            star_color = vec3<f32>(0.8, 0.85, 1.0);
        } else if star_hue > 0.7 {
            star_color = vec3<f32>(1.0, 0.95, 0.85);
        }

        let height_fade = smoothstep(-0.1, 0.1, height);
        color += star_color * stars * night_visibility * height_fade;

        color += milky_way(dir) * night_visibility * height_fade;
    }

    let moon_dir = get_moon_direction(hour);
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

            let surface = moon_surface(local_pos);
            let sun_dir = get_sun_direction(hour);
            let phase_dot = dot(moon_dir, sun_dir);
            let phase = smoothstep(-0.2, 0.8, local_x * sign(phase_dot) * 0.5 + 0.5);

            let moon_color = surface * phase * 0.8;
            color = mix(color, moon_color, moon_disc * moon_horizon_fade);
            color += vec3<f32>(0.7, 0.75, 0.85) * moon_glow * moon_horizon_fade;
        }
    }

    return vec4<f32>(color, 1.0);
}
