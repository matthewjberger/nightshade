struct Glass {
    time: f32,
    amount: f32,
    curvature: f32,
    vignette: f32,
    scanlines: f32,
    scanline_pitch: f32,
    grille: f32,
    bleed: f32,
    warm: f32,
    flicker: f32,
    noise: f32,
    gain: f32,
};

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> glass: Glass;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn fullscreen_vertex(@builtin(vertex_index) index: u32) -> VertexOutput {
    var output: VertexOutput;
    let corner = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    output.position = vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
    output.uv = vec2<f32>(corner.x, 1.0 - corner.y);
    return output;
}

fn grain_at(point: vec2<f32>) -> f32 {
    return fract(sin(dot(point, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let amount = clamp(glass.amount, 0.0, 1.0);
    let extent = vec2<f32>(textureDimensions(source_texture));

    let across = input.uv * 2.0 - 1.0;
    let radius = dot(across, across);
    let curved = across * (1.0 + radius * glass.curvature);

    let shift = vec2<f32>(glass.bleed * amount / extent.x, 0.0);
    let red = textureSample(source_texture, source_sampler, input.uv + shift).r;
    let green = textureSample(source_texture, source_sampler, input.uv).g;
    let blue = textureSample(source_texture, source_sampler, input.uv - shift).b;
    var color = vec3<f32>(red, green, blue);

    let line = sin((curved.y * 0.5 + 0.5) * extent.y * glass.scanline_pitch) * 0.5 + 0.5;
    color = color * (1.0 - glass.scanlines * amount * line);

    let triad = sin((curved.x * 0.5 + 0.5) * extent.x * 3.1) * 0.5 + 0.5;
    let grille = vec3<f32>(
        1.0 - glass.grille * triad,
        1.0 - glass.grille * (1.0 - triad),
        1.0 - glass.grille * triad * 0.5,
    );
    color = color * mix(vec3<f32>(1.0), grille, amount);

    let waver = 1.0 - glass.flicker * amount * (0.5 + 0.5 * sin(glass.time * 47.0));
    color = color * waver;

    let grain = grain_at(input.uv * extent + vec2<f32>(glass.time * 60.0, 0.0)) - 0.5;
    color = color + grain * glass.noise * amount;

    color = color * mix(1.0, glass.gain, amount);

    let falloff = 1.0 - glass.vignette * amount * radius;
    color = color * clamp(falloff, 0.0, 1.0);

    let edge = pow(abs(across.x), 6.0) + pow(abs(across.y), 6.0);
    let corner = 1.0 - smoothstep(1.0, 1.6, edge);
    color = color * mix(1.0, corner, amount);

    let warm = vec3<f32>(
        1.0 + glass.warm * 0.20,
        1.0 + glass.warm * 0.04,
        1.0 - glass.warm * 0.14,
    );
    color = color * mix(vec3<f32>(1.0), warm, amount);

    return vec4<f32>(color, 1.0);
}
