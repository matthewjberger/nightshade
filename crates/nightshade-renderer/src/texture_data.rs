//! Texture and sampler vocabulary shared between the caller and the
//! texture cache: usage, wrap, and filter settings. The wgpu conversions
//! live with the cache so this module stays free of GPU types.

use serde::{Deserialize, Serialize};

/// How a texture's pixels are interpreted, driving the chosen GPU format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TextureUsage {
    /// sRGB color data, decoded to linear on sample.
    Color,
    /// Linear data such as normals or masks, sampled as stored.
    Linear,
}

/// How sampling wraps coordinates outside the `[0, 1]` range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SamplerWrap {
    /// Tile the texture.
    Repeat,
    /// Tile the texture, mirroring every other repeat.
    MirroredRepeat,
    /// Clamp to the edge texel.
    ClampToEdge,
}

/// The interpolation applied when sampling between texels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SamplerFilter {
    /// Nearest-texel sampling.
    Nearest,
    /// Linear interpolation.
    Linear,
}

/// The wrap and filter configuration for one sampler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SamplerSettings {
    /// Wrap mode along the U axis.
    pub wrap_u: SamplerWrap,
    /// Wrap mode along the V axis.
    pub wrap_v: SamplerWrap,
    /// Filter used when magnifying.
    pub mag_filter: SamplerFilter,
    /// Filter used when minifying.
    pub min_filter: SamplerFilter,
    /// Filter used between mip levels.
    pub mipmap_filter: SamplerFilter,
}

impl SamplerSettings {
    /// Repeat wrapping with linear filtering on every axis.
    pub const DEFAULT: Self = Self {
        wrap_u: SamplerWrap::Repeat,
        wrap_v: SamplerWrap::Repeat,
        mag_filter: SamplerFilter::Linear,
        min_filter: SamplerFilter::Linear,
        mipmap_filter: SamplerFilter::Linear,
    };

    /// Returns a five-character key identifying this configuration, for
    /// deduplicating cached samplers.
    pub fn signature(&self) -> String {
        let wrap_char = |w: SamplerWrap| match w {
            SamplerWrap::Repeat => 'r',
            SamplerWrap::MirroredRepeat => 'm',
            SamplerWrap::ClampToEdge => 'c',
        };
        let filter_char = |f: SamplerFilter| match f {
            SamplerFilter::Nearest => 'n',
            SamplerFilter::Linear => 'l',
        };
        format!(
            "{}{}{}{}{}",
            wrap_char(self.wrap_u),
            wrap_char(self.wrap_v),
            filter_char(self.mag_filter),
            filter_char(self.min_filter),
            filter_char(self.mipmap_filter),
        )
    }
}

impl Default for SamplerSettings {
    fn default() -> Self {
        Self::DEFAULT
    }
}
