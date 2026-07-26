use crate::render::texture_data::{SamplerSettings, TextureUsage};
use std::collections::HashMap;

/// CPU-side source bytes for imported textures, keyed by texture name.
/// Populated when textures are imported (e.g. from glTF) so that scene
/// saves can embed the original data instead of relying on transient
/// `texture_cache` GPU state. Not used during rendering.
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
    /// Color space the renderer should treat the data as.
    /// Critical for normal maps and metallic-roughness textures (Linear),
    /// distinct from color/albedo textures (Color/sRGB).
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
