use crate::data::texture_data::{SamplerFilter, SamplerWrap};

pub const NO_TEXTURE_SLOT: u32 = u32::MAX;

pub const SAMPLER_SLOT_COUNT: u32 = 18;

pub const NEAREST_FILTER_BIT: u32 = 1 << 20;

pub fn wrap_code(wrap: SamplerWrap) -> u32 {
    match wrap {
        SamplerWrap::Repeat => 0,
        SamplerWrap::MirroredRepeat => 1,
        SamplerWrap::ClampToEdge => 2,
    }
}

pub fn filter_code(filter: SamplerFilter) -> u32 {
    match filter {
        SamplerFilter::Linear => 0,
        SamplerFilter::Nearest => 1,
    }
}

pub fn pack_texture_slot(
    index: u32,
    wrap_u: SamplerWrap,
    wrap_v: SamplerWrap,
    filter: SamplerFilter,
) -> u32 {
    (index & 0xFFFFu32)
        | (wrap_code(wrap_u) << 16)
        | (wrap_code(wrap_v) << 18)
        | (filter_code(filter) << 20)
}

pub fn sampler_slot(wrap_u: SamplerWrap, wrap_v: SamplerWrap, filter: SamplerFilter) -> u32 {
    filter_code(filter) * 9 + wrap_code(wrap_u) * 3 + wrap_code(wrap_v)
}
