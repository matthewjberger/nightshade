#import nightshade_wgpu::sdf_common::{
    SDF_RANGE_CELLS,
    sdf,
    sdf_brick_touched,
    sdf_level_base,
    sdf_level_brick,
    sdf_level_cell,
    sdf_pointer_index,
}

@group(1) @binding(0)
var<storage, read_write> sdf_stir: array<u32>;

const SDF_MOVED_ROOM: u32 = 256u;
const MARK_THREADS: u32 = 256u;

@group(1) @binding(1)
var<uniform> sdf_moved: array<vec4<f32>, SDF_MOVED_ROOM>;

@compute @workgroup_size(8, 8, 4)
fn main(@builtin(workgroup_id) group: vec3<u32>, @builtin(local_invocation_index) local: u32) {
    let which = group.x;
    let level = group.y;
    if which >= min(sdf.flags.x, SDF_MOVED_ROOM) || level >= sdf.counts.y {
        return;
    }
    let ball = sdf_moved[which];
    let size = sdf_level_brick(level);
    let reach = ball.w + SDF_RANGE_CELLS * sdf_level_cell(level);
    let side = i32(sdf.counts.z);
    let base = sdf_level_base(level);
    let low = max(vec3<i32>(floor((ball.xyz - vec3<f32>(reach)) / size)), base);
    let high = min(
        vec3<i32>(floor((ball.xyz + vec3<f32>(reach)) / size)),
        base + vec3<i32>(side - 1)
    );
    if any(high < low) {
        return;
    }
    let extent = vec3<u32>(high - low + vec3<i32>(1));
    let total = extent.x * extent.y * extent.z;
    for (var at = local; at < total; at = at + MARK_THREADS) {
        let offset = vec3<i32>(
            i32(at % extent.x),
            i32((at / extent.x) % extent.y),
            i32(at / (extent.x * extent.y))
        );
        let brick = low + offset;
        let corner = vec3<f32>(brick) * size;
        if sdf_brick_touched(corner, corner + vec3<f32>(size), vec4<f32>(ball.xyz, reach)) {
            sdf_stir[sdf_pointer_index(level, brick)] = sdf.frame.x;
        }
    }
}
