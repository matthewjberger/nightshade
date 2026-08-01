#import nightshade_wgpu::sdf_common::{
    sdf,
}

@group(1) @binding(0)
var cast_depth: texture_2d<f32>;

struct Written {
    @builtin(frag_depth) depth: f32,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) @invariant vec4<f32> {
    let half = i32(vertex_index) / 2;
    let odd = i32(vertex_index) & 1;
    return vec4<f32>(f32(half) * 4.0 - 1.0, f32(odd) * 4.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> Written {
    let shrink = max(sdf.flags.w, 1u);
    let texel = vec2<u32>(position.xy) / shrink;
    let depth = textureLoad(cast_depth, vec2<i32>(texel), 0).r;
    if depth <= 0.0 {
        discard;
    }
    return Written(depth);
}
