use crate::components::{Battery, Lit};
use crate::resources::FlashlightSettings;

pub fn switch(lit: &mut Lit, battery: &Battery) {
    if battery.0 <= 0.0 {
        lit.0 = false;
        return;
    }
    lit.0 = !lit.0;
}

pub fn douse(lit: &mut Lit) {
    lit.0 = false;
}

pub fn recharge(battery: &mut Battery, settings: &FlashlightSettings, amount: f32) {
    battery.0 = (battery.0 + amount).clamp(0.0, settings.full);
}

pub fn empty(battery: &mut Battery) {
    battery.0 = 0.0;
}
