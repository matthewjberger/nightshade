struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Uniforms {
    time: f32,
    chromatic_aberration: f32,
    wave_distortion: f32,
    color_shift: f32,
    kaleidoscope_segments: f32,
    crt_scanlines: f32,
    vignette: f32,
    plasma_intensity: f32,
    glitch_intensity: f32,
    mirror_mode: f32,
    invert: f32,
    hue_rotation: f32,
    raymarch_mode: f32,
    raymarch_blend: f32,
    film_grain: f32,
    sharpen: f32,
    pixelate: f32,
    color_posterize: f32,
    radial_blur: f32,
    tunnel_speed: f32,
    fractal_iterations: f32,
    glow_intensity: f32,
    screen_shake: f32,
    zoom_pulse: f32,
    speed_lines: f32,
    color_grade_mode: f32,
    vhs_distortion: f32,
    lens_flare: f32,
    edge_glow: f32,
    saturation: f32,
    warp_speed: f32,
    pulse_rings: f32,
    heat_distortion: f32,
    digital_rain: f32,
    strobe: f32,
    color_cycle_speed: f32,
    feedback_amount: f32,
    ascii_mode: f32,
};

@group(0) @binding(0)
var input_texture: texture_2d<f32>;

@group(0) @binding(1)
var input_sampler: sampler;

@group(0) @binding(2)
var<uniform> uniforms: Uniforms;

const PI: f32 = 3.14159265359;
const TAU: f32 = 6.28318530718;

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((vertex_index & 1u) << 1u);
    let y = f32((vertex_index & 2u));
    out.position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, 1.0 - y);
    return out;
}

