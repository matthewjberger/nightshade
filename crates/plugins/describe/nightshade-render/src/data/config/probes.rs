use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProbeVolume {
    pub counts: [u32; 3],
    pub origin: [f32; 3],
    pub spacing: [f32; 3],
    pub intensity: f32,
    pub hysteresis: f32,
    pub reach: f32,
    pub bounce: f32,
}

impl Default for ProbeVolume {
    fn default() -> Self {
        Self {
            counts: [8, 4, 8],
            origin: [-4.0, 0.25, -4.0],
            spacing: [1.0, 1.0, 1.0],
            intensity: 1.0,
            hysteresis: 0.94,
            reach: 0.0,
            bounce: 0.9,
        }
    }
}

pub fn probe_count(volume: &ProbeVolume) -> u32 {
    volume.counts[0].max(1) * volume.counts[1].max(1) * volume.counts[2].max(1)
}

pub fn effective_reach(volume: &ProbeVolume) -> f32 {
    if volume.reach > 0.0 {
        return volume.reach;
    }
    let widest = volume.spacing[0]
        .max(volume.spacing[1])
        .max(volume.spacing[2]);
    widest.max(0.1) * 10.0
}

pub fn settled_counts(volume: &ProbeVolume) -> [u32; 3] {
    [
        volume.counts[0].max(1),
        volume.counts[1].max(1),
        volume.counts[2].max(1),
    ]
}
