#define_import_path nightshade_wgpu::fog_common

const FOG_X: u32 = 160u;
const FOG_Y: u32 = 90u;
const FOG_Z: u32 = 64u;
const FOG_NEAR: f32 = 0.5;

struct FogUniforms {
    air: vec4<f32>,
    screen: vec2<f32>,
    focal: vec2<f32>,
    far: f32,
    frame: u32,
    volumetric: u32,
    resolving: u32,
};

fn fog_far(end: f32) -> f32 {
    return max(end, FOG_NEAR * 2.0);
}

fn fog_depth_of(slice: f32, end: f32) -> f32 {
    let far = fog_far(end);
    let along = slice / f32(FOG_Z);
    return FOG_NEAR + (far - FOG_NEAR) * along * along;
}

fn fog_slice_dither(fragment: vec2<f32>, carried: f32) -> f32 {
    let noise = fract(52.9829189 * fract(dot(fragment, vec2<f32>(0.06711056, 0.00583715))));
    return fract(noise + carried) - 0.5;
}

fn fog_slice_of(view_depth: f32, end: f32) -> f32 {
    let far = fog_far(end);
    let along = sqrt(max(view_depth - FOG_NEAR, 0.0) / (far - FOG_NEAR));
    return clamp(along * f32(FOG_Z), 0.0, f32(FOG_Z));
}
