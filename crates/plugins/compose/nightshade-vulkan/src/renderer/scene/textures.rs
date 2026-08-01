use crate::renderer::context::Context;
use crate::renderer::image_table::{
    ImageTable, ImageUpload, create_image_table, destroy_image_table, scene_texture_capacity,
    upload_image,
};
use ash::vk;
use nightshade_render::data::decoded_image::DecodedImage;
use nightshade_render::data::texture_data::{SamplerSettings, TextureUsage};

pub struct SceneTextures {
    pub table: ImageTable,
    samplers: Vec<SamplerSettings>,
}

pub fn create_textures(context: &Context) -> Result<SceneTextures, Box<dyn std::error::Error>> {
    let capacity = scene_texture_capacity(context);
    Ok(SceneTextures {
        table: create_image_table(
            context,
            capacity,
            vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::COMPUTE,
        )?,
        samplers: vec![SamplerSettings::DEFAULT; capacity as usize],
    })
}

pub fn destroy_textures(context: &Context, textures: &mut SceneTextures) {
    destroy_image_table(context, &mut textures.table);
    textures.samplers.clear();
}

pub fn texture_sampler(textures: &SceneTextures, index: u32) -> SamplerSettings {
    textures
        .samplers
        .get(index as usize)
        .copied()
        .unwrap_or(SamplerSettings::DEFAULT)
}

pub fn upload_texture(
    textures: &mut SceneTextures,
    context: &Context,
    index: u32,
    image: &DecodedImage,
    usage: TextureUsage,
    sampler: SamplerSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    upload_image(
        &mut textures.table,
        context,
        index,
        ImageUpload {
            width: image.width,
            height: image.height,
            pixels: &image.rgba,
            srgb: matches!(usage, TextureUsage::Color),
            mips: true,
            format: image.format,
            levels: &image.mips,
        },
    )?;
    if let Some(slot) = textures.samplers.get_mut(index as usize) {
        *slot = sampler;
    }
    Ok(())
}
