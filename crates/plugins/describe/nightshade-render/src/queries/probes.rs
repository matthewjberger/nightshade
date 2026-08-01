use crate::data::config::EffectiveShading;

pub fn probe_origin(shading: &EffectiveShading) -> [f32; 4] {
    match &shading.probe_volume {
        Some(volume) => [
            volume.origin[0],
            volume.origin[1],
            volume.origin[2],
            volume.intensity.max(0.0),
        ],
        None => [0.0; 4],
    }
}

pub fn probe_spacing(shading: &EffectiveShading) -> [f32; 4] {
    match &shading.probe_volume {
        Some(volume) => [
            volume.spacing[0].max(1.0e-3),
            volume.spacing[1].max(1.0e-3),
            volume.spacing[2].max(1.0e-3),
            crate::data::config::effective_reach(volume),
        ],
        None => [1.0, 1.0, 1.0, 1.0],
    }
}

pub fn probe_counts(shading: &EffectiveShading, caps: [u32; 3]) -> [u32; 4] {
    let Some(volume) = &shading.probe_volume else {
        return [0; 4];
    };
    let counts = crate::data::config::settled_counts(volume);
    [
        counts[0].min(caps[0]),
        counts[1].min(caps[1]),
        counts[2].min(caps[2]),
        1,
    ]
}

pub fn probe_settings(shading: &EffectiveShading) -> [f32; 4] {
    match &shading.probe_volume {
        Some(volume) => [
            volume.hysteresis.clamp(0.0, 0.99),
            volume.bounce.clamp(0.0, 1.0),
            0.0,
            0.0,
        ],
        None => [0.0; 4],
    }
}

pub fn probe_rotation(frame_number: u64) -> [f32; 4] {
    let step = |offset: u64, by: f32| ((frame_number.wrapping_add(offset) as f32) * by).fract();
    let first = step(0, 0.754_877_7);
    let second = step(1, 0.569_840_3);
    let third = step(2, 0.219_223_2);
    let root = (1.0 - first).max(0.0).sqrt();
    let rest = first.sqrt();
    let around = std::f32::consts::TAU;
    [
        root * (around * second).sin(),
        root * (around * second).cos(),
        rest * (around * third).sin(),
        rest * (around * third).cos(),
    ]
}
