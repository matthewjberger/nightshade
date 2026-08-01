#[derive(Default, Clone, Copy)]
pub struct Health(pub f32);

#[derive(Default, Clone, Copy)]
pub struct MaxHealth(pub f32);

#[derive(Default, Clone, Copy)]
pub struct Shield(pub f32);

#[derive(Default, Clone, Copy)]
pub struct Armor(pub f32);

#[derive(Default, Clone, Copy)]
pub struct PendingDamage(pub f32);

#[derive(Default, Clone, Copy)]
pub struct Bounty(pub u32);
