use crate::components::{Armor, Health, MaxHealth, Shield};

pub fn after_armor(amount: f32, armor: &Armor) -> f32 {
    (amount - armor.0).max(0.0)
}

pub fn share(health: &Health, ceiling: &MaxHealth) -> f32 {
    if ceiling.0 <= 0.0 {
        return 0.0;
    }
    (health.0 / ceiling.0).clamp(0.0, 1.0)
}

pub fn shielded(shield: &Shield) -> bool {
    shield.0 > 0.0
}

pub fn fallen(health: &Health) -> bool {
    health.0 <= 0.0
}
