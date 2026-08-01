
struct Intro {
    time: f32,
    width: f32,
    height: f32,
    darkness: f32,
    cover: f32,
    glitch: f32,
    sun: f32,
    grid_speed: f32,
}

@group(0) @binding(0) var<uniform> intro: Intro;

const CYCLE: f32 = 3.2;
const TH: f32 = 0.045;
const BASELINE: f32 = -0.22;
const X_HEIGHT: f32 = 0.17;
const CENTER_I: f32 = -0.53;
const CENTER_A: f32 = -0.22;
const CENTER_M: f32 = 0.31;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn fullscreenVertex(@builtin(vertex_index) index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((index << 1u) & 2u);
    let y = f32(index & 2u);
    out.uv = vec2<f32>(x, y);
    out.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    return out;
}

fn palette(t: f32, a: vec3<f32>, b: vec3<f32>, c: vec3<f32>, d: vec3<f32>) -> vec3<f32> {
    return a + b * cos(6.28318 * (c * t + d));
}

fn hash11(seed: f32) -> f32 {
    var p = fract(seed * 0.1031);
    p = p * (p + 33.33);
    p = p * (p + p);
    return fract(p);
}

fn hash12(seed: vec2<f32>) -> f32 {
    var p = fract(vec3<f32>(seed.x, seed.y, seed.x) * 0.1031);
    p = p + dot(p, p.yzx + 33.33);
    return fract((p.x + p.y) * p.z);
}

fn noise11(at: f32) -> f32 {
    let whole = floor(at);
    let part = at - whole;
    let eased = part * part * (3.0 - 2.0 * part);
    return mix(hash11(whole), hash11(whole + 1.0), eased);
}

fn noise12(at: vec2<f32>) -> f32 {
    let whole = floor(at);
    let part = at - whole;
    let eased = part * part * (3.0 - 2.0 * part);
    return mix(
        mix(hash12(whole), hash12(whole + vec2<f32>(1.0, 0.0)), eased.x),
        mix(
            hash12(whole + vec2<f32>(0.0, 1.0)),
            hash12(whole + vec2<f32>(1.0, 1.0)),
            eased.x,
        ),
        eased.y,
    );
}

fn sd_box(at: vec2<f32>, half: vec2<f32>) -> f32 {
    let from_face = abs(at) - half;
    return length(max(from_face, vec2<f32>(0.0))) + min(max(from_face.x, from_face.y), 0.0);
}

fn sd_ring(at: vec2<f32>, radius: f32, thickness: f32) -> f32 {
    return abs(length(at) - radius) - thickness;
}

fn sd_arch(at: vec2<f32>, radius: f32, thickness: f32) -> f32 {
    let upper = abs(length(at) - radius) - thickness;
    let lower = length(vec2<f32>(abs(at.x) - radius, at.y)) - thickness;
    return select(lower, upper, at.y > 0.0);
}

fn glyph_i(at: vec2<f32>) -> f32 {
    let p = at - vec2<f32>(CENTER_I, 0.0);
    let stem = sd_box(
        p - vec2<f32>(0.0, (BASELINE + X_HEIGHT) * 0.5),
        vec2<f32>(TH, (X_HEIGHT - BASELINE) * 0.5),
    );
    let tittle = sd_box(p - vec2<f32>(0.0, 0.30), vec2<f32>(TH, TH)) - 0.012;
    return min(stem, tittle);
}

fn glyph_a(at: vec2<f32>) -> f32 {
    let p = at - vec2<f32>(CENTER_A, 0.0);
    let bowl = sd_ring(p - vec2<f32>(0.0, -0.025), 0.145, TH);
    let stem = sd_box(p - vec2<f32>(0.145, -0.045), vec2<f32>(TH, 0.20));
    return min(bowl, stem);
}

