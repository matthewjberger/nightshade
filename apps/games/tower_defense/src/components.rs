use nightshade::prelude::*;

#[derive(Debug, Clone, Copy, Default)]
pub struct Position(pub Vec3);

#[derive(Debug, Clone, Copy, Default)]
pub struct Speed(pub f32);

#[derive(Debug, Clone, Copy, Default)]
pub struct Target(pub Option<Entity>);

#[derive(Debug, Clone, Copy, Default)]
pub struct Cooldown(pub f32);

#[derive(Debug, Clone, Copy, Default)]
pub struct Age(pub f32);

#[derive(Debug, Clone, Copy, Default)]
pub struct Lifetime(pub f32);

#[derive(Debug, Clone, Copy, Default)]
pub struct FireAnimation(pub f32);

#[derive(Debug, Clone, Copy, Default)]
pub struct TrackingTime(pub f32);

#[derive(Debug, Clone, Copy, Default)]
pub struct TargetLine(pub Option<Entity>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GridCoord(pub i32, pub i32);

#[derive(Debug, Clone, Copy, Default)]
pub struct Occupied(pub bool);

#[derive(Debug, Clone, Default)]
pub struct IndicatorLines(pub Vec<Entity>);

#[derive(Debug, Clone, Copy, Default)]
pub struct IndicatorVisible(pub bool);

#[derive(Debug, Clone, Copy, Default)]
pub struct Health(pub f32);

#[derive(Debug, Clone, Copy, Default)]
pub struct ShieldHealth(pub f32);

#[derive(Debug, Clone, Copy, Default)]
pub struct PathIndex(pub usize);

#[derive(Debug, Clone, Copy, Default)]
pub struct PathProgress(pub f32);

#[derive(Debug, Clone, Copy, Default)]
pub struct Bounty(pub u32);

#[derive(Debug, Clone, Copy, Default)]
pub struct SlowDuration(pub f32);

#[derive(Debug, Clone, Copy, Default)]
pub struct PoisonDuration(pub f32);

#[derive(Debug, Clone, Copy, Default)]
pub struct PoisonDamage(pub f32);

#[derive(Debug, Clone, Copy, Default)]
pub struct Damage(pub f32);

#[derive(Debug, Clone, Copy, Default)]
pub struct StartPosition(pub Vec3);

#[derive(Debug, Clone, Copy, Default)]
pub struct ArcHeight(pub f32);

#[derive(Debug, Clone, Copy, Default)]
pub struct FlightProgress(pub f32);
