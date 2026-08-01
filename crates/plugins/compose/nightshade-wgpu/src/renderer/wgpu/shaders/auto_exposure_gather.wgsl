
struct Exposure {
    target_log_luminance: f32,
    current_log_luminance: f32,
    adaptation_rate: f32,
    delta_time: f32,
    primed: u32,
    darkest: f32,
    brightest: f32,
    padding: u32,
    histogram: array<atomic<u32>, 256>,
};

@group(0) @binding(0) var hdr_texture: texture_2d<f32>;
@group(0) @binding(1) var<storage, read_write> exposure: Exposure;

var<workgroup> counts: array<atomic<u32>, 256>;

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

@compute @workgroup_size(16, 16)
fn main(
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(local_invocation_index) slot: u32,
) {
    atomicStore(&counts[slot], 0u);
    workgroupBarrier();

    let dims = textureDimensions(hdr_texture, 0);
    let point = gid.xy * 2u;
    if point.x < dims.x && point.y < dims.y {
        let raw = textureLoad(hdr_texture, vec2<i32>(point), 0).rgb;
        let clean = clamp(
            select(vec3<f32>(0.0), raw, raw == raw),
            vec3<f32>(0.0),
            vec3<f32>(65504.0)
        );
        let darkest = exposure.darkest;
        let brightest = exposure.brightest;
        let reading = clamp(log2(max(luminance(clean), 1.0e-4)), darkest, brightest);
        let bin = u32((reading - darkest) / max(brightest - darkest, 1.0e-4) * 255.0 + 0.5);
        atomicAdd(&counts[bin], 1u);
    }
    workgroupBarrier();

    let found = atomicLoad(&counts[slot]);
    if found > 0u {
        atomicAdd(&exposure.histogram[slot], found);
    }
}
