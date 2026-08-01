#import nightshade_wgpu::fog_common::FogUniforms

@group(0) @binding(0)
var<uniform> fog: FogUniforms;

@group(0) @binding(1)
var fog_volume: texture_3d<f32>;

@group(0) @binding(2)
var fog_sampler: sampler;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) @invariant vec4<f32> {
    let half = i32(vertex_index) / 2;
    let odd = i32(vertex_index) & 1;
    return vec4<f32>(f32(half) * 4.0 - 1.0, f32(odd) * 4.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return textureSampleLevel(fog_volume, fog_sampler, vec3<f32>(position.xy / fog.screen, 1.0), 0.0);
}
