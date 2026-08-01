#[derive(Default, Clone, Copy, Debug)]
pub struct Health(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Facing(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Slot(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Speed(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Kind(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Stride(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Pose(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Book(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Strike(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Step(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Combo(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Stun(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Rest(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Lift(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Rise(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Fling(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Piece(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Holding(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Ammo(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Weapon(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Flying(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Flash(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Solid(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Wish(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Leap(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Tap(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Toward(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Dash(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Chief(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Breaks(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Heals(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Tint(pub [f32; 4]);
