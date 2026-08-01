use nightshade::prelude::*;

#[derive(Clone, Copy, Default)]
pub struct Burst {
    pub age: f32,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Border {
    #[default]
    Normal,
    Ready,
    Guard,
    Shield,
    Cursor,
    Target,
    Selected,
    Guide,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Instance(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Generation(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct AttackText(pub Option<Entity>);

#[derive(Default, Clone, Copy, Debug)]
pub struct HealthText(pub Option<Entity>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Spot(pub Vec2);

#[derive(Default, Clone, Copy, Debug)]
pub struct Settled(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Delay(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct ShownAttack(pub i32);

#[derive(Default, Clone, Copy, Debug)]
pub struct ShownHealth(pub i32);

#[derive(Default, Clone, Copy, Debug)]
pub struct WrittenAttack(pub i32);

#[derive(Default, Clone, Copy, Debug)]
pub struct WrittenHealth(pub i32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Dimmed(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Previewed(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct ShieldBadge(pub Option<Entity>);

#[derive(Default, Clone, Copy, Debug)]
pub struct DimOverlay(pub Option<Entity>);

#[derive(Default, Clone, Copy, Debug)]
pub struct ShownShield(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Base(pub Vec2);

#[derive(Default, Clone, Copy, Debug)]
pub struct Rise(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Age(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Card(pub Option<u32>);

#[derive(Clone, Copy, Default)]
pub struct HeroFrame {
    pub border: Border,
}

#[derive(Clone, Copy, Default)]
pub struct BarFill {
    pub percent: f32,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Tint(pub Vec4);

#[derive(Default, Clone, Copy, Debug)]
pub struct Width(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct OfferIndex(pub usize);

#[derive(Default, Clone, Debug)]
pub struct Words(pub Option<String>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Seen(pub Option<bool>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Locked(pub Option<bool>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Tone(pub Option<[u32; 4]>);

#[derive(Default, Clone, Copy, Debug)]
pub struct Tall(pub Option<bool>);
