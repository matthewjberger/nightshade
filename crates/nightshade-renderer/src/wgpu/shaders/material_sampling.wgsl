#define_import_path nightshade_renderer::material_sampling

const NO_LAYER: u32 = 0xFFFFFFFFu;

struct UvGrad {
    ddx: vec2<f32>,
    ddy: vec2<f32>,
}

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
var material_sampler_repeat: sampler;

@group(2) @binding(2)
var material_sampler_mirror: sampler;

@group(2) @binding(3)
var material_sampler_clamp: sampler;

fn sample_srgb_layer(packed: u32, uv: vec2<f32>, grad: UvGrad) -> vec4<f32> {
    if packed == NO_LAYER {
        return vec4<f32>(0.0);
    }
    let index = packed & 0xFFFFu;
    let mode_u = (packed >> 16u) & 0x3u;
    let mode_v = (packed >> 18u) & 0x3u;
    if mode_u == mode_v {
        switch mode_u {
            case 0u: {
                return textureSampleGrad(material_textures[index], material_sampler_repeat, uv, grad.ddx, grad.ddy);
            }
            case 1u: {
                return textureSampleGrad(material_textures[index], material_sampler_mirror, uv, grad.ddx, grad.ddy);
            }
            default: {
                return textureSampleGrad(material_textures[index], material_sampler_clamp, uv, grad.ddx, grad.ddy);
            }
        }
    }
    let wrapped = apply_wrap(uv, packed);
    return textureSampleGrad(material_textures[index], material_sampler_clamp, wrapped, grad.ddx, grad.ddy);
}

fn sample_linear_layer(packed: u32, uv: vec2<f32>, grad: UvGrad) -> vec4<f32> {
    if packed == NO_LAYER {
        return vec4<f32>(0.0);
    }
    let index = packed & 0xFFFFu;
    let mode_u = (packed >> 16u) & 0x3u;
    let mode_v = (packed >> 18u) & 0x3u;
    if mode_u == mode_v {
        switch mode_u {
            case 0u: {
                return textureSampleGrad(material_textures[index], material_sampler_repeat, uv, grad.ddx, grad.ddy);
            }
            case 1u: {
                return textureSampleGrad(material_textures[index], material_sampler_mirror, uv, grad.ddx, grad.ddy);
            }
            default: {
                return textureSampleGrad(material_textures[index], material_sampler_clamp, uv, grad.ddx, grad.ddy);
            }
        }
    }
    let wrapped = apply_wrap(uv, packed);
    return textureSampleGrad(material_textures[index], material_sampler_clamp, wrapped, grad.ddx, grad.ddy);
}

fn sample_srgb_index(layer: u32, uv: vec2<f32>) -> vec4<f32> {
    return textureSampleLevel(material_textures[layer], material_sampler_clamp, uv, 0.0);
}

#else

@group(2) @binding(0)
var material_srgb_array: texture_2d_array<f32>;

@group(2) @binding(1)
var material_linear_array: texture_2d_array<f32>;

@group(2) @binding(2)
var material_sampler_repeat: sampler;

@group(2) @binding(3)
var material_sampler_mirror: sampler;

@group(2) @binding(4)
var material_sampler_clamp: sampler;

fn sample_srgb_layer(packed: u32, uv: vec2<f32>, grad: UvGrad) -> vec4<f32> {
    if packed == NO_LAYER {
        return vec4<f32>(0.0);
    }
    let layer = packed & 0xFFFFu;
    let mode_u = (packed >> 16u) & 0x3u;
    let mode_v = (packed >> 18u) & 0x3u;
    if mode_u == mode_v {
        switch mode_u {
            case 0u: {
                return textureSampleGrad(material_srgb_array, material_sampler_repeat, uv, layer, grad.ddx, grad.ddy);
            }
            case 1u: {
                return textureSampleGrad(material_srgb_array, material_sampler_mirror, uv, layer, grad.ddx, grad.ddy);
            }
            default: {
                return textureSampleGrad(material_srgb_array, material_sampler_clamp, uv, layer, grad.ddx, grad.ddy);
            }
        }
    }
    let wrapped = apply_wrap(uv, packed);
    return textureSampleGrad(material_srgb_array, material_sampler_clamp, wrapped, layer, grad.ddx, grad.ddy);
}

fn sample_linear_layer(packed: u32, uv: vec2<f32>, grad: UvGrad) -> vec4<f32> {
    if packed == NO_LAYER {
        return vec4<f32>(0.0);
    }
    let layer = packed & 0xFFFFu;
    let mode_u = (packed >> 16u) & 0x3u;
    let mode_v = (packed >> 18u) & 0x3u;
    if mode_u == mode_v {
        switch mode_u {
            case 0u: {
                return textureSampleGrad(material_linear_array, material_sampler_repeat, uv, layer, grad.ddx, grad.ddy);
            }
            case 1u: {
                return textureSampleGrad(material_linear_array, material_sampler_mirror, uv, layer, grad.ddx, grad.ddy);
            }
            default: {
                return textureSampleGrad(material_linear_array, material_sampler_clamp, uv, layer, grad.ddx, grad.ddy);
            }
        }
    }
    let wrapped = apply_wrap(uv, packed);
    return textureSampleGrad(material_linear_array, material_sampler_clamp, wrapped, layer, grad.ddx, grad.ddy);
}

fn sample_srgb_index(layer: u32, uv: vec2<f32>) -> vec4<f32> {
    return textureSampleLevel(material_srgb_array, material_sampler_clamp, uv, layer, 0.0);
}

#endif
