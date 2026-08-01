#define_import_path nightshade_wgpu::material_sampling

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

fn sampler_code(packed: u32) -> u32 {
    let mode_u = (packed >> 16u) & 0x3u;
    let mode_v = (packed >> 18u) & 0x3u;
    let nearest = (packed >> 20u) & 0x1u;
    var mode = 2u;
    if mode_u == mode_v {
        mode = mode_u;
    }
    return nearest * 3u + mode;
}

fn wrapped_uv(uv: vec2<f32>, packed: u32) -> vec2<f32> {
    let mode_u = (packed >> 16u) & 0x3u;
    let mode_v = (packed >> 18u) & 0x3u;
    if mode_u == mode_v {
        return uv;
    }
    return apply_wrap(uv, packed);
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

@group(2) @binding(4)
var material_sampler_repeat_nearest: sampler;

@group(2) @binding(5)
var material_sampler_mirror_nearest: sampler;

@group(2) @binding(6)
var material_sampler_clamp_nearest: sampler;

fn sample_texture_grad(index: u32, code: u32, uv: vec2<f32>, grad: UvGrad) -> vec4<f32> {
    switch code {
        case 0u: {
            return textureSampleGrad(material_textures[index], material_sampler_repeat, uv, grad.ddx, grad.ddy);
        }
        case 1u: {
            return textureSampleGrad(material_textures[index], material_sampler_mirror, uv, grad.ddx, grad.ddy);
        }
        case 2u: {
            return textureSampleGrad(material_textures[index], material_sampler_clamp, uv, grad.ddx, grad.ddy);
        }
        case 3u: {
            return textureSampleGrad(material_textures[index], material_sampler_repeat_nearest, uv, grad.ddx, grad.ddy);
        }
        case 4u: {
            return textureSampleGrad(material_textures[index], material_sampler_mirror_nearest, uv, grad.ddx, grad.ddy);
        }
        default: {
            return textureSampleGrad(material_textures[index], material_sampler_clamp_nearest, uv, grad.ddx, grad.ddy);
        }
    }
}

fn sample_srgb_layer_level(packed: u32, uv: vec2<f32>) -> vec4<f32> {
    if packed == NO_LAYER {
        return vec4<f32>(0.0);
    }
    let index = packed & 0xFFFFu;
    let wrapped = apply_wrap(uv, packed);
    if ((packed >> 20u) & 0x1u) != 0u {
        return textureSampleLevel(material_textures[index], material_sampler_clamp_nearest, wrapped, 0.0);
    }
    return textureSampleLevel(material_textures[index], material_sampler_clamp, wrapped, 0.0);
}

fn sample_srgb_layer(packed: u32, uv: vec2<f32>, grad: UvGrad) -> vec4<f32> {
    if packed == NO_LAYER {
        return vec4<f32>(0.0);
    }
    let index = packed & 0xFFFFu;
    return sample_texture_grad(index, sampler_code(packed), wrapped_uv(uv, packed), grad);
}

fn sample_linear_layer(packed: u32, uv: vec2<f32>, grad: UvGrad) -> vec4<f32> {
    if packed == NO_LAYER {
        return vec4<f32>(0.0);
    }
    let index = packed & 0xFFFFu;
    return sample_texture_grad(index, sampler_code(packed), wrapped_uv(uv, packed), grad);
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

@group(2) @binding(5)
var material_sampler_repeat_nearest: sampler;

@group(2) @binding(6)
var material_sampler_mirror_nearest: sampler;

@group(2) @binding(7)
var material_sampler_clamp_nearest: sampler;

fn sample_array_grad(array: texture_2d_array<f32>, layer: u32, code: u32, uv: vec2<f32>, grad: UvGrad) -> vec4<f32> {
    switch code {
        case 0u: {
            return textureSampleGrad(array, material_sampler_repeat, uv, layer, grad.ddx, grad.ddy);
        }
        case 1u: {
            return textureSampleGrad(array, material_sampler_mirror, uv, layer, grad.ddx, grad.ddy);
        }
        case 2u: {
            return textureSampleGrad(array, material_sampler_clamp, uv, layer, grad.ddx, grad.ddy);
        }
        case 3u: {
            return textureSampleGrad(array, material_sampler_repeat_nearest, uv, layer, grad.ddx, grad.ddy);
        }
        case 4u: {
            return textureSampleGrad(array, material_sampler_mirror_nearest, uv, layer, grad.ddx, grad.ddy);
        }
        default: {
            return textureSampleGrad(array, material_sampler_clamp_nearest, uv, layer, grad.ddx, grad.ddy);
        }
    }
}

fn sample_srgb_layer_level(packed: u32, uv: vec2<f32>) -> vec4<f32> {
    if packed == NO_LAYER {
        return vec4<f32>(0.0);
    }
    let layer = packed & 0xFFFFu;
    let wrapped = apply_wrap(uv, packed);
    if ((packed >> 20u) & 0x1u) != 0u {
        return textureSampleLevel(material_srgb_array, material_sampler_clamp_nearest, wrapped, layer, 0.0);
    }
    return textureSampleLevel(material_srgb_array, material_sampler_clamp, wrapped, layer, 0.0);
}

fn sample_srgb_layer(packed: u32, uv: vec2<f32>, grad: UvGrad) -> vec4<f32> {
    if packed == NO_LAYER {
        return vec4<f32>(0.0);
    }
    let layer = packed & 0xFFFFu;
    return sample_array_grad(material_srgb_array, layer, sampler_code(packed), wrapped_uv(uv, packed), grad);
}

fn sample_linear_layer(packed: u32, uv: vec2<f32>, grad: UvGrad) -> vec4<f32> {
    if packed == NO_LAYER {
        return vec4<f32>(0.0);
    }
    let layer = packed & 0xFFFFu;
    return sample_array_grad(material_linear_array, layer, sampler_code(packed), wrapped_uv(uv, packed), grad);
}

fn sample_srgb_index(layer: u32, uv: vec2<f32>) -> vec4<f32> {
    return textureSampleLevel(material_srgb_array, material_sampler_clamp, uv, layer, 0.0);
}

#endif

fn atlas_origin(index: f32, cells: vec2<f32>) -> vec2<f32> {
    let whole = floor(index + 0.5);
    let row = floor((whole + 0.5) / cells.x);
    let column = whole - row * cells.x;
    return vec2<f32>(column, row) / cells;
}

fn sample_atlas_srgb(
    packed: u32,
    uv: vec2<f32>,
    index: f32,
    cells: vec2<f32>,
    grad: UvGrad,
) -> vec4<f32> {
    if cells.x <= 0.0 {
        return sample_srgb_layer(packed, uv, grad);
    }
    let scale = vec2<f32>(1.0, 1.0) / cells;
    var folded: UvGrad;
    folded.ddx = grad.ddx * scale;
    folded.ddy = grad.ddy * scale;
    return sample_srgb_layer(packed, atlas_origin(index, cells) + fract(uv) * scale, folded);
}

fn sample_atlas_srgb_level(
    packed: u32,
    uv: vec2<f32>,
    index: f32,
    cells: vec2<f32>,
) -> vec4<f32> {
    if cells.x <= 0.0 {
        return sample_srgb_layer_level(packed, uv);
    }
    return sample_srgb_layer_level(packed, atlas_origin(index, cells) + fract(uv) / cells);
}
