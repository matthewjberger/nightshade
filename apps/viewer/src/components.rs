use nightshade_graphics::prelude::Light;

#[derive(Clone, Copy, Debug, Default)]
pub struct Deleted(pub bool);

#[derive(Clone, Debug, Default)]
pub struct StowedLight(pub Light);
