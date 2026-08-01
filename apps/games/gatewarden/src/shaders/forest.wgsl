struct Uniform {
    time: f32,
    aspect: f32,
    width: f32,
    height: f32,
    octaves: f32,
    mist: f32,
    flies: f32,
    dither: f32,
};

@group(0) @binding(0)
var<uniform> u: Uniform;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    let x = f32(i32(index & 1u)) * 4.0 - 1.0;
    let y = f32(i32(index >> 1u)) * 4.0 - 1.0;
    var out: VertexOutput;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x * 0.5 + 0.5, 1.0 - (y * 0.5 + 0.5));
    return out;
}

fn mix_bits(value: u32) -> u32 {
    var h = value;
    h = h ^ (h >> 16u);
    h = h * 0x7feb352du;
    h = h ^ (h >> 15u);
    h = h * 0x846ca68bu;
    h = h ^ (h >> 16u);
    return h;
}

fn hash11(x: f32) -> f32 {
    let cell = bitcast<u32>(i32(floor(x)));
    return f32(mix_bits(cell ^ 0x27d4eb2fu)) / 4294967295.0;
}

fn hash21(p: vec2<f32>) -> f32 {
    let cell = vec2<i32>(floor(p));
    let x = bitcast<u32>(cell.x) * 0x9e3779b1u;
    let y = mix_bits(bitcast<u32>(cell.y) + 0x85ebca77u);
    return f32(mix_bits(x ^ y)) / 4294967295.0;
}

fn noise1(x: f32) -> f32 {
    let i = floor(x);
    let f = fract(x);
    let s = f * f * (3.0 - 2.0 * f);
    return mix(hash11(i), hash11(i + 1.0), s);
}

fn noise2(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let s = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
    let a = hash21(i);
    let b = hash21(i + vec2<f32>(1.0, 0.0));
    let c = hash21(i + vec2<f32>(0.0, 1.0));
    let d = hash21(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, s.x), mix(c, d, s.x), s.y);
}

fn fbm(p: vec2<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var q = p;
    for (var index = 0; index < octaves; index = index + 1) {
        value = value + amplitude * noise2(q);
        q = q * 2.03 + vec2<f32>(17.0, 9.0);
        amplitude = amplitude * 0.5;
    }
    return value;
}

fn pine(x: f32) -> f32 {
    let cell = floor(x);
    let f = fract(x);
    let height = 0.35 + hash11(cell) * 0.65;
    let width = 0.3 + hash11(cell + 7.0) * 0.2;
    let d = abs(f - 0.5) / width;
    return max(height * (1.0 - d), 0.0);
}

fn treeline(x: f32, seed: f32, scale: f32) -> f32 {
    let rolling = noise1(x * 0.9 + seed) * 0.35;
    let near = pine(x * scale + seed * 7.0);
    let far = pine(x * scale * 1.9 + seed * 3.0 + 0.5) * 0.6;
    return rolling + max(near, far);
}

