
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

const DARK_SHARE: f32 = 0.05;
const BRIGHT_SHARE: f32 = 0.95;

var<workgroup> counts: array<u32, 256>;

fn brightness_of(bin: u32) -> f32 {
    return exposure.darkest
        + f32(bin) / 255.0 * (exposure.brightest - exposure.darkest);
}

@compute @workgroup_size(256)
fn main(@builtin(local_invocation_index) slot: u32) {
    counts[slot] = atomicLoad(&exposure.histogram[slot]);
    atomicStore(&exposure.histogram[slot], 0u);
    workgroupBarrier();

    if slot != 0u {
        return;
    }

    var total = 0u;
    for (var bin = 0u; bin < 256u; bin = bin + 1u) {
        total = total + counts[bin];
    }
    if total == 0u {
        return;
    }

    let first = f32(total) * DARK_SHARE;
    let last = f32(total) * BRIGHT_SHARE;
    var weighted = 0.0;
    var weight = 0.0;
    var seen = 0.0;
    for (var bin = 0u; bin < 256u; bin = bin + 1u) {
        let count = f32(counts[bin]);
        let lower = max(seen, first);
        let upper = min(seen + count, last);
        if upper > lower {
            weighted = weighted + (upper - lower) * brightness_of(bin);
            weight = weight + (upper - lower);
        }
        seen = seen + count;
    }

    var measured = exposure.target_log_luminance;
    if weight > 0.0 {
        measured = weighted / weight;
    }
    let follow = 1.0 - exp(-exposure.adaptation_rate * exposure.delta_time);
    var settled = measured;
    if exposure.primed != 0u {
        settled = exposure.current_log_luminance
            + (measured - exposure.current_log_luminance) * follow;
    }
    exposure.target_log_luminance = measured;
    exposure.current_log_luminance = clamp(settled, exposure.darkest, exposure.brightest);
    exposure.primed = 1u;
}
