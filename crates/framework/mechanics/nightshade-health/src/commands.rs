use crate::components::PendingDamage;

pub fn hurt(pending: &mut PendingDamage, amount: f32) {
    pending.0 += amount.max(0.0);
}

pub fn heal(health: &mut f32, ceiling: f32, amount: f32) {
    *health = (*health + amount).clamp(0.0, ceiling);
}
