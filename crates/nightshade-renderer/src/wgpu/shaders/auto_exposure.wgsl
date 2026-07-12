struct Output {
    target_log_luminance: f32,
    current_log_luminance: f32,
    adaptation_rate: f32,
    delta_time: f32,
    primed: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

@group(0) @binding(0) var hdr_texture: texture_2d<f32>;
@group(0) @binding(1) var<storage, read_write> output: Output;

const WORKGROUP_SIZE: u32 = 256u;
var<workgroup> shared_lum: array<f32, WORKGROUP_SIZE>;

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

@compute @workgroup_size(WORKGROUP_SIZE)
fn main(@builtin(local_invocation_id) lid: vec3<u32>) {
    let dims = textureDimensions(hdr_texture, 0);

    let stride = sqrt(f32(WORKGROUP_SIZE));
    let stride_int = u32(stride);
    let row = lid.x / stride_int;
    let col = lid.x % stride_int;

    let cell_w = max(dims.x / stride_int, 1u);
    let cell_h = max(dims.y / stride_int, 1u);
    let x = min(col * cell_w + cell_w / 2u, dims.x - 1u);
    let y = min(row * cell_h + cell_h / 2u, dims.y - 1u);

    let raw = textureLoad(hdr_texture, vec2<i32>(i32(x), i32(y)), 0).rgb;
    // Drop non-finite samples so a NaN or Inf from a refractive material can't
    // poison the running mean, which would latch the whole frame black.
    let clean = clamp(select(vec3<f32>(0.0), raw, raw == raw), vec3<f32>(0.0), vec3<f32>(65504.0));
    let lum = max(luminance(clean), 1e-4);
    // Clamp the per-sample contribution so a bright HDR sun or specular
    // hot-spot can't pull the mean far above the bulk of the scene.
    shared_lum[lid.x] = clamp(log2(lum), -10.0, 4.0);

    workgroupBarrier();

    var stride2 = WORKGROUP_SIZE / 2u;
    while stride2 > 0u {
        if lid.x < stride2 {
            shared_lum[lid.x] = shared_lum[lid.x] + shared_lum[lid.x + stride2];
        }
        workgroupBarrier();
        stride2 = stride2 / 2u;
    }

    if lid.x == 0u {
        let mean_log_lum = shared_lum[0] / f32(WORKGROUP_SIZE);
        let rate = output.adaptation_rate;
        let dt = output.delta_time;
        let alpha = 1.0 - exp(-rate * dt);
        // If a prior frame ever latched the accumulator to NaN, drop it and
        // re-seed from the current mean instead of carrying NaN forward.
        let prev = select(
            mean_log_lum,
            output.current_log_luminance,
            output.current_log_luminance == output.current_log_luminance,
        );
        let new_current = select(
            mean_log_lum,
            prev + (mean_log_lum - prev) * alpha,
            output.primed != 0u,
        );
        output.target_log_luminance = mean_log_lum;
        output.current_log_luminance = new_current;
        output.primed = 1u;
    }
}
