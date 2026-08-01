use crate::components::{Battery, Lit};
use crate::resources::FlashlightSettings;

pub fn remaining(battery: &Battery, settings: &FlashlightSettings) -> f32 {
    if settings.full <= 0.0 {
        return 0.0;
    }
    (battery.0 / settings.full).clamp(0.0, 1.0)
}

pub fn flat(battery: &Battery) -> bool {
    battery.0 <= 0.0
}

pub fn low(battery: &Battery, settings: &FlashlightSettings) -> bool {
    battery.0 > 0.0 && battery.0 <= settings.low
}

pub fn shining(lit: &Lit) -> bool {
    lit.0
}

pub fn brightness(battery: &Battery, lit: &Lit, settings: &FlashlightSettings) -> f32 {
    if !lit.0 || battery.0 <= 0.0 {
        return 0.0;
    }
    let held = if settings.low <= 0.0 {
        1.0
    } else {
        (battery.0 / settings.low).clamp(0.0, 1.0)
    };
    settings.brownout + (1.0 - settings.brownout) * held
}
