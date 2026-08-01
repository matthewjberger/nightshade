use nalgebra_glm::Vec2;

#[derive(Default, Clone, Copy, Debug)]
pub struct Beast(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Roam(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Swing(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Angry(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Flinch(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Goal(pub Vec2);

#[derive(Default, Clone, Copy, Debug)]
pub struct Shove(pub Vec2);

#[derive(Default, Clone, Copy, Debug)]
pub struct Lair(pub Vec2);
