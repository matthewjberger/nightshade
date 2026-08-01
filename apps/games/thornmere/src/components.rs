use crate::data::{ABILITY_COUNT, Shape};
use crate::resources::Hit;
use nightshade::prelude::{Entity, Vec3};

#[derive(Default, Clone, Copy, Debug)]
pub struct Speed(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Health(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct MaxHealth(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Mana(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct MaxMana(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Heading(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Stride(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Cooldowns(pub [f32; ABILITY_COUNT]);

#[derive(Default, Clone, Copy, Debug)]
pub struct Recovery(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Sprint(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Chill(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Target(pub Option<Entity>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Approach(pub Option<Entity>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Leap(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Airborne(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Swing(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Strength(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Kills(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Level(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Experience(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Plate(pub Entity);

#[derive(Default, Clone, Copy, Debug)]
pub struct PlateLevel(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Lift(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Goal(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Home(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Respawn(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Wander(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Aggro(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Leash(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Worth(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Bounty(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Rebirth(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Rank(pub u8);

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Job {
    #[default]
    Resting,
    ToCamp,
    Hunting,
    ToGiver,
    Offline,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Task(pub Job);

#[derive(Default, Clone, Copy, Debug)]
pub struct Rest(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Hunt(pub usize);

#[derive(Default, Clone, Debug)]
pub struct Guild(pub String);

#[derive(Default, Clone, Copy, Debug)]
pub struct Class(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Voice(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Giver(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct GiverMark(pub u8);

#[derive(Default, Clone, Copy, Debug)]
pub struct Pickup(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Regrow(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Fade(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Look(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Fit(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Rig(pub Option<Entity>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Limb(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Pivot(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Reach(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Phase(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Strike(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Flinch(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Dying(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Rising(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Projectile(pub Option<Entity>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Carried(pub Hit);

#[derive(Default, Clone, Copy, Debug)]
pub struct Age(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Span(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Form(pub Shape);

#[derive(Default, Clone, Copy, Debug)]
pub struct Peak(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Spin(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Aura(pub Option<Entity>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Beat(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Post(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Pause(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Drift(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Orbit(pub Vec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Merchant(pub usize);