fn glyph_m(at: vec2<f32>) -> f32 {
    let p = at - vec2<f32>(CENTER_M, 0.0);
    var d = sd_arch(p - vec2<f32>(-0.115, 0.02), 0.115, TH);
    d = min(d, sd_arch(p - vec2<f32>(0.115, 0.02), 0.115, TH));
    d = min(d, sd_box(p - vec2<f32>(-0.23, -0.025), vec2<f32>(TH, 0.195)));
    d = min(d, sd_box(p - vec2<f32>(0.0, -0.025), vec2<f32>(TH, 0.195)));
    return min(d, sd_box(p - vec2<f32>(0.23, -0.025), vec2<f32>(TH, 0.195)));
}

fn logo(at: vec2<f32>, showing_i: f32, showing_am: f32) -> f32 {
    let first = glyph_i(at) + (1.0 - showing_i) * 8.0;
    let rest = min(glyph_a(at), glyph_m(at)) + (1.0 - showing_am) * 8.0;
    return min(first, rest);
}

fn gate(t: f32, starts: f32, ends: f32) -> f32 {
    if t < starts || t > ends {
        return 0.0;
    }
    if t - starts < 0.16 {
        return step(0.35, hash11(floor(t * 45.0) + starts * 7.0));
    }
    return 1.0;
}

fn onset(t: f32, starts: f32, ends: f32) -> f32 {
    if t < starts || t > ends {
        return 0.0;
    }
    return exp(-(t - starts) * 7.0);
}

fn sun_grid(uv: vec2<f32>) -> f32 {
    let horizon = -0.46;
    let below = max(horizon - uv.y, 0.0);
    let ruled = vec2<f32>(
        uv.x / (below + 0.03) * 0.45,
        0.32 / (below + 0.03) + intro.time * intro.grid_speed,
    );
    let width = fwidth(ruled);
    let from_line = abs(fract(ruled) - 0.5);
    let lit = smoothstep(width * 2.0, width, from_line);
    let density = clamp(vec2<f32>(1.0) - width * 3.0, vec2<f32>(0.0), vec2<f32>(1.0));
    return clamp(max(lit.x * density.x, lit.y * density.y), 0.0, 1.0)
        * smoothstep(0.0, 0.1, horizon - uv.y)
        * step(0.001, below);
}

