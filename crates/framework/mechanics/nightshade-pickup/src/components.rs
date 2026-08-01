use nalgebra_glm::Vec3;

#[derive(Default, Clone, Copy)]
pub struct Pickup(pub u32);

#[derive(Default, Clone, Copy)]
pub struct PickupCount(pub u32);

#[derive(Default, Clone, Copy)]
pub struct Collector(pub f32);

#[derive(Default, Clone, Copy)]
pub struct Taken(pub bool);

#[derive(Default, Clone, Copy)]
pub struct Middle(pub Vec3);
