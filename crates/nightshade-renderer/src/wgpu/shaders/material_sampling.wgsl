#define_import_path nightshade::material_sampling

const NO_LAYER: u32 = 0xFFFFFFFFu;

fn apply_wrap_axis(value: f32, mode: u32) -> f32 {
    if mode == 0u {
        return fract(value);
    } else if mode == 1u {
        let cycle = value - 2.0 * floor(value * 0.5);
        return min(cycle, 2.0 - cycle);
    }
    return clamp(value, 0.0, 1.0);
}

fn apply_wrap(uv: vec2<f32>, packed: u32) -> vec2<f32> {
    let mode_u = (packed >> 16u) & 0x3u;
    let mode_v = (packed >> 18u) & 0x3u;
    return vec2<f32>(apply_wrap_axis(uv.x, mode_u), apply_wrap_axis(uv.y, mode_v));
}

#ifdef BINDLESS

@group(2) @binding(0)
var material_textures: binding_array<texture_2d<f32>>;

@group(2) @binding(1)
var material_array_sampler: sampler;

fn sample_srgb_layer(packed: u32, uv: vec2<f32>) -> vec4<f32> {
    if packed == NO_LAYER {
        return vec4<f32>(0.0);
    }
    let index = packed & 0xFFFFu;
    let wrapped = apply_wrap(uv, packed);
    return textureSampleLevel(material_textures[index], material_array_sampler, wrapped, 0.0);
}

fn sample_linear_layer(packed: u32, uv: vec2<f32>) -> vec4<f32> {
    if packed == NO_LAYER {
        return vec4<f32>(0.0);
    }
    let index = packed & 0xFFFFu;
    let wrapped = apply_wrap(uv, packed);
    return textureSampleLevel(material_textures[index], material_array_sampler, wrapped, 0.0);
}

fn sample_srgb_index(layer: u32, uv: vec2<f32>) -> vec4<f32> {
    return textureSampleLevel(material_textures[layer], material_array_sampler, uv, 0.0);
}

#else

@group(2) @binding(0)
var material_srgb_array: texture_2d_array<f32>;

@group(2) @binding(1)
var material_linear_array: texture_2d_array<f32>;

@group(2) @binding(2)
var material_array_sampler: sampler;

fn sample_srgb_layer(packed: u32, uv: vec2<f32>) -> vec4<f32> {
    let layer = packed & 0xFFFFu;
    let wrapped = apply_wrap(uv, packed);
    return textureSampleLevel(material_srgb_array, material_array_sampler, wrapped, layer, 0.0);
}

fn sample_linear_layer(packed: u32, uv: vec2<f32>) -> vec4<f32> {
    let layer = packed & 0xFFFFu;
    let wrapped = apply_wrap(uv, packed);
    return textureSampleLevel(material_linear_array, material_array_sampler, wrapped, layer, 0.0);
}

fn sample_srgb_index(layer: u32, uv: vec2<f32>) -> vec4<f32> {
    return textureSampleLevel(material_srgb_array, material_array_sampler, uv, layer, 0.0);
}

#endif