@fragment
fn fragmentMain(in: VertexOutput) -> @location(0) vec4<f32> {
    let size = vec2<f32>(intro.width, intro.height);
    let pixel = vec2<f32>(in.uv.x, 1.0 - in.uv.y) * size;
    let uv = (pixel + pixel - size) / size.y;

    let t = intro.time - CYCLE * floor(intro.time / CYCLE);
    let showing_i = max(gate(t, 0.0, 0.75), gate(t, 2.0, CYCLE));
    let showing_am = max(gate(t, 1.0, 1.75), gate(t, 2.0, CYCLE));
    let flash = clamp(
        max(
            max(onset(t, 0.0, 0.75), onset(t, 1.0, 1.75)),
            onset(t, 2.0, CYCLE) * 1.3,
        ),
        0.0,
        1.0,
    );

    var slip = noise11(floor(uv.y * 20.0)) * sin(floor(uv.y * 0.5) + intro.time);
    slip = slip * slip * slip * intro.glitch;

    var reach = (uv.x + slip * 0.05) * 20.0;
    var stripe = floor(reach);
    reach = reach - stripe;

    var tone = hash11(stripe - 1.0);
    tone = mix(hash11(stripe), tone, step(reach, sin(intro.time + tone * 10.0) * 0.4 + 0.4));
    tone = sin(tone * 0.7 - noise11(uv.x - intro.time * 1.2) * 2.0) * 0.5 + 0.5;

    var color = palette(
        tone,
        vec3<f32>(0.30, 0.42, 0.48),
        vec3<f32>(0.28, 0.42, 0.42),
        vec3<f32>(1.0),
        vec3<f32>(0.65, 0.42, 0.20),
    ) * 0.75 - 0.42;
    color = color * (0.5 + sin(uv.y * 600.0) * 0.5);

    let from_sun = length((uv - vec2<f32>(0.0, 0.02)) * vec2<f32>(0.85, 1.0));
    let disc = smoothstep(0.72, 0.70, from_sun);
    let band = step(fract((uv.y + 0.5) * 7.0), clamp(uv.y * 1.6 + 1.05, 0.0, 1.0));
    let sun_color = mix(
        vec3<f32>(0.00, 0.60, 0.84),
        vec3<f32>(0.86, 0.78, 0.03),
        clamp(uv.y * 1.2 + 0.5, 0.0, 1.0),
    );
    color = mix(color, sun_color, disc * band * intro.sun);
    color = mix(color, vec3<f32>(0.05, 0.88, 1.0), sun_grid(uv) * 0.42);

    var drop = noise11(floor((uv.x + slip * 0.05) * 200.0) + intro.time)
        * noise12(vec2<f32>(uv.x * 5.0, intro.time));
    color = mix(color, vec3<f32>(1.0, 0.23, 0.12), drop * drop * drop * 0.7);

    drop = noise11(floor(uv.y * 200.0) + intro.time)
        * noise12(vec2<f32>(uv.y * 10.0, 0.0) + intro.time);
    drop = drop * drop * drop;
    color = mix(color, vec3<f32>(0.00, 0.94, 1.0), drop);

    var at = uv + vec2<f32>((slip - drop) * 0.05, 0.0);

    let blocks = 20.0;
    var damage = noise11(intro.time);
    damage = damage * damage * damage * noise12(floor(uv * blocks));
    at = ((at * blocks - floor(at * blocks)) * (1.0 - damage * 0.6) + floor(at * blocks)) / blocks;
    damage = clamp(damage + flash * 0.6, 0.0, 1.0);

    let scale = 0.70;
    let letter_scale = vec2<f32>(scale - noise11(intro.time) * 0.06, scale);

    var wobble = noise11(floor(intro.time * 20.0) / 4.0 + 0.2) * 6.28;
    var edge = logo(
        at * letter_scale + vec2<f32>(-1.0, 1.0) * 0.012
            + vec2<f32>(sin(wobble), cos(wobble)) * 0.012,
        showing_i,
        showing_am,
    );
    color = mix(color, vec3<f32>(0.0), smoothstep(0.5, 0.0, edge) * 0.55);
    let side = smoothstep(0.01, -0.03, edge);
    color = mix(
        color,
        vec3<f32>(side * 0.55, side * 0.42, 0.0),
        smoothstep(0.005, 0.0, edge - 0.01),
    );
    color = mix(
        color,
        vec3<f32>(0.00, 0.92, 1.0),
        smoothstep(0.005, -0.005, abs(edge - 0.02)) * damage * 0.5,
    );

    wobble = noise11(floor(intro.time * 20.0) / 4.0) * 6.28;
    edge = logo(
        at * letter_scale + vec2<f32>(sin(wobble), cos(wobble)) * 0.012,
        showing_i,
        showing_am,
    );

    let up = clamp((uv.y + 0.32) / 0.72, 0.0, 1.0);
    var face = mix(vec3<f32>(0.00, 0.60, 0.84), vec3<f32>(0.86, 0.78, 0.03), up);
    face = mix(face, face.gbr, clamp(abs(slip) * 3.0, 0.0, 1.0) * 0.35);
    let bars = step(0.5, fract((uv.y + 0.06) * 12.0 - intro.time * 0.3))
        * smoothstep(0.2, -0.35, uv.y);
    face = mix(face, face * 0.08, bars * 0.75);
    face = mix(face, vec3<f32>(1.0, 1.0, 0.92), flash * 0.8);

    color = mix(
        color,
        vec3<f32>(0.70, 1.0, 0.10) * (1.0 + flash * 3.0),
        smoothstep(0.12, 0.0, edge) * 0.16,
    );
    color = mix(color, face, smoothstep(0.005, 0.0, edge));
    color = mix(
        color,
        vec3<f32>(0.0),
        smoothstep(2.0 / size.y, 0.0, abs(edge - 0.002) - 0.001) * 0.7,
    );
    color = mix(
        color,
        vec3<f32>(0.00, 0.96, 1.0),
        smoothstep(0.005, -0.005, abs(edge - 0.018)) * 0.55 * damage,
    );

    color = color * (0.9 + sin(uv.y * 600.0) * 0.2);
    return vec4<f32>(color * intro.darkness, intro.cover);
}
