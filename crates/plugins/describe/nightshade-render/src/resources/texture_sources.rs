use crate::data::texture_data::{SamplerSettings, TextureUsage};
use std::collections::HashMap;

#[derive(Default)]
pub struct TextureSources(pub HashMap<String, TextureSourceBytes>);

impl std::ops::Deref for TextureSources {
    type Target = HashMap<String, TextureSourceBytes>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for TextureSources {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone)]
pub struct TextureSourceBytes {
    pub data: TextureSourceData,
    pub usage: TextureUsage,
    pub sampler: SamplerSettings,
}

#[derive(Debug, Clone)]
pub enum TextureSourceData {
    Rgba {
        rgba: Vec<u8>,
        width: u32,
        height: u32,
    },
    Png(Vec<u8>),
}
