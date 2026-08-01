use crate::components::{Cooldown, Recharge};

pub fn ready(cooldown: &Cooldown) -> bool {
    cooldown.0 <= 0.0
}

pub fn share(cooldown: &Cooldown, recharge: &Recharge) -> f32 {
    if recharge.0 <= 0.0 {
        return 0.0;
    }
    (cooldown.0 / recharge.0).clamp(0.0, 1.0)
}
