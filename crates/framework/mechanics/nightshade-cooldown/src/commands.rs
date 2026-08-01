use crate::components::{Cooldown, Recharge};

pub fn start(cooldown: &mut Cooldown, recharge: &Recharge) {
    cooldown.0 = recharge.0;
}

pub fn start_for(cooldown: &mut Cooldown, seconds: f32) {
    cooldown.0 = seconds;
}

pub fn clear(cooldown: &mut Cooldown) {
    cooldown.0 = 0.0;
}
