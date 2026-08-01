#import nightshade_wgpu::fog_common::{FogUniforms, FOG_X, FOG_Y, FOG_Z, fog_depth_of}

@group(0) @binding(0)
var<uniform> fog: FogUniforms;

@group(0) @binding(1)
var fog_scatter: texture_3d<f32>;

@group(0) @binding(2)
var fog_marched: texture_storage_3d<rgba16float, write>;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let column = id.xy;
    if column.x >= FOG_X || column.y >= FOG_Y {
        return;
    }
    let ndc = vec2<f32>(
        (f32(column.x) + 0.5) / f32(FOG_X) * 2.0 - 1.0,
        1.0 - (f32(column.y) + 0.5) / f32(FOG_Y) * 2.0
    );
    let along = length(vec3<f32>(ndc.x / fog.focal.x, ndc.y / fog.focal.y, 1.0));
    var reached = vec3<f32>(0.0);
    var showing = 1.0;
    var nearer = fog_depth_of(0.0, fog.far);
    for (var slice = 0u; slice < FOG_Z; slice = slice + 1u) {
        let farther = fog_depth_of(f32(slice) + 1.0, fog.far);
        let crossed = (farther - nearer) * along;
        let cell = textureLoad(fog_scatter, vec3<i32>(vec2<i32>(column), i32(slice)), 0);
        let extinction = max(cell.a, 0.0);
        let through = exp(-extinction * crossed);
        var added = cell.rgb * crossed;
        if extinction > 1.0e-5 {
            added = cell.rgb / extinction * (1.0 - through);
        }
        reached = reached + showing * added;
        showing = showing * through;
        textureStore(fog_marched, vec3<i32>(vec2<i32>(column), i32(slice)), vec4<f32>(reached, showing));
        nearer = farther;
    }
}
