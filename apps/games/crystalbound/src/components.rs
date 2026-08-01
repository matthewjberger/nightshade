use nalgebra_glm::Vec2;

#[derive(Default, Clone, Copy, Debug)]
pub struct Speed(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Facing(pub Vec2);

#[derive(Default, Clone, Copy, Debug)]
pub struct SwingRest(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct TouchRest(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Health(pub i32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Hurt(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Home(pub Vec2);

#[derive(Default, Clone, Copy, Debug)]
pub struct Life(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Spent(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Used(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Foe(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct Slot(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Lunge(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Phase(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Rise(pub f32);
