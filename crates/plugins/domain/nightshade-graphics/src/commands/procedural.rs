pub fn load_procedural_textures(load: &mut crate::resources::loading::TextureLoad<'_>) {
    let default_sampler = nightshade_render::data::texture_data::SamplerSettings::DEFAULT;
    let color = nightshade_render::data::texture_data::TextureUsage::Color;

    let checkerboard = generate_checkerboard_texture();
    crate::commands::loading::queue_decoded_texture(
        load,
        "checkerboard".to_string(),
        checkerboard.0,
        checkerboard.1,
        checkerboard.2,
        color,
        default_sampler,
    );

    let gradient = generate_gradient_texture();
    crate::commands::loading::queue_decoded_texture(
        load,
        "gradient".to_string(),
        gradient.0,
        gradient.1,
        gradient.2,
        color,
        default_sampler,
    );

    let uv_test = generate_uv_test_texture();
    crate::commands::loading::queue_decoded_texture(
        load,
        "uv_test".to_string(),
        uv_test.0,
        uv_test.1,
        uv_test.2,
        color,
        default_sampler,
    );

    for name in ["checkerboard", "gradient", "uv_test"] {
        nightshade_render::commands::texture_cache::texture_cache_protect(
            load.cache,
            name.to_string(),
        );
    }
}

fn generate_checkerboard_texture() -> (Vec<u8>, u32, u32) {
    let width = 256;
    let height = 256;
    let checker_size = 32;

    let mut pixels = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let checker_x = (x / checker_size) % 2;
            let checker_y = (y / checker_size) % 2;
            let is_white = (checker_x + checker_y) % 2 == 0;

            if is_white {
                pixels.extend_from_slice(&[255, 255, 255, 255]);
            } else {
                pixels.extend_from_slice(&[64, 64, 64, 255]);
            }
        }
    }

    (pixels, width, height)
}

fn generate_gradient_texture() -> (Vec<u8>, u32, u32) {
    let width = 256;
    let height = 256;

    let mut pixels = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let r = (x * 255 / width) as u8;
            let g = (y * 255 / height) as u8;
            let b = 128u8;
            pixels.extend_from_slice(&[r, g, b, 255]);
        }
    }

    (pixels, width, height)
}

fn generate_uv_test_texture() -> (Vec<u8>, u32, u32) {
    let width = 256;
    let height = 256;

    let mut pixels = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let u = x as f32 / width as f32;
            let v = y as f32 / height as f32;

            let r = (u * 255.0) as u8;
            let g = (v * 255.0) as u8;
            let b = ((1.0 - u) * (1.0 - v) * 255.0) as u8;

            pixels.extend_from_slice(&[r, g, b, 255]);
        }
    }

    (pixels, width, height)
}
