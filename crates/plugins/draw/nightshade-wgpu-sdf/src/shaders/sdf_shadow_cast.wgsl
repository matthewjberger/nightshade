#import nightshade_wgpu::sdf_common::{
    SDF_CASCADES,
    sdf,
}
#import nightshade_wgpu::sdf_cache::{
    sdf_walk,
}

@group(2) @binding(0)
var cast_depth: texture_storage_2d<r32float, write>;

const SLOTS_PER_ROW: u32 = 2u;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(cast_depth);
    if id.x >= size.x || id.y >= size.y {
        return;
    }
    let slot_size = size / SLOTS_PER_ROW;
    let slot = id.xy / slot_size;
    let cascade = slot.y * SLOTS_PER_ROW + slot.x;
    if cascade >= SDF_CASCADES {
        textureStore(cast_depth, vec2<i32>(id.xy), vec4<f32>(0.0));
        return;
    }
    let local = (vec2<f32>(id.xy - slot * slot_size) + vec2<f32>(0.5)) / vec2<f32>(slot_size);
    let ndc = vec2<f32>(local.x * 2.0 - 1.0, 1.0 - local.y * 2.0);
    let inverse = sdf.cascade_inverse_view_projections[cascade];
    let start_clip = inverse * vec4<f32>(ndc, 1.0, 1.0);
    let stop_clip = inverse * vec4<f32>(ndc, 0.0, 1.0);
    let start = start_clip.xyz / start_clip.w;
    let stop = stop_clip.xyz / stop_clip.w;
    let span = stop - start;
    let limit = length(span);
    if limit <= 0.0 {
        textureStore(cast_depth, vec2<i32>(id.xy), vec4<f32>(0.0));
        return;
    }
    let walk = sdf_walk(start, span / limit, limit, 0.0);
    var depth = 0.0;
    if walk.hit {
        let clip = sdf.cascade_view_projections[cascade] * vec4<f32>(walk.point, 1.0);
        depth = clamp(clip.z / clip.w, 0.0, 1.0);
    }
    textureStore(cast_depth, vec2<i32>(id.xy), vec4<f32>(depth, 0.0, 0.0, 0.0));
}
