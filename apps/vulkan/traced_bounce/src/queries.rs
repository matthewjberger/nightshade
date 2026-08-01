use nightshade_render::data::config::ProbeVolume;

use crate::theme;

pub fn probe_volume() -> ProbeVolume {
    ProbeVolume {
        counts: theme::PROBE_COUNTS,
        origin: theme::PROBE_ORIGIN,
        spacing: theme::PROBE_SPACING,
        intensity: theme::PROBE_INTENSITY,
        hysteresis: theme::PROBE_HYSTERESIS,
        reach: theme::PROBE_REACH,
        bounce: theme::PROBE_BOUNCE,
    }
}