fn firefly(p: vec2<f32>, time: f32, seed: f32) -> f32 {
    let cell = floor(p);
    let id = hash21(cell + seed);
    if (id < 0.55) {
        return 0.0;
    }
    let speed = 0.4 + id * 0.6;
    let phase = id * 6.28318;
    let center = cell + vec2<f32>(0.5, 0.5)
        + vec2<f32>(sin(time * speed + phase), cos(time * speed * 0.7 + phase * 1.3)) * 0.32;
    let d = length(p - center);
    let pulse = 0.55 + 0.45 * sin(time * (1.5 + id * 2.0) + phase);
    return pulse * 0.00045 / (d * d + 0.0004);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let p = vec2<f32>(uv.x * u.aspect, uv.y);
    let time = u.time;

    let sky_top = vec3<f32>(0.03, 0.02, 0.09);
    let sky_mid = vec3<f32>(0.13, 0.08, 0.26);
    let sky_low = vec3<f32>(0.08, 0.22, 0.30);
    var color = mix(sky_top, sky_mid, smoothstep(0.0, 0.45, uv.y));
    color = mix(color, sky_low, smoothstep(0.42, 0.66, uv.y));

    let octaves = i32(u.octaves);
    let warp = vec2<f32>(
        fbm(p * 1.6 + vec2<f32>(time * 0.03, 0.0), octaves),
        fbm(p * 1.6 + vec2<f32>(0.0, time * 0.02) + vec2<f32>(5.2, 1.3), octaves)
    );
    let veil = fbm(p * 2.2 + warp * 1.4 + vec2<f32>(time * 0.015, -time * 0.01), octaves);
    let band = smoothstep(0.45, 0.8, veil) * (1.0 - smoothstep(0.35, 0.7, uv.y)) * smoothstep(0.02, 0.2, uv.y);
    color = color + vec3<f32>(0.12, 0.42, 0.46) * band * 0.7;
    color = color + vec3<f32>(0.35, 0.15, 0.5) * smoothstep(0.55, 0.9, fbm(p * 1.3 - warp + vec2<f32>(time * 0.01, 0.0), octaves)) * (1.0 - smoothstep(0.3, 0.65, uv.y)) * 0.5;

    let star_cell = floor(p * 160.0);
    let star = hash21(star_cell);
    if (star > 0.992 && uv.y < 0.58) {
        let brightness = 0.45 + 0.55 * hash21(star_cell + vec2<f32>(7.0, 3.0));
        let restless = hash21(star_cell + vec2<f32>(31.0, 17.0));
        var twinkle = 1.0;
        if (restless > 0.8) {
            twinkle = 0.8 + 0.2 * sin(time * (0.6 + restless * 1.4) + restless * 40.0);
        }
        let inside = length(fract(p * 160.0) - 0.5);
        color = color + vec3<f32>(0.8, 0.85, 1.0) * brightness * twinkle * (1.0 - smoothstep(0.0, 0.35, inside)) * (1.0 - smoothstep(0.3, 0.58, uv.y));
    }

    let moon_center = vec2<f32>(0.78 * u.aspect, 0.2);
    let moon_d = length(p - moon_center);
    let moon = (1.0 - smoothstep(0.056, 0.062, moon_d));
    let glow = 0.03 / (moon_d + 0.05);
    color = color + vec3<f32>(0.55, 0.6, 0.8) * glow * 0.6;
    color = mix(color, vec3<f32>(0.92, 0.9, 0.82), moon);

    let horizon = 0.64;
    let far_color = vec3<f32>(0.12, 0.17, 0.27);
    let mid_color = vec3<f32>(0.07, 0.11, 0.18);
    let close_color = vec3<f32>(0.035, 0.065, 0.10);
    let near_color = vec3<f32>(0.015, 0.03, 0.045);

    let far_top = horizon - 0.12 * treeline(p.x + time * 0.004, 1.0, 16.0);
    if (uv.y > far_top) {
        color = far_color;
    }
    if (u.mist > 0.5) {
        let mist_far = fbm(p * 3.0 + vec2<f32>(time * 0.02, 0.0), octaves) * (1.0 - smoothstep(far_top - 0.02, far_top + 0.15, uv.y));
        color = mix(color, vec3<f32>(0.2, 0.3, 0.38), mist_far * 0.45);
    }

    let mid_top = horizon + 0.08 - 0.2 * treeline(p.x + time * 0.008, 2.0, 9.0);
    if (uv.y > mid_top) {
        color = mid_color;
    }
    if (u.mist > 0.5) {
        let mist_mid = fbm(p * 2.5 + vec2<f32>(-time * 0.025, 3.0), octaves) * (1.0 - smoothstep(mid_top - 0.03, mid_top + 0.18, uv.y));
        color = mix(color, vec3<f32>(0.14, 0.24, 0.3), mist_mid * 0.5);
    }

    let close_top = horizon + 0.22 - 0.3 * treeline(p.x + time * 0.013 + sin(time * 0.3) * 0.004, 3.0, 5.5);
    if (uv.y > close_top) {
        color = close_color;
    }

    let near_top = horizon + 0.38 - 0.42 * treeline(p.x + time * 0.02 + sin(time * 0.25 + p.x * 3.0) * 0.006, 4.0, 3.2);
    if (uv.y > near_top) {
        color = near_color;
    }
    if (u.mist > 0.5) {
        let ground_mist = fbm(p * 3.5 + vec2<f32>(time * 0.03, -time * 0.01), octaves) * smoothstep(0.55, 1.0, uv.y);
        color = mix(color, vec3<f32>(0.1, 0.2, 0.22), ground_mist * 0.35);
    }

    if (u.flies > 0.5) {
        let fly_zone = smoothstep(0.35, 0.6, uv.y);
        let flies = firefly(p * 7.0 + vec2<f32>(time * 0.05, 0.0), time, 1.0)
            + firefly(p * 11.0 + vec2<f32>(-time * 0.03, 2.0), time * 1.3, 2.0) * 0.6;
        color = color + vec3<f32>(0.85, 1.0, 0.45) * flies * fly_zone;
    }

    let vignette = (1.0 - smoothstep(0.35, 1.15, length((uv - 0.5) * vec2<f32>(1.15, 1.0))));
    color = color * (0.55 + 0.45 * vignette);

    let steps = u.dither;
    let dither = (hash21(in.position.xy) - 0.5) / steps;
    color = floor(color * steps + dither * steps + 0.5) / steps;
    return vec4<f32>(max(color, vec3<f32>(0.0)), 1.0);
}
