use nalgebra_glm::Vec3;
use nightshade::prelude::Entity;
use nightshade_load::LoadHandle;

pub struct Spot {
    pub model: &'static str,
    pub position: Vec3,
    pub yaw: f32,
    pub squeeze: f32,
}

#[derive(Default, Clone, Copy, PartialEq)]
pub struct Wanted {
    pub houses: i64,
    pub seed: i64,
    pub street: i64,
    pub square: i64,
    pub spacing: i64,
    pub trees: i64,
    pub rocks: i64,
    pub reach: f32,
    pub grass: [f32; 4],
}

#[derive(Default)]
pub struct Village {
    pub shown: Option<Wanted>,
    pub spots: Vec<Spot>,
    pub waiting: Vec<(LoadHandle, &'static str)>,
    pub standing: Vec<Entity>,
    pub ground: Entity,
    pub raised: usize,
    pub fresh: bool,
}