fn hash(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

fn hash3(p: vec3<f32>) -> f32 {
    let h = dot(p, vec3<f32>(127.1, 311.7, 74.7));
    return fract(sin(h) * 43758.5453123);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(hash(i + vec2<f32>(0.0, 0.0)), hash(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(hash(i + vec2<f32>(0.0, 1.0)), hash(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y
    );
}

fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    var pos = p;
    for (var i = 0; i < 5; i++) {
        value += amplitude * noise(pos * frequency);
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    return value;
}

fn plasma(uv: vec2<f32>, time: f32) -> f32 {
    var value = 0.0;
    value += sin(uv.x * 10.0 + time);
    value += sin((uv.y * 10.0 + time) * 0.5);
    value += sin((uv.x * 10.0 + uv.y * 10.0 + time) * 0.5);
    let cx = uv.x + 0.5 * sin(time * 0.2);
    let cy = uv.y + 0.5 * cos(time * 0.3);
    value += sin(sqrt(cx * cx + cy * cy + 1.0) * 10.0 + time);
    return value * 0.25;
}

fn rgb_to_hsv(c: vec3<f32>) -> vec3<f32> {
    let k = vec4<f32>(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    let p = mix(vec4<f32>(c.bg, k.wz), vec4<f32>(c.gb, k.xy), step(c.b, c.g));
    let q = mix(vec4<f32>(p.xyw, c.r), vec4<f32>(c.r, p.yzx), step(p.x, c.r));
    let d = q.x - min(q.w, q.y);
    let e = 1.0e-10;
    return vec3<f32>(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

fn hsv_to_rgb(c: vec3<f32>) -> vec3<f32> {
    let k = vec4<f32>(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + k.xyz) * 6.0 - k.www);
    return c.z * mix(k.xxx, clamp(p - k.xxx, vec3<f32>(0.0), vec3<f32>(1.0)), c.y);
}

fn kaleidoscope(uv: vec2<f32>, segments: f32) -> vec2<f32> {
    let centered = uv - vec2<f32>(0.5);
    let angle = atan2(centered.y, centered.x);
    let radius = length(centered);
    let segment_angle = PI * 2.0 / segments;
    var new_angle = abs(mod_f32(angle, segment_angle) - segment_angle * 0.5);
    new_angle = new_angle + segment_angle * 0.5;
    return vec2<f32>(cos(new_angle), sin(new_angle)) * radius + vec2<f32>(0.5);
}

fn barrel_distort(uv: vec2<f32>, amount: f32) -> vec2<f32> {
    let centered = uv - vec2<f32>(0.5);
    let r2 = dot(centered, centered);
    let distorted = centered * (1.0 + amount * r2);
    return distorted + vec2<f32>(0.5);
}

fn rotate2d(angle: f32) -> mat2x2<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return mat2x2<f32>(c, -s, s, c);
}

fn sd_box(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

fn sd_sphere(p: vec3<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn sd_torus(p: vec3<f32>, t: vec2<f32>) -> f32 {
    let q = vec2<f32>(length(p.xz) - t.x, p.y);
    return length(q) - t.y;
}

fn sd_octahedron(p: vec3<f32>, s: f32) -> f32 {
    let q = abs(p);
    return (q.x + q.y + q.z - s) * 0.57735027;
}

fn mod_vec3(p: vec3<f32>, c: vec3<f32>) -> vec3<f32> {
    return p - c * floor(p / c);
}

fn mod_f32(x: f32, y: f32) -> f32 {
    return x - y * floor(x / y);
}

fn op_rep(p: vec3<f32>, c: vec3<f32>) -> vec3<f32> {
    return mod_vec3(p, c) - 0.5 * c;
}

fn op_smooth_union(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return mix(d2, d1, h) - k * h * (1.0 - h);
}

fn tunnel_map(p: vec3<f32>, time: f32) -> f32 {
    var pos = p;
    let twist = sin(pos.z * 0.1 + time * 0.5) * 0.5;
    let xy = rotate2d(twist) * pos.xy;
    pos = vec3<f32>(xy, pos.z);

    let tunnel_radius = 2.0 + sin(pos.z * 0.3 + time) * 0.5;
    let tunnel = tunnel_radius - length(pos.xy);

    let ring_freq = 0.8;
    let ring_pos = mod_f32(pos.z, ring_freq) - ring_freq * 0.5;
    let rings = sd_torus(vec3<f32>(pos.xy, ring_pos).xzy, vec2<f32>(tunnel_radius - 0.1, 0.05));

    let pillars = 8.0;
    let pillar_angle = atan2(pos.y, pos.x);
    let pillar_id = floor(pillar_angle / TAU * pillars + 0.5);
    let pillar_a = pillar_id * TAU / pillars;
    let pillar_pos = vec2<f32>(cos(pillar_a), sin(pillar_a)) * (tunnel_radius + 0.1);
    let pillar = length(pos.xy - pillar_pos) - 0.1;

    return min(tunnel, min(rings, pillar));
}

fn fractal_map(p: vec3<f32>, time: f32, iterations: i32) -> f32 {
    var pos = p;
    let scale = 2.0;
    var d = sd_box(pos, vec3<f32>(1.0));
    var s = 1.0;

    for (var i = 0; i < iterations; i++) {
        let a = (time * 0.1 + f32(i) * 0.5) % TAU;
        let xy = rotate2d(a) * pos.xy;
        pos = vec3<f32>(xy, pos.z);
        let yz = rotate2d(a * 0.7) * pos.yz;
        pos = vec3<f32>(pos.x, yz);

        pos = abs(pos);
        if (pos.x < pos.y) { pos = pos.yxz; }
        if (pos.x < pos.z) { pos = pos.zyx; }
        if (pos.y < pos.z) { pos = pos.xzy; }

        pos = pos * scale - vec3<f32>(scale - 1.0);
        s *= scale;

        let box_d = sd_box(pos, vec3<f32>(1.0)) / s;
        d = min(d, box_d);
    }

    return d;
}

fn mandelbulb_map(p: vec3<f32>, time: f32) -> f32 {
    var w = p;
    var m = dot(w, w);
    var dz = 1.0;
    let power = 8.0 + sin(time * 0.2) * 2.0;

    for (var i = 0; i < 4; i++) {
        dz = power * pow(m, (power - 1.0) * 0.5) * dz + 1.0;

        let r = length(w);
        let theta = acos(w.z / r) * power;
        let phi = atan2(w.y, w.x) * power;
        w = p + pow(r, power) * vec3<f32>(
            sin(theta) * cos(phi),
            sin(theta) * sin(phi),
            cos(theta)
        );

        m = dot(w, w);
        if (m > 256.0) { break; }
    }

    return 0.25 * log(m) * sqrt(m) / dz;
}

fn plasma_vortex_map(p: vec3<f32>, time: f32) -> f32 {
    var pos = p;
    let vortex_twist = pos.z * 0.5 + time;
    let xy = rotate2d(vortex_twist) * pos.xy;
    pos = vec3<f32>(xy, pos.z);

    let core = length(pos.xy) - 0.5 - sin(pos.z * 2.0 + time * 2.0) * 0.2;

    let arms = 5.0;
    let arm_angle = atan2(pos.y, pos.x);
    let arm_pattern = sin(arm_angle * arms + pos.z * 3.0 - time * 3.0);
    let arm_dist = length(pos.xy) - 1.5 - arm_pattern * 0.3;

    let rings_d = abs(length(pos.xy) - 2.0 - sin(pos.z + time) * 0.5) - 0.1;

    return min(core, min(arm_dist, rings_d));
}

fn geometric_map(p: vec3<f32>, time: f32) -> f32 {
    var pos = p;

    let rep = vec3<f32>(4.0);
    pos = op_rep(pos + vec3<f32>(0.0, 0.0, time * 2.0), rep);

    let rot_xy = rotate2d(time * 0.5);
    let rot_yz = rotate2d(time * 0.3);
    let xy = rot_xy * pos.xy;
    pos = vec3<f32>(xy, pos.z);
    let yz = rot_yz * pos.yz;
    pos = vec3<f32>(pos.x, yz);

    let cube = sd_box(pos, vec3<f32>(0.5));
    let sphere = sd_sphere(pos, 0.7);
    let octa = sd_octahedron(pos, 0.6);

    let blend = sin(time) * 0.5 + 0.5;
    var d = op_smooth_union(cube, sphere, 0.3);
    d = op_smooth_union(d, octa, 0.2 * blend);

    return d;
}

fn raymarch_scene(ro: vec3<f32>, rd: vec3<f32>, time: f32, mode: i32, iterations: i32) -> vec4<f32> {
    var t = 0.0;
    var color = vec3<f32>(0.0);
    var glow = vec3<f32>(0.0);
    let max_dist = 50.0;
    let max_steps = 64;

    for (var i = 0; i < max_steps; i++) {
        let p = ro + rd * t;
        var d: f32;

        switch (mode) {
            case 1: { d = tunnel_map(p, time); }
            case 2: { d = fractal_map(p, time, iterations); }
            case 3: { d = mandelbulb_map(p * 0.5, time) * 2.0; }
            case 4: { d = plasma_vortex_map(p, time); }
            case 5: { d = geometric_map(p, time); }
            default: { d = max_dist; }
        }

        let glow_factor = exp(-d * 3.0) * 0.1;
        let glow_hue = fract(t * 0.05 + time * 0.1);
        glow += hsv_to_rgb(vec3<f32>(glow_hue, 0.8, 1.0)) * glow_factor;

        if (d < 0.001) {
            let normal_eps = 0.001;
            var n: vec3<f32>;
            switch (mode) {
                case 1: {
                    n = normalize(vec3<f32>(
                        tunnel_map(p + vec3<f32>(normal_eps, 0.0, 0.0), time) - tunnel_map(p - vec3<f32>(normal_eps, 0.0, 0.0), time),
                        tunnel_map(p + vec3<f32>(0.0, normal_eps, 0.0), time) - tunnel_map(p - vec3<f32>(0.0, normal_eps, 0.0), time),
                        tunnel_map(p + vec3<f32>(0.0, 0.0, normal_eps), time) - tunnel_map(p - vec3<f32>(0.0, 0.0, normal_eps), time)
                    ));
                }
                case 2: {
                    n = normalize(vec3<f32>(
                        fractal_map(p + vec3<f32>(normal_eps, 0.0, 0.0), time, iterations) - fractal_map(p - vec3<f32>(normal_eps, 0.0, 0.0), time, iterations),
                        fractal_map(p + vec3<f32>(0.0, normal_eps, 0.0), time, iterations) - fractal_map(p - vec3<f32>(0.0, normal_eps, 0.0), time, iterations),
                        fractal_map(p + vec3<f32>(0.0, 0.0, normal_eps), time, iterations) - fractal_map(p - vec3<f32>(0.0, 0.0, normal_eps), time, iterations)
                    ));
                }
                case 3: {
                    n = normalize(vec3<f32>(
                        mandelbulb_map((p + vec3<f32>(normal_eps, 0.0, 0.0)) * 0.5, time) - mandelbulb_map((p - vec3<f32>(normal_eps, 0.0, 0.0)) * 0.5, time),
                        mandelbulb_map((p + vec3<f32>(0.0, normal_eps, 0.0)) * 0.5, time) - mandelbulb_map((p - vec3<f32>(0.0, normal_eps, 0.0)) * 0.5, time),
                        mandelbulb_map((p + vec3<f32>(0.0, 0.0, normal_eps)) * 0.5, time) - mandelbulb_map((p - vec3<f32>(0.0, 0.0, normal_eps)) * 0.5, time)
                    ));
                }
                case 4: {
                    n = normalize(vec3<f32>(
                        plasma_vortex_map(p + vec3<f32>(normal_eps, 0.0, 0.0), time) - plasma_vortex_map(p - vec3<f32>(normal_eps, 0.0, 0.0), time),
                        plasma_vortex_map(p + vec3<f32>(0.0, normal_eps, 0.0), time) - plasma_vortex_map(p - vec3<f32>(0.0, normal_eps, 0.0), time),
                        plasma_vortex_map(p + vec3<f32>(0.0, 0.0, normal_eps), time) - plasma_vortex_map(p - vec3<f32>(0.0, 0.0, normal_eps), time)
                    ));
                }
                default: {
                    n = normalize(vec3<f32>(
                        geometric_map(p + vec3<f32>(normal_eps, 0.0, 0.0), time) - geometric_map(p - vec3<f32>(normal_eps, 0.0, 0.0), time),
                        geometric_map(p + vec3<f32>(0.0, normal_eps, 0.0), time) - geometric_map(p - vec3<f32>(0.0, normal_eps, 0.0), time),
                        geometric_map(p + vec3<f32>(0.0, 0.0, normal_eps), time) - geometric_map(p - vec3<f32>(0.0, 0.0, normal_eps), time)
                    ));
                }
            }

            let light_dir = normalize(vec3<f32>(1.0, 1.0, -1.0));
            let diff = max(dot(n, light_dir), 0.0);
            let spec = pow(max(dot(reflect(-light_dir, n), -rd), 0.0), 32.0);

            let base_hue = fract(t * 0.02 + time * 0.05 + dot(n, vec3<f32>(1.0)) * 0.1);
            let base_color = hsv_to_rgb(vec3<f32>(base_hue, 0.7, 0.9));

            color = base_color * (0.2 + diff * 0.6) + vec3<f32>(1.0) * spec * 0.4;
            color += glow * 0.5;

            let fog = 1.0 - exp(-t * 0.05);
            let fog_color = hsv_to_rgb(vec3<f32>(fract(time * 0.02), 0.3, 0.1));
            color = mix(color, fog_color, fog);

            return vec4<f32>(color, 1.0);
        }

        t += d * 0.8;
        if (t > max_dist) { break; }
    }

    let bg_hue = fract(rd.y * 0.2 + time * 0.02);
    let bg = hsv_to_rgb(vec3<f32>(bg_hue, 0.5, 0.05));
    color = bg + glow;

    return vec4<f32>(color, 0.5);
}

fn get_ray(uv: vec2<f32>, time: f32) -> vec3<f32> {
    let fov = 1.5;
    return normalize(vec3<f32>((uv - 0.5) * 2.0 * fov, 1.0));
}

fn speed_lines_effect(uv: vec2<f32>, time: f32, intensity: f32) -> f32 {
    let centered = uv - vec2<f32>(0.5);
    let angle = atan2(centered.y, centered.x);
    let dist = length(centered);

    let num_lines = 64.0;
    let line_pattern = sin(angle * num_lines + time * 5.0) * 0.5 + 0.5;
    let line_sharp = pow(line_pattern, 8.0);

    let fade = smoothstep(0.2, 0.5, dist);
    let pulse = sin(time * 10.0) * 0.3 + 0.7;

    return line_sharp * fade * intensity * pulse;
}

fn lens_flare_effect(uv: vec2<f32>, time: f32, intensity: f32) -> vec3<f32> {
    var flare = vec3<f32>(0.0);

    let light_pos = vec2<f32>(0.3 + sin(time * 0.5) * 0.2, 0.3 + cos(time * 0.7) * 0.2);
    let to_light = uv - light_pos;
    let dist = length(to_light);

    let main_flare = exp(-dist * 8.0) * 0.5;
    flare += vec3<f32>(1.0, 0.8, 0.6) * main_flare;

    let ghost1_pos = vec2<f32>(0.5) + (vec2<f32>(0.5) - light_pos) * 0.5;
    let ghost1_dist = length(uv - ghost1_pos);
    flare += vec3<f32>(0.3, 0.6, 1.0) * exp(-ghost1_dist * 15.0) * 0.3;

    let ghost2_pos = vec2<f32>(0.5) + (vec2<f32>(0.5) - light_pos) * 1.2;
    let ghost2_dist = length(uv - ghost2_pos);
    flare += vec3<f32>(1.0, 0.4, 0.8) * exp(-ghost2_dist * 20.0) * 0.2;

    let anamorphic = exp(-abs(uv.y - light_pos.y) * 10.0) * exp(-dist * 2.0) * 0.4;
    flare += vec3<f32>(0.6, 0.8, 1.0) * anamorphic;

    return flare * intensity;
}

fn warp_speed_effect(uv: vec2<f32>, time: f32, intensity: f32) -> vec3<f32> {
    let centered = uv - vec2<f32>(0.5);
    let dist = length(centered);
    let angle = atan2(centered.y, centered.x);

    var stars = vec3<f32>(0.0);
    let num_layers = 4;

    for (var layer = 0; layer < num_layers; layer++) {
        let layer_f = f32(layer);
        let speed = 2.0 + layer_f * 1.5;
        let star_time = time * speed;

        let num_stars = 32.0 + layer_f * 16.0;
        for (var star_index = 0.0; star_index < num_stars; star_index += 1.0) {
            let star_angle = star_index * TAU / num_stars + layer_f * 0.5;
            let star_dist = fract(star_time * 0.3 + hash(vec2<f32>(star_index, layer_f)) * 10.0);
            let star_pos = vec2<f32>(cos(star_angle), sin(star_angle)) * star_dist * 0.7;

            let to_star = centered - star_pos;
            let streak_length = 0.02 + star_dist * 0.08 * intensity;
            let streak_dir = normalize(star_pos);

            let along = dot(to_star, streak_dir);
            let perp = length(to_star - streak_dir * along);

            if along > -streak_length && along < 0.005 && perp < 0.003 {
                let brightness = (1.0 - star_dist) * (1.0 - abs(along) / streak_length);
                let hue = fract(star_index * 0.1 + layer_f * 0.25);
                stars += hsv_to_rgb(vec3<f32>(hue, 0.3, 1.0)) * brightness * brightness * 2.0;
            }
        }
    }

    return stars * intensity;
}

fn pulse_rings_effect(uv: vec2<f32>, time: f32, intensity: f32) -> vec3<f32> {
    let centered = uv - vec2<f32>(0.5);
    let dist = length(centered);

    var rings = vec3<f32>(0.0);
    let num_rings = 5;

    for (var ring_index = 0; ring_index < num_rings; ring_index++) {
        let ring_f = f32(ring_index);
        let ring_time = time * 1.5 - ring_f * 0.3;
        let ring_dist = fract(ring_time * 0.5) * 0.8;
        let ring_width = 0.02 + ring_f * 0.005;

        let ring_brightness = smoothstep(ring_dist - ring_width, ring_dist, dist)
                            * smoothstep(ring_dist + ring_width, ring_dist, dist);
        let fade = 1.0 - fract(ring_time * 0.5);

        let hue = fract(time * 0.1 + ring_f * 0.2);
        rings += hsv_to_rgb(vec3<f32>(hue, 0.8, 1.0)) * ring_brightness * fade * 2.0;
    }

    return rings * intensity;
}

fn heat_distortion_effect(uv: vec2<f32>, time: f32, intensity: f32) -> vec2<f32> {
    let heat_y = uv.y + sin(uv.x * 30.0 + time * 3.0) * 0.002 * intensity;
    let heat_x = uv.x + sin(uv.y * 25.0 + time * 2.5) * 0.003 * intensity;

    let wave1 = sin(uv.y * 50.0 + time * 5.0) * 0.001;
    let wave2 = sin(uv.y * 80.0 - time * 7.0) * 0.0005;
    let wave3 = noise(uv * 20.0 + time) * 0.002;

    return vec2<f32>(heat_x + (wave1 + wave2 + wave3) * intensity, heat_y);
}

fn digital_rain_effect(uv: vec2<f32>, time: f32, intensity: f32) -> vec3<f32> {
    var rain = vec3<f32>(0.0);
    let columns = 40.0;

    let col = floor(uv.x * columns);
    let col_uv = fract(uv.x * columns);

    let col_seed = hash(vec2<f32>(col, 0.0));
    let fall_speed = 0.5 + col_seed * 0.5;
    let fall_offset = col_seed * 10.0;

    let char_height = 0.05;
    let row = floor((uv.y + time * fall_speed + fall_offset) / char_height);
    let row_uv = fract((uv.y + time * fall_speed + fall_offset) / char_height);

    let char_seed = hash(vec2<f32>(col, row));
    let is_char = step(0.3, char_seed);

    let brightness = smoothstep(0.0, 0.3, row_uv) * smoothstep(1.0, 0.7, row_uv);
    let edge_fade = smoothstep(0.0, 0.2, col_uv) * smoothstep(1.0, 0.8, col_uv);

    let trail_length = 15.0;
    var trail_brightness = 0.0;
    for (var trail_index = 0.0; trail_index < trail_length; trail_index += 1.0) {
        let trail_row = row - trail_index;
        let trail_char_seed = hash(vec2<f32>(col, trail_row));
        if trail_char_seed > 0.3 {
            trail_brightness += (1.0 - trail_index / trail_length) * 0.15;
        }
    }

    let head_brightness = is_char * brightness * edge_fade;
    let green_color = vec3<f32>(0.2, 1.0, 0.3);
    let white_head = vec3<f32>(0.8, 1.0, 0.9);

    rain = mix(green_color * trail_brightness, white_head, head_brightness * 0.5) * (head_brightness + trail_brightness * 0.3);

    return rain * intensity;
}

fn apply_color_grade(color: vec3<f32>, mode: i32) -> vec3<f32> {
    var graded = color;

    switch (mode) {
        case 1: {
            graded.r = pow(color.r, 0.9) * 1.2;
            graded.g = color.g * 0.9;
            graded.b = pow(color.b, 0.8) * 1.3;
            graded = mix(graded, vec3<f32>(graded.r * 1.2, graded.g * 0.5, graded.b * 1.5), 0.3);
        }
        case 2: {
            graded.r = pow(color.r, 1.1) * 1.1;
            graded.g = color.g * 0.7 + 0.1;
            graded.b = pow(color.b, 0.85) * 1.4;
            let sunset = vec3<f32>(1.0, 0.6, 0.8);
            graded = mix(graded, graded * sunset, 0.4);
        }
        case 3: {
            let luminance = dot(color, vec3<f32>(0.299, 0.587, 0.114));
            graded = vec3<f32>(luminance);
        }
        case 4: {
            let luminance = dot(color, vec3<f32>(0.299, 0.587, 0.114));
            graded = vec3<f32>(luminance * 1.1, luminance * 1.05, luminance * 0.9);
        }
        case 5: {
            graded.r = pow(color.r, 0.95) * 1.1;
            graded.g = pow(color.g, 1.1) * 1.2;
            graded.b = color.b * 0.8;
            graded = mix(graded, vec3<f32>(0.0, graded.g * 1.5, graded.b * 0.5), 0.2);
        }
        case 6: {
            graded = pow(color, vec3<f32>(1.2));
            graded.r *= 1.3;
            graded.b *= 0.7;
            let burn = 1.0 - exp(-length(color) * 2.0);
            graded = mix(graded, graded * vec3<f32>(1.0, 0.5, 0.2), burn * 0.3);
        }
        default: { }
    }

    return graded;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;
    let time = uniforms.time;
    var original_uv = uv;

    if uniforms.screen_shake > 0.0 {
        let shake_freq = 20.0;
        let shake_x = sin(time * shake_freq) * cos(time * shake_freq * 1.3) * uniforms.screen_shake * 0.02;
        let shake_y = cos(time * shake_freq * 0.9) * sin(time * shake_freq * 1.1) * uniforms.screen_shake * 0.02;
        uv += vec2<f32>(shake_x, shake_y);
        original_uv = uv;
    }

    if uniforms.zoom_pulse > 0.0 {
        let pulse = sin(time * 3.0) * 0.5 + 0.5;
        let zoom = 1.0 - pulse * uniforms.zoom_pulse * 0.1;
        uv = (uv - vec2<f32>(0.5)) * zoom + vec2<f32>(0.5);
    }

    if uniforms.vhs_distortion > 0.0 {
        let band_pos = mod_f32(time * 0.5, 1.5) - 0.25;
        let band_dist = abs(uv.y - band_pos);
        let band_effect = exp(-band_dist * 10.0) * uniforms.vhs_distortion;
        uv.x += band_effect * 0.1 * sin(time * 50.0);

        let noise_line = floor(uv.y * 200.0);
        let line_noise = hash(vec2<f32>(noise_line, floor(time * 30.0)));
        if line_noise > 0.97 {
            uv.x += (hash(vec2<f32>(noise_line + time, 0.0)) - 0.5) * 0.05 * uniforms.vhs_distortion;
        }

        let tracking = sin(uv.y * 100.0 + time * 2.0) * 0.002 * uniforms.vhs_distortion;
        uv.x += tracking;
    }

    if uniforms.pixelate > 0.0 {
        let pixels = 64.0 + (1.0 - uniforms.pixelate) * 400.0;
        uv = floor(uv * pixels) / pixels;
    }

    if uniforms.mirror_mode > 0.5 {
        if uv.x > 0.5 {
            uv.x = 1.0 - uv.x;
        }
        if uv.y > 0.5 {
            uv.y = 1.0 - uv.y;
        }
    }

    if uniforms.kaleidoscope_segments > 1.5 {
        uv = kaleidoscope(uv, uniforms.kaleidoscope_segments);
    }

    if uniforms.wave_distortion > 0.0 {
        let wave_strength = uniforms.wave_distortion * 0.03;
        uv.x += sin(uv.y * 20.0 + time * 3.0) * wave_strength;
        uv.y += cos(uv.x * 20.0 + time * 2.5) * wave_strength;
        uv.x += sin(uv.y * 40.0 - time * 4.0) * wave_strength * 0.5;
        uv.y += cos(uv.x * 30.0 + time * 3.5) * wave_strength * 0.5;
    }

    if uniforms.heat_distortion > 0.0 {
        uv = heat_distortion_effect(uv, time, uniforms.heat_distortion);
    }

    if uniforms.glitch_intensity > 0.0 {
        let glitch_line = floor(uv.y * 50.0);
        let glitch_time = floor(time * 10.0);
        let glitch_noise = hash(vec2<f32>(glitch_line, glitch_time));
        if glitch_noise > (1.0 - uniforms.glitch_intensity * 0.3) {
            uv.x += (hash(vec2<f32>(glitch_line + glitch_time, 0.0)) - 0.5) * 0.1 * uniforms.glitch_intensity;
        }

        let block_noise = hash(vec2<f32>(floor(time * 5.0), 0.0));
        if block_noise > 0.95 {
            let block_y = hash(vec2<f32>(time * 100.0, 1.0));
            let block_height = hash(vec2<f32>(time * 100.0, 2.0)) * 0.1;
            if uv.y > block_y && uv.y < block_y + block_height {
                uv.x = fract(uv.x + hash(vec2<f32>(time * 100.0, 3.0)) * 0.5 * uniforms.glitch_intensity);
            }
        }
    }

    var color: vec3<f32>;

    if uniforms.radial_blur > 0.0 {
        let center = vec2<f32>(0.5, 0.5);
        let blur_samples = 8;
        var blur_color = vec3<f32>(0.0);
        let blur_strength = uniforms.radial_blur * 0.02;

        for (var i = 0; i < blur_samples; i++) {
            let t = f32(i) / f32(blur_samples);
            let sample_uv = mix(uv, center, t * blur_strength);
            blur_color += textureSample(input_texture, input_sampler, sample_uv).rgb;
        }
        color = blur_color / f32(blur_samples);
    } else if uniforms.chromatic_aberration > 0.0 {
        let ca_amount = uniforms.chromatic_aberration * 0.02;
        let dir = normalize(uv - vec2<f32>(0.5));
        let dist = length(uv - vec2<f32>(0.5));
        let offset = dir * dist * ca_amount;

        let r = textureSample(input_texture, input_sampler, uv + offset * 1.0).r;
        let g = textureSample(input_texture, input_sampler, uv).g;
        let b = textureSample(input_texture, input_sampler, uv - offset * 1.0).b;
        color = vec3<f32>(r, g, b);
    } else {
        color = textureSample(input_texture, input_sampler, uv).rgb;
    }

    if uniforms.sharpen > 0.0 {
        let texel = 1.0 / vec2<f32>(1920.0, 1080.0);
        let sharp_kernel = uniforms.sharpen * 2.0;
        let center = textureSample(input_texture, input_sampler, uv).rgb;
        let left = textureSample(input_texture, input_sampler, uv - vec2<f32>(texel.x, 0.0)).rgb;
        let right = textureSample(input_texture, input_sampler, uv + vec2<f32>(texel.x, 0.0)).rgb;
        let top = textureSample(input_texture, input_sampler, uv - vec2<f32>(0.0, texel.y)).rgb;
        let bottom = textureSample(input_texture, input_sampler, uv + vec2<f32>(0.0, texel.y)).rgb;
        color = color + (center * 4.0 - left - right - top - bottom) * sharp_kernel;
    }

    let raymarch_mode = i32(uniforms.raymarch_mode);
    if raymarch_mode > 0 && uniforms.raymarch_blend > 0.0 {
        let ray_origin = vec3<f32>(0.0, 0.0, -5.0 + time * uniforms.tunnel_speed);
        let ray_dir = get_ray(original_uv, time);
        let iterations = i32(uniforms.fractal_iterations);
        let rm_result = raymarch_scene(ray_origin, ray_dir, time, raymarch_mode, iterations);

        let blend_mode = uniforms.raymarch_blend;
        if blend_mode < 0.33 {
            let t = blend_mode * 3.0;
            color = mix(color, rm_result.rgb, t * rm_result.a);
        } else if blend_mode < 0.66 {
            color = color + rm_result.rgb * (blend_mode - 0.33) * 3.0;
        } else {
            color = color * (1.0 + rm_result.rgb * (blend_mode - 0.66) * 6.0);
        }
    }

    if uniforms.plasma_intensity > 0.0 {
        let plasma_val = plasma(original_uv, time);
        let plasma_color = vec3<f32>(
            sin(plasma_val * PI * 2.0) * 0.5 + 0.5,
            sin(plasma_val * PI * 2.0 + 2.094) * 0.5 + 0.5,
            sin(plasma_val * PI * 2.0 + 4.188) * 0.5 + 0.5
        );
        color = mix(color, color * plasma_color * 2.0, uniforms.plasma_intensity * 0.5);
        color += plasma_color * uniforms.plasma_intensity * 0.15;
    }

    if uniforms.color_shift > 0.0 {
        var hsv = rgb_to_hsv(color);
        hsv.x = fract(hsv.x + sin(time * 0.5 + original_uv.y * 3.0) * uniforms.color_shift * 0.2);
        hsv.y = min(1.0, hsv.y * (1.0 + uniforms.color_shift * 0.3));
        color = hsv_to_rgb(hsv);
    }

    if uniforms.hue_rotation > 0.0 {
        var hsv = rgb_to_hsv(color);
        hsv.x = fract(hsv.x + uniforms.hue_rotation);
        color = hsv_to_rgb(hsv);
    }

    if uniforms.color_posterize > 0.0 {
        let levels = 4.0 + (1.0 - uniforms.color_posterize) * 28.0;
        color = floor(color * levels) / levels;
    }

    if uniforms.crt_scanlines > 0.0 {
        let scanline = sin(in.uv.y * 800.0) * 0.5 + 0.5;
        let scanline_strength = uniforms.crt_scanlines * 0.3;
        color *= 1.0 - scanline_strength + scanline * scanline_strength;

        let pixel_x = fract(in.uv.x * 400.0);
        let rgb_stripe = vec3<f32>(
            step(pixel_x, 0.33),
            step(0.33, pixel_x) * step(pixel_x, 0.66),
            step(0.66, pixel_x)
        );
        color = mix(color, color * (rgb_stripe * 0.5 + 0.7), uniforms.crt_scanlines * 0.3);

        let curvature = 0.02 * uniforms.crt_scanlines;
        let curved_uv = barrel_distort(in.uv, curvature);
        if curved_uv.x < 0.0 || curved_uv.x > 1.0 || curved_uv.y < 0.0 || curved_uv.y > 1.0 {
            color *= 0.0;
        }
    }

    if uniforms.glow_intensity > 0.0 {
        let texel = 1.0 / vec2<f32>(1920.0, 1080.0);
        var glow = vec3<f32>(0.0);
        let glow_radius = 4.0;
        var samples = 0.0;

        for (var x = -2; x <= 2; x++) {
            for (var y = -2; y <= 2; y++) {
                let offset = vec2<f32>(f32(x), f32(y)) * texel * glow_radius;
                let sample_color = textureSample(input_texture, input_sampler, uv + offset).rgb;
                let luminance = dot(sample_color, vec3<f32>(0.299, 0.587, 0.114));
                if luminance > 0.5 {
                    glow += sample_color;
                    samples += 1.0;
                }
            }
        }

        if samples > 0.0 {
            glow /= samples;
            color += glow * uniforms.glow_intensity;
        }
    }

    if uniforms.vignette > 0.0 {
        let centered = in.uv - vec2<f32>(0.5);
        let vignette_amount = 1.0 - dot(centered, centered) * uniforms.vignette * 2.0;
        color *= max(0.0, vignette_amount);
    }

    if uniforms.film_grain > 0.0 {
        let grain = hash(original_uv * 1000.0 + vec2<f32>(time * 100.0, 0.0)) - 0.5;
        color += vec3<f32>(grain) * uniforms.film_grain * 0.15;
    }

    if uniforms.edge_glow > 0.0 {
        let texel = 1.0 / vec2<f32>(1920.0, 1080.0);
        let left = textureSample(input_texture, input_sampler, uv - vec2<f32>(texel.x, 0.0)).rgb;
        let right = textureSample(input_texture, input_sampler, uv + vec2<f32>(texel.x, 0.0)).rgb;
        let top = textureSample(input_texture, input_sampler, uv - vec2<f32>(0.0, texel.y)).rgb;
        let bottom = textureSample(input_texture, input_sampler, uv + vec2<f32>(0.0, texel.y)).rgb;

        let edge_h = abs(left - right);
        let edge_v = abs(top - bottom);
        let edge = (edge_h + edge_v) * 0.5;
        let edge_strength = length(edge);

        let edge_hue = fract(time * 0.1 + edge_strength * 2.0);
        let edge_color = hsv_to_rgb(vec3<f32>(edge_hue, 1.0, 1.0));
        color += edge_color * edge_strength * uniforms.edge_glow * 3.0;
    }

    if uniforms.speed_lines > 0.0 {
        let lines = speed_lines_effect(original_uv, time, uniforms.speed_lines);
        let line_color = hsv_to_rgb(vec3<f32>(fract(time * 0.1), 0.5, 1.0));
        color += line_color * lines * 0.5;
    }

    if uniforms.lens_flare > 0.0 {
        color += lens_flare_effect(original_uv, time, uniforms.lens_flare);
    }

    if uniforms.warp_speed > 0.0 {
        color += warp_speed_effect(original_uv, time, uniforms.warp_speed);
    }

    if uniforms.pulse_rings > 0.0 {
        color += pulse_rings_effect(original_uv, time, uniforms.pulse_rings);
    }

    if uniforms.digital_rain > 0.0 {
        let rain = digital_rain_effect(original_uv, time, uniforms.digital_rain);
        color = mix(color, color + rain, uniforms.digital_rain);
    }

    if uniforms.strobe > 0.0 {
        let strobe_flash = step(0.9, fract(time * 8.0)) * uniforms.strobe;
        color = mix(color, vec3<f32>(1.0), strobe_flash);
    }

    let grade_mode = i32(uniforms.color_grade_mode);
    if grade_mode > 0 {
        color = apply_color_grade(color, grade_mode);
    }

    if uniforms.saturation != 1.0 {
        let luminance = dot(color, vec3<f32>(0.299, 0.587, 0.114));
        color = mix(vec3<f32>(luminance), color, uniforms.saturation);
    }

    if uniforms.invert > 0.5 {
        color = vec3<f32>(1.0) - color;
    }

    color = pow(color, vec3<f32>(0.95));
    color = mix(color, color * color * (3.0 - 2.0 * color), 0.15);
    color = clamp(color, vec3<f32>(0.0), vec3<f32>(1.0));

    return vec4<f32>(color, 1.0);
}
