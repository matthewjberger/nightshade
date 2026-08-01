use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TextureUsage {
    Color,
    Linear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SamplerWrap {
    Repeat,
    MirroredRepeat,
    ClampToEdge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SamplerFilter {
    Nearest,
    Linear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SamplerSettings {
    pub wrap_u: SamplerWrap,
    pub wrap_v: SamplerWrap,
    pub mag_filter: SamplerFilter,
    pub min_filter: SamplerFilter,
    pub mipmap_filter: SamplerFilter,
}

impl SamplerSettings {
    pub const DEFAULT: Self = Self {
        wrap_u: SamplerWrap::Repeat,
        wrap_v: SamplerWrap::Repeat,
        mag_filter: SamplerFilter::Linear,
        min_filter: SamplerFilter::Linear,
        mipmap_filter: SamplerFilter::Linear,
    };

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
