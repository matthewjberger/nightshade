
struct Phosphor {
    decay: f32,
    lift: f32,
    reach: f32,
    gain: f32,
}

@group(0) @binding(0) var current: texture_2d<f32>;
@group(0) @binding(1) var previous: texture_2d<f32>;
@group(0) @binding(2) var flat_sampler: sampler;
@group(0) @binding(3) var<uniform> phosphor: Phosphor;

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

fn held(uv: vec2<f32>, texel: vec2<f32>) -> vec3<f32> {
    let centre = textureSample(previous, flat_sampler, uv).rgb;
    if phosphor.reach <= 0.0 {
        return centre;
    }
    let step = vec2<f32>(texel.x * phosphor.reach, 0.0);
    let left = textureSample(previous, flat_sampler, uv - step).rgb;
    let right = textureSample(previous, flat_sampler, uv + step).rgb;
    return centre * 0.5 + (left + right) * 0.25;
}

@fragment
fn blendMain(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = 1.0 / vec2<f32>(textureDimensions(current));
    let now = textureSample(current, flat_sampler, in.uv).rgb;

    var trail = held(in.uv, texel) * phosphor.decay * phosphor.gain;
    let strength = max(trail.r, max(trail.g, trail.b));
    trail *= smoothstep(0.0, max(phosphor.lift, 0.0001), strength);

    return vec4<f32>(max(now, trail), 1.0);
}

@fragment
fn copyMain(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(textureSample(previous, flat_sampler, in.uv).rgb, 1.0);
}
