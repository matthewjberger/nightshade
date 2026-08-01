use crate::contract::PortalPlane;
use nightshade_ecs::Entity;

#[derive(Clone, Copy, Debug, Default)]
pub struct Portal(pub PortalPlane);

#[derive(Clone, Copy, Debug, Default)]
pub struct Link(pub Entity);

#[derive(Clone, Debug, Default)]
pub struct Eyes(pub Vec<Entity>);

#[derive(Clone, Copy, Debug)]
pub struct Layers(pub u32);

impl Default for Layers {
    fn default() -> Self {
        Self(!0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rim(pub [f32; 4]);

impl Default for Rim {
    fn default() -> Self {
        Self([1.0, 1.0, 1.0, 1.0])
    }
}
