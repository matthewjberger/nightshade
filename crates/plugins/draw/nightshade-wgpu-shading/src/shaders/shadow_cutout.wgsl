#define_import_path nightshade_wgpu::shadow_cutout

#import nightshade_wgpu::material_data::{Material}
#import nightshade_wgpu::material_sampling::{sample_srgb_layer_level, sample_atlas_srgb_level, NO_LAYER}
#import nightshade_wgpu::mesh_common::{texture_uv, interleaved_gradient_noise, ALPHA_MODE_MASK, ALPHA_MODE_DITHER}

const NO_SHADOW_MATERIAL: u32 = 0xFFFFFFFFu;

fn hidden_by_cutout(
    material: Material,
    uv: vec2<f32>,
    uv_1: vec2<f32>,
    color: vec4<f32>,
    pixel: vec2<f32>,
    seed: u32,
    time: f32,
) -> bool {
    if material.alpha_mode != ALPHA_MODE_MASK && material.alpha_mode != ALPHA_MODE_DITHER {
        return false;
    }
    var alpha = material.base_color.a * color.a;
    if material.base_layer != NO_LAYER {
        let base_uv = texture_uv(uv, uv_1, material.base_transform, time);
        alpha = alpha * sample_atlas_srgb_level(
            material.base_layer,
            base_uv,
            uv_1.x,
            material.atlas_cells,
        ).a;
    }
    if material.alpha_mode == ALPHA_MODE_MASK {
        return alpha < material.alpha_cutoff;
    }
    let noise = interleaved_gradient_noise(pixel + f32(seed & 0xFFu));
    return alpha < pow(noise, 2.2);
}
