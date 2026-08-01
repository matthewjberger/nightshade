struct Uniform {
    time: f32,
    aspect: f32,
    width: f32,
    height: f32,
};

@group(0) @binding(0)
var<uniform> u: Uniform;

@group(1) @binding(0) var picture_sampler: sampler;
@group(1) @binding(1) var picture0: texture_2d<f32>;
@group(1) @binding(2) var picture1: texture_2d<f32>;
@group(1) @binding(3) var picture2: texture_2d<f32>;
@group(1) @binding(4) var picture3: texture_2d<f32>;
@group(1) @binding(5) var picture4: texture_2d<f32>;
@group(1) @binding(6) var picture5: texture_2d<f32>;
@group(1) @binding(7) var picture6: texture_2d<f32>;
@group(1) @binding(8) var picture7: texture_2d<f32>;

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

fn hash21(p: vec2<f32>) -> f32 {
    let cell = vec2<i32>(floor(p));
    let x = bitcast<u32>(cell.x) * 0x9e3779b1u;
    let y = mix_bits(bitcast<u32>(cell.y) + 0x85ebca77u);
    return f32(mix_bits(x ^ y)) / 4294967295.0;
}

fn noise2(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let s = f * f * (3.0 - 2.0 * f);
    let a = hash21(i);
    let b = hash21(i + vec2<f32>(1.0, 0.0));
    let c = hash21(i + vec2<f32>(0.0, 1.0));
    let d = hash21(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, s.x), mix(c, d, s.x), s.y);
}

fn fbm(p: vec2<f32>) -> f32 {
    var total = 0.0;
    var amount = 0.5;
    var q = p * 4.0;
    for (var index = 0; index < 5; index = index + 1) {
        total = total + amount * noise2(q);
        q = q * 2.03 + vec2<f32>(17.0, 9.0);
        amount = amount * 0.5;
    }
    return total;
}

fn spin(point: vec2<f32>, turn: f32) -> vec2<f32> {
    let angle = turn * 6.28318;
    let across = cos(angle);
    let lean = sin(angle);
    return vec2<f32>(
        point.x * across - point.y * lean,
        point.x * lean + point.y * across,
    );
}

fn luma(tint: vec3<f32>) -> f32 {
    return dot(tint, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn palette(turn: f32) -> vec4<f32> {
    let phase = vec3<f32>(0.0, 0.33, 0.67);
    let tint = 0.5 + 0.5 * cos(6.28318 * (turn + phase));
    return vec4<f32>(tint, 1.0);
}

fn sweep(seconds: f32, cycle: f32) -> f32 {
    let phase = fract(seconds / max(cycle, 0.001) * 0.5);
    let progress = 1.0 - abs(2.0 * phase - 1.0);
    return progress * progress * (3.0 - 2.0 * progress);
}

fn seam(signed: f32, width: f32) -> f32 {
    return 1.0 - smoothstep(0.0, max(width, 0.0001), abs(signed));
}

fn lattice(point: vec2<f32>, cells: f32) -> f32 {
    let cell = fract(point * cells) - vec2<f32>(0.5);
    let reach = 0.5 - max(abs(cell.x), abs(cell.y));
    return 1.0 - smoothstep(0.0, 0.06, reach);
}
