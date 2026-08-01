struct Draw {
    model_view_projection: mat4x4<f32>,
    rim: vec4<f32>,
    clock: vec4<f32>,
};

@group(0) @binding(0) var portal_texture: texture_2d<f32>;
@group(0) @binding(1) var portal_sampler: sampler;
@group(0) @binding(2) var<uniform> draw: Draw;

const HALO_EDGE: f32 = 1.34;
const BEND: f32 = 0.020;
const RIPPLE: f32 = 0.008;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) clip: vec4<f32>,
    @location(1) corner: vec2<f32>,
};

@vertex
fn vertex_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(-0.5, -0.5),
        vec2<f32>(0.5, -0.5),
        vec2<f32>(0.5, 0.5),
        vec2<f32>(-0.5, -0.5),
        vec2<f32>(0.5, 0.5),
        vec2<f32>(-0.5, 0.5),
    );
    let corner = corners[index] * HALO_EDGE;
    let placed = draw.model_view_projection * vec4<f32>(corner.x, corner.y, 0.0, 1.0);

    var output: VertexOutput;
    output.position = placed;
    output.clip = placed;
    output.corner = corner;
    return output;
}

fn ripple(angle: f32, clock: f32) -> f32 {
    return sin(angle * 7.0 + clock * 2.3) * 0.5
        + sin(angle * 13.0 - clock * 3.1) * 0.32
        + sin(angle * 23.0 + clock * 1.4) * 0.18;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let radius = length(input.corner * 2.0);
    let slope = vec2<f32>(dpdx(radius), dpdy(radius));
    let span = max(length(slope), 1.0e-6);

    let clock = draw.clock.x;
    let angle = atan2(input.corner.y, input.corner.x);
    let wave = ripple(angle, clock);
    let breath = 0.82 + 0.18 * sin(clock * 1.7);

    let ndc = input.clip.xy / max(input.clip.w, 1.0e-5);
    let flat = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
    let outward = slope / span;
    let pixels = vec2<f32>(textureDimensions(portal_texture));
    let pull = smoothstep(0.45, 1.0, radius) * (BEND + RIPPLE * wave);
    let uv = flat + outward * (pull / span) / pixels;

    let seen = textureSample(portal_texture, portal_sampler, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0)));
    let held = clamp(seen.rgb, vec3<f32>(0.0), vec3<f32>(0.995));
    let opened = held / (vec3<f32>(1.0) - held);

    let band = smoothstep(0.72, 1.0, radius) * (0.62 + 0.38 * wave);
    let line = exp(-pow((radius - 0.99) / 0.03, 2.0));
    let halo = (1.0 - smoothstep(1.0, HALO_EDGE, radius)) * (0.45 + 0.55 * wave);
    let tint = draw.rim.rgb * breath;

    if (radius > 1.0) {
        if (radius > HALO_EDGE) {
            discard;
        }
        return vec4<f32>(tint * (halo * 1.5 + line * 3.2), 0.0);
    }
    return vec4<f32>(opened + tint * (band * 2.6 + line * 3.2), 1.0);
}
