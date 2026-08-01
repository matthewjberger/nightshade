use nightshade::prelude::Entity;

#[derive(Default)]
pub struct Jumper {
    pub walker: Option<Entity>,
    pub rise: f32,
    pub lean: f32,
    pub leaps: u32,
}
